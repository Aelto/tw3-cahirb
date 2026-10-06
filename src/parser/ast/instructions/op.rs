use crate::parser::prelude::*;

use super::operand_types::*;

macro_rules! instruction_set {
    ($( $opcode:literal => $name:ident $({ $($field:ident : $ty:ty),+ $(,)? })? ),+ $(,)?) => {
        #[derive(Debug, Clone)]
        pub enum Op {
            $( $name $({ $($field: $ty),+ })? ),+
        }

        impl Op {
            pub fn opcode(&self) -> u8 {
                match self {
                    $( Self::$name { .. } => $opcode ),+
                }
            }

            pub fn mnemo(&self) -> &'static str {
                match self {
                    $( Self::$name { .. } => stringify!($name) ),+
                }
            }

            pub fn runtime_size(&self) -> usize {
                match self {
                    $( Self::$name $({ $($field),+ })? => 1 $($( + Operand::runtime_size($field) )+)? ),+
                }
            }

            pub fn operands(&self) -> Vec<(&'static str, OperandValue)> {
                match self {
                    $( Self::$name $({ $($field),+ })? => vec![
                        $($( (stringify!($field), Operand::to_operand_value($field)) ),+)?
                    ] ),+
                }
            }

            pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
                let (i, opcode) = parse_u8(input)?;

                match opcode {
                    $( $opcode => {
                        $($( let (i, $field) = <$ty as Operand>::parse_operand(i)?; )+)?
                        Ok((i, Self::$name $({ $($field),+ })?))
                    } )+
                    _ => Err(nom::Err::Failure(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Switch,
                    ))),
                }
            }

            pub fn write(&self, w: &mut BlobWriter) {
                w.write_u8(self.opcode());

                match self {
                    $( Self::$name $({ $($field),+ })? => {
                        $($( Operand::write_operand($field, w); )+)?
                    } ),+
                }
            }
        }
    };
}

instruction_set! {
    0 => Nop,
    1 => Null,
    2 => IntOne,
    3 => IntZero,
    4 => IntConst { value: i32 },
    5 => ShortConst { value: i16 },
    6 => FloatConst { value: f32 },
    7 => StringConst { value: String },
    8 => NameConst { value: CName },
    9 => ByteConst { value: u8 },
    10 => BoolTrue,
    11 => BoolFalse,
    12 => Breakpoint { source_line: u32, is_set: bool },
    13 => Assign,
    14 => Target,
    15 => LocalVar { name: VariableRef },
    16 => ParamVar { name: VariableRef },
    17 => ObjectVar { property: PropertyRef },
    18 => ObjectBindableVar { property: PropertyRef },
    19 => DefaultVar { property: PropertyRef },
    20 => Switch { expr_type: ImportTypeRef, skip_offset: i16 },
    21 => SwitchLabel { next_label_offset: i16, expr_skip_offset: i16 },
    22 => SwitchDefault,
    23 => Jump { skip_offset: i16 },
    24 => JumpIfFalse { skip_offset: i16 },
    25 => Skip { skip_offset: i16 },
    26 => Conditional { select_offset: i16, skip_offset: i16 },
    27 => Constructor { num_params: u8, type_ref: ImportTypeRef },
    28 => FinalFunc { skip_offset: u16, source_line: u16, function: FunctionRef },
    29 => VirtualFunc { skip_offset: u16, source_line: u16, function: CName },
    30 => VirtualParentFunc { skip_offset: u16, source_line: u16, function: CName },
    31 => EntryFunc { skip_offset: u16, function: CName },
    32 => ParamEnd,
    33 => Return,
    34 => StructMember { property: PropertyRef },
    35 => Context { skip_offset: u16 },
    36 => TestEqual { type_ref: ImportTypeRef },
    37 => TestNotEqual { type_ref: ImportTypeRef },
    38 => New { type_ref: ImportTypeRef },
    39 => Delete,
    40 => This,
    41 => Parent,
    42 => SavePoint { skip_offset: u16, name: CName },
    43 => SaveValue { name: CName },
    44 => SavePointEnd,
    45 => ArrayClear { type_ref: ImportTypeRef },
    46 => ArraySize { type_ref: ImportTypeRef },
    47 => ArrayResize { type_ref: ImportTypeRef },
    48 => ArrayFindFirst { type_ref: ImportTypeRef },
    49 => ArrayFindFirstFast { type_ref: ImportTypeRef },
    50 => ArrayFindLast { type_ref: ImportTypeRef },
    51 => ArrayFindLastFast { type_ref: ImportTypeRef },
    52 => ArrayContains { type_ref: ImportTypeRef },
    53 => ArrayContainsFast { type_ref: ImportTypeRef },
    54 => ArrayPushBack { type_ref: ImportTypeRef },
    55 => ArrayPopBack { type_ref: ImportTypeRef },
    56 => ArrayInsert { type_ref: ImportTypeRef },
    57 => ArrayRemove { type_ref: ImportTypeRef },
    58 => ArrayRemoveFast { type_ref: ImportTypeRef },
    59 => ArrayGrow { type_ref: ImportTypeRef },
    60 => ArrayErase { type_ref: ImportTypeRef },
    61 => ArrayEraseFast { type_ref: ImportTypeRef },
    62 => ArrayLast { type_ref: ImportTypeRef },
    63 => ArrayElement { type_ref: ImportTypeRef },
    64 => StaticArraySize,
    65 => StaticArrayFindFirst,
    66 => StaticArrayFindFirstFast,
    67 => StaticArrayFindLast,
    68 => StaticArrayFindLastFast,
    69 => StaticArrayContains,
    70 => StaticArrayContainsFast,
    71 => StaticArrayLast,
    72 => StaticArrayElement,
    73 => BoolToByte,
    74 => BoolToInt,
    75 => BoolToFloat,
    76 => BoolToString,
    77 => ByteToBool,
    78 => ByteToInt,
    79 => ByteToFloat,
    80 => ByteToString,
    81 => IntToBool,
    82 => IntToByte,
    83 => IntToFloat,
    84 => IntToString,
    85 => IntToEnum { type_ref: ImportTypeRef },
    86 => FloatToBool,
    87 => FloatToByte,
    88 => FloatToInt,
    89 => FloatToString,
    90 => NameToBool,
    91 => NameToString,
    92 => StringToBool,
    93 => StringToByte,
    94 => StringToInt,
    95 => StringToFloat,
    96 => ObjectToBool,
    97 => ObjectToString,
    98 => EnumToString { type_ref: ImportTypeRef },
    99 => EnumToInt { type_ref: ImportTypeRef },
    100 => DynamicCast { type_ref: ImportTypeRef },
    101 => GetGame,
    102 => GetPlayer,
    103 => GetCamera,
    104 => GetHud,
    105 => GetSound,
    106 => GetDebug,
    107 => GetTimer,
    108 => GetInput,
    109 => GetTelemetry,
}
