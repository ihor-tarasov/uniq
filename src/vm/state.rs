use crate::natives::Natives;

use super::{Value, Res, Error};

pub struct State<'a> {
    pub(super) stack: &'a mut [Value],
    pub(super) stack_pointer: u32,
    pub(super) program_counter: u32,
    pub(super) locals: u32,
    pub(super) message: Option<String>,
    pub(super) natives: &'a Natives,
}

impl<'a> State<'a> {
    pub fn new(start: u32, stack: &'a mut [Value], natives: &'a Natives) -> Self {
        assert!(
            stack.len() <= u32::MAX as usize,
            "Maximum stack length must be u32::MAX."
        );
        Self {
            stack,
            stack_pointer: 0,
            program_counter: start,
            locals: 0,
            message: None,
            natives,
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

    pub fn arg(&self, index: u8) -> Value {
        self.stack[(self.locals + index as u32) as usize].clone()
    }

    pub fn error<T>(&mut self, m: String) -> Res<T> {
        self.message = Some(m);
        Err(Error::Custom)
    }
}
