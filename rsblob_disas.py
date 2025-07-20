
################################################################
# Serialization (simple types, stateless)
################################################################

import io
import struct

def read_u8(st: io.BufferedIOBase):
    return st.read(1)[0] & 0xFF

def read_u16(st: io.BufferedIOBase):
    return struct.unpack("<H", st.read(2))[0]

def read_u32(st: io.BufferedIOBase):
    return struct.unpack("<I", st.read(4))[0]

def read_u64(st: io.BufferedIOBase):
    return struct.unpack("<Q", st.read(8))[0]

def read_i8(st: io.BufferedIOBase):
    return struct.unpack("b", st.read(1))[0]

def read_i16(st: io.BufferedIOBase):
    return struct.unpack("<h", st.read(2))[0]

def read_i32(st: io.BufferedIOBase):
    return struct.unpack("<i", st.read(4))[0]

def read_i64(st: io.BufferedIOBase):
    return struct.unpack("<q", st.read(8))[0]

def read_float(st: io.BufferedIOBase):
    return struct.unpack("<f", st.read(4))[0]


def read_lp8str(st: io.BufferedIOBase):
    """ Read U8-Length-Prefixed String from stream. """
    return st.read(read_u8(st))

def read_compressed_i32(st: io.BufferedIOBase):
    result = 0
    b0 = read_u8(st)
    is_negative = b0 & 0x80
    result |= b0 & 0x3F
    if b0 & 0x40:
        b1 = read_u8(st)
        result |= (b1 & 0x7F) << 6
        if b1 & 0x80:
            b2 = read_u8(st)
            result |= (b2 & 0x7F) << 13
            if b2 & 0x80:
                b3 = read_u8(st)
                result |= (b3 & 0x7F) << 20
                if b3 & 0x80:
                    b4 = read_u8(st)
                    result |= (b4 & 0x7F) << 27
    if is_negative:
        result = -result

    return result

def read_compressed_u32(st: io.BufferedIOBase):
    return read_compressed_i32(st) & 0xFFFFFFFF

def read_string(st: io.BufferedIOBase):
    """ Read RED String from stream. """
    len = read_compressed_i32(st)
    if len <= 0: # utf8
        return st.read(-len).decode("utf-8")
    else: # utf16
        return st.read(2*len).decode("utf-16-le")

################################################################
# Hashing
################################################################

def fnv1a32(s):
    if isinstance(s, str):
        s = s.encode()
    h = 0x811c9dc5
    for b in s:
        h ^= b
        h = (h * 0x01000193) & 0xFFFFffff
    return h

def cname_hash(s):
    if isinstance(s, str):
        s = s.encode()
    return fnv1a32(s + b'\0')

assert fnv1a32("witcher") == 0x12215957

################################################################
# Structure Helpers
################################################################

from ctypes import LittleEndianStructure, sizeof, c_uint32

class Struct(LittleEndianStructure):
    @classmethod
    def read_from(cls, st: io.BufferedIOBase):
        return cls.from_buffer_copy(st.read(sizeof(cls)))

def extract_bitfields_unsigned(storage_value, *fieldsizes):
    """
    Extract bitfields from a storage value.
    
    :param storage_value: The storage value.
    :param *fieldsizes: The size in bits of fields in BE order.
    :return: The extracted bitfields in BE order (same order as fieldsizes).
    """
    fields = []
    for bit_size in fieldsizes[::-1]: # Process in LE order
        fields.append(storage_value & ((1 << bit_size) - 1))
        storage_value >>= bit_size
    return fields[::-1] # LE -> BE


################################################################
# Structures
################################################################

from datetime import datetime

class CDateTime(Struct):
    _fields_ = [
        ("date", c_uint32),
        ("time", c_uint32),
    ]

    def to_datetime(self):
        y, mo, d, _ = extract_bitfields_unsigned(self.date, 12, 5, 5, 10)
        h, m, s, ms = extract_bitfields_unsigned(self.date, 5, 6, 6, 10)
        return datetime(y, mo, d, h, m, s, ms * 1000)

################################################################
# Enums/Flags
################################################################

def FLAG(i):
    return 1 << i

# Class Flags

CF_Abstract                     = FLAG( 0 ) # Class is abstract, no instance of it can be created
CF_Native                       = FLAG( 1 ) # Class is defined in C++
CF_Scripted                     = FLAG( 2 ) # Class has definition in script
CF_Exported                     = FLAG( 3 ) # Class definition has been exported to C++ code
CF_State                        = FLAG( 4 ) # Class is a state class
CF_NoDefaultObjectSerialization = FLAG( 5 ) # Don't compare properties to default object on serialize
CF_AlwaysTransient              = FLAG( 7 ) # NEVER save or load objects of this class to ANY storage
CF_EditorOnly                   = FLAG( 8 ) # Class and all derived classes should be used in editor only
CF_UndefinedFunctions           = FLAG( 9 ) # This class has one or more undefined functions
CF_StateMachine                 = FLAG( 10 ) # Class is allowed to have states (set by scripts checked by script compiler only)
CF_ODRTag                       = FLAG( 11 ) # Used by the compiler to protect against multiple definitions per unit.

