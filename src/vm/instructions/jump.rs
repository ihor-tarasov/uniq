use crate::{vm::{State, Res, utils, Value, Error}, dumpln};

impl<'a> State<'a> {
    pub(super) fn jp2(&mut self, opcodes: &[u8]) -> Res<bool> {
        self.program_counter = utils::fetch_u16(opcodes, utils::checked_add(self.program_counter, 1)?)? as u32;
        dumpln!("JP {}", self.program_counter);
        Ok(true)
    }

    pub(super) fn jp4(&mut self, opcodes: &[u8]) -> Res<bool> {
        self.program_counter = utils::fetch_u32(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        dumpln!("JP {}", self.program_counter);
        Ok(true)
    }

    pub(super) fn jf2(&mut self, opcodes: &[u8]) -> Res<bool> {
        dumpln!(
            "JF {}",
            utils::fetch_u16(opcodes, utils::checked_add(self.program_counter, 1)?)?
        );
        let value = self.pop()?;
        match value {
            Value::Boolean(value) => {
                if value {
                    self.program_counter = utils::checked_add(self.program_counter, 5)?;
                } else {
                    self.program_counter =
                    utils::fetch_u16(opcodes, utils::checked_add(self.program_counter, 1)?)? as u32;
                }
                Ok(true)
            }
            _ => {
                self.message = Some(format!("Expected bool value, found {value}"));
                Err(Error::UnexpectedType)
            }
        }
    }

    pub(super) fn jf4(&mut self, opcodes: &[u8]) -> Res<bool> {
        dumpln!(
            "JF {}",
            utils::fetch_u32(opcodes, utils::checked_add(self.program_counter, 1)?)?
        );
        let value = self.pop()?;
        match value {
            Value::Boolean(value) => {
                if value {
                    self.program_counter = utils::checked_add(self.program_counter, 5)?;
                } else {
                    self.program_counter =
                    utils::fetch_u32(opcodes, utils::checked_add(self.program_counter, 1)?)?;
                }
                Ok(true)
            }
            _ => {
                self.message = Some(format!("Expected bool value, found {value}"));
                Err(Error::UnexpectedType)
            }
        }
    }
}
