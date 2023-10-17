use super::{Res, Error};

pub struct Opcodes(Vec<u8>);

impl Opcodes {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, opcode: u8) -> Res<()> {
        if self.0.len() <= u32::MAX as usize {
            self.0.push(opcode);
            Ok(())
        } else {
            Err(Error::Custom(Box::new(format!(
                "Too large opcodes count."
            ))))
        }
    }

    pub fn extend<I>(&mut self, iter: I) -> Res<()>
    where
        I: IntoIterator<Item = u8>,
    {
        for opcode in iter {
            self.push(opcode)?;
        }
        Ok(())
    }

    pub fn len(&self) -> u32 {
        self.0.len() as u32
    }

    pub fn pop(&mut self) {
        self.0.pop();
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl std::ops::Index<u32> for Opcodes {
    type Output = u8;

    fn index(&self, index: u32) -> &Self::Output {
        self.0.index(index as usize)
    }
}

impl std::ops::IndexMut<u32> for Opcodes {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        self.0.index_mut(index as usize)
    }
}

impl Into<Box<[u8]>> for Opcodes {
    fn into(self) -> Box<[u8]> {
        self.0.into_boxed_slice()
    }
}
