use std::collections::HashMap;

use super::SourceFile;
use super::ast::*;
use super::codegen::{CompiledBody, Wrap, compile_body};
use super::symbols::{Symbols, Type, class_key};
use super::tables::{Tables, definition_type_name};
use crate::parser::RsBlob;
use crate::parser::WithWriting;
use crate::parser::ast::{
    ByteCodeRef, CDateTime, ClassDefinition, DefaultValue, DefaultValueDefinition, EnumDefinition,
    Enumerator, FunctionDefinition, ParsedFunctionBytecode, PropertyDefinition, StructDefinition,
};

type R<T> = Result<T, String>;

const IMPLICIT_ENUM_VALUE: i32 = 0x7FFF_FFFF;

struct Assembler<'a> {
    symbols: &'a Symbols,
    tables: Tables,
    errors: Vec<String>,
}

struct FunctionPlan<'d> {
    decl: &'d FunctionDecl,
    label: String,
    override_class: String,
    flags: u32,
    this_class: Option<String>,
    wrap: Wrap,
    compiled: Option<CompiledFunction>,
}

struct CompiledFunction {
    locals: Vec<(String, Type)>,
    bytecode: Vec<u8>,
}

struct ClassPlan<'d> {
    decl: &'d ClassDecl,
    methods: Vec<FunctionPlan<'d>>,
}

#[derive(Default)]
struct FreeFunctionPlans<'d> {
    global: Vec<FunctionPlan<'d>>,
    replace_global: Vec<FunctionPlan<'d>>,
    replace_class: Vec<FunctionPlan<'d>>,
    add: Vec<FunctionPlan<'d>>,
    wrap: Vec<FunctionPlan<'d>>,
}

pub fn assemble(symbols: &Symbols, mod_files: &[SourceFile]) -> R<RsBlob> {
    let mut assembler = Assembler {
        symbols,
        tables: Tables::default(),
        errors: Vec::new(),
    };

    let items: Vec<&Item> = mod_files.iter().flat_map(|f| f.items.iter()).collect();

    let enums: Vec<&EnumDecl> = items
        .iter()
        .filter_map(|i| match i {
            Item::Enum(decl) => Some(decl),
            _ => None,
        })
        .collect();

    let structs: Vec<&ClassDecl> = items
        .iter()
        .filter_map(|i| match i {
            Item::Class(c) if c.is_struct => Some(c),
            _ => None,
        })
        .collect();

    let classes: Vec<&ClassDecl> = items
        .iter()
        .filter_map(|i| match i {
            Item::Class(c) if !c.is_struct => Some(c),
            _ => None,
        })
        .collect();
    let (states, plain): (Vec<&ClassDecl>, Vec<&ClassDecl>) =
        classes.iter().partition(|c| c.state_machine_of.is_some());

    let mut class_plans: Vec<ClassPlan> = class_order(&plain)
        .into_iter()
        .chain(class_order(&states))
        .map(|decl| ClassPlan {
            decl,
            methods: assembler.method_plans(decl),
        })
        .collect();

    let mut free = FreeFunctionPlans::default();
    for item in &items {
        if let Item::Function(decl) = item {
            if let Err(e) = assembler.plan_free_function(decl, &mut free) {
                assembler.errors.push(format!("function {}: {e}", decl.name));
            }
        }
    }

    let fields: Vec<&FieldDecl> = items
        .iter()
        .filter_map(|i| match i {
            Item::Field(field) => Some(field),
            _ => None,
        })
        .collect();

    for plan in free
        .global
        .iter_mut()
        .chain(class_plans.iter_mut().flat_map(|c| c.methods.iter_mut()))
        .chain(&mut free.replace_global)
        .chain(&mut free.replace_class)
        .chain(&mut free.add)
        .chain(&mut free.wrap)
    {
        assembler.compile(plan);
    }

    let mut blob = RsBlob {
        content: String::new(),
        format_version: 1,
        build_platform: "x64".to_owned(),
        build_version: "1048625".to_owned(),
        timestamp: CDateTime::now(),
        build_config: "Release".to_owned(),
        string_table: Vec::new(),
        enums: Vec::new(),
        structs: Vec::new(),
        classes: Vec::new(),
        global_functions: Vec::new(),
        ext_replace_global_functions: Vec::new(),
        ext_replace_class_functions: Vec::new(),
        ext_add_functions: Vec::new(),
        ext_wrap_functions: Vec::new(),
        ext_add_properties: Vec::new(),
        import_type_table: Vec::new(),
        import_property_table: Vec::new(),
        import_function_table: Vec::new(),
    };

    for decl in enums {
        blob.enums.push(assembler.enum_definition(decl));
    }

    for decl in structs {
        blob.structs.push(assembler.struct_definition(decl)?);
    }

    for plan in class_plans {
        blob.classes.push(
            assembler
                .class_definition(plan)
                .map_err(|e| format!("class {e}"))?,
        );
    }

    blob.global_functions = assembler.function_definitions(free.global);
    blob.ext_replace_global_functions = assembler.function_definitions(free.replace_global);
    blob.ext_replace_class_functions = assembler.function_definitions(free.replace_class);
    blob.ext_add_functions = assembler.function_definitions(free.add);
    blob.ext_wrap_functions = assembler.function_definitions(free.wrap);

    for field in fields {
        let class = field
            .annotation
            .as_ref()
            .and_then(|a| a.class.clone())
            .ok_or("top-level var without @addField")?;
        for name in &field.names {
            let property = assembler.class_property(field, name, &class)?;
            blob.ext_add_properties.push(property);
        }
    }

    if !assembler.errors.is_empty() {
        return Err(format!(
            "{} errors:\n{}",
            assembler.errors.len(),
            assembler.errors.join("\n")
        ));
    }

    let tables = assembler.tables;
    blob.string_table = tables.strings;
    blob.import_type_table = tables.import_types;
    blob.import_function_table = tables.import_functions;

    Ok(blob)
}

