use std::collections::HashMap;
use std::collections::HashSet;

use crate::parser::prelude::*;

mod description;
pub use description::InstructionDescription;

mod operand;
pub use operand::OperandBinding;
pub use operand::OperandValue;

#[derive(Debug)]
pub struct Instruction {
    pub offset: usize,
    pub size: usize,
    pub opcode: u8,
    pub mnemo: &'static str,
    pub operands: HashMap<&'static str, OperandValue>,
}

impl Instruction {
    pub fn new_fake(mnemo: &'static str, offset: usize) -> Self {
        Self {
            offset,
            size: 1,
            opcode: 0,
            mnemo,
            operands: HashMap::new(),
        }
    }

    pub fn is_function_with_param_end(&self) -> bool {
        self.operands
            .get("function")
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
        self.operands
            .get("type")
            .and_then(|op| op.as_import_type())
            .map(|ty| ty.into_emitted_instruction())
    }
}

impl WithParsing for Instruction {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self> {
        let (mut i, byte) = parse_u8(i)?;

        let mut operands = HashMap::new();
        let mut operands_runtime_size = 0;

        let Some(instruction_description) = InstructionDescription::from_index(byte as usize)
        else {
            println!(
                "unknown operand value: {byte}/{}",
                InstructionDescription::size()
            );

            return Err(nom::Err::Incomplete(nom::Needed::Unknown));
        };

        for (opr_type, opr_name) in &instruction_description.operands {
            match *opr_type {
                "bool" => {
                    let (new_i, value) = parse_u8(i)?;
                    i = new_i;

                    operands.insert(*opr_name, OperandValue::Bool(value > 0));
                    operands_runtime_size += 1;
                }
                "u8" => {
                    let (new_i, value) = parse_u8(i)?;
                    i = new_i;

                    operands.insert(*opr_name, OperandValue::Unsigned8(value));
                    operands_runtime_size += 1;
                }
                "u16" => {
                    let (new_i, value) = parse_u16(i)?;
                    i = new_i;

                    operands.insert(*opr_name, OperandValue::Unsigned16(value));
                    operands_runtime_size += 2;
                }
                "u32" => {
                    let (new_i, value) = parse_compressed_u32(i)?;
                    i = new_i;

                    operands.insert(*opr_name, OperandValue::Unsigned32(value));
                    operands_runtime_size += 4;
                }
                "i8" => {
                    let (new_i, value) = parse_i8(i)?;
                    i = new_i;

                    operands.insert(*opr_name, OperandValue::Integer8(value));
                    operands_runtime_size += 1;
                }
                "i16" => {
                    let (new_i, value) = parse_i16(i)?;
                    i = new_i;

                    operands.insert(*opr_name, OperandValue::Integer16(value));
                    operands_runtime_size += 2;
                }
                "i32" => {
                    let (new_i, value) = parse_compressed_i32(i)?;
                    i = new_i;

                    operands.insert(*opr_name, OperandValue::Integer32(value));
                    operands_runtime_size += 4;
                }
                "float" => {
                    let (new_i, value) = parse_f32(i)?;
                    i = new_i;

                    operands.insert(*opr_name, OperandValue::Float(value));
                    operands_runtime_size += 4;
                }
                "name" => {
                    let (new_i, value) = CName::parse(i)?;
                    i = new_i;

                    operands.insert(*opr_name, OperandValue::Name(value));
                    operands_runtime_size += 4;
                }
                "string" => {
                    let (new_i, value) = parse_string(i)?;
                    i = new_i;

                    let len = value.len();
                    operands.insert(*opr_name, OperandValue::String(value));
                    operands_runtime_size += 4 + len;
                }
                "imp_func" => {
                    let (new_i, index) = parse_compressed_i32(i)?;
                    i = new_i;

                    if index > 0 {
                        operands.insert(
                            opr_name,
                            OperandValue::ImportFunction(ImportFunctionRef {
                                table_index: (index - 1) as u32,
                            }),
                        );
                    } else if index == -1 {
                        operands.insert(*opr_name, OperandValue::ImportFunctionSelf);
                    } else if index < -1 {
                        operands.insert(
                            opr_name,
                            OperandValue::ImportFunctionInternal(InternalOperatorRef {
                                table_index: (-1 * (index + 2)) as usize,
                            }),
                        );
                    } else {
                        panic!("function index cannot be 0")
                    }

                    operands_runtime_size += 8; // size of (CFunction*)
                }
                "imp_type" => {
                    let (new_i, type_ref) = ImportTypeRef::parse(i)?;
                    i = new_i;

                    operands.insert(*opr_name, OperandValue::ImportType(type_ref));
                    operands_runtime_size += 8; // sizeof(IRTTIType*)
                }
                "class_prop" => {
                    let (new_i, name) = CName::parse(i)?;
                    i = new_i;

                    let (new_i, type_ref) = ImportTypeRef::parse(i)?;
                    i = new_i;

                    operands.insert(*opr_name, OperandValue::ClassProp { name, type_ref });
                    operands_runtime_size += 8; // sizeof(CProperty*)
                }

                _ => panic!("unhandled operator type {opr_type}"),
            }
        }

        Ok((
            i,
            Self {
                offset: 0,
                size: operands_runtime_size + 1,
                opcode: byte as u8,
                mnemo: instruction_description.mnemo,
                operands,
            },
        ))
    }
}

impl WithInstructionEmitting for Instruction {
    fn emit_instruction(&self, f: &mut String) {
        use std::fmt::Write;

        write!(f, "{}", self.mnemo).unwrap();

        f.push('(');
        for (key, value) in &self.operands {
            write!(f, " {key}=");
            value.emit_instruction(f);
            f.push(' ');
        }
        f.push(')');

        let (size, offset, opcode) = (self.size, self.offset, self.opcode);
        write!(f, " size={size} offset={offset} opcode={opcode}").unwrap();
    }
}
