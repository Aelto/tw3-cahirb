use crate::decompiler::prelude::*;

#[derive(Debug)]
pub struct Return {
    pub expression: Option<Expression>,
}

impl WithDecompiling for Return {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, _) = i.expect("Return")?;

        let (exp_i, expression) = Expression::decompile_maybe(i);
        let (nop_i, nop) = exp_i.find_next("Nop");

        let (i, expression) = expression
            .map(|expr| match exp_i.is_after_or_equal(&nop_i) {
                true => (nop_i, None),
                false => (exp_i, Some(expr)),
            })
            .unwrap_or_else(|| (nop_i, None));

        Ok((i, Self { expression }))
    }
}

impl WithCodeEmitting for Return {
    fn emit_code(&self, f: &mut crate::decompiler::CodeEmitter) {
        f.append("return ");

        if let Some(expr) = &self.expression {
            f.append(" ");
            expr.emit_code(f);
            f.append(";");
            f.linebreak();
        }
    }
}
