mod slice_read;

pub use slice_read::*;

pub mod line;
pub mod utils;
pub mod compiler;
pub mod opcode;
pub mod vm;

#[cfg(test)]
mod tests;
