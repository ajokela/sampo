# Sampo CPU Architecture

A 16-bit RISC processor designed to support workloads from the kz80 ecosystem.

## Design Goals

1. **RISC-inspired instruction set** - Clean, orthogonal design with Z80-friendly extensions
2. **16-bit native word size** - Registers, ALU, memory addressing
3. **Efficient for interpreters and compilers** - Stack operations, indirect addressing
4. **Simple to implement** - Suitable for FPGA or software emulation
5. **Z80-workload compatible** - Port-based I/O, BCD, block operations, alternate registers

---

## Register File

### General Purpose Registers (16 x 16-bit)

| Register | Name | Convention |
|----------|------|------------|
| R0 | ZERO | Always reads as 0, writes ignored |
| R1 | RA | Return address |
| R2 | SP | Stack pointer |
| R3 | GP | Global pointer (optional) |
| R4-R7 | A0-A3 | Arguments / Return values |
| R8-R11 | T0-T3 | Temporaries (caller-saved) |
| R12-R15 | S0-S3 | Saved registers (callee-saved) |

### Alternate Register Set

Registers R4-R11 have shadow copies (R4'-R11') for fast context switching:

| Primary | Alternate | Usage |
|---------|-----------|-------|
| R4-R7 | R4'-R7' | Arguments (swappable) |
| R8-R11 | R8'-R11' | Temporaries (swappable) |

Use `EXX` instruction to swap primary ↔ alternate registers.

### Special Registers

| Register | Width | Description |
|----------|-------|-------------|
| PC | 16-bit | Program counter |
| FLAGS | 8-bit | Status flags |
| IVR | 16-bit | Interrupt vector register |

### Flags Register

```
Bit 7: N (Negative) - Sign of result
Bit 6: Z (Zero) - Result is zero
Bit 5: C (Carry) - Unsigned overflow
Bit 4: V (Overflow) - Signed overflow
Bit 3: H (Half-carry) - For BCD (optional)
Bit 2-0: Reserved
```

---

## Memory Model

- **Address space**: 64KB (16-bit addresses)
- **Byte-addressable**: 8-bit memory access supported
- **Word alignment**: 16-bit words should be aligned (optional enforcement)
- **Endianness**: Little-endian

### Memory Map (suggested)

```
0x0000-0x00FF   Interrupt vectors / Reset
0x0100-0x7FFF   Program ROM (~32KB)
0x8000-0xFEFF   RAM (~32KB)
0xFF00-0xFFFF   Memory-mapped I/O (256 bytes)
```

---

## Instruction Formats

Instructions are **16 bits** by default, with **32-bit extended** forms available.

### Format R: Register-Register Operations (16-bit)
```
15    12 11   8 7    4 3    0
+-------+------+------+------+
| opcode|  Rd  |  Rs1 | Rs2  |
+-------+------+------+------+
```

### Format I: Immediate Operations (16-bit)
```
15    12 11   8 7           0
+-------+------+-------------+
| opcode|  Rd  |   imm8      |
+-------+------+-------------+
```

### Format IX: Extended Immediate (32-bit)
```
Word 0:                          Word 1:
15    12 11   8 7    4 3    0    15                    0
+-------+------+------+------+   +---------------------+
| 1111  |  Rd  |  Rs  | sub  |   |       imm16         |
+-------+------+------+------+   +---------------------+
```
Opcode 0xF indicates extended instruction. Sub-opcode in bits 3:0.

### Format S: Store Operations (16-bit)
```
15    12 11   8 7    4 3    0
+-------+------+------+------+
| opcode| imm4 |  Rs1 | Rs2  |
+-------+------+------+------+
```
Address = Rs1 + sign_extend(imm4), data from Rs2

### Format B: Branch Operations (16-bit)
```
15    12 11   8 7           0
+-------+------+-------------+
| opcode| cond |   offset8   |
+-------+------+-------------+
```
PC-relative, offset in words (range: -256 to +254 bytes)

### Format J: Jump Operations (16-bit)
```
15    12 11                 0
+-------+-------------------+
| opcode|      offset12     |
+-------+-------------------+
```
PC-relative, offset in words (range: -4096 to +4094 bytes)

