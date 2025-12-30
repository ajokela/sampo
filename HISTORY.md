# Sampo Architecture: Lineage and Design Philosophy

## Core Lineage: RISC Foundation

Sampo draws heavily from the **RISC-V** and **MIPS** traditions:

**From RISC-V:**
- The zero register convention (R0 always reads as 0)
- Register naming and calling conventions (RA, SP, GP, argument registers A0-A3, temporaries T0-T3, saved registers S0-S3)
- Clean separation between computation and memory access (load/store architecture)
- Flag-setting compare instruction separate from arithmetic

**From MIPS:**
- Simple, orthogonal instruction formats (R-type for register-register, I-type for immediates)
- 4-bit opcode field allowing 16 primary instruction categories
- PC-relative branching with signed offsets

**From ARM Thumb/Thumb-2:**
- 16-bit base instruction width for code density
- Optional 32-bit extended forms for operations needing larger immediates
- The 0xF prefix mechanism for extended instructions echoes Thumb-2's approach

## Secondary Lineage: Z80 Compatibility Layer

The architecture includes deliberate Z80-isms to support porting retro computing workloads:

| Feature | Z80 Origin | Sampo Implementation |
|---------|-----------|---------------------|
| Port I/O | IN A,(n) / OUT (n),A | INI Rd, imm8 / OUTI imm8, Rs |
| Alternate registers | EXX swaps BC,DE,HL | EXX swaps R4-R11 |
| Block copy | LDIR, LDDR | LDIR, LDDR using R4/R5/R6 convention |
| BCD arithmetic | DAA | DAA Rd |
| 64KB address space | 16-bit addresses | 16-bit addresses |

## What Makes Sampo Unique

### 1. Hybrid Width Encoding

Unlike pure 32-bit RISC (MIPS, RISC-V RV32) or variable-length CISC (x86, Z80), Sampo uses a disciplined two-tier system:
- All instructions are exactly 16 or 32 bits
- 32-bit forms always start with 0xF prefix
- No complex length decoding — just check the first nibble

### 2. Targeted Register Alternates

Z80's EXX swaps all main registers. Sampo is selective:
- Only R4-R11 have shadow copies (arguments + temporaries)
- R0-R3 (zero, RA, SP, GP) and R12-R15 (saved) are never swapped
- This allows interrupt handlers to use alternate registers without corrupting the call stack

### 3. RISC Regularity with CISC Convenience

The block operations (LDIR, LDDR, FILL) are decidedly un-RISC — they're multi-cycle instructions that modify multiple registers. But they're implemented with fixed register conventions (R4=count, R5=source, R6=dest), preserving predictability while enabling efficient memory operations critical for interpreter workloads.

### 4. Code Density vs. Simplicity Tradeoff

Sampo prioritizes code density over decode simplicity:
- 16-bit instructions for common operations fit twice as many in cache/ROM
- The 8-bit immediate field handles most constants (-128 to 127, or 0-255 unsigned)
- Extended forms exist but aren't needed for typical code

## Architectural Comparison

| Aspect | Z80 | MIPS | RISC-V | Sampo |
|--------|-----|------|--------|-------|
| Word size | 8-bit | 32-bit | 32/64-bit | 16-bit |
| Instruction width | 1-4 bytes | 4 bytes | 2/4 bytes | 2/4 bytes |
| Registers | 8 + alternates | 32 | 32 | 16 + alternates |
| Zero register | No | $zero | x0 | R0 |
| I/O model | Port-based | Memory-mapped | Memory-mapped | Port-based |
| Block ops | Yes (LDIR, etc.) | No | No | Yes |
| Instruction count | ~300+ | ~60 | ~50 base | ~66 |

## Design Philosophy

Sampo occupies an intentional middle ground:

```
Z80 ←————————————————————————————————————→ RISC-V
CISC                                        RISC
Variable encoding                           Fixed encoding
Many addressing modes                       Load/store only
Implicit registers                          Orthogonal registers

                        Sampo
                    ↓
            • Fixed 16/32-bit encoding
            • Load/store + block ops
            • Orthogonal registers + alternates
            • Port I/O + memory-mapped option
```

It's a "Z80 programmer's RISC" — familiar enough for 8-bit veterans to understand, clean enough to implement in simple hardware or an emulator, and efficient enough to run interpreters and compilers that were originally targeting Z80.

## Lineage Summary

```
RISC-V ──────┬──→ Register conventions, zero register, load/store
             │
MIPS ────────┼──→ Instruction format regularity, opcode structure
             │
ARM Thumb ───┼──→ 16-bit base encoding, 32-bit extensions
             │
Z80 ─────────┴──→ Port I/O, alternate registers, block ops, BCD, 64KB

             ↓
           Sampo
```
