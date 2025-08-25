use crate::decompiler::prelude::*;

#[derive(Debug)]
pub struct IfFalseCheck {
    condition: Box<Expression>,
    body: Vec<Expression>,
    else_check: Option<ElseCheck>,
}

#[derive(Debug)]
struct ElseCheck {
    body: Vec<Expression>,
}

impl WithDecompiling for IfFalseCheck {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, if_check) = i.expect("JumpIfFalse")?;
        let (i, condition) = Expression::decompile_boxed(i)?;

        let mut i = i.within_offset_limit(if_check);
        let mut body = Vec::new();
        let mut some_else_check = None;
        loop {
            if i.is_finished() {
                break;
            }

            // else cases are outside the skip_offset from the if
            if let Ok((new_i, else_check)) = ElseCheck::decompile(i.release_offset_limit()) {
                i = new_i;
                some_else_check = Some(else_check);
                break;
            }

            let Ok((new_i, expression)) = Expression::decompile(i) else {
                break;
            };

            i = new_i.within_offset_limit(if_check);
            body.push(expression);
        }

        Ok((
            i.release_offset_limit(),
            Self {
                condition,
                body,
                else_check: some_else_check,
            },
        ))
    }
}

impl WithDecompiling for ElseCheck {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, jump) = i.expect("Jump")?;
        let (i, expressions) = Expression::decompile_many(i.within_offset_limit(jump))?;

        Ok((i.release_offset_limit(), Self { body: expressions }))
    }
}

impl WithCodeEmitting for IfFalseCheck {
    fn emit_code(&self, f: &mut crate::decompiler::CodeEmitter) {
        (&"if (", &self.condition, &") {").emit_code(f);

        f.add_indent();
        f.linebreak();
        self.body.emit_code(f);
        f.linebreak();
        f.remove_indent();

        if let Some(else_check) = &self.else_check {
            else_check.emit_code(f);
        }

        "}".emit_code(f);
    }
}

impl WithCodeEmitting for ElseCheck {
    fn emit_code(&self, f: &mut crate::decompiler::CodeEmitter) {
        "else {".emit_code(f);

        f.add_indent();
        f.linebreak();
        self.body.emit_code(f);
        f.linebreak();
        f.remove_indent();

        "}".emit_code(f);
    }
}
