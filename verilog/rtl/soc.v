// Sampo CPU - System-on-Chip Top Level
// Integrates CPU, RAM, and UART

`include "sampo_pkg.vh"

module soc #(
    parameter RESET_VECTOR = 16'h0100,
    parameter RAM_INIT_FILE = ""
) (
    input  wire        clk,
    input  wire        rst_n,

    // External UART interface
    output wire [7:0]  tx_data,
    output wire        tx_valid,
    input  wire        tx_ready,
    input  wire [7:0]  rx_data,
    input  wire        rx_valid,
    output wire        rx_ready,

    // Status outputs
    output wire        halted,
    output wire [15:0] pc,
    output wire [31:0] cycles
);

    // ========================================================================
    // CPU <-> Memory Interface
    // ========================================================================
    wire [15:0] cpu_mem_addr;
    wire [15:0] cpu_mem_rdata;
    wire [15:0] cpu_mem_wdata;
    wire        cpu_mem_we;
    wire [1:0]  cpu_mem_be;
    wire        cpu_mem_valid;
    wire        cpu_mem_ready;

    // ========================================================================
    // CPU <-> I/O Interface
    // ========================================================================
    wire [7:0]  cpu_io_addr;
    wire [7:0]  cpu_io_rdata;
    wire [7:0]  cpu_io_wdata;
    wire        cpu_io_rd;
    wire        cpu_io_wr;

    // ========================================================================
    // CPU Instance
    // ========================================================================
    cpu #(
        .RESET_VECTOR(RESET_VECTOR)
    ) cpu_inst (
        .clk(clk),
        .rst_n(rst_n),
        .mem_addr(cpu_mem_addr),
        .mem_rdata(cpu_mem_rdata),
        .mem_wdata(cpu_mem_wdata),
        .mem_we(cpu_mem_we),
        .mem_be(cpu_mem_be),
        .mem_valid(cpu_mem_valid),
        .mem_ready(cpu_mem_ready),
        .io_addr(cpu_io_addr),
        .io_rdata(cpu_io_rdata),
        .io_wdata(cpu_io_wdata),
        .io_rd(cpu_io_rd),
        .io_wr(cpu_io_wr),
        .halted(halted),
        .pc_out(pc),
        .cycles_out(cycles)
    );

    // ========================================================================
    // RAM Instance
    // ========================================================================
    ram #(
        .INIT_FILE(RAM_INIT_FILE)
    ) ram_inst (
        .clk(clk),
        .addr(cpu_mem_addr),
        .wdata(cpu_mem_wdata),
        .rdata(cpu_mem_rdata),
        .we(cpu_mem_we),
        .be(cpu_mem_be),
        .en(cpu_mem_valid)
    );

    // Memory ready is delayed by 1 cycle for synchronous RAM
    reg mem_valid_d;
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n)
            mem_valid_d <= 1'b0;
        else
            mem_valid_d <= cpu_mem_valid;
    end
    assign cpu_mem_ready = mem_valid_d;

    // ========================================================================
    // UART Instance
    // ========================================================================
    uart uart_inst (
        .clk(clk),
        .rst_n(rst_n),
        .port(cpu_io_addr),
        .wdata(cpu_io_wdata),
        .rdata(cpu_io_rdata),
        .rd(cpu_io_rd),
        .wr(cpu_io_wr),
        .tx_data(tx_data),
        .tx_valid(tx_valid),
        .tx_ready(tx_ready),
        .rx_data(rx_data),
        .rx_valid(rx_valid),
        .rx_ready(rx_ready)
    );

endmodule
