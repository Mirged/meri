const MEMORY_SIZE: usize = 256; // Defines the size of both program memory and RAM in bytes.
const INSTRUCTION_SIZE: u8 = 4; // All instructions are now 4 bytes long.

// Enum to define the type of an operand (Register or Memory).
// This is used internally by the CPU to know how to interpret operand values.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum OperandType {
    Register, // Operand refers to a CPU register (R0-R3).
    Memory,   // Operand refers to a location in RAM (M0-M255).
}

// Bitmasks for CPU flags
const FLAG_ZERO: u8 = 0b00000001; // Zero Flag: set if the result of an operation is zero
const FLAG_CARRY: u8 = 0b00000010; // Carry Flag: set if an arithmetic operation produced a carry/borrow

// Represents the CPU state.
#[derive(Debug)]
struct CPU {
    registers: [u8; 4], // 4 general-purpose 8-bit registers (R0-R3).
    memory: [u8; MEMORY_SIZE], // Program memory, where the loaded instructions reside.
    ram: [u8; MEMORY_SIZE], // Data memory, separate from program memory, for data manipulation.
    program_counter: u8, // Points to the address of the current instruction in `memory`.
    flags: u8, // 8-bit register to hold status flags (Zero, Carry, etc.)
}

impl CPU {
    // Helper to set a specific flag
    fn set_flag(&mut self, flag: u8) {
        self.flags |= flag;
    }

    // Helper to clear a specific flag
    fn clear_flag(&mut self, flag: u8) {
        self.flags &= !flag;
    }

    // Helper to check if a specific flag is set
    fn is_flag_set(&self, flag: u8) -> bool {
        (self.flags & flag) != 0
    }

    // Update Zero and Carry flags based on an operation's result and carry_out status
    fn update_flags(&mut self, result: u8, carry_out: bool) {
        if result == 0 {
            self.set_flag(FLAG_ZERO);
        } else {
            self.clear_flag(FLAG_ZERO);
        }

        if carry_out {
            self.set_flag(FLAG_CARRY);
        } else {
            self.clear_flag(FLAG_CARRY);
        }
    }
}


// Enum for the generalized instructions.
// This is a reduced set compared to the previous version, as operations
// now handle different operand types (Reg/Mem) via the `mode_byte`.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Instructions {
    Mov,       // General purpose move: Moves data between Reg/Reg, Reg/Mem, Mem/Reg.
    MovImm,    // Move Immediate: Moves a constant value into a Reg or Mem location.
    Add,       // General purpose add: Adds values between Reg/Reg, Reg/Mem, Mem/Reg.
    Sub,       // General purpose subtract: Subtracts values between Reg/Reg, Reg/Mem, Mem/Reg.
    Inc,       // General purpose increment: Increments a Reg or Mem location by 1.
    Dec,       // General purpose decrement: Decrements a Reg or Mem location by 1.
    Cmp,       // Compare: Compares two operands and sets flags (Zero, Carry).
    JmpAddr,   // Jump to address: Sets the program counter to a specific address unconditionally.
    JmpEq,     // Jump if Equal: Jumps if Zero Flag is set.
    JmpNe,     // Jump if Not Equal: Jumps if Zero Flag is clear.
    JmpGt,     // Jump if Greater Than: Jumps if Zero Flag is clear AND Carry Flag is clear (for unsigned).
    HLT,       // Halt execution: Stops the CPU.
}

// Helper function to safely read a value from a register or memory based on operand type.
// Returns a Result to propagate errors (e.g., invalid register index or memory address).
fn get_operand_value(cpu: &CPU, operand_type: OperandType, address_or_index: u8, debug_context: &str) -> Result<u8, String> {
    match operand_type {
        OperandType::Register => {
            if address_or_index as usize >= cpu.registers.len() {
                return Err(format!("Runtime error: Invalid register index {} for {} operand. PC: {}", address_or_index, debug_context, cpu.program_counter));
            }
            Ok(cpu.registers[address_or_index as usize])
        },
        OperandType::Memory => {
            if address_or_index as usize >= cpu.ram.len() {
                return Err(format!("Runtime error: Invalid memory address {} for {} operand. PC: {}", address_or_index, debug_context, cpu.program_counter));
            }
            Ok(cpu.ram[address_or_index as usize])
        },
    }
}

