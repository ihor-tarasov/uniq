pub struct ReadIter<R>(R);

impl<R> From<R> for ReadIter<R> {
    fn from(read: R) -> Self {
        Self(read)
    }
}

impl<R> Iterator for ReadIter<R>
where
    R: std::io::Read,
{
    type Item = std::io::Result<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = [0u8; 1];
        match self.0.read_exact(&mut buf) {
            Ok(_) => Some(Ok(buf[0])),
            Err(error) => match error.kind() {
                std::io::ErrorKind::UnexpectedEof => None,
                _ => Some(Err(error)),
            },
        }
    }
}
