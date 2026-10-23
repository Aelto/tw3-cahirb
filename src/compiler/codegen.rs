use std::collections::HashMap;
use std::rc::Rc;

use super::ast::*;
use super::symbols::*;
use super::tables::*;
use crate::parser::INTERNAL_OPERATORS;
use crate::parser::ast::{
    FunctionRef, ImportTypeRef, Instruction, InternalOperatorRef, Op, PropertyRef, VariableRef,
};

type R<T> = Result<T, String>;

#[derive(Clone, Copy)]
enum Slot {
    First,
    Second,
}

struct Fixup {
    instruction: usize,
    slot: Slot,
    label: usize,
}

struct LoopLabels {
    break_label: usize,
    continue_label: Option<usize>,
}

#[derive(Clone)]
pub enum Wrap {
    None,
    Class(Rc<FunctionInfo>),
    Global(Rc<FunctionInfo>),
}

enum Resolved {
    Local(Type),
    Param(Type),
    Field(FieldInfo),
    EnumConstant(String, i32),
    Global(Op, Type),
}

enum Receiver<'e> {
    ImplicitThis,
    Super,
    Object(&'e Expr),
}

enum Callee<'e> {
    Method(Rc<FunctionInfo>, Receiver<'e>),
    Global(Rc<FunctionInfo>),
    Array(&'e str, &'e Expr, Type),
    Constructor(String),
    Wrapped(Rc<FunctionInfo>, bool),
}

enum BinaryPlan {
    Logic(&'static str),
    HandleAnd,
    Internal {
        name: String,
        left: Type,
        right: Type,
        result: Type,
    },
    Test {
        equal: bool,
        ty: Type,
    },
}

pub struct CompiledBody {
    pub instructions: Vec<Instruction>,
    pub locals: Vec<(String, Type)>,
}

pub struct FunctionCompiler<'a> {
    symbols: &'a Symbols,
    tables: &'a mut Tables,
    this_class: Option<String>,
    function_name: String,
    is_global: bool,
    wrap: Wrap,
    params: HashMap<String, Type>,
    locals: HashMap<String, Type>,
    return_type: Type,
    code: Vec<Op>,
    fixups: Vec<Fixup>,
    labels: Vec<Option<usize>>,
    loops: Vec<LoopLabels>,
}

pub fn compile_body(
    symbols: &Symbols,
    tables: &mut Tables,
    decl: &FunctionDecl,
    this_class: Option<String>,
    wrap: Wrap,
) -> R<CompiledBody> {
    let mut locals = Vec::new();
    if let Some(body) = &decl.body {
        collect_locals(body, &mut locals);
    }

    let locals = locals
        .into_iter()
        .map(|(name, ty)| symbols.resolve(&ty).map(|t| (name, t)))
        .collect::<R<Vec<_>>>()?;

    let params = decl
        .params
        .iter()
        .map(|p| symbols.resolve(&p.ty).map(|t| (p.name.clone(), t)))
        .collect::<R<HashMap<_, _>>>()?;

    let return_type = match &decl.return_type {
        Some(ty) => symbols.resolve(ty)?,
        None => Type::Void,
    };

    let mut compiler = FunctionCompiler {
        symbols,
        tables,
        is_global: this_class.is_none(),
        this_class,
        function_name: decl.name.clone(),
        wrap,
        params,
        locals: locals.iter().cloned().collect(),
        return_type,
        code: Vec::new(),
        fixups: Vec::new(),
        labels: Vec::new(),
        loops: Vec::new(),
    };

    let body = decl.body.as_deref().unwrap_or_default();
    if !body.is_empty() {
        for statement in body {
            compiler.statement(statement)?;
        }
        compiler.emit(Op::Nop);
    }

    Ok(CompiledBody {
        instructions: compiler.finish()?,
        locals,
    })
}

fn collect_locals(statements: &[Stmt], out: &mut Vec<(String, TypeRef)>) {
    for statement in statements {
        collect_locals_in(statement, out);
    }
}

fn collect_locals_in(statement: &Stmt, out: &mut Vec<(String, TypeRef)>) {
    match &statement.kind {
        StmtKind::Var(decl) => {
            for name in &decl.names {
                out.push((name.clone(), decl.ty.clone()));
            }
        }
        StmtKind::If(_, then, otherwise) => {
            collect_locals_in(then, out);
            if let Some(otherwise) = otherwise {
                collect_locals_in(otherwise, out);
            }
        }
        StmtKind::While(_, body) | StmtKind::DoWhile(body, _) | StmtKind::For(_, _, _, body) => {
            collect_locals_in(body, out)
        }
        StmtKind::Switch(_, items) => {
            for item in items {
                if let SwitchItem::Stmt(statement) = item {
                    collect_locals_in(statement, out);
                }
            }
        }
        StmtKind::Block(statements) => collect_locals(statements, out),
        _ => {}
    }
}

fn type_tag(ty: &Type) -> Option<String> {
    match ty {
        Type::Int => Some("Int32".to_owned()),
        Type::Float => Some("Float".to_owned()),
        Type::Byte => Some("Uint8".to_owned()),
        Type::String => Some("String".to_owned()),
        Type::Bool => Some("Bool".to_owned()),
        Type::Struct(name) | Type::Primitive(name) => Some(name.clone()),
        _ => None,
    }
}

fn operator_index(name: &str) -> Option<usize> {
    INTERNAL_OPERATORS.iter().position(|o| *o == name)
}

fn operator_prefix(op: &str) -> R<&'static str> {
    Ok(match op {
        "+" | "+=" => "Add",
        "-" | "-=" => "Subtract",
        "*" | "*=" => "Multiply",
        "/" | "/=" => "Divide",
        "%" => "Modulo",
        "&" | "&=" => "And",
        "|" | "|=" => "Or",
        "^" => "Xor",
        "==" => "Equal",
        "!=" => "NotEqual",
        "<" => "Less",
        "<=" => "LessEqual",
        ">" => "Greater",
        ">=" => "GreaterEqual",
        other => return Err(format!("unsupported operator {other}")),
    })
}

