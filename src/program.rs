use std::{collections::HashMap, error::Error, fs::File};

use serde::{Deserialize, Serialize};

use crate::{instruction::Instruction, token::TokenLocation};

#[derive(Serialize, Deserialize)]
pub struct Program {
    version: String,
    instructions: Box<[Instruction]>,
    locations: HashMap<usize, TokenLocation>,
}

impl Program {
    pub(crate) fn new(
        instructions: Box<[Instruction]>,
        locations: HashMap<usize, TokenLocation>,
    ) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            instructions,
            locations,
        }
    }

    pub fn instruction(&self, index: usize) -> Option<Instruction> {
        self.instructions.get(index).copied()
    }

    pub fn location(&self, index: usize) -> Option<TokenLocation> {
        self.locations.get(&index).copied()
    }

    pub fn save_json(&self, path: &str, pretty: bool) -> Result<(), Box<dyn Error>> {
        let writer = File::create(path)?;
        if pretty {
            serde_json::to_writer_pretty(writer, self)?;
        } else {
            serde_json::to_writer(writer, self)?;
        }
        Ok(())
    }

    pub fn load_json(path: &str) -> Result<Self, Box<dyn Error>> {
        let reader = File::open(path)?;
        let result = serde_json::from_reader(reader)?;
        Ok(result)
    }

    pub fn save_bin(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let writer = File::create(path)?;
        bincode::serialize_into(writer, self)?;
        Ok(())
    }

    pub fn load_bin(path: &str) -> Result<Self, Box<dyn Error>> {
        let reader = File::open(path)?;
        let result = bincode::deserialize_from(reader)?;
        Ok(result)
    }
}
