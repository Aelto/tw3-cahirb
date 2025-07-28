use crate::decompiler::prelude::*;

#[derive(Debug)]
pub struct FunctionDeclaration {
    pub body_expressions: Vec<Expression>,
}

impl FunctionDeclaration {
    pub fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, body_expressions) = Expression::decompile_many(i.clone())?;

        Ok((i, Self { body_expressions }))
    }
}
