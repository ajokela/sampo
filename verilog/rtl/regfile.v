// Sampo CPU - Register File
// 16 x 16-bit main registers + 8 x 16-bit alternate registers (R4'-R11')
// R0 is hardwired to zero
// Dual write ports: main port + dedicated SP port for PUSH/POP

module regfile (
    input  wire        clk,
    input  wire        rst_n,
    input  wire [3:0]  rd_addr1,
    input  wire [3:0]  rd_addr2,
    input  wire [3:0]  wr_addr,
    output wire [15:0] rd_data1,
    output wire [15:0] rd_data2,
    input  wire [15:0] wr_data,
    input  wire        wr_en,
    input  wire        exx,         // Exchange R4-R11 with alternates
    // Second write port for SP updates (used by PUSH/POP)
    input  wire [15:0] sp_wr_data,
    input  wire        sp_wr_en
);

    // Main register file (R0-R15)
    reg [15:0] regs [0:15];

    // Alternate registers (R4'-R11', stored as indices 0-7)
    reg [15:0] alt_regs [0:7];

    // Read ports - R0 always returns 0
    assign rd_data1 = (rd_addr1 == 4'h0) ? 16'h0000 : regs[rd_addr1];
    assign rd_data2 = (rd_addr2 == 4'h0) ? 16'h0000 : regs[rd_addr2];

    // Write port and EXX handling
    integer i;
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            // Reset all registers to 0
            for (i = 0; i < 16; i = i + 1) begin
                regs[i] <= 16'h0000;
            end
            for (i = 0; i < 8; i = i + 1) begin
                alt_regs[i] <= 16'h0000;
            end
        end else begin
            // Handle EXX: swap R4-R11 with alternate set
            if (exx) begin
                // Swap R4-R11 with alt_regs[0-7]
                for (i = 0; i < 8; i = i + 1) begin
                    regs[i + 4] <= alt_regs[i];
                    alt_regs[i] <= regs[i + 4];
                end
            end
            // Handle register write (can happen same cycle as EXX)
            // Note: if both exx and wr_en, write happens to swapped value
            else begin
                if (wr_en && (wr_addr != 4'h0)) begin
                    // Write to register (except R0)
                    regs[wr_addr] <= wr_data;
                end
                // SP write port (R2) - for PUSH/POP stack pointer updates
                // This allows writing both Rd and SP in the same cycle
                if (sp_wr_en) begin
                    regs[2] <= sp_wr_data;  // R2 = SP
                end
            end
        end
    end

endmodule
