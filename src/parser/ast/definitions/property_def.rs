use std::hint;

use crate::parser::prelude::*;

#[derive(Debug)]
pub struct PropertyDefinition {
    pub name: CName,
    pub hint: String,
    pub flags: u32,
    pub type_name: CName,
    pub class_name: CName,
    pub binding: CName,
}

impl WithParsing for PropertyDefinition {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self> {
        let (i, name) = CName::parse(i)?;
        let (i, hint) = parse_string(i)?;
        let (i, flags) = parse_compressed_u32(i)?;
        let (i, type_name) = CName::parse(i)?;
        let (i, class_name) = CName::parse(i)?;
        let (i, binding) = CName::parse(i)?;

        Ok((
            i,
            Self {
                name,
                hint,
                flags,
                type_name,
                class_name,
                binding,
            },
        ))
    }
}
