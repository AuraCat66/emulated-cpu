#![feature(duration_millis_float)]

use std::collections::HashMap;

use memory::MemoryState;

mod memory;

// #[derive(Debug)]
// enum Errors {}

#[derive(Clone, Copy, Debug)]
enum InstructionArgument {
    /* Get a value from an address in the current sub stack */
    Stack(u16),
    /* Get a value from a register */
    Register(&'static str),
    /* A hard-coded value */
    Value(u16),
}

#[derive(Clone, Debug)]
/** Everytime a whole instruction is completed,
its result will be pushed to the "res" register */
enum CpuInstruction {
    /** ADD instruction | reg/value + reg/value */
    Add(InstructionArgument, InstructionArgument),
    /** SUB instruction | reg/value - reg/value */
    Sub(InstructionArgument, InstructionArgument),
    /** MOV instruction | reg/value -> reg |
    Moves the first value (or register's content) into another register */
    Mov(InstructionArgument, InstructionArgument),

    /** EQ instruction | reg/value == reg/value |
    Compares the two values and returns 0 if the comparison is false, 1 if it's true */
    Eq(InstructionArgument, InstructionArgument),

    /** FN function | Declares a function. Does nothing when actually executed */
    Fn(&'static str),
    /** RET instruction | Returns from the current function */
    Ret(),
    /** CALL instruction | Calls a function */
    Call(&'static str),

    /** GOTO instruction | Jumps to the instruction at the provided address and executes it
    Use with caution, it is powerful but can have side-effects
    or can lead to undefined behavior */
    Goto(u16),
    /** IF instruction |
    IF reg/value >= 1 then execute the first instruction, ELSE execute the second fall-back instruction */
    If(
        InstructionArgument,
        Box<CpuInstruction>,
        Box<CpuInstruction>,
    ),

    /** EXIT instruction | Sets the status of the CPU to "exiting" */
    Exit(),
}

#[derive(Default)]
/**
    a, b = general-use register

    res = used to store the result of the last instruction
*/
struct CpuRegisters {
    a: u16,
    b: u16,
    c: u16,
    d: u16,
    res: u16,
}

enum CpuStatus {
    NotStarted,
    Running,
    Exiting,
}
struct CpuState {
    frequency: u16,
    /** The minimum duration of an instruction cycle */
    cycle_duration: usize,
    status: CpuStatus,
    instruction_cache: Vec<CpuInstruction>,
    instruction_pointer: u16,
    registers: CpuRegisters,
    memory: MemoryState,
    function_table: HashMap<&'static str, u16>,
}
impl CpuState {
    fn new(frequency: u16) -> CpuState {
        let mut cpu_state = CpuState {
            frequency: 0,
            cycle_duration: 0,
            status: CpuStatus::NotStarted,
            instruction_cache: vec![],
            instruction_pointer: 0,
            registers: Default::default(),
            memory: MemoryState::default(),
            function_table: HashMap::new(),
        };
        // Important for consistent pacing of CPU cycles
        cpu_state.update_frequency(frequency);

        cpu_state
    }

    fn update_frequency(&mut self, new_frequency: u16) {
        self.frequency = new_frequency;
        self.cycle_duration = 1000 / new_frequency as usize;
    }

    fn get_register(&self, register_name: &'static str) -> &u16 {
        match register_name {
            "a" => &self.registers.a,
            "b" => &self.registers.b,
            "c" => &self.registers.c,
            "d" => &self.registers.d,
            "res" => &self.registers.res,
            _ => panic!("Register {register_name} not found"),
        }
    }
    fn get_register_mut(&mut self, register_name: &'static str) -> &mut u16 {
        match register_name {
            "a" => &mut self.registers.a,
            "b" => &mut self.registers.b,
            "c" => &mut self.registers.c,
            "d" => &mut self.registers.d,
            "res" => &mut self.registers.res,
            _ => panic!("Register {register_name} not found"),
        }
    }

    fn register_functions(&mut self, instructions: &[CpuInstruction]) {
        instructions
            .iter()
            .enumerate()
            .for_each(|(i, instruction)| {
                if let CpuInstruction::Fn(fn_name) = instruction {
                    self.function_table.insert(fn_name, i as u16);
                }
            });
    }

    fn append_instructions(&mut self, instructions: &[CpuInstruction]) {
        self.register_functions(instructions);
        self.instruction_cache.append(&mut instructions.to_owned());
    }

    fn fetch_argument_value(&mut self, argument: InstructionArgument) -> u16 {
        match argument {
            InstructionArgument::Stack(address) => {
                if self
                    .memory
                    .get_current_sub_stack()
                    .data
                    .get(address as usize)
                    .is_none()
                {
                    self.memory.write_data(address, 0);
                }

                self.memory.get_current_sub_stack().data[address as usize]
            }
            InstructionArgument::Register(register_name) => *self.get_register(register_name),
            InstructionArgument::Value(value) => value,
        }
    }

    fn handle_instruction(&mut self, instruction: CpuInstruction) {
        match instruction {
            CpuInstruction::Add(a, b) => {
                let a = self.fetch_argument_value(a);
                let b = self.fetch_argument_value(b);

                self.registers.res = a + b
            }
            CpuInstruction::Sub(a, b) => {
                let a = self.fetch_argument_value(a);
                let b = self.fetch_argument_value(b);

                self.registers.res = a - b
            }
            CpuInstruction::Mov(from, to) => {
                let from = self.fetch_argument_value(from);
                match to {
                    InstructionArgument::Stack(address) => {
                        self.memory.write_data(address, from);
                    }
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
            CpuInstruction::Eq(first, second) => {
                let first = self.fetch_argument_value(first);
                let second = self.fetch_argument_value(second);

                self.registers.res = (first == second) as u16
            }
            CpuInstruction::Fn(_) => {}
            CpuInstruction::Ret() => {
                let return_address = self.memory.get_current_sub_stack().return_address;
                self.memory.rewind_stack();
                self.instruction_pointer = return_address;
            }
            CpuInstruction::Call(fn_name) => {
                self.memory.create_new_sub_stack(self.instruction_pointer);
                self.instruction_pointer = *self.function_table.get(fn_name).unwrap();
            }
            CpuInstruction::Goto(new_address) => {
                self.instruction_pointer = new_address;
            }
            CpuInstruction::If(boolean, first, second) => {
                let boolean = self.fetch_argument_value(boolean);

                if boolean >= 1 {
                    self.handle_instruction(*first);
                } else {
                    self.handle_instruction(*second);
                }
            }
            CpuInstruction::Exit() => self.status = CpuStatus::Exiting,
        }
    }

    fn execute(mut self) {
        if !self.function_table.contains_key("main") {
            panic!("No \"main\" function detected, cannot execute program");
        } else {
            self.append_instructions(&[CpuInstruction::Call("main")]);
            self.instruction_pointer = (self.instruction_cache.len() - 1) as u16;
        }

        let start = std::time::Instant::now();
        loop {
            if self.instruction_pointer as usize >= self.instruction_cache.len() {
                break;
            } else if let CpuStatus::Exiting = self.status {
                break;
            }

            let instruction_start = std::time::Instant::now();
            // Simulate one CPU instruction
            let current_instruction =
                self.instruction_cache[self.instruction_pointer as usize].clone();
            self.handle_instruction(current_instruction.clone());
            // println!(
            //     "{:#?}",
            //     self.instruction_cache[self.instruction_pointer as usize]
            // );
            println!("{}: {}", self.instruction_pointer, self.registers.res);

            // Increment the instruction address
            self.instruction_pointer += 1;

            // Sleep to maintain CPU frequency
            let elapsed = instruction_start.elapsed().as_millis_f64();
            let expected = self.cycle_duration as f64;
            if elapsed < expected {
                let sleep_duration = expected - elapsed;
                std::thread::sleep(std::time::Duration::from_millis(sleep_duration as u64));
            }
        }
        println!(
            "Completed all CPU instructions in {} seconds",
            start.elapsed().as_secs_f64()
        );
    }
}

fn main() {
    let mut cpu = CpuState::new(100);

    let instructions = vec![
        CpuInstruction::Fn("main"),
        CpuInstruction::Add(InstructionArgument::Stack(0), InstructionArgument::Value(1)),
        CpuInstruction::Mov(
            InstructionArgument::Register("res"),
            InstructionArgument::Stack(0),
        ),
        CpuInstruction::Eq(
            InstructionArgument::Stack(0),
            InstructionArgument::Value(10),
        ),
        CpuInstruction::If(
            InstructionArgument::Register("res"),
            Box::new(CpuInstruction::Ret()),
            Box::new(CpuInstruction::Goto(0)),
        ),
    ];
    cpu.append_instructions(&instructions);
    cpu.execute();
}
