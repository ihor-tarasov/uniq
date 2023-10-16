use crate::{
    dumpln,
    vm::{utils, Error, Res, State},
};

impl<'a> State<'a> {
    pub(super) fn ld1(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = utils::fetch_u8(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        dumpln!("LD {index}");
        if self.locals + index as u32 >= self.stack.len() as u32 {
            return Err(Error::StackOverflow);
        }
        self.push(self.stack[(self.locals + index as u32) as usize].clone())?;
        self.program_counter = utils::checked_add(self.program_counter, 2)?;
        Ok(true)
    }

    pub(super) fn ld2(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = utils::fetch_u16(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        dumpln!("LD {index}");
        if self.locals + index as u32 >= self.stack.len() as u32 {
            return Err(Error::StackOverflow);
        }
        self.push(self.stack[(self.locals + index as u32) as usize].clone())?;
        self.program_counter = utils::checked_add(self.program_counter, 3)?;
        Ok(true)
    }

    pub(super) fn ld4(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = utils::fetch_u32(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        dumpln!("LD {index}");
        if self.locals + index >= self.stack.len() as u32 {
            return Err(Error::StackOverflow);
        }
        self.push(self.stack[(self.locals + index) as usize].clone())?;
        self.program_counter = utils::checked_add(self.program_counter, 5)?;
        Ok(true)
    }

    pub(super) fn st1(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = utils::fetch_u8(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        dumpln!("ST {index}");
        if self.locals + index as u32 >= self.stack.len() as u32 {
            return Err(Error::StackOverflow);
        }
        self.stack[(self.locals + index as u32) as usize] = self.peek()?;
        self.program_counter = utils::checked_add(self.program_counter, 2)?;
        Ok(true)
    }

    pub(super) fn st2(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = utils::fetch_u16(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        dumpln!("ST {index}");
        if self.locals + index as u32 >= self.stack.len() as u32 {
            return Err(Error::StackOverflow);
        }
        self.stack[(self.locals + index as u32) as usize] = self.peek()?;
        self.program_counter = utils::checked_add(self.program_counter, 3)?;
        Ok(true)
    }

    pub(super) fn st4(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = utils::fetch_u32(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        dumpln!("ST {index}");
        if self.locals + index >= self.stack.len() as u32 {
            return Err(Error::StackOverflow);
        }
        self.stack[(self.locals + index) as usize] = self.peek()?;
        self.program_counter = utils::checked_add(self.program_counter, 5)?;
        Ok(true)
    }

    pub(super) fn gl1(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = utils::fetch_u8(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        dumpln!("GL {index}");
        if index as u32 >= self.stack.len() as u32 {
            return Err(Error::StackOverflow);
        }
        self.push(self.stack[index as usize].clone())?;
        self.program_counter = utils::checked_add(self.program_counter, 2)?;
        Ok(true)
    }

    pub(super) fn gl2(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = utils::fetch_u16(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        dumpln!("GL {index}");
        if index as u32 >= self.stack.len() as u32 {
            return Err(Error::StackOverflow);
        }
        self.push(self.stack[index as usize].clone())?;
        self.program_counter = utils::checked_add(self.program_counter, 3)?;
        Ok(true)
    }

    pub(super) fn gl4(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = utils::fetch_u32(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        dumpln!("GL {index}");
        if index >= self.stack.len() as u32 {
            return Err(Error::StackOverflow);
        }
        self.push(self.stack[index as usize].clone())?;
        self.program_counter = utils::checked_add(self.program_counter, 5)?;
        Ok(true)
    }

    pub(super) fn gs1(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = utils::fetch_u8(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        dumpln!("GS {index}");
        if index as u32 >= self.stack.len() as u32 {
            return Err(Error::StackOverflow);
        }
        self.stack[index as usize] = self.peek()?;
        self.program_counter = utils::checked_add(self.program_counter, 2)?;
        Ok(true)
    }

    pub(super) fn gs2(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = utils::fetch_u16(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        dumpln!("GS {index}");
        if index as u32 >= self.stack.len() as u32 {
            return Err(Error::StackOverflow);
        }
        self.stack[index as usize] = self.peek()?;
        self.program_counter = utils::checked_add(self.program_counter, 3)?;
        Ok(true)
    }

    pub(super) fn gs4(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = utils::fetch_u32(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        dumpln!("GS {index}");
        if index >= self.stack.len() as u32 {
            return Err(Error::StackOverflow);
        }
        self.stack[index as usize] = self.peek()?;
        self.program_counter = utils::checked_add(self.program_counter, 5)?;
        Ok(true)
    }
}
