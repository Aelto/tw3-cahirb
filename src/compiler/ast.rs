#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeRef {
    pub name: String,
    pub args: Vec<TypeRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    #[default]
    Default,
    Public,
    Protected,
    Private,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotationKind {
    AddField,
    AddMethod,
    WrapMethod,
    ReplaceMethod,
}

#[derive(Debug, Clone)]
pub struct Annotation {
    pub kind: AnnotationKind,
    pub class: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct FieldSpecifiers {
    pub visibility: Visibility,
    pub is_import: bool,
    pub is_editable: bool,
    pub is_saved: bool,
    pub is_const: bool,
    pub is_inlined: bool,
}

#[derive(Debug, Clone)]
pub struct FieldDecl {
    pub names: Vec<String>,
    pub ty: TypeRef,
    pub specifiers: FieldSpecifiers,
    pub annotation: Option<Annotation>,
    pub initializer: Option<Expr>,
}

#[derive(Debug, Clone, Default)]
pub struct FunctionSpecifiers {
    pub visibility: Visibility,
    pub is_import: bool,
    pub is_final: bool,
    pub is_latent: bool,
    pub is_entry: bool,
    pub is_event: bool,
    pub is_timer: bool,
    pub is_exec: bool,
    pub is_quest: bool,
    pub is_storyscene: bool,
    pub is_reward: bool,
    pub is_cleanup: bool,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: TypeRef,
    pub is_out: bool,
    pub is_optional: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub specifiers: FunctionSpecifiers,
    pub params: Vec<Param>,
    pub return_type: Option<TypeRef>,
    pub body: Option<Vec<Stmt>>,
    pub annotation: Option<Annotation>,
}

#[derive(Debug, Clone)]
pub struct DefaultAssignment {
    pub field: String,
    pub value: Expr,
}

#[derive(Debug, Clone, Default)]
pub struct ClassDecl {
    pub name: String,
    pub parent: Option<String>,
    pub state_machine_of: Option<String>,
    pub is_import: bool,
    pub is_abstract: bool,
    pub is_statemachine: bool,
    pub is_struct: bool,
    pub fields: Vec<FieldDecl>,
    pub methods: Vec<FunctionDecl>,
    pub defaults: Vec<DefaultAssignment>,
}

#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: String,
    pub members: Vec<(String, Option<i64>)>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Class(ClassDecl),
    Enum(EnumDecl),
    Function(FunctionDecl),
    Field(FieldDecl),
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    pub line: u32,
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    Var(FieldDecl),
    Expr(Expr),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    DoWhile(Box<Stmt>, Expr),
    For(Option<Expr>, Option<Expr>, Option<Expr>, Box<Stmt>),
    Switch(Expr, Vec<SwitchItem>),
    Break,
    Continue,
    Return(Option<Expr>),
    Delete(Expr),
    Block(Vec<Stmt>),
    Empty,
}

#[derive(Debug, Clone)]
pub enum SwitchItem {
    Case(Expr),
    Default,
    Stmt(Stmt),
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub line: u32,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Int(i64),
    Float(f32, String),
    Str(String),
    Name(String),
    Bool(bool),
    Null,
    Ident(String),
    This,
    Super,
    Parent,
    VirtualParent,
    Member(Box<Expr>, String),
    Call(Box<Expr>, Vec<Option<Expr>>),
    Index(Box<Expr>, Box<Expr>),
    Unary(&'static str, Box<Expr>),
    Binary(&'static str, Box<Expr>, Box<Expr>),
    Assign(&'static str, Box<Expr>, Box<Expr>),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
    Cast(TypeRef, Box<Expr>),
    New(TypeRef, Option<Box<Expr>>),
}
