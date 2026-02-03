// Sampo CPU - Register File Testbench

`timescale 1ns/1ps

module regfile_tb;

    reg         clk, rst_n;
    reg  [3:0]  rd_addr1, rd_addr2, wr_addr;
    wire [15:0] rd_data1, rd_data2;
    reg  [15:0] wr_data;
    reg         wr_en, exx;

    regfile uut (
        .clk(clk),
        .rst_n(rst_n),
        .rd_addr1(rd_addr1),
        .rd_addr2(rd_addr2),
        .wr_addr(wr_addr),
        .rd_data1(rd_data1),
        .rd_data2(rd_data2),
        .wr_data(wr_data),
        .wr_en(wr_en),
        .exx(exx),
        .sp_wr_data(16'h0000),
        .sp_wr_en(1'b0)
    );

    // Clock generation
    initial clk = 0;
    always #5 clk = ~clk;

    integer errors;
    integer i;

    task check_read;
        input [3:0] addr;
        input [15:0] expected;
        begin
            rd_addr1 = addr;
            #1;
            if (rd_data1 !== expected) begin
                $display("ERROR: R%d read %h, expected %h", addr, rd_data1, expected);
                errors = errors + 1;
            end
        end
    endtask

    initial begin
        errors = 0;
        rst_n = 0;
        wr_en = 0;
        exx = 0;
        rd_addr1 = 0;
        rd_addr2 = 0;
        wr_addr = 0;
        wr_data = 0;

        $display("=== Register File Testbench ===");

        // Reset
        #20;
        rst_n = 1;
        #10;

        // Test R0 hardwired to zero
        $display("Testing R0 hardwired to zero...");
        check_read(4'h0, 16'h0000);

        // Try to write to R0 - should be ignored
        wr_addr = 4'h0;
        wr_data = 16'hDEAD;
        wr_en = 1;
        @(posedge clk);
        wr_en = 0;
        #1;
        check_read(4'h0, 16'h0000);

        // Test writing and reading registers R1-R15
        $display("Testing R1-R15 write/read...");
        for (i = 1; i < 16; i = i + 1) begin
            wr_addr = i[3:0];
            wr_data = 16'h1000 + i;
            wr_en = 1;
            @(posedge clk);
            #1;
            wr_en = 0;
        end

        // Verify all registers
        @(posedge clk);  // Extra clock to ensure writes complete
        #1;
        for (i = 1; i < 16; i = i + 1) begin
            check_read(i[3:0], 16'h1000 + i);
        end

        // Test dual read ports
        $display("Testing dual read ports...");
        rd_addr1 = 4'h1;
        rd_addr2 = 4'hF;
        #1;
        if (rd_data1 !== 16'h1001 || rd_data2 !== 16'h100F) begin
            $display("ERROR: Dual read failed. Got %h, %h", rd_data1, rd_data2);
            errors = errors + 1;
        end

        // Test EXX (swap R4-R11 with alternates)
        $display("Testing EXX swap...");

        // First write known values to R4-R11
        for (i = 4; i < 12; i = i + 1) begin
            wr_addr = i[3:0];
            wr_data = 16'hAA00 + i;
            wr_en = 1;
            @(posedge clk);
            #1;
            wr_en = 0;
        end

        // Wait for last write to complete
        @(posedge clk);
        #1;

        // Verify R4-R11 have new values
        for (i = 4; i < 12; i = i + 1) begin
            check_read(i[3:0], 16'hAA00 + i);
        end

        // Execute EXX
        exx = 1;
        @(posedge clk);
        #1;
        exx = 0;

        // R4-R11 should now have alternate values (initially 0)
        $display("After first EXX, R4-R11 should be 0 (from reset alternates)...");
        for (i = 4; i < 12; i = i + 1) begin
            check_read(i[3:0], 16'h0000);
        end

        // R0-R3 and R12-R15 should be unchanged
        check_read(4'h0, 16'h0000);
        check_read(4'h1, 16'h1001);
        check_read(4'h2, 16'h1002);
        check_read(4'h3, 16'h1003);
        check_read(4'hC, 16'h100C);
        check_read(4'hD, 16'h100D);
        check_read(4'hE, 16'h100E);
        check_read(4'hF, 16'h100F);

        // EXX again to get back original values
        exx = 1;
        @(posedge clk);
        #1;
        exx = 0;

        $display("After second EXX, R4-R11 should have original values...");
        for (i = 4; i < 12; i = i + 1) begin
            check_read(i[3:0], 16'hAA00 + i);
        end

        // Summary
        $display("=== Register File Tests Complete ===");
        if (errors == 0) begin
            $display("PASSED: All tests passed!");
        end else begin
            $display("FAILED: %d errors", errors);
        end

        $finish;
    end

endmodule
