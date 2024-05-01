fn build(code: &str) -> uniq::Program {
    let ast = match uniq::parse(code.as_bytes()) {
        Ok(ast) => ast,
        Err(error) => {
            uniq::print_error(
                format!("Parsing error: {}", error.message),
                error.location,
                "user code",
                code,
            );
            std::process::exit(1);
        }
    };

    match uniq::compile(&ast) {
        Ok(program) => program,
        Err(error) => {
            uniq::print_error(
                format!("Compilation error: {}", error.message),
                error.location,
                "user code",
                code,
            );
            std::process::exit(1);
        }
    }
}

fn eval(code: &str) -> uniq::Value {
    let program = build(code);

    match uniq::run(&program) {
        Ok(result) => result,
        Err(error) => {
            uniq::print_error(
                format!("Compilation error: {}", error.message),
                error.location,
                "user code",
                code,
            );
            std::process::exit(1);
        }
    }
}

fn main() {
    println!("{}", eval("2 + 2 ** 2"));
}
