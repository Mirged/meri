const MEMORY_SIZE: usize = 256; // Defines the size of both program memory and RAM in bytes.
const INSTRUCTION_SIZE: u8 = 4; // All instructions are now 4 bytes long.

// Enum to define the type of an operand (Register or Memory).
// This is used internally by the CPU to know how to interpret operand values.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum OperandType {
    Register, // Operand refers to a CPU register (R0-R3).
    Memory,   // Operand refers to a location in RAM (M0-M255).
}

// Represents the CPU state.
#[derive(Debug)]
struct CPU {
    registers: [u8; 4], // 4 general-purpose 8-bit registers (R0-R3).
    memory: [u8; MEMORY_SIZE], // Program memory, where the loaded instructions reside.
    ram: [u8; MEMORY_SIZE], // Data memory, separate from program memory, for data manipulation.
    program_counter: u8, // Points to the address of the current instruction in `memory`.
}

// Enum for the generalized instructions.
// This is a reduced set compared to the previous version, as operations
// now handle different operand types (Reg/Mem) via the `mode_byte`.
#[derive(Debug, PartialEq, Eq)]
pub enum Instructions {
    Mov,       // General purpose move: Moves data between Reg/Reg, Reg/Mem, Mem/Reg.
    Add,       // General purpose add: Adds values between Reg/Reg, Reg/Mem, Mem/Reg.
    Sub,       // General purpose subtract: Subtracts values between Reg/Reg, Reg/Mem, Mem/Reg.
    Inc,       // General purpose increment: Increments a Reg or Mem location by 1.
    Dec,       // General purpose decrement: Decrements a Reg or Mem location by 1.
    JmpAddr,   // Jump to address: Sets the program counter to a specific address.
    HLT,       // Halt execution: Stops the CPU.
}

// Helper function to safely read a value from a register or memory based on operand type.
// Returns a Result to propagate errors (e.g., invalid register index or memory address).
fn get_operand_value(cpu: &CPU, operand_type: OperandType, address_or_index: u8) -> Result<u8, String> {
    match operand_type {
        OperandType::Register => {
            if address_or_index as usize >= cpu.registers.len() {
                return Err(format!("Runtime error: Invalid register index: {}. PC: {}", address_or_index, cpu.program_counter));
            }
            Ok(cpu.registers[address_or_index as usize])
        },
        OperandType::Memory => {
            if address_or_index as usize >= cpu.ram.len() {
                return Err(format!("Runtime error: Invalid memory address: {}. PC: {}", address_or_index, cpu.program_counter));
            }
            Ok(cpu.ram[address_or_index as usize])
        },
    }
}

// Helper function to safely write a value to a register or memory based on operand type.
// Returns a Result to propagate errors.
fn set_operand_value(cpu: &mut CPU, operand_type: OperandType, address_or_index: u8, value: u8) -> Result<(), String> {
    match operand_type {
        OperandType::Register => {
            if address_or_index as usize >= cpu.registers.len() {
                return Err(format!("Runtime error: Invalid register index: {}. PC: {}", address_or_index, cpu.program_counter));
            }
            cpu.registers[address_or_index as usize] = value;
        },
        OperandType::Memory => {
            if address_or_index as usize >= cpu.ram.len() {
                return Err(format!("Runtime error: Invalid memory address: {}. PC: {}", address_or_index, cpu.program_counter));
            }
            cpu.ram[address_or_index as usize] = value;
        },
    }
    Ok(())
}

// Executes a single instruction.
// This function implements the "under the hood" logic, branching based on operand types.
// It takes `OperandType` parameters to determine whether `dest_val_or_addr` and `src_val_or_addr`
// refer to registers or memory locations.
fn execute_instruction(
    cpu: &mut CPU,
    opcode: Instructions,
    dest_type: OperandType,     // Type of the destination operand (Reg/Mem)
    dest_val_or_addr: u8,       // Value (register index or memory address) for destination
    src_type: OperandType,      // Type of the source operand (Reg/Mem)
    src_val_or_addr: u8,        // Value (register index or memory address) for source
) -> Result<(), String> {
    match opcode {
        Instructions::Mov => {
            // Lower-level operation: Read source value.
            let src_value = get_operand_value(cpu, src_type, src_val_or_addr)?;
            // Lower-level operation: Write to destination.
            set_operand_value(cpu, dest_type, dest_val_or_addr, src_value)?;
        }
        Instructions::Add => {
            // Lower-level operation: Read source value.
            let src_value = get_operand_value(cpu, src_type, src_val_or_addr)?;
            // Lower-level operation: Read destination value.
            let mut dest_value = get_operand_value(cpu, dest_type, dest_val_or_addr)?;
            // Lower-level operation: Perform addition (wrapping to handle overflow).
            dest_value = dest_value.wrapping_add(src_value);
            // Lower-level operation: Write result back to destination.
            set_operand_value(cpu, dest_type, dest_val_or_addr, dest_value)?;
        }
        Instructions::Sub => {
            // Lower-level operation: Read source value.
            let src_value = get_operand_value(cpu, src_type, src_val_or_addr)?;
            // Lower-level operation: Read destination value.
            let mut dest_value = get_operand_value(cpu, dest_type, dest_val_or_addr)?;
            // Lower-level operation: Perform subtraction (wrapping to handle underflow).
            dest_value = dest_value.wrapping_sub(src_value);
            // Lower-level operation: Write result back to destination.
            set_operand_value(cpu, dest_type, dest_val_or_addr, dest_value)?;
        }
        Instructions::Inc => {
            // Inc only uses the destination operand. src_type and src_val_or_addr are ignored.
            let mut val = get_operand_value(cpu, dest_type, dest_val_or_addr)?;
            val = val.wrapping_add(1); // Wrapping add for increment
            set_operand_value(cpu, dest_type, dest_val_or_addr, val)?;
        }
        Instructions::Dec => {
            // Dec only uses the destination operand. src_type and src_val_or_addr are ignored.
            let mut val = get_operand_value(cpu, dest_type, dest_val_or_addr)?;
            val = val.wrapping_sub(1); // Wrapping sub for decrement
            set_operand_value(cpu, dest_type, dest_val_or_addr, val)?;
        }
        Instructions::JmpAddr => {
            // JmpAddr uses dest_val_or_addr as the target address.
            // dest_type, src_type, and src_val_or_addr are effectively ignored.
            // The lexer ensures dest_val_or_addr is a valid address.
            cpu.program_counter = dest_val_or_addr;
        }
        Instructions::HLT => {
            // HLT is handled directly in run_program to break the loop.
            // No operation performed here, just a placeholder for the enum.
        }
    }
    Ok(())
}

