use crate::decompiler::prelude::*;

#[derive(Debug)]
pub enum BooleanLogic {
    Or {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    And {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Comparison(BooleanComparison),
}

impl WithDecompiling for BooleanLogic {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        Self::decompile_or(i)
            .or_else(|_| Self::decompile_and(i))
            .or_else(|_| Self::decompile_comparison(i))
    }
}

impl BooleanLogic {
    fn decompile_or<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, func) = i.expect("FinalFunc")?;

        if !i
            .operand(func, "function")?
            .is_function_logic_or_bool_bool()
        {
            return Err("BooleanLogic_or, expected OR logic function".to_owned());
        }

        let ((i, left)) = Expression::decompile_boxed(i)?;
        let ((i, right)) = Expression::decompile_boxed(i)?;
        let ((i, _)) = i.expect("ParamEnd")?;

        Ok((i, Self::Or { left, right }))
    }

    fn decompile_and<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, func) = i.expect("FinalFunc")?;

        if !i
            .operand(func, "function")?
            .is_function_logic_and_bool_bool()
        {
            return Err("BooleanLogic_and, expected AND logic function".to_owned());
        }

        let ((i, left)) = Expression::decompile_boxed(i)?;
        let ((i, right)) = Expression::decompile_boxed(i)?;
        let ((i, _)) = i.expect("ParamEnd")?;

        Ok((i, Self::And { left, right }))
    }

    fn decompile_comparison<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, comparison) = BooleanComparison::decompile(i)?;

        Ok((i, Self::Comparison(comparison)))
    }
}
