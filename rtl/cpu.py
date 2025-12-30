"""Sampo CPU Core.

A 16-bit RISC processor with:
- 16 general-purpose registers (R0 hardwired to 0)
- 64KB address space
- 16-bit instructions with 32-bit extended forms
- Port-based I/O (256 ports)
- Alternate register bank (R4-R11)
"""

from amaranth import *
from amaranth.lib.enum import Enum

from .alu import ALU, ALUOp, Shifter
from .regfile import RegisterFile
from .decode import Decoder, InstType


class CPUState(Enum, shape=4):
    """CPU state machine states."""
    RESET       = 0
    FETCH       = 1
    FETCH_EXT   = 2   # Fetch extended instruction word
    DECODE      = 3
    EXECUTE     = 4
    MEMORY      = 5
    WRITEBACK   = 6
    HALTED      = 7


class SampoCPU(Elaboratable):
    """Sampo CPU top-level module.

    Memory Interface (active accent accent accent during MEMORY state):
        mem_addr: Memory address output (16-bit)
        mem_rdata: Memory read data input (16-bit)
        mem_wdata: Memory write data output (16-bit)
        mem_we: Memory write enable
        mem_be: Byte enable (2-bit: bit 0 = low byte, bit 1 = high byte)
        mem_valid: Memory request valid
        mem_ready: Memory ready input (active when data valid)

    I/O Interface:
        io_addr: I/O port address (8-bit)
        io_rdata: I/O read data input (8-bit)
        io_wdata: I/O write data output (8-bit)
        io_rd: I/O read strobe
        io_wr: I/O write strobe

    Control:
        reset: Synchronous reset
        halted: CPU is halted output
        irq: Interrupt request input
    """

    def __init__(self, reset_vector=0x0100):
        self.reset_vector = reset_vector

        # Memory interface
        self.mem_addr = Signal(16)
        self.mem_rdata = Signal(16)
        self.mem_wdata = Signal(16)
        self.mem_we = Signal()
        self.mem_be = Signal(2)
        self.mem_valid = Signal()
        self.mem_ready = Signal()

        # I/O interface
        self.io_addr = Signal(8)
        self.io_rdata = Signal(8)
        self.io_wdata = Signal(8)
        self.io_rd = Signal()
        self.io_wr = Signal()

        # Control
        self.halted = Signal()
        self.irq = Signal()

        # Debug outputs
        self.pc = Signal(16)
        self.state = Signal(CPUState)
        self.cycles = Signal(32)

    def elaborate(self, platform):
        m = Module()

        # Instantiate submodules
        m.submodules.alu = alu = ALU()
        m.submodules.shifter = shifter = Shifter()
        m.submodules.regfile = regfile = RegisterFile()
        m.submodules.decoder = decoder = Decoder()

        # CPU state
        state = Signal(CPUState, reset=CPUState.RESET)
        pc = Signal(16, reset=self.reset_vector)
        flags = Signal(8)
        cycles = Signal(32)

        # Instruction registers
        instr = Signal(16)
        instr_ext = Signal(16)  # Extended word for 32-bit instructions

        # Pipeline registers
        alu_result = Signal(16)
        mem_addr_reg = Signal(16)
        mem_data_reg = Signal(16)

        # Interrupt state
        int_enabled = Signal()
        int_pending = Signal()

        # Debug outputs
        m.d.comb += [
            self.pc.eq(pc),
            self.state.eq(state),
            self.cycles.eq(cycles),
            self.halted.eq(state == CPUState.HALTED),
        ]

        # Connect decoder
        m.d.comb += [
            decoder.instr.eq(instr),
            decoder.imm16.eq(instr_ext),
        ]

        # Register file read addresses (from decoder)
        m.d.comb += [
            regfile.rd_addr1.eq(decoder.rs1),
            regfile.rd_addr2.eq(decoder.rs2),
        ]

        # ALU input selection
        rs1_data = Signal(16)
        rs2_data = Signal(16)
        alu_a = Signal(16)
        alu_b = Signal(16)

        m.d.comb += [
            rs1_data.eq(regfile.rd_data1),
            rs2_data.eq(regfile.rd_data2),
        ]

        # ALU A input: usually rs1, but rd for ADDI
        with m.If(decoder.inst_type == InstType.ALU_IMM):
            m.d.comb += alu_a.eq(regfile.rd_data1)
            # For ADDI, we read rd as rs1
            m.d.comb += regfile.rd_addr1.eq(decoder.rd)
        with m.Else():
            m.d.comb += alu_a.eq(rs1_data)

        # ALU B input: rs2, immediate, or extended immediate
        with m.If(decoder.inst_type == InstType.ALU_IMM):
            m.d.comb += alu_b.eq(decoder.imm8)
        with m.Elif(decoder.is_extended):
            m.d.comb += alu_b.eq(instr_ext)
        with m.Else():
            m.d.comb += alu_b.eq(rs2_data)

        m.d.comb += [
            alu.a.eq(alu_a),
            alu.b.eq(alu_b),
            alu.op.eq(decoder.alu_op),
        ]

        # Shifter connections
        m.d.comb += [
            shifter.value.eq(rs1_data),
            shifter.func.eq(decoder.shift_func),
            shifter.carry_in.eq(flags[5]),  # Carry flag
        ]

        # Branch condition evaluation
        branch_taken = Signal()
        flag_n = flags[7]
        flag_z = flags[6]
        flag_c = flags[5]
        flag_v = flags[4]

        with m.Switch(decoder.branch_cond):
            with m.Case(0x0):  # BEQ
                m.d.comb += branch_taken.eq(flag_z)
            with m.Case(0x1):  # BNE
                m.d.comb += branch_taken.eq(~flag_z)
            with m.Case(0x2):  # BLT
                m.d.comb += branch_taken.eq(flag_n != flag_v)
            with m.Case(0x3):  # BGE
                m.d.comb += branch_taken.eq(flag_n == flag_v)
            with m.Case(0x4):  # BLTU
                m.d.comb += branch_taken.eq(~flag_c)
            with m.Case(0x5):  # BGEU
                m.d.comb += branch_taken.eq(flag_c)
            with m.Case(0x6):  # BMI
                m.d.comb += branch_taken.eq(flag_n)
            with m.Case(0x7):  # BPL
                m.d.comb += branch_taken.eq(~flag_n)
            with m.Case(0x8):  # BVS
                m.d.comb += branch_taken.eq(flag_v)
            with m.Case(0x9):  # BVC
                m.d.comb += branch_taken.eq(~flag_v)
            with m.Case(0xA):  # BCS
                m.d.comb += branch_taken.eq(flag_c)
            with m.Case(0xB):  # BCC
                m.d.comb += branch_taken.eq(~flag_c)
            with m.Case(0xC):  # BGT
                m.d.comb += branch_taken.eq(~flag_z & (flag_n == flag_v))
            with m.Case(0xD):  # BLE
                m.d.comb += branch_taken.eq(flag_z | (flag_n != flag_v))
            with m.Case(0xE):  # BHI
                m.d.comb += branch_taken.eq(flag_c & ~flag_z)
            with m.Case(0xF):  # BLS
                m.d.comb += branch_taken.eq(~flag_c | flag_z)
            with m.Default():
                m.d.comb += branch_taken.eq(0)

        # Next PC calculation
        pc_plus_2 = Signal(16)
        pc_plus_4 = Signal(16)
        pc_branch = Signal(16)
        pc_jump = Signal(16)
        next_pc = Signal(16)

        m.d.comb += [
            pc_plus_2.eq(pc + 2),
            pc_plus_4.eq(pc + 4),
            pc_branch.eq(pc + (decoder.offset8 << 1)),
            pc_jump.eq(pc + (decoder.offset12 << 1)),
        ]

        # Default memory interface signals
        m.d.comb += [
            self.mem_addr.eq(0),
            self.mem_wdata.eq(0),
            self.mem_we.eq(0),
            self.mem_be.eq(0b11),
            self.mem_valid.eq(0),
        ]

        # Default I/O interface signals
        m.d.comb += [
            self.io_addr.eq(0),
            self.io_wdata.eq(0),
            self.io_rd.eq(0),
            self.io_wr.eq(0),
        ]

        # Default register file write signals
        m.d.comb += [
            regfile.wr_addr.eq(0),
            regfile.wr_data.eq(0),
            regfile.wr_en.eq(0),
            regfile.exx.eq(0),
        ]

        # State machine
        with m.FSM(reset=CPUState.RESET) as fsm:
            m.d.comb += state.eq(fsm.state)

            with m.State(CPUState.RESET):
                m.d.sync += [
                    pc.eq(self.reset_vector),
                    flags.eq(0),
                    int_enabled.eq(0),
                    cycles.eq(0),
                ]
                m.next = CPUState.FETCH

            with m.State(CPUState.FETCH):
                # Fetch instruction from memory
                m.d.comb += [
                    self.mem_addr.eq(pc),
                    self.mem_valid.eq(1),
                ]
                with m.If(self.mem_ready):
                    m.d.sync += instr.eq(self.mem_rdata)
                    m.next = CPUState.DECODE

            with m.State(CPUState.DECODE):
                # Check if this is an extended instruction
                with m.If(decoder.is_extended):
                    m.next = CPUState.FETCH_EXT
                with m.Else():
                    m.next = CPUState.EXECUTE

            with m.State(CPUState.FETCH_EXT):
                # Fetch second word for extended instructions
                m.d.comb += [
                    self.mem_addr.eq(pc + 2),
                    self.mem_valid.eq(1),
                ]
                with m.If(self.mem_ready):
                    m.d.sync += instr_ext.eq(self.mem_rdata)
                    m.next = CPUState.EXECUTE

            with m.State(CPUState.EXECUTE):
                # Default: advance PC by 2 (or 4 for extended)
                with m.If(decoder.is_extended):
                    m.d.sync += pc.eq(pc_plus_4)
                with m.Else():
                    m.d.sync += pc.eq(pc_plus_2)

                # Handle different instruction types
                with m.Switch(decoder.inst_type):
                    # ALU register-register
                    with m.Case(InstType.ALU_REG):
                        m.d.sync += alu_result.eq(alu.result)
                        m.d.sync += [
                            flags[7].eq(alu.flag_n),
                            flags[6].eq(alu.flag_z),
                            flags[5].eq(alu.flag_c),
                            flags[4].eq(alu.flag_v),
                        ]
                        m.next = CPUState.WRITEBACK

                    # ALU immediate
                    with m.Case(InstType.ALU_IMM):
                        m.d.sync += alu_result.eq(alu.result)
                        m.d.sync += [
                            flags[7].eq(alu.flag_n),
                            flags[6].eq(alu.flag_z),
                            flags[5].eq(alu.flag_c),
                            flags[4].eq(alu.flag_v),
                        ]
                        m.next = CPUState.WRITEBACK

                    # Load
                    with m.Case(InstType.LOAD):
                        m.d.sync += mem_addr_reg.eq(rs1_data)
                        m.next = CPUState.MEMORY

                    # Store
                    with m.Case(InstType.STORE):
                        m.d.sync += [
                            mem_addr_reg.eq(rs1_data),
                            mem_data_reg.eq(regfile.rd_data1),
                        ]
                        # For store, read rd (the data to store)
                        m.d.comb += regfile.rd_addr1.eq(decoder.rd)
                        m.next = CPUState.MEMORY

                    # Branch
                    with m.Case(InstType.BRANCH):
                        with m.If(branch_taken):
                            m.d.sync += pc.eq(pc_branch)
                        m.next = CPUState.FETCH

                    # Jump (relative)
                    with m.Case(InstType.JUMP):
                        m.d.sync += pc.eq(pc_jump)
                        m.next = CPUState.FETCH

                    # Jump register
                    with m.Case(InstType.JUMP_REG):
                        m.d.sync += pc.eq(rs1_data)
                        with m.If(decoder.is_call):
                            # JALR: save return address
                            m.d.sync += alu_result.eq(pc_plus_2)
                            m.next = CPUState.WRITEBACK
                        with m.Else():
                            m.next = CPUState.FETCH

                    # Shift
                    with m.Case(InstType.SHIFT):
                        m.d.sync += [
                            alu_result.eq(shifter.result),
                            flags[5].eq(shifter.carry_out),
                            flags[7].eq(shifter.result[15]),
                            flags[6].eq(shifter.result == 0),
                        ]
                        m.next = CPUState.WRITEBACK

                    # Multiply/Divide
                    with m.Case(InstType.MULDIV):
                        m.d.sync += alu_result.eq(alu.result)
                        m.next = CPUState.WRITEBACK

                    # Misc
                    with m.Case(InstType.MISC):
                        with m.If(decoder.is_exx):
                            m.d.comb += regfile.exx.eq(1)
                            m.next = CPUState.FETCH
                        with m.Elif(decoder.func == 0x2):  # CMP
                            m.d.sync += [
                                flags[7].eq(alu.flag_n),
                                flags[6].eq(alu.flag_z),
                                flags[5].eq(alu.flag_c),
                                flags[4].eq(alu.flag_v),
                            ]
                            m.next = CPUState.FETCH
                        with m.Elif(decoder.func == 0x4):  # MOV
                            m.d.sync += alu_result.eq(rs1_data)
                            m.next = CPUState.WRITEBACK
                        with m.Else():
                            m.next = CPUState.FETCH

                    # I/O
                    with m.Case(InstType.IO):
                        with m.If(decoder.is_io_in):
                            with m.If(decoder.io_port_imm):
                                m.d.comb += self.io_addr.eq(decoder.rs1)
                            with m.Else():
                                m.d.comb += self.io_addr.eq(rs1_data[:8])
                            m.d.comb += self.io_rd.eq(1)
                            m.d.sync += alu_result[:8].eq(self.io_rdata)
                            m.d.sync += alu_result[8:16].eq(0)
                            m.next = CPUState.WRITEBACK
                        with m.Elif(decoder.is_io_out):
                            with m.If(decoder.io_port_imm):
                                m.d.comb += self.io_addr.eq(decoder.rs1)
                            with m.Else():
                                m.d.comb += self.io_addr.eq(regfile.rd_data1[:8])
                            # Read rd for the data to output
                            m.d.comb += regfile.rd_addr1.eq(decoder.rd)
                            m.d.comb += [
                                self.io_wdata.eq(regfile.rd_data1[:8]),
                                self.io_wr.eq(1),
                            ]
                            m.next = CPUState.FETCH

                    # System
                    with m.Case(InstType.SYSTEM):
                        with m.If(decoder.is_halt):
                            m.next = CPUState.HALTED
                        with m.Elif(decoder.is_ei):
                            m.d.sync += int_enabled.eq(1)
                            m.next = CPUState.FETCH
                        with m.Elif(decoder.is_di):
                            m.d.sync += int_enabled.eq(0)
                            m.next = CPUState.FETCH
                        with m.Else():
                            m.next = CPUState.FETCH

                    # Extended instructions
                    with m.Case(InstType.EXTENDED):
                        with m.If(decoder.mem_load | decoder.mem_store):
                            # LWX/SWX: address = rs1 + imm16
                            m.d.sync += mem_addr_reg.eq(rs1_data + instr_ext)
                            with m.If(decoder.mem_store):
                                m.d.comb += regfile.rd_addr1.eq(decoder.rd)
                                m.d.sync += mem_data_reg.eq(regfile.rd_data1)
                            m.next = CPUState.MEMORY
                        with m.Elif(decoder.is_jump):
                            # JX/JALX: absolute address
                            m.d.sync += pc.eq(instr_ext)
                            with m.If(decoder.is_call):
                                m.d.sync += alu_result.eq(pc_plus_4)
                                m.next = CPUState.WRITEBACK
                            with m.Else():
                                m.next = CPUState.FETCH
                        with m.Elif(decoder.func == 0x7):  # LIX
                            m.d.sync += alu_result.eq(instr_ext)
                            m.next = CPUState.WRITEBACK
                        with m.Elif(decoder.is_io_in):  # INX
                            m.d.comb += [
                                self.io_addr.eq(instr_ext[:8]),
                                self.io_rd.eq(1),
                            ]
                            m.d.sync += alu_result[:8].eq(self.io_rdata)
                            m.d.sync += alu_result[8:16].eq(0)
                            m.next = CPUState.WRITEBACK
                        with m.Elif(decoder.is_io_out):  # OUTX
                            m.d.comb += [
                                self.io_addr.eq(instr_ext[:8]),
                                self.io_wdata.eq(rs1_data[:8]),
                                self.io_wr.eq(1),
                            ]
                            m.next = CPUState.FETCH
                        with m.Else():
                            # ALU with extended immediate
                            m.d.sync += alu_result.eq(alu.result)
                            m.d.sync += [
                                flags[7].eq(alu.flag_n),
                                flags[6].eq(alu.flag_z),
                                flags[5].eq(alu.flag_c),
                                flags[4].eq(alu.flag_v),
                            ]
                            m.next = CPUState.WRITEBACK

                    with m.Default():
                        m.next = CPUState.FETCH

                m.d.sync += cycles.eq(cycles + 1)

            with m.State(CPUState.MEMORY):
                m.d.comb += [
                    self.mem_addr.eq(mem_addr_reg),
                    self.mem_valid.eq(1),
                ]

                with m.If(decoder.mem_store):
                    m.d.comb += [
                        self.mem_wdata.eq(mem_data_reg),
                        self.mem_we.eq(1),
                    ]
                    with m.If(decoder.mem_byte):
                        # Byte store: select which byte based on address bit 0
                        with m.If(mem_addr_reg[0]):
                            m.d.comb += self.mem_be.eq(0b10)
                            m.d.comb += self.mem_wdata.eq(mem_data_reg << 8)
                        with m.Else():
                            m.d.comb += self.mem_be.eq(0b01)

                with m.If(self.mem_ready):
                    with m.If(decoder.mem_load):
                        with m.If(decoder.mem_byte):
                            # Byte load
                            with m.If(mem_addr_reg[0]):
                                byte_data = self.mem_rdata[8:16]
                            with m.Else():
                                byte_data = self.mem_rdata[0:8]
                            with m.If(decoder.mem_signed):
                                m.d.sync += alu_result.eq(byte_data.as_signed())
                            with m.Else():
                                m.d.sync += alu_result.eq(byte_data)
                        with m.Else():
                            m.d.sync += alu_result.eq(self.mem_rdata)
                        m.next = CPUState.WRITEBACK
                    with m.Else():
                        m.next = CPUState.FETCH

            with m.State(CPUState.WRITEBACK):
                # Write result to destination register
                with m.If(decoder.reg_write):
                    m.d.comb += [
                        regfile.wr_addr.eq(decoder.rd),
                        regfile.wr_data.eq(alu_result),
                        regfile.wr_en.eq(1),
                    ]
                m.next = CPUState.FETCH

            with m.State(CPUState.HALTED):
                # Stay halted (can be reset externally)
                pass

        return m
