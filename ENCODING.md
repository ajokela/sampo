# Sampo Instruction Encoding

Complete bit-level specification for all Sampo instructions.

## Notation

- `[15:12]` = bits 15 through 12 (4-bit opcode)
- `Rd` = destination register (4 bits)
- `Rs1`, `Rs2` = source registers (4 bits each)
- `imm8` = 8-bit immediate (signed or unsigned per instruction)
- `imm16` = 16-bit immediate (in second word for extended instructions)

---

## Format Summary

| Format | Bits 15:12 | Bits 11:8 | Bits 7:4 | Bits 3:0 |
|--------|------------|-----------|----------|----------|
| R | opcode | Rd | Rs1 | Rs2/func |
| I | opcode | Rd | imm8[7:0] | |
| S | opcode | imm4 | Rs1 | Rs2 |
| B | opcode | cond | offset8 | |
| J | opcode | offset12[11:0] | | |
| X | 0xF | Rd | Rs/func | sub | + imm16 |

---

## Opcode 0x0: ALU Register Operations

Format: `[15:12]=0x0 [11:8]=Rd [7:4]=Rs1 [3:0]=func`

| func | Mnemonic | Operation |
|------|----------|-----------|
| 0x0 | ADD Rd, Rs1, Rs2 | Rd = Rs1 + Rs2 |
| 0x1 | SUB Rd, Rs1, Rs2 | Rd = Rs1 - Rs2 |
| 0x2 | ADC Rd, Rs1, Rs2 | Rd = Rs1 + Rs2 + C |
| 0x3 | SBC Rd, Rs1, Rs2 | Rd = Rs1 - Rs2 - C |
| 0x4 | AND Rd, Rs1, Rs2 | Rd = Rs1 & Rs2 |
| 0x5 | OR Rd, Rs1, Rs2 | Rd = Rs1 \| Rs2 |
| 0x6 | XOR Rd, Rs1, Rs2 | Rd = Rs1 ^ Rs2 |
| 0x7 | NOT Rd, Rs1 | Rd = ~Rs1 |
| 0x8 | NEG Rd, Rs1 | Rd = -Rs1 |
| 0x9 | MOV Rd, Rs1 | Rd = Rs1 |
| 0xA | (reserved) | |
| 0xB | (reserved) | |
| 0xC | (reserved) | |
| 0xD | (reserved) | |
| 0xE | (reserved) | |
| 0xF | (reserved) | |

Note: For 3-register ops, Rs2 comes from a following byte or use Format R2.

**Alternate encoding for 3-register ALU:**

Format R2: `[15:12]=0x0 [11:10]=func[1:0] [9:8]=Rd[3:2] [7:6]=Rd[1:0],Rs1[3:2] [5:4]=Rs1[1:0],Rs2[3:2] [3:0]=Rs2[1:0],func[3:2]`

Actually, let's simplify. Use this encoding:

### Opcode 0x0: ALU Type A (Rd = Rs1 op Rs2)

```
15 14 13 12 | 11 10  9  8 |  7  6  5  4 |  3  2  1  0
   0  0  0  0 |    Rd       |     Rs1     |     Rs2
```

The specific operation is determined by sub-opcode in a prefix or we split into multiple opcodes.

---

## Revised Opcode Map

Let me reorganize for clarity:

| Opcode | Category | Format |
|--------|----------|--------|
| 0x0 | ADD | R: Rd = Rs1 + Rs2 |
| 0x1 | SUB | R: Rd = Rs1 - Rs2 |
| 0x2 | AND | R: Rd = Rs1 & Rs2 |
| 0x3 | OR | R: Rd = Rs1 \| Rs2 |
| 0x4 | XOR | R: Rd = Rs1 ^ Rs2 |
| 0x5 | ADDI | I: Rd = Rd + sign_ext(imm8) |
| 0x6 | Load/Store | See below |
| 0x7 | Load/Store | See below |
| 0x8 | Branch | B: conditional branch |
| 0x9 | Jump | J: unconditional jump/call |
| 0xA | Shift | R: shift operations |
| 0xB | Mul/Div | R: multiply/divide |
| 0xC | Stack/Misc | Stack and misc operations |
| 0xD | I/O | Port-based I/O |
| 0xE | System | System instructions |
| 0xF | Extended | 32-bit instructions |