fn numeric(ty: &Type) -> Type {
    match ty {
        Type::Enum(_) => Type::Int,
        other => other.clone(),
    }
}

fn global_object(name: &str) -> Option<(Op, &'static str)> {
    Some(match name {
        "theGame" => (Op::GetGame, "CR4Game"),
        "thePlayer" => (Op::GetPlayer, "CR4Player"),
        "theCamera" => (Op::GetCamera, "CCameraDirector"),
        "theSound" => (Op::GetSound, "CScriptSoundSystem"),
        "theDebug" => (Op::GetDebug, "CDebugAttributesManager"),
        "theTimer" => (Op::GetTimer, "CTimerScriptKeyword"),
        "theInput" => (Op::GetInput, "CInputManager"),
        "theTelemetry" => (Op::GetTelemetry, "CR4TelemetryScriptProxy"),
        _ => return None,
    })
}

impl<'a> FunctionCompiler<'a> {
    fn emit(&mut self, op: Op) -> usize {
        self.code.push(op);
        self.code.len() - 1
    }

    fn label(&mut self) -> usize {
        self.labels.push(None);
        self.labels.len() - 1
    }

    fn place(&mut self, label: usize) {
        self.labels[label] = Some(self.code.len());
    }

    fn jump_to(&mut self, instruction: usize, slot: Slot, label: usize) {
        self.fixups.push(Fixup {
            instruction,
            slot,
            label,
        });
    }

    fn breakpoint(&mut self, line: u32) {
        self.emit(Op::Breakpoint {
            source_line: line,
            is_set: false,
        });
    }

    fn finish(self) -> R<Vec<Instruction>> {
        let mut offsets = Vec::with_capacity(self.code.len() + 1);
        let mut offset = 0usize;
        for op in &self.code {
            offsets.push(offset);
            offset += op.runtime_size();
        }
        offsets.push(offset);

        let mut code = self.code;

        for fixup in &self.fixups {
            let target_index = self.labels[fixup.label].ok_or("unplaced label")?;
            let target = offsets[target_index] as i64;
            let base = offsets[fixup.instruction] as i64;
            let op = &mut code[fixup.instruction];

            let field_end = match (&op, fixup.slot) {
                (Op::Switch { .. }, _) => 11,
                (_, Slot::First) => 3,
                (_, Slot::Second) => 5,
            };
            let value = target - (base + field_end);

            match (op, fixup.slot) {
                (Op::Jump { skip_offset }, _)
                | (Op::JumpIfFalse { skip_offset }, _)
                | (Op::Skip { skip_offset }, _)
                | (Op::Switch { skip_offset, .. }, _) => *skip_offset = value as i16,
                (Op::SwitchLabel { next_label_offset, .. }, Slot::First) => {
                    *next_label_offset = value as i16
                }
                (Op::SwitchLabel { expr_skip_offset, .. }, Slot::Second) => {
                    *expr_skip_offset = value as i16
                }
                (Op::Conditional { select_offset, .. }, Slot::First) => *select_offset = value as i16,
                (Op::Conditional { skip_offset, .. }, Slot::Second) => *skip_offset = value as i16,
                (Op::Context { skip_offset }, _)
                | (Op::FinalFunc { skip_offset, .. }, _)
                | (Op::VirtualFunc { skip_offset, .. }, _)
                | (Op::VirtualParentFunc { skip_offset, .. }, _)
                | (Op::EntryFunc { skip_offset, .. }, _) => *skip_offset = value as u16,
                _ => return Err("fixup on non-jump op".to_owned()),
            }
        }

        Ok(code
            .into_iter()
            .zip(offsets)
            .map(|(op, offset)| Instruction::new(op, offset))
            .collect())
    }

