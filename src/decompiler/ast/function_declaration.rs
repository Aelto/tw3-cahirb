use crate::{
    decompiler::prelude::*,
    parser::ast::{FunctionDefinition, ParsedFunctionBytecode, ParsedFunctionDefinition},
};

#[derive(Debug)]
pub struct FunctionDeclaration {
    pub body_expressions: Vec<Expression>,
    pub definition: ParsedFunctionDefinition,
}

impl FunctionDeclaration {
    pub fn decompile<'a>(
        i: InstructionsIter<'a>,
        definition: ParsedFunctionDefinition,
    ) -> DecompileNodeResult<'a, Self> {
        let (i, body_expressions) = Expression::decompile_many(i)?;

        Ok((
            i,
            Self {
                body_expressions,
                definition,
            },
        ))
    }
}

impl WithCodeEmitting for FunctionDeclaration {
    fn emit_code(&self, f: &mut crate::decompiler::CodeEmitter) {
        f.append("function ");
        f.append(&self.definition.name);

        let params = (&self.definition.parameters, ", ");
        (&"(", &params, &")").emit_code(f);

        if let Some(returntype) = &self.definition.return_type {
            f.append(": ");
            f.append(returntype);
        }

        f.append(" {");
        f.add_indent();
        f.linebreak();
        self.body_expressions.emit_code(f);
        f.remove_indent();
        f.linebreak();
        f.append("}");
    }
}
