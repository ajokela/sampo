"""Sampo CPU Instruction Decoder.

Decodes 16-bit instructions into control signals for the datapath.
Also handles 32-bit extended instructions (opcode 0xF).
"""

from amaranth import *
from amaranth.lib.enum import Enum

from .alu import ALUOp


class InstType(Enum, shape=4):
    """Instruction type classification."""
    ALU_REG     = 0   # Register-register ALU (ADD, SUB, AND, OR, XOR)
    ALU_IMM     = 1   # Immediate ALU (ADDI)
    LOAD        = 2   # Load from memory
    STORE       = 3   # Store to memory
    BRANCH      = 4   # Conditional branch
    JUMP        = 5   # Unconditional jump
    JUMP_REG    = 6   # Register jump (JR, JALR)
    SHIFT       = 7   # Shift/rotate
    MULDIV      = 8   # Multiply/divide
    MISC        = 9   # Miscellaneous (PUSH, POP, CMP, etc.)
    IO          = 10  # I/O operations
    SYSTEM      = 11  # System operations (NOP, HALT, etc.)
    EXTENDED    = 12  # 32-bit extended instruction
    INVALID     = 15


class Decoder(Elaboratable):
    """Instruction decoder for Sampo CPU.

    Decodes a 16-bit instruction into control signals.

    Inputs:
        instr: 16-bit instruction
        imm16: 16-bit immediate (for extended instructions)
        extended: This is an extended (32-bit) instruction

    Outputs:
        inst_type: Instruction type classification
        rd: Destination register (4-bit)
        rs1: Source register 1 (4-bit)
        rs2: Source register 2 (4-bit)
        imm8: 8-bit immediate (sign-extended to 16)
        imm16_out: 16-bit immediate (for extended)
        alu_op: ALU operation
        shift_func: Shift function code
        branch_cond: Branch condition code
        mem_load: Memory load operation
        mem_store: Memory store operation
        mem_byte: Byte (vs word) memory access
        mem_signed: Signed byte load
        reg_write: Register write enable
        is_jump: Is a jump instruction
        is_branch: Is a branch instruction
        is_extended: Is extended (32-bit) instruction
        is_halt: Is HALT instruction
    """

    def __init__(self):
        # Inputs
        self.instr = Signal(16)
        self.imm16 = Signal(16)

        # Decoded fields
        self.inst_type = Signal(InstType)
        self.rd = Signal(4)
        self.rs1 = Signal(4)
        self.rs2 = Signal(4)
        self.func = Signal(4)
        self.imm8 = Signal(16)  # Sign-extended
        self.imm16_out = Signal(16)
        self.offset8 = Signal(signed=True, shape=16)  # Branch offset
        self.offset12 = Signal(signed=True, shape=16)  # Jump offset

        # ALU control
        self.alu_op = Signal(ALUOp)
        self.shift_func = Signal(4)

        # Branch control
        self.branch_cond = Signal(4)

        # Memory control
        self.mem_load = Signal()
        self.mem_store = Signal()
        self.mem_byte = Signal()
        self.mem_signed = Signal()

        # Register control
        self.reg_write = Signal()

        # Flow control
        self.is_jump = Signal()
        self.is_branch = Signal()
        self.is_call = Signal()  # JAL or JALR
        self.is_ret = Signal()   # JR R1 (return)
        self.is_extended = Signal()

        # System
        self.is_halt = Signal()
        self.is_nop = Signal()
        self.is_exx = Signal()
        self.is_ei = Signal()
        self.is_di = Signal()
        self.is_reti = Signal()

        # I/O
        self.is_io_in = Signal()
        self.is_io_out = Signal()
        self.io_port_imm = Signal()  # Port is immediate (vs register)

        # Stack
        self.is_push = Signal()
        self.is_pop = Signal()

    def elaborate(self, platform):
        m = Module()

        # Extract instruction fields
        opcode = self.instr[12:16]
        rd = self.instr[8:12]
        rs1 = self.instr[4:8]
        rs2 = self.instr[0:4]
        func = self.instr[0:4]
        imm8_raw = self.instr[0:8]
        offset12_raw = self.instr[0:12]

        # Output decoded fields
        m.d.comb += [
            self.rd.eq(rd),
            self.rs1.eq(rs1),
            self.rs2.eq(rs2),
            self.func.eq(func),
            self.imm16_out.eq(self.imm16),
            self.shift_func.eq(func),
            self.branch_cond.eq(rd),
        ]

        # Sign-extend 8-bit immediate
        m.d.comb += self.imm8.eq(imm8_raw.as_signed())

        # Sign-extend 8-bit branch offset
        m.d.comb += self.offset8.eq(imm8_raw.as_signed())

        # Sign-extend 12-bit jump offset
        with m.If(offset12_raw[11]):
            m.d.comb += self.offset12.eq(Cat(offset12_raw, Const(0xF, 4)).as_signed())
        with m.Else():
            m.d.comb += self.offset12.eq(offset12_raw.as_signed())

        # Default control signals
        m.d.comb += [
            self.inst_type.eq(InstType.INVALID),
            self.alu_op.eq(ALUOp.ADD),
            self.mem_load.eq(0),
            self.mem_store.eq(0),
            self.mem_byte.eq(0),
            self.mem_signed.eq(0),
            self.reg_write.eq(0),
            self.is_jump.eq(0),
            self.is_branch.eq(0),
            self.is_call.eq(0),
            self.is_ret.eq(0),
            self.is_extended.eq(0),
            self.is_halt.eq(0),
            self.is_nop.eq(0),
            self.is_exx.eq(0),
            self.is_ei.eq(0),
            self.is_di.eq(0),
            self.is_reti.eq(0),
            self.is_io_in.eq(0),
            self.is_io_out.eq(0),
            self.io_port_imm.eq(0),
            self.is_push.eq(0),
            self.is_pop.eq(0),
        ]

        # Main decode logic
        with m.Switch(opcode):
            # ADD Rd, Rs1, Rs2
            with m.Case(0x0):
                m.d.comb += [
                    self.inst_type.eq(InstType.ALU_REG),
                    self.alu_op.eq(ALUOp.ADD),
                    self.reg_write.eq(1),
                ]

            # SUB Rd, Rs1, Rs2
            with m.Case(0x1):
                m.d.comb += [
                    self.inst_type.eq(InstType.ALU_REG),
                    self.alu_op.eq(ALUOp.SUB),
                    self.reg_write.eq(1),
                ]

            # AND Rd, Rs1, Rs2
            with m.Case(0x2):
                m.d.comb += [
                    self.inst_type.eq(InstType.ALU_REG),
                    self.alu_op.eq(ALUOp.AND),
                    self.reg_write.eq(1),
                ]

            # OR Rd, Rs1, Rs2
            with m.Case(0x3):
                m.d.comb += [
                    self.inst_type.eq(InstType.ALU_REG),
                    self.alu_op.eq(ALUOp.OR),
                    self.reg_write.eq(1),
                ]

            # XOR Rd, Rs1, Rs2
            with m.Case(0x4):
                m.d.comb += [
                    self.inst_type.eq(InstType.ALU_REG),
                    self.alu_op.eq(ALUOp.XOR),
                    self.reg_write.eq(1),
                ]

            # ADDI Rd, imm8
            with m.Case(0x5):
                m.d.comb += [
                    self.inst_type.eq(InstType.ALU_IMM),
                    self.alu_op.eq(ALUOp.ADD),
                    self.reg_write.eq(1),
                ]

            # LOAD operations
            with m.Case(0x6):
                m.d.comb += [
                    self.inst_type.eq(InstType.LOAD),
                    self.mem_load.eq(1),
                    self.reg_write.eq(1),
                ]
                with m.Switch(func):
                    with m.Case(0x1):  # LB
                        m.d.comb += [
                            self.mem_byte.eq(1),
                            self.mem_signed.eq(1),
                        ]
                    with m.Case(0x2):  # LBU
                        m.d.comb += [
                            self.mem_byte.eq(1),
                            self.mem_signed.eq(0),
                        ]
                    with m.Case(0x8):  # LUI
                        m.d.comb += [
                            self.mem_load.eq(0),
                            self.inst_type.eq(InstType.ALU_IMM),
                        ]

            # STORE operations
            with m.Case(0x7):
                m.d.comb += [
                    self.inst_type.eq(InstType.STORE),
                    self.mem_store.eq(1),
                ]
                with m.If(func == 0x1):  # SB
                    m.d.comb += self.mem_byte.eq(1)

            # BRANCH
            with m.Case(0x8):
                m.d.comb += [
                    self.inst_type.eq(InstType.BRANCH),
                    self.is_branch.eq(1),
                ]

            # JUMP
            with m.Case(0x9):
                # Check for JR (9F0R pattern)
                with m.If((self.instr & 0x0F0F) == 0x0F00):
                    m.d.comb += [
                        self.inst_type.eq(InstType.JUMP_REG),
                        self.is_jump.eq(1),
                    ]
                    # JR R1 is a return
                    with m.If(rs1 == 1):
                        m.d.comb += self.is_ret.eq(1)
                # JALR (func == 1 and rd != 0)
                with m.Elif((func == 1) & (rd != 0)):
                    m.d.comb += [
                        self.inst_type.eq(InstType.JUMP_REG),
                        self.is_jump.eq(1),
                        self.is_call.eq(1),
                        self.reg_write.eq(1),
                    ]
                # J offset12
                with m.Else():
                    m.d.comb += [
                        self.inst_type.eq(InstType.JUMP),
                        self.is_jump.eq(1),
                    ]

            # SHIFT
            with m.Case(0xA):
                m.d.comb += [
                    self.inst_type.eq(InstType.SHIFT),
                    self.reg_write.eq(1),
                ]

            # MULDIV
            with m.Case(0xB):
                m.d.comb += [
                    self.inst_type.eq(InstType.MULDIV),
                    self.reg_write.eq(1),
                ]
                with m.Switch(func):
                    with m.Case(0x0):  # MUL
                        m.d.comb += self.alu_op.eq(ALUOp.MUL)
                    with m.Case(0x1):  # MULH
                        m.d.comb += self.alu_op.eq(ALUOp.MULH)
                    with m.Case(0x3):  # DIV
                        m.d.comb += self.alu_op.eq(ALUOp.DIV)
                    with m.Case(0x5):  # REM
                        m.d.comb += self.alu_op.eq(ALUOp.REM)

            # MISC
            with m.Case(0xC):
                m.d.comb += self.inst_type.eq(InstType.MISC)
                with m.Switch(func):
                    with m.Case(0x0):  # PUSH
                        m.d.comb += self.is_push.eq(1)
                    with m.Case(0x1):  # POP
                        m.d.comb += [
                            self.is_pop.eq(1),
                            self.reg_write.eq(1),
                        ]
                    with m.Case(0x2):  # CMP
                        m.d.comb += self.alu_op.eq(ALUOp.SUB)
                    with m.Case(0x3):  # TEST
                        m.d.comb += self.alu_op.eq(ALUOp.AND)
                    with m.Case(0x4):  # MOV
                        m.d.comb += [
                            self.alu_op.eq(ALUOp.PASS_B),
                            self.reg_write.eq(1),
                        ]
                    with m.Case(0xB):  # EXX
                        m.d.comb += self.is_exx.eq(1)
                    with m.Case(0xC):  # GETF
                        m.d.comb += self.reg_write.eq(1)
                    with m.Case(0xD):  # SETF
                        pass

            # I/O
            with m.Case(0xD):
                m.d.comb += self.inst_type.eq(InstType.IO)
                with m.Switch(func):
                    with m.Case(0x0):  # INI
                        m.d.comb += [
                            self.is_io_in.eq(1),
                            self.io_port_imm.eq(1),
                            self.reg_write.eq(1),
                        ]
                    with m.Case(0x1):  # OUTI
                        m.d.comb += [
                            self.is_io_out.eq(1),
                            self.io_port_imm.eq(1),
                        ]
                    with m.Case(0x2):  # IN
                        m.d.comb += [
                            self.is_io_in.eq(1),
                            self.reg_write.eq(1),
                        ]
                    with m.Case(0x3):  # OUT
                        m.d.comb += self.is_io_out.eq(1)

            # SYSTEM
            with m.Case(0xE):
                m.d.comb += self.inst_type.eq(InstType.SYSTEM)
                with m.Switch(rd):
                    with m.Case(0x0):  # NOP
                        m.d.comb += self.is_nop.eq(1)
                    with m.Case(0x1):  # HALT
                        m.d.comb += self.is_halt.eq(1)
                    with m.Case(0x2):  # DI
                        m.d.comb += self.is_di.eq(1)
                    with m.Case(0x3):  # EI
                        m.d.comb += self.is_ei.eq(1)
                    with m.Case(0x4):  # RETI
                        m.d.comb += self.is_reti.eq(1)

            # EXTENDED (32-bit)
            with m.Case(0xF):
                m.d.comb += [
                    self.inst_type.eq(InstType.EXTENDED),
                    self.is_extended.eq(1),
                ]
                with m.Switch(func):
                    with m.Case(0x0):  # ADDIX
                        m.d.comb += [
                            self.alu_op.eq(ALUOp.ADD),
                            self.reg_write.eq(1),
                        ]
                    with m.Case(0x1):  # SUBIX
                        m.d.comb += [
                            self.alu_op.eq(ALUOp.SUB),
                            self.reg_write.eq(1),
                        ]
                    with m.Case(0x2):  # ANDIX
                        m.d.comb += [
                            self.alu_op.eq(ALUOp.AND),
                            self.reg_write.eq(1),
                        ]
                    with m.Case(0x3):  # ORIX
                        m.d.comb += [
                            self.alu_op.eq(ALUOp.OR),
                            self.reg_write.eq(1),
                        ]
                    with m.Case(0x4):  # XORIX
                        m.d.comb += [
                            self.alu_op.eq(ALUOp.XOR),
                            self.reg_write.eq(1),
                        ]
                    with m.Case(0x5):  # LWX
                        m.d.comb += [
                            self.mem_load.eq(1),
                            self.reg_write.eq(1),
                        ]
                    with m.Case(0x6):  # SWX
                        m.d.comb += self.mem_store.eq(1)
                    with m.Case(0x7):  # LIX
                        m.d.comb += [
                            self.alu_op.eq(ALUOp.PASS_B),
                            self.reg_write.eq(1),
                        ]
                    with m.Case(0x8):  # JX
                        m.d.comb += self.is_jump.eq(1)
                    with m.Case(0x9):  # JALX
                        m.d.comb += [
                            self.is_jump.eq(1),
                            self.is_call.eq(1),
                            self.reg_write.eq(1),
                        ]
                    with m.Case(0xA):  # CMPIX
                        m.d.comb += self.alu_op.eq(ALUOp.SUB)
                    with m.Case(0xB):  # INX
                        m.d.comb += [
                            self.is_io_in.eq(1),
                            self.io_port_imm.eq(1),
                            self.reg_write.eq(1),
                        ]
                    with m.Case(0xC):  # OUTX
                        m.d.comb += [
                            self.is_io_out.eq(1),
                            self.io_port_imm.eq(1),
                        ]

        return m
