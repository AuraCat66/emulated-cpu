#[derive(Default)]
pub struct SubStack {
    pub return_address: u16,
    pub data: Vec<u16>,
}

#[derive(Default)]
pub struct MemoryState {
    stack: Vec<SubStack>,
}
impl MemoryState {
    pub fn create_new_sub_stack(&mut self, return_address: u16) {
        let sub_stack = SubStack {
            return_address,
            ..Default::default()
        };

        self.stack.insert(0, sub_stack);
    }

    pub fn get_current_sub_stack(&self) -> &SubStack {
        &self.stack[0]
    }

    pub fn get_current_sub_stack_mut(&mut self) -> &mut SubStack {
        &mut self.stack[0]
    }

    pub fn rewind_stack(&mut self) {
        self.stack.remove(0);
    }
}
