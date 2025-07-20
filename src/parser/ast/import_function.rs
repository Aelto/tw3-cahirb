use crate::parser::prelude::*;

#[derive(Clone, Debug)]
pub struct ImportFunction {
    pub name: CName,
    pub scope_type: ImportTypeRef,
}

#[derive(Debug)]
pub struct ImportFunctionRef {
    pub table_index: u32,
}

impl WithParsing for ImportFunction {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self> {
        let (i, name) = CName::parse(i)?;
        let (i, scope_type) = ImportTypeRef::parse(i)?;

        Ok((i, Self { name, scope_type }))
    }
}

impl WithCodeEmitting for ImportFunction {
    fn emit_code(&self, f: &mut String) {
        use std::fmt::Write;

        self.name.emit_code(f);
        self.scope_type.emit_code(f);
    }
}

impl WithTableResolving<ImportFunction> for ImportFunctionRef {
    fn try_resolve<'a>(&self) -> Option<&ImportFunction> {
        TableManager::try_resolve_import_function(self.table_index as usize)
    }
}

impl WithCodeEmitting for ImportFunctionRef {
    fn emit_code(&self, f: &mut String) {
        use std::fmt::Write;

        match self.try_resolve() {
            Some(import_function) => import_function.emit_code(f),
            None => write!(f, "__unresolved_function_ref__").unwrap(),
        }
    }
}
