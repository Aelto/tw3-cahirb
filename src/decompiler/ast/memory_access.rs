use crate::decompiler::prelude::*;

#[derive(Debug)]
pub enum MemoryAccess {
    This,

    /// Which is implicitly a combination of `parent.this` in the bytecode
    Parent,

    Var {
        prefix: Option<Box<MemoryAccess>>,
        var_name: String,
    },
}

impl WithDecompiling for MemoryAccess {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        Self::decompile_var_access(i)
            .or_else(|_| Self::decompile_parent_access(i))
            .or_else(|_| Self::decompile_this_access(i))
    }
}

impl MemoryAccess {
    fn decompile_this_access<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, _) = i.expect("This")?;

        Ok((i, Self::This))
    }

    fn decompile_parent_access<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, _) = i.expect("Parent")?;
        let (i, _) = i.expect("This")?;

        Ok((i, Self::Parent))
    }

    fn decompile_var_access<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, some_prefix) = i.ok(Self::decompile_parent_access(i));
        let (i, var_name) = i.expect("ObjectVar")?;

        Ok((
            i,
            Self::Var {
                prefix: some_prefix.map(Box::new),
                var_name: var_name.into_emitted_code(),
            },
        ))
    }
}
