use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
mod run;


fn parse_operand(operand: &str) -> u8 {
    operand.parse().unwrap_or_else(|_| {
        panic!("Failed to parse operand: {}", operand);
    })
}

fn lexer(mut source: String) -> Vec<u8> {
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
                _ => panic!("Unknown opcode: {}", opcode_str),
            };

            let mut operands = Vec::new();
            while let Some(operand_str) = tokens.next() {
                operands.push(parse_operand(operand_str));
            }

            // Add default operands (0) for instructions with missing operands
            while operands.len() < 2 {
                operands.push(0);
            }

            let instruction = vec![opcode, operands[0], operands[1]];
            program.extend_from_slice(&instruction);
        }
    }
    program
}



fn main() {


    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("AssA emulator");
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
        Err(why) => panic!("Couldn't open {}: {}", display, why),
        Ok(file) => file
    };

    let mut source = String::new();
    match file.read_to_string(&mut source) {
        Err(why) => panic!("Couldn't read {}: {}", display, why),
        _ => {}
    }

    let program = lexer(source);
    run::run_emulation(program, print_usage);
}