# Function Flags

FF_NativeFunction       = FLAG( 0 ) # Function is native ( implemented in C++ )
FF_StaticFunction       = FLAG( 1 ) # Function is static
FF_OperatorFunction     = FLAG( 2 ) # Function is data operator
FF_ExportedFunction     = FLAG( 3 ) # Function is native function that was exported to script
FF_FinalFunction        = FLAG( 4 ) # Function is final and cannot be overridden in child classes
FF_EventFunction        = FLAG( 5 ) # Function is special event function
FF_LatentFunction       = FLAG( 6 ) # Function takes time to execute
FF_EntryFunction        = FLAG( 7 ) # Function is a state entry function
FF_ExecFunction         = FLAG( 8 ) # Function can be called from console
FF_UndefinedBody        = FLAG( 9 ) # Function has no body (just a declaration)
FF_TimerFunction        = FLAG( 10 ) # Function is a timer
FF_SceneFunction        = FLAG( 11 ) # Function can be used in Scenes
FF_QuestFunction        = FLAG( 12 ) # Function can be used in Quests
FF_CleanupFunction      = FLAG( 13 ) # Function is a cleanup
FF_PrivateFunction      = FLAG( 14 ) # Function is private
FF_ProtectedFunction    = FLAG( 15 ) # Function is protected
FF_PublicFunction       = FLAG( 16 ) # Function is public
FF_RewardFunction       = FLAG( 17 ) # Function can be attached to reward
FF_ExtReplaceFunction   = FLAG( 18 ) # Function is an overwrite function
FF_ExtAddFunction       = FLAG( 19 ) # Function is an add function
FF_ExtWrapFunction      = FLAG( 20 ) # Function is a wrap function
FF_ODR                  = FLAG( 21 ) # Used by the compiler to protect against multiple definitions per unit.
FF_HidingNative         = FLAG( 22 ) # Function is scripted but stole the name of a native. Used by wrappers.

# Property Flags

PF_Editable             = FLAG( 0 ) # Property is visible in the editor's property browser
PF_ReadOnly             = FLAG( 1 ) # Property is read only
PF_Inlined              = FLAG( 2 ) # Inline property edition ( for object properties only )
PF_NotSerialized        = FLAG( 3 ) # Use this flag to grant RTTI access to the field, but prevent serializing it
PF_NotCooked            = FLAG( 4 ) # Property with this flag is not cooked to final build packages
PF_Scripted             = FLAG( 5 ) # Property is script property of CLASS
PF_FuncRetValue         = FLAG( 6 ) # Return property of function
PF_FuncParam            = FLAG( 7 ) # Function parameter
PF_FuncLocal            = FLAG( 8 ) # Function local variable
PF_FuncOutParam         = FLAG( 9 ) # Function parameter that is passed by reference ( can by modified by function )
PF_FuncOptionaParam     = FLAG( 10 ) # Function parameter is optional, does not need to be specified
PF_FuncSkipParam        = FLAG( 11 ) # Function parameter which evaluation can be skipped, used in native functions
PF_Config               = FLAG( 12 ) # Property is saved/loaded from config
PF_Exported             = FLAG( 13 ) # Property was exported from C++ and can be used in script code
PF_Native               = FLAG( 14 ) # Property is defined in C++
PF_Saved                = FLAG( 15 ) # Property that will be saved to a gamesave file
PF_Private              = FLAG( 16 ) # Property is private
PF_Protected            = FLAG( 17 ) # Property is protected
PF_Public               = FLAG( 18 ) # Property is public
PF_AutoBind             = FLAG( 19 ) # Property is automatically bindable
PF_AutoBindOptional     = FLAG( 20 ) # Failed autobind will not result in runtime script errors

PF_AccessModifiers = PF_Private | PF_Protected | PF_Public

# RTTI Types

RT_Simple       = 0
RT_Enum         = 1
RT_Class        = 2
RT_Array        = 3    # dynamic array
RT_StaticArray  = 4
RT_NativeArray  = 5
RT_Pointer      = 6
RT_Handle       = 7
RT_SoftHandle   = 8
RT_BitField     = 9
RT_Void         = 10
RT_Fundamental  = 11

RT_NAMES = {v: k for k, v in globals().items() if k.startswith('RT_')}

def get_rt_name(rt):
    return RT_NAMES[rt]

