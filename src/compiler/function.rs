use super::block::Block;

pub struct Function {
    blocks: Vec<Block>,
    local_counter: u32,
    stack_size: u32,
}

impl Function {
    pub fn new() -> Self {
        Self {
            blocks: vec![Block::new()],
            local_counter: 0,
            stack_size: 0,
        }
    }

    pub fn push(&mut self) {
        self.blocks.push(Block::new());
    }

    pub fn pop(&mut self) {
        let block = self.blocks.pop().unwrap();
        self.local_counter -= block.len() as u32;
    }

    pub fn var(&mut self, name: &[u8]) -> u32 {
        debug_assert!(!self.blocks.is_empty());
        let len = self.blocks.len();
        let id = self.local_counter;
        if !self.blocks[len - 1].var(name, id) {
            self.local_counter += 1;
            if self.local_counter > self.stack_size {
                self.stack_size = self.local_counter;
            }
        }
        id
    }

    pub fn get(&self, name: &[u8]) -> Option<u32> {
        for block in self.blocks.iter().rev() {
            if let Some(id) = block.get(name) {
                return Some(id);
            }
        }
        None
    }

    pub fn stack_size(&self) -> u32 {
        self.stack_size
    }

    pub fn clear(&mut self) {
        self.blocks.clear();
        self.blocks.push(Block::new());
        self.local_counter = 0;
        self.stack_size = 0;
    }
}
