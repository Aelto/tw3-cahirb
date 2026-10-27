use crate::decompiler::{iterator::obtain_offset_limit, prelude::*};

#[derive(Debug)]
pub struct Context {
    expressions: Vec<Expression>,
}

impl WithDecompiling for Context {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, context) = i.expect_raw("Context")?;
        let (i, expressions) = Expression::decompile_many(i.within_offset_limit(context, -1))?;

        Ok((i.release_exhausted_offset_limit()?, Self { expressions }))
    }
}

impl WithCodeEmitting for Context {
    fn emit_code(&self, f: &mut crate::decompiler::CodeEmitter) {
        f.append("/*context*/");
        self.expressions.emit_code(f);
    }
}
