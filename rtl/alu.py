"""Sampo CPU ALU (Arithmetic Logic Unit)."""

from amaranth import *
from amaranth.lib.enum import Enum


class ALUOp(Enum, shape=4):
    """ALU operation codes."""
    ADD  = 0x0
    SUB  = 0x1
    AND  = 0x2
    OR   = 0x3
    XOR  = 0x4
    SLL  = 0x5  # Shift left logical
    SRL  = 0x6  # Shift right logical
    SRA  = 0x7  # Shift right arithmetic
    MUL  = 0x8  # Multiply (low)
    MULH = 0x9  # Multiply (high, signed)
    DIV  = 0xA  # Divide
    REM  = 0xB  # Remainder
    PASS_A = 0xC  # Pass through A
    PASS_B = 0xD  # Pass through B
    NOT  = 0xE  # Bitwise NOT of B
    NEG  = 0xF  # Negate B


class ALU(Elaboratable):
    """16-bit ALU for Sampo CPU.

    Inputs:
        a: First operand (16-bit)
        b: Second operand (16-bit)
        op: ALU operation (4-bit)
        carry_in: Carry input for ADC/SBC

    Outputs:
        result: Result (16-bit)
        flag_n: Negative flag (result[15])
        flag_z: Zero flag (result == 0)
        flag_c: Carry/borrow flag
        flag_v: Overflow flag (signed)
    """

    def __init__(self):
        # Inputs
        self.a = Signal(16)
        self.b = Signal(16)
        self.op = Signal(ALUOp)
        self.carry_in = Signal()
        self.shift_amt = Signal(4)  # For variable shifts

        # Outputs
        self.result = Signal(16)
        self.flag_n = Signal()
        self.flag_z = Signal()
        self.flag_c = Signal()
        self.flag_v = Signal()

    def elaborate(self, platform):
        m = Module()

        # Extended result for carry detection
        result_wide = Signal(17)

        # Multiplication result (32-bit)
        mul_result = Signal(32)
        m.d.comb += mul_result.eq(self.a.as_signed() * self.b.as_signed())

        # Unsigned multiplication for MULHU
        mul_result_u = Signal(32)
        m.d.comb += mul_result_u.eq(self.a * self.b)

        # Main ALU operation
        with m.Switch(self.op):
            with m.Case(ALUOp.ADD):
                m.d.comb += result_wide.eq(self.a + self.b)

            with m.Case(ALUOp.SUB):
                m.d.comb += result_wide.eq(self.a - self.b)

            with m.Case(ALUOp.AND):
                m.d.comb += result_wide.eq(self.a & self.b)

            with m.Case(ALUOp.OR):
                m.d.comb += result_wide.eq(self.a | self.b)

            with m.Case(ALUOp.XOR):
                m.d.comb += result_wide.eq(self.a ^ self.b)

            with m.Case(ALUOp.SLL):
                m.d.comb += result_wide.eq(self.a << self.shift_amt)

            with m.Case(ALUOp.SRL):
                m.d.comb += result_wide.eq(self.a >> self.shift_amt)

            with m.Case(ALUOp.SRA):
                m.d.comb += result_wide.eq(self.a.as_signed() >> self.shift_amt)

            with m.Case(ALUOp.MUL):
                m.d.comb += result_wide.eq(mul_result[:16])

            with m.Case(ALUOp.MULH):
                m.d.comb += result_wide.eq(mul_result[16:32])

            with m.Case(ALUOp.DIV):
                with m.If(self.b != 0):
                    m.d.comb += result_wide.eq(
                        (self.a.as_signed() // self.b.as_signed()).as_unsigned()
                    )
                with m.Else():
                    m.d.comb += result_wide.eq(0xFFFF)

            with m.Case(ALUOp.REM):
                with m.If(self.b != 0):
                    m.d.comb += result_wide.eq(
                        (self.a.as_signed() % self.b.as_signed()).as_unsigned()
                    )
                with m.Else():
                    m.d.comb += result_wide.eq(self.a)

            with m.Case(ALUOp.PASS_A):
                m.d.comb += result_wide.eq(self.a)

            with m.Case(ALUOp.PASS_B):
                m.d.comb += result_wide.eq(self.b)

            with m.Case(ALUOp.NOT):
                m.d.comb += result_wide.eq(~self.b)

            with m.Case(ALUOp.NEG):
                m.d.comb += result_wide.eq(-self.b.as_signed())

            with m.Default():
                m.d.comb += result_wide.eq(0)

        # Output result (lower 16 bits)
        m.d.comb += self.result.eq(result_wide[:16])

        # Flag generation
        m.d.comb += [
            # Negative: MSB of result
            self.flag_n.eq(self.result[15]),

            # Zero: result is zero
            self.flag_z.eq(self.result == 0),

            # Carry: bit 16 of result (for ADD/SUB)
            self.flag_c.eq(result_wide[16]),
        ]

        # Overflow detection (signed arithmetic)
        # Overflow occurs when:
        # ADD: same sign inputs, different sign output
        # SUB: different sign inputs, output sign != a sign
        a_sign = self.a[15]
        b_sign = self.b[15]
        r_sign = self.result[15]

        with m.Switch(self.op):
            with m.Case(ALUOp.ADD):
                # Overflow if signs of a and b match but result differs
                m.d.comb += self.flag_v.eq((a_sign == b_sign) & (a_sign != r_sign))
            with m.Case(ALUOp.SUB):
                # Overflow if signs of a and b differ and result sign != a sign
                m.d.comb += self.flag_v.eq((a_sign != b_sign) & (r_sign != a_sign))
            with m.Default():
                m.d.comb += self.flag_v.eq(0)

        return m


class Shifter(Elaboratable):
    """Barrel shifter for shift/rotate operations.

    Handles the various shift modes from the SHIFT opcode.
    """

    def __init__(self):
        self.value = Signal(16)
        self.func = Signal(4)  # ShiftFunc
        self.carry_in = Signal()

        self.result = Signal(16)
        self.carry_out = Signal()

    def elaborate(self, platform):
        m = Module()

        v = self.value

        with m.Switch(self.func):
            # Single bit shifts
            with m.Case(0x0):  # SLL1
                m.d.comb += [
                    self.result.eq(v << 1),
                    self.carry_out.eq(v[15]),
                ]
            with m.Case(0x1):  # SRL1
                m.d.comb += [
                    self.result.eq(v >> 1),
                    self.carry_out.eq(v[0]),
                ]
            with m.Case(0x2):  # SRA1
                m.d.comb += [
                    self.result.eq(Cat(v[1:16], v[15])),
                    self.carry_out.eq(v[0]),
                ]
            with m.Case(0x3):  # ROL1
                m.d.comb += [
                    self.result.eq(Cat(v[15], v[0:15])),
                    self.carry_out.eq(v[15]),
                ]
            with m.Case(0x4):  # ROR1
                m.d.comb += [
                    self.result.eq(Cat(v[1:16], v[0])),
                    self.carry_out.eq(v[0]),
                ]
            with m.Case(0x5):  # RCL1 (rotate through carry left)
                m.d.comb += [
                    self.result.eq(Cat(self.carry_in, v[0:15])),
                    self.carry_out.eq(v[15]),
                ]
            with m.Case(0x6):  # RCR1 (rotate through carry right)
                m.d.comb += [
                    self.result.eq(Cat(v[1:16], self.carry_in)),
                    self.carry_out.eq(v[0]),
                ]
            with m.Case(0x7):  # SWAP bytes
                m.d.comb += [
                    self.result.eq(Cat(v[8:16], v[0:8])),
                    self.carry_out.eq(0),
                ]

            # 4-bit shifts
            with m.Case(0x8):  # SLL4
                m.d.comb += [
                    self.result.eq(v << 4),
                    self.carry_out.eq(v[12]),
                ]
            with m.Case(0x9):  # SRL4
                m.d.comb += [
                    self.result.eq(v >> 4),
                    self.carry_out.eq(v[3]),
                ]
            with m.Case(0xA):  # SRA4
                m.d.comb += [
                    self.result.eq(v.as_signed() >> 4),
                    self.carry_out.eq(v[3]),
                ]
            with m.Case(0xB):  # ROL4
                m.d.comb += [
                    self.result.eq(v.rotate_left(4)),
                    self.carry_out.eq(v[12]),
                ]

            # 8-bit shifts
            with m.Case(0xC):  # SLL8
                m.d.comb += [
                    self.result.eq(v << 8),
                    self.carry_out.eq(v[8]),
                ]
            with m.Case(0xD):  # SRL8
                m.d.comb += [
                    self.result.eq(v >> 8),
                    self.carry_out.eq(v[7]),
                ]
            with m.Case(0xE):  # SRA8
                m.d.comb += [
                    self.result.eq(v.as_signed() >> 8),
                    self.carry_out.eq(v[7]),
                ]
            with m.Case(0xF):  # ROL8
                m.d.comb += [
                    self.result.eq(v.rotate_left(8)),
                    self.carry_out.eq(v[8]),
                ]

            with m.Default():
                m.d.comb += [
                    self.result.eq(v),
                    self.carry_out.eq(0),
                ]

        return m
