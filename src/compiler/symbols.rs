use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::SourceFile;
use super::ast::*;
use super::rtti::Rtti;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Void,
    Bool,
    Int,
    Float,
    Byte,
    String,
    Name,
    Null,
    Primitive(String),
    Enum(String),
    Struct(String),
    Class(String),
    Array {
        element: Box<Type>,
        layout: ArrayLayout,
    },
    Map {
        key: Box<Type>,
        value: Box<Type>,
    },
}

pub fn map_type_name(key_name: &str, value_name: &str) -> String {
    format!("map:2,0,{key_name},{value_name}")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArrayLayout {
    pub memory_class: u32,
    pub memory_pool: u32,
}

impl Default for ArrayLayout {
    fn default() -> Self {
        Self {
            memory_class: 2,
            memory_pool: 0,
        }
    }
}

impl ArrayLayout {
    pub fn type_name(&self, element_name: &str) -> String {
        format!(
            "array:{},{},{element_name}",
            self.memory_class, self.memory_pool
        )
    }

    fn parse(rtti_name: &str) -> Option<(Self, &str)> {
        let mut parts = rtti_name.strip_prefix("array:")?.splitn(3, ',');
        let memory_class = parts.next()?.parse().ok()?;
        let memory_pool = parts.next()?.parse().ok()?;
        let element_name = parts.next()?;

        Some((
            Self {
                memory_class,
                memory_pool,
            },
            element_name,
        ))
    }
}

impl Type {
    pub fn array(element: Type) -> Self {
        Type::Array {
            element: Box::new(element),
            layout: ArrayLayout::default(),
        }
    }

    pub fn map(key: Type, value: Type) -> Self {
        Type::Map {
            key: Box::new(key),
            value: Box::new(value),
        }
    }

    pub fn rtti_name(&self) -> String {
        match self {
            Type::Void => "void".to_owned(),
            Type::Bool => "Bool".to_owned(),
            Type::Int => "Int32".to_owned(),
            Type::Float => "Float".to_owned(),
            Type::Byte => "Uint8".to_owned(),
            Type::String => "String".to_owned(),
            Type::Name => "CName".to_owned(),
            Type::Null => "NULL".to_owned(),
            Type::Primitive(name) | Type::Enum(name) | Type::Struct(name) | Type::Class(name) => {
                name.clone()
            }
            Type::Array { element, layout } => layout.type_name(&element.rtti_name()),
            Type::Map { key, value } => map_type_name(&key.rtti_name(), &value.rtti_name()),
        }
    }

    pub fn is_class(&self) -> bool {
        matches!(self, Type::Class(_) | Type::Null)
    }
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub ty: Type,
    pub owner: String,
}

#[derive(Debug, Clone)]
pub struct ParamInfo {
    pub name: String,
    pub ty: Type,
    pub is_optional: bool,
    pub is_out: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub owner: Option<String>,
    pub params: Vec<ParamInfo>,
    pub return_type: Type,
    pub specifiers: FunctionSpecifiers,
}

impl FunctionInfo {
    pub fn is_final_call(&self) -> bool {
        self.owner.is_none() || self.specifiers.is_final
    }
}

#[derive(Debug, Clone, Default)]
pub struct ClassInfo {
    pub name: String,
    pub blob_name: String,
    pub parent: Option<String>,
    pub is_struct: bool,
    pub state_machine: Option<String>,
    pub fields: HashMap<String, FieldInfo>,
    pub field_order: Vec<String>,
    pub methods: HashMap<String, Rc<FunctionInfo>>,
}

#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub name: String,
    pub values: Vec<(String, i32)>,
    pub is_script: bool,
}

impl EnumInfo {
    pub fn value_of(&self, member: &str) -> Option<i32> {
        self.values.iter().find(|(n, _)| n == member).map(|(_, v)| *v)
    }
}

