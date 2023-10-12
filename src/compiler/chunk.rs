use super::Pos;
use std::collections::HashMap;

pub struct Chunk {
    pub opcodes: Box<[u8]>,
    pub ranges: HashMap<u32, Pos>,
}
