use crate::parser::prelude::*;

#[derive(Clone)]
pub struct CName {
    /// Refers to the index in the global string table of the blob
    string_index: u32,
}

impl CName {
    pub fn to_string_or_default(&self) -> String {
        self.try_resolve().map(String::to_owned).unwrap_or_default()
    }
}

impl WithTableResolving<String> for CName {
    fn try_resolve<'a>(&self) -> Option<&String> {
        crate::parser::TableManager::try_resolve_string(self.string_index as usize)
    }
}

impl WithParsing for CName {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, string_index) = parse_compressed_u32(i)?;

        Ok((i, Self { string_index }))
    }
}

impl WithInstructionEmitting for CName {
    fn emit_instruction(&self, f: &mut String) {
        use std::fmt::Write;

        match self.try_resolve() {
            Some(name_ref) => write!(f, "{name_ref}"),
            None => write!(f, "__unresolved_name__"),
        }
        .unwrap();
    }
}

impl std::fmt::Debug for CName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.try_resolve() {
            Some(name_ref) => f
                .debug_struct("CName")
                .field("resolved_string", &name_ref)
                .finish(),
            None => f
                .debug_struct("CName")
                .field("string_index", &self.string_index)
                .finish(),
        }
    }
}

impl std::fmt::Display for CName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.try_resolve() {
            Some(name_ref) => write!(f, "CName('{}')", name_ref),
            None => write!(f, "CName({})", self.string_index),
        }
    }
}