#[derive(Default)]
pub struct Symbols {
    pub classes: HashMap<String, ClassInfo>,
    pub enums: HashMap<String, EnumInfo>,
    pub enum_members: HashMap<String, String>,
    pub functions: HashMap<String, Rc<FunctionInfo>>,
}

pub fn state_class_key(machine: &str, state: &str) -> String {
    format!("{machine}State{state}")
}

impl Symbols {
    pub fn add_enums(&mut self, files: &[SourceFile], is_script: bool) {
        for file in files {
            for item in &file.items {
                let Item::Enum(decl) = item else { continue };
                if !is_script && self.enums.contains_key(&decl.name) {
                    continue;
                }

                let mut next = 0i64;
                let values = decl
                    .members
                    .iter()
                    .map(|(name, value)| {
                        let value = value.unwrap_or(next);
                        next = value + 1;
                        (name.clone(), value as i32)
                    })
                    .collect::<Vec<_>>();

                for (member, _) in &values {
                    self.enum_members.insert(member.clone(), decl.name.clone());
                }

                self.enums.insert(
                    decl.name.clone(),
                    EnumInfo {
                        name: decl.name.clone(),
                        values,
                        is_script,
                    },
                );
            }
        }
    }

    pub fn add_type_names(&mut self, files: &[SourceFile]) {
        for file in files {
            for item in &file.items {
                let Item::Class(decl) = item else { continue };
                let key = class_key(decl);
                if self.classes.contains_key(&key) {
                    continue;
                }

                let parent = match &decl.state_machine_of {
                    Some(_) => Some(
                        decl.parent
                            .as_ref()
                            .map(|p| state_class_key(decl.state_machine_of.as_ref().unwrap(), p))
                            .unwrap_or_else(|| "CScriptableState".to_owned()),
                    ),
                    None if decl.parent.is_none()
                        && !decl.is_struct
                        && !matches!(decl.name.as_str(), "IScriptable" | "ISerializable") =>
                    {
                        Some("IScriptable".to_owned())
                    }
                    None => decl.parent.clone(),
                };

                self.classes.insert(
                    key.clone(),
                    ClassInfo {
                        name: key,
                        blob_name: decl.name.clone(),
                        parent,
                        is_struct: decl.is_struct,
                        state_machine: decl.state_machine_of.clone(),
                        ..Default::default()
                    },
                );
            }
        }
    }

    pub fn add_members(&mut self, files: &[SourceFile]) -> Result<(), String> {
        for file in files {
            for item in &file.items {
                match item {
                    Item::Class(decl) => {
                        let key = class_key(decl);
                        for field in &decl.fields {
                            let ty = self.resolve(&field.ty)?;
                            for name in &field.names {
                                self.add_field(&key, name, ty.clone());
                            }
                        }
                        for method in &decl.methods {
                            let info = self.function_info(method, Some(key.clone()))?;
                            let class = self.classes.get_mut(&key).unwrap();
                            class.methods.entry(method.name.clone()).or_insert(Rc::new(info));
                        }
                    }
                    Item::Function(decl) => {
                        let target = decl.annotation.as_ref().and_then(|a| a.class.clone());
                        let kind = decl.annotation.as_ref().map(|a| a.kind);

                        match (kind, target) {
                            (Some(AnnotationKind::AddMethod), Some(class)) => {
                                let info = self.function_info(decl, Some(class.clone()))?;
                                self.classes
                                    .get_mut(&class)
                                    .ok_or_else(|| format!("@addMethod: unknown class {class}"))?
                                    .methods
                                    .insert(decl.name.clone(), Rc::new(info));
                            }
                            (None, _) => {
                                let info = self.function_info(decl, None)?;
                                self.functions
                                    .entry(decl.name.clone())
                                    .or_insert(Rc::new(info));
                            }
                            _ => {}
                        }
                    }
                    Item::Field(field) => {
                        let Some(class) = field.annotation.as_ref().and_then(|a| a.class.clone())
                        else {
                            continue;
                        };
                        let ty = self.resolve(&field.ty)?;
                        for name in &field.names {
                            self.add_field(&class, name, ty.clone());
                        }
                    }
                    Item::Enum(_) => {}
                }
            }
        }

        Ok(())
    }