---

## Complete Encoding Specification

### 0x0: ADD
```
15       12 11     8 7      4 3      0
+----------+--------+--------+--------+
|  0 0 0 0 |   Rd   |  Rs1   |  Rs2   |
+----------+--------+--------+--------+
```
Rd = Rs1 + Rs2. Sets N, Z, C, V flags.

### 0x1: SUB
```
15       12 11     8 7      4 3      0
+----------+--------+--------+--------+
|  0 0 0 1 |   Rd   |  Rs1   |  Rs2   |
+----------+--------+--------+--------+
```
Rd = Rs1 - Rs2. Sets N, Z, C, V flags.

### 0x2: AND
```
15       12 11     8 7      4 3      0
+----------+--------+--------+--------+
|  0 0 1 0 |   Rd   |  Rs1   |  Rs2   |
+----------+--------+--------+--------+
```
Rd = Rs1 & Rs2. Sets N, Z flags, clears C, V.

### 0x3: OR
```
15       12 11     8 7      4 3      0
+----------+--------+--------+--------+
|  0 0 1 1 |   Rd   |  Rs1   |  Rs2   |
+----------+--------+--------+--------+
```
Rd = Rs1 | Rs2. Sets N, Z flags.

### 0x4: XOR
```
15       12 11     8 7      4 3      0
+----------+--------+--------+--------+
|  0 1 0 0 |   Rd   |  Rs1   |  Rs2   |
+----------+--------+--------+--------+
```
Rd = Rs1 ^ Rs2. Sets N, Z flags.

### 0x5: ADDI (Add Immediate)
```
15       12 11     8 7                0
+----------+--------+------------------+
|  0 1 0 1 |   Rd   |      imm8        |
+----------+--------+------------------+
```
Rd = Rd + sign_extend(imm8). Sets N, Z, C, V flags.

### 0x6: Load Operations
```
15       12 11     8 7      4 3      0
+----------+--------+--------+--------+
|  0 1 1 0 |   Rd   |  Rs1   |  func  |
+----------+--------+--------+--------+
```

| func | Mnemonic | Operation |
|------|----------|-----------|
| 0x0 | LW Rd, (Rs1) | Rd = mem16[Rs1] |
| 0x1 | LB Rd, (Rs1) | Rd = sign_ext(mem8[Rs1]) |
| 0x2 | LBU Rd, (Rs1) | Rd = zero_ext(mem8[Rs1]) |
| 0x3 | LW Rd, 2(Rs1) | Rd = mem16[Rs1+2] |
| 0x4 | LW Rd, 4(Rs1) | Rd = mem16[Rs1+4] |
| 0x5 | LW Rd, 6(Rs1) | Rd = mem16[Rs1+6] |
| 0x6 | LW Rd, -2(Rs1) | Rd = mem16[Rs1-2] |
| 0x7 | LW Rd, -4(Rs1) | Rd = mem16[Rs1-4] |
| 0x8 | LUI Rd, Rs1 | Rd = Rs1 << 8 (load upper) |
| 0x9-0xF | (reserved) | |

### 0x7: Store Operations
```
15       12 11     8 7      4 3      0
+----------+--------+--------+--------+
|  0 1 1 1 |  Rs2   |  Rs1   |  func  |
+----------+--------+--------+--------+
```
Note: Rs2 is the value to store, Rs1 is the address base.

