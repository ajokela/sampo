// Sampo CPU - ALU Testbench

`timescale 1ns/1ps

`include "sampo_pkg.vh"

module alu_tb;

    reg  [15:0] a, b;
    reg  [3:0]  op;
    reg         carry_in;
    wire [15:0] result;
    wire        flag_n, flag_z, flag_c, flag_v;

    alu uut (
        .a(a),
        .b(b),
        .op(op),
        .carry_in(carry_in),
        .result(result),
        .flag_n(flag_n),
        .flag_z(flag_z),
        .flag_c(flag_c),
        .flag_v(flag_v)
    );

    integer errors;

    task check;
        input [15:0] expected;
        input expected_n, expected_z, expected_c, expected_v;
        begin
            if (result !== expected || flag_n !== expected_n ||
                flag_z !== expected_z || flag_c !== expected_c ||
                flag_v !== expected_v) begin
                $display("ERROR: op=%h a=%h b=%h c_in=%b", op, a, b, carry_in);
                $display("  Got:      result=%h N=%b Z=%b C=%b V=%b",
                         result, flag_n, flag_z, flag_c, flag_v);
                $display("  Expected: result=%h N=%b Z=%b C=%b V=%b",
                         expected, expected_n, expected_z, expected_c, expected_v);
                errors = errors + 1;
            end
        end
    endtask

    initial begin
        errors = 0;
        carry_in = 0;

        $display("=== ALU Testbench ===");

        // Test ADD
        $display("Testing ADD...");
        op = `ALU_ADD;
        a = 16'h0001; b = 16'h0002; #10; check(16'h0003, 0, 0, 0, 0);
        a = 16'hFFFF; b = 16'h0001; #10; check(16'h0000, 0, 1, 1, 0);  // Overflow to 0
        a = 16'h7FFF; b = 16'h0001; #10; check(16'h8000, 1, 0, 0, 1);  // Signed overflow
        a = 16'h8000; b = 16'h8000; #10; check(16'h0000, 0, 1, 1, 1);  // Both overflow

        // Test SUB
        $display("Testing SUB...");
        op = `ALU_SUB;
        a = 16'h0005; b = 16'h0003; #10; check(16'h0002, 0, 0, 1, 0);  // C=1 (no borrow)
        a = 16'h0003; b = 16'h0005; #10; check(16'hFFFE, 1, 0, 0, 0);  // C=0 (borrow)
        a = 16'h0000; b = 16'h0000; #10; check(16'h0000, 0, 1, 1, 0);  // Zero
        a = 16'h8000; b = 16'h0001; #10; check(16'h7FFF, 0, 0, 1, 1);  // Signed overflow

        // Test AND
        $display("Testing AND...");
        op = `ALU_AND;
        a = 16'hF0F0; b = 16'h0FF0; #10; check(16'h00F0, 0, 0, 0, 0);
        a = 16'h0000; b = 16'hFFFF; #10; check(16'h0000, 0, 1, 0, 0);
        a = 16'hFFFF; b = 16'hFFFF; #10; check(16'hFFFF, 1, 0, 0, 0);

        // Test OR
        $display("Testing OR...");
        op = `ALU_OR;
        a = 16'hF0F0; b = 16'h0F0F; #10; check(16'hFFFF, 1, 0, 0, 0);
        a = 16'h0000; b = 16'h0000; #10; check(16'h0000, 0, 1, 0, 0);

        // Test XOR
        $display("Testing XOR...");
        op = `ALU_XOR;
        a = 16'hFFFF; b = 16'hFFFF; #10; check(16'h0000, 0, 1, 0, 0);
        a = 16'hAAAA; b = 16'h5555; #10; check(16'hFFFF, 1, 0, 0, 0);

        // Test NOT
        $display("Testing NOT...");
        op = `ALU_NOT;
        a = 16'h0000; b = 16'h0000; #10; check(16'hFFFF, 1, 0, 0, 0);
        a = 16'hFFFF; b = 16'h0000; #10; check(16'h0000, 0, 1, 0, 0);
        a = 16'hAAAA; b = 16'h0000; #10; check(16'h5555, 0, 0, 0, 0);

        // Test NEG
        $display("Testing NEG...");
        op = `ALU_NEG;
        a = 16'h0001; b = 16'h0000; #10; check(16'hFFFF, 1, 0, 1, 0);
        a = 16'hFFFF; b = 16'h0000; #10; check(16'h0001, 0, 0, 1, 0);
        a = 16'h0000; b = 16'h0000; #10; check(16'h0000, 0, 1, 0, 0);
        a = 16'h8000; b = 16'h0000; #10; check(16'h8000, 1, 0, 1, 1);  // -32768 overflow

        // Test PASS_A
        $display("Testing PASS_A...");
        op = `ALU_PASS_A;
        a = 16'h1234; b = 16'h5678; #10; check(16'h1234, 0, 0, 0, 0);

        // Test PASS_B
        $display("Testing PASS_B...");
        op = `ALU_PASS_B;
        a = 16'h1234; b = 16'h5678; #10; check(16'h5678, 0, 0, 0, 0);

        // Test MUL
        $display("Testing MUL...");
        op = `ALU_MUL;
        a = 16'h0003; b = 16'h0004; #10; check(16'h000C, 0, 0, 0, 0);
        a = 16'hFFFF; b = 16'h0002; #10; check(16'hFFFE, 1, 0, 0, 0);  // -1 * 2 = -2

        // Test MULH
        $display("Testing MULH...");
        op = `ALU_MULH;
        a = 16'h0100; b = 16'h0100; #10; check(16'h0001, 0, 0, 0, 0);  // 256 * 256 = 65536
        a = 16'hFFFF; b = 16'hFFFF; #10; check(16'h0000, 0, 1, 0, 0);  // -1 * -1 = 1

        // Test DIV
        $display("Testing DIV...");
        op = `ALU_DIV;
        a = 16'h000C; b = 16'h0004; #10; check(16'h0003, 0, 0, 0, 0);
        a = 16'h000C; b = 16'h0000; #10; check(16'hFFFF, 1, 0, 0, 0);  // Div by zero

        // Test REM
        $display("Testing REM...");
        op = `ALU_REM;
        a = 16'h000D; b = 16'h0004; #10; check(16'h0001, 0, 0, 0, 0);
        a = 16'h000D; b = 16'h0000; #10; check(16'h000D, 0, 0, 0, 0);  // Unchanged on div by zero

        // Summary
        $display("=== ALU Tests Complete ===");
        if (errors == 0) begin
            $display("PASSED: All tests passed!");
        end else begin
            $display("FAILED: %d errors", errors);
        end

        $finish;
    end

endmodule