    pub fn add_rtti_enums(&mut self, rtti: &Rtti) {
        for native in &rtti.enums {
            if self.enums.contains_key(&native.name) {
                continue;
            }

            for (member, _) in &native.options {
                self.enum_members
                    .entry(member.clone())
                    .or_insert_with(|| native.name.clone());
            }

            self.enums.insert(
                native.name.clone(),
                EnumInfo {
                    name: native.name.clone(),
                    values: native.options.clone(),
                    is_script: false,
                },
            );
        }
    }

    pub fn add_rtti_classes(&mut self, rtti: &Rtti) {
        for native in &rtti.classes {
            if self.classes.contains_key(&native.name) {
                continue;
            }

            self.classes.insert(
                native.name.clone(),
                ClassInfo {
                    name: native.name.clone(),
                    blob_name: native.name.clone(),
                    parent: native.base.clone(),
                    is_struct: native.is_struct(),
                    ..Default::default()
                },
            );
        }
    }

    pub fn add_rtti_properties(&mut self, rtti: &Rtti) {
        let declared: HashMap<&str, (Option<&str>, HashSet<&str>)> = rtti
            .classes
            .iter()
            .map(|c| {
                (
                    c.name.as_str(),
                    (
                        c.base.as_deref(),
                        c.properties.iter().map(|(n, _)| n.as_str()).collect(),
                    ),
                )
            })
            .collect();

        let inherited = |class: &str, property: &str| {
            let mut current = declared.get(class).and_then(|(base, _)| *base);
            while let Some(name) = current {
                let Some((base, properties)) = declared.get(name) else {
                    return false;
                };
                if properties.contains(property) {
                    return true;
                }
                current = *base;
            }
            false
        };

        for native in &rtti.classes {
            for (name, type_name) in &native.properties {
                if inherited(&native.name, name) {
                    continue;
                }
                let ty = self.resolve_rtti(type_name);
                self.set_native_field(&native.name, name, ty);
            }
        }
    }

    fn set_native_field(&mut self, class: &str, name: &str, ty: Type) {
        let declared_field = self
            .classes
            .get_mut(class)
            .and_then(|info| info.fields.get_mut(name));

        match declared_field {
            Some(field) => field.ty = ty,
            None => self.add_field(class, name, ty),
        }
    }

    fn resolve_rtti(&self, name: &str) -> Type {
        if let Some((layout, element_name)) = ArrayLayout::parse(name) {
            return Type::Array {
                element: Box::new(self.resolve_rtti(element_name)),
                layout,
            };
        }
        if let Some(class) = name.strip_prefix("handle:").or_else(|| name.strip_prefix("ptr:")) {
            return Type::Class(class.to_owned());
        }

        let simple = TypeRef {
            name: name.to_owned(),
            args: Vec::new(),
        };
        match self.resolve(&simple) {
            Ok(Type::Class(class)) if !self.classes.contains_key(&class) => Type::Primitive(class),
            Ok(ty) => ty,
            Err(_) => Type::Primitive(name.to_owned()),
        }
    }

    fn add_field(&mut self, class: &str, name: &str, ty: Type) {
        let Some(info) = self.classes.get_mut(class) else {
            return;
        };
        if info.fields.contains_key(name) {
            return;
        }
        info.field_order.push(name.to_owned());
        info.fields.insert(
            name.to_owned(),
            FieldInfo {
                name: name.to_owned(),
                ty,
                owner: class.to_owned(),
            },
        );
    }

