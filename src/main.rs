#![feature(duration_millis_float)]

#[derive(Debug)]
enum Errors {}

#[derive(Clone, Copy, Debug)]
enum InstructionArgument {
    Register(&'static str),
    Value(u16),
}

#[derive(Clone, Copy, Debug)]
enum CPUInstruction {
    Add(InstructionArgument, InstructionArgument),
    Sub(InstructionArgument, InstructionArgument),
    Mov(InstructionArgument, InstructionArgument),
}

#[derive(Default)]
struct Registers {
    a: u16,
    b: u16,
    // Read-only - The result of the last instruction
    res: u16,
}
struct CpuState {
    frequency: u32,
    cycle_duration: usize,
    instruction_cache: Vec<CPUInstruction>,
    instruction_index: usize,
    registers: Registers,
}
impl CpuState {
    fn new(frequency: u32) -> CpuState {
        let mut cpu_state = CpuState {
            frequency: 0,
            cycle_duration: 0,
            instruction_cache: vec![],
            instruction_index: 0,
            registers: Default::default(),
        };
        // Important for consistent pacing of CPU cycles
        cpu_state.update_frequency(frequency);

        cpu_state
    }

    fn update_frequency(&mut self, new_frequency: u32) {
        self.frequency = new_frequency;
        self.cycle_duration = 1000 / new_frequency as usize;
    }

    fn get_register(&self, register_name: &'static str) -> &u16 {
        match register_name {
            "a" => &self.registers.a,
            "b" => &self.registers.b,
            "res" => &self.registers.res,
            _ => panic!("Register {register_name} not found"),
        }
    }
    fn get_register_mut(&mut self, register_name: &'static str) -> &mut u16 {
        match register_name {
            "a" => &mut self.registers.a,
            "b" => &mut self.registers.b,
            "res" => &mut self.registers.res,
            _ => panic!("Register {register_name} not found"),
        }
    }

    fn fetch_argument_value(&self, argument: InstructionArgument) -> u16 {
        match argument {
            InstructionArgument::Register(register_name) => *self.get_register(register_name),
            InstructionArgument::Value(value) => value,
        }
    }

    fn handle_instruction(&mut self, instruction: CPUInstruction) -> u16 {
        match instruction {
            CPUInstruction::Add(a, b) => {
                let a = self.fetch_argument_value(a);
                let b = self.fetch_argument_value(b);

                a + b
            }
            CPUInstruction::Sub(a, b) => {
                let a = self.fetch_argument_value(a);
                let b = self.fetch_argument_value(b);

                a - b
            }
            CPUInstruction::Mov(from, to) => {
                let from = self.fetch_argument_value(from);
                match to {
                    InstructionArgument::Register(register_name) => {
                        if register_name != "res" {
                            let register = self.get_register_mut(register_name);

                            *register = from;
                        }
                    }
                    InstructionArgument::Value(_) => {
                        panic!(
                            "Cannot move a value or a register to another value, must be a register"
                        )
                    }
                };

                from
            }
        }
    }

    fn execute(mut self) {
        let mut total_instructions = 0;

        let start = std::time::Instant::now();
        while self.instruction_index < self.instruction_cache.len() {
            // Simulate one CPU instruction
            let current_instruction = self.instruction_cache[self.instruction_index];
            self.registers.res = self.handle_instruction(current_instruction);
            println!("{}: {}", self.instruction_index, self.registers.res);

            // Increment the instruction index offset
            self.instruction_index += 1;
            // Increment the total amount of instructions executed
            total_instructions += 1;

            // Sleep to maintain cpu frequency
            let elapsed = start.elapsed().as_millis_f64();
            let expected = (self.instruction_index * self.cycle_duration) as f64;
            if elapsed < expected {
                let sleep_duration = expected - elapsed;
                std::thread::sleep(std::time::Duration::from_millis(sleep_duration as u64));
            }
        }
        println!(
            "Completed {total_instructions} CPU instructions in {} seconds",
            start.elapsed().as_secs_f64()
        );
    }
}

fn main() {
    let mut cpu = CpuState::new(10_000);

    let instructions = vec![
        CPUInstruction::Add(InstructionArgument::Value(1), InstructionArgument::Value(1)),
        CPUInstruction::Mov(
            InstructionArgument::Register("res"),
            InstructionArgument::Register("a"),
        ),
        CPUInstruction::Sub(
            InstructionArgument::Value(10),
            InstructionArgument::Value(5),
        ),
        CPUInstruction::Sub(
            InstructionArgument::Register("res"),
            InstructionArgument::Register("a"),
        ),
    ];

    cpu.instruction_cache = instructions;
    cpu.execute();
}
