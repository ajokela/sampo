# FPGA Implementation Guide

This document covers implementing the Sampo CPU on inexpensive FPGA hardware.

## Recommended Hardware

For a 16-bit RISC CPU like Sampo (~66 instructions, 16 registers, 64KB address space), the resource requirements are modest.

### Budget FPGA Boards

| Board | FPGA | LUTs | Price | Notes |
|-------|------|------|-------|-------|
| **Tang Nano 9K** | Gowin GW1NR-9 | 8,640 | ~$15 | Best value, includes 64Mbit PSRAM |
| **Tang Nano 20K** | Gowin GW2A | 20,736 | ~$25 | More headroom for peripherals |
| **UPduino v3.1** | Lattice iCE40UP5K | 5,280 | ~$20 | Great open-source toolchain |
| **ICEBreaker** | Lattice iCE40UP5K | 5,280 | ~$70 | Best open-source experience, PMODs |

### Resource Estimate for Sampo

| Component | LUTs (estimated) |
|-----------|------------------|
| 16 × 16-bit registers | ~256 FFs |
| ALU (16-bit) | ~200-400 |
| Control logic | ~500-1000 |
| Instruction decode | ~300-500 |
| **Total** | **~1500-2500** |

All recommended boards have sufficient capacity with room for peripherals (UART, SPI, timers).

### Top Pick: Tang Nano 9K (~$15)

- 8,640 LUTs (plenty for Sampo + peripherals)
- 64Mbit PSRAM (can map as the 64KB address space)
- HDMI output (for video terminal)
- USB-C programming
- Supported by open-source toolchain (Yosys + nextpnr-gowin)

## HDL Options

### Amaranth HDL (Python) - Recommended

Amaranth is a Python-based HDL that generates standard Verilog. It offers excellent readability and strong type checking.

```python
from amaranth import *

class SampoALU(Elaboratable):
    def __init__(self):
        self.a = Signal(16)
        self.b = Signal(16)
        self.op = Signal(4)
        self.result = Signal(16)
        self.flags = Signal(4)  # N, Z, C, V

    def elaborate(self, platform):
        m = Module()

        with m.Switch(self.op):
            with m.Case(0x0):  # ADD
                m.d.comb += self.result.eq(self.a + self.b)
            with m.Case(0x1):  # SUB
                m.d.comb += self.result.eq(self.a - self.b)
            with m.Case(0x2):  # AND
                m.d.comb += self.result.eq(self.a & self.b)

        # Flags
        m.d.comb += self.flags[3].eq(self.result[15])  # N
        m.d.comb += self.flags[2].eq(self.result == 0)  # Z

        return m
```

**Advantages:**
- Python syntax is expressive and readable
- Declarative style maps well to hardware thinking
- Generates Verilog for any toolchain
- Built-in simulation with Python testbenches
- Strong community around open-source FPGAs

### Verilog - Most Portable

Standard Verilog works with any FPGA toolchain.

```verilog
module sampo_alu (
    input  [15:0] a, b,
    input  [3:0]  op,
    output reg [15:0] result,
    output [3:0] flags
);
    always @(*) begin
        case (op)
            4'h0: result = a + b;  // ADD
            4'h1: result = a - b;  // SUB
            4'h2: result = a & b;  // AND
            default: result = 16'h0;
        endcase
    end

    assign flags[3] = result[15];      // N
    assign flags[2] = (result == 0);   // Z
endmodule
```

**Advantages:**
- Universal - works everywhere
- Direct control over hardware
- No build dependencies beyond the FPGA toolchain

### SpinalHDL (Scala)

SpinalHDL offers strong typing and functional programming features.

```scala
class SampoALU extends Component {
  val io = new Bundle {
    val a, b = in UInt(16 bits)
    val op = in UInt(4 bits)
    val result = out UInt(16 bits)
    val flags = out Bits(4 bits)
  }

  io.result := io.op.mux(
    0 -> (io.a + io.b),
    1 -> (io.a - io.b),
    2 -> (io.a & io.b),
    default -> U(0)
  )

  io.flags(3) := io.result.msb      // N
  io.flags(2) := (io.result === 0)  // Z
}
```

### Comparison

| Factor | Amaranth | Verilog | SpinalHDL |
|--------|----------|---------|-----------|
| Readability | Excellent | Medium | Good |
| Error catching | Excellent | Poor | Good |
| Toolchain | Yosys | Any | Yosys |
| Simulation | Python | Verilator | Scala |
| Learning curve | Low | Medium | Higher |

## Open-Source Toolchain

All recommended boards work with the fully open-source FPGA toolchain:

- **Yosys** - Synthesis
- **nextpnr** - Place & route (nextpnr-ice40, nextpnr-gowin, nextpnr-ecp5)
- **Project IceStorm** - iCE40 bitstream generation
- **Apicula** - Gowin bitstream generation
- **Project Trellis** - ECP5 bitstream generation

### Installation

```bash
# Amaranth
pip install amaranth amaranth-boards

# Open-source FPGA tools (macOS)
brew install yosys nextpnr-ice40 nextpnr-gowin icestorm

# Or from source / package manager on Linux
```

### Build Flow for Tang Nano 9K

```bash
# Generate Verilog from Amaranth
python -m amaranth generate sampo.py > sampo.v

# Synthesize
yosys -p "read_verilog sampo.v; synth_gowin -top sampo_top -json sampo.json"

# Place and route
nextpnr-gowin --json sampo.json --write sampo.fs --device GW1NR-LV9QN88PC6/I5

# Program the board
openFPGALoader -b tangnano9k sampo.fs
```

### Build Flow for iCE40 (UPduino, ICEBreaker)

```bash
# Synthesize
yosys -p "read_verilog sampo.v; synth_ice40 -top sampo_top -json sampo.json"

# Place and route
nextpnr-ice40 --up5k --json sampo.json --asc sampo.asc --pcf pins.pcf

# Generate bitstream
icepack sampo.asc sampo.bin

# Program
iceprog sampo.bin
```

## Peripheral Integration

### UART (Serial I/O)

For compatibility with the emulator's MC6850 ACIA emulation:

- Port 0x80: Status register (bit 1 = TX ready, bit 0 = RX ready)
- Port 0x81: Data register (read/write)

A simple UART at 115200 baud requires ~200-300 additional LUTs.

### Memory

**Option 1: Block RAM (internal)**
- Tang Nano 9K has 46KB of block RAM
- Sufficient for 32KB ROM + some RAM

**Option 2: PSRAM (external)**
- Tang Nano 9K includes 64Mbit (8MB) PSRAM
- Requires SPI/QPI controller (~500 LUTs)
- Can provide full 64KB address space + more

### Clock

- Tang Nano 9K has 27MHz oscillator
- Use PLL to generate desired CPU clock (e.g., 10MHz for ~10 MIPS)

## References

- [Amaranth HDL Documentation](https://amaranth-lang.org/)
- [Tang Nano 9K Wiki](https://wiki.sipeed.com/hardware/en/tang/Tang-Nano-9K/Nano-9K.html)
- [Project IceStorm](https://clifford.at/icestorm/)
- [Apicula (Gowin tools)](https://github.com/YosysHQ/apicula)
- [nextpnr](https://github.com/YosysHQ/nextpnr)
