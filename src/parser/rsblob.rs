use std::fs::read_to_string;

use nom::{AsBytes, Parser};

use crate::parser::prelude::*;

pub struct RsBlob {
    pub content: String,

    pub format_version: u32,
    pub build_platform: String,
    pub build_version: String,
    pub timestamp: CDateTime,
    pub build_config: String,
    // Names
    pub string_table: Vec<String>,
    // Definitions
    pub enums: Vec<EnumDefinition>,
    pub structs: Vec<StructDefinition>,
    pub classes: Vec<ClassDefinition>,
    pub global_functions: Vec<FunctionDefinition>,
    // Annotations
    pub ext_replace_global_functions: Vec<FunctionDefinition>, // @replaceMethod,
    pub ext_replace_class_functions: Vec<FunctionDefinition>,  // @replaceMethod,
    pub ext_add_functions: Vec<FunctionDefinition>,            // @addMethod,
    pub ext_wrap_functions: Vec<FunctionDefinition>,           // @wrapMethod,
    pub ext_add_properties: Vec<PropertyDefinition>,           // @addProperty,
    // Symbols for linkage (in functions bytecode)
    pub import_type_table: Vec<ImportType>,
    pub import_property_table: Vec<ImportProperty>, // unused atm,
    pub import_function_table: Vec<ImportFunction>,
}

impl RsBlob {
    pub fn from_file(path: &str) -> Self {
        let data: Vec<u8> = std::fs::read(path).expect("failed to read .rsblob bytes");
        let i = data.as_bytes();

        Self::parse(&i).map_err(|_| ()).expect("Failed to parse").1
    }

    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        let whole = i;

        // header:
        let (i, format_version) = parse_u32(i)?;
        let (i, build_platform) = parse_string(i)?;
        let (i, build_version) = parse_string(i)?;
        let (i, timestamp) = CDateTime::parse(i)?;
        let (i, build_config) = parse_string(i)?;
        let (_, string_table) = Self::parse_string_table(whole, i)?;

        // definitions:
        let (i, enums) = ast::EnumDefinition::parse_array(i)?;
        let (i, structs) = ast::StructDefinition::parse_array(i)?;
        let (i, classes) = ast::ClassDefinition::parse_array(i)?;
        let (i, global_functions) = ast::FunctionDefinition::parse_array(i)?;

        // annotations:
        let (i, ext_replace_global_functions) = ast::FunctionDefinition::parse_array(i)?;
        let (i, ext_replace_class_functions) = ast::FunctionDefinition::parse_array(i)?;
        let (i, ext_add_functions) = ast::FunctionDefinition::parse_array(i)?;
        let (i, ext_wrap_functions) = ast::FunctionDefinition::parse_array(i)?;
        let (i, ext_add_properties) = ast::PropertyDefinition::parse_array(i)?;

        // symbols for linkage
        let (i, import_type_table) = ImportType::parse_array(i)?;
        let (i, import_property_table) = ImportProperty::parse_array(i)?;
        let (i, import_function_table) = ImportFunction::parse_array(i)?;

        Ok((
            i,
            Self {
                content: String::new(),
                format_version,
                build_platform,
                build_version,
                timestamp,
                build_config,
                string_table,
                enums,
                structs,
                classes,
                global_functions,
                ext_replace_global_functions,
                ext_replace_class_functions,
                ext_add_functions,
                ext_wrap_functions,
                ext_add_properties,
                import_type_table,
                import_property_table,
                import_function_table,
            },
        ))
    }

    fn parse_string_table<'a>(whole: &'a [u8], i: &'a [u8]) -> IResult<&'a [u8], Vec<String>> {
        let string_table_offset = {
            let len = i.len();
            let end = &i[len - 4..len];

            let (end, string_table_offset) = parse_u32(end)?;

            string_table_offset as usize
        };

        let string_table_slice = &whole[string_table_offset..];
        let (mut sts, string_table_size) = parse_u32(string_table_slice)?;
        let mut string_table = Vec::new();

        for _ in 0..string_table_size {
            let (new_sts, string) = parse_string(sts)?;

            sts = new_sts;
            string_table.push(string);
        }

        Ok((sts, string_table))
    }
}
