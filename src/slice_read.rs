pub struct SliceRead<'a> {
    slice: &'a [u8],
    offset: usize,
}

impl<'a> From<&'a [u8]> for SliceRead<'a> {
    fn from(slice: &'a [u8]) -> Self {
        Self {
            slice,
            offset: 0,
        }
    }
}

impl<'a> From<&'a str> for SliceRead<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            slice: value.as_bytes(),
            offset: 0,
        }
    }
}

impl<'a> std::io::Read for SliceRead<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = buf.len().min(self.slice.len() - self.offset);
        for i in 0..len {
            buf[i] = self.slice[self.offset + i];
        }
        self.offset += len;
        Ok(len)
    }
}

impl<'a> std::io::Seek for SliceRead<'a> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        match pos {
            std::io::SeekFrom::Start(pos) => {
                self.offset = pos as usize;
                Ok(pos)
            },
            std::io::SeekFrom::End(pos) => {
                self.offset = (self.slice.len() as i64 + pos) as usize;
                Ok(self.offset as u64)
            },
            std::io::SeekFrom::Current(pos) => {
                self.offset = (self.offset as i64 + pos) as usize;
                Ok(self.offset as u64)
            },
        }
    }
}