| func | Mnemonic | Operation |
|------|----------|-----------|
| 0x0 | SW (Rs1), Rs2 | mem16[Rs1] = Rs2 |
| 0x1 | SB (Rs1), Rs2 | mem8[Rs1] = Rs2[7:0] |
| 0x2 | SW 2(Rs1), Rs2 | mem16[Rs1+2] = Rs2 |
| 0x3 | SW 4(Rs1), Rs2 | mem16[Rs1+4] = Rs2 |
| 0x4 | SW 6(Rs1), Rs2 | mem16[Rs1+6] = Rs2 |
| 0x5 | SW -2(Rs1), Rs2 | mem16[Rs1-2] = Rs2 |
| 0x6 | SW -4(Rs1), Rs2 | mem16[Rs1-4] = Rs2 |
| 0x7-0xF | (reserved) | |

### 0x8: Branch Operations
```
15       12 11     8 7                0
+----------+--------+------------------+
|  1 0 0 0 |  cond  |     offset8      |
+----------+--------+------------------+
```
PC = PC + 2 + sign_extend(offset8) * 2, if condition is true.

| cond | Mnemonic | Condition |
|------|----------|-----------|
| 0x0 | BEQ | Z = 1 (equal) |
| 0x1 | BNE | Z = 0 (not equal) |
| 0x2 | BLT | N != V (signed less than) |
| 0x3 | BGE | N = V (signed greater or equal) |
| 0x4 | BLTU | C = 0 (unsigned less than) |
| 0x5 | BGEU | C = 1 (unsigned greater or equal) |
| 0x6 | BMI | N = 1 (minus/negative) |
| 0x7 | BPL | N = 0 (plus/positive) |
| 0x8 | BVS | V = 1 (overflow set) |
| 0x9 | BVC | V = 0 (overflow clear) |
| 0xA | BCS | C = 1 (carry set) |
| 0xB | BCC | C = 0 (carry clear) |
| 0xC | BGT | Z=0 and N=V (signed greater) |
| 0xD | BLE | Z=1 or N!=V (signed less or equal) |
| 0xE | BHI | C=1 and Z=0 (unsigned higher) |
| 0xF | BLS | C=0 or Z=1 (unsigned lower or same) |

### 0x9: Jump Operations
```
15       12 11                       0
+----------+--------------------------+
|  1 0 0 1 |       offset12           |
+----------+--------------------------+
```
For J (jump): PC = PC + 2 + sign_extend(offset12) * 2

**Alternate encoding for register jumps:**
```
15       12 11     8 7      4 3      0
+----------+--------+--------+--------+
|  1 0 0 1 |   Rd   |  Rs1   |  func  |
+----------+--------+--------+--------+
```
When bit 11 = 1 and bits 10:8 != 0, use register form:

| func | Mnemonic | Operation |
|------|----------|-----------|
| 0x0 | JR Rs1 | PC = Rs1 |
| 0x1 | JALR Rd, Rs1 | Rd = PC + 2; PC = Rs1 |
| 0x2 | JAL offset | RA = PC + 2; PC = PC + offset*2 |

Actually, let's use bit patterns:
- `1001 0xxx xxxx xxxx` = J offset11 (PC-relative, range ±2K)
- `1001 1RRR RRRR 0000` = JR Rs (register jump)
- `1001 1RRR RRRR 0001` = JALR Rd, Rs

Simplified:
```
J offset:     1001 | offset12 (signed, *2)
              1001 0xxxxxxxxxxx

JR Rs:        1001 1111 | Rs | 0000
              1001 1111 RRRR 0000

JALR Rd, Rs:  1001 | Rd | Rs | 0001
              1001 DDDD SSSS 0001

JAL offset:   (use extended format)
```

### 0xA: Shift Operations
```
15       12 11     8 7      4 3      0
+----------+--------+--------+--------+
|  1 0 1 0 |   Rd   |  Rs1   |  func  |
+----------+--------+--------+--------+
```

