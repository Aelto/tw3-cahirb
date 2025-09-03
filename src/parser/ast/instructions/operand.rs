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
    ImportFunctionInternal(InternalOperatorRef),
    /// calling itself
    ImportFunctionSelf,
    ImportType(ImportTypeRef),
    ClassProp {
        name: CName,
        type_ref: ImportTypeRef,
    },
}

impl WithInstructionEmitting for OperandValue {
    fn emit_instruction(&self, f: &mut String) {
        use std::fmt::Write;

        match self {
            OperandValue::Bool(v) => write!(f, "{v}").unwrap(),
            OperandValue::Unsigned8(v) => write!(f, "{v}").unwrap(),
            OperandValue::Unsigned16(v) => write!(f, "{v}").unwrap(),
            OperandValue::Unsigned32(v) => write!(f, "{v}").unwrap(),
            OperandValue::Integer8(v) => write!(f, "{v}").unwrap(),
            OperandValue::Integer16(v) => write!(f, "{v}").unwrap(),
            OperandValue::Integer32(v) => write!(f, "{v}").unwrap(),
            OperandValue::Float(v) => write!(f, "{v}").unwrap(),
            OperandValue::Name(cname) => cname.emit_instruction(f),
            OperandValue::String(s) => f.push_str(s),
            OperandValue::ImportFunction(fn_ref) => fn_ref.emit_instruction(f),
            OperandValue::ImportFunctionInternal(op_ref) => op_ref.emit_instruction(f),
            OperandValue::ImportFunctionSelf => f.push_str("ImportFunctionSelf"),
            OperandValue::ImportType(type_ref) => type_ref.emit_instruction(f),
            OperandValue::ClassProp { name, type_ref } => {
                write!(f, "(").unwrap();
                type_ref.emit_instruction(f);
                write!(f, ")").unwrap();
                name.emit_instruction(f);
            }
        }
    }
}

impl TryFrom<&OperandValue> for i32 {
    type Error = String;

    fn try_from(value: &OperandValue) -> Result<Self, Self::Error> {
        match value {
            OperandValue::Integer8(n) => Ok(*n as i32),
            OperandValue::Integer16(n) => Ok(*n as i32),
            OperandValue::Integer32(n) => Ok(*n),
            OperandValue::Unsigned8(n) => Ok(*n as i32),
            OperandValue::Unsigned16(n) => Ok(*n as i32),
            _ => Err(format!("Failed to convert {value:?} to i32")),
        }
    }
}

impl TryFrom<&OperandValue> for f32 {
    type Error = String;

    fn try_from(value: &OperandValue) -> Result<Self, Self::Error> {
        match value {
            OperandValue::Float(n) => Ok(*n),
            _ => Err(format!("Failed to convert {value:?} to i32")),
        }
    }
}

impl OperandValue {
    pub fn to_i32(&self) -> Option<u32> {
        match self {
            OperandValue::Unsigned8(v) => Some(*v as u32),
            OperandValue::Unsigned16(v) => Some(*v as u32),
            OperandValue::Unsigned32(v) => Some(*v as u32),
            OperandValue::Integer8(v) => Some(*v as u32),
            OperandValue::Integer16(v) => Some(*v as u32),
            OperandValue::Integer32(v) => Some(*v as u32),
            _ => None,
        }
    }

    pub fn as_import_type(&self) -> Option<&ImportType> {
        match self {
            Self::ImportType(r) => r.try_resolve(),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            OperandValue::String(v) => Some(&v),
            OperandValue::Name(v) => v.try_resolve().map(|op| op.as_str()),
            OperandValue::ClassProp { name, type_ref: _ } => {
                name.try_resolve().map(|op| op.as_str())
            }
            OperandValue::ImportFunction(import_function_ref) => import_function_ref
                .try_resolve()
                .and_then(|f| f.name.try_resolve())
                .map(|s| s.as_str()),
            OperandValue::ImportFunctionInternal(internal_operator_ref) => {
                internal_operator_ref.try_resolve().map(|s| *s)
            }

            _ => None,
        }
    }

    pub fn as_name(&self) -> Option<&CName> {
        match self {
            OperandValue::Name(v) => Some(v),
            _ => None,
        }
    }

    /// Returns the optional `function` operand's value
    pub fn function_operand(&self) -> Option<&str> {
        match self {
            OperandValue::ImportFunction(import_function_ref) => import_function_ref
                .try_resolve()
                .and_then(|f| f.name.try_resolve())
                .map(|s| s.as_str()),
            OperandValue::ImportFunctionInternal(internal_operator_ref) => {
                internal_operator_ref.try_resolve().map(|s| *s)
            }
            _ => None,
        }
    }

    pub fn is_function_internal_operator(&self) -> bool {
        match self {
            OperandValue::ImportFunctionInternal(_) => true,
            _ => false,
        }
    }

    pub fn is_function(&self, fn_name: &str) -> bool {
        self.function_operand()
            .map(|s| fn_name == s)
            .unwrap_or(false)
    }

    pub fn is_function_logic_or_bool_bool(&self) -> bool {
        self.is_function("LogicOr_Bool_Bool") // index: 81
    }

    pub fn is_function_logic_not_bool(&self) -> bool {
        self.is_function("LogicNot_Bool")
    }

    pub fn is_function_logic_and_bool_bool(&self) -> bool {
        self.is_function("LogicAnd_Bool_Bool")
    }

    pub fn get_comparison_operator(&self) -> Option<crate::decompiler::ast::ComparisonOperator> {
        use crate::decompiler::ast::ComparisonOperator;

        let Some(fn_name) = self.function_operand() else {
            return None;
        };

        match fn_name {
            s if s.starts_with("Equal") => Some(ComparisonOperator::Equal),
            s if s.starts_with("NotEqual") => Some(ComparisonOperator::NotEqual),

            s if s.starts_with("Greater") => Some(ComparisonOperator::Greater),
            s if s.starts_with("GreaterEqual") => Some(ComparisonOperator::GreaterEqual),

            s if s.starts_with("Less") => Some(ComparisonOperator::Less),
            s if s.starts_with("LessEqual") => Some(ComparisonOperator::LessEqual),

            _ => None,
        }
    }
}
