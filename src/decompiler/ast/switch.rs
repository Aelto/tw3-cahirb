use crate::decompiler::prelude::*;

#[derive(Debug)]
pub struct Switch {
    expression: Expression,
    labels: Vec<SwitchLabel>,
}

#[derive(Debug)]
struct SwitchLabel {
    label_expression: Expression,
    body_expressions: Vec<Expression>,
}

impl WithDecompiling for Switch {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, switch) = i.expect("Switch")?;
        let (i, expression) = Expression::decompile(i)?;
        let (i, labels) = SwitchLabel::decompile_many(i.release_offset_limit())?;

        Ok((i, Self { expression, labels }))
    }
}

impl WithDecompiling for SwitchLabel {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, label) = i.expect("SwitchLabel")?;

        let Some(label_expr_offset) = label
            .operands
            .get("expr_skip_offset")
            .and_then(|op| op.to_i32())
        else {
            return Err("SwitchLabel, expected operand expr_skip_offset on instruction but did not find any".to_owned());
        };

        let i = i.within_offset_limit_raw(&label, label_expr_offset as usize + label.size);
        let (i, label_expression) = Expression::decompile(i)?;
        let (i, body_expressions) = Expression::decompile_many(i.within_offset_limit(label))?;

        Ok((
            i.release_offset_limit(),
            Self {
                label_expression,
                body_expressions,
            },
        ))
    }
}
