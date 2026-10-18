use super::ast::*;
use super::lexer::{Tok, Token, tokenize};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    skip_bodies: bool,
}

type PResult<T> = Result<T, String>;

const ASSIGN_OPS: [&str; 7] = ["=", "+=", "-=", "*=", "/=", "&=", "|="];

const BINARY_LEVELS: [&[&str]; 9] = [
    &["||"],
    &["&&"],
    &["|"],
    &["^"],
    &["&"],
    &["==", "!="],
    &["<", "<=", ">", ">="],
    &["+", "-"],
    &["*", "/", "%"],
];

pub fn parse_source(source: &str, skip_bodies: bool) -> PResult<Vec<Item>> {
    let tokens = tokenize(source)?;
    let mut parser = Parser {
        tokens,
        pos: 0,
        skip_bodies,
    };

    parser.items()
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.tokens[self.pos].tok
    }

    fn peek_at(&self, offset: usize) -> &Tok {
        &self.tokens[(self.pos + offset).min(self.tokens.len() - 1)].tok
    }

    fn line(&self) -> u32 {
        self.tokens[self.pos].line
    }

    fn prev_line(&self) -> u32 {
        self.tokens[self.pos.saturating_sub(1)].line
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        token
    }

    fn error<T>(&self, message: &str) -> PResult<T> {
        let token = &self.tokens[self.pos];
        Err(format!("line {}: {message}, found '{}'", token.line, token.text))
    }

    fn is_punct(&self, punct: &str) -> bool {
        matches!(self.peek(), Tok::Punct(p) if *p == punct)
    }

    fn is_keyword(&self, keyword: &str) -> bool {
        matches!(self.peek(), Tok::Ident(i) if i == keyword)
    }

    fn eat_punct(&mut self, punct: &str) -> bool {
        let found = self.is_punct(punct);
        if found {
            self.advance();
        }
        found
    }

    fn eat_keyword(&mut self, keyword: &str) -> bool {
        let found = self.is_keyword(keyword);
        if found {
            self.advance();
        }
        found
    }

    fn expect_punct(&mut self, punct: &str) -> PResult<()> {
        match self.eat_punct(punct) {
            true => Ok(()),
            false => self.error(&format!("expected '{punct}'")),
        }
    }

    fn ident(&mut self) -> PResult<String> {
        match self.peek().clone() {
            Tok::Ident(name) => {
                self.advance();
                Ok(name)
            }
            _ => self.error("expected identifier"),
        }
    }

    fn items(&mut self) -> PResult<Vec<Item>> {
        let mut items = Vec::new();

        while *self.peek() != Tok::Eof {
            if self.eat_punct(";") {
                continue;
            }
            items.push(self.item()?);
        }

        Ok(items)
    }

    fn annotation(&mut self) -> PResult<Option<Annotation>> {
        if !self.eat_punct("@") {
            return Ok(None);
        }

        let kind = match self.ident()?.as_str() {
            "addField" => AnnotationKind::AddField,
            "addMethod" => AnnotationKind::AddMethod,
            "wrapMethod" => AnnotationKind::WrapMethod,
            "replaceMethod" => AnnotationKind::ReplaceMethod,
            other => return self.error(&format!("unknown annotation @{other}")),
        };

        let class = match self.eat_punct("(") {
            true => {
                let class = self.ident()?;
                self.expect_punct(")")?;
                Some(class)
            }
            false => None,
        };

        Ok(Some(Annotation { kind, class }))
    }

    fn item(&mut self) -> PResult<Item> {
        let annotation = self.annotation()?;

        let mut class = ClassDecl::default();
        let mut function_specifiers = FunctionSpecifiers::default();
        let mut field_specifiers = FieldSpecifiers::default();

        loop {
            let Tok::Ident(word) = self.peek().clone() else {
                return self.error("expected declaration");
            };

            match word.as_str() {
                "import" => {
                    class.is_import = true;
                    function_specifiers.is_import = true;
                    field_specifiers.is_import = true;
                }
                "abstract" => class.is_abstract = true,
                "statemachine" => class.is_statemachine = true,
                "class" => {
                    self.advance();
                    return self.class_body(class).map(Item::Class);
                }
                "struct" => {
                    self.advance();
                    class.is_struct = true;
                    return self.class_body(class).map(Item::Class);
                }
                "state" => {
                    self.advance();
                    return self.state(class).map(Item::Class);
                }
                "enum" => {
                    self.advance();
                    return self.enum_decl().map(Item::Enum);
                }
                "var" => {
                    let mut field = self.field(field_specifiers)?;
                    field.annotation = annotation;
                    return Ok(Item::Field(field));
                }
                _ if apply_specifier(&word, &mut function_specifiers, &mut field_specifiers) => {}
                "function" | "event" => {
                    let mut function = self.function(function_specifiers)?;
                    function.annotation = annotation;
                    return Ok(Item::Function(function));
                }
                _ => return self.error("expected declaration"),
            }

            self.advance();
        }
    }

    fn state(&mut self, mut class: ClassDecl) -> PResult<ClassDecl> {
        let name = self.ident()?;
        if !self.eat_keyword("in") {
            return self.error("expected 'in'");
        }
        class.state_machine_of = Some(self.ident()?);
        self.class_body_named(class, name)
    }

    fn class_body(&mut self, class: ClassDecl) -> PResult<ClassDecl> {
        let name = self.ident()?;
        self.class_body_named(class, name)
    }

    fn class_body_named(&mut self, mut class: ClassDecl, name: String) -> PResult<ClassDecl> {
        class.name = name;

        if self.eat_keyword("extends") {
            class.parent = Some(self.ident()?);
        }

        self.expect_punct("{")?;

        while !self.eat_punct("}") {
            if self.eat_punct(";") {
                continue;
            }
            self.member(&mut class)?;
        }

        Ok(class)
    }

    fn member(&mut self, class: &mut ClassDecl) -> PResult<()> {
        let mut function_specifiers = FunctionSpecifiers::default();
        let mut field_specifiers = FieldSpecifiers::default();

        loop {
            let Tok::Ident(word) = self.peek().clone() else {
                return self.error("expected member");
            };

            match word.as_str() {
                "import" => {
                    function_specifiers.is_import = true;
                    field_specifiers.is_import = true;
                }
                "var" => {
                    let field = self.field(field_specifiers)?;
                    class.fields.push(field);
                    return Ok(());
                }
                "autobind" => {
                    self.advance();
                    let name = self.ident()?;
                    self.expect_punct(":")?;
                    let ty = self.type_ref()?;
                    self.skip_until(";")?;
                    class.fields.push(FieldDecl {
                        names: vec![name],
                        ty,
                        specifiers: field_specifiers,
                        annotation: None,
                        initializer: None,
                    });
                    return Ok(());
                }
                "default" => {
                    self.advance();
                    let field = self.ident()?;
                    self.expect_punct("=")?;
                    match self.skip_bodies {
                        true => self.skip_until(";")?,
                        false => {
                            let value = self.expr()?;
                            self.expect_punct(";")?;
                            class.defaults.push(DefaultAssignment { field, value });
                        }
                    }
                    return Ok(());
                }
                "hint" => {
                    self.advance();
                    self.skip_until(";")?;
                    return Ok(());
                }
                "defaults" => {
                    self.advance();
                    self.skip_braces()?;
                    return Ok(());
                }
                _ if apply_specifier(&word, &mut function_specifiers, &mut field_specifiers) => {}
                "function" | "event" => {
                    let function = self.function(function_specifiers)?;
                    class.methods.push(function);
                    return Ok(());
                }
                _ => return self.error("expected member"),
            }

            self.advance();
        }
    }

    fn skip_until(&mut self, punct: &str) -> PResult<()> {
        let mut depth = 0i32;
        loop {
            match self.peek() {
                Tok::Eof => return self.error(&format!("expected '{punct}'")),
                Tok::Punct("(" | "{" | "[") => depth += 1,
                Tok::Punct(")" | "}" | "]") => depth -= 1,
                Tok::Punct(p) if *p == punct && depth == 0 => {
                    self.advance();
                    return Ok(());
                }
                _ => {}
            }
            self.advance();
        }
    }

    fn skip_braces(&mut self) -> PResult<()> {
        self.expect_punct("{")?;
        let mut depth = 1;
        while depth > 0 {
            match self.advance().tok {
                Tok::Punct("{") => depth += 1,
                Tok::Punct("}") => depth -= 1,
                Tok::Eof => return self.error("unbalanced braces"),
                _ => {}
            }
        }
        Ok(())
    }

    fn enum_decl(&mut self) -> PResult<EnumDecl> {
        let name = self.ident()?;
        self.expect_punct("{")?;

        let mut members = Vec::new();
        while !self.eat_punct("}") {
            let member = self.ident()?;
            let value = match self.eat_punct("=") {
                true => {
                    let negative = self.eat_punct("-");
                    match self.advance().tok {
                        Tok::Int(v) => Some(if negative { -v } else { v }),
                        _ => return self.error("expected enum value"),
                    }
                }
                false => None,
            };
            members.push((member, value));

            if !self.eat_punct(",") {
                self.expect_punct("}")?;
                break;
            }
        }

        Ok(EnumDecl { name, members })
    }

    fn field(&mut self, specifiers: FieldSpecifiers) -> PResult<FieldDecl> {
        if !self.eat_keyword("var") {
            return self.error("expected 'var'");
        }

        let mut names = vec![self.ident()?];
        while self.eat_punct(",") {
            names.push(self.ident()?);
        }
        self.expect_punct(":")?;
        let ty = self.type_ref()?;

        let initializer = match self.eat_punct("=") {
            true => Some(self.expr()?),
            false => None,
        };
        self.expect_punct(";")?;

        Ok(FieldDecl {
            names,
            ty,
            specifiers,
            annotation: None,
            initializer,
        })
    }

    fn type_ref(&mut self) -> PResult<TypeRef> {
        let name = self.ident()?;
        let mut args = Vec::new();

        if self.eat_punct("<") {
            args.push(self.type_ref()?);
            while self.eat_punct(",") {
                args.push(self.type_ref()?);
            }
            self.expect_punct(">")?;
        }

        Ok(TypeRef { name, args })
    }

    fn function(&mut self, mut specifiers: FunctionSpecifiers) -> PResult<FunctionDecl> {
        if self.eat_keyword("event") {
            specifiers.is_event = true;
        } else if !self.eat_keyword("function") {
            return self.error("expected 'function'");
        }

        let name = self.ident()?;
        self.expect_punct("(")?;

        let mut params = Vec::new();
        while !self.eat_punct(")") {
            let mut is_out = false;
            let mut is_optional = false;
            loop {
                if self.eat_keyword("out") {
                    is_out = true;
                } else if self.eat_keyword("optional") {
                    is_optional = true;
                } else {
                    break;
                }
            }

            let mut names = vec![self.ident()?];
            while self.eat_punct(",") {
                names.push(self.ident()?);
            }
            self.expect_punct(":")?;
            let ty = self.type_ref()?;

            params.extend(names.into_iter().map(|name| Param {
                name,
                ty: ty.clone(),
                is_out,
                is_optional,
            }));

            if !self.eat_punct(",") {
                self.expect_punct(")")?;
                break;
            }
        }

        let return_type = match self.eat_punct(":") {
            true => Some(self.type_ref()?),
            false => None,
        };

        let body = match self.is_punct("{") {
            true if self.skip_bodies => {
                self.skip_braces()?;
                Some(Vec::new())
            }
            true => Some(self.block()?),
            false => {
                self.expect_punct(";")?;
                None
            }
        };

        Ok(FunctionDecl {
            name,
            specifiers,
            params,
            return_type,
            body,
            annotation: None,
        })
    }

    fn block(&mut self) -> PResult<Vec<Stmt>> {
        self.expect_punct("{")?;
        let mut statements = Vec::new();
        while !self.eat_punct("}") {
            statements.push(self.statement()?);
        }
        Ok(statements)
    }

    fn statement(&mut self) -> PResult<Stmt> {
        let start_line = self.line();

        let kind = match self.peek().clone() {
            Tok::Punct("{") => StmtKind::Block(self.block()?),
            Tok::Punct(";") => {
                self.advance();
                StmtKind::Empty
            }
            Tok::Ident(word) => match word.as_str() {
                "var" => StmtKind::Var(self.field(FieldSpecifiers::default())?),
                "if" => {
                    self.advance();
                    let cond = self.paren_expr()?;
                    let line = start_line;
                    let then = Box::new(self.statement()?);
                    let otherwise = match self.eat_keyword("else") {
                        true => Some(Box::new(self.statement()?)),
                        false => None,
                    };
                    return Ok(Stmt {
                        kind: StmtKind::If(cond, then, otherwise),
                        line,
                    });
                }
                "while" => {
                    self.advance();
                    let cond = self.paren_expr()?;
                    let line = self.prev_line();
                    let body = Box::new(self.statement()?);
                    return Ok(Stmt {
                        kind: StmtKind::While(cond, body),
                        line,
                    });
                }
                "do" => {
                    self.advance();
                    let body = Box::new(self.statement()?);
                    if !self.eat_keyword("while") {
                        return self.error("expected 'while'");
                    }
                    let cond = self.paren_expr()?;
                    let line = self.prev_line();
                    self.eat_punct(";");
                    return Ok(Stmt {
                        kind: StmtKind::DoWhile(body, cond),
                        line,
                    });
                }
                "for" => {
                    self.advance();
                    self.expect_punct("(")?;
                    let init = self.optional_expr(";")?;
                    self.expect_punct(";")?;
                    let cond = self.optional_expr(";")?;
                    self.expect_punct(";")?;
                    let step = self.optional_expr(")")?;
                    self.expect_punct(")")?;
                    let line = self.prev_line();
                    let body = Box::new(self.statement()?);
                    return Ok(Stmt {
                        kind: StmtKind::For(init, cond, step, body),
                        line,
                    });
                }
                "switch" => {
                    self.advance();
                    let expr = self.paren_expr()?;
                    let line = self.prev_line();
                    let items = self.switch_items()?;
                    return Ok(Stmt {
                        kind: StmtKind::Switch(expr, items),
                        line,
                    });
                }
                "break" => {
                    self.advance();
                    self.expect_punct(";")?;
                    StmtKind::Break
                }
                "continue" => {
                    self.advance();
                    self.expect_punct(";")?;
                    StmtKind::Continue
                }
                "return" => {
                    self.advance();
                    let value = self.optional_expr(";")?;
                    self.expect_punct(";")?;
                    StmtKind::Return(value)
                }
                "delete" => {
                    self.advance();
                    let value = self.expr()?;
                    self.expect_punct(";")?;
                    StmtKind::Delete(value)
                }
                _ => self.expr_statement()?,
            },
            _ => self.expr_statement()?,
        };

        let line = match kind {
            StmtKind::Block(_) => start_line,
            _ => self.prev_line(),
        };

        Ok(Stmt { kind, line })
    }

    fn expr_statement(&mut self) -> PResult<StmtKind> {
        let expr = self.expr()?;
        self.expect_punct(";")?;
        Ok(StmtKind::Expr(expr))
    }

    fn switch_items(&mut self) -> PResult<Vec<SwitchItem>> {
        self.expect_punct("{")?;
        let mut items = Vec::new();

        while !self.eat_punct("}") {
            if self.eat_keyword("case") {
                let value = self.expr()?;
                self.expect_punct(":")?;
                items.push(SwitchItem::Case(value));
            } else if self.is_keyword("default") && matches!(self.peek_at(1), Tok::Punct(":")) {
                self.advance();
                self.advance();
                items.push(SwitchItem::Default);
            } else {
                items.push(SwitchItem::Stmt(self.statement()?));
            }
        }

        Ok(items)
    }

    fn paren_expr(&mut self) -> PResult<Expr> {
        self.expect_punct("(")?;
        let expr = self.expr()?;
        self.expect_punct(")")?;
        Ok(expr)
    }

    fn optional_expr(&mut self, terminator: &str) -> PResult<Option<Expr>> {
        match self.is_punct(terminator) {
            true => Ok(None),
            false => self.expr().map(Some),
        }
    }

    pub fn expr(&mut self) -> PResult<Expr> {
        let target = self.ternary()?;

        let op = ASSIGN_OPS.iter().find(|op| self.is_punct(op)).copied();
        let Some(op) = op else {
            return Ok(target);
        };

        self.advance();
        let value = self.expr()?;
        let line = self.line();

        Ok(Expr {
            kind: ExprKind::Assign(op, Box::new(target), Box::new(value)),
            line,
        })
    }

    fn ternary(&mut self) -> PResult<Expr> {
        let cond = self.binary(0)?;
        if !self.eat_punct("?") {
            return Ok(cond);
        }

        let then = self.expr()?;
        self.expect_punct(":")?;
        let otherwise = self.ternary()?;
        let line = self.line();

        Ok(Expr {
            kind: ExprKind::Ternary(Box::new(cond), Box::new(then), Box::new(otherwise)),
            line,
        })
    }

    fn binary(&mut self, level: usize) -> PResult<Expr> {
        if level == BINARY_LEVELS.len() {
            return self.unary();
        }

        let mut left = self.binary(level + 1)?;

        loop {
            let op = BINARY_LEVELS[level]
                .iter()
                .find(|op| self.is_punct(op))
                .copied();
            let Some(op) = op else {
                return Ok(left);
            };

            self.advance();
            let right = self.binary(level + 1)?;
            let line = self.line();

            left = Expr {
                kind: ExprKind::Binary(op, Box::new(left), Box::new(right)),
                line,
            };
        }
    }

    fn unary(&mut self) -> PResult<Expr> {
        for op in ["!", "-", "~"] {
            if self.eat_punct(op) {
                let operand = self.unary()?;
                let line = self.line();
                return Ok(Expr {
                    kind: ExprKind::Unary(op, Box::new(operand)),
                    line,
                });
            }
        }

        if self.is_cast() {
            self.advance();
            let ty = self.type_ref()?;
            self.expect_punct(")")?;
            let operand = self.unary()?;
            let line = self.line();
            return Ok(Expr {
                kind: ExprKind::Cast(ty, Box::new(operand)),
                line,
            });
        }

        self.postfix()
    }

    fn is_cast(&self) -> bool {
        if !self.is_punct("(") {
            return false;
        }
        let Tok::Ident(name) = self.peek_at(1) else {
            return false;
        };
        if matches!(name.as_str(), "this" | "super" | "parent" | "NULL" | "true" | "false") {
            return false;
        }
        if !matches!(self.peek_at(2), Tok::Punct(")")) {
            return false;
        }

        match self.peek_at(3) {
            Tok::Ident(_) | Tok::Int(_) | Tok::Float(_) | Tok::Str(_) | Tok::Name(_) => true,
            Tok::Punct("(") => true,
            _ => false,
        }
    }

    fn postfix(&mut self) -> PResult<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.eat_punct(".") {
                let member = self.ident()?;
                let line = self.prev_line();
                expr = Expr {
                    kind: ExprKind::Member(Box::new(expr), member),
                    line,
                };
            } else if self.eat_punct("(") {
                let args = self.arguments()?;
                let line = self.prev_line();
                expr = Expr {
                    kind: ExprKind::Call(Box::new(expr), args),
                    line,
                };
            } else if self.eat_punct("[") {
                let index = self.expr()?;
                self.expect_punct("]")?;
                let line = self.line();
                expr = Expr {
                    kind: ExprKind::Index(Box::new(expr), Box::new(index)),
                    line,
                };
            } else {
                return Ok(expr);
            }
        }
    }

    fn arguments(&mut self) -> PResult<Vec<Option<Expr>>> {
        let mut args = Vec::new();
        if self.eat_punct(")") {
            return Ok(args);
        }

        loop {
            match self.is_punct(",") || self.is_punct(")") {
                true => args.push(None),
                false => args.push(Some(self.expr()?)),
            }

            if self.eat_punct(")") {
                return Ok(args);
            }
            self.expect_punct(",")?;
        }
    }

    fn primary(&mut self) -> PResult<Expr> {
        let token = self.advance();
        let line = token.line;

        let kind = match token.tok {
            Tok::Int(v) => ExprKind::Int(v),
            Tok::Float(v) => ExprKind::Float(v, token.text.trim_end_matches(['f', 'F']).to_owned()),
            Tok::Str(s) => ExprKind::Str(s),
            Tok::Name(s) => ExprKind::Name(s),
            Tok::Punct("(") => {
                let inner = self.expr()?;
                self.expect_punct(")")?;
                return Ok(inner);
            }
            Tok::Ident(word) => match word.as_str() {
                "true" => ExprKind::Bool(true),
                "false" => ExprKind::Bool(false),
                "NULL" => ExprKind::Null,
                "this" => ExprKind::This,
                "super" => ExprKind::Super,
                "parent" => ExprKind::Parent,
                "virtual_parent" => ExprKind::VirtualParent,
                "new" => {
                    let ty = self.type_ref()?;
                    let owner = match self.eat_keyword("in") {
                        true => Some(Box::new(self.postfix()?)),
                        false => None,
                    };
                    ExprKind::New(ty, owner)
                }
                _ => ExprKind::Ident(word),
            },
            _ => {
                self.pos -= 1;
                return self.error("expected expression");
            }
        };

        Ok(Expr { kind, line })
    }
}

fn apply_specifier(
    word: &str,
    function: &mut FunctionSpecifiers,
    field: &mut FieldSpecifiers,
) -> bool {
    let visibility = match word {
        "private" => Some(Visibility::Private),
        "protected" => Some(Visibility::Protected),
        "public" => Some(Visibility::Public),
        _ => None,
    };

    if let Some(visibility) = visibility {
        function.visibility = visibility;
        field.visibility = visibility;
        return true;
    }

    match word {
        "final" => function.is_final = true,
        "latent" => function.is_latent = true,
        "entry" => function.is_entry = true,
        "timer" => function.is_timer = true,
        "exec" => function.is_exec = true,
        "quest" => function.is_quest = true,
        "storyscene" => function.is_storyscene = true,
        "reward" => function.is_reward = true,
        "cleanup" => function.is_cleanup = true,
        "native" => {}
        "editable" => field.is_editable = true,
        "saved" => field.is_saved = true,
        "const" => field.is_const = true,
        "inlined" => field.is_inlined = true,
        "optional" => {}
        _ => return false,
    }
    true
}
