// Sampo CPU - MC6850-compatible UART Interface
// Port 0x80: Status register
// Port 0x81: Data register

module uart (
    input  wire        clk,
    input  wire        rst_n,

    // CPU I/O interface
    input  wire [7:0]  port,
    input  wire [7:0]  wdata,
    output reg  [7:0]  rdata,
    input  wire        rd,
    input  wire        wr,

    // External UART interface
    output reg  [7:0]  tx_data,
    output reg         tx_valid,
    input  wire        tx_ready,
    input  wire [7:0]  rx_data,
    input  wire        rx_valid,
    output reg         rx_ready
);

    // Port addresses
    localparam [7:0] PORT_STATUS = 8'h80;
    localparam [7:0] PORT_DATA   = 8'h81;

    // Status bits (MC6850-compatible)
    // Bit 0: RDRF - Receive Data Register Full
    // Bit 1: TDRE - Transmit Data Register Empty (TX ready)
    wire [7:0] status;
    assign status = {6'b0, tx_ready, rx_pending};

    // TX state
    reg [7:0] tx_reg;
    reg       tx_pending;

    // RX state
    reg [7:0] rx_reg;
    reg       rx_pending;

    // Combined TX logic (handles writes and handshaking)
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            tx_valid <= 1'b0;
            tx_pending <= 1'b0;
            tx_data <= 8'h00;
            tx_reg <= 8'h00;
        end else begin
            // CPU write to data port
            if (wr && port == PORT_DATA) begin
                tx_reg <= wdata;
                tx_pending <= 1'b1;
            end

            // Clear valid after it's accepted
            if (tx_valid && tx_ready) begin
                tx_valid <= 1'b0;
            end

            // Set valid when we have pending data and TX is ready and not already valid
            if (tx_pending && !tx_valid && tx_ready) begin
                tx_valid <= 1'b1;
                tx_data <= tx_reg;
                tx_pending <= 1'b0;
            end
        end
    end

    // Combined RX logic (handles reads and handshaking)
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            rx_ready <= 1'b0;
            rx_pending <= 1'b0;
            rx_reg <= 8'h00;
        end else begin
            // CPU read from data port clears pending
            if (rd && port == PORT_DATA) begin
                rx_pending <= 1'b0;
            end

            // Accept new data when available and not already pending
            if (rx_valid && !rx_pending) begin
                rx_reg <= rx_data;
                rx_pending <= 1'b1;
                rx_ready <= 1'b1;
            end

            // Clear ready after one cycle
            if (rx_ready) begin
                rx_ready <= 1'b0;
            end
        end
    end

    // CPU read (combinational)
    always @(*) begin
        rdata = 8'h00;
        if (rd) begin
            case (port)
                PORT_STATUS: rdata = status;
                PORT_DATA:   rdata = rx_reg;
                default:     rdata = 8'h00;
            endcase
        end
    end

endmodule
