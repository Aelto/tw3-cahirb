use std::collections::HashMap;

use super::symbols::{Symbols, Type, map_type_name};
use crate::parser::ast::{CName, ImportFunction, ImportFunctionRef, ImportType, ImportTypeRef};

const NO_IMPORT_TYPE: u32 = u32::MAX;

#[derive(Default)]
pub struct Tables {
    pub strings: Vec<String>,
    string_index: HashMap<String, u32>,
    pub import_types: Vec<ImportType>,
    type_index: HashMap<String, u32>,
    pub import_functions: Vec<ImportFunction>,
    function_index: HashMap<(String, Option<String>), u32>,
}

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum TypeKind {
    Simple = 0,
    Enum = 1,
    Class = 2,
    Array = 3,
    Handle = 7,
    Fundamental = 11,
    Map = 12,
}

impl Tables {
    pub fn name(&mut self, value: &str) -> CName {
        if let Some(index) = self.string_index.get(value) {
            return CName::new(*index);
        }

        let index = self.strings.len() as u32;
        self.strings.push(value.to_owned());
        self.string_index.insert(value.to_owned(), index);
        CName::new(index)
    }

    pub fn import_type_named(&mut self, name: &str, kind: TypeKind) -> ImportTypeRef {
        if let Some(index) = self.type_index.get(name) {
            return ImportTypeRef { index: *index };
        }

        let index = self.import_types.len() as u32;
        let cname = self.name(name);
        self.import_types.push(ImportType {
            name: cname,
            kind: kind as u32,
        });
        self.type_index.insert(name.to_owned(), index);
        ImportTypeRef { index }
    }

    pub fn import_type(&mut self, ty: &Type, symbols: &Symbols) -> ImportTypeRef {
        let kind = match ty {
            Type::Bool | Type::Int | Type::Float | Type::Byte | Type::Primitive(_) => {
                TypeKind::Fundamental
            }
            Type::String | Type::Name | Type::Void | Type::Null => TypeKind::Simple,
            Type::Enum(_) => TypeKind::Enum,
            Type::Struct(_) => TypeKind::Class,
            Type::Class(_) => TypeKind::Handle,
            Type::Array { .. } => TypeKind::Array,
            Type::Map { .. } => TypeKind::Map,
        };

        self.import_type_named(&import_element_name(ty, symbols), kind)
    }

    pub fn class_type(&mut self, class: &str, symbols: &Symbols) -> ImportTypeRef {
        self.import_type_named(&symbols.blob_name(class), TypeKind::Class)
    }

    pub fn import_function(
        &mut self,
        name: &str,
        scope_class: Option<&str>,
        symbols: &Symbols,
    ) -> ImportFunctionRef {
        let key = (name.to_owned(), scope_class.map(|class| symbols.blob_name(class)));

        if let Some(index) = self.function_index.get(&key) {
            return ImportFunctionRef { table_index: *index };
        }

        let index = self.import_functions.len() as u32;
        let cname = self.name(name);
        let scope_type = match scope_class {
            Some(class) => self.class_type(class, symbols),
            None => ImportTypeRef { index: NO_IMPORT_TYPE },
        };
        self.import_functions.push(ImportFunction {
            name: cname,
            scope_type,
        });
        self.function_index.insert(key, index);
        ImportFunctionRef { table_index: index }
    }
}

fn import_element_name(ty: &Type, symbols: &Symbols) -> String {
    match ty {
        Type::Class(class) => format!("handle:{}", symbols.blob_name(class)),
        Type::Array { element, layout } => {
            layout.type_name(&import_element_name(element, symbols))
        }
        Type::Map { key, value } => map_type_name(
            &import_element_name(key, symbols),
            &import_element_name(value, symbols),
        ),
        other => other.rtti_name(),
    }
}

pub fn definition_type_name(ty: &Type, symbols: &Symbols) -> String {
    match ty {
        Type::Class(class) | Type::Struct(class) => symbols.blob_name(class),
        Type::Array { element, layout } => {
            layout.type_name(&definition_type_name(element, symbols))
        }
        Type::Map { key, value } => map_type_name(
            &definition_type_name(key, symbols),
            &definition_type_name(value, symbols),
        ),
        other => other.rtti_name(),
    }
}
