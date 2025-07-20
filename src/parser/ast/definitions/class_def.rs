use crate::parser::prelude::*;

#[derive(Debug)]
pub struct ClassDefinition {
    pub name: CName,
    pub parent_name: CName,
    pub machine_name: CName,
    /// this could perhaps be turned into a boolean? Or a method?
    pub is_state: u8,
    pub flags: u32,
    pub properties: Vec<PropertyDefinition>,
    pub functions: Vec<FunctionDefinition>,
    pub default_values: Vec<DefaultValueDefinition>,
}

impl WithParsing for ClassDefinition {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self> {
        let (i, name) = CName::parse(i)?;
        let (i, parent_name) = CName::parse(i)?;
        let (i, machine_name) = CName::parse(i)?;
        let (i, is_state) = parse_u8(i)?;
        let (i, flags) = parse_compressed_u32(i)?;
        let (i, properties) = PropertyDefinition::parse_array(i)?;
        let (i, functions) = FunctionDefinition::parse_array(i)?;
        let (i, default_values) = DefaultValueDefinition::parse_array(i)?;

        Ok((
            i,
            Self {
                name,
                parent_name,
                machine_name,
                is_state,
                flags,
                properties,
                functions,
                default_values,
            },
        ))
    }
}
