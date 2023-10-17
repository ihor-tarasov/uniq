
pub struct Slice<'a>(pub &'a [u8]);

impl<'a> std::fmt::Display for Slice<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for c in self.0 {
            write!(f, "{}", *c as char)?;
        }
        Ok(())
    }
}
