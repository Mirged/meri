use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
mod run; // Import the `run` module which contains CPU, instructions, and emulation logic.

// Import `OperandType` from the `run` module so `lexer` can use it.
use run::OperandType;

// Local constants for lexer error checking, mirroring the CPU's architecture limits.
const LEXER_MEMORY_SIZE: usize = 256;
const LEXER_REGISTER_COUNT: usize = 4;

// Helper function for the lexer to parse register (R#) or memory (M#) operands.
// It returns the numerical value (index or address) and its corresponding `OperandType`.
fn parse_reg_mem_operand(operand_str: &str) -> Result<(u8, OperandType), String> {
    if operand_str.starts_with('R') {
        // Parse register index
        let reg_idx = operand_str[1..].parse::<u8>()
            .map_err(|e| format!("Invalid register index '{}': {}", operand_str, e))?;
        // Validate register index bounds
        if reg_idx as usize >= LEXER_REGISTER_COUNT {
            return Err(format!("Register index {} out of bounds (max {}).", reg_idx, LEXER_REGISTER_COUNT - 1));
        }
        Ok((reg_idx, OperandType::Register))
    } else if operand_str.starts_with('M') {
        // Parse memory address
        let mem_addr = operand_str[1..].parse::<u8>()
            .map_err(|e| format!("Invalid memory address '{}': {}", operand_str, e))?;
        // Validate memory address bounds
        if mem_addr as usize >= LEXER_MEMORY_SIZE {
            return Err(format!("Memory address {} out of bounds (max {}).", mem_addr, LEXER_MEMORY_SIZE - 1));
        }
        Ok((mem_addr, OperandType::Memory))
    } else {
        // If neither R# nor M# format is found, it's an error for this type of operand.
        Err(format!("Expected register (R#) or memory (M#) operand, found '{}'.", operand_str))
    }
}