### Format JX: Extended Jump (32-bit)
```
Word 0:                          Word 1:
15    12 11                 0    15                    0
+-------+-------------------+    +---------------------+
| 1111  |     0x800         |    |      addr16         |
+-------+-------------------+    +---------------------+
```
Absolute 16-bit jump/call target.

---

## Instruction Set

### Load/Store (6 instructions)

| Mnemonic | Format | Description |
|----------|--------|-------------|
| LW Rd, imm(Rs) | I | Load word: Rd = mem[Rs + imm] |
| LB Rd, imm(Rs) | I | Load byte (sign-extend) |
| LBU Rd, imm(Rs) | I | Load byte (zero-extend) |
| SW Rs2, imm(Rs1) | S | Store word |
| SB Rs2, imm(Rs1) | S | Store byte |
| LUI Rd, imm8 | I | Load upper immediate: Rd = imm8 << 8 |

### Arithmetic (12 instructions)

| Mnemonic | Format | Description |
|----------|--------|-------------|
| ADD Rd, Rs1, Rs2 | R | Rd = Rs1 + Rs2 |
| SUB Rd, Rs1, Rs2 | R | Rd = Rs1 - Rs2 |
| ADDI Rd, Rs, imm | I | Rd = Rs + sign_extend(imm8) |
| ADDIX Rd, Rs, imm16 | IX | Rd = Rs + imm16 (32-bit encoding) |
| ADC Rd, Rs1, Rs2 | R | Rd = Rs1 + Rs2 + C |
| SBC Rd, Rs1, Rs2 | R | Rd = Rs1 - Rs2 - C |
| MUL Rd, Rs1, Rs2 | R | Rd = (Rs1 * Rs2)[15:0] |
| MULH Rd, Rs1, Rs2 | R | Rd = (Rs1 * Rs2)[31:16] (signed) |
| MULHU Rd, Rs1, Rs2 | R | Rd = (Rs1 * Rs2)[31:16] (unsigned) |
| DIV Rd, Rs1, Rs2 | R | Rd = Rs1 / Rs2 (signed) |
| DIVU Rd, Rs1, Rs2 | R | Rd = Rs1 / Rs2 (unsigned) |
| REM Rd, Rs1, Rs2 | R | Rd = Rs1 % Rs2 (signed) |
| CMP Rs1, Rs2 | R | Set flags from Rs1 - Rs2 |
| NEG Rd, Rs | R | Rd = -Rs (two's complement) |
| DAA Rd | R | Decimal adjust Rd for BCD |

### Logic (6 instructions)

| Mnemonic | Format | Description |
|----------|--------|-------------|
| AND Rd, Rs1, Rs2 | R | Rd = Rs1 & Rs2 |
| OR Rd, Rs1, Rs2 | R | Rd = Rs1 \| Rs2 |
| XOR Rd, Rs1, Rs2 | R | Rd = Rs1 ^ Rs2 |
| ANDI Rd, Rs, imm | I | Rd = Rs & zero_extend(imm8) |
| ORI Rd, Rs, imm | I | Rd = Rs \| zero_extend(imm8) |
| NOT Rd, Rs | R | Rd = ~Rs |

### Shift (4 instructions)

| Mnemonic | Format | Description |
|----------|--------|-------------|
| SLL Rd, Rs1, Rs2 | R | Rd = Rs1 << Rs2[3:0] |
| SRL Rd, Rs1, Rs2 | R | Rd = Rs1 >> Rs2[3:0] (logical) |
| SRA Rd, Rs1, Rs2 | R | Rd = Rs1 >> Rs2[3:0] (arithmetic) |
| SLLI Rd, Rs, imm | I | Rd = Rs << imm[3:0] |

### Branch (8 conditions)

| Mnemonic | Condition | Description |
|----------|-----------|-------------|
| BEQ | Z=1 | Branch if equal |
| BNE | Z=0 | Branch if not equal |
| BLT | N!=V | Branch if less than (signed) |
| BGE | N=V | Branch if greater or equal (signed) |
| BLTU | C=0 | Branch if less than (unsigned) |
| BGEU | C=1 | Branch if greater or equal (unsigned) |
| BMI | N=1 | Branch if minus |
| BPL | N=0 | Branch if plus |

### Jump/Call (4 instructions)

| Mnemonic | Format | Description |
|----------|--------|-------------|
| J offset | J | PC = PC + sign_extend(offset12) * 2 |
| JAL offset | J | RA = PC + 2; PC = PC + offset * 2 |
| JR Rs | R | PC = Rs |
| JALR Rd, Rs | R | Rd = PC + 2; PC = Rs |

### Stack (4 instructions)

| Mnemonic | Description |
|----------|-------------|
| PUSH Rs | SP -= 2; mem[SP] = Rs |
| POP Rd | Rd = mem[SP]; SP += 2 |
| PUSHM mask | Push multiple (R4-R15 based on mask) |
| POPM mask | Pop multiple |

### Block Operations (6 instructions)

| Mnemonic | Description |
|----------|-------------|
| LDI | mem[Rd]++ = mem[Rs]++; Rc--; Z=(Rc==0) |
| LDD | mem[Rd]-- = mem[Rs]--; Rc--; Z=(Rc==0) |
| LDIR | Repeat LDI until Rc == 0 |
| LDDR | Repeat LDD until Rc == 0 |
| FILL Rd, Rs, Rc | Fill Rc bytes at Rd with value Rs |
| CPIR | Compare and search forward |

Block operations use: Rd=R6 (dest), Rs=R5 (src), Rc=R4 (count)

### I/O (Port-Based)

| Mnemonic | Format | Description |
|----------|--------|-------------|
| IN Rd, (Rs) | R | Rd = port_read(Rs) |
| INI Rd, imm8 | I | Rd = port_read(imm8) |
| OUT (Rd), Rs | R | port_write(Rd, Rs) |
| OUTI imm8, Rs | I | port_write(imm8, Rs) |

Port address space is 256 bytes (8-bit port numbers).

### System (8 instructions)

| Mnemonic | Description |
|----------|-------------|
| NOP | No operation |
| HALT | Halt processor |
| DI | Disable interrupts |
| EI | Enable interrupts |
| EXX | Swap R4-R11 with alternate registers |
| RETI | Return from interrupt |
| SWI imm | Software interrupt (trap) |
| GETF Rd | Rd = FLAGS register |
| SETF Rs | FLAGS = Rs (low 8 bits) |

---

## Total Instruction Count

| Category | Count |
|----------|-------|
| Load/Store | 6 |
| Arithmetic | 15 |
| Logic | 6 |
| Shift | 4 |
| Branch | 8 |
| Jump/Call | 4 |
| Stack | 4 |
| Block Ops | 6 |
| I/O | 4 |
| System | 9 |
| **Total** | **~66** |

Plus 32-bit extended forms for larger immediates.

---

## Addressing Modes

1. **Register direct**: `ADD R4, R5, R6`
2. **Immediate**: `ADDI R4, R5, 42`
3. **Register indirect with offset**: `LW R4, 8(R5)`
4. **PC-relative**: `BEQ label` or `J label`

---

## Interrupt Model

- Single interrupt vector at 0x0004
- Interrupt saves PC to a dedicated register (or stack)
- Use `RETI` to return from interrupt (restores PC and re-enables interrupts)

---

## Calling Convention

```
Arguments:  R4-R7 (A0-A3), then stack
Return:     R4-R5 (A0-A1)
Caller-saved: R4-R11 (A0-A3, T0-T3)
Callee-saved: R12-R15 (S0-S3), SP
Return addr:  R1 (RA)
```

### Function Prologue
```asm
    PUSH RA          ; Save return address
    PUSH S0          ; Save callee-saved registers
    ADDI SP, SP, -N  ; Allocate local variables
```

### Function Epilogue
```asm
    ADDI SP, SP, N   ; Deallocate locals
    POP S0           ; Restore callee-saved
    POP RA           ; Restore return address
    JR RA            ; Return
```

---

## Example Code

### Hello World (serial output via port I/O)
```asm
        .text
        .org 0x0100

; Serial port definitions (MC6850 ACIA style)
.equ    ACIA_STATUS, 0x80       ; Status register port
.equ    ACIA_DATA,   0x81       ; Data register port
.equ    TX_READY,    0x02       ; Transmit ready bit

start:
        LUI  R4, hi(message)    ; Load upper byte of address
        ORI  R4, R4, lo(message) ; Load lower byte
loop:
        LBU  R5, 0(R4)          ; Load byte from string
        BEQ  R5, R0, done       ; If null, done

wait_tx:
        INI  R6, ACIA_STATUS    ; Read serial status port
        ANDI R6, R6, TX_READY   ; Check transmit ready
        BEQ  R6, R0, wait_tx    ; Wait if not ready

        OUTI ACIA_DATA, R5      ; Write character to data port
        ADDI R4, R4, 1          ; Next character
        J    loop
done:
        HALT

message:
        .asciz "Hello, Sampo!\n"
```

### Fibonacci
```asm
; fib(n) - compute nth Fibonacci number
; Input: R4 = n
; Output: R4 = fib(n)
fib:
        ADDI R5, R0, 0      ; a = 0
        ADDI R6, R0, 1      ; b = 1
        BEQ  R4, R0, fib_done
fib_loop:
        ADD  R7, R5, R6     ; temp = a + b
        ADD  R5, R6, R0     ; a = b
        ADD  R6, R7, R0     ; b = temp
        ADDI R4, R4, -1     ; n--
        BNE  R4, R0, fib_loop
fib_done:
        ADD  R4, R5, R0     ; return a
        JR   RA
```

---

## Design Decisions (Resolved)

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Instruction width | 16-bit + 32-bit extended | Code density + flexibility |
| I/O model | Port-based (IN/OUT) | Z80 compatibility |
| Multiply/Divide | Yes (MUL, MULH, DIV, REM) | Interpreter performance |
| BCD support | Yes (DAA) | Calculator applications |
| Block operations | Yes (LDIR, LDDR, FILL, CPIR) | String/memory efficiency |
| Alternate registers | Yes (EXX for R4-R11) | Fast interrupt handling |

---

## Comparison with Z80

| Feature | Z80 | Sampo |
|---------|-----|-------|
| Data width | 8-bit | 16-bit |
| Address width | 16-bit | 16-bit |
| Registers | 8 + alternates | 16 + alternates (R4-R11) |
| Instruction width | 1-4 bytes | 2 or 4 bytes |
| Instruction count | ~300+ | ~66 |
| Addressing modes | Many (complex) | 4 (simple) |
| I/O | Port-based | Port-based |
| Hardware multiply | No | Yes |
| BCD | DAA | DAA |
| Block ops | LDIR, LDDR, etc. | LDIR, LDDR, FILL, CPIR |

### Block Copy Example
```asm
; Copy 256 bytes from src to dest using LDIR
; Uses R4=count, R5=source, R6=dest (convention)
        LUI  R5, hi(src)
        ORI  R5, R5, lo(src)    ; R5 = source address
        LUI  R6, hi(dest)
        ORI  R6, R6, lo(dest)   ; R6 = dest address
        LUI  R4, 0x01           ; R4 = 256 (0x0100)
        LDIR                    ; Copy R4 bytes from [R5] to [R6]
```

### Interrupt Handler with Alternate Registers
```asm
irq_handler:
        EXX                     ; Swap to alternate R4-R11
        ; ... handle interrupt using R4'-R11' ...
        ; Primary registers preserved automatically
        EXX                     ; Swap back
        RETI                    ; Return from interrupt
```

---

## Instruction Encoding Details

### Opcode Map (4-bit primary opcode)

| Opcode | Category |
|--------|----------|
| 0x0 | ALU register ops (ADD, SUB, AND, OR, XOR) |
| 0x1 | ALU immediate (ADDI, ANDI, ORI) |
| 0x2 | Shift/Rotate |
| 0x3 | Multiply/Divide |
| 0x4 | Load word |
| 0x5 | Load byte |
| 0x6 | Store word |
| 0x7 | Store byte |
| 0x8 | Branch |
| 0x9 | Jump/Call |
| 0xA | Stack (PUSH/POP) |
| 0xB | Block operations |
| 0xC | I/O (IN/OUT) |
| 0xD | System |
| 0xE | Compare/Test |
| 0xF | Extended (32-bit) |

---

## Next Steps

1. Finalize instruction encoding (bit-level)
2. Write assembler (likely in Rust or Python)
3. Build emulator/simulator
4. Create test suite
5. Port simple Z80 programs
6. Consider FPGA implementation
