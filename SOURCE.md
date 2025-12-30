# FPGA Board Sourcing

Where to buy FPGA development boards for Sampo.

## Recommended Boards

| Board | Price | LUTs | RAM | Best For |
|-------|-------|------|-----|----------|
| Tang Nano 9K | ~$15 | 8,640 | 64Mbit PSRAM | Budget, best value |
| Tang Nano 20K | ~$25 | 20,736 | 64Mbit PSRAM | More headroom |
| Alchitry Cu | ~$60 | 7,680 | 128KB BRAM | iCE40 + good docs |
| ICEBreaker | ~$80 | 5,280 | 128KB BRAM | Best open-source experience |

Sampo requires ~2,500 LUTs - all of these boards have sufficient capacity.

## ICEBreaker

The ICEBreaker is designed specifically for the open-source FPGA toolchain (Yosys, nextpnr, Amaranth).

**Price:** $79.95 USD

**Where to buy:**
- [1BitSquared US](https://1bitsquared.com/products/icebreaker) - Direct from maker
- [1BitSquared EU](https://1bitsquared.de/products/icebreaker) - European store
- [Mouser](https://www.mouser.com/new/1bitsquared/1bitsquared-icebreaker-fpga-dev-boards/) - Large distributor
- [Crowd Supply](https://www.crowdsupply.com/1bitsquared/icebreaker-fpga) - May have backorder delays

**Features:**
- Lattice iCE40 UP5K FPGA (5,280 LUTs)
- Breakaway PMOD with 3 buttons + 5 LEDs
- USB programming built-in
- Two PMOD connectors for expansion
- Works out-of-box with open-source tools

**Smaller version:** [ICEBreaker Bitsy](https://1bitsquared.com/products/icebreaker-bitsy) - Teensy form factor, breadboard compatible

## Tang Nano 9K

Best value option with onboard PSRAM.

**Price:** ~$15 USD

**Where to buy:**
- [Sipeed Official Store (AliExpress)](https://www.aliexpress.com/item/1005003624073682.html)
- [Amazon](https://www.amazon.com/s?k=tang+nano+9k)

**Features:**
- Gowin GW1NR-9 FPGA (8,640 LUTs)
- 64Mbit PSRAM onboard
- HDMI output
- USB-C programming
- Supported by open-source toolchain (Yosys + Apicula)

## Tang Nano 20K

More capacity for complex designs.

**Price:** ~$25 USD

**Where to buy:**
- [Sipeed Official Store (AliExpress)](https://www.aliexpress.com/item/1005005581256682.html)
- [Amazon](https://www.amazon.com/s?k=tang+nano+20k)

**Features:**
- Gowin GW2A FPGA (20,736 LUTs)
- 64Mbit PSRAM onboard
- HDMI output
- USB-C programming

## Alchitry Cu

Good middle-ground with excellent documentation.

**Price:** ~$60 USD

**Where to buy:**
- [SparkFun](https://www.sparkfun.com/products/16526)
- [Digi-Key](https://www.digikey.com/en/products/detail/sparkfun-electronics/DEV-16526/11506ul)

**Features:**
- Lattice iCE40 HX8K FPGA (7,680 LUTs)
- 128KB block RAM
- Alchitry tooling and tutorials
- Breadboard-friendly headers

## Recommendation

| Budget | Recommendation |
|--------|----------------|
| Cheapest | Tang Nano 9K (~$15) |
| Best value | Tang Nano 9K or 20K |
| Best iCE40 experience | ICEBreaker (~$80) |
| Good docs + iCE40 | Alchitry Cu (~$60) |

For Sampo development, the **Tang Nano 9K** offers the best value - more LUTs than the ICEBreaker at 1/5 the price, plus onboard PSRAM for the full 64KB address space without using block RAM.
