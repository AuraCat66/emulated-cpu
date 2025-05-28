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

    pub fn write_data(&mut self, address: u16, data: u16) {
        let current_sub_stack = self.get_current_sub_stack_mut();

        if current_sub_stack.data.get(address as usize).is_none() {
            while current_sub_stack.data.get(address as usize).is_none() {
                current_sub_stack.data.push(0);
            }
        }
        current_sub_stack.data[address as usize] = data;
    }

    pub fn rewind_stack(&mut self) {
        self.stack.remove(0);
    }
}
