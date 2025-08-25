use crate::decompiler::prelude::*;

#[derive(Debug)]
pub struct FunctionCall {
    prefix: Option<MemoryAccess>,
    fn_name: String,
    parameters: Vec<Expression>,
}

impl WithDecompiling for FunctionCall {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        Self::decompile_standard_call(i)
            .or_else(|_| Self::decompile_entry_call(i))
            .or_else(|e| {
                // perform a prefix check to avoid doing any of the array function
                // calls if the next instruction doesn't start with an "Array"
                // mnemo.
                match i.peek().map(|instr| instr.mnemo.starts_with("Array")) {
                    Some(true) => Self::decompile_array_size_call(i)
                        .or_else(|_| Self::decompile_array_push_back(i))
                        .or_else(|_| Self::decompile_array_clear_call(i))
                        .or_else(|_| Self::decompile_array_insert_call(i))
                        .or_else(|_| Self::decompile_array_popback_call(i))
                        .or_else(|_| Self::decompile_array_remove_call(i))
                        .or_else(|_| Self::decompile_array_remove_fast_call(i))
                        .or_else(|_| Self::decompile_array_erase_call(i))
                        .or_else(|_| Self::decompile_array_erase_fast_call(i))
                        .or_else(|_| Self::decompile_array_last_call(i))
                        .or_else(|_| Self::decompile_array_element_call(i))
                        .or_else(|_| Self::decompile_array_contains_call(i))
                        .or_else(|_| Self::decompile_array_contains_fast_call(i))
                        .or_else(|_| Self::decompile_array_find_first_call(i))
                        .or_else(|_| Self::decompile_array_find_first_fast_call(i))
                        .or_else(|_| Self::decompile_array_find_last_call(i))
                        .or_else(|_| Self::decompile_array_find_last_fast_call(i))
                        .or_else(|_| Self::decompile_array_resize_call(i))
                        .or_else(|_| Self::decompile_array_grow_call(i)),
                    _ => Err(e),
                }
            })
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
                fn_name: fn_name.to_function_name(),
                parameters,
            },
        ))
    }

    fn decompile_entry_call<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, prefix) = MemoryAccess::decompile_maybe(i);
        let (i, fn_name) = i.expect("EntryFunc")?;

        let (i, parameters) = Expression::decompile_many(i.within_offset_limit(fn_name))?;
        let (i, _) = i.expect("ParamEnd")?;

        Ok((
            i.release_offset_limit(),
            Self {
                prefix,
                fn_name: fn_name.to_function_name(),
                parameters,
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
            },
        ))
    }

    fn decompile_array_insert_call<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        // array clear goes first, the memory access comes after unlike standard
        // function calls:
        let (i, fn_name) = i.expect("ArrayInsert")?;
        let (i, prefix) = i.ok(MemoryAccess::decompile(i));
        let (i, value) = Expression::decompile(i)?;
        let (i, index) = Expression::decompile(i)?;

        Ok((
            i,
            Self {
                prefix,
                fn_name: "Insert".to_owned(),
                parameters: vec![value, index],
            },
        ))
    }

    fn decompile_array_remove_call<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        // array clear goes first, the memory access comes after unlike standard
        // function calls:
        let (i, fn_name) = i.expect("ArrayRemove")?;
        let (i, prefix) = i.ok(MemoryAccess::decompile(i));
        let (i, value) = Expression::decompile(i)?;

        Ok((
            i,
            Self {
                prefix,
                fn_name: "Remove".to_owned(),
                parameters: vec![value],
            },
        ))
    }

    fn decompile_array_remove_fast_call<'a>(
        i: InstructionsIter<'a>,
    ) -> DecompileNodeResult<'a, Self> {
        // array clear goes first, the memory access comes after unlike standard
        // function calls:
        let (i, fn_name) = i.expect("ArrayRemoveFast")?;
        let (i, prefix) = i.ok(MemoryAccess::decompile(i));
        let (i, value) = Expression::decompile(i)?;

        Ok((
            i,
            Self {
                prefix,
                fn_name: "RemoveFast".to_owned(),
                parameters: vec![value],
            },
        ))
    }

    fn decompile_array_erase_call<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        // array clear goes first, the memory access comes after unlike standard
        // function calls:
        let (i, fn_name) = i.expect("ArrayErase")?;
        let (i, prefix) = i.ok(MemoryAccess::decompile(i));
        let (i, value) = Expression::decompile(i)?;

        Ok((
            i,
            Self {
                prefix,
                fn_name: "Erase".to_owned(),
                parameters: vec![value],
            },
        ))
    }

    fn decompile_array_erase_fast_call<'a>(
        i: InstructionsIter<'a>,
    ) -> DecompileNodeResult<'a, Self> {
        // array clear goes first, the memory access comes after unlike standard
        // function calls:
        let (i, fn_name) = i.expect("ArrayEraseFast")?;
        let (i, prefix) = i.ok(MemoryAccess::decompile(i));
        let (i, value) = Expression::decompile(i)?;

        Ok((
            i,
            Self {
                prefix,
                fn_name: "EraseFast".to_owned(),
                parameters: vec![value],
            },
        ))
    }

    fn decompile_array_resize_call<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        // array clear goes first, the memory access comes after unlike standard
        // function calls:
        let (i, fn_name) = i.expect("ArrayResize")?;
        let (i, prefix) = i.ok(MemoryAccess::decompile(i));
        let (i, value) = Expression::decompile(i)?;

        Ok((
            i,
            Self {
                prefix,
                fn_name: "Resize".to_owned(),
                parameters: vec![value],
            },
        ))
    }

    fn decompile_array_grow_call<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        // array clear goes first, the memory access comes after unlike standard
        // function calls:
        let (i, fn_name) = i.expect("ArrayGrow")?;
        let (i, prefix) = i.ok(MemoryAccess::decompile(i));
        let (i, value) = Expression::decompile(i)?;

        Ok((
            i,
            Self {
                prefix,
                fn_name: "Grow".to_owned(),
                parameters: vec![value],
            },
        ))
    }

    fn decompile_array_contains_call<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        // array clear goes first, the memory access comes after unlike standard
        // function calls:
        let (i, fn_name) = i.expect("ArrayContains")?;
        let (i, prefix) = i.ok(MemoryAccess::decompile(i));
        let (i, value) = Expression::decompile(i)?;

        Ok((
            i,
            Self {
                prefix,
                fn_name: "Contains".to_owned(),
                parameters: vec![value],
            },
        ))
    }

    fn decompile_array_contains_fast_call<'a>(
        i: InstructionsIter<'a>,
    ) -> DecompileNodeResult<'a, Self> {
        // array clear goes first, the memory access comes after unlike standard
        // function calls:
        let (i, fn_name) = i.expect("ArrayContainsFast")?;
        let (i, prefix) = i.ok(MemoryAccess::decompile(i));
        let (i, value) = Expression::decompile(i)?;

        Ok((
            i,
            Self {
                prefix,
                fn_name: "ContainsFast".to_owned(),
                parameters: vec![value],
            },
        ))
    }

    fn decompile_array_find_first_call<'a>(
        i: InstructionsIter<'a>,
    ) -> DecompileNodeResult<'a, Self> {
        // array clear goes first, the memory access comes after unlike standard
        // function calls:
        let (i, fn_name) = i.expect("ArrayFindFirst")?;
        let (i, prefix) = i.ok(MemoryAccess::decompile(i));
        let (i, value) = Expression::decompile(i)?;

        Ok((
            i,
            Self {
                prefix,
                fn_name: "FindFirst".to_owned(),
                parameters: vec![value],
            },
        ))
    }

    fn decompile_array_find_first_fast_call<'a>(
        i: InstructionsIter<'a>,
    ) -> DecompileNodeResult<'a, Self> {
        // array clear goes first, the memory access comes after unlike standard
        // function calls:
        let (i, fn_name) = i.expect("ArrayFindFirstFast")?;
        let (i, prefix) = i.ok(MemoryAccess::decompile(i));
        let (i, value) = Expression::decompile(i)?;

        Ok((
            i,
            Self {
                prefix,
                fn_name: "FindFirstFast".to_owned(),
                parameters: vec![value],
            },
        ))
    }

    fn decompile_array_find_last_call<'a>(
        i: InstructionsIter<'a>,
    ) -> DecompileNodeResult<'a, Self> {
        // array clear goes first, the memory access comes after unlike standard
        // function calls:
        let (i, fn_name) = i.expect("ArrayFindLast")?;
        let (i, prefix) = i.ok(MemoryAccess::decompile(i));
        let (i, value) = Expression::decompile(i)?;

        Ok((
            i,
            Self {
                prefix,
                fn_name: "FindLast".to_owned(),
                parameters: vec![value],
            },
        ))
    }

    fn decompile_array_find_last_fast_call<'a>(
        i: InstructionsIter<'a>,
    ) -> DecompileNodeResult<'a, Self> {
        // array clear goes first, the memory access comes after unlike standard
        // function calls:
        let (i, fn_name) = i.expect("ArrayFindLastFast")?;
        let (i, prefix) = i.ok(MemoryAccess::decompile(i));
        let (i, value) = Expression::decompile(i)?;

        Ok((
            i,
            Self {
                prefix,
                fn_name: "FindLastFast".to_owned(),
                parameters: vec![value],
            },
        ))
    }

    fn decompile_array_popback_call<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        // array clear goes first, the memory access comes after unlike standard
        // function calls:
        let (i, fn_name) = i.expect("ArrayPopBack")?;
        let (i, prefix) = i.ok(MemoryAccess::decompile(i));

        Ok((
            i,
            Self {
                prefix,
                fn_name: "PopBack".to_owned(),
                parameters: Vec::new(),
            },
        ))
    }

    fn decompile_array_last_call<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        // array clear goes first, the memory access comes after unlike standard
        // function calls:
        let (i, fn_name) = i.expect("ArrayLast")?;
        let (i, prefix) = i.ok(MemoryAccess::decompile(i));

        Ok((
            i,
            Self {
                prefix,
                fn_name: "Last".to_owned(),
                parameters: Vec::new(),
            },
        ))
    }

    fn decompile_array_element_call<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        // array clear goes first, the memory access comes after unlike standard
        // function calls:
        let (i, fn_name) = i.expect("ArrayElement")?;
        let (i, prefix) = i.ok(MemoryAccess::decompile(i));
        let (i, index) = Expression::decompile(i)?;

        Ok((
            i,
            Self {
                prefix,
                fn_name: "Element".to_owned(),
                parameters: vec![index],
            },
        ))
    }
}

impl WithCodeEmitting for FunctionCall {
    fn emit_code(&self, f: &mut crate::decompiler::CodeEmitter) {
        if let Some(prefix) = &self.prefix {
            prefix.emit_code(f);
            f.append(".");
        }

        self.fn_name.as_str().emit_code(f);
        "(".emit_code(f);
        (&self.parameters, ", ").emit_code(f);
        ")".emit_code(f);
    }
}
