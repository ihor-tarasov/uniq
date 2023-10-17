use crate::{
    dumpln,
    vm::{utils, Error, Res, State, Value},
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

    fn global_load(&mut self, index: u32) -> Res {
        dumpln!("GL {index}");
        if let Some(value) = self.globals.get(index as usize).cloned() {
            self.push(value)
        } else {
            Err(Error::FetchGlobal)
        }
    }

    pub(super) fn gl1(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = utils::fetch_u8(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        self.global_load(index as u32)?;
        self.program_counter = utils::checked_add(self.program_counter, 2)?;
        Ok(true)
    }

    pub(super) fn gl2(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = utils::fetch_u16(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        self.global_load(index as u32)?;
        self.program_counter = utils::checked_add(self.program_counter, 3)?;
        Ok(true)
    }

    pub(super) fn gl4(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = utils::fetch_u32(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        self.global_load(index)?;
        self.program_counter = utils::checked_add(self.program_counter, 5)?;
        Ok(true)
    }

    fn global_store(&mut self, index: u32) -> Res {
        dumpln!("GS {index}");
        let peek = self.peek()?;
        if let Some(value) = self.globals.get_mut(index as usize) {
            *value = peek;
        } else {
            self.globals.resize(utils::checked_add(index, 1)? as usize, Value::Void);
            self.globals[index as usize] = peek;
        }
        Ok(())
    }

    pub(super) fn gs1(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = utils::fetch_u8(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        self.global_store(index as u32)?;
        self.program_counter = utils::checked_add(self.program_counter, 2)?;
        Ok(true)
    }

    pub(super) fn gs2(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = utils::fetch_u16(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        self.global_store(index as u32)?;
        self.program_counter = utils::checked_add(self.program_counter, 3)?;
        Ok(true)
    }

    pub(super) fn gs4(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = utils::fetch_u32(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        self.global_store(index as u32)?;
        self.program_counter = utils::checked_add(self.program_counter, 5)?;
        Ok(true)
    }
}
