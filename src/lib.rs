mod compiler;
mod instruction;
mod lexer;
mod node;
mod parser;
mod program;
mod source_error;
mod state;
mod token;
mod value;
mod vm_error;

pub use instruction::*;
pub use node::*;
pub use program::*;
pub use source_error::*;
pub use state::*;
use token::TokenLocation;
pub use value::*;
pub use vm_error::*;

pub fn print_error(message: String, location: TokenLocation, file: &str, source: &str) {
    println!("In file: {file}, line {}", (location.line + 1));
    println!("{}", message);
    if let Some(line) = source.lines().skip(location.line as usize).next() {
        println!("{line}");
        for _ in 0..location.column {
            print!(" ");
        }
        for _ in 0..location.length {
            print!("^");
        }
        println!();
    }
}

pub fn parse(code: &[u8]) -> SourceResult<Option<Node>> {
    parser::Parser::new(code.iter().copied()).parse()
}

pub fn compile(ast: &Option<Node>) -> SourceResult<Program> {
    let mut compiler = compiler::Compiler::new();
    compiler.compile(ast)?;
    Ok(compiler.finish())
}

pub fn parse_and_compile(code: &[u8]) -> SourceResult<Program> {
    compile(&parse(code)?)
}

pub fn run(program: &Program) -> SourceResult<Value> {
    let mut state = State::new();
    state.run(program).map_err(|error| {
        let location = program.location(state.program_counter()).unwrap();
        Box::new(SourceError {
            message: error.to_string(),
            location,
        })
    })
}

pub fn eval(code: &str) -> Value {
    let program = parse_and_compile(code.as_bytes()).unwrap_or_else(|error| {
        print_error(
            format!("Compilation error: {}", error.message),
            error.location,
            "user code",
            code,
        );
        std::process::exit(1);
    });
    run(&program).unwrap_or_else(|error| {
        print_error(
            format!("Runtime error: {}", error.message),
            error.location,
            "user code",
            code,
        );
        std::process::exit(1);
    })
}
