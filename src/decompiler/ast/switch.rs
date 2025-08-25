use crate::decompiler::{iterator::obtain_offset_limit, prelude::*};

#[derive(Debug)]
pub struct Switch {
    expression: Expression,
    cases: Vec<SwitchCase>,
}

#[derive(Debug)]
struct SwitchCase {
    label_expressions: Vec<SwitchCaseLabel>,

    body_expressions: Vec<Expression>,
}

#[derive(Debug)]
struct SwitchCaseLabel {
    expression: Expression,
    pub offset_limit: i64,
}

impl WithDecompiling for Switch {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, switch) = i.expect("Switch")?;
        let (i, expression) = Expression::decompile(i)?;
        let (i, cases) = SwitchCase::decompile_many(i.release_offset_limit())?;

        Ok((i, Self { expression, cases }))
    }
}

impl WithDecompiling for SwitchCase {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, label_expressions) = SwitchCaseLabel::decompile_many(i)?;

        let Some(first_label) = label_expressions.first() else {
            return Err(
                "SwitchCase, expected at least one SwitchLabel but did not find any".to_owned(),
            );
        };

        let (i, body_expressions) = Expression::decompile_many(
            i.within_offset_limit_absolute(first_label.offset_limit as usize),
        )?;

        Ok((
            i.release_offset_limit(),
            Self {
                label_expressions,
                body_expressions,
            },
        ))
    }
}

impl WithDecompiling for SwitchCaseLabel {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, label) = i.expect("SwitchLabel")?;

        let offset_limit = obtain_offset_limit(label);

        let Some(label_expr_offset) = label
            .operands
            .get("expr_skip_offset")
            .and_then(|op| op.to_i32())
        else {
            return Err("SwitchLabel, expected operand expr_skip_offset on instruction but did not find any".to_owned());
        };

        let i = i.within_offset_limit_raw(&label, label_expr_offset as usize + label.size);
        let (i, label_expression) = Expression::decompile(i)?;

        Ok((
            i,
            Self {
                expression: label_expression,
                offset_limit: offset_limit,
            },
        ))
    }
}

impl WithCodeEmitting for SwitchCaseLabel {
    fn emit_code(&self, f: &mut crate::decompiler::CodeEmitter) {
        (&"case ", &self.expression, &":").emit_code(f);
    }
}

impl WithCodeEmitting for SwitchCase {
    fn emit_code(&self, f: &mut crate::decompiler::CodeEmitter) {
        for case in &self.label_expressions {
            case.emit_code(f);
            f.linebreak();
        }

        f.add_indent();
        self.body_expressions.emit_code(f);
        "break;".emit_code(f);
        f.linebreak();
        f.remove_indent();
    }
}

impl WithCodeEmitting for Switch {
    fn emit_code(&self, f: &mut crate::decompiler::CodeEmitter) {
        (&"switch (", &self.expression, &") {").emit_code(f);

        f.add_indent();
        f.linebreak();
        self.cases.emit_code(f);
        f.remove_indent();
        f.linebreak();
        "}".emit_code(f);
    }
}