    fn statement(&mut self, statement: &Stmt) -> R<()> {
        let line = statement.line;

        match &statement.kind {
            StmtKind::Var(decl) => {
                if let Some(init) = &decl.initializer {
                    for name in &decl.names {
                        let ty = self.locals[name].clone();
                        self.breakpoint(line);
                        self.emit(Op::Assign);
                        let name = self.tables.name(name);
                        self.emit(Op::LocalVar {
                            name: VariableRef(name),
                        });
                        self.expr_as(init, &ty)?;
                    }
                }
            }
            StmtKind::Expr(expr) => {
                self.breakpoint(line);
                self.expr(expr)?;
            }
            StmtKind::If(cond, then, otherwise) => {
                self.breakpoint(line);
                let else_label = self.label();
                let at = self.emit(Op::JumpIfFalse { skip_offset: 0 });
                self.jump_to(at, Slot::First, else_label);
                self.expr_as(cond, &Type::Bool)?;
                self.statement(then)?;

                match otherwise {
                    Some(otherwise) => {
                        let end = self.label();
                        let at = self.emit(Op::Jump { skip_offset: 0 });
                        self.jump_to(at, Slot::First, end);
                        self.place(else_label);
                        self.statement(otherwise)?;
                        self.place(end);
                    }
                    None => self.place(else_label),
                }
            }
            StmtKind::While(cond, body) => {
                let start = self.label();
                let end = self.label();
                self.place(start);
                let at = self.emit(Op::JumpIfFalse { skip_offset: 0 });
                self.jump_to(at, Slot::First, end);
                self.breakpoint(line);
                self.expr_as(cond, &Type::Bool)?;
                self.loop_body(body, end, Some(start))?;
                let at = self.emit(Op::Jump { skip_offset: 0 });
                self.jump_to(at, Slot::First, start);
                self.place(end);
            }
            StmtKind::DoWhile(body, cond) => {
                let start = self.label();
                let check = self.label();
                let end = self.label();
                self.place(start);
                self.loop_body(body, end, Some(check))?;
                self.place(check);
                let at = self.emit(Op::JumpIfFalse { skip_offset: 0 });
                self.jump_to(at, Slot::First, end);
                self.breakpoint(line);
                self.expr_as(cond, &Type::Bool)?;
                let at = self.emit(Op::Jump { skip_offset: 0 });
                self.jump_to(at, Slot::First, start);
                self.place(end);
            }
            StmtKind::For(init, cond, step, body) => {
                if let Some(init) = init {
                    self.breakpoint(line);
                    self.expr(init)?;
                }

                let start = self.label();
                let next = self.label();
                let end = self.label();
                self.place(start);

                if let Some(cond) = cond {
                    let at = self.emit(Op::JumpIfFalse { skip_offset: 0 });
                    self.jump_to(at, Slot::First, end);
                    self.expr_as(cond, &Type::Bool)?;
                }

                self.loop_body(body, end, Some(next))?;
                self.place(next);

                if let Some(step) = step {
                    self.breakpoint(line);
                    self.expr(step)?;
                }

                let at = self.emit(Op::Jump { skip_offset: 0 });
                self.jump_to(at, Slot::First, start);
                self.place(end);
            }
            StmtKind::Switch(expr, items) => {
                self.breakpoint(line);
                self.switch(expr, items)?;
            }
            StmtKind::Break => {
                let target = self.loops.last().ok_or("break outside loop")?.break_label;
                self.breakpoint(line);
                let at = self.emit(Op::Jump { skip_offset: 0 });
                self.jump_to(at, Slot::First, target);
            }
            StmtKind::Continue => {
                let target = self
                    .loops
                    .iter()
                    .rev()
                    .find_map(|l| l.continue_label)
                    .ok_or("continue outside loop")?;
                self.breakpoint(line);
                let at = self.emit(Op::Jump { skip_offset: 0 });
                self.jump_to(at, Slot::First, target);
            }
            StmtKind::Return(value) => {
                self.emit(Op::Return);
                match value {
                    Some(value) => {
                        let source = self.type_of(value)?;
                        let target = self.return_type.clone();
                        for op in self.conversion(&source, &target)? {
                            self.emit(op);
                        }
                        self.breakpoint(line);
                        self.expr(value)?;
                    }
                    None => {
                        self.breakpoint(line);
                        self.emit(Op::Nop);
                    }
                }
            }
            StmtKind::Delete(value) => {
                self.breakpoint(line);
                self.emit(Op::Delete);
                self.expr(value)?;
            }
            StmtKind::Block(statements) => {
                for statement in statements {
                    self.statement(statement)?;
                }
            }
            StmtKind::Empty => {}
        }

        Ok(())
    }

    fn loop_body(&mut self, body: &Stmt, end: usize, next: Option<usize>) -> R<()> {
        self.loops.push(LoopLabels {
            break_label: end,
            continue_label: next,
        });
        let result = self.statement(body);
        self.loops.pop();
        result
    }

    fn switch(&mut self, expr: &Expr, items: &[SwitchItem]) -> R<()> {
        let ty = self.type_of(expr)?;
        let expr_type = self.tables.import_type(&ty, self.symbols);
        let end = self.label();
        let default_label = self.label();
        let has_default = items.iter().any(|i| matches!(i, SwitchItem::Default));

        let switch_at = self.emit(Op::Switch {
            expr_type,
            skip_offset: 0,
        });
        self.expr(expr)?;

        let mut groups: Vec<Vec<(usize, usize)>> = Vec::new();
        let mut in_group = false;

        self.loops.push(LoopLabels {
            break_label: end,
            continue_label: None,
        });

        for item in items {
            match item {
                SwitchItem::Case(value) => {
                    let label = self.label();
                    self.place(label);
                    let at = self.emit(Op::SwitchLabel {
                        next_label_offset: 0,
                        expr_skip_offset: 0,
                    });

                    match in_group {
                        true => {
                            let group = groups.last_mut().unwrap();
                            let (_, previous) = *group.last().unwrap();
                            self.jump_to(at, Slot::First, previous);
                            groups.last_mut().unwrap().push((at, label));
                        }
                        false => groups.push(vec![(at, label)]),
                    }
                    in_group = true;

                    self.case_value(at, value, &ty)?;
                }
                SwitchItem::Default => {
                    in_group = false;
                    self.place(default_label);
                    self.emit(Op::SwitchDefault);
                }
                SwitchItem::Stmt(statement) => {
                    in_group = false;
                    self.statement(statement)?;
                }
            }
        }

        self.loops.pop();
        self.place(end);

        let fallback = if has_default { default_label } else { end };
        let entry_of = |group: Option<&Vec<(usize, usize)>>| {
            group.and_then(|g| g.last()).map(|(_, label)| *label).unwrap_or(fallback)
        };

        self.jump_to(switch_at, Slot::First, entry_of(groups.first()));
        for index in 0..groups.len() {
            let (head_at, _) = groups[index][0];
            let next = entry_of(groups.get(index + 1));
            self.jump_to(head_at, Slot::First, next);
        }

        Ok(())
    }

    fn case_value(&mut self, label_at: usize, value: &Expr, ty: &Type) -> R<()> {
        let after = self.label();
        self.jump_to(label_at, Slot::Second, after);
        self.expr_as(value, ty)?;
        self.place(after);
        Ok(())
    }

    fn resolve_ident(&self, name: &str) -> R<Resolved> {
        if let Some(ty) = self.locals.get(name) {
            return Ok(Resolved::Local(ty.clone()));
        }
        if let Some(ty) = self.params.get(name) {
            return Ok(Resolved::Param(ty.clone()));
        }
        if let Some(class) = &self.this_class {
            if let Some(field) = self.symbols.find_field(class, name) {
                return Ok(Resolved::Field(field.clone()));
            }
        }
        if let Some(enum_name) = self.symbols.enum_members.get(name) {
            let info = &self.symbols.enums[enum_name];
            let value = info.value_of(name).unwrap_or_default();
            return Ok(Resolved::EnumConstant(enum_name.clone(), value));
        }
        if let Some((op, class)) = global_object(name) {
            return Ok(Resolved::Global(op, Type::Class(class.to_owned())));
        }

        Err(format!("unknown identifier '{name}'"))
    }

