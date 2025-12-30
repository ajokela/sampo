#!/usr/bin/env python3
"""Sampo CPU testbench.

Run with: python -m rtl.test_cpu
"""

import sys
from pathlib import Path

from amaranth import *
from amaranth.sim import Simulator

from .soc import SampoSoC


def load_binary(filename):
    """Load a binary file as a list of bytes."""
    with open(filename, 'rb') as f:
        return list(f.read())


def test_hello_world():
    """Test the hello world program."""

    # Try to load the hello.bin program
    bin_path = Path(__file__).parent.parent / "examples" / "hello.bin"

    if not bin_path.exists():
        print(f"Binary not found: {bin_path}")
        print("Assemble with: ./sasm/target/release/sasm examples/hello.s -o examples/hello.bin")
        return False

    program = load_binary(bin_path)
    print(f"Loaded {len(program)} bytes from {bin_path}")

    # Create SoC
    soc = SampoSoC(program=program, reset_vector=0x0100)

    # Collected output
    output = []

    def testbench():
        # Always ready to receive TX
        yield soc.tx_ready.eq(1)

        # Run for up to 10000 cycles
        for cycle in range(10000):
            # Check for TX output
            if (yield soc.tx_valid):
                char = yield soc.tx_data
                output.append(chr(char))
                print(chr(char), end='', flush=True)

            # Check if halted
            if (yield soc.halted):
                print(f"\n\nCPU halted after {cycle} cycles")
                print(f"PC: 0x{(yield soc.pc):04X}")
                break

            yield

        return True

    sim = Simulator(soc)
    sim.add_clock(1e-6)  # 1 MHz clock
    sim.add_testbench(testbench)

    with sim.write_vcd("sampo_test.vcd"):
        sim.run()

    result = ''.join(output)
    print(f"\nOutput: {repr(result)}")

    expected = "Hello, Sampo!\n"
    if result == expected:
        print("TEST PASSED!")
        return True
    else:
        print(f"TEST FAILED! Expected: {repr(expected)}")
        return False


def test_alu():
    """Test ALU operations."""
    from .alu import ALU, ALUOp

    alu = ALU()

    def testbench():
        # Test ADD
        yield alu.a.eq(100)
        yield alu.b.eq(50)
        yield alu.op.eq(ALUOp.ADD)
        yield
        result = yield alu.result
        assert result == 150, f"ADD failed: {result} != 150"
        print(f"ADD: 100 + 50 = {result}")

        # Test SUB
        yield alu.a.eq(100)
        yield alu.b.eq(30)
        yield alu.op.eq(ALUOp.SUB)
        yield
        result = yield alu.result
        assert result == 70, f"SUB failed: {result} != 70"
        print(f"SUB: 100 - 30 = {result}")

        # Test AND
        yield alu.a.eq(0xFF00)
        yield alu.b.eq(0x0FF0)
        yield alu.op.eq(ALUOp.AND)
        yield
        result = yield alu.result
        assert result == 0x0F00, f"AND failed: 0x{result:04X} != 0x0F00"
        print(f"AND: 0xFF00 & 0x0FF0 = 0x{result:04X}")

        # Test OR
        yield alu.a.eq(0xFF00)
        yield alu.b.eq(0x00FF)
        yield alu.op.eq(ALUOp.OR)
        yield
        result = yield alu.result
        assert result == 0xFFFF, f"OR failed: 0x{result:04X} != 0xFFFF"
        print(f"OR: 0xFF00 | 0x00FF = 0x{result:04X}")

        # Test zero flag
        yield alu.a.eq(50)
        yield alu.b.eq(50)
        yield alu.op.eq(ALUOp.SUB)
        yield
        result = yield alu.result
        zero = yield alu.flag_z
        assert result == 0, f"Zero result failed: {result} != 0"
        assert zero == 1, f"Zero flag not set"
        print(f"SUB: 50 - 50 = {result}, Z={zero}")

        # Test negative flag
        yield alu.a.eq(0)
        yield alu.b.eq(1)
        yield alu.op.eq(ALUOp.SUB)
        yield
        result = yield alu.result
        neg = yield alu.flag_n
        assert result == 0xFFFF, f"Negative result failed: 0x{result:04X}"
        assert neg == 1, f"Negative flag not set"
        print(f"SUB: 0 - 1 = 0x{result:04X}, N={neg}")

        print("\nALU tests passed!")

    sim = Simulator(alu)
    sim.add_testbench(testbench)
    sim.run()

    return True


def test_regfile():
    """Test register file."""
    from .regfile import RegisterFile

    rf = RegisterFile()

    def testbench():
        # Write to R4
        yield rf.wr_addr.eq(4)
        yield rf.wr_data.eq(0x1234)
        yield rf.wr_en.eq(1)
        yield
        yield rf.wr_en.eq(0)
        yield

        # Read from R4
        yield rf.rd_addr1.eq(4)
        yield
        data = yield rf.rd_data1
        assert data == 0x1234, f"R4 read failed: 0x{data:04X} != 0x1234"
        print(f"R4 = 0x{data:04X}")

        # Write to R0 (should be ignored)
        yield rf.wr_addr.eq(0)
        yield rf.wr_data.eq(0xFFFF)
        yield rf.wr_en.eq(1)
        yield
        yield rf.wr_en.eq(0)
        yield

        # Read from R0 (should be 0)
        yield rf.rd_addr1.eq(0)
        yield
        data = yield rf.rd_data1
        assert data == 0, f"R0 read failed: 0x{data:04X} != 0x0000"
        print(f"R0 = 0x{data:04X} (writes ignored)")

        print("\nRegister file tests passed!")

    sim = Simulator(rf)
    sim.add_clock(1e-6)
    sim.add_testbench(testbench)
    sim.run()

    return True


def main():
    print("=" * 60)
    print("Sampo CPU Tests")
    print("=" * 60)

    tests = [
        ("ALU", test_alu),
        ("Register File", test_regfile),
        ("Hello World", test_hello_world),
    ]

    results = []
    for name, test_fn in tests:
        print(f"\n{'=' * 60}")
        print(f"Running: {name}")
        print("=" * 60)
        try:
            passed = test_fn()
            results.append((name, passed))
        except Exception as e:
            print(f"ERROR: {e}")
            import traceback
            traceback.print_exc()
            results.append((name, False))

    print(f"\n{'=' * 60}")
    print("Results:")
    print("=" * 60)
    all_passed = True
    for name, passed in results:
        status = "PASS" if passed else "FAIL"
        print(f"  {name}: {status}")
        if not passed:
            all_passed = False

    return 0 if all_passed else 1


if __name__ == "__main__":
    sys.exit(main())
