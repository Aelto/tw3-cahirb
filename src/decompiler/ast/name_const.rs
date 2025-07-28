use crate::decompiler::prelude::*;

#[derive(Debug)]
pub struct NameConst {
    name: String,
}

impl WithDecompiling for NameConst {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, name) = i.expect("NameConst")?;

        Ok((
            i,
            Self {
                name: name.into_emitted_code(),
            },
        ))
    }
}