# dumped
INTERNAL_OPERATORS = [
    "Add_Int32_Int32",
    "Subtract_Int32_Int32",
    "Multiply_Int32_Int32",
    "Divide_Int32_Int32",
    "Modulo_Int32_Int32",
    "Neg_Int32",
    "BitNot_Int32",
    "And_Int32_Int32",
    "Or_Int32_Int32",
    "Xor_Int32_Int32",
    "Equal_Int32_Int32",
    "NotEqual_Int32_Int32",
    "Less_Int32_Int32",
    "LessEqual_Int32_Int32",
    "Greater_Int32_Int32",
    "GreaterEqual_Int32_Int32",
    "AssignAdd_Int32_Int32",
    "AssignSubtract_Int32_Int32",
    "AssignMultiply_Int32_Int32",
    "AssignDivide_Int32_Int32",
    "AssignAnd_Int32_Int32",
    "AssignOr_Int32_Int32",
    "Add_Uint64_Uint64",
    "Subtract_Uint64_Uint64",
    "Multiply_Uint64_Uint64",
    "Divide_Uint64_Uint64",
    "Modulo_Uint64_Uint64",
    "And_Uint64_Uint64",
    "Or_Uint64_Uint64",
    "Xor_Uint64_Uint64",
    "Equal_Uint64_Uint64",
    "NotEqual_Uint64_Uint64",
    "Less_Uint64_Uint64",
    "LessEqual_Uint64_Uint64",
    "Greater_Uint64_Uint64",
    "GreaterEqual_Uint64_Uint64",
    "AssignAdd_Uint64_Uint64",
    "AssignSubtract_Uint64_Uint64",
    "AssignMultiply_Uint64_Uint64",
    "AssignDivide_Uint64_Uint64",
    "AssignAnd_Uint64_Uint64",
    "AssignOr_Uint64_Uint64",
    "Add_String_String",
    "AssignAdd_String_String",
    "Add_Float_Float",
    "Subtract_Float_Float",
    "Multiply_Float_Float",
    "Divide_Float_Float",
    "Modulo_Float_Float",
    "Neg_Float",
    "Equal_Float_Float",
    "NotEqual_Float_Float",
    "Less_Float_Float",
    "LessEqual_Float_Float",
    "Greater_Float_Float",
    "GreaterEqual_Float_Float",
    "AssignAdd_Float_Float",
    "AssignSubtract_Float_Float",
    "AssignMultiply_Float_Float",
    "AssignDivide_Float_Float",
    "Add_Uint8_Int32",
    "Subtract_Uint8_Int32",
    "Multiply_Uint8_Int32",
    "Divide_Uint8_Int32",
    "Modulo_Uint8_Int32",
    "And_Uint8_Int32",
    "Or_Uint8_Int32",
    "Xor_Uint8_Int32",
    "Equal_Uint8_Uint8",
    "NotEqual_Uint8_Uint8",
    "Less_Uint8_Uint8",
    "LessEqual_Uint8_Uint8",
    "Greater_Uint8_Uint8",
    "GreaterEqual_Uint8_Int32",
    "AssignAdd_Uint8_Int32",
    "AssignSubtract_Uint8_Int32",
    "AssignMultiply_Uint8_Int32",
    "AssignDivide_Uint8_Int32",
    "AssignAnd_Uint8_Int32",
    "AssignOr_Uint8_Int32",
    "LogicAnd_Bool_Bool",
    "LogicOr_Bool_Bool",
    "LogicNot_Bool",
    "Neg_Vector",
    "Add_Vector_Vector",
    "Subtract_Vector_Vector",
    "Multiply_Vector_Vector",
    "Divide_Vector_Vector",
    "AssignAdd_Vector_Vector",
    "AssignSubtract_Vector_Vector",
    "AssignMultiply_Vector_Vector",
    "AssignDivide_Vector_Vector",
    "Add_Vector_Float",
    "Subtract_Vector_Float",
    "Multiply_Vector_Float",
    "Divide_Vector_Float",
    "AssignAdd_Vector_Float",
    "AssignSubtract_Vector_Float",
    "AssignMultiply_Vector_Float",
    "AssignDivide_Vector_Float",
    "Multiply_Float_Vector",
    "Multiply_Matrix_Matrix",
    "AssignMultiply_Matrix_Matrix",
    "Add_EngineTime_EngineTime",
    "Add_EngineTime_Float",
    "AssignAdd_EngineTime_EngineTime",
    "AssignAdd_EngineTime_Float",
    "Subtract_EngineTime_EngineTime",
    "Subtract_EngineTime_Float",
    "AssignSubtract_EngineTime_EngineTime",
    "AssignSubtract_EngineTime_Float",
    "Multiply_EngineTime_Float",
    "AssignMultiply_EngineTime_Float",
    "Divide_EngineTime_Float",
    "AssignDivide_EngineTime_Float",
    "Modulo_EngineTime_Float",
    "Equal_EngineTime_EngineTime",
    "NotEqual_EngineTime_EngineTime",
    "Greater_EngineTime_EngineTime",
    "GreaterEqual_EngineTime_EngineTime",
    "Less_EngineTime_EngineTime",
    "LessEqual_EngineTime_EngineTime",
    "Greater_EngineTime_Float",
    "GreaterEqual_EngineTime_Float",
    "Less_EngineTime_Float",
    "LessEqual_EngineTime_Float",
    "LogicAnd_handle_IScriptable_handle_IScriptable",
    "Add_GameTime_GameTime",
    "Add_GameTime_Int32",
    "Subtract_GameTime_GameTime",
    "Subtract_GameTime_Int32",
    "Multiply_GameTime_Float",
    "Divide_GameTime_Float",
    "Neg_GameTime",
    "Equal_GameTime_GameTime",
    "NotEqual_GameTime_GameTime",
    "Less_GameTime_GameTime",
    "LessEqual_GameTime_GameTime",
    "Greater_GameTime_GameTime",
    "GreaterEqual_GameTime_GameTime",
    "AssignAdd_GameTime_GameTime",
    "AssignSubtract_GameTime_GameTime",
    "AssignAdd_GameTime_Int32",
    "AssignSubtract_GameTime_Int32",
    "AssignMultiply_GameTime_Float",
    "AssignDivide_GameTime_Float",
    "Equal_SItemUniqueId_SItemUniqueId",
    "NotEqual_SItemUniqueId_SItemUniqueId",
    "Add_SAbilityAttributeValue_SAbilityAttributeValue",
    "Subtract_SAbilityAttributeValue_SAbilityAttributeValue",
    "AssignAdd_SAbilityAttributeValue_SAbilityAttributeValue",
    "AssignSubtract_SAbilityAttributeValue_SAbilityAttributeValue",
    "Multiply_SAbilityAttributeValue_Float",
]
assert len(INTERNAL_OPERATORS) == 153

