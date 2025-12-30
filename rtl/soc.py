"""Sampo System-on-Chip.

Top-level module integrating:
- Sampo CPU
- RAM (64KB)
- UART (MC6850-compatible)
"""

from amaranth import *

from .cpu import SampoCPU


class RAM(Elaboratable):
    """Simple synchronous RAM.

    Single-port RAM with byte enables.
    """

    def __init__(self, size=65536, init=None):
        self.size = size
        self.init = init or []

        self.addr = Signal(16)
        self.rdata = Signal(16)
        self.wdata = Signal(16)
        self.we = Signal()
        self.be = Signal(2)  # Byte enables
        self.valid = Signal()
        self.ready = Signal()

    def elaborate(self, platform):
        m = Module()

        # Memory array (as 16-bit words)
        mem = Memory(width=16, depth=self.size // 2, init=self._init_words())
        m.submodules.rd = rd = mem.read_port()
        m.submodules.wr = wr = mem.write_port(granularity=8)

        # Word address (drop bit 0)
        word_addr = self.addr[1:16]

        m.d.comb += [
            rd.addr.eq(word_addr),
            self.rdata.eq(rd.data),

            wr.addr.eq(word_addr),
            wr.data.eq(self.wdata),
            wr.en[0].eq(self.we & self.be[0]),
            wr.en[1].eq(self.we & self.be[1]),
        ]

        # Single-cycle ready
        m.d.sync += self.ready.eq(self.valid)

        return m

    def _init_words(self):
        """Convert byte init data to 16-bit words."""
        words = []
        for i in range(0, len(self.init), 2):
            lo = self.init[i] if i < len(self.init) else 0
            hi = self.init[i+1] if i+1 < len(self.init) else 0
            words.append(lo | (hi << 8))
        # Pad to full size
        while len(words) < self.size // 2:
            words.append(0)
        return words


class UART(Elaboratable):
    """Simple UART with MC6850-compatible registers.

    Port 0x80: Status register
        Bit 0: RX data ready
        Bit 1: TX ready (always 1 for now)
        Bit 2: DCD (always 0)
        Bit 3: CTS (always 0)
        Bit 7: IRQ (not implemented)

    Port 0x81: Data register
        Read: RX data
        Write: TX data
    """

    def __init__(self):
        # I/O interface
        self.addr = Signal(8)
        self.rdata = Signal(8)
        self.wdata = Signal(8)
        self.rd = Signal()
        self.wr = Signal()

        # External serial interface
        self.tx_data = Signal(8)
        self.tx_valid = Signal()
        self.tx_ready = Signal()

        self.rx_data = Signal(8)
        self.rx_valid = Signal()
        self.rx_ready = Signal()

    def elaborate(self, platform):
        m = Module()

        # TX buffer
        tx_buf = Signal(8)
        tx_pending = Signal()

        # RX buffer
        rx_buf = Signal(8)
        rx_ready_int = Signal()

        # Status register
        status = Signal(8)
        m.d.comb += [
            status[0].eq(rx_ready_int),  # RX ready
            status[1].eq(~tx_pending),    # TX ready
        ]

        # Read handling
        with m.If(self.rd):
            with m.Switch(self.addr):
                with m.Case(0x80):  # Status
                    m.d.comb += self.rdata.eq(status)
                with m.Case(0x81):  # Data
                    m.d.comb += self.rdata.eq(rx_buf)
                    m.d.sync += rx_ready_int.eq(0)
                with m.Default():
                    m.d.comb += self.rdata.eq(0)

        # Write handling
        with m.If(self.wr):
            with m.Switch(self.addr):
                with m.Case(0x81):  # Data
                    m.d.sync += [
                        tx_buf.eq(self.wdata),
                        tx_pending.eq(1),
                    ]

        # TX output
        m.d.comb += [
            self.tx_data.eq(tx_buf),
            self.tx_valid.eq(tx_pending),
        ]
        with m.If(self.tx_ready & tx_pending):
            m.d.sync += tx_pending.eq(0)

        # RX input
        m.d.comb += self.rx_ready.eq(~rx_ready_int)
        with m.If(self.rx_valid & ~rx_ready_int):
            m.d.sync += [
                rx_buf.eq(self.rx_data),
                rx_ready_int.eq(1),
            ]

        return m


class SampoSoC(Elaboratable):
    """Complete Sampo System-on-Chip.

    Integrates CPU, RAM, and UART.
    """

    def __init__(self, program=None, reset_vector=0x0100):
        self.program = program or []
        self.reset_vector = reset_vector

        # External UART interface
        self.tx_data = Signal(8)
        self.tx_valid = Signal()
        self.tx_ready = Signal()

        self.rx_data = Signal(8)
        self.rx_valid = Signal()
        self.rx_ready = Signal()

        # Debug
        self.halted = Signal()
        self.pc = Signal(16)
        self.cycles = Signal(32)

    def elaborate(self, platform):
        m = Module()

        # Instantiate components
        m.submodules.cpu = cpu = SampoCPU(reset_vector=self.reset_vector)
        m.submodules.ram = ram = RAM(size=65536, init=self.program)
        m.submodules.uart = uart = UART()

        # Connect CPU to RAM
        m.d.comb += [
            ram.addr.eq(cpu.mem_addr),
            ram.wdata.eq(cpu.mem_wdata),
            ram.we.eq(cpu.mem_we),
            ram.be.eq(cpu.mem_be),
            ram.valid.eq(cpu.mem_valid),
            cpu.mem_rdata.eq(ram.rdata),
            cpu.mem_ready.eq(ram.ready),
        ]

        # Connect CPU to UART
        m.d.comb += [
            uart.addr.eq(cpu.io_addr),
            uart.wdata.eq(cpu.io_wdata),
            uart.rd.eq(cpu.io_rd),
            uart.wr.eq(cpu.io_wr),
            cpu.io_rdata.eq(uart.rdata),
        ]

        # Connect UART to external interface
        m.d.comb += [
            self.tx_data.eq(uart.tx_data),
            self.tx_valid.eq(uart.tx_valid),
            uart.tx_ready.eq(self.tx_ready),

            uart.rx_data.eq(self.rx_data),
            uart.rx_valid.eq(self.rx_valid),
            self.rx_ready.eq(uart.rx_ready),
        ]

        # Debug outputs
        m.d.comb += [
            self.halted.eq(cpu.halted),
            self.pc.eq(cpu.pc),
            self.cycles.eq(cpu.cycles),
        ]

        return m
