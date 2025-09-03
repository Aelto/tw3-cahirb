use crate::{
    decompiler::WithCodeEmitting,
    parser::{ast::bytecode, prelude::*},
};

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
    pub definition_description: String,
}

#[derive(Debug)]
pub struct ParsedFunctionDefinition {
    pub name: String,
    pub parameters: Vec<TypedString>,
    pub local_variables: Vec<TypedString>,
    pub return_type: Option<String>,
}

#[derive(Debug)]
pub struct TypedString {
    pub name: String,
    pub typename: String,
}

impl WithCodeEmitting for TypedString {
    fn emit_code(&self, f: &mut crate::decompiler::CodeEmitter) {
        f.append(&self.name);
        f.append(": ");
        f.append(&self.typename);
    }
}

impl From<&PropertyDefinition> for TypedString {
    fn from(value: &PropertyDefinition) -> Self {
        Self {
            name: value.name.to_string_or_default(),
            typename: value.to_resolved_typename(),
        }
    }
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

        Ok((
            i,
            Self {
                instructions,
                definition_description: String::new(),
            },
        ))
    }
}

impl WithInstructionEmitting for FunctionDefinition {
    fn emit_instruction(&self, f: &mut String) {
        use std::fmt::Write;

        write!(f, "function ").unwrap();
        self.name.emit_instruction(f);
        f.push('\n');

        write!(f, " - override_class = ").unwrap();
        self.override_class.emit_instruction(f);
        f.push('\n');

        write!(f, " - flags = {}", self.flags).unwrap();
        f.push('\n');

        write!(f, " - return_type = ").unwrap();
        if let Some(rt) = self.return_type.as_ref() {
            rt.emit_instruction(f);
        }
        f.push('\n');

        writeln!(f, " - parameters:").unwrap();
        for param in &self.parameters {
            f.push_str("   - ");
            param.emit_instruction(f);
            f.push('\n');
        }

        writeln!(f, " - locals:").unwrap();
        for param in &self.locals {
            f.push_str("   - ");
            param.emit_instruction(f);
            f.push('\n');
        }
    }
}

impl FunctionDefinition {
    pub fn parse_bytecode(&self) -> ParsedFunctionBytecode {
        let mut output = ParsedFunctionBytecode::parse(self.bytecode.as_ref())
            .expect("function_bytecode_parsing_failure")
            .1;

        self.emit_instruction(&mut output.definition_description);

        output
    }

    pub fn parse_definition(&self) -> ParsedFunctionDefinition {
        ParsedFunctionDefinition {
            name: self.name.to_string_or_default(),
            local_variables: self.locals.iter().map(TypedString::from).collect(),
            parameters: self.parameters.iter().map(TypedString::from).collect(),
            return_type: self
                .return_type
                .as_ref()
                .map(|prop| prop.to_resolved_typename()),
        }
    }
}

impl WithInstructionEmitting for ParsedFunctionBytecode {
    fn emit_instruction(&self, output: &mut String) {
        for instr in &self.instructions {
            instr.emit_instruction(output);
            output.push('\n');
        }
    }
}
