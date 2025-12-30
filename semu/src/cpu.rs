//! Sampo CPU emulation core

use std::io::{self, Write};

const MEM_SIZE: usize = 65536; // 64KB

// Flag bits
const FLAG_N: u8 = 0x80; // Negative
const FLAG_Z: u8 = 0x40; // Zero
const FLAG_C: u8 = 0x20; // Carry
const FLAG_V: u8 = 0x10; // Overflow
const FLAG_H: u8 = 0x08; // Half-carry (BCD)
const FLAG_I: u8 = 0x04; // Interrupt enable

pub struct Cpu {
    // Registers
    regs: [u16; 16],
    regs_alt: [u16; 8], // Alternate R4-R11
    pc: u16,
    flags: u8,

    // Memory
    memory: Vec<u8>,

    // I/O ports
    ports: [u8; 256],

    // State
    halted: bool,
    trace: bool,
    cycles: u64,

    // Serial output buffer
    serial_out: Vec<u8>,
}

impl Cpu {
    pub fn new() -> Self {
        let mut cpu = Cpu {
            regs: [0; 16],
            regs_alt: [0; 8],
            pc: 0x0100, // Default start address
            flags: 0,
            memory: vec![0; MEM_SIZE],
            ports: [0; 256],
            halted: false,
            trace: false,
            cycles: 0,
            serial_out: Vec::new(),
        };

        // Initialize SP to top of RAM
        cpu.regs[2] = 0xFFFE;

        // Set serial TX ready bit
        cpu.ports[0x80] = 0x02;

        cpu
    }

    pub fn load_program(&mut self, program: &[u8]) {
        for (i, &byte) in program.iter().enumerate() {
            if i < MEM_SIZE {
                self.memory[i] = byte;
            }
        }

        // Find first non-zero word as entry point
        for i in (0..program.len()).step_by(2) {
            if i + 1 < program.len() {
                let word = u16::from_le_bytes([program[i], program[i + 1]]);
                if word != 0 {
                    self.pc = i as u16;
                    break;
                }
            }
        }
    }

    pub fn set_trace(&mut self, trace: bool) {
        self.trace = trace;
    }

    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    pub fn step(&mut self) -> Result<bool, String> {
        if self.halted {
            return Ok(false);
        }

        // Fetch instruction
        let instr = self.fetch_word()?;

        if self.trace {
            self.trace_instruction(instr);
        }

        // Decode and execute
        self.execute(instr)?;

        self.cycles += 1;
        Ok(!self.halted)
    }

    fn fetch_word(&mut self) -> Result<u16, String> {
        if self.pc as usize + 1 >= MEM_SIZE {
            return Err("PC out of bounds".to_string());
        }
        let lo = self.memory[self.pc as usize];
        let hi = self.memory[self.pc as usize + 1];
        self.pc = self.pc.wrapping_add(2);
        Ok(u16::from_le_bytes([lo, hi]))
    }

