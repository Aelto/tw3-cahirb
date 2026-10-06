use crate::parser::prelude::*;

mod op;
pub use op::Op;

mod operand_types;
pub use operand_types::{FunctionRef, PropertyRef, VariableRef};

mod operand;
pub use operand::OperandValue;

#[derive(Debug, Clone)]
pub struct Instruction {
    pub offset: usize,
    pub op: Op,
}

impl Instruction {
    pub fn new(op: Op, offset: usize) -> Self {
        Self { offset, op }
    }

    pub fn mnemo(&self) -> &'static str {
        self.op.mnemo()
    }

    pub fn size(&self) -> usize {
        self.op.runtime_size()
    }

    pub fn opcode(&self) -> u8 {
        self.op.opcode()
    }

    pub fn operand(&self, name: &str) -> Option<OperandValue> {
        self.op
            .operands()
            .into_iter()
            .find(|(operand_name, _)| *operand_name == name)
            .map(|(_, value)| value)
    }

    pub fn is_function_with_param_end(&self) -> bool {
        self.operand("function")
            // internal operators rarely have a ParamEnd instruction behind,
            // EXCEPT for a few
            .map(|op| {
                !op.is_function_internal_operator()
                    || match op.function_operand() {
                        Some("Add_String_String")
                        | Some("LogicNot_Bool")
                        | Some("Neg_Int32") // not sure
                        | Some("Subtract_Float_Float")
                        // | Some("Add_Int32_Int32")
                        // | Some("LessEqual_Float_Float")
                            => true,
                        | Some(s) => s.starts_with("Add_") || s.starts_with("Substract_") || s.starts_with("Multiply_") || s.starts_with("Divide_") || s.starts_with("Assign"),
                        _ => true,
                    }
            })
            .unwrap_or(true)
    }

    pub fn to_dynamic_cast_type(&self) -> Option<String> {
        self.operand("type_ref")
            .and_then(|op| op.as_import_type().map(|ty| ty.into_emitted_instruction()))
    }

    pub fn to_name(&self) -> String {
        self.operand_string("value")
    }

    pub fn to_string(&self) -> String {
        self.to_name()
    }

    pub fn to_var_name(&self) -> String {
        self.operand_string("name")
    }

    pub fn to_property_name(&self) -> String {
        self.operand_string("property")
    }

    pub fn to_function_name(&self) -> String {
        self.operand_string("function")
    }

    fn operand_string(&self, name: &str) -> String {
        self.operand(name)
            .and_then(|op| op.as_str().map(str::to_owned))
            .unwrap_or_default()
    }
}

impl WithParsing for Instruction {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self> {
        Op::parse(i).map(|(i, op)| (i, Self::new(op, 0)))
    }
}

impl WithWriting for Instruction {
    fn write(&self, w: &mut BlobWriter) {
        self.op.write(w);
    }
}

impl WithInstructionEmitting for Instruction {
    fn emit_instruction(&self, f: &mut String) {
        use std::fmt::Write;

        write!(f, "{}", self.mnemo()).unwrap();

        f.push('(');
        for (key, value) in self.op.operands() {
            write!(f, " {key}=").unwrap();
            value.emit_instruction(f);
            f.push(' ');
        }
        f.push(')');

        write!(
            f,
            " size={} offset={} opcode={}",
            self.size(),
            self.offset,
            self.opcode()
        )
        .unwrap();
    }
}
