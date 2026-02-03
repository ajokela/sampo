// Sampo CPU - Main System Testbench
// Runs a test program and captures UART output

`timescale 1ns/1ps

`include "sampo_pkg.vh"

module sampo_tb;

    // Parameters
    parameter CLK_PERIOD = 10;  // 100 MHz
    parameter MAX_CYCLES = 100000;
    parameter RAM_INIT_FILE = "../programs/hello.hex";

    // Signals
    reg         clk;
    reg         rst_n;
    wire [7:0]  tx_data;
    wire        tx_valid;
    reg         tx_ready;
    reg  [7:0]  rx_data;
    reg         rx_valid;
    wire        rx_ready;
    wire        halted;
    wire [15:0] pc;
    wire [31:0] cycles;

    // UART output capture
    reg [7:0]  uart_buffer [0:1023];
    integer    uart_idx;

    // Instantiate SoC
    soc #(
        .RESET_VECTOR(16'h0100),
        .RAM_INIT_FILE(RAM_INIT_FILE)
    ) uut (
        .clk(clk),
        .rst_n(rst_n),
        .tx_data(tx_data),
        .tx_valid(tx_valid),
        .tx_ready(tx_ready),
        .rx_data(rx_data),
        .rx_valid(rx_valid),
        .rx_ready(rx_ready),
        .halted(halted),
        .pc(pc),
        .cycles(cycles)
    );

    // Clock generation
    initial clk = 0;
    always #(CLK_PERIOD/2) clk = ~clk;

    // UART TX capture (always ready to receive)
    initial begin
        tx_ready = 1;
        uart_idx = 0;
        rx_data = 8'h00;
        rx_valid = 0;
    end

    always @(posedge clk) begin
        if (tx_valid && tx_ready) begin
            uart_buffer[uart_idx] = tx_data;
            uart_idx = uart_idx + 1;
            // Display character immediately
            if (tx_data >= 8'h20 && tx_data < 8'h7F) begin
                $write("%c", tx_data);
            end else if (tx_data == 8'h0A) begin
                $write("\n");
            end else if (tx_data == 8'h0D) begin
                // Ignore CR
            end else begin
                $write("[%02X]", tx_data);
            end
            $fflush();
        end
    end

    // Main test sequence
    initial begin
        $display("=== Sampo CPU Testbench ===");
        $display("RAM init file: %s", RAM_INIT_FILE);
        $display("");

        // Initialize
        rst_n = 0;
        #100;
        rst_n = 1;

        $display("CPU started at PC=0x%04X", pc);
        $display("UART output:");
        $display("----------------------------------------");

        // Wait for halt or timeout
        fork
            begin : timeout_block
                #(CLK_PERIOD * MAX_CYCLES);
                $display("");
                $display("----------------------------------------");
                $display("TIMEOUT: Max cycles (%d) reached", MAX_CYCLES);
                disable wait_halt;
            end

            begin : wait_halt
                wait(halted);
                #10;  // Small delay
                disable timeout_block;
            end
        join

        $display("");
        $display("----------------------------------------");
        $display("");
        $display("Simulation complete:");
        $display("  Final PC:    0x%04X", pc);
        $display("  Cycles:      %d", cycles);
        $display("  UART chars:  %d", uart_idx);
        $display("  Status:      %s", halted ? "HALTED" : "RUNNING");

        // Dump UART buffer as string
        if (uart_idx > 0) begin
            $display("");
            $display("Full UART output:");
            $write("  \"");
            for (integer i = 0; i < uart_idx; i = i + 1) begin
                if (uart_buffer[i] >= 8'h20 && uart_buffer[i] < 8'h7F) begin
                    $write("%c", uart_buffer[i]);
                end else if (uart_buffer[i] == 8'h0A) begin
                    $write("\\n");
                end else if (uart_buffer[i] == 8'h0D) begin
                    $write("\\r");
                end else begin
                    $write("\\x%02X", uart_buffer[i]);
                end
            end
            $display("\"");
        end

        $finish;
    end

    // Optional: Dump waveforms
    initial begin
        $dumpfile("sampo.vcd");
        $dumpvars(0, sampo_tb);
    end

    // Debug: Print state changes
    always @(posedge clk) begin
        if (rst_n && cycles < 100) begin
            $display("Cycle %5d: PC=%04X St=%d Ins=%04X R4=%04X R5=%04X Flags=%02X",
                cycles, pc, uut.cpu_inst.state, uut.cpu_inst.instr,
                uut.cpu_inst.regfile_inst.regs[4],
                uut.cpu_inst.regfile_inst.regs[5],
                uut.cpu_inst.flags);
        end
    end

endmodule
