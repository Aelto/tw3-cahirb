use crate::decompiler::prelude::*;

#[derive(Debug)]
pub struct Return {
    pub expression: Option<Expression>,
}

impl WithDecompiling for Return {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, _) = i.expect("Return")?;

        let (nop_i, _) = i.clone().skip_to_next_nop()?;
        let (exp_i, expression) = Expression::decompile_maybe(i);

        let (i, expression) = expression
            .map(|expr| match exp_i.is_after_or_equal(&nop_i) {
                true => (nop_i, None),
                false => (exp_i, Some(expr)),
            })
            .unwrap_or_else(|| (nop_i, None));

        Ok((i, Self { expression }))
    }
}
