use crate::decompiler::prelude::*;

#[derive(Debug)]
pub enum ConstructorCall {
    StructStyle {
        called_type: String,
        parameters: Vec<Expression>,
    },
    ClassStyle {
        called_type: String,
    },
}

impl WithDecompiling for ConstructorCall {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        Self::decompile_struct_style(i).or_else(|_| Self::decompile_class_style(i))
    }
}

impl ConstructorCall {
    fn decompile_struct_style<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (mut i, constructor) = i.expect("Constructor")?;
        let params_count = constructor
            .operands
            .get("num_params")
            .and_then(|op| op.to_i32())
            .unwrap_or(0);

        let Some(called_type) = constructor
            .operands
            .get("type")
            .map(|op| op.into_emitted_code())
        else {
            return Err("ConstructorCall, no 'type' operand found in instruction".to_owned());
        };

        let mut parameters = Vec::new();
        parameters.reserve(params_count as usize);

        for _ in 0..params_count {
            let (new_i, parameter) = Expression::decompile(i)?;

            i = new_i;
            parameters.push(parameter);
        }

        Ok((
            i,
            Self::StructStyle {
                called_type,
                parameters,
            },
        ))
    }

    fn decompile_class_style<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (mut i, constructor) = i.expect("New")?;

        let Some(called_type) = constructor
            .operands
            .get("type")
            .map(|op| op.into_emitted_code())
        else {
            return Err("ConstructorCall, no 'type' operand found in instruction".to_owned());
        };

        Ok((i, Self::ClassStyle { called_type }))
    }
}
