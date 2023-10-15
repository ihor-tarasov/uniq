use crate::{vm::{State, Value, Res, utils}, dumpln};

impl<'a> State<'a> {
    fn index_get_list(&mut self, data: &Vec<Value>, key: Value) -> Res<Value> {
        match key {
            Value::Integer(index) => {
                if index >= 0 && (index as usize) < data.len() {
                    Ok(data[index as usize].clone())
                } else {
                    self.error(format!("Index out of range."))
                }
            }
            _ => self.error(format!("Can't to index list by {key}.")),
        }
    }

    fn index_get(&mut self, data: Value, key: Value) -> Res<Value> {
        match data {
            Value::List(list) => {
                let list = list.borrow();
                self.index_get_list(&list, key)
            }
            _ => self.error(format!("Can't to index {data}.")),
        }
    }

    fn index_set_list(&mut self, data: &mut Vec<Value>, key: Value, value: Value) -> Res {
        match key {
            Value::Integer(index) => {
                if index >= 0 && (index as usize) < data.len() {
                    data[index as usize] = value;
                    Ok(())
                } else {
                    self.error(format!("Index out of range."))
                }
            }
            _ => self.error(format!("Can't to index list by {key}.")),
        }
    }

    fn index_set(&mut self, data: Value, key: Value, value: Value) -> Res {
        match data {
            Value::List(list) => {
                let mut list = list.borrow_mut();
                self.index_set_list(&mut list, key, value)
            }
            _ => self.error(format!("Can't to index {data}.")),
        }
    }

    pub(super) fn get(&mut self) -> Res<bool> {
        dumpln!("GET");
        let key = self.pop()?;
        let data = self.pop()?;
        let result = self.index_get(data, key)?;
        self.push(result)?;
        self.program_counter = utils::checked_add(self.program_counter, 1)?;
        Ok(true)
    }

    pub(super) fn set(&mut self) -> Res<bool> {
        dumpln!("SET");
        let value = self.pop()?;
        let key = self.pop()?;
        let data = self.pop()?;
        self.index_set(data, key, value.clone())?;
        self.push(value)?;
        self.program_counter = utils::checked_add(self.program_counter, 1)?;
        Ok(true)
    }
}
