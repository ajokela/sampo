// Sampo CPU - 64KB Synchronous RAM
// Little-endian, supports word and byte access

module ram #(
    parameter INIT_FILE = ""
) (
    input  wire        clk,
    input  wire [15:0] addr,
    input  wire [15:0] wdata,
    output reg  [15:0] rdata,
    input  wire        we,
    input  wire [1:0]  be,      // Byte enables: [0] = low byte, [1] = high byte
    input  wire        en
);

    // 64KB = 32K x 16-bit words
    reg [15:0] mem [0:32767];

    // Word-aligned address (ignore LSB)
    wire [14:0] word_addr = addr[15:1];

    // Initialize memory from hex file if specified
    initial begin
        if (INIT_FILE != "") begin
            $readmemh(INIT_FILE, mem);
        end
    end

    // Synchronous read and write
    always @(posedge clk) begin
        if (en) begin
            if (we) begin
                // Write with byte enables
                if (be[0]) mem[word_addr][7:0]  <= wdata[7:0];
                if (be[1]) mem[word_addr][15:8] <= wdata[15:8];
            end
            // Read (always returns full word)
            rdata <= mem[word_addr];
        end
    end

endmodule
