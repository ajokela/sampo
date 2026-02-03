#!/usr/bin/env python3
"""Convert binary file to Verilog hex format for $readmemh"""

import sys

def main():
    if len(sys.argv) < 3:
        print("Usage: bin2hex.py <input.bin> <output.hex>")
        sys.exit(1)

    input_file = sys.argv[1]
    output_file = sys.argv[2]

    with open(input_file, 'rb') as f:
        data = f.read()

    # Pad to even length if necessary
    if len(data) % 2:
        data += b'\x00'

    # Convert to 16-bit words (little-endian)
    words = []
    for i in range(0, len(data), 2):
        # Little-endian: low byte first
        word = data[i] | (data[i+1] << 8)
        words.append(word)

    # Write hex file
    # Format: @address for starting address, then hex words
    with open(output_file, 'w') as f:
        # Start at word address 0x00 (binary already includes proper padding)
        f.write("@0000\n")
        for i, word in enumerate(words):
            f.write(f"{word:04X}\n")

    print(f"Converted {len(data)} bytes to {len(words)} words")

if __name__ == '__main__':
    main()
