#![feature(duration_millis_float)]

#[derive(Debug)]
enum Errors {}

#[derive(Clone, Copy, Debug)]
enum InstructionArgument {
    Register(&'static str),
    Value(u16),
}

#[derive(Clone, Copy, Debug)]
/** Everytime a whole instruction is completed,
its result will be pushed to the "res" register */
enum CPUInstruction {
    /** ADD instruction | reg/value + reg/value */
    Add(InstructionArgument, InstructionArgument),
    /** SUB instruction | reg/value - reg/value */
    Sub(InstructionArgument, InstructionArgument),
    /** MOV instruction | reg/value -> reg |
    Moves the first value (or register's content) into another register */
    Mov(InstructionArgument, InstructionArgument),
    /** GOTO instruction | Jumps to the instruction at the provided address and executes it

    Use with caution, it is powerful but can have side-effects
    or can lead to undefined behavior */
    Goto(usize),
    /** IF instruction |
    IF reg/value >= 1 then go to first address, ELSE go to second fallback address */
    If(InstructionArgument, usize, usize),
    /** EQ instruction | reg/value == reg/value |
    Compares the two values and returns 0 if the comparison is false, 1 if it's true */
    Eq(InstructionArgument, InstructionArgument),
    /** EXIT instruction | Sets the status of the CPU to "exiting" */
    Exit(),
}

#[derive(Default)]
/**
    a, b = general-use register

    res = used to store the result of the last instruction
*/
struct Registers {
    a: u16,
    b: u16,
    res: u16,
}

#[derive(PartialEq)]
enum Status {
    NotStarted,
    Running,
    Exiting,
}
struct CpuState {
    frequency: u32,
    /** The minimum duration of an instruction cycle */
    cycle_duration: usize,
    instruction_cache: Vec<CPUInstruction>,
    instruction_address: usize,
    registers: Registers,
    status: Status,
}
impl CpuState {
    fn new(frequency: u32) -> CpuState {
        let mut cpu_state = CpuState {
            frequency: 0,
            cycle_duration: 0,
            instruction_cache: vec![],
            instruction_address: 0,
            registers: Default::default(),
            status: Status::NotStarted,
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

    fn append_instructions(&mut self, instructions: &[CPUInstruction]) {
        self.instruction_cache.append(&mut instructions.to_owned());
    }

    fn fetch_argument_value(&self, argument: InstructionArgument) -> u16 {
        match argument {
            InstructionArgument::Register(register_name) => *self.get_register(register_name),
            InstructionArgument::Value(value) => value,
        }
    }

    fn handle_instruction(&mut self, instruction: CPUInstruction) {
        match instruction {
            CPUInstruction::Add(a, b) => {
                let a = self.fetch_argument_value(a);
                let b = self.fetch_argument_value(b);

                self.registers.res = a + b
            }
            CPUInstruction::Sub(a, b) => {
                let a = self.fetch_argument_value(a);
                let b = self.fetch_argument_value(b);

                self.registers.res = a - b
            }
            CPUInstruction::Mov(from, to) => {
                let from = self.fetch_argument_value(from);
                match to {
                    InstructionArgument::Register(register_name) => {
                        let register = self.get_register_mut(register_name);

                        *register = from;
                    }
                    InstructionArgument::Value(_) => {
                        panic!(
                            "Cannot move a value or a register to another value, must be a register"
                        )
                    }
                };
            }
            CPUInstruction::Goto(new_address) => self.instruction_address = new_address,
            CPUInstruction::If(boolean, first_address, second_address) => {
                let boolean = self.fetch_argument_value(boolean);

                if boolean >= 1 {
                    self.instruction_address = first_address;
                } else {
                    self.instruction_address = second_address;
                }
            }
            CPUInstruction::Eq(first, second) => {
                let first = self.fetch_argument_value(first);
                let second = self.fetch_argument_value(second);

                self.registers.res = (first == second) as u16
            }
            CPUInstruction::Exit() => self.status = Status::Exiting,
        }
    }

    fn execute(mut self) {
        let mut total_instructions = 0;

        let start = std::time::Instant::now();
        loop {
            if self.instruction_address >= self.instruction_cache.len() {
                break;
            } else if let Status::Exiting = self.status {
                break;
            }

            let instruction_start = std::time::Instant::now();
            // Simulate one CPU instruction
            let current_instruction = self.instruction_cache[self.instruction_address];
            self.handle_instruction(current_instruction);
            println!("{}: {}", self.instruction_address, self.registers.res);

            match current_instruction {
                CPUInstruction::Goto(_) | CPUInstruction::If(_, _, _) => {}
                _ => {
                    // Increment the instruction address
                    self.instruction_address += 1;
                }
            }

            // Increment the total amount of instructions executed
            total_instructions += 1;

            // Sleep to maintain CPU frequency
            let elapsed = instruction_start.elapsed().as_millis_f64();
            let expected = self.cycle_duration as f64;
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
    let mut cpu = CpuState::new(100);

    let instructions = vec![
        CPUInstruction::Add(
            InstructionArgument::Register("a"),
            InstructionArgument::Value(1),
        ),
        CPUInstruction::Mov(
            InstructionArgument::Register("res"),
            InstructionArgument::Register("a"),
        ),
        CPUInstruction::Eq(
            InstructionArgument::Register("a"),
            InstructionArgument::Value(10),
        ),
        CPUInstruction::If(InstructionArgument::Register("res"), 100, 0),
    ];
    cpu.append_instructions(&instructions);
    cpu.execute();
}
