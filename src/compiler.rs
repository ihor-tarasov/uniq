use crate::{Instruction, Node, Program};

pub struct Compiler {
    instructions: Vec<Instruction>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
        }
    }

    fn push(&mut self, instruction: Instruction) -> Result<(), String> {
        self.instructions.push(instruction);
        Ok(())
    }

    fn binary(&mut self, data: &(Node, Node, Instruction)) -> Result<(), String> {
        self.node(&data.0)?;
        self.node(&data.1)?;
        self.instructions.push(data.2);
        Ok(())
    }

    fn node(&mut self, node: &Node) -> Result<(), String> {
        match node {
            Node::Integer(value) => self.push(Instruction::Integer(*value)),
            Node::Binary(data) => self.binary(data),
        }
    }

    pub fn compile(&mut self, node: &Option<Node>) -> Result<(), String> {
        if let Some(node) = node {
            self.node(node)?;
        }
        self.push(Instruction::End)
    }

    pub fn finish(self) -> Program {
        Program::new(self.instructions.into_boxed_slice())
    }
}