| func | Mnemonic | Operation |
|------|----------|-----------|
| 0x0 | SLL Rd, Rs1, 1 | Rd = Rs1 << 1 |
| 0x1 | SRL Rd, Rs1, 1 | Rd = Rs1 >> 1 (logical) |
| 0x2 | SRA Rd, Rs1, 1 | Rd = Rs1 >> 1 (arithmetic) |
| 0x3 | ROL Rd, Rs1, 1 | Rd = rotate_left(Rs1, 1) |
| 0x4 | ROR Rd, Rs1, 1 | Rd = rotate_right(Rs1, 1) |
| 0x5 | RCL Rd, Rs1, 1 | Rotate left through carry |
| 0x6 | RCR Rd, Rs1, 1 | Rotate right through carry |
| 0x7 | SWAP Rd, Rs1 | Rd = swap_bytes(Rs1) |
| 0x8-0xB | SLL/SRL/SRA/ROL Rd, Rs1, 4 | Shift by 4 |
| 0xC-0xF | SLL/SRL/SRA/ROL Rd, Rs1, 8 | Shift by 8 |

For variable shifts, use extended format with shift amount in Rs2.

### 0xB: Multiply/Divide
```
15       12 11     8 7      4 3      0
+----------+--------+--------+--------+
|  1 0 1 1 |   Rd   |  Rs1   |  func  |
+----------+--------+--------+--------+
```

For MUL/DIV, Rs2 is encoded in next instruction or we use:
```
15       12 11     8 7      4 3      0
+----------+--------+--------+--------+
|  1 0 1 1 |   Rd   |  Rs1   |  Rs2   |  (when func embedded in opcode variant)
```

Actually, let's use the high bit of func:

| [3] | [2:0] | Mnemonic | Operation |
|-----|-------|----------|-----------|
| 0 | 0x0 | MUL Rd, Rs1 | Rd = (Rd * Rs1)[15:0] |
| 0 | 0x1 | MULH Rd, Rs1 | Rd = (Rd * Rs1)[31:16] signed |
| 0 | 0x2 | MULHU Rd, Rs1 | Rd = (Rd * Rs1)[31:16] unsigned |
| 0 | 0x3 | DIV Rd, Rs1 | Rd = Rd / Rs1 (signed) |
| 0 | 0x4 | DIVU Rd, Rs1 | Rd = Rd / Rs1 (unsigned) |
| 0 | 0x5 | REM Rd, Rs1 | Rd = Rd % Rs1 (signed) |
| 0 | 0x6 | REMU Rd, Rs1 | Rd = Rd % Rs1 (unsigned) |
| 0 | 0x7 | DAA Rd | Decimal adjust Rd |
| 1 | xxx | (reserved for 3-reg forms) |

### 0xC: Stack and Misc Operations
```
15       12 11     8 7      4 3      0
+----------+--------+--------+--------+
|  1 1 0 0 |   Rd   |  Rs1   |  func  |
+----------+--------+--------+--------+
```

| func | Mnemonic | Operation |
|------|----------|-----------|
| 0x0 | PUSH Rs1 | SP -= 2; mem[SP] = Rs1 |
| 0x1 | POP Rd | Rd = mem[SP]; SP += 2 |
| 0x2 | CMP Rd, Rs1 | flags = Rd - Rs1 (no store) |
| 0x3 | TEST Rd, Rs1 | flags = Rd & Rs1 (no store) |
| 0x4 | MOV Rd, Rs1 | Rd = Rs1 |
| 0x5 | LDI | Block load increment |
| 0x6 | LDD | Block load decrement |
| 0x7 | LDIR | Block load repeat inc |
| 0x8 | LDDR | Block load repeat dec |
| 0x9 | CPIR | Compare and search |
| 0xA | FILL | Fill memory |
| 0xB | EXX | Swap alternate registers |
| 0xC | GETF Rd | Rd = FLAGS |
| 0xD | SETF Rs1 | FLAGS = Rs1[7:0] |
| 0xE | (reserved) | |
| 0xF | (reserved) | |

### 0xD: I/O Operations
```
15       12 11     8 7      4 3      0
+----------+--------+--------+--------+
|  1 1 0 1 |   Rd   |  port  |  func  |
+----------+--------+--------+--------+
```

