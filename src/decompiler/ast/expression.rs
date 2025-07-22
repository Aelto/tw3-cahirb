use crate::{decompiler::prelude::*, parser::WithCodeEmitting};

#[derive(Debug)]
pub enum Expression {
    MemoryAccess(MemoryAccess),
    FunctionCall(FunctionCall),
}

impl WithDecompiling for Expression {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        Self::decompile_function_call(i).or_else(|_| Self::decompile_memory_access(i))
    }
}

impl Expression {
    fn decompile_memory_access<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, memory_access) = MemoryAccess::decompile(i)?;

        Ok((i, Self::MemoryAccess(memory_access)))
    }

    fn decompile_function_call<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, function_call) = FunctionCall::decompile(i)?;

        Ok((i, Self::FunctionCall(function_call)))
    }
}
