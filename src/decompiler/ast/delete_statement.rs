use crate::decompiler::prelude::*;

#[derive(Debug)]
pub struct Delete {
    pub access: MemoryAccess,
}

impl WithDecompiling for Delete {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, _) = i.expect("Delete")?;
        let (i, access) = MemoryAccess::decompile(i)?;

        Ok((i, Self { access }))
    }
}
