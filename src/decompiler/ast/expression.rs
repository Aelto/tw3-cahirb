use crate::{decompiler::prelude::*, parser::WithCodeEmitting};

#[derive(Debug)]
pub enum Expression {
    MemoryAccess(MemoryAccess),
    FunctionCall(FunctionCall),
    MemoryAssign(MemoryAssign),
    NameConst(NameConst),
    StringConst(StringConst),
    Number(Number),
    IfFalseCheck(IfFalseCheck),
    BooleanLogic(BooleanLogic),
    Return(Box<Return>),
    TypeConvertedExpression(Box<TypeConversion>),
}

impl WithDecompiling for Expression {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        Self::decompile_boolean_logic(i)
            .or_else(|_| Self::decompile_function_call(i))
            .or_else(|_| Self::decompile_memory_access(i))
            .or_else(|_| Self::decompile_memory_assign(i))
            .or_else(|_| Self::decompile_name_const(i))
            .or_else(|_| Self::decompile_string_const(i))
            .or_else(|_| Self::decompile_number(i))
            .or_else(|_| Self::decompile_if_false_check(i))
            .or_else(|_| Self::decompile_return(i))
            .or_else(|_| Self::decompile_type_converted(i))
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

    fn decompile_memory_assign<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, memory_assign) = MemoryAssign::decompile(i)?;

        Ok((i, Self::MemoryAssign(memory_assign)))
    }

    fn decompile_name_const<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, name_const) = NameConst::decompile(i)?;

        Ok((i, Self::NameConst(name_const)))
    }

    fn decompile_string_const<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, string_const) = StringConst::decompile(i)?;

        Ok((i, Self::StringConst(string_const)))
    }

    fn decompile_number<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, number) = Number::decompile(i)?;

        Ok((i, Self::Number(number)))
    }

    fn decompile_if_false_check<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, if_false_check) = IfFalseCheck::decompile(i)?;

        Ok((i, Self::IfFalseCheck(if_false_check)))
    }

    fn decompile_boolean_logic<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, boolean_logic) = BooleanLogic::decompile(i)?;

        Ok((i, Self::BooleanLogic(boolean_logic)))
    }

    fn decompile_return<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, return_statement) = Return::decompile_boxed(i)?;

        Ok((i, Self::Return(return_statement)))
    }

    fn decompile_type_converted<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, type_conversion) = TypeConversion::decompile_boxed(i)?;

        Ok((i, Self::TypeConvertedExpression(type_conversion)))
    }
}
