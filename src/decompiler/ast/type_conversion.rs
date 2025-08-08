use crate::decompiler::prelude::*;

#[derive(Debug)]
pub enum TypeConversion {
    ImplicitCasting {
        casting: ImplicitCastingType,
        expression: Expression,
    },
    DynamicCasting {
        target_type: String,
        expression: Expression,
    },
}

impl WithDecompiling for TypeConversion {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        Self::decompile_implicit_casting(i).or_else(|_| Self::decompile_dynamic_casting(i))
    }
}

impl TypeConversion {
    fn decompile_implicit_casting<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, casting) = ImplicitCastingType::decompile(i)?;
        let (i, expression) = Expression::decompile(i)?;

        Ok((
            i,
            Self::ImplicitCasting {
                casting,
                expression,
            },
        ))
    }

    fn decompile_dynamic_casting<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, casting) = i.expect("DynamicCast")?;
        let (i, expression) = Expression::decompile(i)?;

        let Some(target_type) = casting.to_dynamic_cast_type() else {
            return Err(
                "TypeConversion::DynamicCasting, expected type operand on instruction but did not find any".to_owned(),
            );
        };

        Ok((
            i,
            Self::DynamicCasting {
                target_type: target_type,
                expression: expression,
            },
        ))
    }
}

#[derive(Debug)]
pub enum ImplicitCastingType {
    BoolToByte,
    BoolToInt,
    BoolToFloat,
    BoolToString,
    ByteToBool,
    ByteToInt,
    ByteToFloat,
    ByteToString,
    IntToBool,
    IntToByte,
    IntToFloat,
    IntToString,
    IntToEnum,
    FloatToBool,
    FloatToByte,
    FloatToInt,
    FloatToString,
    NameToBool,
    NameToString,
    StringToBool,
    StringToByte,
    StringToInt,
    StringToFloat,
    ObjectToBool,
    ObjectToString,
    EnumToString,
    EnumToInt,
}
impl WithDecompiling for ImplicitCastingType {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, casting) = i.expect_any(&[
            "BoolToByte",
            "BoolToInt",
            "BoolToFloat",
            "BoolToString",
            "ByteToBool",
            "ByteToInt",
            "ByteToFloat",
            "ByteToString",
            "IntToBool",
            "IntToByte",
            "IntToFloat",
            "IntToString",
            "IntToEnum",
            "FloatToBool",
            "FloatToByte",
            "FloatToInt",
            "FloatToString",
            "NameToBool",
            "NameToString",
            "StringToBool",
            "StringToByte",
            "StringToInt",
            "StringToFloat",
            "ObjectToBool",
            "ObjectToString",
            "EnumToString",
            "EnumToInt",
        ])?;

        let casting_type = match casting.mnemo {
            "BoolToByte" => Self::BoolToByte,
            "BoolToInt" => Self::BoolToInt,
            "BoolToFloat" => Self::BoolToFloat,
            "BoolToString" => Self::BoolToString,
            "ByteToBool" => Self::ByteToBool,
            "ByteToInt" => Self::ByteToInt,
            "ByteToFloat" => Self::ByteToFloat,
            "ByteToString" => Self::ByteToString,
            "IntToBool" => Self::IntToBool,
            "IntToByte" => Self::IntToByte,
            "IntToFloat" => Self::IntToFloat,
            "IntToString" => Self::IntToString,
            "IntToEnum" => Self::IntToEnum,
            "FloatToBool" => Self::FloatToBool,
            "FloatToByte" => Self::FloatToByte,
            "FloatToInt" => Self::FloatToInt,
            "FloatToString" => Self::FloatToString,
            "NameToBool" => Self::NameToBool,
            "NameToString" => Self::NameToString,
            "StringToBool" => Self::StringToBool,
            "StringToByte" => Self::StringToByte,
            "StringToInt" => Self::StringToInt,
            "StringToFloat" => Self::StringToFloat,
            "ObjectToBool" => Self::ObjectToBool,
            "ObjectToString" => Self::ObjectToString,
            "EnumToString" => Self::EnumToString,
            "EnumToInt" => Self::EnumToInt,
            _ => {
                return Err(format!(
                    "ImplicitCastingType, unknown casting type {}",
                    casting.mnemo
                ));
            }
        };

        Ok((i, casting_type))
    }
}
