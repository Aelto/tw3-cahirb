use crate::parser::prelude::*;

#[derive(Debug)]
pub struct ByteCodeRef {
    size: u32,
    inner: Vec<u8>,
}

impl WithParsing for ByteCodeRef {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self> {
        let (i, size) = parse_compressed_u32(i)?;
        let (i, inner) = take(size)(i)?;

        Ok((
            i,
            Self {
                size,
                // currently copy the slice, perhaps in the future hold a ref
                inner: inner.to_vec(),
            },
        ))
    }
}

impl WithWriting for ByteCodeRef {
    fn write(&self, w: &mut BlobWriter) {
        w.write_compressed_u32(self.inner.len() as u32);
        w.write_bytes(&self.inner);
    }
}

impl AsRef<[u8]> for ByteCodeRef {
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}
