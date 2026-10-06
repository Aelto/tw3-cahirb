use crate::parser::WithWriting;

#[derive(Default)]
pub struct BlobWriter {
    bytes: Vec<u8>,
}

impl BlobWriter {
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn write_u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    pub fn write_u16(&mut self, value: u16) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_i16(&mut self, value: i16) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_f32(&mut self, value: f32) {
        self.bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    }

    pub fn write_u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    pub fn write_compressed_i32(&mut self, value: i32) {
        let sign = if value <= 0 { 0x80 } else { 0x00 };
        let mut magnitude = value.unsigned_abs();

        let first = sign | (magnitude & 0x3F) as u8;
        magnitude >>= 6;

        if magnitude == 0 {
            self.write_u8(first);
            return;
        }

        self.write_u8(first | 0x40);

        while magnitude > 0x7F {
            self.write_u8((magnitude & 0x7F) as u8 | 0x80);
            magnitude >>= 7;
        }

        self.write_u8(magnitude as u8);
    }

    pub fn write_compressed_u32(&mut self, value: u32) {
        self.write_compressed_i32(value as i32);
    }

    pub fn write_string(&mut self, value: &str) {
        self.write_compressed_i32(-(value.len() as i32));
        self.write_bytes(value.as_bytes());
    }

    pub fn write<T: WithWriting>(&mut self, value: &T) {
        value.write(self);
    }

    pub fn write_array<T: WithWriting>(&mut self, items: &[T]) {
        self.write_compressed_i32(items.len() as i32);

        for item in items {
            item.write(self);
        }
    }
}