    fn execute(&mut self, instr: u16) -> Result<(), String> {
        let opcode = (instr >> 12) & 0xF;
        let rd = ((instr >> 8) & 0xF) as usize;
        let rs1 = ((instr >> 4) & 0xF) as usize;
        let rs2 = (instr & 0xF) as usize;
        let imm8 = (instr & 0xFF) as i8 as i16 as u16;
        let func = instr & 0xF;

        match opcode {
            0x0 => {
                // ADD Rd, Rs1, Rs2
                let a = self.get_reg(rs1);
                let b = self.get_reg(rs2);
                let (result, carry) = a.overflowing_add(b);
                self.set_reg(rd, result);
                self.set_flags_add(a, b, result, carry);
            }
            0x1 => {
                // SUB Rd, Rs1, Rs2
                let a = self.get_reg(rs1);
                let b = self.get_reg(rs2);
                let (result, borrow) = a.overflowing_sub(b);
                self.set_reg(rd, result);
                self.set_flags_sub(a, b, result, borrow);
            }
            0x2 => {
                // AND Rd, Rs1, Rs2
                let result = self.get_reg(rs1) & self.get_reg(rs2);
                self.set_reg(rd, result);
                self.set_flags_logic(result);
            }
            0x3 => {
                // OR Rd, Rs1, Rs2
                let result = self.get_reg(rs1) | self.get_reg(rs2);
                self.set_reg(rd, result);
                self.set_flags_logic(result);
            }
            0x4 => {
                // XOR Rd, Rs1, Rs2
                let result = self.get_reg(rs1) ^ self.get_reg(rs2);
                self.set_reg(rd, result);
                self.set_flags_logic(result);
            }
            0x5 => {
                // ADDI Rd, imm8
                let a = self.get_reg(rd);
                let b = imm8;
                let (result, carry) = a.overflowing_add(b);
                self.set_reg(rd, result);
                self.set_flags_add(a, b, result, carry);
            }
            0x6 => {
                // Load operations
                self.execute_load(rd, rs1, func)?;
            }
            0x7 => {
                // Store operations
                self.execute_store(rd, rs1, func)?;
            }
            0x8 => {
                // Branch operations
                let cond = rd as u16;
                let offset = (instr & 0xFF) as i8 as i16;
                if self.check_condition(cond) {
                    self.pc = (self.pc as i16).wrapping_add(offset * 2) as u16;
                }
            }
            0x9 => {
                // Jump operations
                // JR is encoded as 9F0R (R in bits 7:4, low nibble = 0)
                // JALR is encoded as 9DR1 (D in bits 11:8, R in bits 7:4, func = 1)
                if (instr & 0x0F0F) == 0x0F00 {
                    // JR Rs (9F0R pattern)
                    let rs = rs1;
                    self.pc = self.get_reg(rs);
                } else if func == 0x1 && rd != 0 {
                    // JALR Rd, Rs
                    let ret_addr = self.pc;
                    self.pc = self.get_reg(rs1);
                    self.set_reg(rd, ret_addr);
                } else {
                    // J offset12
                    let offset = (instr & 0x0FFF) as i16;
                    let offset = if offset & 0x800 != 0 {
                        offset | 0xF000u16 as i16
                    } else {
                        offset
                    };
                    self.pc = (self.pc as i16).wrapping_add(offset * 2) as u16;
                }
            }
            0xA => {
                // Shift operations
                self.execute_shift(rd, rs1, func)?;
            }
            0xB => {
                // Multiply/Divide
                self.execute_muldiv(rd, rs1, func)?;
            }
            0xC => {
                // Stack and misc
                self.execute_misc(rd, rs1, func)?;
            }
            0xD => {
                // I/O operations
                self.execute_io(rd, rs1, func)?;
            }
            0xE => {
                // System operations
                self.execute_system(rd, imm8 as u8)?;
            }
            0xF => {
                // Extended (32-bit) instructions
                let imm16 = self.fetch_word()?;
                self.execute_extended(rd, rs1, func, imm16)?;
            }
            _ => {
                return Err(format!("Unknown opcode: 0x{:X}", opcode));
            }
        }

        Ok(())
    }

    fn execute_load(&mut self, rd: usize, rs1: usize, func: u16) -> Result<(), String> {
        let base = self.get_reg(rs1);
        let offset: i16 = match func {
            0x0 => 0,
            0x1 => 0, // LB
            0x2 => 0, // LBU
            0x3 => 2,
            0x4 => 4,
            0x5 => 6,
            0x6 => -2,
            0x7 => -4,
            0x8 => {
                // LUI - load upper immediate (Rs1 used as immediate here)
                let val = (rs1 as u16) << 8;
                self.set_reg(rd, val);
                return Ok(());
            }
            _ => return Err(format!("Unknown load func: 0x{:X}", func)),
        };

        let addr = (base as i16).wrapping_add(offset) as u16;

        match func {
            0x1 => {
                // LB (sign extend)
                let val = self.read_byte(addr)? as i8 as i16 as u16;
                self.set_reg(rd, val);
            }
            0x2 => {
                // LBU (zero extend)
                let val = self.read_byte(addr)? as u16;
                self.set_reg(rd, val);
            }
            _ => {
                // LW
                let val = self.read_word(addr)?;
                self.set_reg(rd, val);
            }
        }

        Ok(())
    }