################################################################
# Definition Objects
################################################################

class EnumDef:
    def __init__(self, name="", enumerators=[]):
        self.name = name
        self.enumerators = enumerators

class StructDef:
    def __init__(self, name=""):
        self.name = name
        self.flags = 0
        self.properties = []
        self.default_values = []

class ClassDef:
    def __init__(self, name=""):
        self.name = name
        self.parent_name = ""
        self.machine_name = "" # for state classes
        self.is_state = False
        self.flags = 0
        self.properties = []
        self.functions = []
        self.default_values = []

class FunctionDef:
    def __init__(self, name=""):
        self.name = name
        self.override_class = "" # for annotations
        self.flags = 0
        self.return_type = None
        self.parameters = []
        self.locals = []
        self.bytecode = b""

class PropertyDef:
    def __init__(self, name=""):
        self.name = name
        self.hint = ""
        self.flags = 0
        self.type_name = ""
        self.class_name = ""
        self.binding = ""

# It's a tree
class DefaultValue:
    def __init__(self, name=""):
        self.name = name
        self.value = ""
        self.sub_values = []

class DefaultValueDef:
    def __init__(self, name=""):
        self.name = name
        self.value = None

class ImportType:
    def __init__(self, name, kind=RT_Simple):
        self.name = name
        self.kind = kind

    def __repr__(self):
        return f"imp_type<{get_rt_name(self.kind)}, \"{self.name}\">"

# unused, rework leftover
class ImportProperty:
    def __init__(self, name, type=-1, scope_type=-1):
        self.name = name
        self.type = type
        self.scope_type = scope_type

class ImportFunction:
    def __init__(self, name, scope_type=-1):
        self.name = name
        self.scope_type = scope_type

    def __repr__(self):
        return f"imp_func<{self.scope_type}, \"{self.name}\">"
    
################################################################
# Blob
################################################################

class RSBlob:
    def __init__(self):
        # Metadata
        self.format_version = 0
        self.build_platform = ""
        self.build_version = ""
        self.timestamp = CDateTime()
        self.build_config = ""
        # Names
        self.string_table = []
        # Definitions
        self.enums = []
        self.structs = []
        self.classes = []
        self.global_functions = []
        # Annotations
        self.ext_replace_global_functions = [] # @replaceMethod
        self.ext_replace_class_functions = [] # @replaceMethod
        self.ext_add_functions = [] # @addMethod
        self.ext_wrap_functions = [] # @wrapMethod
        self.ext_add_properties = [] # @addProperty
        # Symbols for linkage (in functions bytecode)
        self.import_type_table = []
        self.import_property_table = [] # unused atm
        self.import_function_table = []

