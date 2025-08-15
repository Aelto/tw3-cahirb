use crate::decompiler::prelude::*;

#[derive(Debug)]
pub struct FunctionCall {
    prefix: Option<MemoryAccess>,
    fn_name: String,
    parameters: Vec<Expression>,
    dbg: bool,
}

impl WithDecompiling for FunctionCall {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        Self::decompile_standard_call(i)
            .or_else(|_| Self::decompile_array_size_call(i))
            .or_else(|_| Self::decompile_array_push_back(i))
            .or_else(|_| Self::decompile_array_clear_call(i))
    }
}

impl FunctionCall {
    fn decompile_standard_call<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, prefix) = MemoryAccess::decompile_maybe(i);
        let (i, fn_name) = i.expect_any(&["VirtualFunc", "FinalFunc"])?;

        let (i, parameters) = Expression::decompile_many(i.within_offset_limit(fn_name))?;
        let mut i = i;

        // if fn_name.is_function_with_param_end() {
        let (new_i, _) = i.expect("ParamEnd")?;
        i = new_i;
        // }

        Ok((
            i.release_offset_limit(),
            Self {
                prefix,
                fn_name: fn_name.into_emitted_code(),
                parameters,
                dbg: fn_name.is_function_with_param_end(),
            },
        ))
    }

    fn decompile_array_size_call<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        // array size goes first, the memory access comes after unlike standard
        // function calls:
        let (i, fn_name) = i.expect("ArraySize")?;
        let (i, prefix) = i.ok(MemoryAccess::decompile(i));

        Ok((
            i,
            Self {
                prefix,
                fn_name: "Size".to_owned(),
                parameters: Vec::new(),
                dbg: false,
            },
        ))
    }

    fn decompile_array_push_back<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        // array push back goes first, the memory access comes after unlike standard
        // function calls:
        let (i, fn_name) = i.expect("ArrayPushBack")?;
        let (i, prefix) = i.ok(MemoryAccess::decompile(i));
        let (i, param) = Expression::decompile(i)?;

        Ok((
            i,
            Self {
                prefix,
                fn_name: "PushBack".to_owned(),
                parameters: vec![param],
                dbg: false,
            },
        ))
    }

    fn decompile_array_clear_call<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        // array clear goes first, the memory access comes after unlike standard
        // function calls:
        let (i, fn_name) = i.expect("ArrayClear")?;
        let (i, prefix) = i.ok(MemoryAccess::decompile(i));

        Ok((
            i,
            Self {
                prefix,
                fn_name: "Clear".to_owned(),
                parameters: Vec::new(),
                dbg: false,
            },
        ))
    }
}
