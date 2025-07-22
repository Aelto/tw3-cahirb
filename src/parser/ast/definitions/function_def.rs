use crate::parser::{ast::bytecode, prelude::*};

#[derive(Debug)]
pub struct FunctionDefinition {
    pub name: CName,
    pub override_class: CName,
    pub flags: u32,
    pub return_type: Option<PropertyDefinition>,
    pub parameters: Vec<PropertyDefinition>,
    pub locals: Vec<PropertyDefinition>,
    pub bytecode: ByteCodeRef,
}

#[derive(Debug)]
pub struct ParsedFunctionBytecode {
    pub instructions: Vec<Instruction>,
}

impl WithParsing for FunctionDefinition {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self> {
        let (i, name) = CName::parse(i)?;
        let (i, override_class) = CName::parse(i)?;
        let (i, flags) = parse_compressed_u32(i)?;

        let (i, has_return_type) = parse_u8(i)?;
        let (i, return_type) = match has_return_type > 0 {
            false => (i, None),
            true => {
                let (i, return_type) = PropertyDefinition::parse(i)?;

                (i, Some(return_type))
            }
        };

        let (i, parameters) = PropertyDefinition::parse_array(i)?;
        let (i, locals) = PropertyDefinition::parse_array(i)?;
        let (i, bytecode) = ByteCodeRef::parse(i)?;

        Ok((
            i,
            Self {
                name,
                override_class,
                flags,
                return_type,
                parameters,
                locals,
                bytecode,
            },
        ))
    }
}

impl WithParsing for ParsedFunctionBytecode {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self> {
        let (mut i, runtime_size) = parse_u32(i)?;
        let mut offset = 0;

        let mut instructions = Vec::new();

        while let Ok((new_i, mut instruction)) = Instruction::parse(i) {
            i = new_i;

            instruction.offset = offset;
            offset += instruction.size;
            instructions.push(instruction);
        }

        Ok((i, Self { instructions }))
    }
}

impl FunctionDefinition {
    pub fn parse_bytecode(&self) -> ParsedFunctionBytecode {
        ParsedFunctionBytecode::parse(self.bytecode.as_ref())
            .expect("function_bytecode_parsing_failure")
            .1
    }
}

impl WithCodeEmitting for ParsedFunctionBytecode {
    fn emit_code(&self, output: &mut String) {
        for instr in &self.instructions {
            instr.emit_code(output);
            output.push('\n');
        }
    }
}
