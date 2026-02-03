// Sampo CPU - Barrel Shifter
// Performs 16 different shift and rotate operations

`include "sampo_pkg.vh"

module shifter (
    input  wire [15:0] value,
    input  wire [3:0]  func,
    input  wire        carry_in,
    output reg  [15:0] result,
    output reg         carry_out
);

    always @(*) begin
        // Default values
        result = value;
        carry_out = carry_in;

        case (func)
            `SHIFT_SLL1: begin
                // Shift left logical by 1
                result = {value[14:0], 1'b0};
                carry_out = value[15];
            end

            `SHIFT_SRL1: begin
                // Shift right logical by 1
                result = {1'b0, value[15:1]};
                carry_out = value[0];
            end

            `SHIFT_SRA1: begin
                // Shift right arithmetic by 1 (preserve sign)
                result = {value[15], value[15:1]};
                carry_out = value[0];
            end

            `SHIFT_ROL1: begin
                // Rotate left by 1
                result = {value[14:0], value[15]};
                carry_out = value[15];
            end

            `SHIFT_ROR1: begin
                // Rotate right by 1
                result = {value[0], value[15:1]};
                carry_out = value[0];
            end

            `SHIFT_RCL1: begin
                // Rotate left through carry by 1
                result = {value[14:0], carry_in};
                carry_out = value[15];
            end

            `SHIFT_RCR1: begin
                // Rotate right through carry by 1
                result = {carry_in, value[15:1]};
                carry_out = value[0];
            end

            `SHIFT_SWAP: begin
                // Swap high and low bytes
                result = {value[7:0], value[15:8]};
                carry_out = carry_in;  // Carry unchanged
            end

            `SHIFT_SLL4: begin
                // Shift left logical by 4
                result = {value[11:0], 4'b0000};
                carry_out = value[15];  // Last bit shifted out
            end

            `SHIFT_SRL4: begin
                // Shift right logical by 4
                result = {4'b0000, value[15:4]};
                carry_out = value[3];  // Last bit shifted out
            end

            `SHIFT_SRA4: begin
                // Shift right arithmetic by 4
                result = {{4{value[15]}}, value[15:4]};
                carry_out = value[3];
            end

            `SHIFT_ROL4: begin
                // Rotate left by 4
                result = {value[11:0], value[15:12]};
                carry_out = value[15];
            end

            `SHIFT_SLL8: begin
                // Shift left logical by 8
                result = {value[7:0], 8'b0};
                carry_out = value[15];
            end

            `SHIFT_SRL8: begin
                // Shift right logical by 8
                result = {8'b0, value[15:8]};
                carry_out = value[7];
            end

            `SHIFT_SRA8: begin
                // Shift right arithmetic by 8
                result = {{8{value[15]}}, value[15:8]};
                carry_out = value[7];
            end

            `SHIFT_ROL8: begin
                // Rotate left by 8 (same as swap)
                result = {value[7:0], value[15:8]};
                carry_out = value[15];
            end

            default: begin
                result = value;
                carry_out = carry_in;
            end
        endcase
    end

endmodule
