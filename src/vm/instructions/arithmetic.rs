use crate::{
    dumpln,
    vm::{Error, Res, State, Value},
};

impl<'a> State<'a> {
    pub(super) fn add(&mut self, l: Value, r: Value) -> Res<Value> {
        dumpln!("ADD");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l.wrapping_add(r))),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Real((l as f64) + r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Real(l + (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Real(l + r)),
            (Value::List(list), value) => {
                {
                    list.push(value);
                }
                Ok(Value::List(list))
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

    pub(super) fn inc(&mut self, value: Value) -> Res<Value> {
        dumpln!("INC");
        match value.clone() {
            Value::Integer(value) => Ok(Value::Integer(value.wrapping_add(1))),
            Value::Real(value) => Ok(Value::Real(value + 1.0)),
            _ => {
                self.message = Some(format!("Unable to increment {value} value."));
                Err(Error::UnaryOperation)
            }
        }
    }

    pub(super) fn dec(&mut self, value: Value) -> Res<Value> {
        dumpln!("DEC");
        match value.clone() {
            Value::Integer(value) => Ok(Value::Integer(value.wrapping_sub(1))),
            Value::Real(value) => Ok(Value::Real(value - 1.0)),
            _ => {
                self.message = Some(format!("Unable to decrement {value} value."));
                Err(Error::UnaryOperation)
            }
        }
    }

    pub(super) fn neg(&mut self, value: Value) -> Res<Value> {
        dumpln!("NEG");
        match value.clone() {
            Value::Integer(value) => Ok(Value::Integer(value.wrapping_neg())),
            Value::Real(value) => Ok(Value::Real(-value)),
            _ => {
                self.message = Some(format!("Unable to negate {value} value."));
                Err(Error::UnaryOperation)
            }
        }
    }

    pub(super) fn not(&mut self, value: Value) -> Res<Value> {
        dumpln!("NOT");
        match value.clone() {
            Value::Boolean(value) => Ok(Value::Boolean(!value)),
            Value::Integer(value) => Ok(Value::Integer(!value)),
            _ => {
                self.message = Some(format!("Unable to perform 'not' operator for {value} value."));
                Err(Error::UnaryOperation)
            }
        }
    }
}
