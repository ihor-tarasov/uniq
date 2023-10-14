use crate::{
    dumpln,
    vm::{utils, Error, Res, State, Value},
};

impl<'a> State<'a> {
    fn call_by_pointer(
        &mut self,
        opcodes: &[u8],
        in_stack_offset: u32,
        address: u32,
        params_count: u8,
    ) -> Res {
        self.stack[in_stack_offset as usize] =
            Value::CallState(utils::checked_add(self.program_counter, 2)?, self.locals);
        self.program_counter = address;
        self.locals = self.stack_pointer - params_count as u32;
        let params_count_for_check = utils::fetch_u8(opcodes, self.program_counter)?;
        if params_count != params_count_for_check {
            return self.error(format!(
                "Expected {params_count_for_check} function call arguments, found {params_count}."
            ));
        }
        let stack_size = utils::fetch_u32(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        self.stack_pointer = utils::checked_add(self.stack_pointer, stack_size)?;
        dumpln!("Call info: parameters count: {params_count}, stack_size: {stack_size}");
        self.program_counter = utils::checked_add(self.program_counter, 5)?;
        Ok(())
    }

    fn call_native(&mut self, id: u32, params_count: u8) -> Res {
        let old_locals = self.locals;
        self.locals = self.stack_pointer - params_count as u32;
        let (argc, func) = self.natives.get_by_index(id);
        if *argc != params_count {
            return self.error(format!(
                "Expected {argc} function call arguments, found {params_count}."
            ));
        }
        let result = func(self)?;
        self.program_counter = utils::checked_add(self.program_counter, 2)?;
        self.locals = old_locals;
        self.stack_pointer = self.stack_pointer - params_count as u32 - 1;
        self.push(result)
    }

    pub(super) fn call(&mut self, opcodes: &[u8]) -> Res<bool> {
        let params_count = utils::fetch_u8(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        dumpln!("CALL {params_count}");
        if self.stack_pointer < params_count as u32 + 1 {
            return Err(Error::StackUnderflow);
        }
        let in_stack_offset = self.stack_pointer - params_count as u32 - 1;
        let address = self.stack[in_stack_offset as usize].clone();
        match address {
            Value::Pointer(address) => {
                self.call_by_pointer(opcodes, in_stack_offset, address, params_count)?
            }
            Value::Native(id) => self.call_native(id, params_count)?,
            _ => return self.error(format!("Expected address, found {address}")),
        }
        Ok(true)
    }

    pub(super) fn ret(&mut self) -> Res<bool> {
        dumpln!("RET");
        if self.locals == 0 {
            return Ok(false);
        }
        let result = self.pop()?;
        self.stack_pointer = self.locals - 1;
        let call_state = self.stack[self.stack_pointer as usize].clone();
        match call_state {
            Value::CallState(new_pc, new_locals) => {
                self.push(result)?;
                self.program_counter = new_pc;
                self.locals = new_locals;
            }
            _ => return self.error(format!("Expected CallState, found {call_state}")),
        }
        Ok(true)
    }
}
