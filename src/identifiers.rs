use std::ops::Range;

#[derive(PartialEq)]
pub struct IdentifierId(Range<u32>);

#[derive(Clone, Copy)]
pub struct IdentifierStart(u32);

pub struct Identifiers {
    buffer: Vec<u8>,
}

impl Identifiers {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn push(&mut self, c: u8) {
        self.buffer.push(c);
    }

    pub fn start(&mut self) -> IdentifierStart {
        IdentifierStart(self.buffer.len() as u32)
    }

    pub fn finish(&mut self, start: IdentifierStart) -> IdentifierId {
        IdentifierId(start.0..(self.buffer.len() as u32))
    }

    pub fn get(&self, id: &IdentifierId) -> &[u8] {
        &self.buffer[(id.0.start as usize)..(id.0.end as usize)]
    }

    pub fn restart(&mut self, start: IdentifierStart) {
        self.buffer.truncate(start.0 as usize);
    }
}
