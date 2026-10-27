use crate::parser::prelude::*;

#[derive(Debug)]
pub struct DefaultValueDefinition {
    pub name: CName,
    pub value: DefaultValue,
}

#[derive(Debug)]
pub struct DefaultValue {
    pub name: CName,
    pub value: String,
    pub sub_values: Vec<Self>,
}

impl WithParsing for DefaultValueDefinition {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self> {
        let (i, name) = CName::parse(i)?;
        let (i, value) = DefaultValue::parse(i)?;

        Ok((i, Self { name, value }))
    }
}

impl WithParsing for DefaultValue {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self> {
        let (i, name) = CName::parse(i)?;
        let (i, value) = parse_string(i)?;
        let (i, sub_values) = Self::parse_array(i)?;

        Ok((
            i,
            Self {
                name,
                value,
                sub_values,
            },
        ))
    }
}

impl WithWriting for DefaultValueDefinition {
    fn write(&self, w: &mut BlobWriter) {
        w.write(&self.name);
        w.write(&self.value);
    }
}

impl WithWriting for DefaultValue {
    fn write(&self, w: &mut BlobWriter) {
        w.write(&self.name);
        w.write_string(&self.value);
        w.write_array(&self.sub_values);
    }
}
