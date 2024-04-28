mod token;
mod lexer;
mod node;
mod instruction;
mod program;
mod compiler;
mod parser;
mod state;
mod value;
mod vm_error;

pub use node::*;
pub use program::*;
pub use instruction::*;
pub use state::*;
pub use value::*;
pub use vm_error::*;

pub fn parse(code: &[u8]) -> Result<Option<Node>, String> {
    parser::Parser::new(code.iter().copied()).parse()
}

pub fn compile(ast: &Option<Node>) -> Result<Program, String> {
    let mut compiler = compiler::Compiler::new();
    compiler.compile(ast)?;
    Ok(compiler.finish())
}

pub fn parse_and_compile(code: &[u8]) -> Result<Program, String> {
    compile(&parse(code)?)
}

pub fn run(program: &Program) -> Result<Value, VMError> {
    State::new().run(program)
}

pub fn eval(code: &str) -> Value {
    let program =
        parse_and_compile(code.as_bytes()).unwrap_or_else(|error| panic!("Compilation error: {error}"));
    run(&program).unwrap_or_else(|error| panic!("Runtime error: {error}"))
}
