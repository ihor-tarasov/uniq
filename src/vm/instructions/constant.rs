use crate::{
    dumpln,
    vm::{utils, Res, State, Value},
};

impl<'a> State<'a> {
    pub(super) fn int1(&mut self, opcodes: &[u8]) -> Res<bool> {
        let value = utils::fetch_u8(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        dumpln!("INT {value}");
        self.push(Value::Integer(value as i64))?;
        self.program_counter = utils::checked_add(self.program_counter, 2)?;
        Ok(true)
    }

    pub(super) fn int2(&mut self, opcodes: &[u8]) -> Res<bool> {
        let value = utils::fetch_u16(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        dumpln!("INT {value}");
        self.push(Value::Integer(value as i64))?;
        self.program_counter = utils::checked_add(self.program_counter, 3)?;
        Ok(true)
    }

    pub(super) fn int8(&mut self, opcodes: &[u8]) -> Res<bool> {
        let value = utils::fetch_u64(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        dumpln!("INT {value}");
        self.push(Value::Integer(value as i64))?;
        self.program_counter = utils::checked_add(self.program_counter, 9)?;
        Ok(true)
    }

    pub(super) fn real(&mut self, opcodes: &[u8]) -> Res<bool> {
        let value = utils::fetch_f64(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        dumpln!("REAL {value}");
        self.push(Value::Real(value))?;
        self.program_counter = utils::checked_add(self.program_counter, 9)?;
        Ok(true)
    }

    pub(super) fn boolean(&mut self, value: bool) -> Res<bool> {
        if value {
            dumpln!("TRUE");
        } else {
            dumpln!("FALSE");
        }
        self.push(Value::Boolean(value))?;
        self.program_counter = utils::checked_add(self.program_counter, 1)?;
        Ok(true)
    }

    pub(super) fn void(&mut self) -> Res<bool> {
        dumpln!("VOID");
        self.push(Value::Void)?;
        self.program_counter = utils::checked_add(self.program_counter, 1)?;
        Ok(true)
    }
}
