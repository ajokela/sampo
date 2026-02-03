// Sampo CPU - 16-bit RISC CPU Core
// FSM-based implementation with 8 states

`include "sampo_pkg.vh"

module cpu #(
    parameter RESET_VECTOR = 16'h0100
) (
    input  wire        clk,
    input  wire        rst_n,

    // Memory interface
    output reg  [15:0] mem_addr,
    input  wire [15:0] mem_rdata,
    output reg  [15:0] mem_wdata,
    output reg         mem_we,
    output reg  [1:0]  mem_be,
    output reg         mem_valid,
    input  wire        mem_ready,

    // I/O interface
    output reg  [7:0]  io_addr,
    input  wire [7:0]  io_rdata,
    output reg  [7:0]  io_wdata,
    output reg         io_rd,
    output reg         io_wr,

    // Status outputs
    output wire        halted,
    output wire [15:0] pc_out,
    output wire [31:0] cycles_out
);

    // ========================================================================
    // State Machine
    // ========================================================================
    reg [3:0] state, next_state;

    // ========================================================================
    // Registers
    // ========================================================================
    reg [15:0] pc;           // Program counter
    reg [7:0]  flags;        // Status flags
    reg [15:0] instr;        // Current instruction
    reg [15:0] imm16;        // Extended immediate
    reg [31:0] cycle_count;  // Cycle counter

    // ========================================================================
    // Register File
    // ========================================================================
    reg  [3:0]  rf_rd_addr1, rf_rd_addr2, rf_wr_addr;
    wire [15:0] rf_rd_data1, rf_rd_data2;
    reg  [15:0] rf_wr_data;
    reg         rf_wr_en;
    reg         rf_exx;
    reg  [15:0] rf_sp_wr_data;
    reg         rf_sp_wr_en;

    regfile regfile_inst (
        .clk(clk),
        .rst_n(rst_n),
        .rd_addr1(rf_rd_addr1),
        .rd_addr2(rf_rd_addr2),
        .wr_addr(rf_wr_addr),
        .rd_data1(rf_rd_data1),
        .rd_data2(rf_rd_data2),
        .wr_data(rf_wr_data),
        .wr_en(rf_wr_en),
        .exx(rf_exx),
        .sp_wr_data(rf_sp_wr_data),
        .sp_wr_en(rf_sp_wr_en)
    );

    // ========================================================================
    // ALU
    // ========================================================================
    reg  [15:0] alu_a, alu_b;
    reg  [3:0]  alu_op;
    wire [15:0] alu_result;
    wire        alu_flag_n, alu_flag_z, alu_flag_c, alu_flag_v;

    alu alu_inst (
        .a(alu_a),
        .b(alu_b),
        .op(alu_op),
        .carry_in(flags[`FLAG_C]),
        .result(alu_result),
        .flag_n(alu_flag_n),
        .flag_z(alu_flag_z),
        .flag_c(alu_flag_c),
        .flag_v(alu_flag_v)
    );

    // ========================================================================
    // Shifter
    // ========================================================================
    reg  [15:0] shift_value;
    reg  [3:0]  shift_func;
    wire [15:0] shift_result;
    wire        shift_carry;

    shifter shifter_inst (
        .value(shift_value),
        .func(shift_func),
        .carry_in(flags[`FLAG_C]),
        .result(shift_result),
        .carry_out(shift_carry)
    );

    // ========================================================================
    // Decoder
    // ========================================================================
    wire [3:0]  dec_opcode, dec_rd, dec_rs1, dec_rs2, dec_func;
    wire [3:0]  dec_alu_op, dec_shift_func, dec_branch_cond;
    wire [15:0] dec_imm8_sext, dec_offset8, dec_offset12;
    wire [15:0] dec_mem_offset;
    wire        dec_mem_load, dec_mem_store, dec_mem_byte, dec_mem_signed;
    wire        dec_reg_write, dec_is_jump, dec_is_jump_reg, dec_is_branch;
    wire        dec_is_call, dec_is_ret, dec_is_extended;
    wire        dec_is_halt, dec_is_nop, dec_is_exx;
    wire        dec_is_push, dec_is_pop, dec_is_cmp, dec_is_test;
    wire        dec_is_getf, dec_is_setf;
    wire        dec_is_ei, dec_is_di, dec_is_reti;
    wire        dec_is_scf, dec_is_ccf;
    wire        dec_is_io_in, dec_is_io_out, dec_io_port_imm;
    wire [7:0]  dec_io_port;
    wire        dec_is_muldiv;
    wire [3:0]  dec_muldiv_func, dec_ext_sub;

    decode decode_inst (
        .instr(instr),
        .imm16(imm16),
        .opcode(dec_opcode),
        .rd(dec_rd),
        .rs1(dec_rs1),
        .rs2(dec_rs2),
        .func(dec_func),
        .alu_op(dec_alu_op),
        .shift_func(dec_shift_func),
        .branch_cond(dec_branch_cond),
        .imm8_sext(dec_imm8_sext),
        .offset8(dec_offset8),
        .offset12(dec_offset12),
        .mem_load(dec_mem_load),
        .mem_store(dec_mem_store),
        .mem_byte(dec_mem_byte),
        .mem_signed(dec_mem_signed),
        .mem_offset(dec_mem_offset),
        .reg_write(dec_reg_write),
        .is_jump(dec_is_jump),
        .is_jump_reg(dec_is_jump_reg),
        .is_branch(dec_is_branch),
        .is_call(dec_is_call),
        .is_ret(dec_is_ret),
        .is_extended(dec_is_extended),
        .is_halt(dec_is_halt),
        .is_nop(dec_is_nop),
        .is_exx(dec_is_exx),
        .is_push(dec_is_push),
        .is_pop(dec_is_pop),
        .is_cmp(dec_is_cmp),
        .is_test(dec_is_test),
        .is_getf(dec_is_getf),
        .is_setf(dec_is_setf),
        .is_ei(dec_is_ei),
        .is_di(dec_is_di),
        .is_reti(dec_is_reti),
        .is_scf(dec_is_scf),
        .is_ccf(dec_is_ccf),
        .is_io_in(dec_is_io_in),
        .is_io_out(dec_is_io_out),
        .io_port_imm(dec_io_port_imm),
        .io_port(dec_io_port),
        .is_muldiv(dec_is_muldiv),
        .muldiv_func(dec_muldiv_func),
        .ext_sub(dec_ext_sub)
    );

    // ========================================================================
    // Internal Registers
    // ========================================================================
    reg [15:0] rs1_val, rs2_val;  // Latched register values
    reg [15:0] exec_result;       // Execution result
    reg [15:0] mem_result;        // Memory load result
    reg [15:0] next_pc;           // Next PC value
    reg        update_flags;      // Flag update enable
    reg [7:0]  new_flags;         // New flag values
    reg        sp_update_pending; // Need to update SP after POP
    reg [15:0] sp_new_val;        // New SP value for POP

    // ========================================================================
    // Branch Condition Evaluation
    // ========================================================================
    reg branch_taken;

    always @(*) begin
        case (dec_branch_cond)
            `BR_EQ:  branch_taken = flags[`FLAG_Z];
            `BR_NE:  branch_taken = ~flags[`FLAG_Z];
            `BR_LT:  branch_taken = flags[`FLAG_N] != flags[`FLAG_V];
            `BR_GE:  branch_taken = flags[`FLAG_N] == flags[`FLAG_V];
            `BR_LTU: branch_taken = ~flags[`FLAG_C];
            `BR_GEU: branch_taken = flags[`FLAG_C];
            `BR_MI:  branch_taken = flags[`FLAG_N];
            `BR_PL:  branch_taken = ~flags[`FLAG_N];
            `BR_VS:  branch_taken = flags[`FLAG_V];
            `BR_VC:  branch_taken = ~flags[`FLAG_V];
            `BR_CS:  branch_taken = flags[`FLAG_C];
            `BR_CC:  branch_taken = ~flags[`FLAG_C];
            `BR_GT:  branch_taken = ~flags[`FLAG_Z] && (flags[`FLAG_N] == flags[`FLAG_V]);
            `BR_LE:  branch_taken = flags[`FLAG_Z] || (flags[`FLAG_N] != flags[`FLAG_V]);
            `BR_HI:  branch_taken = flags[`FLAG_C] && ~flags[`FLAG_Z];
            `BR_LS:  branch_taken = ~flags[`FLAG_C] || flags[`FLAG_Z];
            default: branch_taken = 1'b0;
        endcase
    end

    // ========================================================================
    // Status Outputs
    // ========================================================================
    assign halted = (state == `ST_HALTED);
    assign pc_out = pc;
    assign cycles_out = cycle_count;

    // ========================================================================
    // State Machine
    // ========================================================================
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            state <= `ST_RESET;
        end else begin
            state <= next_state;
        end
    end

    // Next state logic
    always @(*) begin
        next_state = state;

        case (state)
            `ST_RESET: begin
                next_state = `ST_FETCH;
            end

            `ST_FETCH: begin
                if (mem_ready) begin
                    next_state = `ST_DECODE;
                end
            end

            `ST_DECODE: begin
                // Check if we need to fetch extended immediate
                if (dec_is_extended) begin
                    next_state = `ST_FETCH_EXT;
                end else begin
                    next_state = `ST_EXECUTE;
                end
            end

            `ST_FETCH_EXT: begin
                if (mem_ready) begin
                    next_state = `ST_EXECUTE;
                end
            end

            `ST_EXECUTE: begin
                if (dec_is_halt) begin
                    next_state = `ST_HALTED;
                end else if (dec_mem_load || dec_mem_store) begin
                    next_state = `ST_MEMORY;
                end else if (dec_reg_write) begin
                    next_state = `ST_WRITEBACK;
                end else begin
                    next_state = `ST_FETCH;
                end
            end

            `ST_MEMORY: begin
                if (mem_ready) begin
                    if (dec_mem_load && dec_reg_write) begin
                        next_state = `ST_WRITEBACK;
                    end else begin
                        next_state = `ST_FETCH;
                    end
                end
            end

            `ST_WRITEBACK: begin
                next_state = `ST_FETCH;
            end

            `ST_HALTED: begin
                next_state = `ST_HALTED;  // Stay halted
            end

            default: begin
                next_state = `ST_RESET;
            end
        endcase
    end

    // ========================================================================
    // Main Control Logic
    // ========================================================================
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            pc <= RESET_VECTOR;
            flags <= 8'h00;
            instr <= 16'h0000;
            imm16 <= 16'h0000;
            cycle_count <= 32'h0;
            rs1_val <= 16'h0000;
            rs2_val <= 16'h0000;
            exec_result <= 16'h0000;
            mem_result <= 16'h0000;
            sp_update_pending <= 1'b0;
            sp_new_val <= 16'h0000;
        end else begin
            cycle_count <= cycle_count + 1;

            case (state)
                `ST_RESET: begin
                    pc <= RESET_VECTOR;
                    flags <= 8'h00;
                end

                `ST_FETCH: begin
                    if (mem_ready) begin
                        instr <= mem_rdata;
                    end
                end

                `ST_DECODE: begin
                    // Latch register values
                    rs1_val <= rf_rd_data1;
                    rs2_val <= rf_rd_data2;
                end

                `ST_FETCH_EXT: begin
                    if (mem_ready) begin
                        imm16 <= mem_rdata;
                    end
                end

                `ST_EXECUTE: begin
                    // Calculate next PC
                    if (dec_is_halt) begin
                        // PC unchanged
                    end else if (dec_is_branch && branch_taken) begin
                        pc <= pc + 16'h2 + dec_offset8;  // Branch relative to next instruction
                    end else if (dec_is_jump && !dec_is_extended) begin
                        pc <= pc + 16'h2 + dec_offset12;  // Jump relative to next instruction
                    end else if (dec_is_jump && dec_is_extended) begin
                        pc <= imm16;  // JX or JALX
                    end else if (dec_is_jump_reg) begin
                        pc <= rs1_val;
                    end else if (dec_is_extended) begin
                        pc <= pc + 16'h4;  // Extended instruction is 4 bytes
                    end else begin
                        pc <= pc + 16'h2;  // Normal instruction is 2 bytes
                    end

                    // Store execution result
                    if (dec_opcode == `OP_SHIFT) begin
                        exec_result <= shift_result;
                    end else if (dec_is_getf) begin
                        exec_result <= {8'h00, flags};
                    end else if (dec_is_call) begin
                        // Return address
                        exec_result <= dec_is_extended ? (pc + 16'h4) : (pc + 16'h2);
                    end else if (dec_is_io_in) begin
                        exec_result <= {8'h00, io_rdata};
                    end else begin
                        exec_result <= alu_result;
                    end

                    // Update flags
                    if (dec_is_setf) begin
                        flags <= rs1_val[7:0];
                    end else if (dec_is_ei) begin
                        flags[`FLAG_I] <= 1'b1;
                    end else if (dec_is_di) begin
                        flags[`FLAG_I] <= 1'b0;
                    end else if (dec_is_scf) begin
                        flags[`FLAG_C] <= 1'b1;
                    end else if (dec_is_ccf) begin
                        flags[`FLAG_C] <= ~flags[`FLAG_C];
                    end else if (dec_opcode == `OP_SHIFT) begin
                        flags[`FLAG_N] <= shift_result[15];
                        flags[`FLAG_Z] <= (shift_result == 16'h0000);
                        flags[`FLAG_C] <= shift_carry;
                        flags[`FLAG_V] <= 1'b0;
                    end else if (dec_opcode == `OP_ADD || dec_opcode == `OP_SUB ||
                                 dec_opcode == `OP_ADDI || dec_is_cmp ||
                                 (dec_is_extended && (dec_ext_sub <= `EXT_XORIX || dec_ext_sub == `EXT_CMPIX))) begin
                        flags[`FLAG_N] <= alu_flag_n;
                        flags[`FLAG_Z] <= alu_flag_z;
                        flags[`FLAG_C] <= alu_flag_c;
                        flags[`FLAG_V] <= alu_flag_v;
                    end else if (dec_opcode == `OP_AND || dec_opcode == `OP_OR ||
                                 dec_opcode == `OP_XOR || dec_is_test) begin
                        flags[`FLAG_N] <= alu_flag_n;
                        flags[`FLAG_Z] <= alu_flag_z;
                        flags[`FLAG_C] <= 1'b0;
                        flags[`FLAG_V] <= 1'b0;
                    end else if (dec_is_muldiv) begin
                        flags[`FLAG_N] <= alu_flag_n;
                        flags[`FLAG_Z] <= alu_flag_z;
                    end
                end

                `ST_MEMORY: begin
                    if (mem_ready && dec_mem_load) begin
                        // Handle byte loads with sign/zero extension
                        if (dec_mem_byte) begin
                            if (mem_addr[0]) begin
                                // Odd address: high byte
                                mem_result <= dec_mem_signed ?
                                    {{8{mem_rdata[15]}}, mem_rdata[15:8]} :
                                    {8'h00, mem_rdata[15:8]};
                            end else begin
                                // Even address: low byte
                                mem_result <= dec_mem_signed ?
                                    {{8{mem_rdata[7]}}, mem_rdata[7:0]} :
                                    {8'h00, mem_rdata[7:0]};
                            end
                        end else begin
                            mem_result <= mem_rdata;
                        end
                    end

                    // Mark SP update needed after POP (SP += 2)
                    if (mem_ready && dec_is_pop) begin
                        sp_update_pending <= 1'b1;
                        sp_new_val <= rs1_val + 16'h2;
                    end
                end

                `ST_WRITEBACK: begin
                    // Register write happens via rf_wr_en
                    // Clear SP update pending flag
                    sp_update_pending <= 1'b0;
                end

                default: begin
                end
            endcase
        end
    end

    // ========================================================================
    // Memory Interface Control
    // ========================================================================
    always @(*) begin
        mem_addr = 16'h0000;
        mem_wdata = 16'h0000;
        mem_we = 1'b0;
        mem_be = 2'b11;
        mem_valid = 1'b0;

        case (state)
            `ST_FETCH: begin
                mem_addr = pc;
                mem_valid = 1'b1;
            end

            `ST_FETCH_EXT: begin
                mem_addr = pc + 16'h2;
                mem_valid = 1'b1;
            end

            `ST_EXECUTE: begin
                // No memory access during execute (I/O handled separately)
            end

            `ST_MEMORY: begin
                mem_valid = 1'b1;

                if (dec_is_push) begin
                    // PUSH: write to SP-2
                    mem_addr = rs1_val - 16'h2;  // rs1 is SP
                    mem_wdata = rs2_val;         // rs2 is source register
                    mem_we = 1'b1;
                end else if (dec_is_pop) begin
                    // POP: read from SP
                    mem_addr = rs1_val;
                end else if (dec_is_extended && dec_ext_sub == `EXT_LWX) begin
                    // LWX: Rs + imm16
                    mem_addr = rs1_val + imm16;
                end else if (dec_is_extended && dec_ext_sub == `EXT_SWX) begin
                    // SWX: Rs + imm16
                    mem_addr = rs1_val + imm16;
                    mem_wdata = rs2_val;  // rd contains source data for store
                    mem_we = 1'b1;
                end else if (dec_mem_store) begin
                    // Regular store
                    mem_addr = rs1_val + dec_mem_offset;
                    mem_wdata = dec_mem_byte ? {rs2_val[7:0], rs2_val[7:0]} : rs2_val;
                    mem_we = 1'b1;
                    if (dec_mem_byte) begin
                        mem_be = mem_addr[0] ? 2'b10 : 2'b01;
                    end
                end else if (dec_mem_load) begin
                    // Regular load
                    mem_addr = rs1_val + dec_mem_offset;
                end
            end

            default: begin
            end
        endcase
    end

    // ========================================================================
    // I/O Interface Control
    // ========================================================================
    always @(*) begin
        io_addr = 8'h00;
        io_wdata = 8'h00;
        io_rd = 1'b0;
        io_wr = 1'b0;

        if (state == `ST_EXECUTE) begin
            if (dec_is_io_in) begin
                io_rd = 1'b1;
                if (dec_io_port_imm) begin
                    io_addr = {4'b0, dec_rs1};  // Port in rs1 field for INI
                end else if (dec_is_extended) begin
                    io_addr = imm16[7:0];  // INX uses imm16
                end else begin
                    io_addr = rs1_val[7:0];  // IN uses Rs1
                end
            end else if (dec_is_io_out) begin
                io_wr = 1'b1;
                if (dec_io_port_imm) begin
                    io_addr = {4'b0, dec_rs1};  // Port in rs1 field for OUTI
                    io_wdata = rs2_val[7:0];   // Data in rs2
                end else if (dec_is_extended) begin
                    io_addr = imm16[7:0];      // OUTX uses imm16
                    io_wdata = rs1_val[7:0];   // Data in Rs
                end else begin
                    io_addr = rs1_val[7:0];    // OUT uses Rs1 as port
                    io_wdata = rs2_val[7:0];   // Data in Rd
                end
            end
        end
    end

    // ========================================================================
    // Register File Control
    // ========================================================================
    always @(*) begin
        rf_rd_addr1 = dec_rs1;
        rf_rd_addr2 = dec_rs2;
        rf_wr_addr = dec_rd;
        rf_wr_data = 16'h0000;
        rf_wr_en = 1'b0;
        rf_exx = 1'b0;
        rf_sp_wr_data = 16'h0000;
        rf_sp_wr_en = 1'b0;

        case (state)
            `ST_DECODE: begin
                // Read operands
                if (dec_opcode == `OP_ADDI) begin
                    rf_rd_addr1 = dec_rd;  // ADDI uses Rd as source
                end else if (dec_is_cmp || dec_is_test) begin
                    rf_rd_addr1 = dec_rd;   // CMP/TEST uses Rd as first operand
                    rf_rd_addr2 = dec_rs1;  // Rs1 as second operand
                end else if (dec_is_push) begin
                    rf_rd_addr1 = `REG_SP;
                    rf_rd_addr2 = dec_rs1;  // Source register
                end else if (dec_is_pop) begin
                    rf_rd_addr1 = `REG_SP;
                end else if (dec_is_muldiv) begin
                    rf_rd_addr1 = dec_rd;   // MUL/DIV uses Rd as first operand
                    rf_rd_addr2 = dec_rs1;
                end else if (dec_opcode == `OP_STORE) begin
                    rf_rd_addr1 = dec_rs1;  // Base address
                    rf_rd_addr2 = dec_rd;   // Data to store (in rd field for store)
                end
            end

            `ST_EXECUTE: begin
                // Handle EXX
                if (dec_is_exx) begin
                    rf_exx = 1'b1;
                end
            end

            `ST_WRITEBACK: begin
                rf_wr_en = dec_reg_write && (dec_rd != `REG_ZERO);

                // Select write data source
                if (dec_mem_load) begin
                    rf_wr_data = mem_result;
                end else if (dec_is_pop) begin
                    rf_wr_data = mem_result;
                    rf_wr_addr = dec_rd;
                end else begin
                    rf_wr_data = exec_result;
                end

                // Handle PUSH: update SP via main port
                if (dec_is_push) begin
                    rf_wr_en = 1'b1;
                    rf_wr_addr = `REG_SP;
                    rf_wr_data = rs1_val - 16'h2;
                end

                // Handle POP: write Rd via main port, SP via dedicated SP port
                if (sp_update_pending) begin
                    rf_sp_wr_en = 1'b1;
                    rf_sp_wr_data = sp_new_val;
                end
            end

            default: begin
            end
        endcase
    end

    // ========================================================================
    // ALU Input Selection
    // ========================================================================
    always @(*) begin
        alu_a = rs1_val;
        alu_b = rs2_val;
        alu_op = dec_alu_op;

        case (dec_opcode)
            `OP_ADDI: begin
                alu_a = rs1_val;  // Rd value (read via rs1)
                alu_b = dec_imm8_sext;
            end

            `OP_LOAD: begin
                if (dec_func == `LOAD_LUI) begin
                    alu_a = rs1_val;
                    alu_b = 16'h0008;  // Shift by 8
                    alu_op = `ALU_SLL;
                end
            end

            `OP_MISC: begin
                if (dec_is_cmp || dec_is_test) begin
                    alu_a = rs1_val;   // Rd value (latched during decode)
                    alu_b = rs2_val;   // Rs1 value (latched during decode)
                end else if (dec_func == `MISC_MOV) begin
                    alu_b = rs1_val;
                end
            end

            `OP_MULDIV: begin
                alu_a = rs1_val;   // Rd value (first operand, latched)
                alu_b = rs2_val;   // Rs1 value (second operand, latched)
            end

            `OP_EXTENDED: begin
                case (dec_ext_sub)
                    `EXT_ADDIX, `EXT_SUBIX, `EXT_ANDIX, `EXT_ORIX, `EXT_XORIX: begin
                        alu_a = rs1_val;
                        alu_b = imm16;
                    end
                    `EXT_LIX: begin
                        alu_b = imm16;
                    end
                    `EXT_CMPIX: begin
                        alu_a = rf_rd_data1;  // Rd value
                        alu_b = imm16;
                    end
                    `EXT_SLLX, `EXT_SRLX, `EXT_SRAX: begin
                        alu_a = rs1_val;
                        alu_b = {12'b0, imm16[3:0]};
                    end
                    default: begin
                    end
                endcase
            end

            default: begin
            end
        endcase
    end

    // ========================================================================
    // Shifter Input Selection
    // ========================================================================
    always @(*) begin
        shift_value = rs1_val;
        shift_func = dec_shift_func;
    end

endmodule
