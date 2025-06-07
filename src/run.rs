const MEMORY_SIZE: usize = 256;

#[derive(Debug)]
struct CPU {
    registers: [u8; 4],  // 4 general-purpose registers
    memory: [u8; MEMORY_SIZE],
    ram: [u8; MEMORY_SIZE],
    program_counter: u8,
}

#[derive(Debug)]
pub enum Instructions {
    MovRegReg,
    AddRegReg,
    SubRegReg,
    IncReg,
    DecReg,
    JmpAddr,
    MovRegMem,
    MovMemReg,
    AddRegMem,
    AddMemReg,
    SubRegMem,
    SubMemReg,
    IncMem,
    DecMem,
    HLT,
}

fn execute_instruction(cpu: &mut CPU, opcode: Instructions, operand1: u8, operand2: u8) {
    match opcode {
        Instructions::MovRegReg => cpu.registers[operand1 as usize] = cpu.registers[operand2 as usize],
        Instructions::AddRegReg => cpu.registers[operand1 as usize] += cpu.registers[operand2 as usize],
        Instructions::SubRegReg => cpu.registers[operand1 as usize] -= cpu.registers[operand2 as usize],
        Instructions::IncReg => cpu.registers[operand1 as usize] += 1,
        Instructions::DecReg => cpu.registers[operand1 as usize] -= 1,
        Instructions::JmpAddr => cpu.program_counter = operand1,
        Instructions::MovRegMem => cpu.registers[operand1 as usize] = cpu.ram[operand2 as usize],
        Instructions::MovMemReg => cpu.ram[operand1 as usize] = cpu.registers[operand2 as usize],
        Instructions::AddRegMem => cpu.registers[operand1 as usize] += cpu.ram[operand2 as usize],
        Instructions::AddMemReg => cpu.ram[operand1 as usize] += cpu.registers[operand2 as usize],
        Instructions::SubRegMem => cpu.registers[operand1 as usize] -= cpu.ram[operand2 as usize],
        Instructions::SubMemReg => cpu.ram[operand1 as usize] -= cpu.registers[operand2 as usize],
        Instructions::IncMem => cpu.ram[operand1 as usize] += 1,
        Instructions::DecMem => cpu.ram[operand1 as usize] -= 1,
        Instructions::HLT => {
            println!("Halted.");
        }
    }
}

fn load_program(cpu: &mut CPU, program: &[u8]) {
    for (i, &instruction) in program.iter().enumerate() {
        cpu.memory[i] = instruction;
    }
}

// run_program now returns a Result to indicate if an error occurred during execution (e.g., unknown instruction)
fn run_program(cpu: &mut CPU, program_size: usize) -> Result<(), String> {
    while cpu.program_counter < program_size as u8 {
        let opcode_val = cpu.memory[cpu.program_counter as usize];
        let operand1 = cpu.memory[(cpu.program_counter + 1) as usize];
        let operand2 = cpu.memory[(cpu.program_counter + 2) as usize];

        // Safely convert u8 to Instructions, propagating errors
        let opcode = Instructions::try_from(opcode_val)?;

        if opcode as u8 == Instructions::HLT as u8 {
            println!("Halted.");
            return Ok(());
        }
        execute_instruction(cpu, opcode, operand1, operand2);

        // Move to the next instruction
        cpu.program_counter += 3;
    }
    Ok(())
}

impl TryFrom<u8> for Instructions {
    type Error = String; // Define the error type for the conversion

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Instructions::MovRegReg),
            1 => Ok(Instructions::AddRegReg),
            2 => Ok(Instructions::SubRegReg),
            3 => Ok(Instructions::IncReg),
            4 => Ok(Instructions::DecReg),
            5 => Ok(Instructions::JmpAddr),
            6 => Ok(Instructions::MovRegMem),
            7 => Ok(Instructions::MovMemReg),
            8 => Ok(Instructions::AddRegMem),
            9 => Ok(Instructions::AddMemReg),
            10 => Ok(Instructions::SubRegMem),
            11 => Ok(Instructions::SubMemReg),
            12 => Ok(Instructions::IncMem),
            13 => Ok(Instructions::DecMem),
            14 => Ok(Instructions::HLT),
            _ => Err(format!("Unknown instruction opcode: {}", value)), // Return an error
        }
    }
}

pub fn run_emulation(program_vector: Vec<u8>, print_usage: bool) {
    let mut cpu = CPU {
        registers: [0, 0, 0, 0],
        memory: [0; MEMORY_SIZE],
        ram: [0; MEMORY_SIZE],
        program_counter: 0,
    };

    let program = &program_vector[..];
    load_program(&mut cpu, &program);
    if let Err(e) = run_program(&mut cpu, program.len()) {
        eprintln!("Emulation error: {}", e);
    }
    if print_usage {
        println!("################### CPU STATE AFTER PROGRAM ###################");
        println!("PC = {}", cpu.program_counter);
        println!(
            "reg1 = {}, reg2 = {}, reg3 = {}, reg4 = {}",
            cpu.registers[0], cpu.registers[1], cpu.registers[2], cpu.registers[3]
        );
    }
}
