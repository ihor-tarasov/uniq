use std::collections::HashMap;

pub struct Block {
    locals: HashMap<Box<[u8]>, u32>,
}

impl Block {
    pub fn new() -> Self {
        Self {
            locals: HashMap::new(),
        }
    }

    pub fn var(&mut self, name: &[u8], id: u32) -> bool {
        if let Some(local) = self.locals.get_mut(name) {
            *local = id;
            true
        } else {
            self.locals.insert(Vec::from(name).into_boxed_slice(), id);
            false
        }
    }

    pub fn get(&self, name: &[u8]) -> Option<u32> {
        self.locals.get(name).cloned()
    }

    pub fn len(&self) -> usize {
        self.locals.len()
    }
}