    fn this_type(&self) -> R<Type> {
        let class = self.this_class.as_ref().ok_or("'this' outside class")?;
        Ok(Type::Class(class.clone()))
    }

    fn parent_class(&self) -> R<String> {
        let class = self.this_class.as_ref().ok_or("'super' outside class")?;
        self.symbols
            .classes
            .get(class)
            .and_then(|c| c.parent.clone())
            .ok_or_else(|| format!("{class} has no parent"))
    }

    fn state_machine(&self) -> R<String> {
        let class = self.this_class.as_ref().ok_or("'parent' outside state")?;
        self.symbols
            .classes
            .get(class)
            .and_then(|c| c.state_machine.clone())
            .ok_or_else(|| format!("'parent' used outside state ({class})"))
    }

    fn field_of(&self, ty: &Type, name: &str) -> R<FieldInfo> {
        let class = match ty {
            Type::Class(class) | Type::Struct(class) => class,
            other => return Err(format!("field '{name}' on non-object {other:?}")),
        };

        self.symbols
            .find_field(class, name)
            .cloned()
            .ok_or_else(|| format!("unknown field {class}.{name}"))
    }

    fn type_of(&self, expr: &Expr) -> R<Type> {
        Ok(match &expr.kind {
            ExprKind::Int(_) => Type::Int,
            ExprKind::Float(..) => Type::Float,
            ExprKind::Str(_) => Type::String,
            ExprKind::Name(_) => Type::Name,
            ExprKind::Bool(_) => Type::Bool,
            ExprKind::Null => Type::Null,
            ExprKind::Ident(name) => match self.resolve_ident(name)? {
                Resolved::Local(ty) | Resolved::Param(ty) | Resolved::Global(_, ty) => ty,
                Resolved::Field(field) => field.ty,
                Resolved::EnumConstant(name, _) => Type::Enum(name),
            },
            ExprKind::This => self.this_type()?,
            ExprKind::Super => Type::Class(self.parent_class()?),
            ExprKind::Parent | ExprKind::VirtualParent => Type::Class(self.state_machine()?),
            ExprKind::Member(object, name) => {
                let object_type = match object.kind {
                    ExprKind::Super => self.this_type()?,
                    _ => self.type_of(object)?,
                };
                self.field_of(&object_type, name)?.ty
            }
            ExprKind::Call(callee, _) => match self.resolve_call(callee)? {
                Callee::Method(info, _) | Callee::Global(info) | Callee::Wrapped(info, _) => {
                    info.return_type.clone()
                }
                Callee::Array(method, _, element) => match method {
                    "Size" | "FindFirst" | "FindLast" => Type::Int,
                    "Contains" | "Remove" => Type::Bool,
                    "Last" | "PopBack" => element,
                    _ => Type::Void,
                },
                Callee::Constructor(name) => Type::Struct(name),
            },
            ExprKind::Index(array, _) => match self.type_of(array)? {
                Type::Array(element) => *element,
                other => return Err(format!("indexing non-array {other:?}")),
            },
            ExprKind::Unary("!", _) => Type::Bool,
            ExprKind::Unary("~", _) => Type::Int,
            ExprKind::Unary(_, operand) => numeric(&self.type_of(operand)?),
            ExprKind::Binary(op, left, right) => {
                match self.plan_binary(op, &self.type_of(left)?, &self.type_of(right)?)? {
                    BinaryPlan::Logic(_) | BinaryPlan::HandleAnd | BinaryPlan::Test { .. } => {
                        Type::Bool
                    }
                    BinaryPlan::Internal { result, .. } => result,
                }
            }
            ExprKind::Assign(_, target, _) => self.type_of(target)?,
            ExprKind::Ternary(_, then, otherwise) => match self.type_of(then)? {
                Type::Null => self.type_of(otherwise)?,
                ty => ty,
            },
            ExprKind::Cast(ty, _) => self.symbols.resolve(ty)?,
            ExprKind::New(ty, _) => self.symbols.resolve(ty)?,
        })
    }