    fn execute_store(&mut self, rs2: usize, rs1: usize, func: u16) -> Result<(), String> {
        let base = self.get_reg(rs1);
        let value = self.get_reg(rs2);

        let offset: i16 = match func {
            0x0 => 0,
            0x1 => 0, // SB
            0x2 => 2,
            0x3 => 4,
            0x4 => 6,
            0x5 => -2,
            0x6 => -4,
            _ => return Err(format!("Unknown store func: 0x{:X}", func)),
        };

        let addr = (base as i16).wrapping_add(offset) as u16;

        if func == 0x1 {
            self.write_byte(addr, value as u8)?;
        } else {
            self.write_word(addr, value)?;
        }

        Ok(())
    }

    fn execute_shift(&mut self, rd: usize, rs1: usize, func: u16) -> Result<(), String> {
        let val = self.get_reg(rs1);
        let result = match func {
            0x0 => val << 1,                           // SLL 1
            0x1 => val >> 1,                           // SRL 1
            0x2 => ((val as i16) >> 1) as u16,         // SRA 1
            0x3 => val.rotate_left(1),                 // ROL 1
            0x4 => val.rotate_right(1),                // ROR 1
            0x5 => {                                   // RCL 1
                let c = (self.flags & FLAG_C) != 0;
                let new_c = (val & 0x8000) != 0;
                let result = (val << 1) | (c as u16);
                if new_c { self.flags |= FLAG_C; } else { self.flags &= !FLAG_C; }
                result
            }
            0x6 => {                                   // RCR 1
                let c = (self.flags & FLAG_C) != 0;
                let new_c = (val & 1) != 0;
                let result = (val >> 1) | ((c as u16) << 15);
                if new_c { self.flags |= FLAG_C; } else { self.flags &= !FLAG_C; }
                result
            }
            0x7 => ((val & 0xFF) << 8) | ((val >> 8) & 0xFF), // SWAP
            0x8 => val << 4,                           // SLL 4
            0x9 => val >> 4,                           // SRL 4
            0xA => ((val as i16) >> 4) as u16,         // SRA 4
            0xB => val.rotate_left(4),                 // ROL 4
            0xC => val << 8,                           // SLL 8
            0xD => val >> 8,                           // SRL 8
            0xE => ((val as i16) >> 8) as u16,         // SRA 8
            0xF => val.rotate_left(8),                 // ROL 8
            _ => return Err(format!("Unknown shift func: 0x{:X}", func)),
        };
        self.set_reg(rd, result);
        self.set_flags_logic(result);
        Ok(())
    }

    fn execute_muldiv(&mut self, rd: usize, rs1: usize, func: u16) -> Result<(), String> {
        let a = self.get_reg(rd);
        let b = self.get_reg(rs1);

        match func {
            0x0 => {
                // MUL (low 16 bits)
                let result = (a as u32).wrapping_mul(b as u32);
                self.set_reg(rd, result as u16);
            }
            0x1 => {
                // MULH (high 16 bits, signed)
                let result = (a as i16 as i32).wrapping_mul(b as i16 as i32);
                self.set_reg(rd, (result >> 16) as u16);
            }
            0x2 => {
                // MULHU (high 16 bits, unsigned)
                let result = (a as u32).wrapping_mul(b as u32);
                self.set_reg(rd, (result >> 16) as u16);
            }
            0x3 => {
                // DIV (signed)
                if b == 0 {
                    self.set_reg(rd, 0xFFFF);
                } else {
                    let result = (a as i16).wrapping_div(b as i16);
                    self.set_reg(rd, result as u16);
                }
            }
            0x4 => {
                // DIVU (unsigned)
                if b == 0 {
                    self.set_reg(rd, 0xFFFF);
                } else {
                    let result = a / b;
                    self.set_reg(rd, result);
                }
            }
            0x5 => {
                // REM (signed)
                if b == 0 {
                    self.set_reg(rd, a);
                } else {
                    let result = (a as i16).wrapping_rem(b as i16);
                    self.set_reg(rd, result as u16);
                }
            }
            0x6 => {
                // REMU (unsigned)
                if b == 0 {
                    self.set_reg(rd, a);
                } else {
                    let result = a % b;
                    self.set_reg(rd, result);
                }
            }
            0x7 => {
                // DAA (decimal adjust)
                let mut val = self.get_reg(rd);
                let mut carry = false;

                // Adjust lower nibble
                if (val & 0x0F) > 9 || (self.flags & FLAG_H) != 0 {
                    val = val.wrapping_add(6);
                    if val > 0xFF {
                        carry = true;
                    }
                }

                // Adjust upper nibble
                if ((val >> 4) & 0x0F) > 9 || (self.flags & FLAG_C) != 0 {
                    val = val.wrapping_add(0x60);
                    carry = true;
                }

                self.set_reg(rd, val & 0xFF);
                if carry {
                    self.flags |= FLAG_C;
                }
                self.set_flags_logic(val);
            }
            _ => return Err(format!("Unknown muldiv func: 0x{:X}", func)),
        }
        Ok(())
    }

