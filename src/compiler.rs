use crate::{Instruction, Node, Operator, Program};

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

    fn binary(&mut self, data: &(Node, Node, Operator)) -> Result<(), String> {
        self.node(&data.0)?;
        self.node(&data.1)?;
        match data.2 {
            Operator::Addict => self.instructions.push(Instruction::Addict),
            Operator::Subtract => self.instructions.push(Instruction::Subtract),
            Operator::Multiply => self.instructions.push(Instruction::Multiply),
            Operator::Divide => self.instructions.push(Instruction::Divide),
            Operator::Modulo => self.instructions.push(Instruction::Modulo),
        }
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