    pub fn function_info(
        &self,
        decl: &FunctionDecl,
        owner: Option<String>,
    ) -> Result<FunctionInfo, String> {
        let params = decl
            .params
            .iter()
            .map(|p| {
                Ok(ParamInfo {
                    name: p.name.clone(),
                    ty: self.resolve(&p.ty)?,
                    is_optional: p.is_optional,
                    is_out: p.is_out,
                })
            })
            .collect::<Result<Vec<_>, String>>()
            .map_err(|e| format!("{}: {e}", decl.name))?;

        let return_type = match &decl.return_type {
            Some(ty) => self.resolve(ty).map_err(|e| format!("{}: {e}", decl.name))?,
            None if decl.specifiers.is_event => Type::Bool,
            None => Type::Void,
        };

        Ok(FunctionInfo {
            name: decl.name.clone(),
            owner,
            params,
            return_type,
            specifiers: decl.specifiers.clone(),
        })
    }

    pub fn resolve(&self, ty: &TypeRef) -> Result<Type, String> {
        if ty.name == "array" {
            let inner = ty.args.first().ok_or("array without element type")?;
            return Ok(Type::array(self.resolve(inner)?));
        }
        if ty.name == "map" {
            let [key, value] = ty.args.as_slice() else {
                return Err("map requires key and value types".to_owned());
            };
            return Ok(Type::map(self.resolve(key)?, self.resolve(value)?));
        }

        Ok(match ty.name.as_str() {
            "void" => Type::Void,
            "bool" | "Bool" => Type::Bool,
            "int" | "Int32" => Type::Int,
            "float" | "Float" => Type::Float,
            "byte" | "Uint8" => Type::Byte,
            "string" | "String" => Type::String,
            "name" | "CName" => Type::Name,
            "Uint64" | "Int8" | "Int16" | "Uint16" | "Uint32" | "Int64" | "Double" => {
                Type::Primitive(ty.name.clone())
            }
            name if self.enums.contains_key(name) => Type::Enum(name.to_owned()),
            name => match self.classes.get(name) {
                Some(class) if class.is_struct => Type::Struct(name.to_owned()),
                Some(_) => Type::Class(name.to_owned()),
                None if ty.args.is_empty() => Type::Class(name.to_owned()),
                None => Type::Primitive(name.to_owned()),
            },
        })
    }

    pub fn find_field(&self, class: &str, name: &str) -> Option<&FieldInfo> {
        let mut current = Some(class);
        let mut depth = 0;
        while let Some(class_name) = current {
            let class = self.classes.get(class_name)?;
            if let Some(field) = class.fields.get(name) {
                return Some(field);
            }
            current = class.parent.as_deref();
            depth += 1;
            if depth > 64 {
                return None;
            }
        }
        None
    }

    pub fn find_method(&self, class: &str, name: &str) -> Option<Rc<FunctionInfo>> {
        let mut current = Some(class);
        let mut depth = 0;
        while let Some(class_name) = current {
            let class = self.classes.get(class_name)?;
            if let Some(method) = class.methods.get(name) {
                return Some(method.clone());
            }
            current = class.parent.as_deref();
            depth += 1;
            if depth > 64 {
                return None;
            }
        }
        None
    }

    pub fn is_subclass(&self, class: &str, ancestor: &str) -> bool {
        let mut current = Some(class);
        let mut depth = 0;
        while let Some(name) = current {
            if name == ancestor {
                return true;
            }
            current = self.classes.get(name).and_then(|c| c.parent.as_deref());
            depth += 1;
            if depth > 64 {
                return false;
            }
        }
        false
    }

    pub fn blob_name(&self, class: &str) -> String {
        self.classes
            .get(class)
            .map(|c| c.blob_name.clone())
            .unwrap_or_else(|| class.to_owned())
    }
}

pub fn class_key(decl: &ClassDecl) -> String {
    match &decl.state_machine_of {
        Some(machine) => state_class_key(machine, &decl.name),
        None => decl.name.clone(),
    }
}
