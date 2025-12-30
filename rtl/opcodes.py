"""Sampo CPU opcode definitions and constants."""

from enum import IntEnum

# Primary opcodes (4-bit, bits 15:12)
class Opcode(IntEnum):
    ADD     = 0x0
    SUB     = 0x1
    AND     = 0x2
    OR      = 0x3
    XOR     = 0x4
    ADDI    = 0x5
    LOAD    = 0x6
    STORE   = 0x7
    BRANCH  = 0x8
    JUMP    = 0x9
    SHIFT   = 0xA
    MULDIV  = 0xB
    MISC    = 0xC
    IO      = 0xD
    SYSTEM  = 0xE
    EXTENDED = 0xF

# Load function codes (bits 3:0)
class LoadFunc(IntEnum):
    LW      = 0x0  # Load word
    LB      = 0x1  # Load byte (sign extend)
    LBU     = 0x2  # Load byte (zero extend)
    LUI     = 0x8  # Load upper immediate

# Store function codes (bits 3:0)
class StoreFunc(IntEnum):
    SW      = 0x0  # Store word
    SB      = 0x1  # Store byte

# Branch conditions (bits 11:8)
class BranchCond(IntEnum):
    BEQ     = 0x0  # Equal (Z=1)
    BNE     = 0x1  # Not equal (Z=0)
    BLT     = 0x2  # Less than signed (N!=V)
    BGE     = 0x3  # Greater or equal signed (N=V)
    BLTU    = 0x4  # Less than unsigned (C=0)
    BGEU    = 0x5  # Greater or equal unsigned (C=1)
    BMI     = 0x6  # Minus (N=1)
    BPL     = 0x7  # Plus (N=0)
    BVS     = 0x8  # Overflow set (V=1)
    BVC     = 0x9  # Overflow clear (V=0)
    BCS     = 0xA  # Carry set (C=1)
    BCC     = 0xB  # Carry clear (C=0)
    BGT     = 0xC  # Greater than signed
    BLE     = 0xD  # Less or equal signed
    BHI     = 0xE  # Higher unsigned
    BLS     = 0xF  # Lower or same unsigned

# Shift function codes (bits 3:0)
class ShiftFunc(IntEnum):
    SLL1    = 0x0  # Shift left 1
    SRL1    = 0x1  # Shift right logical 1
    SRA1    = 0x2  # Shift right arithmetic 1
    ROL1    = 0x3  # Rotate left 1
    ROR1    = 0x4  # Rotate right 1
    RCL1    = 0x5  # Rotate through carry left 1
    RCR1    = 0x6  # Rotate through carry right 1
    SWAP    = 0x7  # Swap bytes
    SLL4    = 0x8  # Shift left 4
    SRL4    = 0x9  # Shift right logical 4
    SRA4    = 0xA  # Shift right arithmetic 4
    ROL4    = 0xB  # Rotate left 4
    SLL8    = 0xC  # Shift left 8
    SRL8    = 0xD  # Shift right logical 8
    SRA8    = 0xE  # Shift right arithmetic 8
    ROL8    = 0xF  # Rotate left 8

# Multiply/Divide function codes (bits 3:0)
class MulDivFunc(IntEnum):
    MUL     = 0x0  # Multiply (low 16 bits)
    MULH    = 0x1  # Multiply high (signed)
    MULHU   = 0x2  # Multiply high (unsigned)
    DIV     = 0x3  # Divide (signed)
    DIVU    = 0x4  # Divide (unsigned)
    REM     = 0x5  # Remainder (signed)
    REMU    = 0x6  # Remainder (unsigned)
    DAA     = 0x7  # Decimal adjust

# Misc function codes (bits 3:0)
class MiscFunc(IntEnum):
    PUSH    = 0x0
    POP     = 0x1
    CMP     = 0x2
    TEST    = 0x3
    MOV     = 0x4
    LDI     = 0x5  # Block load increment
    LDD     = 0x6  # Block load decrement
    LDIR    = 0x7  # Block load repeat increment
    LDDR    = 0x8  # Block load repeat decrement
    CPIR    = 0x9  # Compare and search
    FILL    = 0xA
    EXX     = 0xB  # Exchange alternate registers
    GETF    = 0xC  # Get flags
    SETF    = 0xD  # Set flags

# I/O function codes (bits 3:0)
class IOFunc(IntEnum):
    INI     = 0x0  # Input immediate port
    OUTI    = 0x1  # Output immediate port
    IN      = 0x2  # Input register port
    OUT     = 0x3  # Output register port

# System function codes (bits 11:8)
class SystemFunc(IntEnum):
    NOP     = 0x0
    HALT    = 0x1
    DI      = 0x2  # Disable interrupts
    EI      = 0x3  # Enable interrupts
    RETI    = 0x4  # Return from interrupt
    SWI     = 0x5  # Software interrupt
    SCF     = 0x6  # Set carry flag
    CCF     = 0x7  # Complement carry flag

# Extended sub-opcodes (bits 3:0 when opcode = 0xF)
class ExtendedFunc(IntEnum):
    ADDIX   = 0x0
    SUBIX   = 0x1
    ANDIX   = 0x2
    ORIX    = 0x3
    XORIX   = 0x4
    LWX     = 0x5
    SWX     = 0x6
    LIX     = 0x7
    JX      = 0x8
    JALX    = 0x9
    CMPIX   = 0xA
    INX     = 0xB
    OUTX    = 0xC
    SLLX    = 0xD
    SRLX    = 0xE
    SRAX    = 0xF

# Flag bit positions
class Flag(IntEnum):
    N = 7  # Negative
    Z = 6  # Zero
    C = 5  # Carry
    V = 4  # Overflow
    H = 3  # Half-carry (BCD)
    I = 2  # Interrupt enable

# Register aliases
class RegAlias(IntEnum):
    ZERO = 0   # Always zero
    RA   = 1   # Return address
    SP   = 2   # Stack pointer
    GP   = 3   # Global pointer
    A0   = 4   # Argument 0
    A1   = 5   # Argument 1
    A2   = 6   # Argument 2
    A3   = 7   # Argument 3
    T0   = 8   # Temporary 0
    T1   = 9   # Temporary 1
    T2   = 10  # Temporary 2
    T3   = 11  # Temporary 3
    S0   = 12  # Saved 0
    S1   = 13  # Saved 1
    S2   = 14  # Saved 2
    S3   = 15  # Saved 3