    fn resolve_call<'e>(&self, callee: &'e Expr) -> R<Callee<'e>> {
        match &callee.kind {
            ExprKind::Ident(name) => {
                if name == "wrappedMethod" {
                    match &self.wrap {
                        Wrap::Class(info) => return Ok(Callee::Wrapped(info.clone(), true)),
                        Wrap::Global(info) => return Ok(Callee::Wrapped(info.clone(), false)),
                        Wrap::None => {}
                    }
                }

                if let Some(class) = &self.this_class {
                    if let Some(method) = self.symbols.find_method(class, name) {
                        return Ok(Callee::Method(method, Receiver::ImplicitThis));
                    }
                }

                if let Some(function) = self.symbols.functions.get(name) {
                    return Ok(Callee::Global(function.clone()));
                }

                match self.symbols.classes.get(name) {
                    Some(class) if class.is_struct => Ok(Callee::Constructor(name.clone())),
                    _ => Err(format!("unknown function '{name}'")),
                }
            }
            ExprKind::Member(object, name) => {
                if let ExprKind::Super = object.kind {
                    let parent = self.parent_class()?;
                    let method = self
                        .symbols
                        .find_method(&parent, name)
                        .ok_or_else(|| format!("unknown method super.{name}"))?;
                    return Ok(Callee::Method(method, Receiver::Super));
                }

                match self.type_of(object)? {
                    Type::Array(element) => Ok(Callee::Array(name, object, *element)),
                    Type::Class(class) => {
                        let method = self
                            .symbols
                            .find_method(&class, name)
                            .ok_or_else(|| format!("unknown method {class}.{name}"))?;
                        Ok(Callee::Method(method, Receiver::Object(object)))
                    }
                    other => Err(format!("method call {name} on {other:?}")),
                }
            }
            _ => Err("unsupported call target".to_owned()),
        }
    }

    fn plan_binary(&self, op: &str, left: &Type, right: &Type) -> R<BinaryPlan> {
        match op {
            "&&" if left.is_class() && right.is_class() => return Ok(BinaryPlan::HandleAnd),
            "&&" => return Ok(BinaryPlan::Logic("LogicAnd_Bool_Bool")),
            "||" => return Ok(BinaryPlan::Logic("LogicOr_Bool_Bool")),
            "+" if *left == Type::String || *right == Type::String => {
                return Ok(BinaryPlan::Internal {
                    name: "Add_String_String".to_owned(),
                    left: Type::String,
                    right: Type::String,
                    result: Type::String,
                });
            }
            _ => {}
        }

        let prefix = operator_prefix(op)?;
        let is_comparison = matches!(op, "==" | "!=" | "<" | "<=" | ">" | ">=");
        let left_numeric = numeric(left);
        let right_numeric = numeric(right);

        let mut candidates = vec![(left_numeric.clone(), right_numeric.clone())];
        match (&left_numeric, &right_numeric) {
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => {
                candidates.push((Type::Float, Type::Float))
            }
            (Type::Byte, Type::Int) | (Type::Int, Type::Byte) | (Type::Byte, Type::Byte) => {
                candidates.push((Type::Int, Type::Int));
                candidates.push((Type::Byte, Type::Byte));
            }
            (l, Type::Int) => candidates.push((l.clone(), Type::Float)),
            (Type::Int, r) => candidates.push((Type::Float, r.clone())),
            _ => {}
        }

        for (l, r) in candidates {
            let (Some(lt), Some(rt)) = (type_tag(&l), type_tag(&r)) else {
                continue;
            };
            let name = format!("{prefix}_{lt}_{rt}");
            if operator_index(&name).is_none() {
                continue;
            }

            let result = match (is_comparison, &l, &r) {
                (true, _, _) => Type::Bool,
                (false, Type::Float, Type::Struct(_)) => r.clone(),
                _ => l.clone(),
            };

            return Ok(BinaryPlan::Internal {
                name,
                left: l,
                right: r,
                result,
            });
        }

        match op {
            "==" | "!=" => Ok(BinaryPlan::Test {
                equal: op == "==",
                ty: match left {
                    Type::Null => right.clone(),
                    other => other.clone(),
                },
            }),
            _ => Err(format!("no operator {op} for {left:?} and {right:?}")),
        }
    }

    fn conversion(&mut self, from: &Type, to: &Type) -> R<Vec<Op>> {
        if from == to {
            return Ok(Vec::new());
        }

        let enum_ref = |tables: &mut Tables, symbols: &Symbols, name: &str| -> ImportTypeRef {
            tables.import_type(&Type::Enum(name.to_owned()), symbols)
        };

        Ok(match (from, to) {
            (Type::Null, _) => vec![],
            (Type::Class(_), Type::Class(_)) => vec![],
            (Type::Class(_), Type::Bool) => vec![Op::ObjectToBool],
            (Type::Class(_), Type::String) => vec![Op::ObjectToString],
            (Type::Int, Type::Float) => vec![Op::IntToFloat],
            (Type::Int, Type::Byte) => vec![Op::IntToByte],
            (Type::Int, Type::Bool) => vec![Op::IntToBool],
            (Type::Int, Type::String) => vec![Op::IntToString],
            (Type::Int, Type::Enum(name)) => vec![Op::IntToEnum {
                type_ref: enum_ref(self.tables, self.symbols, name),
            }],
            (Type::Byte, Type::Int) => vec![Op::ByteToInt],
            (Type::Byte, Type::Float) => vec![Op::ByteToFloat],
            (Type::Byte, Type::Bool) => vec![Op::ByteToBool],
            (Type::Byte, Type::String) => vec![Op::ByteToString],
            (Type::Float, Type::Int) => vec![Op::FloatToInt],
            (Type::Float, Type::Byte) => vec![Op::FloatToByte],
            (Type::Float, Type::Bool) => vec![Op::FloatToBool],
            (Type::Float, Type::String) => vec![Op::FloatToString],
            (Type::Bool, Type::Int) => vec![Op::BoolToInt],
            (Type::Bool, Type::Byte) => vec![Op::BoolToByte],
            (Type::Bool, Type::Float) => vec![Op::BoolToFloat],
            (Type::Bool, Type::String) => vec![Op::BoolToString],
            (Type::Name, Type::String) => vec![Op::NameToString],
            (Type::Name, Type::Bool) => vec![Op::NameToBool],
            (Type::String, Type::Int) => vec![Op::StringToInt],
            (Type::String, Type::Float) => vec![Op::StringToFloat],
            (Type::String, Type::Bool) => vec![Op::StringToBool],
            (Type::String, Type::Byte) => vec![Op::StringToByte],
            (Type::Enum(name), Type::Int) => vec![Op::EnumToInt {
                type_ref: enum_ref(self.tables, self.symbols, name),
            }],
            (Type::Enum(name), Type::String) => vec![Op::EnumToString {
                type_ref: enum_ref(self.tables, self.symbols, name),
            }],
            (Type::Enum(name), other @ (Type::Float | Type::Bool | Type::Byte)) => {
                let mut ops = self.conversion(&Type::Int, other)?;
                ops.push(Op::EnumToInt {
                    type_ref: enum_ref(self.tables, self.symbols, name),
                });
                ops
            }
            (Type::Enum(_), Type::Enum(_)) => vec![],
            (Type::Array(_), Type::Array(_)) => vec![],
            (Type::Struct(_), Type::Struct(_)) => vec![],
            (Type::Primitive(_), _) | (_, Type::Primitive(_)) => vec![],
            (_, Type::Void) => vec![],
            _ => return Err(format!("cannot convert {from:?} to {to:?}")),
        })
    }

    fn expr_as(&mut self, expr: &Expr, target: &Type) -> R<()> {
        let source = self.type_of(expr)?;
        for op in self.conversion(&source, target)? {
            self.emit(op);
        }
        self.expr(expr)
    }

    fn internal_call(
        &mut self,
        name: &str,
        line: u32,
        operands: impl FnOnce(&mut Self) -> R<()>,
    ) -> R<()> {
        let table_index = operator_index(name).ok_or_else(|| format!("unknown operator {name}"))?;
        let end = self.label();
        let at = self.emit(Op::FinalFunc {
            skip_offset: 0,
            source_line: line as u16,
            function: FunctionRef::InternalOperator(InternalOperatorRef { table_index }),
        });
        self.jump_to(at, Slot::First, end);
        operands(self)?;
        self.emit(Op::ParamEnd);
        self.place(end);
        Ok(())
    }

    fn property(&mut self, field: &FieldInfo) -> PropertyRef {
        let name = self.tables.name(&field.name);
        let owner_is_state = self
            .symbols
            .classes
            .get(&field.owner)
            .is_some_and(|class| class.state_machine.is_some());
        let scope = match owner_is_state {
            true => self.tables.import_type_named(&field.owner, TypeKind::Class),
            false => self.tables.class_type(&field.owner, self.symbols),
        };

        PropertyRef { name, scope }
    }

    fn enum_constant(&mut self, enum_name: &str, value: i32) {
        let info = &self.symbols.enums[enum_name];
        let fits = |range: std::ops::RangeInclusive<i32>| info.values.iter().all(|(_, v)| range.contains(v));

        let op = match info.is_script {
            true if fits(i8::MIN as i32..=i8::MAX as i32) => Op::ByteConst { value: value as u8 },
            true if fits(i16::MIN as i32..=i16::MAX as i32) => Op::ShortConst {
                value: value as i16,
            },
            _ => Op::IntConst { value },
        };
        self.emit(op);
    }

    fn expr(&mut self, expr: &Expr) -> R<()> {
        let line = expr.line;

        match &expr.kind {
            ExprKind::Int(0) => {
                self.emit(Op::IntZero);
            }
            ExprKind::Int(1) => {
                self.emit(Op::IntOne);
            }
            ExprKind::Int(value) => {
                self.emit(Op::IntConst {
                    value: *value as i32,
                });
            }
            ExprKind::Float(value, _) => {
                self.emit(Op::FloatConst { value: *value });
            }
            ExprKind::Str(value) => {
                self.emit(Op::StringConst {
                    value: value.clone(),
                });
            }
            ExprKind::Name(value) => {
                let value = self.tables.name(value);
                self.emit(Op::NameConst { value });
            }
            ExprKind::Bool(true) => {
                self.emit(Op::BoolTrue);
            }
            ExprKind::Bool(false) => {
                self.emit(Op::BoolFalse);
            }
            ExprKind::Null => {
                self.emit(Op::Null);
            }
            ExprKind::Ident(name) => match self.resolve_ident(name)? {
                Resolved::Local(_) => {
                    let name = self.tables.name(name);
                    self.emit(Op::LocalVar {
                        name: VariableRef(name),
                    });
                }
                Resolved::Param(_) => {
                    let name = self.tables.name(name);
                    self.emit(Op::ParamVar {
                        name: VariableRef(name),
                    });
                }
                Resolved::Field(field) => {
                    let property = self.property(&field);
                    self.emit(Op::ObjectVar { property });
                }
                Resolved::EnumConstant(enum_name, value) => self.enum_constant(&enum_name, value),
                Resolved::Global(op, _) => {
                    self.emit(op);
                }
            },
            ExprKind::This => {
                self.emit(Op::This);
            }
            ExprKind::Parent | ExprKind::VirtualParent => {
                self.emit(Op::Parent);
                self.emit(Op::This);
            }
            ExprKind::Super => return Err("bare 'super'".to_owned()),
            ExprKind::Member(object, name) => self.member(object, name)?,
            ExprKind::Call(callee, args) => self.call(callee, args, line)?,
            ExprKind::Index(array, index) => {
                let array_type = self.type_of(array)?;
                let type_ref = self.tables.import_type(&array_type, self.symbols);
                self.emit(Op::ArrayElement { type_ref });
                self.expr(array)?;
                self.expr_as(index, &Type::Int)?;
            }
            ExprKind::Unary(op, operand) => self.unary(op, operand, line)?,
            ExprKind::Binary(op, left, right) => self.binary(op, left, right, line)?,
            ExprKind::Assign(op, target, value) => self.assign(op, target, value, line)?,
            ExprKind::Ternary(cond, then, otherwise) => {
                let ty = self.type_of(expr)?;
                let else_label = self.label();
                let end = self.label();
                let at = self.emit(Op::Conditional {
                    select_offset: 0,
                    skip_offset: 0,
                });
                self.jump_to(at, Slot::First, else_label);
                self.jump_to(at, Slot::Second, end);
                self.expr_as(cond, &Type::Bool)?;
                self.expr_as(then, &ty)?;
                self.place(else_label);
                self.expr_as(otherwise, &ty)?;
                self.place(end);
            }
            ExprKind::Cast(ty, operand) => {
                let target = self.symbols.resolve(ty)?;
                let source = self.type_of(operand)?;
                match (&source, &target) {
                    (Type::Class(from), Type::Class(to)) if !self.symbols.is_subclass(from, to) => {
                        let type_ref = self.tables.class_type(to, self.symbols);
                        self.emit(Op::DynamicCast { type_ref });
                    }
                    _ => {
                        for op in self.conversion(&source, &target)? {
                            self.emit(op);
                        }
                    }
                }
                self.expr(operand)?;
            }
            ExprKind::New(ty, owner) => {
                let class = self.symbols.resolve(ty)?;
                let Type::Class(class) = class else {
                    return Err(format!("new on non-class {}", ty.name));
                };
                let type_ref = self.tables.class_type(&class, self.symbols);
                self.emit(Op::New { type_ref });
                match owner {
                    Some(owner) => self.expr(owner)?,
                    None => {
                        self.emit(Op::Null);
                    }
                }
            }
        }

        Ok(())
    }

    fn member(&mut self, object: &Expr, name: &str) -> R<()> {
        if matches!(object.kind, ExprKind::Super) {
            let field = self.field_of(&self.this_type()?, name)?;
            let property = self.property(&field);
            self.emit(Op::ObjectVar { property });
            return Ok(());
        }

        let object_type = self.type_of(object)?;
        let field = self.field_of(&object_type, name)?;

        match object_type {
            Type::Struct(_) => {
                let property = self.property(&field);
                self.emit(Op::StructMember { property });
                self.expr(object)
            }
            _ => {
                let end = self.label();
                let at = self.emit(Op::Context { skip_offset: 0 });
                self.jump_to(at, Slot::First, end);
                self.expr(object)?;
                let property = self.property(&field);
                self.emit(Op::ObjectVar { property });
                self.place(end);
                Ok(())
            }
        }
    }

    fn args(&mut self, params: &[ParamInfo], args: &[Option<Expr>]) -> R<()> {
        if args.len() > params.len() {
            return Err(format!(
                "too many arguments ({} for {})",
                args.len(),
                params.len()
            ));
        }

        for (index, param) in params.iter().enumerate() {
            match args.get(index).and_then(Option::as_ref) {
                Some(arg) => self.expr_as(arg, &param.ty)?,
                None => {
                    self.emit(Op::Nop);
                }
            }
        }

        Ok(())
    }

    fn call(&mut self, callee: &Expr, args: &[Option<Expr>], line: u32) -> R<()> {
        match self.resolve_call(callee)? {
            Callee::Wrapped(info, is_class) => {
                let end = self.label();
                let at = match is_class {
                    true => {
                        let function = self.tables.name("@self");
                        self.emit(Op::VirtualFunc {
                            skip_offset: 0,
                            source_line: line as u16,
                            function,
                        })
                    }
                    false => self.emit(Op::FinalFunc {
                        skip_offset: 0,
                        source_line: line as u16,
                        function: FunctionRef::CurrentFunction,
                    }),
                };
                self.jump_to(at, Slot::First, end);
                self.args(&info.params, args)?;
                self.emit(Op::ParamEnd);
                self.place(end);
            }
            Callee::Global(info) => {
                let end = self.label();
                let function = match self.is_global && info.name == self.function_name {
                    true => FunctionRef::CurrentFunction,
                    false => FunctionRef::Import(self.tables.import_function(&info.name, None, self.symbols)),
                };
                let at = self.emit(Op::FinalFunc {
                    skip_offset: 0,
                    source_line: line as u16,
                    function,
                });
                self.jump_to(at, Slot::First, end);
                self.args(&info.params, args)?;
                self.emit(Op::ParamEnd);
                self.place(end);
            }
            Callee::Method(info, receiver) => {
                let context_end = match receiver {
                    Receiver::Object(object) => {
                        let end = self.label();
                        let at = self.emit(Op::Context { skip_offset: 0 });
                        self.jump_to(at, Slot::First, end);
                        self.expr(object)?;
                        Some(end)
                    }
                    _ => None,
                };

                let static_dispatch = match receiver {
                    Receiver::Super => true,
                    Receiver::Object(object) => {
                        matches!(object.kind, ExprKind::Parent | ExprKind::VirtualParent)
                    }
                    Receiver::ImplicitThis => false,
                };
                self.invoke(&info, args, line, static_dispatch)?;

                if let Some(end) = context_end {
                    self.place(end);
                }
            }
            Callee::Array(method, array, element) => {
                let array_type = Type::Array(Box::new(element.clone()));
                let type_ref = self.tables.import_type(&array_type, self.symbols);

                let by_value = match args.first().and_then(Option::as_ref).map(|arg| &arg.kind) {
                    Some(
                        ExprKind::Call(..)
                        | ExprKind::Name(_)
                        | ExprKind::This
                        | ExprKind::Parent
                        | ExprKind::VirtualParent,
                    ) => true,
                    Some(ExprKind::Ident(name)) => {
                        matches!(self.resolve_ident(name)?, Resolved::Global(..))
                    }
                    _ => false,
                };

                let (op, params): (Op, Vec<Type>) = match method {
                    "Size" => (Op::ArraySize { type_ref }, vec![]),
                    "Clear" => (Op::ArrayClear { type_ref }, vec![]),
                    "PushBack" => (Op::ArrayPushBack { type_ref }, vec![element]),
                    "PopBack" => (Op::ArrayPopBack { type_ref }, vec![]),
                    "Insert" => (Op::ArrayInsert { type_ref }, vec![Type::Int, element]),
                    "Remove" if by_value => (Op::ArrayRemove { type_ref }, vec![element]),
                    "Remove" => (Op::ArrayRemoveFast { type_ref }, vec![element]),
                    "Contains" if by_value => (Op::ArrayContains { type_ref }, vec![element]),
                    "Contains" => (Op::ArrayContainsFast { type_ref }, vec![element]),
                    "FindFirst" if by_value => (Op::ArrayFindFirst { type_ref }, vec![element]),
                    "FindFirst" => (Op::ArrayFindFirstFast { type_ref }, vec![element]),
                    "FindLast" if by_value => (Op::ArrayFindLast { type_ref }, vec![element]),
                    "FindLast" => (Op::ArrayFindLastFast { type_ref }, vec![element]),
                    "Resize" => (Op::ArrayResize { type_ref }, vec![Type::Int]),
                    "Grow" => (Op::ArrayGrow { type_ref }, vec![Type::Int]),
                    "Erase" => (Op::ArrayErase { type_ref }, vec![Type::Int]),
                    "EraseFast" => (Op::ArrayEraseFast { type_ref }, vec![Type::Int]),
                    "Last" => (Op::ArrayLast { type_ref }, vec![]),
                    other => return Err(format!("unknown array method {other}")),
                };

                if args.len() != params.len() {
                    return Err(format!("array {method}: wrong argument count"));
                }

                self.emit(op);
                self.expr(array)?;
                for (arg, ty) in args.iter().zip(&params) {
                    let arg = arg.as_ref().ok_or("missing array argument")?;
                    self.expr_as(arg, ty)?;
                }
            }
            Callee::Constructor(name) => {
                let struct_type = Type::Struct(name.clone());
                let fields: Vec<Type> = {
                    let class = &self.symbols.classes[&name];
                    class
                        .field_order
                        .iter()
                        .map(|f| class.fields[f].ty.clone())
                        .collect()
                };
                let type_ref = self.tables.import_type(&struct_type, self.symbols);
                self.emit(Op::Constructor {
                    num_params: args.len() as u8,
                    type_ref,
                });
                for (arg, ty) in args.iter().zip(&fields) {
                    let arg = arg.as_ref().ok_or("missing constructor argument")?;
                    self.expr_as(arg, ty)?;
                }
            }
        }

        Ok(())
    }

    fn invoke(&mut self, info: &FunctionInfo, args: &[Option<Expr>], line: u32, is_super: bool) -> R<()> {
        let end = self.label();

        let at = if info.specifiers.is_entry && !is_super {
            let function = self.tables.name(&info.name);
            self.emit(Op::EntryFunc {
                skip_offset: 0,
                function,
            })
        } else if is_super || info.is_final_call() {
            let owner = info.owner.as_deref().ok_or("method without owner")?;
            let function =
                FunctionRef::Import(self.tables.import_function(&info.name, Some(owner), self.symbols));
            self.emit(Op::FinalFunc {
                skip_offset: 0,
                source_line: line as u16,
                function,
            })
        } else {
            let is_wrap = !matches!(self.wrap, Wrap::None);
            let name = match !is_wrap && info.name == self.function_name {
                true => "@self",
                false => info.name.as_str(),
            };
            let function = self.tables.name(name);
            self.emit(Op::VirtualFunc {
                skip_offset: 0,
                source_line: line as u16,
                function,
            })
        };

        self.jump_to(at, Slot::First, end);
        self.args(&info.params, args)?;
        self.emit(Op::ParamEnd);
        self.place(end);
        Ok(())
    }

    fn unary(&mut self, op: &str, operand: &Expr, line: u32) -> R<()> {
        match op {
            "!" => self.internal_call("LogicNot_Bool", line, |c| c.expr_as(operand, &Type::Bool)),
            "~" => self.internal_call("BitNot_Int32", line, |c| c.expr_as(operand, &Type::Int)),
            _ => {
                let ty = numeric(&self.type_of(operand)?);
                let tag = type_tag(&ty).ok_or_else(|| format!("cannot negate {ty:?}"))?;
                self.internal_call(&format!("Neg_{tag}"), line, |c| c.expr_as(operand, &ty))
            }
        }
    }

    fn binary(&mut self, op: &str, left: &Expr, right: &Expr, line: u32) -> R<()> {
        let left_type = self.type_of(left)?;
        let right_type = self.type_of(right)?;

        match self.plan_binary(op, &left_type, &right_type)? {
            BinaryPlan::Logic(name) => self.internal_call(name, line, |c| {
                c.expr_as(left, &Type::Bool)?;
                let after = c.label();
                let at = c.emit(Op::Skip { skip_offset: 0 });
                c.jump_to(at, Slot::First, after);
                c.expr_as(right, &Type::Bool)?;
                c.place(after);
                Ok(())
            }),
            BinaryPlan::HandleAnd => {
                self.internal_call("LogicAnd_handle_IScriptable_handle_IScriptable", line, |c| {
                    c.expr(left)?;
                    c.expr(right)
                })
            }
            BinaryPlan::Internal {
                name,
                left: lt,
                right: rt,
                ..
            } => self.internal_call(&name, line, |c| {
                c.expr_as(left, &lt)?;
                c.expr_as(right, &rt)
            }),
            BinaryPlan::Test { equal, ty } => {
                let type_ref = self.tables.import_type(&ty, self.symbols);
                self.emit(match equal {
                    true => Op::TestEqual { type_ref },
                    false => Op::TestNotEqual { type_ref },
                });
                self.expr_as(left, &ty)?;
                self.expr_as(right, &ty)
            }
        }
    }

    fn assign(&mut self, op: &str, target: &Expr, value: &Expr, line: u32) -> R<()> {
        let target_type = self.type_of(target)?;

        if op == "=" {
            self.emit(Op::Assign);
            self.expr(target)?;
            return self.expr_as(value, &target_type);
        }

        let value_type = numeric(&self.type_of(value)?);
        let prefix = operator_prefix(op)?;
        let target_tag = type_tag(&target_type).ok_or_else(|| format!("cannot {op} on {target_type:?}"))?;

        let candidates = [value_type.clone(), target_type.clone(), Type::Float, Type::Int];
        for candidate in candidates {
            let Some(tag) = type_tag(&candidate) else { continue };
            let name = format!("Assign{prefix}_{target_tag}_{tag}");
            if operator_index(&name).is_some() {
                return self.internal_call(&name, line, |c| {
                    c.expr(target)?;
                    c.expr_as(value, &candidate)
                });
            }
        }

        Err(format!("no operator {op} for {target_type:?} and {value_type:?}"))
    }
}
