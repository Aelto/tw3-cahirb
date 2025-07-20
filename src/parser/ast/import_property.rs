use crate::parser::prelude::*;

pub struct ImportProperty {
    pub name: CName,
    pub type_ref: ImportTypeRef,
    pub scope_type: ImportTypeRef,
}

pub struct ImportPropertyRef {
    pub table_index: u32,
}

impl WithParsing for ImportProperty {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self> {
        let (i, name) = CName::parse(i)?;
        let (i, type_ref) = ImportTypeRef::parse(i)?;
        let (i, scope_type) = ImportTypeRef::parse(i)?;

        Ok((
            i,
            Self {
                name,
                type_ref,
                scope_type,
            },
        ))
    }
}

impl WithTableResolving<ImportProperty> for ImportPropertyRef {
    fn try_resolve<'a>(&self) -> Option<&ImportProperty> {
        crate::parser::TableManager::try_resolve_import_property(self.table_index as usize)
    }
}

impl WithCodeEmitting for ImportProperty {
    fn emit_code(&self, f: &mut String) {
        use std::fmt::Write;

        write!(f, "ImportProperty(").unwrap();
        self.name.emit_code(f);
        self.scope_type.emit_code(f);
        self.type_ref.emit_code(f);
        write!(f, ")").unwrap();
    }
}

impl WithCodeEmitting for ImportPropertyRef {
    fn emit_code(&self, f: &mut String) {
        use std::fmt::Write;

        match self.try_resolve() {
            Some(import_prop) => import_prop.emit_code(f),
            None => {
                write!(f, "__unresolved_import_propery__").unwrap();
            }
        }
    }
}
