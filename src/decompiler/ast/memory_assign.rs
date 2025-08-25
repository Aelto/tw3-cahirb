use crate::decompiler::prelude::*;

#[derive(Debug)]
pub struct MemoryAssign {
    pub left: MemoryAccess,
    pub right: Box<Expression>,
}

impl WithDecompiling for MemoryAssign {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, _) = i.expect("Assign")?;
        let (i, left) = MemoryAccess::decompile(i)?;
        let (i, right) = Expression::decompile_boxed(i)?;

        Ok((i, Self { left, right }))
    }
}

impl WithCodeEmitting for MemoryAssign {
    fn emit_code(&self, f: &mut crate::decompiler::CodeEmitter) {
        (&self.left, &"=", &self.right).emit_code(f);
    }
}
