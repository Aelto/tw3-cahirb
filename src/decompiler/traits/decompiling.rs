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

    fn decompile_boxed<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Box<Self>> {
        let (i, res) = Self::decompile(i)?;

        Ok((i, Box::new(res)))
    }

    /// A decompile method that returns an option of Self instead of a result,
    /// it's an alternative to [InstructionsIter::ok].
    fn decompile_maybe<'a>(i: InstructionsIter<'a>) -> (InstructionsIter<'a>, Option<Self>) {
        match Self::decompile(i) {
            Ok((i, res)) => ((i, Some(res))),
            Err(_) => ((i, None)),
        }
    }
}
