use crate::{vm::{State, Value, Res, Error}, dumpln};

impl<'a> State<'a> {
    pub(super) fn eq(&mut self, l: Value, r: Value) -> Res<Value> {
        dumpln!("EQ");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l == r)),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((l as f64) == r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(l == (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l == r)),
            _ => {
                self.message = Some(format!("Unable to compare {l} and {r} values."));
                Err(Error::BinaryOperation)
            }
        }
    }

    pub(super) fn ne(&mut self, l: Value, r: Value) -> Res<Value> {
        dumpln!("NE");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l != r)),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((l as f64) != r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(l != (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l != r)),
            _ => {
                self.message = Some(format!("Unable to compare {l} and {r} values."));
                Err(Error::BinaryOperation)
            }
        }
    }

    pub(super) fn ls(&mut self, l: Value, r: Value) -> Res<Value> {
        dumpln!("LS");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l < r)),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((l as f64) < r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(l < (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l < r)),
            _ => {
                self.message = Some(format!("Unable to compare {l} and {r} values."));
                Err(Error::BinaryOperation)
            }
        }
    }

    pub(super) fn le(&mut self, l: Value, r: Value) -> Res<Value> {
        dumpln!("LE");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l <= r)),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((l as f64) <= r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(l <= (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l <= r)),
            _ => {
                self.message = Some(format!("Unable to compare {l} and {r} values."));
                Err(Error::BinaryOperation)
            }
        }
    }

    pub(super) fn gr(&mut self, l: Value, r: Value) -> Res<Value> {
        dumpln!("GR");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l > r)),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((l as f64) > r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(l > (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l > r)),
            _ => {
                self.message = Some(format!("Unable to compare {l} and {r} values."));
                Err(Error::BinaryOperation)
            }
        }
    }

    pub(super) fn ge(&mut self, l: Value, r: Value) -> Res<Value> {
        dumpln!("GE");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l >= r)),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((l as f64) >= r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(l >= (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l >= r)),
            _ => {
                self.message = Some(format!("Unable to compare {l} and {r} values."));
                Err(Error::BinaryOperation)
            }
        }
    }
}
