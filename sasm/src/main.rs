//! Sampo Assembler (sasm)
//! Assembler for the Sampo 16-bit RISC CPU

use std::env;
use std::fs;

mod lexer;
mod parser;
mod codegen;

use lexer::Lexer;
use parser::Parser;
use codegen::CodeGen;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: sasm <input.s> [-o output.bin]");
        eprintln!("       sasm --help");
        std::process::exit(1);
    }

    if args[1] == "--help" || args[1] == "-h" {
        print_help();
        return;
    }

    let input_file = &args[1];
    let output_file = if args.len() >= 4 && args[2] == "-o" {
        args[3].clone()
    } else {
        input_file.replace(".s", ".bin").replace(".asm", ".bin")
    };

    let source = match fs::read_to_string(input_file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading {}: {}", input_file, e);
            std::process::exit(1);
        }
    };

    // Lexical analysis
    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Lexer error: {}", e);
            std::process::exit(1);
        }
    };

    // Parsing
    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parser error: {}", e);
            std::process::exit(1);
        }
    };

    // Code generation
    let mut codegen = CodeGen::new();
    let binary = match codegen.generate(&program) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Code generation error: {}", e);
            std::process::exit(1);
        }
    };

    // Write output
    match fs::write(&output_file, &binary) {
        Ok(_) => {
            println!("Assembled {} -> {} ({} bytes)", input_file, output_file, binary.len());
        }
        Err(e) => {
            eprintln!("Error writing {}: {}", output_file, e);
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("Sampo Assembler (sasm) v0.1.0");
    println!();
    println!("Usage: sasm <input.s> [-o output.bin]");
    println!();
    println!("Options:");
    println!("  -o <file>    Output file (default: input with .bin extension)");
    println!("  -h, --help   Show this help message");
    println!();
    println!("Registers:");
    println!("  R0/ZERO  R1/RA   R2/SP   R3/GP");
    println!("  R4/A0    R5/A1   R6/A2   R7/A3");
    println!("  R8/T0    R9/T1   R10/T2  R11/T3");
    println!("  R12/S0   R13/S1  R14/S2  R15/S3");
    println!();
    println!("Directives:");
    println!("  .org <addr>     Set origin address");
    println!("  .equ <sym> <v>  Define constant");
    println!("  .db <bytes>     Define bytes");
    println!("  .dw <words>     Define words");
    println!("  .ascii \"str\"    Define ASCII string");
    println!("  .asciz \"str\"    Define null-terminated string");
}
