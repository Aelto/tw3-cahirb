use crate::decompiler::prelude::*;

#[derive(Debug)]
pub struct FunctionDeclaration {
    pub expressions: Vec<Expression>,
}

impl FunctionDeclaration {
    pub fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let mut expressions = Vec::new();

        let mut i = i;
        while let Ok((new_i, expr)) = Expression::decompile(i.clone()) {
            expressions.push(expr);
            i = new_i;
        }

        Ok((i, Self { expressions }))
    }
}
