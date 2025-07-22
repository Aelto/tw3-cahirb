use crate::decompiler::prelude::*;

#[derive(Debug)]
pub enum Expression {
    ParentAccess,
    ThisAccess,
    VarAccess {
        prefix: Option<Box<Expression>>,
        var_name: String,
    },
    FunctionCall {
        fn_name: String,
        parameters: Vec<Expression>,
    },
}

impl Expression {
    pub fn decompile<'a>(
        mut i: impl InstructionsIterator<'a>,
    ) -> DecompileNodeResult<'a, impl InstructionsIterator<'a>, Self> {
        Self::decompile_parent_access(i)
    }

    fn decompile_parent_access<'a>(
        mut i: impl InstructionsIterator<'a>,
    ) -> DecompileNodeResult<'a, impl InstructionsIterator<'a>, Self> {
        expect(&mut i, "Parent")?;
        expect(&mut i, "This")?;

        Ok((i, Self::ParentAccess))
    }

    fn decompile_var_access<'a>(
        mut i: impl InstructionsIterator<'a>,
    ) -> DecompileNodeResult<'a, impl InstructionsIterator<'a>, Self> {
        let (i, some_prefix) = maybe(Self::decompile_parent_access(i.clone()), i);

        Ok((
            i,
            Self::VarAccess {
                prefix: some_prefix.map(Box::new),
                var_name: todo!(),
            },
        ))
    }
}
