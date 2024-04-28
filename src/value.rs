use std::fmt;

use crate::{vm_error, State, VMResult};

#[derive(Clone, Copy)]
pub enum Value {
    Void,
    Boolean(bool),
    Integer(i64),
    Real(f64),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Void => write!(f, "()"),
            Value::Boolean(value) => write!(f, "{value}"),
            Value::Integer(value) => write!(f, "{value}"),
            Value::Real(value) => write!(f, "{value}"),
        }
    }
}

impl State {
    pub fn addict(&mut self, l: Value, r: Value) -> VMResult<Value> {
        match (l, r) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l.wrapping_add(r))),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Real(l as f64 + r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Real(l + r as f64)),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Real(l + r)),
            (l, r) => vm_error(format!("Unable to addict {l} and {r}")),
        }
    }

    pub fn subtract(&mut self, l: Value, r: Value) -> VMResult<Value> {
        match (l, r) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l.wrapping_sub(r))),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Real(l as f64 - r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Real(l - r as f64)),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Real(l - r)),
            (l, r) => vm_error(format!("Unable to subtract {l} and {r}")),
        }
    }

    pub fn multiply(&mut self, l: Value, r: Value) -> VMResult<Value> {
        match (l, r) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l.wrapping_mul(r))),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Real(l as f64 * r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Real(l * r as f64)),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Real(l * r)),
            (l, r) => vm_error(format!("Unable to multiply {l} and {r}")),
        }
    }

    pub fn divide(&mut self, l: Value, r: Value) -> VMResult<Value> {
        match (l, r) {
            (Value::Integer(l), Value::Integer(r)) => {
                if r == 0 {
                    vm_error(format!("Dividing by zero."))
                } else {
                    Ok(Value::Integer(l.wrapping_div(r)))
                }
            }
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Real(l as f64 / r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Real(l / r as f64)),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Real(l / r)),
            (l, r) => vm_error(format!("Unable to divide {l} and {r}")),
        }
    }
}