################################################################
# Blob Serializer
################################################################

class RSBlobSerializer:
    def __init__(self):
        self.string_table = []
        self.import_type_table = []
        self.__st = None

    def read(self, st: io.BufferedIOBase):
        self.__st = st
        blob = RSBlob()
        # Header
        blob.format_version = read_u32(st)
        blob.build_platform = read_string(st)
        blob.build_version = read_string(st)
        blob.timestamp = CDateTime.read_from(st)
        blob.build_config = read_string(st)
        # Names (end of file)
        saved_spos = st.tell()
        st.seek(-4, 2)
        st.seek(read_u32(st)) # string table offset
        self.string_table = [read_string(st) for _ in range(read_u32(st))]
        blob.string_table = self.string_table
        st.seek(saved_spos)
        # Definitions
        blob.enums = self.read_array(self.read_enum_def)
        blob.structs = self.read_array(self.read_struct_def)
        blob.classes = self.read_array(self.read_class_def)
        blob.global_functions = self.read_array(self.read_function_def)
        # Annotations
        blob.ext_replace_global_functions = self.read_array(self.read_function_def)
        blob.ext_replace_class_functions = self.read_array(self.read_function_def)
        blob.ext_add_functions = self.read_array(self.read_function_def) 
        blob.ext_wrap_functions = self.read_array(self.read_function_def)
        blob.ext_add_properties = self.read_array(self.read_property_def)
        # Symbols for linkage (in functions bytecode)
        self.import_type_table = self.read_array(self.read_import_type)
        blob.import_type_table = self.import_type_table
        blob.import_property_table = self.read_array(self.read_import_property)
        blob.import_function_table = self.read_array(self.read_import_function)

        return blob

    def read_array(self, element_read_fn):
        len = read_compressed_i32(self.__st)
        return [element_read_fn() for _ in range(len)]

    def read_cname(self):
        index = read_compressed_u32(self.__st)
        return self.string_table[index]

    def read_enum_def(self):
        st = self.__st
        enum = EnumDef(self.read_cname()) 
        for _ in range(read_compressed_u32(st)):
            name = self.read_cname()
            value = read_compressed_i32(st)
            enum.enumerators.append((name, value))
        return enum
    
    def read_struct_def(self):
        st = self.__st
        d = StructDef(self.read_cname()) 
        d.flags = read_compressed_u32(st)
        d.properties = self.read_array(self.read_property_def)
        d.default_values = self.read_array(self.read_default_value_def)
        return d

    def read_class_def(self):
        st = self.__st
        d = ClassDef(self.read_cname())
        d.parent_name = self.read_cname()
        d.machine_name = self.read_cname()
        d.is_state = read_u8(st)
        d.flags = read_compressed_u32(st)
        d.properties = self.read_array(self.read_property_def)
        d.functions = self.read_array(self.read_function_def)
        d.default_values = self.read_array(self.read_default_value_def)
        return d
    
    def read_function_def(self):
        st = self.__st
        d = FunctionDef(self.read_cname())
        d.override_class = self.read_cname()
        d.flags = read_compressed_u32(st)
        if read_u8(st):
            d.return_type = self.read_property_def()
        d.parameters = self.read_array(self.read_property_def)
        d.locals = self.read_array(self.read_property_def)
        d.bytecode = st.read(read_compressed_u32(st))
        return d

    def read_property_def(self):
        st = self.__st
        d = PropertyDef(self.read_cname())
        d.hint = read_string(st) # not indexed
        d.flags = read_compressed_u32(st)
        d.type_name = self.read_cname()
        d.class_name = self.read_cname()
        d.binding = self.read_cname()
        return d
    
    def read_default_value_def(self):
        d = DefaultValueDef(self.read_cname())
        d.value = self.read_default_value()
        return d
    
    def read_default_value(self):
        d = DefaultValue(self.read_cname())
        d.value = read_string(self.__st) # not indexed
        d.sub_values = self.read_array(self.read_default_value)
        return d
    
    def read_import_type(self):
        d = ImportType(self.read_cname())
        d.kind = read_compressed_u32(self.__st)
        return d
    
    def read_import_type_ref(self):
        index = read_compressed_u32(self.__st)
        if index != 0xFFFFFFFF:
            return self.import_type_table[index]
        return None

    def read_import_property(self):
        d = ImportProperty(self.read_cname())
        d.type = self.read_import_type_ref()
        d.scope_type = self.read_import_type_ref()
        return d
    
    def read_import_function(self):
        d = ImportFunction(self.read_cname())
        d.scope_type = self.read_import_type_ref()
        return d

################################################################
# Function Bytecode Opcodes
################################################################

class RSInstructionDesc:
    def __init__(self, mnemo, comment="", operands=()):
        self.mnemo = mnemo
        self.operands = operands
        self.comment = comment

