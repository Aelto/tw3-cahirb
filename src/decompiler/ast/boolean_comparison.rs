use crate::decompiler::prelude::*;

#[derive(Debug)]
pub struct BooleanComparison {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub operator: ComparisonOperator,
}

#[derive(Debug)]
pub enum ComparisonOperator {
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    Less,
    LessEqual,
}

impl WithDecompiling for BooleanComparison {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        Self::decompile_internal_operator(i)
            .or_else(|_| Self::decompile_test_equal(i))
            .or_else(|_| Self::decompile_test_not_equal(i))
    }
}

impl BooleanComparison {
    fn decompile_internal_operator<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, func) = i.expect("FinalFunc")?;

        let Some(operator) = i.operand(func, "function")?.get_comparison_operator() else {
            return Err("BooleanComparison, expected final comparison function".to_owned());
        };

        let ((i, left)) = Expression::decompile_boxed(i)?;
        let ((i, right)) = Expression::decompile_boxed(i)?;
        let ((i, _)) = i.expect("ParamEnd")?;

        Ok((
            i,
            Self {
                left,
                right,
                operator,
            },
        ))
    }

    fn decompile_test_not_equal<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, _t) = i.expect("TestNotEqual")?;
        let (i, left) = Expression::decompile_boxed(i)?;
        let (i, right) = Expression::decompile_boxed(i)?;

        Ok((
            i,
            Self {
                left,
                right,
                operator: ComparisonOperator::NotEqual,
            },
        ))
    }

    fn decompile_test_equal<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, _) = i.expect("TestEqual")?;
        let (i, left) = Expression::decompile_boxed(i)?;
        let (i, right) = Expression::decompile_boxed(i)?;

        Ok((
            i,
            Self {
                left,
                right,
                operator: ComparisonOperator::Equal,
            },
        ))
    }
}

impl WithCodeEmitting for ComparisonOperator {
    fn emit_code(&self, f: &mut crate::decompiler::CodeEmitter) {
        match self {
            ComparisonOperator::Greater => f.append(">"),
            ComparisonOperator::GreaterEqual => f.append(">="),
            ComparisonOperator::Equal => f.append("=="),
            ComparisonOperator::NotEqual => f.append("!="),
            ComparisonOperator::Less => f.append("<"),
            ComparisonOperator::LessEqual => f.append("<="),
        }
    }
}

impl WithCodeEmitting for BooleanComparison {
    fn emit_code(&self, f: &mut crate::decompiler::CodeEmitter) {
        (&self.left, &self.operator, &self.right).emit_code(f);
    }
}
