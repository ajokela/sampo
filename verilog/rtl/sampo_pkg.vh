// Sampo CPU - Parameters and Constants
// This file should be `included in modules that need these definitions

`ifndef SAMPO_PKG_VH
`define SAMPO_PKG_VH

// Primary opcodes (4-bit, bits 15:12)
`define OP_ADD      4'h0
`define OP_SUB      4'h1
`define OP_AND      4'h2
`define OP_OR       4'h3
`define OP_XOR      4'h4
`define OP_ADDI     4'h5
`define OP_LOAD     4'h6
`define OP_STORE    4'h7
`define OP_BRANCH   4'h8
`define OP_JUMP     4'h9
`define OP_SHIFT    4'hA
`define OP_MULDIV   4'hB
`define OP_MISC     4'hC
`define OP_IO       4'hD
`define OP_SYSTEM   4'hE
`define OP_EXTENDED 4'hF

// ALU operations (active during execution)
`define ALU_ADD     4'h0
`define ALU_SUB     4'h1
`define ALU_AND     4'h2
`define ALU_OR      4'h3
`define ALU_XOR     4'h4
`define ALU_SLL     4'h5
`define ALU_SRL     4'h6
`define ALU_SRA     4'h7
`define ALU_MUL     4'h8
`define ALU_MULH    4'h9
`define ALU_DIV     4'hA
`define ALU_REM     4'hB
`define ALU_PASS_A  4'hC
`define ALU_PASS_B  4'hD
`define ALU_NOT     4'hE
`define ALU_NEG     4'hF

// Load function codes (bits 3:0 when opcode = 0x6)
`define LOAD_LW     4'h0
`define LOAD_LB     4'h1
`define LOAD_LBU    4'h2
`define LOAD_LUI    4'h8

// Store function codes (bits 3:0 when opcode = 0x7)
`define STORE_SW    4'h0
`define STORE_SB    4'h1

// Shift function codes (16 functions)
`define SHIFT_SLL1  4'h0
`define SHIFT_SRL1  4'h1
`define SHIFT_SRA1  4'h2
`define SHIFT_ROL1  4'h3
`define SHIFT_ROR1  4'h4
`define SHIFT_RCL1  4'h5
`define SHIFT_RCR1  4'h6
`define SHIFT_SWAP  4'h7
`define SHIFT_SLL4  4'h8
`define SHIFT_SRL4  4'h9
`define SHIFT_SRA4  4'hA
`define SHIFT_ROL4  4'hB
`define SHIFT_SLL8  4'hC
`define SHIFT_SRL8  4'hD
`define SHIFT_SRA8  4'hE
`define SHIFT_ROL8  4'hF

// Multiply/Divide function codes (bits 3:0 when opcode = 0xB)
`define MULDIV_MUL   4'h0
`define MULDIV_MULH  4'h1
`define MULDIV_MULHU 4'h2
`define MULDIV_DIV   4'h3
`define MULDIV_DIVU  4'h4
`define MULDIV_REM   4'h5
`define MULDIV_REMU  4'h6
`define MULDIV_DAA   4'h7

// Misc function codes (bits 3:0 when opcode = 0xC)
`define MISC_PUSH   4'h0
`define MISC_POP    4'h1
`define MISC_CMP    4'h2
`define MISC_TEST   4'h3
`define MISC_MOV    4'h4
`define MISC_LDI    4'h5
`define MISC_LDD    4'h6
`define MISC_LDIR   4'h7
`define MISC_LDDR   4'h8
`define MISC_CPIR   4'h9
`define MISC_FILL   4'hA
`define MISC_EXX    4'hB
`define MISC_GETF   4'hC
`define MISC_SETF   4'hD

// I/O function codes (bits 3:0 when opcode = 0xD)
`define IO_INI      4'h0
`define IO_OUTI     4'h1
`define IO_IN       4'h2
`define IO_OUT      4'h3

// System function codes (bits 11:8 when opcode = 0xE)
`define SYS_NOP     4'h0
`define SYS_HALT    4'h1
`define SYS_DI      4'h2
`define SYS_EI      4'h3
`define SYS_RETI    4'h4
`define SYS_SWI     4'h5
`define SYS_SCF     4'h6
`define SYS_CCF     4'h7

// Extended sub-opcodes (bits 3:0 when opcode = 0xF)
`define EXT_ADDIX   4'h0
`define EXT_SUBIX   4'h1
`define EXT_ANDIX   4'h2
`define EXT_ORIX    4'h3
`define EXT_XORIX   4'h4
`define EXT_LWX     4'h5
`define EXT_SWX     4'h6
`define EXT_LIX     4'h7
`define EXT_JX      4'h8
`define EXT_JALX    4'h9
`define EXT_CMPIX   4'hA
`define EXT_INX     4'hB
`define EXT_OUTX    4'hC
`define EXT_SLLX    4'hD
`define EXT_SRLX    4'hE
`define EXT_SRAX    4'hF

// Branch conditions (16 conditions, bits 11:8 when opcode = 0x8)
`define BR_EQ       4'h0
`define BR_NE       4'h1
`define BR_LT       4'h2
`define BR_GE       4'h3
`define BR_LTU      4'h4
`define BR_GEU      4'h5
`define BR_MI       4'h6
`define BR_PL       4'h7
`define BR_VS       4'h8
`define BR_VC       4'h9
`define BR_CS       4'hA
`define BR_CC       4'hB
`define BR_GT       4'hC
`define BR_LE       4'hD
`define BR_HI       4'hE
`define BR_LS       4'hF

// Flag bit positions in FLAGS register
`define FLAG_N 7
`define FLAG_Z 6
`define FLAG_C 5
`define FLAG_V 4
`define FLAG_H 3
`define FLAG_I 2

// CPU states
`define ST_RESET     4'h0
`define ST_FETCH     4'h1
`define ST_FETCH_EXT 4'h2
`define ST_DECODE    4'h3
`define ST_EXECUTE   4'h4
`define ST_MEMORY    4'h5
`define ST_WRITEBACK 4'h6
`define ST_HALTED    4'h7

// Register aliases
`define REG_ZERO 4'h0
`define REG_RA   4'h1
`define REG_SP   4'h2
`define REG_GP   4'h3

// UART port addresses
`define UART_STATUS 8'h80
`define UART_DATA   8'h81

`endif // SAMPO_PKG_VH
