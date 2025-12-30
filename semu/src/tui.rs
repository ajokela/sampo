//! TUI (Terminal User Interface) for Sampo Emulator
//!
//! Provides a rich terminal interface with:
//! - CPU register display
//! - Memory viewer
//! - Disassembly view
//! - Terminal emulator with VT220 support
//! - Stack view
//! - Interactive debugging controls

use std::collections::VecDeque;
use std::io::{self, stdout};
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use sysinfo::System;

use crate::cpu::{Cpu, FLAG_C, FLAG_I, FLAG_N, FLAG_V, FLAG_Z};

// Terminal emulator constants
const TERM_COLS: usize = 80;
const TERM_ROWS: usize = 24;

// Execution constants
const TICK_RATE_MS: u64 = 16; // ~60 FPS
const DEFAULT_CYCLES_PER_FRAME: usize = 50000;
const OUTPUT_CHARS_PER_FRAME: usize = 120;

/// VT220 Terminal Emulator
pub struct TerminalEmulator {
    buffer: [[char; TERM_COLS]; TERM_ROWS],
    cursor_row: usize,
    cursor_col: usize,
    cursor_visible: bool,
    escape_state: EscapeState,
    escape_buffer: String,
}

#[derive(Clone, Copy, PartialEq)]
enum EscapeState {
    Normal,
    Escape,
    Csi,
}

impl TerminalEmulator {
    pub fn new() -> Self {
        Self {
            buffer: [[' '; TERM_COLS]; TERM_ROWS],
            cursor_row: 0,
            cursor_col: 0,
            cursor_visible: true,
            escape_state: EscapeState::Normal,
            escape_buffer: String::new(),
        }
    }

    pub fn putchar(&mut self, c: u8) {
        match self.escape_state {
            EscapeState::Normal => self.handle_normal(c),
            EscapeState::Escape => self.handle_escape(c),
            EscapeState::Csi => self.handle_csi(c),
        }
    }