    fn execute_misc(&mut self, rd: usize, rs1: usize, func: u16) -> Result<(), String> {
        match func {
            0x0 => {
                // PUSH Rs1
                let sp = self.get_reg(2).wrapping_sub(2);
                self.set_reg(2, sp);
                let val = self.get_reg(rs1);
                self.write_word(sp, val)?;
            }
            0x1 => {
                // POP Rd
                let sp = self.get_reg(2);
                let val = self.read_word(sp)?;
                self.set_reg(rd, val);
                self.set_reg(2, sp.wrapping_add(2));
            }
            0x2 => {
                // CMP Rd, Rs1
                let a = self.get_reg(rd);
                let b = self.get_reg(rs1);
                let (result, borrow) = a.overflowing_sub(b);
                self.set_flags_sub(a, b, result, borrow);
            }
            0x3 => {
                // TEST Rd, Rs1
                let result = self.get_reg(rd) & self.get_reg(rs1);
                self.set_flags_logic(result);
            }
            0x4 => {
                // MOV Rd, Rs1
                let val = self.get_reg(rs1);
                self.set_reg(rd, val);
            }
            0x5 => {
                // LDI - block load increment
                let src = self.get_reg(5);
                let dst = self.get_reg(6);
                let count = self.get_reg(4);

                let byte = self.read_byte(src)?;
                self.write_byte(dst, byte)?;

                self.set_reg(5, src.wrapping_add(1));
                self.set_reg(6, dst.wrapping_add(1));
                self.set_reg(4, count.wrapping_sub(1));

                if count.wrapping_sub(1) == 0 {
                    self.flags |= FLAG_Z;
                } else {
                    self.flags &= !FLAG_Z;
                }
            }
            0x6 => {
                // LDD - block load decrement
                let src = self.get_reg(5);
                let dst = self.get_reg(6);
                let count = self.get_reg(4);

                let byte = self.read_byte(src)?;
                self.write_byte(dst, byte)?;

                self.set_reg(5, src.wrapping_sub(1));
                self.set_reg(6, dst.wrapping_sub(1));
                self.set_reg(4, count.wrapping_sub(1));

                if count.wrapping_sub(1) == 0 {
                    self.flags |= FLAG_Z;
                } else {
                    self.flags &= !FLAG_Z;
                }
            }
            0x7 => {
                // LDIR - block load repeat increment
                loop {
                    let src = self.get_reg(5);
                    let dst = self.get_reg(6);
                    let count = self.get_reg(4);

                    if count == 0 {
                        break;
                    }

                    let byte = self.read_byte(src)?;
                    self.write_byte(dst, byte)?;

                    self.set_reg(5, src.wrapping_add(1));
                    self.set_reg(6, dst.wrapping_add(1));
                    self.set_reg(4, count.wrapping_sub(1));
                }
                self.flags |= FLAG_Z;
            }
            0x8 => {
                // LDDR - block load repeat decrement
                loop {
                    let src = self.get_reg(5);
                    let dst = self.get_reg(6);
                    let count = self.get_reg(4);

                    if count == 0 {
                        break;
                    }

                    let byte = self.read_byte(src)?;
                    self.write_byte(dst, byte)?;

                    self.set_reg(5, src.wrapping_sub(1));
                    self.set_reg(6, dst.wrapping_sub(1));
                    self.set_reg(4, count.wrapping_sub(1));
                }
                self.flags |= FLAG_Z;
            }
            0x9 => {
                // CPIR - compare and search
                let needle = self.get_reg(4) as u8;
                let mut addr = self.get_reg(5);
                let mut count = self.get_reg(6);

                while count > 0 {
                    let byte = self.read_byte(addr)?;
                    if byte == needle {
                        self.flags |= FLAG_Z;
                        self.set_reg(5, addr);
                        self.set_reg(6, count);
                        return Ok(());
                    }
                    addr = addr.wrapping_add(1);
                    count = count.wrapping_sub(1);
                }

                self.flags &= !FLAG_Z;
                self.set_reg(5, addr);
                self.set_reg(6, 0);
            }
            0xA => {
                // FILL
                let val = self.get_reg(5) as u8;
                let mut dst = self.get_reg(6);
                let mut count = self.get_reg(4);

                while count > 0 {
                    self.write_byte(dst, val)?;
                    dst = dst.wrapping_add(1);
                    count = count.wrapping_sub(1);
                }

                self.set_reg(6, dst);
                self.set_reg(4, 0);
            }
            0xB => {
                // EXX - swap alternate registers
                for i in 0..8 {
                    std::mem::swap(&mut self.regs[4 + i], &mut self.regs_alt[i]);
                }
            }
            0xC => {
                // GETF Rd
                self.set_reg(rd, self.flags as u16);
            }
            0xD => {
                // SETF Rs1
                self.flags = self.get_reg(rs1) as u8;
            }
            _ => return Err(format!("Unknown misc func: 0x{:X}", func)),
        }
        Ok(())
    }