// Helper function to safely write a value to a register or memory based on operand type.
// Returns a Result to propagate errors.
fn set_operand_value(cpu: &mut CPU, operand_type: OperandType, address_or_index: u8, value: u8, debug_context: &str) -> Result<(), String> {
    match operand_type {
        OperandType::Register => {
            if address_or_index as usize >= cpu.registers.len() {
                return Err(format!("Runtime error: Invalid register index {} for {} operand. PC: {}", address_or_index, debug_context, cpu.program_counter));
            }
            cpu.registers[address_or_index as usize] = value;
        },
        OperandType::Memory => {
            if address_or_index as usize >= cpu.ram.len() {
                return Err(format!("Runtime error: Invalid memory address {} for {} operand. PC: {}", address_or_index, debug_context, cpu.program_counter));
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
            let src_value = get_operand_value(cpu, src_type, src_val_or_addr, "Mov source")?;
            // Lower-level operation: Write to destination.
            set_operand_value(cpu, dest_type, dest_val_or_addr, src_value, "Mov destination")?;
        }
        Instructions::MovImm => {
            // For MovImm, src_val_or_addr is the immediate value itself.
            // src_type is ignored for MovImm.
            set_operand_value(cpu, dest_type, dest_val_or_addr, src_val_or_addr, "MovImm destination")?;
        }
        Instructions::Add => {
            // Lower-level operation: Read source value.
            let src_value = get_operand_value(cpu, src_type, src_val_or_addr, "Add source")?;
            // Lower-level operation: Read destination value.
            let mut dest_value = get_operand_value(cpu, dest_type, dest_val_or_addr, "Add destination read")?;
            // Perform addition and get carry status.
            let (result, carry) = dest_value.overflowing_add(src_value);
            dest_value = result;
            // Update flags based on the result and carry.
            cpu.update_flags(dest_value, carry);
            // Lower-level operation: Write result back to destination.
            set_operand_value(cpu, dest_type, dest_val_or_addr, dest_value, "Add destination write")?;
        }
        Instructions::Sub => {
            // Lower-level operation: Read source value.
            let src_value = get_operand_value(cpu, src_type, src_val_or_addr, "Sub source")?;
            // Lower-level operation: Read destination value.
            let mut dest_value = get_operand_value(cpu, dest_type, dest_val_or_addr, "Sub destination read")?;
            // Perform subtraction and get borrow status (overflowing_sub for unsigned).
            let (result, borrow) = dest_value.overflowing_sub(src_value);
            dest_value = result;
            // Update flags based on the result and borrow (carry flag often used for borrow in sub).
            cpu.update_flags(dest_value, borrow); // Borrow sets carry flag for unsigned subtraction
            // Lower-level operation: Write result back to destination.
            set_operand_value(cpu, dest_type, dest_val_or_addr, dest_value, "Sub destination write")?;
        }
        Instructions::Inc => {
            // Inc only uses the destination operand. src_type and src_val_or_addr are ignored.
            let mut val = get_operand_value(cpu, dest_type, dest_val_or_addr, "Inc operand read")?;
            let (result, carry) = val.overflowing_add(1);
            val = result;
            cpu.update_flags(val, carry);
            set_operand_value(cpu, dest_type, dest_val_or_addr, val, "Inc operand write")?;
        }
        Instructions::Dec => {
            // Dec only uses the destination operand. src_type and src_val_or_addr are ignored.
            let mut val = get_operand_value(cpu, dest_type, dest_val_or_addr, "Dec operand read")?;
            let (result, borrow) = val.overflowing_sub(1);
            val = result;
            cpu.update_flags(val, borrow); // Borrow sets carry flag for unsigned subtraction
            set_operand_value(cpu, dest_type, dest_val_or_addr, val, "Dec operand write")?;
        }
        Instructions::Cmp => {
            // Compare: Calculates dest - src and sets flags without storing the result.
            // dest_val_or_addr is operand1, src_val_or_addr is operand2
            let op1_value = get_operand_value(cpu, dest_type, dest_val_or_addr, "Cmp operand1")?;
            let op2_value = get_operand_value(cpu, src_type, src_val_or_addr, "Cmp operand2")?;

            // Perform subtraction to set flags. We only care about the flags, not the result.
            let (result, borrow) = op1_value.overflowing_sub(op2_value);
            cpu.update_flags(result, borrow);
        }
        Instructions::JmpAddr => {
            // JmpAddr uses dest_val_or_addr as the target address.
            cpu.program_counter = dest_val_or_addr;
        }
        Instructions::JmpEq => {
            // Jump if Equal (ZF is set)
            if cpu.is_flag_set(FLAG_ZERO) {
                cpu.program_counter = dest_val_or_addr;
            } else {
                cpu.program_counter += INSTRUCTION_SIZE; // No jump, move to next instruction
            }
        }
        Instructions::JmpNe => {
            // Jump if Not Equal (ZF is clear)
            if !cpu.is_flag_set(FLAG_ZERO) {
                cpu.program_counter = dest_val_or_addr;
            } else {
                cpu.program_counter += INSTRUCTION_SIZE; // No jump, move to next instruction
            }
        }
        Instructions::JmpGt => {
            // Jump if Greater Than (ZF is clear AND Carry Flag is clear) for unsigned comparison
            // If A > B, then A - B does not borrow and result is not zero.
            if !cpu.is_flag_set(FLAG_ZERO) && !cpu.is_flag_set(FLAG_CARRY) {
                cpu.program_counter = dest_val_or_addr;
            } else {
                cpu.program_counter += INSTRUCTION_SIZE; // No jump, move to next instruction
            }
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

        // For jump instructions, PC is handled within execute_instruction.
        // For all other instructions, we advance PC by INSTRUCTION_SIZE.
        match opcode {
            Instructions::JmpAddr | Instructions::JmpEq | Instructions::JmpNe | Instructions::JmpGt => {
                // PC was already set/incremented inside execute_instruction. Do nothing here.
            },
            _ => {
                // For all non-jump instructions, advance PC to the next instruction.
                cpu.program_counter += INSTRUCTION_SIZE;
            }
        }
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
            1 => Ok(Instructions::MovImm),
            2 => Ok(Instructions::Add),
            3 => Ok(Instructions::Sub),
            4 => Ok(Instructions::Inc),
            5 => Ok(Instructions::Dec),
            6 => Ok(Instructions::Cmp),      // New opcode for Cmp
            7 => Ok(Instructions::JmpAddr),  // Opcode for JmpAddr (shifted)
            8 => Ok(Instructions::JmpEq),    // New opcode for JmpEq
            9 => Ok(Instructions::JmpNe),    // New opcode for JmpNe
            10 => Ok(Instructions::JmpGt),   // New opcode for JmpGt
            11 => Ok(Instructions::HLT),     // HLT opcode (shifted)
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
        flags: 0, // Initialize flags to 0
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
        println!("Flags (binary): {:08b}", cpu.flags);
        println!("  Zero Flag (ZF): {}", cpu.is_flag_set(FLAG_ZERO));
        println!("  Carry Flag (CF): {}", cpu.is_flag_set(FLAG_CARRY));
        // Print a snippet of RAM contents for debugging.
        println!("RAM contents (first 10 bytes): {:?}", &cpu.ram[0..10]);
    }
}
