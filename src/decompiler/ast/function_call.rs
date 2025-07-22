use crate::decompiler::prelude::*;

#[derive(Debug)]
pub struct FunctionCall {
    prefix: Option<MemoryAccess>,
    fn_name: String,
    parameters: Vec<Expression>,
}

impl WithDecompiling for FunctionCall {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, prefix) = i.ok(MemoryAccess::decompile(i));
        let (i, fn_name) = i.expect("VirtualFunc")?;

        let (i, parameters) = Expression::decompile_many(i.within_offset_limit(fn_name))?;
        let (i, _) = i.release_offset_limit().expect("ParamEnd")?;

        Ok((
            i,
            Self {
                prefix,
                fn_name: fn_name.into_emitted_code(),
                parameters,
            },
        ))
    }
}
