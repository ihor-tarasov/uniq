use crate::{vm::{State, Value, Res, Object, utils}, dumpln};

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

    fn index_get_object(&mut self, data: &Object, key: Value) -> Res<Value> {
        match data {
            Object::List(list) => self.index_get_list(list, key),
        }
    }

    fn index_get(&mut self, data: Value, key: Value) -> Res<Value> {
        match data {
            Value::Object(object) => {
                let object = object.borrow();
                self.index_get_object(&object, key)
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

    fn index_set_object(&mut self, data: &mut Object, key: Value, value: Value) -> Res {
        match data {
            Object::List(list) => self.index_set_list(list, key, value),
        }
    }

    fn index_set(&mut self, data: Value, key: Value, value: Value) -> Res {
        match data {
            Value::Object(object) => {
                let mut object = object.borrow_mut();
                self.index_set_object(&mut object, key, value)
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