    fn execute_io(&mut self, rd: usize, rs1: usize, func: u16) -> Result<(), String> {
        match func {
            0x0 => {
                // INI Rd, port (immediate port in next nibble)
                let port = rs1 as u8;
                let val = self.port_read(port);
                self.set_reg(rd, val as u16);
            }
            0x1 => {
                // OUTI port, Rd
                let port = rs1 as u8;
                let val = self.get_reg(rd) as u8;
                self.port_write(port, val);
            }
            0x2 => {
                // IN Rd, (Rs1)
                let port = self.get_reg(rs1) as u8;
                let val = self.port_read(port);
                self.set_reg(rd, val as u16);
            }
            0x3 => {
                // OUT (Rd), Rs1
                let port = self.get_reg(rd) as u8;
                let val = self.get_reg(rs1) as u8;
                self.port_write(port, val);
            }
            _ => return Err(format!("Unknown I/O func: 0x{:X}", func)),
        }
        Ok(())
    }

    fn execute_system(&mut self, func: usize, imm: u8) -> Result<(), String> {
        match func {
            0x0 => {} // NOP
            0x1 => {
                // HALT
                self.halted = true;
            }
            0x2 => {
                // DI
                self.flags &= !FLAG_I;
            }
            0x3 => {
                // EI
                self.flags |= FLAG_I;
            }
            0x4 => {
                // RETI
                // Pop PC from stack
                let sp = self.get_reg(2);
                let pc = self.read_word(sp)?;
                self.set_reg(2, sp.wrapping_add(2));
                self.pc = pc;
                self.flags |= FLAG_I;
            }
            0x5 => {
                // SWI imm
                // Push PC, jump to interrupt handler
                let sp = self.get_reg(2).wrapping_sub(2);
                self.set_reg(2, sp);
                self.write_word(sp, self.pc)?;
                self.pc = (imm as u16) * 2; // Simple vector table
            }
            0x6 => {
                // SCF
                self.flags |= FLAG_C;
            }
            0x7 => {
                // CCF
                self.flags ^= FLAG_C;
            }
            _ => return Err(format!("Unknown system func: 0x{:X}", func)),
        }
        Ok(())
    }

