// Sampo CPU - Instruction Decoder
// Decodes 16-bit instructions and generates control signals

`include "sampo_pkg.vh"

module decode (
    input  wire [15:0] instr,
    input  wire [15:0] imm16,       // Second word for extended instructions

    // Instruction fields
    output wire [3:0]  opcode,
    output wire [3:0]  rd,
    output wire [3:0]  rs1,
    output wire [3:0]  rs2,
    output wire [3:0]  func,

    // ALU control
    output reg  [3:0]  alu_op,

    // Shift control
    output wire [3:0]  shift_func,

    // Branch control
    output wire [3:0]  branch_cond,

    // Immediate values
    output wire [15:0] imm8_sext,   // Sign-extended 8-bit immediate
    output wire [15:0] offset8,     // Branch offset (sign-extended, *2)
    output wire [15:0] offset12,    // Jump offset (sign-extended, *2)

    // Memory control
    output reg         mem_load,
    output reg         mem_store,
    output reg         mem_byte,
    output reg         mem_signed,
    output reg  [15:0] mem_offset,

    // Register control
    output reg         reg_write,

    // Branch/Jump flags
    output reg         is_jump,
    output reg         is_jump_reg,
    output reg         is_branch,
    output reg         is_call,
    output reg         is_ret,

    // Extended instruction flag
    output wire        is_extended,

    // Special operations
    output reg         is_halt,
    output reg         is_nop,
    output reg         is_exx,
    output reg         is_push,
    output reg         is_pop,
    output reg         is_cmp,
    output reg         is_test,
    output reg         is_getf,
    output reg         is_setf,

    // Interrupt control
    output reg         is_ei,
    output reg         is_di,
    output reg         is_reti,

    // Flag control
    output reg         is_scf,
    output reg         is_ccf,

    // I/O control
    output reg         is_io_in,
    output reg         is_io_out,
    output reg         io_port_imm,
    output wire [7:0]  io_port,

    // MulDiv operation type
    output reg         is_muldiv,
    output wire [3:0]  muldiv_func,

    // Extended instruction fields
    output wire [3:0]  ext_sub
);

    // Extract instruction fields
    assign opcode = instr[15:12];
    assign rd     = instr[11:8];
    assign rs1    = instr[7:4];
    assign rs2    = instr[3:0];
    assign func   = instr[3:0];

    // Extended sub-opcode
    assign ext_sub = instr[3:0];

    // Shift function (same as func for SHIFT opcode)
    assign shift_func = func;

    // MulDiv function
    assign muldiv_func = func;

    // Branch condition (stored in rd field for branch instructions)
    assign branch_cond = rd;

    // Sign-extend 8-bit immediate (bits 7:0)
    assign imm8_sext = {{8{instr[7]}}, instr[7:0]};

    // Branch offset: sign-extend 8-bit offset and multiply by 2
    assign offset8 = {{7{instr[7]}}, instr[7:0], 1'b0};

    // Jump offset: sign-extend 12-bit offset and multiply by 2
    assign offset12 = {{3{instr[11]}}, instr[11:0], 1'b0};

    // Extended instruction detection
    assign is_extended = (opcode == `OP_EXTENDED);

    // I/O port: immediate from rs1 field (4 bits) or from imm16 for extended
    // For INI/OUTI: port is in rs1 field (bits 7:4), only 4-bit port number
    assign io_port = io_port_imm ? {4'b0, rs1} : imm16[7:0];

    // Main decode logic
    always @(*) begin
        // Default all outputs
        alu_op      = `ALU_PASS_B;
        mem_load    = 1'b0;
        mem_store   = 1'b0;
        mem_byte    = 1'b0;
        mem_signed  = 1'b0;
        mem_offset  = 16'h0000;
        reg_write   = 1'b0;
        is_jump     = 1'b0;
        is_jump_reg = 1'b0;
        is_branch   = 1'b0;
        is_call     = 1'b0;
        is_ret      = 1'b0;
        is_halt     = 1'b0;
        is_nop      = 1'b0;
        is_exx      = 1'b0;
        is_push     = 1'b0;
        is_pop      = 1'b0;
        is_cmp      = 1'b0;
        is_test     = 1'b0;
        is_getf     = 1'b0;
        is_setf     = 1'b0;
        is_ei       = 1'b0;
        is_di       = 1'b0;
        is_reti     = 1'b0;
        is_scf      = 1'b0;
        is_ccf      = 1'b0;
        is_io_in    = 1'b0;
        is_io_out   = 1'b0;
        io_port_imm = 1'b0;
        is_muldiv   = 1'b0;

        case (opcode)
            `OP_ADD: begin
                alu_op = `ALU_ADD;
                reg_write = 1'b1;
            end

            `OP_SUB: begin
                alu_op = `ALU_SUB;
                reg_write = 1'b1;
            end

            `OP_AND: begin
                alu_op = `ALU_AND;
                reg_write = 1'b1;
            end

            `OP_OR: begin
                alu_op = `ALU_OR;
                reg_write = 1'b1;
            end

            `OP_XOR: begin
                alu_op = `ALU_XOR;
                reg_write = 1'b1;
            end

            `OP_ADDI: begin
                alu_op = `ALU_ADD;
                reg_write = 1'b1;
            end

            `OP_LOAD: begin
                mem_load = (func != `LOAD_LUI);
                reg_write = 1'b1;

                case (func)
                    `LOAD_LW:     begin mem_offset = 16'h0000; end
                    `LOAD_LB:     begin mem_offset = 16'h0000; mem_byte = 1'b1; mem_signed = 1'b1; end
                    `LOAD_LBU:    begin mem_offset = 16'h0000; mem_byte = 1'b1; mem_signed = 1'b0; end
                    4'h3:        begin mem_offset = 16'h0002; end  // LW +2
                    4'h4:        begin mem_offset = 16'h0004; end  // LW +4
                    4'h5:        begin mem_offset = 16'h0006; end  // LW +6
                    4'h6:        begin mem_offset = 16'hFFFE; end  // LW -2
                    4'h7:        begin mem_offset = 16'hFFFC; end  // LW -4
                    `LOAD_LUI:    begin alu_op = `ALU_SLL; end       // LUI: shift left 8
                    default:     begin mem_offset = 16'h0000; end
                endcase
            end

            `OP_STORE: begin
                mem_store = 1'b1;

                case (func)
                    `STORE_SW:    begin mem_offset = 16'h0000; end
                    `STORE_SB:    begin mem_offset = 16'h0000; mem_byte = 1'b1; end
                    4'h2:        begin mem_offset = 16'h0002; end  // SW +2
                    4'h3:        begin mem_offset = 16'h0004; end  // SW +4
                    4'h4:        begin mem_offset = 16'h0006; end  // SW +6
                    4'h5:        begin mem_offset = 16'hFFFE; end  // SW -2
                    4'h6:        begin mem_offset = 16'hFFFC; end  // SW -4
                    default:     begin mem_offset = 16'h0000; end
                endcase
            end

            `OP_BRANCH: begin
                is_branch = 1'b1;
            end

            `OP_JUMP: begin
                // Check if it's JR/JALR (rd = 0xF and func = 0x0 or 0x1)
                if (rd == 4'hF && (func == 4'h0)) begin
                    // JR Rs1
                    is_jump_reg = 1'b1;
                    is_ret = (rs1 == `REG_RA);  // JR R1 is return
                end else if (func == 4'h1 && rd != 4'hF) begin
                    // JALR Rd, Rs1
                    is_jump_reg = 1'b1;
                    is_call = 1'b1;
                    reg_write = 1'b1;
                end else begin
                    // J offset12
                    is_jump = 1'b1;
                end
            end

            `OP_SHIFT: begin
                reg_write = 1'b1;
                // Shift operations handled separately in CPU
            end

            `OP_MULDIV: begin
                is_muldiv = 1'b1;
                reg_write = 1'b1;

                case (func)
                    `MULDIV_MUL:   alu_op = `ALU_MUL;
                    `MULDIV_MULH:  alu_op = `ALU_MULH;
                    `MULDIV_MULHU: alu_op = `ALU_MULH;  // Will use unsigned in ALU
                    `MULDIV_DIV:   alu_op = `ALU_DIV;
                    `MULDIV_DIVU:  alu_op = `ALU_DIV;   // Will use unsigned in ALU
                    `MULDIV_REM:   alu_op = `ALU_REM;
                    `MULDIV_REMU:  alu_op = `ALU_REM;   // Will use unsigned in ALU
                    `MULDIV_DAA:   begin end           // DAA handled separately
                    default:      alu_op = `ALU_PASS_A;
                endcase
            end

            `OP_MISC: begin
                case (func)
                    `MISC_PUSH: begin
                        is_push = 1'b1;
                        mem_store = 1'b1;
                        alu_op = `ALU_SUB;  // SP - 2
                    end
                    `MISC_POP: begin
                        is_pop = 1'b1;
                        mem_load = 1'b1;
                        reg_write = 1'b1;
                        alu_op = `ALU_ADD;  // SP + 2
                    end
                    `MISC_CMP: begin
                        is_cmp = 1'b1;
                        alu_op = `ALU_SUB;
                    end
                    `MISC_TEST: begin
                        is_test = 1'b1;
                        alu_op = `ALU_AND;
                    end
                    `MISC_MOV: begin
                        alu_op = `ALU_PASS_B;
                        reg_write = 1'b1;
                    end
                    `MISC_EXX: begin
                        is_exx = 1'b1;
                    end
                    `MISC_GETF: begin
                        is_getf = 1'b1;
                        reg_write = 1'b1;
                    end
                    `MISC_SETF: begin
                        is_setf = 1'b1;
                    end
                    default: begin end
                endcase
            end

            `OP_IO: begin
                case (func)
                    `IO_INI: begin
                        is_io_in = 1'b1;
                        io_port_imm = 1'b1;
                        reg_write = 1'b1;
                    end
                    `IO_OUTI: begin
                        is_io_out = 1'b1;
                        io_port_imm = 1'b1;
                    end
                    `IO_IN: begin
                        is_io_in = 1'b1;
                        reg_write = 1'b1;
                    end
                    `IO_OUT: begin
                        is_io_out = 1'b1;
                    end
                    default: begin end
                endcase
            end

            `OP_SYSTEM: begin
                case (rd)  // System func is in rd field
                    `SYS_NOP:  is_nop = 1'b1;
                    `SYS_HALT: is_halt = 1'b1;
                    `SYS_DI:   is_di = 1'b1;
                    `SYS_EI:   is_ei = 1'b1;
                    `SYS_RETI: is_reti = 1'b1;
                    `SYS_SCF:  is_scf = 1'b1;
                    `SYS_CCF:  is_ccf = 1'b1;
                    default:  begin end
                endcase
            end

            `OP_EXTENDED: begin
                case (ext_sub)
                    `EXT_ADDIX: begin
                        alu_op = `ALU_ADD;
                        reg_write = 1'b1;
                    end
                    `EXT_SUBIX: begin
                        alu_op = `ALU_SUB;
                        reg_write = 1'b1;
                    end
                    `EXT_ANDIX: begin
                        alu_op = `ALU_AND;
                        reg_write = 1'b1;
                    end
                    `EXT_ORIX: begin
                        alu_op = `ALU_OR;
                        reg_write = 1'b1;
                    end
                    `EXT_XORIX: begin
                        alu_op = `ALU_XOR;
                        reg_write = 1'b1;
                    end
                    `EXT_LWX: begin
                        mem_load = 1'b1;
                        reg_write = 1'b1;
                    end
                    `EXT_SWX: begin
                        mem_store = 1'b1;
                    end
                    `EXT_LIX: begin
                        alu_op = `ALU_PASS_B;
                        reg_write = 1'b1;
                    end
                    `EXT_JX: begin
                        is_jump = 1'b1;
                    end
                    `EXT_JALX: begin
                        is_jump = 1'b1;
                        is_call = 1'b1;
                        reg_write = 1'b1;
                    end
                    `EXT_CMPIX: begin
                        is_cmp = 1'b1;
                        alu_op = `ALU_SUB;
                    end
                    `EXT_INX: begin
                        is_io_in = 1'b1;
                        reg_write = 1'b1;
                    end
                    `EXT_OUTX: begin
                        is_io_out = 1'b1;
                    end
                    `EXT_SLLX: begin
                        alu_op = `ALU_SLL;
                        reg_write = 1'b1;
                    end
                    `EXT_SRLX: begin
                        alu_op = `ALU_SRL;
                        reg_write = 1'b1;
                    end
                    `EXT_SRAX: begin
                        alu_op = `ALU_SRA;
                        reg_write = 1'b1;
                    end
                    default: begin end
                endcase
            end

            default: begin
                is_nop = 1'b1;
            end
        endcase
    end

endmodule