For immediate port (4-bit port number in instruction):
| func | Mnemonic | Operation |
|------|----------|-----------|
| 0x0 | INI Rd, port | Rd = port_read(port) |
| 0x1 | OUTI port, Rd | port_write(port, Rd) |

For register port:
```
15       12 11     8 7      4 3      0
+----------+--------+--------+--------+
|  1 1 0 1 |   Rd   |  Rs1   |  func  |
+----------+--------+--------+--------+
```
| func | Mnemonic | Operation |
|------|----------|-----------|
| 0x2 | IN Rd, (Rs1) | Rd = port_read(Rs1[7:0]) |
| 0x3 | OUT (Rs1), Rd | port_write(Rs1[7:0], Rd) |

Extended I/O with 8-bit port:
```
15       12 11     8 7                0
+----------+--------+------------------+
|  1 1 0 1 |   Rd   |      port8       | + func nibble
```
Encode as: `1101 RRRR pppp pppp` where low 4 bits double as func=0x0 (IN) or use 0xF prefix.

Simpler: Use 0xF extended for 8-bit ports.

### 0xE: System Operations
```
15       12 11     8 7                0
+----------+--------+------------------+
|  1 1 1 0 |  func  |     operand      |
+----------+--------+------------------+
```

| func | Mnemonic | Operation |
|------|----------|-----------|
| 0x0 | NOP | No operation |
| 0x1 | HALT | Halt processor |
| 0x2 | DI | Disable interrupts |
| 0x3 | EI | Enable interrupts |
| 0x4 | RETI | Return from interrupt |
| 0x5 | SWI imm8 | Software interrupt |
| 0x6 | SCF | Set carry flag |
| 0x7 | CCF | Complement carry flag |
| 0x8 | SEI | Set interrupt flag |
| 0x9 | CLI | Clear interrupt flag |
| 0xA | (reserved) | |
| 0xB | (reserved) | |
| 0xC | (reserved) | |
| 0xD | (reserved) | |
| 0xE | (reserved) | |
| 0xF | (reserved) | |

### 0xF: Extended Instructions (32-bit)

First word:
```
15       12 11     8 7      4 3      0
+----------+--------+--------+--------+
|  1 1 1 1 |   Rd   |  Rs1   |  sub   |
+----------+--------+--------+--------+
```

Second word:
```
15                                   0
+-------------------------------------+
|              imm16                  |
+-------------------------------------+
```

| sub | Mnemonic | Operation |
|-----|----------|-----------|
| 0x0 | ADDIX Rd, Rs1, imm16 | Rd = Rs1 + imm16 |
| 0x1 | SUBIX Rd, Rs1, imm16 | Rd = Rs1 - imm16 |
| 0x2 | ANDIX Rd, Rs1, imm16 | Rd = Rs1 & imm16 |
| 0x3 | ORIX Rd, Rs1, imm16 | Rd = Rs1 \| imm16 |
| 0x4 | XORIX Rd, Rs1, imm16 | Rd = Rs1 ^ imm16 |
| 0x5 | LWX Rd, imm16(Rs1) | Rd = mem16[Rs1 + imm16] |
| 0x6 | SWX Rd, imm16(Rs1) | mem16[Rs1 + imm16] = Rd |
| 0x7 | LIX Rd, imm16 | Rd = imm16 (load immediate) |
| 0x8 | JX addr16 | PC = addr16 (absolute jump) |
| 0x9 | JALX addr16 | RA = PC + 4; PC = addr16 |
| 0xA | CMPIX Rd, imm16 | flags = Rd - imm16 |
| 0xB | INX Rd, port8 | Rd = port_read(imm16[7:0]) |
| 0xC | OUTX port8, Rs1 | port_write(imm16[7:0], Rs1) |
| 0xD | SLLX Rd, Rs1, imm4 | Rd = Rs1 << imm16[3:0] |
| 0xE | SRLX Rd, Rs1, imm4 | Rd = Rs1 >> imm16[3:0] |
| 0xF | SRAX Rd, Rs1, imm4 | Rd = Rs1 >>> imm16[3:0] |

