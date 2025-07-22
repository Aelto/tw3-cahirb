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
    pub fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        Self::decompile_parent_access(i)
    }

    fn decompile_parent_access<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, _) = i.expect("Parent")?;
        let (i, _) = i.expect("This")?;

        Ok((i, Self::ParentAccess))
    }

    fn decompile_var_access<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, some_prefix) = i.ok(Self::decompile_parent_access(i.clone()));

        Ok((
            i,
            Self::VarAccess {
                prefix: some_prefix.map(Box::new),
                var_name: todo!(),
            },
        ))
    }
}
