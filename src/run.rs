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

fn run_program(cpu: &mut CPU, program_size: usize) {
    while cpu.program_counter < program_size as u8 {
        let opcode = cpu.memory[cpu.program_counter as usize];
        let operand1 = cpu.memory[(cpu.program_counter + 1) as usize];
        let operand2 = cpu.memory[(cpu.program_counter + 2) as usize];

        if opcode as u8 == Instructions::HLT as u8 {
            println!("Halted.");
            return;
        }
        execute_instruction(cpu, Instructions::from(opcode), operand1, operand2);

        // Move to the next instruction
        cpu.program_counter += 3;
    }
}

impl From<u8> for Instructions {
    fn from(value: u8) -> Self {
        match value {
            0 => Instructions::MovRegReg,
            1 => Instructions::AddRegReg,
            2 => Instructions::SubRegReg,
            3 => Instructions::IncReg,
            4 => Instructions::DecReg,
            5 => Instructions::JmpAddr,
            6 => Instructions::MovRegMem,
            7 => Instructions::MovMemReg,
            8 => Instructions::AddRegMem,
            9 => Instructions::AddMemReg,
            10 => Instructions::SubRegMem,
            11 => Instructions::SubMemReg,
            12 => Instructions::IncMem,
            13 => Instructions::DecMem,
            14 => Instructions::HLT,
            _ => panic!("Unknown instruction: {}", value),
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
    run_program(&mut cpu, program.len());
    if print_usage {
        println!("################### CPU STATE AFTER PROGRAM ###################");
        println!("PC = {}", cpu.program_counter);
        println!(
            "reg1 = {}, reg2 = {}, reg3 = {}, reg4 = {}",
            cpu.registers[0], cpu.registers[1], cpu.registers[2], cpu.registers[3]
        );
    }
}
