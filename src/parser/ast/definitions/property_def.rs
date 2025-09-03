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

impl PropertyDefinition {
    pub fn to_resolved_name(&self) -> String {
        self.name.to_string_or_default()
    }

    pub fn to_resolved_typename(&self) -> String {
        self.type_name.to_string_or_default()
    }
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

impl WithInstructionEmitting for PropertyDefinition {
    fn emit_instruction(&self, f: &mut String) {
        use std::fmt::Write;

        write!(f, "PropertyDefinition(").unwrap();
        self.name.emit_instruction(f);
        write!(f, ", hint={}", self.hint).unwrap();
        write!(f, ", flags={}", self.flags).unwrap();
        write!(f, ", type_name=").unwrap();
        self.type_name.emit_instruction(f);
        write!(f, ", class_name=").unwrap();
        self.class_name.emit_instruction(f);
        write!(f, ", binding=").unwrap();
        self.binding.emit_instruction(f);
        write!(f, ")").unwrap();
    }
}