fn class_order<'c>(classes: &[&'c ClassDecl]) -> Vec<&'c ClassDecl> {
    let keys: HashMap<String, usize> = classes
        .iter()
        .enumerate()
        .map(|(i, c)| (class_key(c), i))
        .collect();

    let parent_of = |class: &ClassDecl| -> Option<usize> {
        let parent = class.parent.as_ref()?;
        let key = match &class.state_machine_of {
            Some(machine) => super::symbols::state_class_key(machine, parent),
            None => parent.clone(),
        };
        keys.get(&key).copied()
    };

    let mut children: Vec<Vec<usize>> = vec![Vec::new(); classes.len()];
    let mut roots = Vec::new();
    for (index, class) in classes.iter().enumerate() {
        match parent_of(class) {
            Some(parent) if parent != index => children[parent].push(index),
            _ => roots.push(index),
        }
    }

    let mut order = Vec::new();
    let mut stack: Vec<usize> = roots.into_iter().rev().collect();
    while let Some(index) = stack.pop() {
        order.push(classes[index]);
        stack.extend(children[index].iter().rev());
    }
    order
}

fn visibility_flags(visibility: Visibility) -> u32 {
    match visibility {
        Visibility::Private => 0x4000,
        Visibility::Protected => 0x8000,
        _ => 0x10000,
    }
}

fn kind_flags(specifiers: &FunctionSpecifiers) -> u32 {
    let mut flags = 0;
    if specifiers.is_event {
        flags |= 0x20;
    }
    if specifiers.is_timer {
        flags |= 0x400;
    }
    if specifiers.is_entry {
        flags |= 0x80;
    }
    flags
}

impl<'d> FunctionPlan<'d> {
    fn new(decl: &'d FunctionDecl, label: String, flags: u32) -> Self {
        Self {
            decl,
            label,
            override_class: String::new(),
            flags,
            this_class: None,
            wrap: Wrap::None,
            compiled: None,
        }
    }

    fn on_class(mut self, class: &str, override_class: &str) -> Self {
        self.this_class = Some(class.to_owned());
        self.override_class = override_class.to_owned();
        self
    }

    fn wrapping(mut self, wrap: Wrap) -> Self {
        self.wrap = wrap;
        self
    }
}

