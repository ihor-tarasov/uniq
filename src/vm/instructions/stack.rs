use crate::{vm::{State, Value, Res, Error, utils}, dumpln};

impl<'a> State<'a> {
    pub fn push(&mut self, value: Value) -> Res {
        if self.stack_pointer < self.stack.len() as u32 {
            self.stack[self.stack_pointer as usize] = value;
            self.stack_pointer += 1;
            Ok(())
        } else {
            Err(Error::StackOverflow)
        }
    }

    pub fn pop(&mut self) -> Res<Value> {
        if self.stack_pointer >= self.stack.len() as u32 {
            Err(Error::StackOverflow)
        } else if self.stack_pointer == 0 {
            Err(Error::StackUnderflow)
        } else {
            self.stack_pointer -= 1;
            Ok(self.stack[self.stack_pointer as usize].clone())
        }
    }

    pub fn peek(&mut self) -> Res<Value> {
        if self.stack_pointer >= self.stack.len() as u32 {
            Err(Error::StackOverflow)
        } else if self.stack_pointer == 0 {
            Err(Error::StackUnderflow)
        } else {
            Ok(self.stack[(self.stack_pointer - 1) as usize].clone())
        }
    }

    pub(super) fn drop(&mut self) -> Res<bool> {
        dumpln!("DROP");
        self.pop()?;
        self.program_counter = utils::checked_add(self.program_counter, 1)?;
        Ok(true)
    }
}
