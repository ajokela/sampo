//! Sampo CPU Emulator (semu)
//! Emulator for the Sampo 16-bit RISC CPU

use std::env;
use std::fs;
use std::io::{self, Read, Write};

mod cpu;

use cpu::Cpu;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: semu <program.bin> [options]");
        eprintln!("       semu --help");
        std::process::exit(1);
    }

    if args[1] == "--help" || args[1] == "-h" {
        print_help();
        return;
    }

    let input_file = &args[1];
    let trace = args.iter().any(|a| a == "-t" || a == "--trace");
    let interactive = args.iter().any(|a| a == "-i" || a == "--interactive");

    // Load program
    let program = match fs::read(input_file) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error reading {}: {}", input_file, e);
            std::process::exit(1);
        }
    };

    // Create and initialize CPU
    let mut cpu = Cpu::new();
    cpu.load_program(&program);
    cpu.set_trace(trace);

    println!("Sampo Emulator - Loaded {} bytes", program.len());
    println!("Starting execution at 0x{:04X}", cpu.get_pc());
    println!();

    if interactive {
        run_interactive(&mut cpu);
    } else {
        run(&mut cpu);
    }
}

fn run(cpu: &mut Cpu) {
    loop {
        match cpu.step() {
            Ok(true) => {} // Continue
            Ok(false) => {
                println!("\nCPU halted at 0x{:04X}", cpu.get_pc());
                break;
            }
            Err(e) => {
                eprintln!("\nError at 0x{:04X}: {}", cpu.get_pc(), e);
                cpu.dump_state();
                std::process::exit(1);
            }
        }
    }
    cpu.dump_state();
}

fn run_interactive(cpu: &mut Cpu) {
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        print!("semu> ");
        io::stdout().flush().unwrap();

        input.clear();
        if stdin.read_line(&mut input).is_err() {
            break;
        }

        let cmd = input.trim();
        match cmd {
            "s" | "step" => {
                match cpu.step() {
                    Ok(true) => cpu.dump_short(),
                    Ok(false) => {
                        println!("CPU halted");
                        break;
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            "r" | "run" => {
                run(cpu);
                break;
            }
            "d" | "dump" => {
                cpu.dump_state();
            }
            "m" | "mem" => {
                cpu.dump_memory(cpu.get_pc(), 32);
            }
            "q" | "quit" => {
                break;
            }
            "h" | "help" => {
                println!("Commands:");
                println!("  s, step  - Execute one instruction");
                println!("  r, run   - Run until halt");
                println!("  d, dump  - Dump CPU state");
                println!("  m, mem   - Dump memory at PC");
                println!("  q, quit  - Exit");
            }
            _ => {
                if !cmd.is_empty() {
                    println!("Unknown command: {}", cmd);
                }
            }
        }
    }
}

fn print_help() {
    println!("Sampo Emulator (semu) v0.1.0");
    println!();
    println!("Usage: semu <program.bin> [options]");
    println!();
    println!("Options:");
    println!("  -t, --trace       Trace execution");
    println!("  -i, --interactive Interactive mode");
    println!("  -h, --help        Show this help message");
}
