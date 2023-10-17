use crate::{natives::Natives, vm::Value};

pub fn load() -> Natives {
    let mut natives = Natives::new();

    natives.function(b"print", 1, |state| {
        println!("{}", state.arg(0));
        Ok(Value::Void)
    });

    natives.function(b"len", 1, |state| match state.arg(0) {
        Value::List(object) => Ok(Value::Integer(object.len())),
        _ => state.error(format!(
            "Unsupported value {} for function \"len\".",
            state.arg(0)
        )),
    });

    natives.function(b"cos", 1, |state| match state.arg(0) {
        Value::Real(value) => Ok(Value::Real(value.cos())),
        _ => state.error(format!("Function \"cos\" supports only real types.")),
    });

    natives
}
