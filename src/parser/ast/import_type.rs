use crate::parser::prelude::*;

#[derive(Debug)]
pub struct ImportType {
    pub name: CName,
    pub kind: u32,
}

#[derive(Clone, Debug)]
pub struct ImportTypeRef {
    pub index: u32,
}

impl WithParsing for ImportType {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self> {
        let (i, name) = CName::parse(i)?;
        let (i, kind) = parse_compressed_u32(i)?;

        Ok((i, Self { name, kind }))
    }
}

impl WithInstructionEmitting for ImportType {
    fn emit_instruction(&self, f: &mut String) {
        self.name.emit_instruction(f);
    }
}

impl WithParsing for ImportTypeRef {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self> {
        let (i, index) = parse_compressed_u32(i)?;

        Ok((i, Self { index }))
    }
}

impl WithTableResolving<ImportType> for ImportTypeRef {
    fn try_resolve<'a>(&self) -> Option<&ImportType> {
        crate::parser::TableManager::try_resolve_import_type(self.index as usize)
    }
}

impl WithInstructionEmitting for ImportTypeRef {
    fn emit_instruction(&self, f: &mut String) {
        use std::fmt::Write;

        match self.try_resolve() {
            Some(import_type) => import_type.emit_instruction(f),
            None => {
                // don't emit anything on purpose
                // write!(f, "__unresolved_import_type__").unwrap();
            }
        }
    }
}