    fn handle_normal(&mut self, c: u8) {
        match c {
            0x1B => {
                self.escape_state = EscapeState::Escape;
                self.escape_buffer.clear();
            }
            0x0D => {
                // Carriage return
                self.cursor_col = 0;
            }
            0x0A => {
                // Line feed
                self.cursor_row += 1;
                if self.cursor_row >= TERM_ROWS {
                    self.scroll_up();
                    self.cursor_row = TERM_ROWS - 1;
                }
            }
            0x08 => {
                // Backspace
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
            }
            0x09 => {
                // Tab
                self.cursor_col = (self.cursor_col + 8) & !7;
                if self.cursor_col >= TERM_COLS {
                    self.cursor_col = TERM_COLS - 1;
                }
            }
            0x07 => {
                // Bell - ignore
            }
            0x20..=0x7E => {
                // Printable character
                if self.cursor_col < TERM_COLS && self.cursor_row < TERM_ROWS {
                    self.buffer[self.cursor_row][self.cursor_col] = c as char;
                    self.cursor_col += 1;
                    if self.cursor_col >= TERM_COLS {
                        self.cursor_col = 0;
                        self.cursor_row += 1;
                        if self.cursor_row >= TERM_ROWS {
                            self.scroll_up();
                            self.cursor_row = TERM_ROWS - 1;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_escape(&mut self, c: u8) {
        match c {
            b'[' => {
                self.escape_state = EscapeState::Csi;
            }
            _ => {
                self.escape_state = EscapeState::Normal;
            }
        }
    }

    fn handle_csi(&mut self, c: u8) {
        if c >= 0x40 && c <= 0x7E {
            // End of CSI sequence
            self.escape_buffer.push(c as char);
            self.execute_csi();
            self.escape_state = EscapeState::Normal;
        } else {
            self.escape_buffer.push(c as char);
        }
    }

    fn execute_csi(&mut self) {
        let seq = &self.escape_buffer;

        if seq.ends_with('H') || seq.ends_with('f') {
            // Cursor position
            let params: Vec<usize> = seq[..seq.len()-1]
                .split(';')
                .filter_map(|s| s.parse().ok())
                .collect();
            self.cursor_row = params.get(0).copied().unwrap_or(1).saturating_sub(1).min(TERM_ROWS - 1);
            self.cursor_col = params.get(1).copied().unwrap_or(1).saturating_sub(1).min(TERM_COLS - 1);
        } else if seq.ends_with('J') {
            // Erase display
            let param: usize = seq[..seq.len()-1].parse().unwrap_or(0);
            match param {
                0 => self.clear_to_end(),
                1 => self.clear_to_start(),
                2 => self.clear_screen(),
                _ => {}
            }
        } else if seq.ends_with('K') {
            // Erase line
            let param: usize = seq[..seq.len()-1].parse().unwrap_or(0);
            match param {
                0 => self.clear_line_to_end(),
                1 => self.clear_line_to_start(),
                2 => self.clear_line(),
                _ => {}
            }
        } else if seq.ends_with('A') {
            // Cursor up
            let n: usize = seq[..seq.len()-1].parse().unwrap_or(1);
            self.cursor_row = self.cursor_row.saturating_sub(n);
        } else if seq.ends_with('B') {
            // Cursor down
            let n: usize = seq[..seq.len()-1].parse().unwrap_or(1);
            self.cursor_row = (self.cursor_row + n).min(TERM_ROWS - 1);
        } else if seq.ends_with('C') {
            // Cursor forward
            let n: usize = seq[..seq.len()-1].parse().unwrap_or(1);
            self.cursor_col = (self.cursor_col + n).min(TERM_COLS - 1);
        } else if seq.ends_with('D') {
            // Cursor back
            let n: usize = seq[..seq.len()-1].parse().unwrap_or(1);
            self.cursor_col = self.cursor_col.saturating_sub(n);
        } else if seq == "?25h" {
            self.cursor_visible = true;
        } else if seq == "?25l" {
            self.cursor_visible = false;
        }
    }

    fn scroll_up(&mut self) {
        for row in 1..TERM_ROWS {
            self.buffer[row - 1] = self.buffer[row];
        }
        self.buffer[TERM_ROWS - 1] = [' '; TERM_COLS];
    }

    fn clear_screen(&mut self) {
        self.buffer = [[' '; TERM_COLS]; TERM_ROWS];
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    fn clear_to_end(&mut self) {
        for col in self.cursor_col..TERM_COLS {
            self.buffer[self.cursor_row][col] = ' ';
        }
        for row in (self.cursor_row + 1)..TERM_ROWS {
            self.buffer[row] = [' '; TERM_COLS];
        }
    }

    fn clear_to_start(&mut self) {
        for col in 0..=self.cursor_col {
            self.buffer[self.cursor_row][col] = ' ';
        }
        for row in 0..self.cursor_row {
            self.buffer[row] = [' '; TERM_COLS];
        }
    }

    fn clear_line(&mut self) {
        self.buffer[self.cursor_row] = [' '; TERM_COLS];
    }

    fn clear_line_to_end(&mut self) {
        for col in self.cursor_col..TERM_COLS {
            self.buffer[self.cursor_row][col] = ' ';
        }
    }

    fn clear_line_to_start(&mut self) {
        for col in 0..=self.cursor_col {
            self.buffer[self.cursor_row][col] = ' ';
        }
    }

    pub fn get_lines(&self) -> Vec<String> {
        self.buffer.iter()
            .map(|row| row.iter().collect::<String>())
            .collect()
    }

    pub fn cursor_position(&self) -> (usize, usize) {
        (self.cursor_row, self.cursor_col)
    }

    pub fn is_cursor_visible(&self) -> bool {
        self.cursor_visible
    }
}

/// Execution state
#[derive(Clone, Copy, PartialEq)]
pub enum RunState {
    Paused,
    Running,
    Halted,
}

/// Application state
pub struct App {
    pub run_state: RunState,
    pub cycles_per_frame: usize,
    pub memory_view_addr: u16,
    pub terminal: TerminalEmulator,
    pub output_buffer: VecDeque<u8>,
    #[allow(dead_code)]
    pub input_buffer: VecDeque<u8>,
    pub cursor_blink: bool,
    pub last_blink: Instant,
    pub effective_mhz: f64,
    pub host_cpu_percent: f32,
    pub host_memory_mb: u64,
    pub last_metrics_update: Instant,
    #[allow(dead_code)]
    pub cycles_this_second: u64,
    pub last_cycle_count: u64,
    pub system: System,
    pub start_pc: u16,
}

impl App {
    pub fn new(start_pc: u16) -> Self {
        Self {
            run_state: RunState::Paused,
            cycles_per_frame: DEFAULT_CYCLES_PER_FRAME,
            memory_view_addr: 0x0100,
            terminal: TerminalEmulator::new(),
            output_buffer: VecDeque::new(),
            input_buffer: VecDeque::new(),
            cursor_blink: true,
            last_blink: Instant::now(),
            effective_mhz: 0.0,
            host_cpu_percent: 0.0,
            host_memory_mb: 0,
            last_metrics_update: Instant::now(),
            cycles_this_second: 0,
            last_cycle_count: 0,
            system: System::new_all(),
            start_pc,
        }
    }

    pub fn update_metrics(&mut self, cpu: &Cpu) {
        let now = Instant::now();
        if now.duration_since(self.last_metrics_update) >= Duration::from_millis(500) {
            let cycles_now = cpu.get_cycles();
            let delta = cycles_now - self.last_cycle_count;
            self.effective_mhz = (delta as f64) / 500_000.0; // MHz
            self.last_cycle_count = cycles_now;

            self.system.refresh_cpu_all();
            self.system.refresh_memory();

            let cpus = self.system.cpus();
            if !cpus.is_empty() {
                self.host_cpu_percent = cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32;
            }
            self.host_memory_mb = self.system.used_memory() / 1024 / 1024;

            self.last_metrics_update = now;
        }

        // Cursor blink
        if now.duration_since(self.last_blink) >= Duration::from_millis(500) {
            self.cursor_blink = !self.cursor_blink;
            self.last_blink = now;
        }
    }

    pub fn flush_output(&mut self) {
        let mut count = 0;
        while count < OUTPUT_CHARS_PER_FRAME && !self.output_buffer.is_empty() {
            if let Some(c) = self.output_buffer.pop_front() {
                self.terminal.putchar(c);
                count += 1;
            }
        }
    }
}

/// Disassemble a single Sampo instruction
pub fn disassemble(cpu: &Cpu, addr: u16) -> (String, u16) {
    let lo = cpu.read_memory(addr);
    let hi = cpu.read_memory(addr.wrapping_add(1));
    let instr = u16::from_le_bytes([lo, hi]);

    let opcode = (instr >> 12) & 0xF;
    let rd = (instr >> 8) & 0xF;
    let rs1 = (instr >> 4) & 0xF;
    let rs2 = instr & 0xF;
    let imm8 = (instr & 0xFF) as i8;
    let func = instr & 0xF;

    let (mnemonic, size) = match opcode {
        0x0 => (format!("ADD R{}, R{}, R{}", rd, rs1, rs2), 2),
        0x1 => (format!("SUB R{}, R{}, R{}", rd, rs1, rs2), 2),
        0x2 => (format!("AND R{}, R{}, R{}", rd, rs1, rs2), 2),
        0x3 => (format!("OR R{}, R{}, R{}", rd, rs1, rs2), 2),
        0x4 => (format!("XOR R{}, R{}, R{}", rd, rs1, rs2), 2),
        0x5 => (format!("ADDI R{}, {}", rd, imm8), 2),
        0x6 => {
            match func {
                0x0 => (format!("LW R{}, (R{})", rd, rs1), 2),
                0x1 => (format!("LB R{}, (R{})", rd, rs1), 2),
                0x2 => (format!("LBU R{}, (R{})", rd, rs1), 2),
                0x8 => (format!("LUI R{}, 0x{:02X}", rd, rs1 << 4), 2),
                _ => (format!("LOAD R{}, (R{}) f={}", rd, rs1, func), 2),
            }
        }
        0x7 => {
            match func {
                0x0 => (format!("SW R{}, (R{})", rd, rs1), 2),
                0x1 => (format!("SB R{}, (R{})", rd, rs1), 2),
                _ => (format!("STORE R{}, (R{}) f={}", rd, rs1, func), 2),
            }
        }
        0x8 => {
            let cond_str = match rd {
                0x0 => "EQ", 0x1 => "NE", 0x2 => "LT", 0x3 => "GE",
                0x4 => "LTU", 0x5 => "GEU", 0x6 => "MI", 0x7 => "PL",
                _ => "??",
            };
            (format!("B{} {:+}", cond_str, imm8 * 2), 2)
        }
        0x9 => {
            if (instr & 0x0F0F) == 0x0F00 {
                (format!("JR R{}", rs1), 2)
            } else if func == 0x1 && rd != 0 {
                (format!("JALR R{}, R{}", rd, rs1), 2)
            } else {
                let offset = (instr & 0x0FFF) as i16;
                let offset = if offset & 0x800 != 0 { offset | 0xF000u16 as i16 } else { offset };
                (format!("J {:+}", offset * 2), 2)
            }
        }
        0xA => {
            let shift_name = match func {
                0x0 => "SLL1", 0x1 => "SRL1", 0x2 => "SRA1",
                0x3 => "ROL1", 0x4 => "ROR1", 0x7 => "SWAP",
                0x8 => "SLL4", 0x9 => "SRL4", 0xA => "SRA4",
                0xC => "SLL8", 0xD => "SRL8", 0xE => "SRA8",
                _ => "SHIFT",
            };
            (format!("{} R{}, R{}", shift_name, rd, rs1), 2)
        }
        0xB => {
            let op = match func {
                0x0 => "MUL", 0x1 => "MULH", 0x2 => "MULHU",
                0x3 => "DIV", 0x4 => "DIVU", 0x5 => "REM",
                0x6 => "REMU", 0x7 => "DAA",
                _ => "MULDIV",
            };
            (format!("{} R{}, R{}", op, rd, rs1), 2)
        }
        0xC => {
            let op = match func {
                0x0 => format!("PUSH R{}", rs1),
                0x1 => format!("POP R{}", rd),
                0x2 => format!("CMP R{}, R{}", rd, rs1),
                0x3 => format!("TEST R{}, R{}", rd, rs1),
                0x4 => format!("MOV R{}, R{}", rd, rs1),
                0x5 => "LDI".to_string(),
                0x6 => "LDD".to_string(),
                0x7 => "LDIR".to_string(),
                0x8 => "LDDR".to_string(),
                0x9 => "CPIR".to_string(),
                0xA => "FILL".to_string(),
                0xB => "EXX".to_string(),
                0xC => format!("GETF R{}", rd),
                0xD => format!("SETF R{}", rs1),
                _ => format!("MISC f={}", func),
            };
            (op, 2)
        }
        0xD => {
            let op = match func {
                0x0 => format!("INI R{}, 0x{:X}", rd, rs1),
                0x1 => format!("OUTI 0x{:X}, R{}", rs1, rd),
                0x2 => format!("IN R{}, (R{})", rd, rs1),
                0x3 => format!("OUT (R{}), R{}", rd, rs1),
                _ => format!("I/O f={}", func),
            };
            (op, 2)
        }
        0xE => {
            let op = match rd {
                0x0 => "NOP".to_string(),
                0x1 => "HALT".to_string(),
                0x2 => "DI".to_string(),
                0x3 => "EI".to_string(),
                0x4 => "RETI".to_string(),
                0x5 => format!("SWI 0x{:02X}", instr & 0xFF),
                0x6 => "SCF".to_string(),
                0x7 => "CCF".to_string(),
                _ => format!("SYS f={}", rd),
            };
            (op, 2)
        }
        0xF => {
            // Extended instruction - need to read imm16
            let lo2 = cpu.read_memory(addr.wrapping_add(2));
            let hi2 = cpu.read_memory(addr.wrapping_add(3));
            let imm16 = u16::from_le_bytes([lo2, hi2]);

            let op = match func {
                0x0 => format!("ADDIX R{}, R{}, 0x{:04X}", rd, rs1, imm16),
                0x1 => format!("SUBIX R{}, R{}, 0x{:04X}", rd, rs1, imm16),
                0x2 => format!("ANDIX R{}, R{}, 0x{:04X}", rd, rs1, imm16),
                0x3 => format!("ORIX R{}, R{}, 0x{:04X}", rd, rs1, imm16),
                0x4 => format!("XORIX R{}, R{}, 0x{:04X}", rd, rs1, imm16),
                0x5 => format!("LWX R{}, 0x{:04X}(R{})", rd, imm16, rs1),
                0x6 => format!("SWX R{}, 0x{:04X}(R{})", rd, imm16, rs1),
                0x7 => format!("LIX R{}, 0x{:04X}", rd, imm16),
                0x8 => format!("JX 0x{:04X}", imm16),
                0x9 => format!("JALX R{}, 0x{:04X}", rd, imm16),
                0xA => format!("CMPIX R{}, 0x{:04X}", rd, imm16),
                0xB => format!("INX R{}, 0x{:02X}", rd, imm16 as u8),
                0xC => format!("OUTX 0x{:02X}, R{}", imm16 as u8, rs1),
                0xD => format!("SLLX R{}, R{}, {}", rd, rs1, imm16 & 0xF),
                0xE => format!("SRLX R{}, R{}, {}", rd, rs1, imm16 & 0xF),
                0xF => format!("SRAX R{}, R{}, {}", rd, rs1, imm16 & 0xF),
                _ => format!("EXT sub={}", func),
            };
            (op, 4)
        }
        _ => (format!("??? {:04X}", instr), 2),
    };

    (mnemonic, size)
}

/// Render the registers panel
fn render_registers(f: &mut Frame, area: Rect, cpu: &Cpu) {
    let flags = cpu.get_flags();

    let mut lines = vec![
        Line::from(vec![
            Span::styled("PC ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:04X}", cpu.get_pc()), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("SP ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:04X}", cpu.get_sp()), Style::default().fg(Color::Yellow)),
            Span::raw("  "),
            Span::styled("Flags ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}{}{}{}{}",
                    if flags & FLAG_N != 0 { 'N' } else { '-' },
                    if flags & FLAG_Z != 0 { 'Z' } else { '-' },
                    if flags & FLAG_C != 0 { 'C' } else { '-' },
                    if flags & FLAG_V != 0 { 'V' } else { '-' },
                    if flags & FLAG_I != 0 { 'I' } else { '-' },
                ),
                Style::default().fg(Color::Cyan)
            ),
        ]),
    ];

    // Register rows (4 registers per row)
    for row in 0..4 {
        let mut spans = vec![];
        for col in 0..4 {
            let r = row * 4 + col;
            let name = match r {
                0 => "R0/ZR",
                1 => "R1/RA",
                2 => "R2/SP",
                3 => "R3/GP",
                4 => "R4/A0",
                5 => "R5/A1",
                6 => "R6/A2",
                7 => "R7/A3",
                8 => "R8/T0",
                9 => "R9/T1",
                10 => "R10/T2",
                11 => "R11/T3",
                12 => "R12/S0",
                13 => "R13/S1",
                14 => "R14/S2",
                15 => "R15/S3",
                _ => "???",
            };
            if col > 0 {
                spans.push(Span::raw(" "));
            }
            spans.push(Span::styled(format!("{:7}", name), Style::default().fg(Color::DarkGray)));
            spans.push(Span::styled(format!("{:04X}", cpu.get_register(r)), Style::default().fg(Color::White)));
        }
        lines.push(Line::from(spans));
    }

    let block = Block::default()
        .title(" Registers ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

/// Render the disassembly panel
fn render_disassembly(f: &mut Frame, area: Rect, cpu: &Cpu) {
    let pc = cpu.get_pc();
    let mut lines = vec![];

    // Show instructions before and after PC
    let mut addr = pc.saturating_sub(8);
    let visible_lines = area.height.saturating_sub(2) as usize;

    for _ in 0..visible_lines {
        let (mnemonic, size) = disassemble(cpu, addr);

        // Get instruction bytes
        let mut bytes = String::new();
        for i in 0..size {
            bytes.push_str(&format!("{:02X}", cpu.read_memory(addr.wrapping_add(i))));
        }

        let is_current = addr == pc;
        let marker = if is_current { ">" } else { " " };

        let style = if is_current {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        lines.push(Line::from(vec![
            Span::styled(marker, Style::default().fg(Color::Green)),
            Span::styled(format!("{:04X} ", addr), Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:8} ", bytes), Style::default().fg(Color::Blue)),
            Span::styled(mnemonic, style),
        ]));

        addr = addr.wrapping_add(size);
    }

    let block = Block::default()
        .title(" Disassembly ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

/// Render the memory viewer
fn render_memory(f: &mut Frame, area: Rect, cpu: &Cpu, view_addr: u16) {
    let mut lines = vec![];
    let visible_lines = area.height.saturating_sub(2) as usize;

    for row in 0..visible_lines {
        let addr = view_addr.wrapping_add((row * 16) as u16);

        let mut hex_spans = vec![
            Span::styled(format!("{:04X}: ", addr), Style::default().fg(Color::DarkGray)),
        ];

        let mut ascii = String::new();
        for col in 0..16 {
            let byte = cpu.read_memory(addr.wrapping_add(col));
            hex_spans.push(Span::styled(format!("{:02X} ", byte), Style::default().fg(Color::White)));
            ascii.push(if byte >= 0x20 && byte < 0x7F { byte as char } else { '.' });
        }

        hex_spans.push(Span::styled(ascii, Style::default().fg(Color::Yellow)));
        lines.push(Line::from(hex_spans));
    }

    let block = Block::default()
        .title(format!(" Memory @ {:04X} ", view_addr))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

/// Render the stack panel
fn render_stack(f: &mut Frame, area: Rect, cpu: &Cpu) {
    let sp = cpu.get_sp();
    let mut lines = vec![];
    let visible_lines = area.height.saturating_sub(2) as usize;

    for i in 0..visible_lines {
        let addr = sp.wrapping_add((i * 2) as u16);
        let lo = cpu.read_memory(addr);
        let hi = cpu.read_memory(addr.wrapping_add(1));
        let val = u16::from_le_bytes([lo, hi]);

        let marker = if i == 0 { ">" } else { " " };

        lines.push(Line::from(vec![
            Span::styled(marker, Style::default().fg(Color::Green)),
            Span::styled(format!("{:04X}: ", addr), Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:04X}", val), Style::default().fg(Color::White)),
        ]));
    }

    let block = Block::default()
        .title(" Stack ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

/// Render the terminal emulator panel
fn render_terminal(f: &mut Frame, area: Rect, app: &App) {
    let term_lines = app.terminal.get_lines();
    let (cursor_row, cursor_col) = app.terminal.cursor_position();

    let mut lines: Vec<Line> = vec![];

    for (row_idx, row) in term_lines.iter().enumerate() {
        if row_idx == cursor_row && app.terminal.is_cursor_visible() && app.cursor_blink {
            // Insert cursor
            let chars: Vec<char> = row.chars().collect();
            if cursor_col < chars.len() {
                let mut spans = vec![];
                spans.push(Span::raw(chars[..cursor_col].iter().collect::<String>()));
                spans.push(Span::styled(
                    chars[cursor_col].to_string(),
                    Style::default().bg(Color::White).fg(Color::Black),
                ));
                if cursor_col + 1 < chars.len() {
                    spans.push(Span::raw(chars[cursor_col + 1..].iter().collect::<String>()));
                }
                lines.push(Line::from(spans));
            } else {
                lines.push(Line::from(row.clone()));
            }
        } else {
            lines.push(Line::from(row.clone()));
        }
    }

    let block = Block::default()
        .title(" Terminal ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

/// Render the status bar
fn render_status(f: &mut Frame, area: Rect, app: &App, cpu: &Cpu) {
    let state_span = match app.run_state {
        RunState::Running => Span::styled("[RUNNING]", Style::default().fg(Color::Green)),
        RunState::Paused => Span::styled("[PAUSED]", Style::default().fg(Color::Yellow)),
        RunState::Halted => Span::styled("[HALTED]", Style::default().fg(Color::Red)),
    };

    let line = Line::from(vec![
        state_span,
        Span::raw(" "),
        Span::styled(format!("{:.2} MHz", app.effective_mhz), Style::default().fg(Color::Cyan)),
        Span::raw("  "),
        Span::styled(format!("CPU:{:.0}%", app.host_cpu_percent), Style::default().fg(Color::Gray)),
        Span::raw("  "),
        Span::styled(format!("Mem:{}MB", app.host_memory_mb), Style::default().fg(Color::Gray)),
        Span::raw("  "),
        Span::styled(format!("Cycles:{}", cpu.get_cycles()), Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled(
            "F5:Run F6:Step F7:Pause F8:Reset F12:Quit",
            Style::default().fg(Color::DarkGray)
        ),
    ]);

    let paragraph = Paragraph::new(vec![line]);
    f.render_widget(paragraph, area);
}

/// Main UI render function
fn ui(f: &mut Frame, app: &App, cpu: &Cpu) {
    // Main layout: content + status bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(f.area());

    // Content area: left (registers + memory) | right (disasm + terminal)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .split(main_chunks[0]);

    // Left panel: registers on top, memory below
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Min(10),
        ])
        .split(content_chunks[0]);

    // Right panel: disassembly + stack on top, terminal below
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(45),
            Constraint::Percentage(55),
        ])
        .split(content_chunks[1]);

    // Upper right: disassembly | stack
    let upper_right_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ])
        .split(right_chunks[0]);

    // Render all panels
    render_registers(f, left_chunks[0], cpu);
    render_memory(f, left_chunks[1], cpu, app.memory_view_addr);
    render_disassembly(f, upper_right_chunks[0], cpu);
    render_stack(f, upper_right_chunks[1], cpu);
    render_terminal(f, right_chunks[1], app);
    render_status(f, main_chunks[1], app, cpu);
}

/// Run the TUI emulator
pub fn run_tui(cpu: &mut Cpu) -> io::Result<()> {
    // Suppress direct stdout output in TUI mode
    cpu.set_quiet(true);

    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let start_pc = cpu.get_pc();
    let mut app = App::new(start_pc);

    let tick_rate = Duration::from_millis(TICK_RATE_MS);

    loop {
        // Draw UI
        terminal.draw(|f| ui(f, &app, cpu))?;

        // Handle input
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                match handle_key(&mut app, cpu, key) {
                    Ok((true, needs_clear)) => {
                        if needs_clear {
                            terminal.clear()?;
                        }
                    }
                    Ok((false, _)) => break, // Quit
                    Err(_) => {}
                }
            }
        }

        // Execute CPU cycles if running
        if app.run_state == RunState::Running && !cpu.is_halted() {
            for _ in 0..app.cycles_per_frame {
                match cpu.step() {
                    Ok(true) => {
                        // Check for serial output
                        let output = cpu.get_serial_output();
                        if !output.is_empty() {
                            for &b in output {
                                app.output_buffer.push_back(b);
                            }
                            cpu.clear_serial_output();
                        }
                    }
                    Ok(false) => {
                        app.run_state = RunState::Halted;
                        break;
                    }
                    Err(_) => {
                        app.run_state = RunState::Halted;
                        break;
                    }
                }
            }
        }

        // Flush output to terminal emulator
        app.flush_output();

        // Update metrics
        app.update_metrics(cpu);
    }

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

/// Handle keyboard input
/// Returns (continue, needs_clear)
fn handle_key(app: &mut App, cpu: &mut Cpu, key: KeyEvent) -> io::Result<(bool, bool)> {
    let mut needs_clear = false;
    match key.code {
        KeyCode::F(5) => {
            // Run
            if !cpu.is_halted() {
                app.run_state = RunState::Running;
            }
        }
        KeyCode::F(6) => {
            // Step
            if !cpu.is_halted() {
                match cpu.step() {
                    Ok(true) => {
                        let output = cpu.get_serial_output();
                        if !output.is_empty() {
                            for &b in output {
                                app.output_buffer.push_back(b);
                            }
                            cpu.clear_serial_output();
                        }
                    }
                    Ok(false) => {
                        app.run_state = RunState::Halted;
                    }
                    Err(_) => {
                        app.run_state = RunState::Halted;
                    }
                }
                app.run_state = RunState::Paused;
            }
        }
        KeyCode::F(7) => {
            // Pause
            app.run_state = RunState::Paused;
        }
        KeyCode::F(8) => {
            // Reset
            cpu.reset();
            cpu.set_pc(app.start_pc);
            app.run_state = RunState::Paused;
            app.terminal = TerminalEmulator::new();
            app.output_buffer.clear();
            needs_clear = true;
        }
        KeyCode::F(9) => {
            // Memory view up
            app.memory_view_addr = app.memory_view_addr.wrapping_sub(16);
        }
        KeyCode::F(10) => {
            // Memory view down
            app.memory_view_addr = app.memory_view_addr.wrapping_add(16);
        }
        KeyCode::PageUp => {
            app.memory_view_addr = app.memory_view_addr.wrapping_sub(256);
        }
        KeyCode::PageDown => {
            app.memory_view_addr = app.memory_view_addr.wrapping_add(256);
        }
        KeyCode::F(12) => {
            return Ok((false, false)); // Quit
        }
        KeyCode::Char('=') if key.modifiers.contains(KeyModifiers::ALT) => {
            // Increase speed
            app.cycles_per_frame = (app.cycles_per_frame + 10000).min(500000);
        }
        KeyCode::Char('-') if key.modifiers.contains(KeyModifiers::ALT) => {
            // Decrease speed
            app.cycles_per_frame = app.cycles_per_frame.saturating_sub(10000).max(1000);
        }
        KeyCode::Char(c) => {
            // Send character to CPU
            if app.run_state == RunState::Running {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Ctrl + key
                    let ctrl_char = (c as u8).wrapping_sub(b'a' - 1);
                    cpu.send_key(ctrl_char);
                } else {
                    cpu.send_key(c as u8);
                }
            }
        }
        KeyCode::Enter => {
            if app.run_state == RunState::Running {
                cpu.send_key(0x0D);
            }
        }
        KeyCode::Backspace => {
            if app.run_state == RunState::Running {
                cpu.send_key(0x08);
            }
        }
        KeyCode::Esc => {
            if app.run_state == RunState::Running {
                cpu.send_key(0x1B);
            }
        }
        KeyCode::Up => {
            if app.run_state == RunState::Running {
                cpu.send_key(0x1B);
                cpu.send_key(b'[');
                cpu.send_key(b'A');
            }
        }
        KeyCode::Down => {
            if app.run_state == RunState::Running {
                cpu.send_key(0x1B);
                cpu.send_key(b'[');
                cpu.send_key(b'B');
            }
        }
        KeyCode::Right => {
            if app.run_state == RunState::Running {
                cpu.send_key(0x1B);
                cpu.send_key(b'[');
                cpu.send_key(b'C');
            }
        }
        KeyCode::Left => {
            if app.run_state == RunState::Running {
                cpu.send_key(0x1B);
                cpu.send_key(b'[');
                cpu.send_key(b'D');
            }
        }
        _ => {}
    }

    Ok((true, needs_clear))
}