    fn execute_extended(&mut self, rd: usize, rs1: usize, sub: u16, imm16: u16) -> Result<(), String> {
        match sub {
            0x0 => {
                // ADDIX Rd, Rs1, imm16
                let a = self.get_reg(rs1);
                let (result, carry) = a.overflowing_add(imm16);
                self.set_reg(rd, result);
                self.set_flags_add(a, imm16, result, carry);
            }
            0x1 => {
                // SUBIX Rd, Rs1, imm16
                let a = self.get_reg(rs1);
                let (result, borrow) = a.overflowing_sub(imm16);
                self.set_reg(rd, result);
                self.set_flags_sub(a, imm16, result, borrow);
            }
            0x2 => {
                // ANDIX Rd, Rs1, imm16
                let result = self.get_reg(rs1) & imm16;
                self.set_reg(rd, result);
                self.set_flags_logic(result);
            }
            0x3 => {
                // ORIX Rd, Rs1, imm16
                let result = self.get_reg(rs1) | imm16;
                self.set_reg(rd, result);
                self.set_flags_logic(result);
            }
            0x4 => {
                // XORIX Rd, Rs1, imm16
                let result = self.get_reg(rs1) ^ imm16;
                self.set_reg(rd, result);
                self.set_flags_logic(result);
            }
            0x5 => {
                // LWX Rd, imm16(Rs1)
                let addr = self.get_reg(rs1).wrapping_add(imm16);
                let val = self.read_word(addr)?;
                self.set_reg(rd, val);
            }
            0x6 => {
                // SWX Rd, imm16(Rs1)
                let addr = self.get_reg(rs1).wrapping_add(imm16);
                let val = self.get_reg(rd);
                self.write_word(addr, val)?;
            }
            0x7 => {
                // LIX Rd, imm16
                self.set_reg(rd, imm16);
            }
            0x8 => {
                // JX addr16
                self.pc = imm16;
            }
            0x9 => {
                // JALX addr16
                self.set_reg(rd, self.pc);
                self.pc = imm16;
            }
            0xA => {
                // CMPIX Rd, imm16
                let a = self.get_reg(rd);
                let (result, borrow) = a.overflowing_sub(imm16);
                self.set_flags_sub(a, imm16, result, borrow);
            }
            0xB => {
                // INX Rd, port8
                let val = self.port_read(imm16 as u8);
                self.set_reg(rd, val as u16);
            }
            0xC => {
                // OUTX port8, Rs1
                let val = self.get_reg(rs1) as u8;
                self.port_write(imm16 as u8, val);
            }
            0xD => {
                // SLLX Rd, Rs1, imm4
                let shift = (imm16 & 0xF) as u32;
                let result = self.get_reg(rs1) << shift;
                self.set_reg(rd, result);
                self.set_flags_logic(result);
            }
            0xE => {
                // SRLX Rd, Rs1, imm4
                let shift = (imm16 & 0xF) as u32;
                let result = self.get_reg(rs1) >> shift;
                self.set_reg(rd, result);
                self.set_flags_logic(result);
            }
            0xF => {
                // SRAX Rd, Rs1, imm4
                let shift = (imm16 & 0xF) as u32;
                let val = self.get_reg(rs1) as i16;
                let result = (val >> shift) as u16;
                self.set_reg(rd, result);
                self.set_flags_logic(result);
            }
            _ => return Err(format!("Unknown extended sub: 0x{:X}", sub)),
        }
        Ok(())
    }

    // Register access (R0 always returns 0)
    fn get_reg(&self, r: usize) -> u16 {
        if r == 0 {
            0
        } else {
            self.regs[r]
        }
    }

    fn set_reg(&mut self, r: usize, val: u16) {
        if r != 0 {
            self.regs[r] = val;
        }
    }

    // Memory access
    fn read_byte(&self, addr: u16) -> Result<u8, String> {
        Ok(self.memory[addr as usize])
    }

    fn write_byte(&mut self, addr: u16, val: u8) -> Result<(), String> {
        self.memory[addr as usize] = val;
        Ok(())
    }

    fn read_word(&self, addr: u16) -> Result<u16, String> {
        let lo = self.memory[addr as usize];
        let hi = self.memory[addr.wrapping_add(1) as usize];
        Ok(u16::from_le_bytes([lo, hi]))
    }

