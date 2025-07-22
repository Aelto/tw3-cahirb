use crate::decompiler::{DecompileNodeResult, InstructionsIter};

pub trait WithDecompiling: Sized {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self>;

    fn decompile_many<'a>(mut i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Vec<Self>> {
        let mut output = Vec::new();
        while let Ok((new_i, item)) = Self::decompile(i) {
            output.push(item);
            i = new_i;
        }

        Ok((i, output))
    }
}
