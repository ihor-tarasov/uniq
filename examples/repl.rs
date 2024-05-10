use std::io::Write;

fn build(code: &str) -> Option<uniq::Program> {
    let ast = match uniq::parse(code.as_bytes()) {
        Ok(ast) => ast,
        Err(error) => {
            uniq::print_error(
                format!("Parsing error: {}", error.message),
                error.location,
                "stdin",
                code,
            );
            return None;
        }
    };

    match uniq::compile(&ast) {
        Ok(program) => Some(program),
        Err(error) => {
            uniq::print_error(
                format!("Compilation error: {}", error.message),
                error.location,
                "stdin",
                code,
            );
            None
        }
    }
}

fn eval(code: &str) -> Option<uniq::Value> {
    let program = build(code)?;

    match uniq::run(&program) {
        Ok(result) => Some(result),
        Err(error) => {
            uniq::print_error(
                format!("Compilation error: {}", error.message),
                error.location,
                "user code",
                code,
            );
            None
        }
    }
}

fn main() {
    let mut line = String::new();
    loop {
        line.clear();
        print!("-> ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut line).unwrap();
        if let Some(value) = eval(line.as_str()) {
            if value != uniq::Value::Void {
                println!("{value}");
            }
        }
    }
}
