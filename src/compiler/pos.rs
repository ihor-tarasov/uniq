use std::ops::Range;

pub struct Pos {
    pub range: Range<usize>,
    pub source_id: usize,
}
