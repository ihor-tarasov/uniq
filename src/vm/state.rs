use super::Value;

pub struct State<'a> {
    pub(super) stack: &'a mut [Value],
    pub(super) stack_pointer: u32,
    pub(super) program_counter: u32,
    pub(super) locals: u32,
    pub(super) message: Option<String>,
}

impl<'a> State<'a> {
    pub fn new(stack: &'a mut [Value]) -> Self {
        assert!(
            stack.len() <= u32::MAX as usize,
            "Maximum stack length must be u32::MAX."
        );
        Self {
            stack,
            stack_pointer: 0,
            program_counter: 0,
            locals: 0,
            message: None,
        }
    }

    pub(super) fn dump_stack(&self) {
        for i in 0..self.stack_pointer {
            print!("[{}]", self.stack[i as usize]);
        }
        println!();
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_ref().map(|s| s.as_str())
    }

    pub fn program_counter(&self) -> u32 {
        self.program_counter
    }
}
