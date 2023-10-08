pub struct SliceIter<'a>(&'a [u8]);

impl<'a> From<&'a [u8]> for SliceIter<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self(value)
    }
}

impl<'a> From<&'a str> for SliceIter<'a> {
    fn from(value: &'a str) -> Self {
        Self(value.as_bytes())
    }
}

impl<'a> Iterator for SliceIter<'a> {
    type Item = std::io::Result<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = *self.0.first()?;
        self.0 = &self.0[1..];
        Some(Ok(result))
    }
}