    fn write_word(&mut self, addr: u16, val: u16) -> Result<(), String> {
        let bytes = val.to_le_bytes();
        self.memory[addr as usize] = bytes[0];
        self.memory[addr.wrapping_add(1) as usize] = bytes[1];
        Ok(())
    }

    // Port I/O
    fn port_read(&mut self, port: u8) -> u8 {
        match port {
            0x80 => {
                // ACIA status - always ready
                0x02 // TX ready
            }
            0x81 => {
                // ACIA data - nothing to read
                0
            }
            _ => self.ports[port as usize],
        }
    }

    fn port_write(&mut self, port: u8, val: u8) {
        match port {
            0x81 => {
                // ACIA data - output character
                self.serial_out.push(val);
                print!("{}", val as char);
                io::stdout().flush().ok();
            }
            _ => {
                self.ports[port as usize] = val;
            }
        }
    }

    // Flag operations
    fn check_condition(&self, cond: u16) -> bool {
        match cond {
            0x0 => (self.flags & FLAG_Z) != 0,           // BEQ
            0x1 => (self.flags & FLAG_Z) == 0,           // BNE
            0x2 => {                                      // BLT (signed)
                let n = (self.flags & FLAG_N) != 0;
                let v = (self.flags & FLAG_V) != 0;
                n != v
            }
            0x3 => {                                      // BGE (signed)
                let n = (self.flags & FLAG_N) != 0;
                let v = (self.flags & FLAG_V) != 0;
                n == v
            }
            0x4 => (self.flags & FLAG_C) == 0,           // BLTU
            0x5 => (self.flags & FLAG_C) != 0,           // BGEU
            0x6 => (self.flags & FLAG_N) != 0,           // BMI
            0x7 => (self.flags & FLAG_N) == 0,           // BPL
            0x8 => (self.flags & FLAG_V) != 0,           // BVS
            0x9 => (self.flags & FLAG_V) == 0,           // BVC
            0xA => (self.flags & FLAG_C) != 0,           // BCS
            0xB => (self.flags & FLAG_C) == 0,           // BCC
            0xC => {                                      // BGT (signed)
                let z = (self.flags & FLAG_Z) != 0;
                let n = (self.flags & FLAG_N) != 0;
                let v = (self.flags & FLAG_V) != 0;
                !z && (n == v)
            }
            0xD => {                                      // BLE (signed)
                let z = (self.flags & FLAG_Z) != 0;
                let n = (self.flags & FLAG_N) != 0;
                let v = (self.flags & FLAG_V) != 0;
                z || (n != v)
            }
            0xE => {                                      // BHI (unsigned)
                let c = (self.flags & FLAG_C) != 0;
                let z = (self.flags & FLAG_Z) != 0;
                c && !z
            }
            0xF => {                                      // BLS (unsigned)
                let c = (self.flags & FLAG_C) != 0;
                let z = (self.flags & FLAG_Z) != 0;
                !c || z
            }
            _ => false,
        }
    }

    fn set_flags_add(&mut self, a: u16, b: u16, result: u16, carry: bool) {
        self.flags = 0;
        if result == 0 {
            self.flags |= FLAG_Z;
        }
        if (result & 0x8000) != 0 {
            self.flags |= FLAG_N;
        }
        if carry {
            self.flags |= FLAG_C;
        }
        // Overflow: sign of result differs from sign of both operands
        let a_neg = (a & 0x8000) != 0;
        let b_neg = (b & 0x8000) != 0;
        let r_neg = (result & 0x8000) != 0;
        if a_neg == b_neg && a_neg != r_neg {
            self.flags |= FLAG_V;
        }
    }

    fn set_flags_sub(&mut self, a: u16, b: u16, result: u16, borrow: bool) {
        self.flags = 0;
        if result == 0 {
            self.flags |= FLAG_Z;
        }
        if (result & 0x8000) != 0 {
            self.flags |= FLAG_N;
        }
        if !borrow {
            self.flags |= FLAG_C; // Carry set if no borrow
        }
        // Overflow for subtraction
        let a_neg = (a & 0x8000) != 0;
        let b_neg = (b & 0x8000) != 0;
        let r_neg = (result & 0x8000) != 0;
        if a_neg != b_neg && b_neg == r_neg {
            self.flags |= FLAG_V;
        }
    }