impl Assembler<'_> {
    fn method_plans<'d>(&self, decl: &'d ClassDecl) -> Vec<FunctionPlan<'d>> {
        let key = class_key(decl);
        decl.methods
            .iter()
            .map(|method| {
                let flags = visibility_flags(method.specifiers.visibility)
                    | kind_flags(&method.specifiers)
                    | if method.body.is_none() { 0x200 } else { 0 };
                FunctionPlan::new(method, format!("{}::{}", decl.name, method.name), flags)
                    .on_class(&key, "")
            })
            .collect()
    }

    fn plan_free_function<'d>(
        &self,
        decl: &'d FunctionDecl,
        plans: &mut FreeFunctionPlans<'d>,
    ) -> R<()> {
        let label = format!("function {}", decl.name);
        let Some(annotation) = decl.annotation.as_ref() else {
            let flags = 0x2 | if decl.specifiers.is_exec { 0x100 } else { 0 };
            plans.global.push(FunctionPlan::new(decl, label, flags));
            return Ok(());
        };

        match (annotation.kind, annotation.class.as_deref()) {
            (AnnotationKind::AddMethod, Some(class)) => {
                let flags = 0x80000
                    | visibility_flags(decl.specifiers.visibility)
                    | kind_flags(&decl.specifiers);
                plans
                    .add
                    .push(FunctionPlan::new(decl, label, flags).on_class(class, class));
            }
            (AnnotationKind::WrapMethod, Some(class)) => {
                let target = self
                    .symbols
                    .find_method(class, &decl.name)
                    .ok_or_else(|| format!("@wrapMethod target {class}.{} not found", decl.name))?;
                let flags = 0x100000 | 0x10000 | kind_flags(&target.specifiers);
                plans.wrap.push(
                    FunctionPlan::new(decl, label, flags)
                        .on_class(class, class)
                        .wrapping(Wrap::Class(target)),
                );
            }
            (AnnotationKind::WrapMethod, None) => {
                let target = self
                    .symbols
                    .functions
                    .get(&decl.name)
                    .cloned()
                    .ok_or_else(|| format!("@wrapMethod target {} not found", decl.name))?;
                plans.wrap.push(
                    FunctionPlan::new(decl, label, 0x100000 | 0x2).wrapping(Wrap::Global(target)),
                );
            }
            (AnnotationKind::ReplaceMethod, Some(class)) => {
                plans.replace_class.push(
                    FunctionPlan::new(decl, label, 0x40000 | 0x10000).on_class(class, class),
                );
            }
            (AnnotationKind::ReplaceMethod, None) => {
                plans
                    .replace_global
                    .push(FunctionPlan::new(decl, label, 0x40000 | 0x2));
            }
            (kind, _) => return Err(format!("unsupported annotation {kind:?}")),
        }

        Ok(())
    }

    fn compile(&mut self, plan: &mut FunctionPlan) {
        let compiled = compile_body(
            self.symbols,
            &mut self.tables,
            plan.decl,
            plan.this_class.clone(),
            plan.wrap.clone(),
        );

        match compiled {
            Ok(CompiledBody {
                instructions,
                locals,
            }) => {
                let bytecode = ParsedFunctionBytecode {
                    instructions,
                    definition_description: String::new(),
                }
                .to_bytes();
                plan.compiled = Some(CompiledFunction { locals, bytecode });
            }
            Err(e) => self.errors.push(format!("{}: {e}", plan.label)),
        }
    }

    fn enum_definition(&mut self, decl: &EnumDecl) -> EnumDefinition {
        let name = self.tables.name(&decl.name);
        let enumerators = decl
            .members
            .iter()
            .map(|(name, value)| Enumerator {
                name: self.tables.name(name),
                value: value.map(|v| v as i32).unwrap_or(IMPLICIT_ENUM_VALUE),
            })
            .collect();

        EnumDefinition { name, enumerators }
    }

    fn property(&mut self, name: &str, ty: &Type, flags: u32, class: &str) -> PropertyDefinition {
        let type_name = definition_type_name(ty, self.symbols);
        PropertyDefinition {
            name: self.tables.name(name),
            hint: String::new(),
            flags,
            type_name: self.tables.name(&type_name),
            class_name: self.tables.name(class),
            binding: self.tables.name(""),
        }
    }

    fn class_property(&mut self, field: &FieldDecl, name: &str, class: &str) -> R<PropertyDefinition> {
        let ty = self.symbols.resolve(&field.ty)?;
        let specifiers = &field.specifiers;

        let mut flags = 0x20
            | match specifiers.visibility {
                Visibility::Private => 0x10000,
                Visibility::Protected => 0x20000,
                _ => 0x40000,
            };
        if specifiers.is_editable {
            flags |= 0x1;
        }
        if specifiers.is_const {
            flags |= 0x2;
        }
        if specifiers.is_saved {
            flags |= 0x8000;
        }

        Ok(self.property(name, &ty, flags, class))
    }

    fn default_values(&mut self, decl: &ClassDecl) -> R<Vec<DefaultValueDefinition>> {
        decl.defaults
            .iter()
            .map(|d| {
                Ok(DefaultValueDefinition {
                    name: self.tables.name(&d.field),
                    value: DefaultValue {
                        name: self.tables.name(""),
                        value: default_text(&d.value)?,
                        sub_values: Vec::new(),
                    },
                })
            })
            .collect()
    }

    fn struct_definition(&mut self, decl: &ClassDecl) -> R<StructDefinition> {
        let name = self.tables.name(&decl.name);

        let mut properties = Vec::new();
        for field in &decl.fields {
            let ty = self.symbols.resolve(&field.ty)?;
            for field_name in &field.names {
                properties.push(self.property(field_name, &ty, 0, ""));
            }
        }

        Ok(StructDefinition {
            name,
            flags: 0,
            properties,
            default_values: self.default_values(decl)?,
        })
    }

    fn class_definition(&mut self, plan: ClassPlan) -> R<ClassDefinition> {
        let decl = plan.decl;
        let name = self.tables.name(&decl.name);
        let parent_name = self.tables.name(match decl.state_machine_of {
            Some(_) => "",
            None => decl.parent.as_deref().unwrap_or(""),
        });
        let machine_name = self
            .tables
            .name(decl.state_machine_of.as_deref().unwrap_or(""));

        let mut properties = Vec::new();
        for field in &decl.fields {
            for field_name in &field.names {
                properties.push(
                    self.class_property(field, field_name, "")
                        .map_err(|e| format!("{}: {e}", decl.name))?,
                );
            }
        }

        let functions = self.function_definitions(plan.methods);

        Ok(ClassDefinition {
            name,
            parent_name,
            machine_name,
            is_state: decl.state_machine_of.is_some() as u8,
            flags: decl.is_abstract as u32 | if decl.is_statemachine { 0x400 } else { 0 },
            properties,
            functions,
            default_values: self
                .default_values(decl)
                .map_err(|e| format!("{}: {e}", decl.name))?,
        })
    }

    fn function_definitions(&mut self, plans: Vec<FunctionPlan>) -> Vec<FunctionDefinition> {
        plans
            .into_iter()
            .filter_map(|plan| {
                let label = plan.label.clone();
                match self.function_definition(plan) {
                    Ok(definition) => definition,
                    Err(e) => {
                        self.errors.push(format!("{label}: {e}"));
                        None
                    }
                }
            })
            .collect()
    }

    fn function_definition(&mut self, plan: FunctionPlan) -> R<Option<FunctionDefinition>> {
        let Some(compiled) = plan.compiled else {
            return Ok(None);
        };
        let decl = plan.decl;

        let name = self.tables.name(&decl.name);
        let override_class = self.tables.name(&plan.override_class);

        let return_type = match &decl.return_type {
            Some(ty) => match self.symbols.resolve(ty)? {
                Type::Void => None,
                ty => Some(self.property("__return", &ty, 0x40, "")),
            },
            None => None,
        };

        let mut parameters = Vec::new();
        for param in &decl.params {
            let ty = self.symbols.resolve(&param.ty)?;
            let flags = 0x80
                | if param.is_out { 0x200 } else { 0 }
                | if param.is_optional { 0x400 } else { 0 };
            parameters.push(self.property(&param.name, &ty, flags, ""));
        }

        let locals = compiled
            .locals
            .iter()
            .map(|(name, ty)| self.property(name, ty, 0x100, ""))
            .collect();

        Ok(Some(FunctionDefinition {
            name,
            override_class,
            flags: plan.flags,
            return_type,
            parameters,
            locals,
            bytecode: ByteCodeRef::new(compiled.bytecode),
        }))
    }
}

fn default_text(expr: &Expr) -> R<String> {
    Ok(match &expr.kind {
        ExprKind::Int(v) => v.to_string(),
        ExprKind::Float(_, text) => text.clone(),
        ExprKind::Str(s) | ExprKind::Name(s) => s.clone(),
        ExprKind::Bool(b) => b.to_string(),
        ExprKind::Ident(name) => name.clone(),
        ExprKind::Unary("-", inner) => format!("-{}", default_text(inner)?),
        ExprKind::Null => "NULL".to_owned(),
        other => return Err(format!("unsupported default value {other:?}")),
    })
}