---

## Register Encoding

| Code | Register | Alias |
|------|----------|-------|
| 0x0 | R0 | ZERO |
| 0x1 | R1 | RA |
| 0x2 | R2 | SP |
| 0x3 | R3 | GP |
| 0x4 | R4 | A0 |
| 0x5 | R5 | A1 |
| 0x6 | R6 | A2 |
| 0x7 | R7 | A3 |
| 0x8 | R8 | T0 |
| 0x9 | R9 | T1 |
| 0xA | R10 | T2 |
| 0xB | R11 | T3 |
| 0xC | R12 | S0 |
| 0xD | R13 | S1 |
| 0xE | R14 | S2 |
| 0xF | R15 | S3 |

---

## Encoding Examples

### ADD R4, R5, R6
```
Opcode = 0x0 (ADD)
Rd = 0x4 (R4)
Rs1 = 0x5 (R5)
Rs2 = 0x6 (R6)

Binary: 0000 0100 0101 0110 = 0x0456
```

### ADDI R4, 10
```
Opcode = 0x5 (ADDI)
Rd = 0x4 (R4)
imm8 = 10 = 0x0A

Binary: 0101 0100 0000 1010 = 0x540A
```

### LW R4, (R5)
```
Opcode = 0x6 (Load)
Rd = 0x4 (R4)
Rs1 = 0x5 (R5)
func = 0x0 (LW, offset 0)

Binary: 0110 0100 0101 0000 = 0x6450
```

### BEQ +8 (branch forward 8 bytes = 4 instructions)
```
Opcode = 0x8 (Branch)
cond = 0x0 (BEQ)
offset8 = 4 (words) = 0x04

Binary: 1000 0000 0000 0100 = 0x8004
```

### J -100 (jump back 100 bytes = 50 words)
```
Opcode = 0x9 (Jump)
offset12 = -50 = 0xFCE (12-bit signed)

Binary: 1001 1111 1100 1110 = 0x9FCE
```

### PUSH R4
```
Opcode = 0xC (Stack/Misc)
Rd = 0x0 (unused)
Rs1 = 0x4 (R4)
func = 0x0 (PUSH)

Binary: 1100 0000 0100 0000 = 0xC040
```

### INI R4, 0x80 (read from port 0x80)
Using extended format:
```
Word 0: 1111 0100 0000 1011 = 0xF40B (sub=0xB for INX)
Word 1: 0000 0000 1000 0000 = 0x0080 (port 0x80)
```

### LIX R4, 0x1234 (load 16-bit immediate)
```
Word 0: 1111 0100 0000 0111 = 0xF407 (sub=0x7 for LIX)
Word 1: 0001 0010 0011 0100 = 0x1234
```

---

## Quick Reference Card

| Opcode | Mnemonic | Format |
|--------|----------|--------|
| 0x0 | ADD | R: Rd = Rs1 + Rs2 |
| 0x1 | SUB | R: Rd = Rs1 - Rs2 |
| 0x2 | AND | R: Rd = Rs1 & Rs2 |
| 0x3 | OR | R: Rd = Rs1 \| Rs2 |
| 0x4 | XOR | R: Rd = Rs1 ^ Rs2 |
| 0x5 | ADDI | I: Rd = Rd + imm8 |
| 0x6 | LOAD | R: LW/LB/LBU |
| 0x7 | STORE | R: SW/SB |
| 0x8 | BRANCH | B: conditional |
| 0x9 | JUMP | J: J/JR/JALR |
| 0xA | SHIFT | R: SLL/SRL/SRA/ROL/ROR |
| 0xB | MULDIV | R: MUL/DIV/REM/DAA |
| 0xC | MISC | R: PUSH/POP/CMP/LDI/EXX |
| 0xD | I/O | I: IN/OUT |
| 0xE | SYSTEM | S: NOP/HALT/DI/EI/RETI |
| 0xF | EXTENDED | X: 32-bit ops |