    fn set_flags_logic(&mut self, result: u16) {
        self.flags &= !(FLAG_Z | FLAG_N | FLAG_C | FLAG_V);
        if result == 0 {
            self.flags |= FLAG_Z;
        }
        if (result & 0x8000) != 0 {
            self.flags |= FLAG_N;
        }
    }

    // Debug/trace
    fn trace_instruction(&self, instr: u16) {
        let opcode = (instr >> 12) & 0xF;
        let rd = (instr >> 8) & 0xF;
        let rs1 = (instr >> 4) & 0xF;
        let rs2 = instr & 0xF;

        print!("{:04X}: {:04X}  ", self.pc.wrapping_sub(2), instr);

        match opcode {
            0x0 => println!("ADD R{}, R{}, R{}", rd, rs1, rs2),
            0x1 => println!("SUB R{}, R{}, R{}", rd, rs1, rs2),
            0x2 => println!("AND R{}, R{}, R{}", rd, rs1, rs2),
            0x3 => println!("OR R{}, R{}, R{}", rd, rs1, rs2),
            0x4 => println!("XOR R{}, R{}, R{}", rd, rs1, rs2),
            0x5 => println!("ADDI R{}, {}", rd, (instr & 0xFF) as i8),
            0x6 => println!("LOAD R{}, (R{}) func={}", rd, rs1, rs2),
            0x7 => println!("STORE R{}, (R{}) func={}", rd, rs1, rs2),
            0x8 => println!("BRANCH cond={}, offset={}", rd, (instr & 0xFF) as i8),
            0x9 => println!("JUMP {:03X}", instr & 0xFFF),
            0xA => println!("SHIFT R{}, R{}, func={}", rd, rs1, rs2),
            0xB => println!("MULDIV R{}, R{}, func={}", rd, rs1, rs2),
            0xC => println!("MISC R{}, R{}, func={}", rd, rs1, rs2),
            0xD => println!("I/O R{}, port={}, func={}", rd, rs1, rs2),
            0xE => println!("SYSTEM func={}", rd),
            0xF => println!("EXTENDED R{}, R{}, sub={}", rd, rs1, rs2),
            _ => println!("???"),
        }
    }

    pub fn dump_short(&self) {
        println!(
            "PC={:04X} R4={:04X} R5={:04X} R6={:04X} FLAGS={:02X}",
            self.pc, self.regs[4], self.regs[5], self.regs[6], self.flags
        );
    }

    pub fn dump_state(&self) {
        println!("\n=== CPU State ===");
        println!("PC: {:04X}  Flags: {:02X} [{}{}{}{}]",
            self.pc, self.flags,
            if self.flags & FLAG_N != 0 { 'N' } else { '-' },
            if self.flags & FLAG_Z != 0 { 'Z' } else { '-' },
            if self.flags & FLAG_C != 0 { 'C' } else { '-' },
            if self.flags & FLAG_V != 0 { 'V' } else { '-' },
        );
        println!();

        println!("Registers:");
        for i in 0..4 {
            let base = i * 4;
            println!(
                "  R{:2}={:04X}  R{:2}={:04X}  R{:2}={:04X}  R{:2}={:04X}",
                base, self.regs[base],
                base + 1, self.regs[base + 1],
                base + 2, self.regs[base + 2],
                base + 3, self.regs[base + 3],
            );
        }

        println!();
        println!("Cycles: {}", self.cycles);

        if !self.serial_out.is_empty() {
            println!();
            println!("Serial output:");
            let s: String = self.serial_out.iter().map(|&b| b as char).collect();
            println!("  \"{}\"", s.escape_default());
        }
    }

    pub fn dump_memory(&self, addr: u16, len: usize) {
        println!("Memory at {:04X}:", addr);
        for i in (0..len).step_by(16) {
            let a = addr.wrapping_add(i as u16);
            print!("{:04X}: ", a);
            for j in 0..16 {
                if i + j < len {
                    print!("{:02X} ", self.memory[(a.wrapping_add(j as u16)) as usize]);
                }
            }
            println!();
        }
    }
}
