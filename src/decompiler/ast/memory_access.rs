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

    ParamVar {
        var_name: String,
    },

    LocalVar {
        var_name: String,
    },

    StructMember {
        member_name: String,
        prefix: Box<MemoryAccess>,
    },

    Global(Globals),
}

impl WithDecompiling for MemoryAccess {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        Self::decompile_var_access(i)
            .or_else(|_| Self::decompile_struct_member_access(i))
            .or_else(|_| Self::decompile_prefix(i))
    }
}

impl MemoryAccess {
    /// a separated decompile function to catch var access prefixes without
    /// causing a stack overflow.
    fn decompile_prefix<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        Self::decompile_parent_access(i)
            .or_else(|_| Self::decompile_this_access(i))
            .or_else(|_| Self::decompile_var_param_access(i))
            .or_else(|_| Self::decompile_local_var_access(i))
            .or_else(|_| Self::decompile_local_global_access(i))
    }

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
        let (i, some_prefix) = i.ok(Self::decompile_prefix(i));
        let (i, var_name) = i.expect("ObjectVar")?;

        Ok((
            i,
            Self::Var {
                prefix: some_prefix.map(Box::new),
                var_name: var_name.into_emitted_code(),
            },
        ))
    }

    fn decompile_var_param_access<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, var_name) = i.expect("ParamVar")?;

        Ok((
            i,
            Self::ParamVar {
                var_name: var_name.into_emitted_code(),
            },
        ))
    }

    fn decompile_local_var_access<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, var_name) = i.expect("LocalVar")?;

        Ok((
            i,
            Self::LocalVar {
                var_name: var_name.into_emitted_code(),
            },
        ))
    }

    fn decompile_local_global_access<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, global) = Globals::decompile(i)?;

        Ok((i, Self::Global(global)))
    }

    fn decompile_struct_member_access<'a>(
        i: InstructionsIter<'a>,
    ) -> DecompileNodeResult<'a, Self> {
        let (i, struct_node) = i.expect("StructMember")?;
        let (i, prefix) = Self::decompile_prefix(i)?;

        Ok((
            i,
            Self::StructMember {
                member_name: struct_node.into_emitted_code(),
                prefix: Box::new(prefix),
            },
        ))
    }
}
