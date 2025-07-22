mod cdatetime;
pub use cdatetime::CDateTime;

mod cname;
pub use cname::CName;

mod bytecode;
pub use bytecode::ByteCodeRef;

mod import_type;
pub use import_type::ImportType;
pub use import_type::ImportTypeRef;

mod import_property;
pub use import_property::ImportProperty;
pub use import_property::ImportPropertyRef;

mod import_function;
pub use import_function::ImportFunction;
pub use import_function::ImportFunctionRef;

mod internal_operator;
pub use internal_operator::InternalOperatorRef;

mod definitions;
pub use definitions::*;

mod instructions;
pub use instructions::*;

use crate::parser::prelude::*;

pub fn parse_u8(i: &[u8]) -> IResult<&[u8], u8> {
    nom::number::complete::le_u8(i)
}

pub fn parse_u16(i: &[u8]) -> IResult<&[u8], u16> {
    nom::number::complete::le_u16(i)
}

pub fn parse_u32(i: &[u8]) -> IResult<&[u8], u32> {
    nom::number::complete::le_u32(i)
}

pub fn parse_u64(i: &[u8]) -> IResult<&[u8], u64> {
    nom::number::complete::le_u64(i)
}

pub fn parse_i8(i: &[u8]) -> IResult<&[u8], i8> {
    nom::number::complete::le_i8(i)
}

pub fn parse_i16(i: &[u8]) -> IResult<&[u8], i16> {
    nom::number::complete::le_i16(i)
}

pub fn parse_i32(i: &[u8]) -> IResult<&[u8], i32> {
    nom::number::complete::le_i32(i)
}

pub fn parse_i64(i: &[u8]) -> IResult<&[u8], i64> {
    nom::number::complete::le_i64(i)
}

pub fn parse_f32(i: &[u8]) -> IResult<&[u8], f32> {
    nom::number::complete::le_f32(i)
}

pub fn parse_compressed_i32(mut i: &[u8]) -> IResult<&[u8], i32> {
    let mut result: i32 = 0;
    let (new_i, b0) = parse_u8(i)?;
    i = new_i;
    let b0 = b0 as i32;
    let is_negative = b0 & 0x80;

    result |= b0 & 0x3F;

    if b0 & 0x40 > 0 {
        let (new_i, b1) = parse_u8(i)?;
        i = new_i;
        let b1 = b1 as i32;
        result |= (b1 & 0x7F) << 6;

        if b1 & 0x80 > 0 {
            let (new_i, b2) = parse_u8(i)?;
            i = new_i;
            let b2 = b2 as i32;
            result |= (b2 & 0x7F) << 13;

            if b2 & 0x80 > 0 {
                let (new_i, b3) = parse_u8(i)?;
                i = new_i;
                let b3 = b3 as i32;
                result |= (b3 & 0x7F) << 20;

                if b3 & 0x80 > 0 {
                    let (new_i, b4) = parse_u8(i)?;
                    i = new_i;
                    let b4 = b4 as i32;
                    result |= (b4 & 0x7F) << 27;
                }
            }
        }
    }

    match is_negative > 0 {
        true => Ok((i, result as i32 * -1)),
        false => Ok((i, result as i32)),
    }
}

pub fn parse_compressed_u32(i: &[u8]) -> IResult<&[u8], u32> {
    parse_compressed_i32(i).map(|(i, n)| (i, (n as u32)))
}

pub fn parse_string(i: &[u8]) -> IResult<&[u8], String> {
    let (i, len) = parse_compressed_i32(i)?;

    if len <= 0 {
        // utf8
        let (i, slice) = take((len * -1) as usize)(i)?;

        return Ok((i, String::from_utf8_lossy(slice).to_string()));
    } else {
        // utf16
        let (i, slice) = take((len * 2) as usize)(i)?;

        use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
        let mut cursor = std::io::Cursor::new(slice);
        let mut u16_slice = Vec::new();

        while let Ok(utf16_char) = cursor.read_u16::<LittleEndian>() {
            u16_slice.push(utf16_char);
        }

        return Ok((i, String::from_utf16_lossy(&u16_slice)));
    }
}
