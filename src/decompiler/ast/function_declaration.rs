use crate::decompiler::prelude::*;

pub struct FunctionDeclaration {
    pub expressions: Vec<Expression>,
}

impl FunctionDeclaration {
    pub fn decompile<'a>(
        i: &mut impl Iterator<Item = &'a Instruction>,
    ) -> DecompileNodeResult<Self> {
        let mut expressions = Vec::new();
        while let Ok(expr) = Expression::decompile(i) {
            expressions.push(expr);
        }

        Ok(Self { expressions })
    }
}
