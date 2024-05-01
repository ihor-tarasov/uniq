use std::collections::HashMap;

use crate::{token::TokenLocation, Binary, Instruction, Node, Program, SourceResult};

pub struct Compiler {
    instructions: Vec<Instruction>,
    locations: HashMap<usize, TokenLocation>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            locations: HashMap::new(),
        }
    }

    fn push(&mut self, instruction: Instruction) -> SourceResult<()> {
        self.instructions.push(instruction);
        Ok(())
    }

    fn binary(&mut self, binary: &Binary) -> SourceResult<()> {
        self.node(&binary.left)?;
        self.node(&binary.right)?;
        self.locations
            .insert(self.instructions.len(), binary.location);
        self.instructions.push(binary.instruction);
        Ok(())
    }

    fn node(&mut self, node: &Node) -> SourceResult<()> {
        match node {
            Node::Integer(value) => self.push(Instruction::Integer(*value)),
            Node::Float(value) => self.push(Instruction::Float(*value)),
            Node::Binary(binary) => self.binary(binary),
        }
    }

    pub fn compile(&mut self, node: &Option<Node>) -> SourceResult<()> {
        if let Some(node) = node {
            self.node(node)?;
        }
        self.push(Instruction::End)
    }

    pub fn finish(self) -> Program {
        Program::new(self.instructions.into_boxed_slice(), self.locations)
    }
}
