use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
mod run;

// Changed to return a Result for better error handling
fn parse_operand(operand: &str) -> Result<u8, String> {
    operand.parse().map_err(|e| format!("Failed to parse operand '{}': {}", operand, e))
}

fn lexer(mut source: String) -> Result<Vec<u8>, String> {
    let mut program = Vec::new();
    source.retain(|c| c != '\n');
    let parts = source.split(";");
    for part in parts {
        let mut tokens = part.split_whitespace();
        if let Some(opcode_str) = tokens.next() {
            let opcode: u8 = match opcode_str {
                "MovRegReg" => 0,
                "AddRegReg" => 1,
                "SubRegReg" => 2,
                "IncReg" => 3,
                "DecReg" => 4,
                "JmpAddr" => 5,
                "MovRegMem" => 6,
                "MovMemReg" => 7,
                "AddRegMem" => 8,
                "AddMemReg" => 9,
                "SubRegMem" => 10,
                "SubMemReg" => 11,
                "IncMem" => 12,
                "DecMem" => 13,
                "HLT" => 14,
                _ => return Err(format!("Unknown opcode: {}", opcode_str)), // Return error for unknown opcode
            };

            let mut operands = Vec::new();
            while let Some(operand_str) = tokens.next() {
                operands.push(parse_operand(operand_str)?); // Use '?' to propagate errors from parse_operand
            }

            // Add default operands (0) for instructions with missing operands
            while operands.len() < 2 {
                operands.push(0);
            }

            let instruction = vec![opcode, operands[0], operands[1]];
            program.extend_from_slice(&instruction);
        }
    }
    Ok(program)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Meri emulator");
        println!("Usage:\n {} <file_path> [OPTIONS]", args[0]);
        println!("OPTIONS:\n --print-state - Print CPU state after program execution");
        return;
    }
    // FLAGS
    let mut print_usage: bool = false;

    if args.len() > 2 {
        for arg in args.iter().skip(2) {
            match arg.as_str() {
                "--print-state" => print_usage = true,
                _ => {}
            }
        }
    }

    let file_name: Vec<_> = env::args().collect();

    let path = Path::new(file_name[1].as_str());
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => {
            eprintln!("Error: Couldn't open {}: {}", display, why); // Use eprintln for errors
            return;
        }
        Ok(file) => file,
    };

    let mut source = String::new();
    if let Err(why) = file.read_to_string(&mut source) {
        eprintln!("Error: Couldn't read {}: {}", display, why); // Use eprintln for errors
        return;
    }

    let program = match lexer(source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Lexer error: {}", e);
            return;
        }
    };
    run::run_emulation(program, print_usage);
}
