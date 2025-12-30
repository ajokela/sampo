# Sampo

A 16-bit RISC CPU architecture with assembler and emulator.

## About the Name

The **Sampo** is a magical artifact from Finnish mythology, central to the epic poem *Kalevala* compiled by Elias Lönnrot in 1835. According to legend, the Sampo was forged by Ilmarinen, a legendary blacksmith and sky god, from a swan's feather, a grain of barley, a ball of wool, a drop of milk, and a shaft of a distaff.

The Sampo took the form of a magical mill that could produce flour, salt, and gold endlessly — bringing riches and good fortune to its holder, much like the Greek cornucopia or the Nordic mill Grótti. The Sampo became the object of a great quest and battle between the heroes of Kalevala and Louhi, the witch queen of Pohjola.

The exact nature of the Sampo has been debated by scholars since 1818, with over 30 theories proposed — ranging from a world pillar, to an astrolabe, to a decorated shield. This mystery makes it a fitting namesake for a CPU architecture: something that transforms simple inputs into useful outputs, whose inner workings invite exploration.

## Architecture Overview

Sampo is a 16-bit RISC processor designed to support workloads from Z80-based retro computing ecosystems.

**Key Features:**
- 16 general-purpose 16-bit registers (R0-R15)
- 64KB address space
- Fixed 16-bit instructions with 32-bit extended forms
- Port-based I/O (256 ports, Z80 compatible)
- Alternate register set for fast context switching (EXX)
- Hardware multiply/divide
- Block operations (LDIR, LDDR, FILL)
- BCD support (DAA)

**Register Conventions:**
| Register | Alias | Purpose |
|----------|-------|---------|
| R0 | ZERO | Always zero |
| R1 | RA | Return address |
| R2 | SP | Stack pointer |
| R3 | GP | Global pointer |
| R4-R7 | A0-A3 | Arguments/returns |
| R8-R11 | T0-T3 | Temporaries |
| R12-R15 | S0-S3 | Saved registers |

See [ARCHITECTURE.md](ARCHITECTURE.md) for full specification and [ENCODING.md](ENCODING.md) for instruction encoding details.

## Project Structure

```
sampo/
├── ARCHITECTURE.md    # CPU architecture specification
├── ENCODING.md        # Instruction encoding details
├── sasm/              # Sampo Assembler
│   └── src/
│       ├── main.rs    # CLI entry point
│       ├── lexer.rs   # Tokenizer
│       ├── parser.rs  # Parser
│       └── codegen.rs # Code generator
├── semu/              # Sampo Emulator
│   └── src/
│       ├── main.rs    # CLI entry point
│       └── cpu.rs     # CPU emulation core
└── examples/          # Example programs
    └── hello.s        # Hello world
```

## Building

Both tools are written in Rust. Build with Cargo:

```bash
# Build the assembler
cd sasm && cargo build --release

# Build the emulator
cd semu && cargo build --release
```

## Usage

### Assembler (sasm)

```bash
# Assemble a program
sasm input.s -o output.bin

# Options
sasm input.s -o output.bin -v    # Verbose output
sasm --help                       # Show help
```

### Emulator (semu)

```bash
# Run a program
semu program.bin

# Options
semu program.bin -t              # Trace execution
semu program.bin -i              # Interactive debugger
semu --help                      # Show help
```

**Interactive Debugger Commands:**
- `s`, `step` - Execute one instruction
- `r`, `run` - Run until halt
- `d`, `dump` - Dump CPU state
- `m`, `mem` - Dump memory at PC
- `q`, `quit` - Exit
- `h`, `help` - Show commands

## Example

```asm
; Hello World for Sampo CPU
        .org 0x0100

.equ    ACIA_STATUS 0x80
.equ    ACIA_DATA   0x81

start:
        LIX  R4, message        ; Load address of message

loop:
        LBU  R5, (R4)           ; Load byte from string
        CMP  R5, R0             ; Compare with zero
        BEQ  done               ; If null, we're done

wait_tx:
        INI  R6, ACIA_STATUS    ; Read serial status
        AND  R7, R6, R6         ; Copy to R7
        ADDI R7, -2             ; Check TX ready
        BNE  wait_tx            ; Wait if not ready

        OUTI ACIA_DATA, R5      ; Write character
        ADDI R4, 1              ; Next character
        J    loop

done:
        HALT

message:
        .asciz "Hello, Sampo!\n"
```

Assemble and run:

```bash
sasm examples/hello.s -o hello.bin
semu hello.bin
```

Output:
```
Sampo Emulator - Loaded 301 bytes
Starting execution at 0x0100

Hello, Sampo!

CPU halted at 0x011E
```

## Instruction Set Summary

| Category | Instructions |
|----------|-------------|
| Arithmetic | ADD, SUB, ADDI, MUL, DIV, CMP, NEG, DAA |
| Logic | AND, OR, XOR, NOT |
| Shift | SLL, SRL, SRA |
| Load/Store | LW, LB, LBU, SW, SB, LUI |
| Branch | BEQ, BNE, BLT, BGE, BLTU, BGEU |
| Jump | J, JAL, JR, JALR |
| Stack | PUSH, POP |
| Block | LDIR, LDDR, FILL |
| I/O | IN, OUT, INI, OUTI |
| System | NOP, HALT, EI, DI, EXX, RETI |

Extended 32-bit forms (LIX, JX, etc.) allow full 16-bit immediates.

## License

BSD 3-Clause License. See [LICENSE](LICENSE) for details.

## References

- [Sampo - Wikipedia](https://en.wikipedia.org/wiki/Sampo)
- [The Magical Sampo in Finnish Folklore](https://www.ancient-origins.net/myths-legends-europe/magical-sampo-object-power-and-riches-finnish-folklore-002891)
- [Finnish Mythology and The Kalevala](https://www.routesnorth.com/finland/finnish-mythology-and-the-kalevala-the-complete-guide/)
