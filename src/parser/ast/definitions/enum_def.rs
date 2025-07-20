use crate::parser::prelude::*;

#[derive(Debug)]
pub struct EnumDefinition {
    pub name: CName,
    pub enumerators: Vec<Enumerator>,
}

#[derive(Debug)]
pub struct Enumerator {
    pub name: CName,
    pub value: i32,
}

impl WithParsing for EnumDefinition {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, name) = CName::parse(i)?;
        let (i, enumerators) = Enumerator::parse_array_u32(i)?;

        Ok((i, Self { name, enumerators }))
    }
}

impl WithParsing for Enumerator {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, name) = CName::parse(i)?;
        let (i, value) = parse_compressed_i32(i)?;

        Ok((i, Self { name, value }))
    }
}
