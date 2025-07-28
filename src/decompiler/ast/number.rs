use crate::decompiler::prelude::*;

#[derive(Debug)]
pub enum Number {
    Int(i32),
    Float(f32),
}

impl WithDecompiling for Number {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        Self::decompile_int(i)
            .or_else(|_| Self::decompile_int_zero(i))
            .or_else(|_| Self::decompile_int_one(i))
            .or_else(|_| Self::decompile_short(i))
            .or_else(|_| Self::decompile_float(i))
    }
}

impl Number {
    fn decompile_int_zero<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, _) = i.expect("IntZero")?;

        Ok((i, Self::Int(0)))
    }

    fn decompile_int_one<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, _) = i.expect("IntOne")?;

        Ok((i, Self::Int(1)))
    }

    fn decompile_int<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, instr) = i.expect("IntConst")?;
        let value = i.operand(instr, "value")?;

        let value: i32 = i32::try_from(value)?;

        Ok((i, Self::Int(value)))
    }

    fn decompile_short<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, instr) = i.expect("ShortConst")?;
        let value = i.operand(instr, "value")?;

        let value: i32 = i32::try_from(value)?;

        Ok((i, Self::Int(value)))
    }

    fn decompile_float<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, instr) = i.expect("FloatConst")?;
        let value = i.operand(instr, "value")?;

        let value: f32 = f32::try_from(value)?;

        Ok((i, Self::Float(value)))
    }
}
