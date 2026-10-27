use crate::parser::BlobWriter;

pub trait WithWriting {
    fn write(&self, w: &mut BlobWriter);

    fn to_bytes(&self) -> Vec<u8> {
        let mut writer = BlobWriter::default();
        self.write(&mut writer);

        writer.into_bytes()
    }
}