INSTRUCTION_SET = [
    RSInstructionDesc('Nop', "No operation"),
    RSInstructionDesc('Null', "CObject* NULL"),
    RSInstructionDesc('IntOne', "Integer '1'"),
    RSInstructionDesc('IntZero', "Integer '0'"),
    RSInstructionDesc('IntConst', "Int32 constant", (('i32', 'value'),)),
    RSInstructionDesc('ShortConst', "Short ( 16bit int ) constant", (('i16', 'value'),)),
    RSInstructionDesc('FloatConst', "Float constant", (('float', 'value'),)),
    RSInstructionDesc('StringConst', "String constant", (('string', 'value'),)),
    RSInstructionDesc('NameConst', "Name constant", (('name', 'value'),)),
    RSInstructionDesc('ByteConst', "Byte constant", (('u8', 'value'),)),
    RSInstructionDesc('BoolTrue', "True"),
    RSInstructionDesc('BoolFalse', "False"),
    RSInstructionDesc('Breakpoint', "Breakpoint wrapper, generated only in debug code", (('u32', 'source_line'), ('bool', 'is_set'))),
    RSInstructionDesc('Assign', "Assign value"),
    RSInstructionDesc('Target', "Target of a label"),
    RSInstructionDesc('LocalVar', "Access to local variable", (('name', 'name'),)),
    RSInstructionDesc('ParamVar', "Access to function parameter variable", (('name', 'name'),)),
    RSInstructionDesc('ObjectVar', "Access to object variable", (('class_prop', 'property'),)), # may use imp_prop in the future
    RSInstructionDesc('ObjectBindableVar', "Access to bindable object variable", (('class_prop', 'property'),)), # may use imp_prop in the future
    RSInstructionDesc('DefaultVar', "Access to variable from default object", (('class_prop', 'property'),)), # may use imp_prop in the future

    RSInstructionDesc('Switch', "Switch statement", (('imp_type', 'expr_type'), ('i16', 'skip_offset'))),
    RSInstructionDesc('SwitchLabel', "Label in switch statement", (('i16', 'unused'), ('i16', 'expr_skip_offset'))),
    RSInstructionDesc('SwitchDefault', "Default switch statement"),
    RSInstructionDesc('Jump', "Jump to target", (('i16', 'skip_offset'),)),
    RSInstructionDesc('JumpIfFalse', "Jump if condition is false", (('i16', 'skip_offset'),)),

    # Used as check mark in a few places: Bool && Bool, Bool || Bool, PF_FuncSkipParam
    RSInstructionDesc("Skip", "Special marker in some constructs, not an executable instruction", (('i16', 'skip_offset'),)),

    RSInstructionDesc("Conditional", "Conditional expression. Looks broken and unusable.", (('i16', 'select_offset'), ('i16', 'skip_offset'))),
    RSInstructionDesc("Constructor", "Constructor", (('u8', 'num_params'), ('imp_type', 'type'))),
    RSInstructionDesc("FinalFunc", "Call to final function ( static function binding )", (('u16', 'skip_offset'), ('u16', 'source_line'), ('imp_func', 'function'))),
    RSInstructionDesc("VirtualFunc", "Call to virtual function", (('u16', 'skip_offset'), ('u16', 'source_line'), ('name', 'function'))),
    RSInstructionDesc("VirtualParentFunc", "Call to derived parent function ( no state machine )", (('u16', 'skip_offset'), ('u16', 'source_line'), ('name', 'function'))),
    RSInstructionDesc("EntryFunc", "Call to state entry function", (('u16', 'skip_offset'), ('name', 'function'))),
    RSInstructionDesc("ParamEnd", "End of parameters"),
    RSInstructionDesc("Return", "Return from function"),

    RSInstructionDesc("StructMember", "Access to structure member ( slow )", (('class_prop', 'property'),)), # may use imp_prop in the future
    RSInstructionDesc("Context", "Evaluation context change", (('u16', 'skip_offset'),)),
    RSInstructionDesc("TestEqual", "Test if two given shit is default", (('imp_type', 'type'),)),
    RSInstructionDesc("TestNotEqual", "Test if two given shit is default", (('imp_type', 'type'),)),
    RSInstructionDesc("New", "Create object", (('imp_type', 'type'),)),
    RSInstructionDesc("Delete", "Delete object"),
    RSInstructionDesc("This", "Reference to self"),
    RSInstructionDesc("Parent", "State machine context "),
    RSInstructionDesc("SavePoint", "Function state SavePoint", (('u16', 'skip_offset'), ('name', 'name'))),
    RSInstructionDesc("SaveValue", "Value of a function param saved by a SavePoint", (('name', 'name'),)),
    RSInstructionDesc("SavePointEnd", "End-of-savepoint-datablock marker"),

    # Array access opcodes
    RSInstructionDesc("ArrayClear", "Clear the array", (('imp_type', 'type'),)),
    RSInstructionDesc("ArraySize", "Get the size of the array", (('imp_type', 'type'),)),
    RSInstructionDesc("ArrayResize", "Resize array", (('imp_type', 'type'),)),
    RSInstructionDesc("ArrayFindFirst", "Find index of first matching element from the array", (('imp_type', 'type'),)),
    RSInstructionDesc("ArrayFindFirstFast", "Find index of first matching element from the array ( faster version )", (('imp_type', 'type'),)),
    RSInstructionDesc("ArrayFindLast", "Find index of last matching element from the array", (('imp_type', 'type'),)),
    RSInstructionDesc("ArrayFindLastFast", "Find index of last matching element from the array ( faster version )", (('imp_type', 'type'),)),
    RSInstructionDesc("ArrayContains", "Check if array contains a given item", (('imp_type', 'type'),)),
    RSInstructionDesc("ArrayContainsFast", "Check if array contains a given item ( faster version )", (('imp_type', 'type'),)),
    RSInstructionDesc("ArrayPushBack", "Add element to array", (('imp_type', 'type'),)),
    RSInstructionDesc("ArrayPopBack", "Remove last element from array", (('imp_type', 'type'),)),
    RSInstructionDesc("ArrayInsert", "Insert element to array", (('imp_type', 'type'),)),
    RSInstructionDesc("ArrayRemove", "Remove element from array", (('imp_type', 'type'),)),
    RSInstructionDesc("ArrayRemoveFast", "Remove element from array ( faster version )", (('imp_type', 'type'),)),
    RSInstructionDesc("ArrayGrow", "Add space to array", (('imp_type', 'type'),)),
    RSInstructionDesc("ArrayErase", "Erase place in array", (('imp_type', 'type'),)),
    RSInstructionDesc("ArrayEraseFast", "Fast erase from array, without preserving order of elements", (('imp_type', 'type'),)),
    RSInstructionDesc("ArrayLast", "Get the last element from array", (('imp_type', 'type'),)),
    RSInstructionDesc("ArrayElement", "Access to array element", (('imp_type', 'type'),)),

    # Static array access opcodes
    RSInstructionDesc("StaticArraySize", "Get the size of the static array"),
    RSInstructionDesc("StaticArrayFindFirst", "Find index of first matching element from the static array"),
    RSInstructionDesc("StaticArrayFindFirstFast", "Find index of first matching element from the static array ( faster version )"),
    RSInstructionDesc("StaticArrayFindLast", "Find index of last matching element from the static array"),
    RSInstructionDesc("StaticArrayFindLastFast", "Find index of last matching element from the static array ( faster version )"),
    RSInstructionDesc("StaticArrayContains", "Check if static array contains a given item"),
    RSInstructionDesc("StaticArrayContainsFast", "Check if static array contains a given item ( faster version )"),
    RSInstructionDesc("StaticArrayLast", "Get the last element from static array"),
    RSInstructionDesc("StaticArrayElement", "Access to array static element"),

    # Casting
    RSInstructionDesc("BoolToByte"),
    RSInstructionDesc("BoolToInt"),
    RSInstructionDesc("BoolToFloat"),
    RSInstructionDesc("BoolToString"),
    RSInstructionDesc("ByteToBool"),
    RSInstructionDesc("ByteToInt"),
    RSInstructionDesc("ByteToFloat"),
    RSInstructionDesc("ByteToString"),
    RSInstructionDesc("IntToBool"),
    RSInstructionDesc("IntToByte"),
    RSInstructionDesc("IntToFloat"),
    RSInstructionDesc("IntToString"),
    RSInstructionDesc("IntToEnum", operands=(('imp_type', 'type'),)),
    RSInstructionDesc("FloatToBool"),
    RSInstructionDesc("FloatToByte"),
    RSInstructionDesc("FloatToInt"),
    RSInstructionDesc("FloatToString"),
    RSInstructionDesc("NameToBool"),
    RSInstructionDesc("NameToString"),
    RSInstructionDesc("StringToBool"),
    RSInstructionDesc("StringToByte"),
    RSInstructionDesc("StringToInt"),
    RSInstructionDesc("StringToFloat"),
    RSInstructionDesc("ObjectToBool"),
    RSInstructionDesc("ObjectToString"),
    RSInstructionDesc("EnumToString", operands=(('imp_type', 'type'),)),
    RSInstructionDesc("EnumToInt", operands=(('imp_type', 'type'),)),
    RSInstructionDesc("DynamicCast", operands=(('imp_type', 'type'),)),

    # Globals
    RSInstructionDesc("GetGame"),
    RSInstructionDesc("GetPlayer"),
    RSInstructionDesc("GetCamera"),
    RSInstructionDesc("GetHud"),
    RSInstructionDesc("GetSound"),
    RSInstructionDesc("GetDebug"),
    RSInstructionDesc("GetTimer"),
    RSInstructionDesc("GetInput"),
    RSInstructionDesc("GetTelemetry"),
]

