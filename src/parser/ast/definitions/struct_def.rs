use crate::parser::prelude::*;

#[derive(Debug)]
pub struct StructDefinition {
    pub name: CName,
    pub flags: u32,
    pub properties: Vec<PropertyDefinition>,
    pub default_values: Vec<DefaultValueDefinition>,
}

impl WithParsing for StructDefinition {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self> {
        let (i, name) = CName::parse(i)?;
        let (i, flags) = parse_compressed_u32(i)?;
        let (i, properties) = PropertyDefinition::parse_array(i)?;
        let (i, default_values) = DefaultValueDefinition::parse_array(i)?;

        Ok((
            i,
            Self {
                name,
                flags,
                properties,
                default_values,
            },
        ))
    }
}
