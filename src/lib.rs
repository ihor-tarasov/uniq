mod token;
mod lexer;
mod node;
mod instruction;
mod program;
mod compiler;
mod parser;
mod vm_error;
mod state;

pub use node::*;
pub use program::*;
pub use instruction::*;
pub use compiler::*;
pub use vm_error::*;
pub use state::*;

pub fn parse(code: &[u8]) -> Result<Option<Node>, String> {
    parser::Parser::new(code.iter().copied()).parse()
}

pub fn compile(ast: &Option<Node>) -> Result<Program, String> {
    let mut compiler = Compiler::new();
    compiler.compile(ast)?;
    Ok(compiler.finish())
}

pub fn parse_and_compile(code: &[u8]) -> Result<Program, String> {
    compile(&parse(code)?)
}

pub fn run(program: &Program) -> Result<i32, VMError> {
    State::new().run(program)
}

pub fn eval(code: &str) -> i32 {
    let program =
        parse_and_compile(code.as_bytes()).unwrap_or_else(|error| panic!("Compilation error: {error}"));
    run(&program).unwrap_or_else(|error| panic!("Runtime error: {error}"))
}