// Loads the program bytes into the CPU's program memory.
fn load_program(cpu: &mut CPU, program: &[u8]) {
    for (i, &instruction_byte) in program.iter().enumerate() {
        if i < cpu.memory.len() { // Ensure we don't write beyond memory bounds
            cpu.memory[i] = instruction_byte;
        } else {
            eprintln!("Warning: Program exceeds memory size. Instruction at index {} ignored.", i);
            break;
        }
    }
}

// Runs the loaded program in the CPU.
// It fetches, decodes, and executes instructions sequentially.
// Returns a Result to indicate if any runtime errors occurred (e.g., unknown opcode, invalid address).
fn run_program(cpu: &mut CPU, program_size: usize) -> Result<(), String> {
    while (cpu.program_counter as usize) < program_size {
        // Check if there are enough bytes for a full 4-byte instruction
        if (cpu.program_counter as usize) + (INSTRUCTION_SIZE as usize) > program_size {
            return Err(format!("Program ended unexpectedly at PC {}. Incomplete instruction.", cpu.program_counter));
        }

        // Fetch the 4 bytes of the current instruction
        let opcode_val = cpu.memory[cpu.program_counter as usize];
        let mode_byte = cpu.memory[(cpu.program_counter + 1) as usize];
        let operand1_val = cpu.memory[(cpu.program_counter + 2) as usize];
        let operand2_val = cpu.memory[(cpu.program_counter + 3) as usize];

        // Convert the opcode byte to an `Instructions` enum variant.
        // `try_from` will return an error if the opcode is unknown.
        let opcode = Instructions::try_from(opcode_val)?;

        // If the instruction is HLT, print message and terminate execution.
        if opcode == Instructions::HLT {
            println!("Halted.");
            return Ok(());
        }

        // Decode operand types from the `mode_byte`:
        // Bit 0 (0b01) controls dest_type: 1 means Memory, 0 means Register.
        // Bit 1 (0b10) controls src_type: 1 means Memory, 0 means Register.
        let dest_type = if (mode_byte & 0b01) != 0 { OperandType::Memory } else { OperandType::Register };
        let src_type = if (mode_byte & 0b10) != 0 { OperandType::Memory } else { OperandType::Register };

        // Execute the decoded instruction with its operands and types.
        // Errors from `execute_instruction` (e.g., invalid register/memory access) are propagated.
        execute_instruction(
            cpu,
            opcode,
            dest_type,
            operand1_val,
            src_type,
            operand2_val,
        )?;

        // Move the program counter to the next instruction.
        // Since all instructions are 4 bytes, we always increment by INSTRUCTION_SIZE.
        if opcode != Instructions::JmpAddr { // Only increment if not a jump
            cpu.program_counter += INSTRUCTION_SIZE;
        }
        // For JmpAddr, the PC is set directly in execute_instruction, so we don't increment here.
    }
    Ok(())
}

// Implements the `TryFrom` trait to safely convert a `u8` (opcode byte) into an `Instructions` enum.
// This allows error handling for invalid opcode values.
impl TryFrom<u8> for Instructions {
    type Error = String; // The error type for conversion failures.

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Instructions::Mov),
            1 => Ok(Instructions::Add),
            2 => Ok(Instructions::Sub),
            3 => Ok(Instructions::Inc),
            4 => Ok(Instructions::Dec),
            5 => Ok(Instructions::JmpAddr),
            6 => Ok(Instructions::HLT),
            _ => Err(format!("Unknown instruction opcode: {}", value)), // Return an error for unrecognized opcodes.
        }
    }
}

// Public function to start the emulation process.
pub fn run_emulation(program_vector: Vec<u8>, print_usage: bool) {
    // Initialize CPU with all registers and memory set to 0.
    let mut cpu = CPU {
        registers: [0, 0, 0, 0],
        memory: [0; MEMORY_SIZE], // Program memory
        ram: [0; MEMORY_SIZE],    // Data memory
        program_counter: 0,
    };

    // Load the provided program into the CPU's memory.
    let program = &program_vector[..];
    load_program(&mut cpu, &program);

    // Run the program and handle any emulation errors.
    if let Err(e) = run_program(&mut cpu, program.len()) {
        eprintln!("Emulation error: {}", e);
    }

    // If `--print-state` flag is set, print the final CPU state.
    if print_usage {
        println!("################### CPU STATE AFTER PROGRAM ###################");
        println!("PC = {}", cpu.program_counter);
        println!(
            "reg1 = {}, reg2 = {}, reg3 = {}, reg4 = {}",
            cpu.registers[0], cpu.registers[1], cpu.registers[2], cpu.registers[3]
        );
        // Print a snippet of RAM contents for debugging.
        println!("RAM contents (first 10 bytes): {:?}", &cpu.ram[0..10]);
    }
}
