use crate::{decompiler::prelude::*, parser::WithCodeEmitting};

#[derive(Debug)]
pub enum Expression {
    MemoryAccess(MemoryAccess),
    FunctionCall(FunctionCall),
    ConstructorCall(ConstructorCall),
    MemoryAssign(MemoryAssign),
    NameConst(NameConst),
    StringConst(StringConst),
    Number(Number),
    Null,
    IfFalseCheck(IfFalseCheck),
    BooleanLogic(BooleanLogic),
    Return(Box<Return>),
    Delete(Box<Delete>),
    TypeConvertedExpression(Box<TypeConversion>),
    Switch(Box<Switch>),

    /// There are sometimes random Jump instructions, until i've figured out why
    /// there is a blank Jump type of [Expression]
    Jump,
}

impl WithDecompiling for Expression {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        Self::decompile_boolean_logic(i)
            .or_else(|_| Self::decompile_null(i))
            .or_else(|_| Self::decompile_function_call(i))
            .or_else(|_| Self::decompile_constructor_call(i))
            .or_else(|_| Self::decompile_return(i))
            .or_else(|_| Self::decompile_delete(i))
            .or_else(|_| Self::decompile_memory_access(i))
            .or_else(|_| Self::decompile_memory_assign(i))
            .or_else(|_| Self::decompile_name_const(i))
            .or_else(|_| Self::decompile_string_const(i))
            .or_else(|_| Self::decompile_number(i))
            .or_else(|_| Self::decompile_if_false_check(i))
            .or_else(|_| Self::decompile_type_converted(i))
            .or_else(|_| Self::decompile_switch(i))
            .or_else(|_| Self::decompile_jump(i))
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

    fn decompile_constructor_call<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, constructor_call) = ConstructorCall::decompile(i)?;

        Ok((i, Self::ConstructorCall(constructor_call)))
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

    fn decompile_delete<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, delete_statement) = Delete::decompile_boxed(i)?;

        Ok((i, Self::Delete(delete_statement)))
    }

    fn decompile_type_converted<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, type_conversion) = TypeConversion::decompile_boxed(i)?;

        Ok((i, Self::TypeConvertedExpression(type_conversion)))
    }

    fn decompile_switch<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, switch) = Switch::decompile_boxed(i)?;

        Ok((i, Self::Switch(switch)))
    }

    fn decompile_jump<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, _) = i.expect("Jump")?;

        Ok((i, Self::Jump))
    }

    fn decompile_null<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, _) = i.expect("Null")?;

        Ok((i, Self::Null))
    }
}