// The lexer function converts human-readable assembly source code into a byte vector
// that the Meri CPU emulator can execute.
// It now handles the new generalized instruction syntax and encodes addressing modes.
fn lexer(source: String) -> Result<Vec<u8>, String> {
    let mut program = Vec::new();
    
    // Split the source code into individual lines first, and track line numbers
    for (line_num, line) in source.lines().enumerate() {
        // Ignore anything after a "//" comment
        let instruction_part = line.split("//").next().unwrap_or("").trim();

        // Skip empty lines or lines that were entirely comments
        if instruction_part.is_empty() {
            continue;
        }

        // Split the instruction line by semicolon to handle multiple instructions on one line
        // (though current examples usually have one per line)
        let parts: Vec<&str> = instruction_part.split(";").collect();

        for part in parts {
            let trimmed_part = part.trim(); // Remove leading/trailing whitespace
            if trimmed_part.is_empty() {
                continue;
            }

            // Split the instruction line into tokens (opcode and operands).
            let mut tokens = trimmed_part.split_whitespace();
            // The first token is expected to be the opcode string.
            let opcode_str = tokens.next().ok_or_else(|| format!("Line {}: Empty instruction part after semicolon.", line_num + 1))?;

            // Variables to hold the components of the 4-byte instruction.
            let instruction_bytes: [u8; 4] = match opcode_str {
                "Mov" | "Add" | "Sub" | "Cmp" => { // Cmp added here
                    // These instructions expect two operands (destination and source).
                    let dest_str = tokens.next().ok_or_else(|| format!("Line {}: Missing destination operand for instruction '{}'. Expected format: {} <DEST> <SOURCE>", line_num + 1, opcode_str, opcode_str))?;
                    let src_str = tokens.next().ok_or_else(|| format!("Line {}: Missing source operand for instruction '{}'. Expected format: {} <DEST> <SOURCE>", line_num + 1, opcode_str, opcode_str))?;

                    // Parse destination and source operands using the helper function.
                    let (dest_val, dest_type) = parse_reg_mem_operand(dest_str)
                        .map_err(|e| format!("Line {}: {}", line_num + 1, e))?;
                    let (src_val, src_type) = parse_reg_mem_operand(src_str)
                        .map_err(|e| format!("Line {}: {}", line_num + 1, e))?;

                    let mut mode_byte = 0; // Initialize mode byte to 0

                    // Encode addressing modes into the `mode_byte`:
                    // Bit 0 (0b01) for destination type: 1 if Memory, 0 if Register.
                    // Bit 1 (0b10) for source type: 1 if Memory, 0 if Register.
                    if dest_type == OperandType::Memory {
                        mode_byte |= 0b01;
                    }
                    if src_type == OperandType::Memory {
                        mode_byte |= 0b10;
                    }

                    // Assign the numerical opcode based on the instruction string.
                    let opcode_val = match opcode_str {
                        "Mov" => 0,
                        "Add" => 2,
                        "Sub" => 3,
                        "Cmp" => 6, // Opcode for Cmp
                        _ => unreachable!(), // This case should theoretically not be reached.
                    };
                    [opcode_val, mode_byte, dest_val, src_val]
                },
                "MovImm" => {
                    // MovImm expects a destination (R#/M#) and an immediate value.
                    let dest_str = tokens.next().ok_or_else(|| format!("Line {}: Missing destination operand for instruction '{}'. Expected format: {} <DEST> <VALUE>", line_num + 1, opcode_str, opcode_str))?;
                    let value_str = tokens.next().ok_or_else(|| format!("Line {}: Missing immediate value for instruction '{}'. Expected format: {} <DEST> <VALUE>", line_num + 1, opcode_str, opcode_str))?;

                    let (dest_val, dest_type) = parse_reg_mem_operand(dest_str)
                        .map_err(|e| format!("Line {}: {}", line_num + 1, e))?;
                    
                    let immediate_value = value_str.parse::<u8>()
                        .map_err(|e| format!("Line {}: Invalid immediate value '{}': {}", line_num + 1, value_str, e))?;

                    let mut mode_byte = 0;
                    // Encode destination type into mode_byte. Source type is irrelevant for MovImm.
                    if dest_type == OperandType::Memory {
                        mode_byte |= 0b01;
                    }
                    // Opcode for MovImm
                    [1, mode_byte, dest_val, immediate_value]
                },
                "Inc" | "Dec" => {
                    // These instructions expect one operand.
                    let op_str = tokens.next().ok_or_else(|| format!("Line {}: Missing operand for instruction '{}'. Expected format: {} <OPERAND>", line_num + 1, opcode_str, opcode_str))?;
                    let (op_val, op_type) = parse_reg_mem_operand(op_str)
                        .map_err(|e| format!("Line {}: {}", line_num + 1, e))?;

                    let mut mode_byte = 0;
                    // Encode addressing mode for the single operand into the `mode_byte`.
                    if op_type == OperandType::Memory {
                        mode_byte |= 0b01; // Only set the destination bit as it's the only operand.
                    }

                    // Assign the numerical opcode.
                    let opcode_val = match opcode_str {
                        "Inc" => 4,
                        "Dec" => 5,
                        _ => unreachable!(),
                    };
                    [opcode_val, mode_byte, op_val, 0] // operand2_val is 0 for single-operand instructions
                },
                // New conditional jump instructions
                "JmpAddr" | "JmpEq" | "JmpNe" | "JmpGt" => { // JmpEq, JmpNe, JmpGt added here
                    // These instructions expect one numeric address operand.
                    let addr_str = tokens.next().ok_or_else(|| format!("Line {}: Missing address for instruction '{}'. Expected format: {} <ADDRESS>", line_num + 1, opcode_str, opcode_str))?;
                    let address_val = addr_str.parse::<u8>()
                        .map_err(|e| format!("Line {}: Invalid jump address '{}': {}", line_num + 1, addr_str, e))?;
                    
                    // mode_byte and operand2_val remain 0 as they are not applicable for jumps.
                    let opcode_val = match opcode_str {
                        "JmpAddr" => 7,
                        "JmpEq" => 8,
                        "JmpNe" => 9,
                        "JmpGt" => 10,
                        _ => unreachable!(),
                    };
                    [opcode_val, 0, address_val, 0]
                },
                "HLT" => {
                    // HLT takes no operands. All operand values and mode_byte remain 0.
                    [11, 0, 0, 0]
                },
                _ => return Err(format!("Line {}: Unknown opcode: {}", line_num + 1, opcode_str)), // Error for unrecognized instruction.
            };
            
            // After parsing, check if there are any unexpected extra tokens on the line.
            if tokens.next().is_some() {
                return Err(format!("Line {}: Too many operands or unexpected tokens for instruction '{}' on line: '{}'.", line_num + 1, opcode_str, trimmed_part));
            }

            // Assemble the 4-byte instruction and add it to the program byte vector.
            program.extend_from_slice(&instruction_bytes);
        }
    }
    
    Ok(program) // Return the successfully lexed program as a byte vector.
}

// Main entry point of the emulator.
fn main() {
    let args: Vec<String> = env::args().collect(); // Collect command line arguments.

    // Display usage information if not enough arguments are provided.
    if args.len() < 2 {
        println!("Meri emulator");
        println!("Usage:\n {} <file_path> [OPTIONS]", args[0]);
        println!("OPTIONS:\n --print-state - Print CPU state after program execution");
        return;
    }

    // Parse command line flags.
    let mut print_usage: bool = false;
    if args.len() > 2 {
        for arg in args.iter().skip(2) { // Skip the program name and file path.
            match arg.as_str() {
                "--print-state" => print_usage = true, // Set flag to print CPU state.
                _ => { /* Ignore unknown options */ }
            }
        }
    }

    // Get the assembly file path from arguments.
    let file_name = &args[1];
    let path = Path::new(file_name);
    let display = path.display();

    // Attempt to open the specified assembly file.
    let mut file = match File::open(path) {
        Err(why) => {
            eprintln!("Error: Couldn't open {}: {}", display, why); // Print error to stderr.
            return; // Exit program.
        }
        Ok(file) => file,
    };

    // Attempt to read the file content into a String.
    let mut source = String::new();
    if let Err(why) = file.read_to_string(&mut source) {
        eprintln!("Error: Couldn't read {}: {}", display, why); // Print error to stderr.
        return; // Exit program.
    }

    // Lex the source code into an executable program byte vector.
    // Handle potential lexer errors.
    let program = match lexer(source) {
        Ok(p) => p, // If successful, get the program bytes.
        Err(e) => {
            eprintln!("Lexer error: {}", e); // Print lexer error.
            return; // Exit program.
        }
    };

    // Run the emulation with the lexed program and the print_usage flag.
    run::run_emulation(program, print_usage);
}
