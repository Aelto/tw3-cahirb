use crate::decompiler::prelude::*;

#[derive(Debug)]
pub struct BooleanConst {
    value: bool,
}

impl WithDecompiling for BooleanConst {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, boolean) = i.expect_any(&["BoolTrue", "BoolFalse"])?;

        Ok((
            i,
            Self {
                value: boolean.mnemo == "BoolTrue",
            },
        ))
    }
}
