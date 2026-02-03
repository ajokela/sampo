// Sampo CPU - 16-bit ALU
// Performs arithmetic, logical, and comparison operations

`include "sampo_pkg.vh"

module alu (
    input  wire [15:0] a,
    input  wire [15:0] b,
    input  wire [3:0]  op,
    input  wire        carry_in,
    output reg  [15:0] result,
    output wire        flag_n,
    output wire        flag_z,
    output reg         flag_c,
    output reg         flag_v
);

    // Internal signals for 17-bit arithmetic (to capture carry)
    wire [16:0] add_result;
    wire [16:0] sub_result;
    wire [16:0] adc_result;
    wire [16:0] sbc_result;

    // Signed versions for signed operations
    wire signed [15:0] a_signed = $signed(a);
    wire signed [15:0] b_signed = $signed(b);
    wire signed [31:0] mul_result_signed;
    wire        [31:0] mul_result_unsigned;

    // Calculate extended results
    assign add_result = {1'b0, a} + {1'b0, b};
    assign sub_result = {1'b0, a} - {1'b0, b};
    assign adc_result = {1'b0, a} + {1'b0, b} + {16'b0, carry_in};
    assign sbc_result = {1'b0, a} - {1'b0, b} - {16'b0, carry_in};

    // Multiply results
    assign mul_result_signed = a_signed * b_signed;
    assign mul_result_unsigned = a * b;

    // Overflow detection for addition: when signs of operands match but result sign differs
    wire add_overflow = (a[15] == b[15]) && (add_result[15] != a[15]);
    wire adc_overflow = (a[15] == b[15]) && (adc_result[15] != a[15]);

    // Overflow detection for subtraction: when signs of operands differ and result sign matches b
    wire sub_overflow = (a[15] != b[15]) && (sub_result[15] == b[15]);
    wire sbc_overflow = (a[15] != b[15]) && (sbc_result[15] == b[15]);

    // Main ALU operation
    always @(*) begin
        // Default values
        result = 16'h0000;
        flag_c = 1'b0;
        flag_v = 1'b0;

        case (op)
            `ALU_ADD: begin
                result = add_result[15:0];
                flag_c = add_result[16];
                flag_v = add_overflow;
            end

            `ALU_SUB: begin
                result = sub_result[15:0];
                // Carry is set when there's NO borrow (a >= b for unsigned)
                flag_c = ~sub_result[16];
                flag_v = sub_overflow;
            end

            `ALU_AND: begin
                result = a & b;
                // C and V cleared for logical ops
            end

            `ALU_OR: begin
                result = a | b;
            end

            `ALU_XOR: begin
                result = a ^ b;
            end

            `ALU_SLL: begin
                result = a << b[3:0];
                flag_c = (b[3:0] != 0) ? a[16 - b[3:0]] : 1'b0;
            end

            `ALU_SRL: begin
                result = a >> b[3:0];
                flag_c = (b[3:0] != 0) ? a[b[3:0] - 1] : 1'b0;
            end

            `ALU_SRA: begin
                result = $signed(a) >>> b[3:0];
                flag_c = (b[3:0] != 0) ? a[b[3:0] - 1] : 1'b0;
            end

            `ALU_MUL: begin
                result = mul_result_signed[15:0];
            end

            `ALU_MULH: begin
                result = mul_result_signed[31:16];
            end

            `ALU_DIV: begin
                if (b == 16'h0000) begin
                    result = 16'hFFFF;  // Division by zero
                end else begin
                    result = $signed(a_signed / b_signed);
                end
            end

            `ALU_REM: begin
                if (b == 16'h0000) begin
                    result = a;  // Remainder unchanged on div by zero
                end else begin
                    result = $signed(a_signed % b_signed);
                end
            end

            `ALU_PASS_A: begin
                result = a;
            end

            `ALU_PASS_B: begin
                result = b;
            end

            `ALU_NOT: begin
                result = ~a;
            end

            `ALU_NEG: begin
                // NEG is 0 - a
                result = -a;
                flag_c = (a != 16'h0000);  // Carry set if result is non-zero
                flag_v = (a == 16'h8000);  // Overflow only for -32768
            end

            default: begin
                result = 16'h0000;
            end
        endcase
    end

    // N flag: negative (MSB of result)
    assign flag_n = result[15];

    // Z flag: zero
    assign flag_z = (result == 16'h0000);

endmodule
