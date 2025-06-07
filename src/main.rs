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
fn lexer(mut source: String) -> Result<Vec<u8>, String> {
    let mut program = Vec::new();
    source.retain(|c| c != '\n'); // Remove newline characters for simpler splitting.
    let lines: Vec<&str> = source.split(";").collect(); // Split the source code into individual instruction lines by semicolon.
    
    // Process each line (instruction) individually.
    for part in lines {
        let trimmed_part = part.trim(); // Remove leading/trailing whitespace from the instruction line.
        if trimmed_part.is_empty() { // Skip empty lines, e.g., if there's a trailing semicolon.
            continue;
        }

        // Split the instruction line into tokens (opcode and operands).
        let mut tokens = trimmed_part.split_whitespace();
        // The first token is expected to be the opcode string.
        let opcode_str = tokens.next().ok_or_else(|| "Empty instruction line.".to_string())?;

        // All instructions are now 4 bytes: [opcode, mode_byte, operand1_val, operand2_val].
        // Initialize with default values.
        let (mut opcode_val, mut mode_byte, mut operand1_val, mut operand2_val) = (0, 0, 0, 0);

        // Match on the opcode string to determine instruction type and parse operands.
        match opcode_str {
            "Mov" | "Add" | "Sub" => {
                // These instructions expect two operands (destination and source).
                let dest_str = tokens.next().ok_or_else(|| format!("Missing destination operand for instruction '{}'. Expected format: {} <DEST> <SOURCE>", opcode_str, opcode_str))?;
                let src_str = tokens.next().ok_or_else(|| format!("Missing source operand for instruction '{}'. Expected format: {} <DEST> <SOURCE>", opcode_str, opcode_str))?;

                // Parse destination and source operands using the helper function.
                let (dest_val, dest_type) = parse_reg_mem_operand(dest_str)?;
                let (src_val, src_type) = parse_reg_mem_operand(src_str)?;

                // Store parsed operand values.
                operand1_val = dest_val;
                operand2_val = src_val;

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
                opcode_val = match opcode_str {
                    "Mov" => 0,
                    "Add" => 1,
                    "Sub" => 2,
                    _ => unreachable!(), // This case should theoretically not be reached.
                };
            },
            "Inc" | "Dec" => {
                // These instructions expect one operand.
                let op_str = tokens.next().ok_or_else(|| format!("Missing operand for instruction '{}'. Expected format: {} <OPERAND>", opcode_str, opcode_str))?;
                let (op_val, op_type) = parse_reg_mem_operand(op_str)?;

                // Store the parsed operand value.
                operand1_val = op_val;
                // operand2_val remains 0, as it's not used by these single-operand instructions.

                // Encode addressing mode for the single operand into the `mode_byte`.
                if op_type == OperandType::Memory {
                    mode_byte |= 0b01; // Only set the destination bit as it's the only operand.
                }

                // Assign the numerical opcode.
                opcode_val = match opcode_str {
                    "Inc" => 3,
                    "Dec" => 4,
                    _ => unreachable!(),
                };
            },
            "JmpAddr" => {
                // JmpAddr expects one numeric address operand.
                let addr_str = tokens.next().ok_or_else(|| format!("Missing address for instruction '{}'. Expected format: JmpAddr <ADDRESS>", opcode_str))?;
                operand1_val = addr_str.parse::<u8>()
                    .map_err(|e| format!("Invalid jump address '{}': {}", addr_str, e))?;
                
                // mode_byte and operand2_val remain 0 as they are not applicable for JmpAddr.
                opcode_val = 5;
            },
            "HLT" => {
                // HLT takes no operands. All operand values and mode_byte remain 0.
                opcode_val = 6;
            },
            _ => return Err(format!("Unknown opcode: {}", opcode_str)), // Error for unrecognized instruction.
        }
        
        // After parsing, check if there are any unexpected extra tokens on the line.
        if tokens.next().is_some() {
            return Err(format!("Too many operands or unexpected tokens for instruction '{}' on line: '{}'.", opcode_str, trimmed_part));
        }

        // Assemble the 4-byte instruction and add it to the program byte vector.
        program.push(opcode_val);
        program.push(mode_byte);
        program.push(operand1_val);
        program.push(operand2_val);
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
