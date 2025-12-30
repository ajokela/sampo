"""Sampo CPU Register File.

16 general-purpose 16-bit registers with:
- R0 hardwired to zero
- Alternate register bank for R4-R11 (swappable via EXX)
- Two read ports, one write port
"""

from amaranth import *


class RegisterFile(Elaboratable):
    """16 x 16-bit register file with alternate bank.

    Register 0 always reads as 0 and ignores writes.
    Registers 4-11 have alternate copies swappable via EXX.

    Inputs:
        rd_addr1: Read port 1 address (4-bit)
        rd_addr2: Read port 2 address (4-bit)
        wr_addr: Write port address (4-bit)
        wr_data: Write data (16-bit)
        wr_en: Write enable
        exx: Swap alternate registers (pulse)

    Outputs:
        rd_data1: Read port 1 data (16-bit)
        rd_data2: Read port 2 data (16-bit)
    """

    def __init__(self):
        # Read ports
        self.rd_addr1 = Signal(4)
        self.rd_addr2 = Signal(4)
        self.rd_data1 = Signal(16)
        self.rd_data2 = Signal(16)

        # Write port
        self.wr_addr = Signal(4)
        self.wr_data = Signal(16)
        self.wr_en = Signal()

        # Alternate register swap
        self.exx = Signal()

        # Debug access to all registers
        self.regs_out = Signal(16 * 16)

    def elaborate(self, platform):
        m = Module()

        # Main register bank (R0-R15)
        regs = Array([Signal(16, name=f"r{i}") for i in range(16)])

        # Alternate register bank (R4'-R11')
        regs_alt = Array([Signal(16, name=f"r{i}_alt") for i in range(4, 12)])

        # Read port 1 (R0 always reads as 0)
        with m.If(self.rd_addr1 == 0):
            m.d.comb += self.rd_data1.eq(0)
        with m.Else():
            m.d.comb += self.rd_data1.eq(regs[self.rd_addr1])

        # Read port 2 (R0 always reads as 0)
        with m.If(self.rd_addr2 == 0):
            m.d.comb += self.rd_data2.eq(0)
        with m.Else():
            m.d.comb += self.rd_data2.eq(regs[self.rd_addr2])

        # Write port (writes to R0 are ignored)
        with m.If(self.wr_en & (self.wr_addr != 0)):
            m.d.sync += regs[self.wr_addr].eq(self.wr_data)

        # EXX: Swap R4-R11 with alternate bank
        with m.If(self.exx):
            for i in range(8):
                reg_idx = i + 4
                m.d.sync += [
                    regs[reg_idx].eq(regs_alt[i]),
                    regs_alt[i].eq(regs[reg_idx]),
                ]

        # Debug output - concatenate all registers
        for i in range(16):
            m.d.comb += self.regs_out[i*16:(i+1)*16].eq(regs[i])

        return m


class RegisterFileAsync(Elaboratable):
    """Asynchronous-read register file variant.

    Same interface as RegisterFile but with combinational reads.
    This allows read-during-write behavior where the new value
    is immediately visible.
    """

    def __init__(self):
        # Read ports
        self.rd_addr1 = Signal(4)
        self.rd_addr2 = Signal(4)
        self.rd_data1 = Signal(16)
        self.rd_data2 = Signal(16)

        # Write port
        self.wr_addr = Signal(4)
        self.wr_data = Signal(16)
        self.wr_en = Signal()

        # Alternate register swap
        self.exx = Signal()

        # SP direct access (for PUSH/POP optimization)
        self.sp = Signal(16)
        self.sp_wr = Signal(16)
        self.sp_wr_en = Signal()

    def elaborate(self, platform):
        m = Module()

        # Main register bank (R0-R15)
        regs = Array([Signal(16, name=f"r{i}") for i in range(16)])

        # Alternate register bank (R4'-R11')
        regs_alt = Array([Signal(16, name=f"r{i}_alt") for i in range(4, 12)])

        # Read port 1 with bypass
        rd1_bypass = (self.wr_en & (self.wr_addr == self.rd_addr1) &
                      (self.rd_addr1 != 0))
        with m.If(self.rd_addr1 == 0):
            m.d.comb += self.rd_data1.eq(0)
        with m.Elif(rd1_bypass):
            m.d.comb += self.rd_data1.eq(self.wr_data)
        with m.Else():
            m.d.comb += self.rd_data1.eq(regs[self.rd_addr1])

        # Read port 2 with bypass
        rd2_bypass = (self.wr_en & (self.wr_addr == self.rd_addr2) &
                      (self.rd_addr2 != 0))
        with m.If(self.rd_addr2 == 0):
            m.d.comb += self.rd_data2.eq(0)
        with m.Elif(rd2_bypass):
            m.d.comb += self.rd_data2.eq(self.wr_data)
        with m.Else():
            m.d.comb += self.rd_data2.eq(regs[self.rd_addr2])

        # Write port
        with m.If(self.wr_en & (self.wr_addr != 0)):
            m.d.sync += regs[self.wr_addr].eq(self.wr_data)

        # SP direct access (R2)
        m.d.comb += self.sp.eq(regs[2])
        with m.If(self.sp_wr_en):
            m.d.sync += regs[2].eq(self.sp_wr)

        # EXX: Swap R4-R11 with alternate bank
        with m.If(self.exx):
            for i in range(8):
                reg_idx = i + 4
                m.d.sync += [
                    regs[reg_idx].eq(regs_alt[i]),
                    regs_alt[i].eq(regs[reg_idx]),
                ]

        return m
