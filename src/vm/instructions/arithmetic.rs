use crate::{vm::{State, Value, Res, Error}, dumpln};

impl<'a> State<'a> {
    pub(super) fn add(&mut self, l: Value, r: Value) -> Res<Value> {
        dumpln!("ADD");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l.wrapping_add(r))),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Real((l as f64) + r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Real(l + (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Real(l + r)),
            (Value::Object(object), value) => {
                {
                    let mut object = object.borrow_mut();
                    object.push(value);
                }
                Ok(Value::Object(object))
            }
            _ => {
                self.message = Some(format!("Unable to addict {l} and {r} values."));
                Err(Error::BinaryOperation)
            }
        }
    }

    pub(super) fn sub(&mut self, l: Value, r: Value) -> Res<Value> {
        dumpln!("SUB");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l.wrapping_sub(r))),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Real((l as f64) - r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Real(l - (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Real(l - r)),
            _ => {
                self.message = Some(format!("Unable to subtract {l} and {r} values."));
                Err(Error::BinaryOperation)
            }
        }
    }

    pub(super) fn mul(&mut self, l: Value, r: Value) -> Res<Value> {
        dumpln!("MUL");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l.wrapping_mul(r))),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Real((l as f64) * r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Real(l * (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Real(l * r)),
            _ => {
                self.message = Some(format!("Unable to multiply {l} and {r} values."));
                Err(Error::BinaryOperation)
            }
        }
    }

    pub(super) fn div(&mut self, l: Value, r: Value) -> Res<Value> {
        dumpln!("DIV");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => {
                if r == 0 {
                    Err(Error::DividingByZero)
                } else {
                    Ok(Value::Integer(l.wrapping_div(r)))
                }
            }
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Real((l as f64) / r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Real(l / (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Real(l / r)),
            _ => {
                self.message = Some(format!("Unable to divide {l} and {r} values."));
                Err(Error::BinaryOperation)
            }
        }
    }
}