################################################################
# Disassembler
################################################################

class RSInstruction:
    def __init__(self, offset, size, opcode, mnemo, operands):
        self.offset = offset
        self.size = size
        self.opcode = opcode
        self.mnemo = mnemo
        self.operands = operands

    def __str__(self):
        operands_str = " ".join(f"{k}:{v}" for k, v in self.operands.items())
        return f"{self.offset:04d}: {self.mnemo} {operands_str}"

class RSDisassembler:
    def __init__(self, blob: RSBlob):
        self.__blob = blob

    def read_string(self, st: io.BufferedIOBase):
        slen = read_compressed_u32(st)
        return st.read(slen).decode()
        
    def read_imp_type(self, st: io.BufferedIOBase):
        index = read_compressed_u32(st)
        return self.__blob.import_type_table[index]

    def read_imp_func(self, st: io.BufferedIOBase):
        index = read_compressed_u32(st)
        return self.__blob.import_function_table[index]

    def disas_iter(self, bytecode):
        st = io.BytesIO(bytecode)
        runtime_size = read_u32(st) # size when operands are linked to runtime (executable form)
        offset = 0
        while b := st.read(1):
            op = int(b[0])
            operands = dict()
            operands_runtime_size = 0
            instr_desc = INSTRUCTION_SET[op]

            if op > 100:
                print(op)

            for opr_type, opr_name in instr_desc.operands:
                match opr_type:
                    case 'bool':
                        operands[opr_name] = bool(read_u8(st))
                        operands_runtime_size += 1
                    case 'u8':
                        operands[opr_name] = read_u8(st)
                        operands_runtime_size += 1
                    case 'u16':
                        operands[opr_name] = read_u16(st)
                        operands_runtime_size += 2
                    case 'i16':
                        operands[opr_name] = read_i16(st)
                        operands_runtime_size += 2
                    case 'u32':
                        operands[opr_name] = read_compressed_u32(st)
                        operands_runtime_size += 4
                    case 'i32':
                        operands[opr_name] = read_compressed_i32(st)
                        operands_runtime_size += 4
                    case 'float':
                        operands[opr_name] = read_float(st)
                        operands_runtime_size += 4
                    case 'name':
                        index = read_compressed_u32(st)
                        operands[opr_name] = self.__blob.string_table[index]
                        operands_runtime_size += 4
                    case 'string':
                        slen = read_compressed_u32(st)
                        operands[opr_name] = st.read(slen).decode()
                        operands_runtime_size += 4 + slen # length is not compressed in executable form
                    case 'imp_func':
                        index = read_compressed_i32(st)
                        if index > 0:
                            operands[opr_name] = self.__blob.import_function_table[index - 1]
                        elif index == -1:
                            # calling itself
                            operands[opr_name] = 'thisMethod'
                        elif index < -1:
                            operator_index = -( index + 2 )
                            operands[opr_name] = INTERNAL_OPERATORS[operator_index]
                        else:
                            raise ValueError("function index cannot be 0")
                        operands_runtime_size += 8 # sizeof(CFunction*)
                    case 'imp_type':
                        index = read_compressed_u32(st)
                        operands[opr_name] = self.__blob.import_type_table[index]
                        operands_runtime_size += 8 # sizeof(IRTTIType*)
                    case 'class_prop':
                        index = read_compressed_u32(st)
                        prop_name = self.__blob.string_table[index]
                        index = read_compressed_u32(st)
                        class_type = self.__blob.import_type_table[index]
                        operands[opr_name] = (prop_name, class_type)
                        operands_runtime_size += 8 # sizeof(CProperty*)
                    case _:
                        print("operand not implemented {opr_type}")
                        raise NotImplementedError(f"Operand {opr_type}")
            instr_runtime_size = 1 + operands_runtime_size
            #print(op, instr_runtime_size)
            yield RSInstruction(offset, instr_runtime_size, op, instr_desc.mnemo, operands)
            offset += instr_runtime_size
        # compiler reserves more than necessary, runtime bytecode has padding.
        assert runtime_size >= offset

with open('precompiled.rsblob', 'rb') as f:
    blob = RSBlobSerializer().read(f)
    dis = RSDisassembler(blob)
    for cls in blob.classes:
        for f in cls.functions:
            # print(f"==== {cls.name}::{f.name} ====")
            for i in dis.disas_iter(f.bytecode):
                i
    print("All functions disassembled successfully")

