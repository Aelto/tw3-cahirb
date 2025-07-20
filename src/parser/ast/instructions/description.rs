use crate::parser::prelude::*;

#[derive(Default)]
pub struct InstructionDescription {
    pub mnemo: &'static str,
    pub operands: Vec<OperandBinding>,
    pub comment: &'static str,
}

static INSTRUCTIONS_SET: std::sync::OnceLock<Vec<InstructionDescription>> =
    std::sync::OnceLock::new();

impl InstructionDescription {
    pub fn from_index<'a>(index: usize) -> Option<&'a Self> {
        get_instructions_set().get(index)
    }

    pub fn size() -> usize {
        get_instructions_set().len()
    }
}

fn get_instructions_set<'a>() -> &'a Vec<InstructionDescription> {
    INSTRUCTIONS_SET.get_or_init(|| vec![
    InstructionDescription {
        mnemo: "Nop",
        comment: "No operation",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "Null",
        comment: "CObject* NULL",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "IntOne",
        comment: "Integer '1'",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "IntZero",
        comment: "Integer '0'",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "IntConst",
        comment: "Int32 constant",
        operands: vec![("i32", "value")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ShortConst",
        comment: "Short ( 16bit int ) constant",
        operands: vec![("i16", "value")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "FloatConst",
        comment: "Float constant",
        operands: vec![("float", "value")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "StringConst",
        comment: "String constant",
        operands: vec![("string", "value")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "NameConst",
        comment: "Name constant",
        operands: vec![("name", "value")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ByteConst",
        comment: "Byte constant",
        operands: vec![("u8", "value")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "BoolTrue",
        comment: "True",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "BoolFalse",
        comment: "False",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "Breakpoint",
        comment: "Breakpoint wrapper, generated only in debug code",
        operands: vec![("u32", "source_line"), ("bool", "is_set")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "Assign",
        comment: "Assign value",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "Target",
        comment: "Target of a label",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "LocalVar",
        comment: "Access to local variable",
        operands: vec![("name", "name")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ParamVar",
        comment: "Access to function parameter variable",
        operands: vec![("name", "name")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ObjectVar",
        comment: "Access to object variable",
        operands: vec![("class_prop", "property")],
    }, // may use imp_prop in the futu, ..Default::default()re
    InstructionDescription {
        mnemo: "ObjectBindableVar",
        comment: "Access to bindable object variable",
        operands: vec![("class_prop", "property")],
    }, // may use imp_prop in the futu, ..Default::default()re
    InstructionDescription {
        mnemo: "DefaultVar",
        comment: "Access to variable from default object",
        operands: vec![("class_prop", "property")],
    }, // may use imp_prop in the futu, ..Default::default()re
    InstructionDescription {
        mnemo: "Switch",
        comment: "Switch statement",
        operands: vec![("imp_type", "expr_type"), ("i16", "skip_offset")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "SwitchLabel",
        comment: "Label in switch statement",
        operands: vec![("i16", "unused"), ("i16", "expr_skip_offset")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "SwitchDefault",
        comment: "Default switch statement",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "Jump",
        comment: "Jump to target",
        operands: vec![("i16", "skip_offset")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "JumpIfFalse",
        comment: "Jump if condition is false",
        operands: vec![("i16", "skip_offset")],
        ..Default::default()
    },
    // Used as check mark in a few places: Bool && Bool, Bool || Bool, PF_FuncSkipParam
    InstructionDescription {
        mnemo: "Skip",
        comment: "Special marker in some constructs, not an executable instruction",
        operands: vec![("i16", "skip_offset")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "Conditional",
        comment: "Conditional expression. Looks broken and unusable.",
        operands: vec![("i16", "select_offset"), ("i16", "skip_offset")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "Constructor",
        comment: "Constructor",
        operands: vec![("u8", "num_params"), ("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "FinalFunc",
        comment: "Call to final function ( static function binding )",
        operands: vec![
            ("u16", "skip_offset"),
            ("u16", "source_line"),
            ("imp_func", "function"),
        ],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "VirtualFunc",
        comment: "Call to virtual function",
        operands: vec![
            ("u16", "skip_offset"),
            ("u16", "source_line"),
            ("name", "function"),
        ],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "VirtualParentFunc",
        comment: "Call to derived parent function ( no state machine )",
        operands: vec![
            ("u16", "skip_offset"),
            ("u16", "source_line"),
            ("name", "function"),
        ],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "EntryFunc",
        comment: "Call to state entry function",
        operands: vec![("u16", "skip_offset"), ("name", "function")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ParamEnd",
        comment: "End of parameters",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "Return",
        comment: "Return from function",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "StructMember",
        comment: "Access to structure member ( slow )",
        operands: vec![("class_prop", "property")],
    }, // may use imp_prop in the futu, ..Default::default()re
    InstructionDescription {
        mnemo: "Context",
        comment: "Evaluation context change",
        operands: vec![("u16", "skip_offset")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "TestEqual",
        comment: "Test if two given shit is default",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "TestNotEqual",
        comment: "Test if two given shit is default",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "New",
        comment: "Create object",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "Delete",
        comment: "Delete object",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "This",
        comment: "Reference to self",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "Parent",
        comment: "State machine context ",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "SavePoint",
        comment: "Function state SavePoint",
        operands: vec![("u16", "skip_offset"), ("name", "name")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "SaveValue",
        comment: "Value of a function param saved by a SavePoint",
        operands: vec![("name", "name")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "SavePointEnd",
        comment: "End-of-savepoint-datablock marker",
        ..Default::default()
    },
    // Array access opcodes
    InstructionDescription {
        mnemo: "ArrayClear",
        comment: "Clear the array",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ArraySize",
        comment: "Get the size of the array",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ArrayResize",
        comment: "Resize array",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ArrayFindFirst",
        comment: "Find index of first matching element from the array",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ArrayFindFirstFast",
        comment: "Find index of first matching element from the array ( faster version )",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ArrayFindLast",
        comment: "Find index of last matching element from the array",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ArrayFindLastFast",
        comment: "Find index of last matching element from the array ( faster version )",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ArrayContains",
        comment: "Check if array contains a given item",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ArrayContainsFast",
        comment: "Check if array contains a given item ( faster version )",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ArrayPushBack",
        comment: "Add element to array",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ArrayPopBack",
        comment: "Remove last element from array",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ArrayInsert",
        comment: "Insert element to array",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ArrayRemove",
        comment: "Remove element from array",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ArrayRemoveFast",
        comment: "Remove element from array ( faster version )",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ArrayGrow",
        comment: "Add space to array",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ArrayErase",
        comment: "Erase place in array",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ArrayEraseFast",
        comment: "Fast erase from array, without preserving order of elements",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ArrayLast",
        comment: "Get the last element from array",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ArrayElement",
        comment: "Access to array element",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    // Static array access opcodes
    InstructionDescription {
        mnemo: "StaticArraySize",
        comment: "Get the size of the static array",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "StaticArrayFindFirst",
        comment: "Find index of first matching element from the static array",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "StaticArrayFindFirstFast",
        comment: "Find index of first matching element from the static array ( faster version )",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "StaticArrayFindLast",
        comment: "Find index of last matching element from the static array",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "StaticArrayFindLastFast",
        comment: "Find index of last matching element from the static array ( faster version )",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "StaticArrayContains",
        comment: "Check if static array contains a given item",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "StaticArrayContainsFast",
        comment: "Check if static array contains a given item ( faster version )",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "StaticArrayLast",
        comment: "Get the last element from static array",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "StaticArrayElement",
        comment: "Access to array static element",
        ..Default::default()
    },
    // Casting
    InstructionDescription {
        mnemo: "BoolToByte",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "BoolToInt",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "BoolToFloat",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "BoolToString",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ByteToBool",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ByteToInt",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ByteToFloat",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ByteToString",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "IntToBool",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "IntToByte",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "IntToFloat",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "IntToString",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "IntToEnum",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "FloatToBool",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "FloatToByte",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "FloatToInt",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "FloatToString",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "NameToBool",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "NameToString",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "StringToBool",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "StringToByte",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "StringToInt",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "StringToFloat",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ObjectToBool",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "ObjectToString",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "EnumToString",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "EnumToInt",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "DynamicCast",
        operands: vec![("imp_type", "type")],
        ..Default::default()
    },
    // Globals
    InstructionDescription {
        mnemo: "GetGame",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "GetPlayer",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "GetCamera",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "GetHud",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "GetSound",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "GetDebug",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "GetTimer",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "GetInput",
        ..Default::default()
    },
    InstructionDescription {
        mnemo: "GetTelemetry",
        ..Default::default()
    },
])
}
