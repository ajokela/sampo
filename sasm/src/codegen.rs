//! Code generator for Sampo assembly

use crate::parser::{Operand, Program, Statement, DirectiveArg};
use std::collections::HashMap;

pub struct CodeGen {
    origin: u16,
    pc: u16,
    symbols: HashMap<String, u16>,
    output: Vec<u8>,
    fixups: Vec<Fixup>,
}

struct Fixup {
    address: u16,
    symbol: String,
    kind: FixupKind,
}

#[derive(Clone, Copy)]
enum FixupKind {
    Absolute16,
    Relative8,
    Relative12,
}

impl CodeGen {
    pub fn new() -> Self {
        CodeGen {
            origin: 0,
            pc: 0,
            symbols: HashMap::new(),
            output: Vec::new(),
            fixups: Vec::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> Result<Vec<u8>, String> {
        // Pass 1: Collect labels
        self.pass1(program)?;

        // Pass 2: Generate code
        self.pass2(program)?;

        // Pass 3: Apply fixups
        self.apply_fixups()?;

        Ok(self.output.clone())
    }

    fn pass1(&mut self, program: &Program) -> Result<(), String> {
        self.pc = self.origin;

        for stmt in &program.statements {
            match stmt {
                Statement::Label(name) => {
                    self.symbols.insert(name.clone(), self.pc);
                }
                Statement::Directive { name, args } => {
                    match name.as_str() {
                        "org" => {
                            if let Some(DirectiveArg::Number(addr)) = args.first() {
                                self.origin = *addr as u16;
                                self.pc = self.origin;
                            }
                        }
                        "equ" => {
                            if args.len() >= 2 {
                                if let (DirectiveArg::Ident(sym), DirectiveArg::Number(val)) =
                                    (&args[0], &args[1])
                                {
                                    self.symbols.insert(sym.clone(), *val as u16);
                                }
                            }
                        }
                        "db" => {
                            self.pc += args.len() as u16;
                        }
                        "dw" => {
                            self.pc += (args.len() * 2) as u16;
                        }
                        "ascii" | "asciz" => {
                            for arg in args {
                                if let DirectiveArg::String(s) = arg {
                                    self.pc += s.len() as u16;
                                    if name == "asciz" {
                                        self.pc += 1;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Statement::Instruction { mnemonic, operands } => {
                    self.pc += self.instruction_size(mnemonic, operands)?;
                }
            }
        }

        Ok(())
    }

    fn pass2(&mut self, program: &Program) -> Result<(), String> {
        self.pc = self.origin;

        // Pad output to origin if needed
        while self.output.len() < self.origin as usize {
            self.output.push(0);
        }

        for stmt in &program.statements {
            match stmt {
                Statement::Label(_) => {}
                Statement::Directive { name, args } => {
                    self.emit_directive(name, args)?;
                }
                Statement::Instruction { mnemonic, operands } => {
                    self.emit_instruction(mnemonic, operands)?;
                }
            }
        }

        Ok(())
    }

    fn instruction_size(&self, mnemonic: &str, _operands: &[Operand]) -> Result<u16, String> {
        // Most instructions are 2 bytes (16-bit)
        // Extended instructions (0xF prefix) are 4 bytes
        match mnemonic.to_uppercase().as_str() {
            // Extended 32-bit instructions
            "LIX" | "ADDIX" | "SUBIX" | "ANDIX" | "ORIX" | "XORIX" |
            "LWX" | "SWX" | "JX" | "JALX" | "CMPIX" | "INX" | "OUTX" |
            "SLLX" | "SRLX" | "SRAX" |
            // INI and OUTI also use extended format for 8-bit port
            "INI" | "OUTI" => Ok(4),
            // All others are 16-bit
            _ => Ok(2),
        }
    }

    fn emit_directive(&mut self, name: &str, args: &[DirectiveArg]) -> Result<(), String> {
        match name {
            "org" => {
                if let Some(DirectiveArg::Number(addr)) = args.first() {
                    self.pc = *addr as u16;
                    while self.output.len() < self.pc as usize {
                        self.output.push(0);
                    }
                }
            }
            "equ" => {} // Already handled in pass 1
            "db" => {
                for arg in args {
                    match arg {
                        DirectiveArg::Number(n) => {
                            self.emit_byte(*n as u8);
                        }
                        DirectiveArg::String(s) => {
                            for b in s.bytes() {
                                self.emit_byte(b);
                            }
                        }
                        DirectiveArg::Ident(sym) => {
                            if let Some(&val) = self.symbols.get(sym) {
                                self.emit_byte(val as u8);
                            } else {
                                return Err(format!("Undefined symbol: {}", sym));
                            }
                        }
                    }
                }
            }
            "dw" => {
                for arg in args {
                    match arg {
                        DirectiveArg::Number(n) => {
                            self.emit_word(*n as u16);
                        }
                        DirectiveArg::Ident(sym) => {
                            if let Some(&val) = self.symbols.get(sym) {
                                self.emit_word(val);
                            } else {
                                // Add fixup
                                self.fixups.push(Fixup {
                                    address: self.pc,
                                    symbol: sym.clone(),
                                    kind: FixupKind::Absolute16,
                                });
                                self.emit_word(0);
                            }
                        }
                        _ => return Err("Invalid .dw argument".to_string()),
                    }
                }
            }
            "ascii" => {
                for arg in args {
                    if let DirectiveArg::String(s) = arg {
                        for b in s.bytes() {
                            self.emit_byte(b);
                        }
                    }
                }
            }
            "asciz" => {
                for arg in args {
                    if let DirectiveArg::String(s) = arg {
                        for b in s.bytes() {
                            self.emit_byte(b);
                        }
                        self.emit_byte(0);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn emit_instruction(&mut self, mnemonic: &str, operands: &[Operand]) -> Result<(), String> {
        let upper = mnemonic.to_uppercase();
        match upper.as_str() {
            // Opcode 0x0: ADD Rd, Rs1, Rs2
            "ADD" => {
                let (rd, rs1, rs2) = self.get_three_regs(operands)?;
                self.emit_word(0x0000 | ((rd as u16) << 8) | ((rs1 as u16) << 4) | (rs2 as u16));
            }
            // Opcode 0x1: SUB Rd, Rs1, Rs2
            "SUB" => {
                let (rd, rs1, rs2) = self.get_three_regs(operands)?;
                self.emit_word(0x1000 | ((rd as u16) << 8) | ((rs1 as u16) << 4) | (rs2 as u16));
            }
            // Opcode 0x2: AND Rd, Rs1, Rs2
            "AND" => {
                let (rd, rs1, rs2) = self.get_three_regs(operands)?;
                self.emit_word(0x2000 | ((rd as u16) << 8) | ((rs1 as u16) << 4) | (rs2 as u16));
            }
            // Opcode 0x3: OR Rd, Rs1, Rs2
            "OR" => {
                let (rd, rs1, rs2) = self.get_three_regs(operands)?;
                self.emit_word(0x3000 | ((rd as u16) << 8) | ((rs1 as u16) << 4) | (rs2 as u16));
            }
            // Opcode 0x4: XOR Rd, Rs1, Rs2
            "XOR" => {
                let (rd, rs1, rs2) = self.get_three_regs(operands)?;
                self.emit_word(0x4000 | ((rd as u16) << 8) | ((rs1 as u16) << 4) | (rs2 as u16));
            }
            // Opcode 0x5: ADDI Rd, imm8
            "ADDI" => {
                let (rd, imm) = self.get_reg_imm(operands)?;
                if imm < -128 || imm > 127 {
                    return Err(format!("Immediate {} out of range for ADDI", imm));
                }
                self.emit_word(0x5000 | ((rd as u16) << 8) | ((imm as u8) as u16));
            }
            // Opcode 0x6: Load operations
            "LW" => {
                let (rd, rs, offset) = self.get_load_store_ops(operands)?;
                let func = self.offset_to_func(offset, true)?;
                self.emit_word(0x6000 | ((rd as u16) << 8) | ((rs as u16) << 4) | func);
            }
            "LB" => {
                let (rd, rs, _) = self.get_load_store_ops(operands)?;
                self.emit_word(0x6000 | ((rd as u16) << 8) | ((rs as u16) << 4) | 0x1);
            }
            "LBU" => {
                let (rd, rs, _) = self.get_load_store_ops(operands)?;
                self.emit_word(0x6000 | ((rd as u16) << 8) | ((rs as u16) << 4) | 0x2);
            }
            "LUI" => {
                let (rd, imm) = self.get_reg_imm(operands)?;
                self.emit_word(0x6000 | ((rd as u16) << 8) | ((imm as u8) as u16) | 0x08);
            }
            // Opcode 0x7: Store operations
            "SW" => {
                let (rs2, rs1, offset) = self.get_store_ops(operands)?;
                let func = self.offset_to_func(offset, false)?;
                self.emit_word(0x7000 | ((rs2 as u16) << 8) | ((rs1 as u16) << 4) | func);
            }
            "SB" => {
                let (rs2, rs1, _) = self.get_store_ops(operands)?;
                self.emit_word(0x7000 | ((rs2 as u16) << 8) | ((rs1 as u16) << 4) | 0x1);
            }
            // Opcode 0x8: Branch operations
            "BEQ" => self.emit_branch(0x0, operands)?,
            "BNE" => self.emit_branch(0x1, operands)?,
            "BLT" => self.emit_branch(0x2, operands)?,
            "BGE" => self.emit_branch(0x3, operands)?,
            "BLTU" => self.emit_branch(0x4, operands)?,
            "BGEU" => self.emit_branch(0x5, operands)?,
            "BMI" => self.emit_branch(0x6, operands)?,
            "BPL" => self.emit_branch(0x7, operands)?,
            "BVS" => self.emit_branch(0x8, operands)?,
            "BVC" => self.emit_branch(0x9, operands)?,
            "BCS" => self.emit_branch(0xA, operands)?,
            "BCC" => self.emit_branch(0xB, operands)?,
            "BGT" => self.emit_branch(0xC, operands)?,
            "BLE" => self.emit_branch(0xD, operands)?,
            "BHI" => self.emit_branch(0xE, operands)?,
            "BLS" => self.emit_branch(0xF, operands)?,
            // Opcode 0x9: Jump operations
            "J" => self.emit_jump(operands)?,
            "JR" => {
                let rs = self.get_one_reg(operands)?;
                self.emit_word(0x9F00 | ((rs as u16) << 4));
            }
            "JALR" => {
                let (rd, rs) = self.get_two_regs(operands)?;
                self.emit_word(0x9000 | ((rd as u16) << 8) | ((rs as u16) << 4) | 0x1);
            }
            "JAL" => {
                // JAL uses extended format for full address
                if let Some(Operand::Label(label)) = operands.first() {
                    self.emit_word(0xF000 | ((1 as u16) << 8) | 0x09); // RA, sub=9 (JALX)
                    self.fixups.push(Fixup {
                        address: self.pc,
                        symbol: label.clone(),
                        kind: FixupKind::Absolute16,
                    });
                    self.emit_word(0);
                } else if let Some(Operand::Immediate(addr)) = operands.first() {
                    self.emit_word(0xF000 | ((1 as u16) << 8) | 0x09);
                    self.emit_word(*addr as u16);
                } else {
                    return Err("JAL requires a label or address".to_string());
                }
            }
            // Opcode 0xA: Shift operations
            "SLL" => {
                let (rd, rs) = self.get_two_regs(operands)?;
                self.emit_word(0xA000 | ((rd as u16) << 8) | ((rs as u16) << 4) | 0x0);
            }
            "SRL" => {
                let (rd, rs) = self.get_two_regs(operands)?;
                self.emit_word(0xA000 | ((rd as u16) << 8) | ((rs as u16) << 4) | 0x1);
            }
            "SRA" => {
                let (rd, rs) = self.get_two_regs(operands)?;
                self.emit_word(0xA000 | ((rd as u16) << 8) | ((rs as u16) << 4) | 0x2);
            }
            "ROL" => {
                let (rd, rs) = self.get_two_regs(operands)?;
                self.emit_word(0xA000 | ((rd as u16) << 8) | ((rs as u16) << 4) | 0x3);
            }
            "ROR" => {
                let (rd, rs) = self.get_two_regs(operands)?;
                self.emit_word(0xA000 | ((rd as u16) << 8) | ((rs as u16) << 4) | 0x4);
            }
            "SWAP" => {
                let (rd, rs) = self.get_two_regs(operands)?;
                self.emit_word(0xA000 | ((rd as u16) << 8) | ((rs as u16) << 4) | 0x7);
            }
            // Opcode 0xB: Multiply/Divide
            "MUL" => {
                let (rd, rs) = self.get_two_regs(operands)?;
                self.emit_word(0xB000 | ((rd as u16) << 8) | ((rs as u16) << 4) | 0x0);
            }
            "MULH" => {
                let (rd, rs) = self.get_two_regs(operands)?;
                self.emit_word(0xB000 | ((rd as u16) << 8) | ((rs as u16) << 4) | 0x1);
            }
            "MULHU" => {
                let (rd, rs) = self.get_two_regs(operands)?;
                self.emit_word(0xB000 | ((rd as u16) << 8) | ((rs as u16) << 4) | 0x2);
            }
            "DIV" => {
                let (rd, rs) = self.get_two_regs(operands)?;
                self.emit_word(0xB000 | ((rd as u16) << 8) | ((rs as u16) << 4) | 0x3);
            }
            "DIVU" => {
                let (rd, rs) = self.get_two_regs(operands)?;
                self.emit_word(0xB000 | ((rd as u16) << 8) | ((rs as u16) << 4) | 0x4);
            }
            "REM" => {
                let (rd, rs) = self.get_two_regs(operands)?;
                self.emit_word(0xB000 | ((rd as u16) << 8) | ((rs as u16) << 4) | 0x5);
            }
            "REMU" => {
                let (rd, rs) = self.get_two_regs(operands)?;
                self.emit_word(0xB000 | ((rd as u16) << 8) | ((rs as u16) << 4) | 0x6);
            }
            "DAA" => {
                let rd = self.get_one_reg(operands)?;
                self.emit_word(0xB000 | ((rd as u16) << 8) | 0x7);
            }
            // Opcode 0xC: Stack and misc
            "PUSH" => {
                let rs = self.get_one_reg(operands)?;
                self.emit_word(0xC000 | ((rs as u16) << 4) | 0x0);
            }
            "POP" => {
                let rd = self.get_one_reg(operands)?;
                self.emit_word(0xC000 | ((rd as u16) << 8) | 0x1);
            }
            "CMP" => {
                let (rd, rs) = self.get_two_regs(operands)?;
                self.emit_word(0xC000 | ((rd as u16) << 8) | ((rs as u16) << 4) | 0x2);
            }
            "TEST" => {
                let (rd, rs) = self.get_two_regs(operands)?;
                self.emit_word(0xC000 | ((rd as u16) << 8) | ((rs as u16) << 4) | 0x3);
            }
            "MOV" => {
                let (rd, rs) = self.get_two_regs(operands)?;
                self.emit_word(0xC000 | ((rd as u16) << 8) | ((rs as u16) << 4) | 0x4);
            }
            "LDI" => self.emit_word(0xC005),
            "LDD" => self.emit_word(0xC006),
            "LDIR" => self.emit_word(0xC007),
            "LDDR" => self.emit_word(0xC008),
            "CPIR" => self.emit_word(0xC009),
            "FILL" => self.emit_word(0xC00A),
            "EXX" => self.emit_word(0xC00B),
            "GETF" => {
                let rd = self.get_one_reg(operands)?;
                self.emit_word(0xC000 | ((rd as u16) << 8) | 0xC);
            }
            "SETF" => {
                let rs = self.get_one_reg(operands)?;
                self.emit_word(0xC000 | ((rs as u16) << 4) | 0xD);
            }
            // Opcode 0xD: I/O
            "IN" => {
                let (rd, port) = self.get_in_operands(operands)?;
                self.emit_word(0xD000 | ((rd as u16) << 8) | ((port as u16) << 4) | 0x2);
            }
            "INI" => {
                let (rd, port) = self.get_reg_imm(operands)?;
                if port < 0 || port > 255 {
                    return Err("Port number out of range".to_string());
                }
                // Use extended format for 8-bit port
                self.emit_word(0xF000 | ((rd as u16) << 8) | 0x0B);
                self.emit_word(port as u16);
            }
            "OUT" => {
                let (port, rs) = self.get_out_operands(operands)?;
                self.emit_word(0xD000 | ((rs as u16) << 8) | ((port as u16) << 4) | 0x3);
            }
            "OUTI" => {
                let (port, rs) = self.get_imm_reg(operands)?;
                if port < 0 || port > 255 {
                    return Err("Port number out of range".to_string());
                }
                // Use extended format for 8-bit port
                self.emit_word(0xF000 | ((rs as u16) << 4) | 0x0C);
                self.emit_word(port as u16);
            }
            // Opcode 0xE: System
            "NOP" => self.emit_word(0xE000),
            "HALT" => self.emit_word(0xE100),
            "DI" => self.emit_word(0xE200),
            "EI" => self.emit_word(0xE300),
            "RETI" => self.emit_word(0xE400),
            "SWI" => {
                let imm = self.get_imm(operands)?;
                self.emit_word(0xE500 | ((imm as u8) as u16));
            }
            "SCF" => self.emit_word(0xE600),
            "CCF" => self.emit_word(0xE700),
            // Extended 32-bit instructions
            "LIX" => {
                let (rd, imm) = self.get_reg_imm_or_label(operands)?;
                self.emit_word(0xF000 | ((rd as u16) << 8) | 0x07);
                match imm {
                    Either::Imm(v) => self.emit_word(v as u16),
                    Either::Label(l) => {
                        self.fixups.push(Fixup {
                            address: self.pc,
                            symbol: l,
                            kind: FixupKind::Absolute16,
                        });
                        self.emit_word(0);
                    }
                }
            }
            "JX" => {
                if let Some(Operand::Label(label)) = operands.first() {
                    self.emit_word(0xF008);
                    self.fixups.push(Fixup {
                        address: self.pc,
                        symbol: label.clone(),
                        kind: FixupKind::Absolute16,
                    });
                    self.emit_word(0);
                } else if let Some(Operand::Immediate(addr)) = operands.first() {
                    self.emit_word(0xF008);
                    self.emit_word(*addr as u16);
                } else {
                    return Err("JX requires address".to_string());
                }
            }
            "JALX" => {
                if let Some(Operand::Label(label)) = operands.first() {
                    self.emit_word(0xF109); // Rd=1 (RA)
                    self.fixups.push(Fixup {
                        address: self.pc,
                        symbol: label.clone(),
                        kind: FixupKind::Absolute16,
                    });
                    self.emit_word(0);
                } else if let Some(Operand::Immediate(addr)) = operands.first() {
                    self.emit_word(0xF109);
                    self.emit_word(*addr as u16);
                } else {
                    return Err("JALX requires address".to_string());
                }
            }
            "NEG" => {
                let (rd, rs) = self.get_two_regs(operands)?;
                // NEG is SUB Rd, R0, Rs
                self.emit_word(0x1000 | ((rd as u16) << 8) | ((rs as u16)));
            }
            "NOT" => {
                let (rd, rs) = self.get_two_regs(operands)?;
                // NOT is XOR Rd, Rs, 0xFFFF - use extended
                self.emit_word(0xF000 | ((rd as u16) << 8) | ((rs as u16) << 4) | 0x04);
                self.emit_word(0xFFFF);
            }
            _ => return Err(format!("Unknown instruction: {}", mnemonic)),
        }
        Ok(())
    }

    fn emit_byte(&mut self, b: u8) {
        self.output.push(b);
        self.pc += 1;
    }

    fn emit_word(&mut self, w: u16) {
        // Little-endian
        self.output.push((w & 0xFF) as u8);
        self.output.push((w >> 8) as u8);
        self.pc += 2;
    }

    fn emit_branch(&mut self, cond: u16, operands: &[Operand]) -> Result<(), String> {
        match operands.first() {
            Some(Operand::Label(label)) => {
                self.emit_word(0x8000 | (cond << 8));
                let fixup_addr = self.pc - 2;
                self.fixups.push(Fixup {
                    address: fixup_addr,
                    symbol: label.clone(),
                    kind: FixupKind::Relative8,
                });
            }
            Some(Operand::Immediate(offset)) => {
                let off = *offset / 2; // Convert to words
                if off < -128 || off > 127 {
                    return Err("Branch offset out of range".to_string());
                }
                self.emit_word(0x8000 | (cond << 8) | ((off as u8) as u16));
            }
            _ => return Err("Branch requires target".to_string()),
        }
        Ok(())
    }

    fn emit_jump(&mut self, operands: &[Operand]) -> Result<(), String> {
        match operands.first() {
            Some(Operand::Label(label)) => {
                self.emit_word(0x9000);
                let fixup_addr = self.pc - 2;
                self.fixups.push(Fixup {
                    address: fixup_addr,
                    symbol: label.clone(),
                    kind: FixupKind::Relative12,
                });
            }
            Some(Operand::Immediate(offset)) => {
                let off = *offset / 2;
                if off < -2048 || off > 2047 {
                    return Err("Jump offset out of range".to_string());
                }
                self.emit_word(0x9000 | ((off as u16) & 0x0FFF));
            }
            _ => return Err("Jump requires target".to_string()),
        }
        Ok(())
    }

    fn get_one_reg(&self, operands: &[Operand]) -> Result<u8, String> {
        match operands.first() {
            Some(Operand::Register(r)) => Ok(*r),
            _ => Err("Expected register".to_string()),
        }
    }

    fn get_two_regs(&self, operands: &[Operand]) -> Result<(u8, u8), String> {
        if operands.len() < 2 {
            return Err("Expected two registers".to_string());
        }
        match (&operands[0], &operands[1]) {
            (Operand::Register(r1), Operand::Register(r2)) => Ok((*r1, *r2)),
            _ => Err("Expected two registers".to_string()),
        }
    }

    fn get_three_regs(&self, operands: &[Operand]) -> Result<(u8, u8, u8), String> {
        if operands.len() < 3 {
            return Err("Expected three registers".to_string());
        }
        match (&operands[0], &operands[1], &operands[2]) {
            (Operand::Register(r1), Operand::Register(r2), Operand::Register(r3)) => {
                Ok((*r1, *r2, *r3))
            }
            _ => Err("Expected three registers".to_string()),
        }
    }

    fn get_reg_imm(&self, operands: &[Operand]) -> Result<(u8, i32), String> {
        if operands.len() < 2 {
            return Err("Expected register and immediate".to_string());
        }
        match (&operands[0], &operands[1]) {
            (Operand::Register(r), Operand::Immediate(i)) => Ok((*r, *i)),
            (Operand::Register(r), Operand::Label(sym)) => {
                if let Some(&val) = self.symbols.get(sym) {
                    Ok((*r, val as i32))
                } else {
                    Err(format!("Undefined symbol: {}", sym))
                }
            }
            _ => Err("Expected register and immediate".to_string()),
        }
    }

    fn get_imm_reg(&self, operands: &[Operand]) -> Result<(i32, u8), String> {
        if operands.len() < 2 {
            return Err("Expected immediate and register".to_string());
        }
        match (&operands[0], &operands[1]) {
            (Operand::Immediate(i), Operand::Register(r)) => Ok((*i, *r)),
            (Operand::Label(sym), Operand::Register(r)) => {
                if let Some(&val) = self.symbols.get(sym) {
                    Ok((val as i32, *r))
                } else {
                    Err(format!("Undefined symbol: {}", sym))
                }
            }
            _ => Err("Expected immediate and register".to_string()),
        }
    }

    fn get_imm(&self, operands: &[Operand]) -> Result<i32, String> {
        match operands.first() {
            Some(Operand::Immediate(i)) => Ok(*i),
            Some(Operand::Label(sym)) => {
                if let Some(&val) = self.symbols.get(sym) {
                    Ok(val as i32)
                } else {
                    Err(format!("Undefined symbol: {}", sym))
                }
            }
            _ => Err("Expected immediate".to_string()),
        }
    }

    fn get_load_store_ops(&self, operands: &[Operand]) -> Result<(u8, u8, i32), String> {
        if operands.len() < 2 {
            return Err("Expected register and address".to_string());
        }
        match (&operands[0], &operands[1]) {
            (Operand::Register(rd), Operand::Indirect(rs, off)) => Ok((*rd, *rs, *off)),
            (Operand::Register(rd), Operand::Register(rs)) => Ok((*rd, *rs, 0)),
            _ => Err("Expected register and indirect address".to_string()),
        }
    }

    fn get_store_ops(&self, operands: &[Operand]) -> Result<(u8, u8, i32), String> {
        // SW (Rs1), Rs2  or  SW offset(Rs1), Rs2
        if operands.len() < 2 {
            return Err("Expected address and register".to_string());
        }
        match (&operands[0], &operands[1]) {
            (Operand::Indirect(rs1, off), Operand::Register(rs2)) => Ok((*rs2, *rs1, *off)),
            (Operand::Register(rs1), Operand::Register(rs2)) => Ok((*rs2, *rs1, 0)),
            _ => Err("Expected address and register for store".to_string()),
        }
    }

    fn get_in_operands(&self, operands: &[Operand]) -> Result<(u8, u8), String> {
        // IN Rd, (Rs)
        if operands.len() < 2 {
            return Err("Expected register and port".to_string());
        }
        match (&operands[0], &operands[1]) {
            (Operand::Register(rd), Operand::Indirect(rs, _)) => Ok((*rd, *rs)),
            (Operand::Register(rd), Operand::Register(rs)) => Ok((*rd, *rs)),
            _ => Err("Expected register and port register".to_string()),
        }
    }

    fn get_out_operands(&self, operands: &[Operand]) -> Result<(u8, u8), String> {
        // OUT (Rd), Rs
        if operands.len() < 2 {
            return Err("Expected port and register".to_string());
        }
        match (&operands[0], &operands[1]) {
            (Operand::Indirect(rd, _), Operand::Register(rs)) => Ok((*rd, *rs)),
            (Operand::Register(rd), Operand::Register(rs)) => Ok((*rd, *rs)),
            _ => Err("Expected port register and register".to_string()),
        }
    }

    fn offset_to_func(&self, offset: i32, is_load: bool) -> Result<u16, String> {
        match offset {
            0 => Ok(0x0),
            2 => Ok(0x3),
            4 => Ok(0x4),
            6 => Ok(0x5),
            -2 => Ok(0x6),
            -4 => Ok(0x7),
            _ => Err(format!("Unsupported offset {} for short load/store", offset)),
        }
    }

    fn get_reg_imm_or_label(&self, operands: &[Operand]) -> Result<(u8, Either), String> {
        if operands.len() < 2 {
            return Err("Expected register and value".to_string());
        }
        match (&operands[0], &operands[1]) {
            (Operand::Register(r), Operand::Immediate(i)) => Ok((*r, Either::Imm(*i))),
            (Operand::Register(r), Operand::Label(l)) => Ok((*r, Either::Label(l.clone()))),
            _ => Err("Expected register and immediate or label".to_string()),
        }
    }

    fn apply_fixups(&mut self) -> Result<(), String> {
        for fixup in &self.fixups {
            let target = *self.symbols.get(&fixup.symbol)
                .ok_or_else(|| format!("Undefined symbol: {}", fixup.symbol))?;

            let addr = fixup.address as usize;

            match fixup.kind {
                FixupKind::Absolute16 => {
                    self.output[addr] = (target & 0xFF) as u8;
                    self.output[addr + 1] = (target >> 8) as u8;
                }
                FixupKind::Relative8 => {
                    let pc_after = fixup.address + 2;
                    let offset = (target as i32 - pc_after as i32) / 2;
                    if offset < -128 || offset > 127 {
                        return Err(format!("Branch to {} out of range", fixup.symbol));
                    }
                    self.output[addr] = (offset as i8) as u8;
                }
                FixupKind::Relative12 => {
                    let pc_after = fixup.address + 2;
                    let offset = (target as i32 - pc_after as i32) / 2;
                    if offset < -2048 || offset > 2047 {
                        return Err(format!("Jump to {} out of range", fixup.symbol));
                    }
                    let existing = u16::from_le_bytes([self.output[addr], self.output[addr + 1]]);
                    let new_word = (existing & 0xF000) | ((offset as u16) & 0x0FFF);
                    self.output[addr] = (new_word & 0xFF) as u8;
                    self.output[addr + 1] = (new_word >> 8) as u8;
                }
            }
        }
        Ok(())
    }
}

enum Either {
    Imm(i32),
    Label(String),
}
