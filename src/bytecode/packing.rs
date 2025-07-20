use serde::{Deserialize, Serialize};

// fn pack<T: Serialize>(data: &T) -> Vec<u8> {
//     let bytes = serialize::<_, _, BigEndian>(data, Infinite).expect("failed to pack data");

//     bytes
// }

// fn unpack<T: Deserialize>(bytes: &[u8]) -> Result<T> {
//     use bincode::endian_choice::deserialize;
//     let data = deserialize::<_, BigEndian>(bytes).expect("failed to unpack data");

//     data
// }

pub fn read_u8(bytes: &Vec<u8>) -> u8 {
    let config = bincode::config::standard().with_little_endian();

    bincode::decode_from_slice(&bytes[..1], config)
        .expect("decode fail")
        .0
}

pub fn read_u16(bytes: &Vec<u8>) -> u16 {
    let config = bincode::config::standard().with_little_endian();

    bincode::decode_from_slice(&bytes[..2], config)
        .expect("decode fail")
        .0
}

pub fn read_u32(bytes: &Vec<u8>) -> u32 {
    let config = bincode::config::standard().with_little_endian();

    bincode::decode_from_slice(&bytes[..4], config)
        .expect("decode fail")
        .0
}

pub fn read_u64(bytes: &Vec<u8>) -> u64 {
    let config = bincode::config::standard().with_little_endian();

    bincode::decode_from_slice(&bytes[..8], config)
        .expect("decode fail")
        .0
}

pub fn read_i8(bytes: &Vec<u8>) -> i8 {
    let config = bincode::config::standard().with_little_endian();

    bincode::decode_from_slice(&bytes[..1], config)
        .expect("decode fail")
        .0
}

pub fn read_i16(bytes: &Vec<u8>) -> i16 {
    let config = bincode::config::standard().with_little_endian();

    bincode::decode_from_slice(&bytes[..2], config)
        .expect("decode fail")
        .0
}

pub fn read_i32(bytes: &Vec<u8>) -> i32 {
    let config = bincode::config::standard().with_little_endian();

    bincode::decode_from_slice(&bytes[..4], config)
        .expect("decode fail")
        .0
}

pub fn read_i64(bytes: &Vec<u8>) -> u64 {
    let config = bincode::config::standard().with_little_endian();

    bincode::decode_from_slice(&bytes[..8], config)
        .expect("decode fail")
        .0
}

pub fn read_compressed_i32(bytes: &Vec<u8>) -> i32 {
    let mut result = 0;
    let b0 = read_u8(bytes);
    let is_negative = b0 & 0x80;

    result |= b0 & 0x3F;

    if b0 & 0x40 > 0 {
        let b1 = read_u8(bytes);
        result |= (b1 & 0x7F) << 6;

        if b1 & 0x80 > 0 {
            let b2 = read_u8(bytes);
            result |= (b2 & 0x7F) << 13;

            if b2 & 0x80 > 0 {
                let b3 = read_u8(bytes);
                result |= (b3 & 0x7F) << 20;

                if b3 & 0x80 > 0 {
                    let b4 = read_u8(bytes);
                    result |= (b4 & 0x7F) << 27;
                }
            }
        }
    }

    match is_negative > 0 {
        true => result as i32 * -1,
        false => result as i32,
    }
}

pub fn read_compressed_u32(bytes: &Vec<u8>) -> u32 {
    read_compressed_i32(bytes) as u32 & 0xFFFFFFFF
}
