use crate::parser::prelude::*;

pub type OperandBinding = (&'static str, &'static str);

#[derive(Debug)]
pub enum OperandValue {
    Bool(bool),
    Unsigned8(u8),
    Unsigned16(u16),
    Unsigned32(u32),
    Integer8(i8),
    Integer16(i16),
    Integer32(i32),
    Float(f32),
    Name(CName),
    String(String),
    ImportFunction(ImportFunctionRef),
    /// A call to an internal function,
    ImportFunctionInternal {
        internal_table_index: usize,
    },
    /// calling itself
    ImportFunctionSelf,
    ImportType(ImportTypeRef),
    ClassProp {
        name: CName,
        type_ref: ImportTypeRef,
    },
}

impl WithCodeEmitting for OperandValue {
    fn emit_code(&self, f: &mut String) {
        use std::fmt::Write;

        match self {
            OperandValue::Bool(v) => write!(f, " {v} ").unwrap(),
            OperandValue::Unsigned8(v) => write!(f, " {v} ").unwrap(),
            OperandValue::Unsigned16(v) => write!(f, " {v} ").unwrap(),
            OperandValue::Unsigned32(v) => write!(f, " {v} ").unwrap(),
            OperandValue::Integer8(v) => write!(f, " {v} ").unwrap(),
            OperandValue::Integer16(v) => write!(f, " {v} ").unwrap(),
            OperandValue::Integer32(v) => write!(f, " {v} ").unwrap(),
            OperandValue::Float(v) => write!(f, " {v} ").unwrap(),
            OperandValue::Name(cname) => cname.emit_code(f),
            OperandValue::String(s) => f.push_str(s),
            OperandValue::ImportFunction(fn_ref) => fn_ref.emit_code(f),
            OperandValue::ImportFunctionInternal {
                internal_table_index,
            } => f.push_str("ImportFunctionInternal"),
            OperandValue::ImportFunctionSelf => f.push_str("ImportFunctionSelf"),
            OperandValue::ImportType(type_ref) => type_ref.emit_code(f),
            OperandValue::ClassProp { name, type_ref } => {
                write!(f, " (").unwrap();
                type_ref.emit_code(f);
                write!(f, ")").unwrap();
                name.emit_code(f);
                f.push(' ');
            }
        }
    }
}
