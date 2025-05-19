use crate::backend::ttype::Type;
pub(crate) use colored::Colorize;
use core::panic;
use std::{fmt::Debug, process::exit};

use crate::lexer::{Span, Token, TokenType};
#[derive(Debug, Clone)]
pub struct FunctionArg {
    pub name: String,
    pub type_hint: Type,
    pub is_ref: bool,
}
#[derive(Debug, Clone)]
pub enum Node {
    BREAK,
    BinaryExpr {
        lhs: Box<Spanned<Node>>,
        opr: TokenType,
        rhs: Box<Spanned<Node>>,
    },
    StructInit {
        fields: Vec<Spanned<Node>>,
    },
    StructStmt {
        name: String,
        fields: Vec<Spanned<Node>>,
        meths: Vec<Result<Spanned<Node>, String>>,
    },
    BundleAccess {
        base: Box<Spanned<Node>>,
        field: Box<Spanned<Node>>,
    },
    BundleStmt {
        path: String,
        alias: String,
    },
    CONTINUE,
    Cast {
        expr: Box<Spanned<Node>>,
        ty: Type,
    },
    DeRefExpr {
        expr: Box<Spanned<Node>>,
    },
    ExTernStmt {
        name: String,
        args: Vec<FunctionArg>,
        return_type: Type,
        vardaic: bool,
    },
    FcCall {
        params: Vec<Spanned<Node>>,
        callee: Box<Spanned<Node>>,
    },
    Feilds {
        name: String,
        type_hint: Type,
    },
    FnStmt {
        body: Box<Spanned<Node>>,
        args: Vec<FunctionArg>,
        name: String,
        ret_type: Type,
        returns: bool,
        return_val: Box<Option<Spanned<Node>>>,
        vardaic: bool,
        mangled_name: String,
    },
    ForStmt {
        init: Box<Spanned<Node>>,
        cond: Box<Spanned<Node>>,
        inc: Box<Spanned<Node>>,
        body: Box<Spanned<Node>>,
    },
    IfStmt {
        cond: Box<Spanned<Node>>,
        body: Box<Spanned<Node>>,
        elseifs: Option<Vec<Spanned<Node>>>,
        elsestmt: Option<Box<Spanned<Node>>>,
    },
    LetStmt {
        name: String,
        type_hint: Type,
        value: Box<Spanned<Node>>,
    },
    ListAccess {
        name: Box<Spanned<Node>>,
        index: Box<Spanned<Node>>,
    },
    ListInit {
        content: Vec<Box<Spanned<Node>>>,
    },
    GenericFnStmt {
        generics: Vec<String>,
        body: Box<Spanned<Node>>,
        args: Vec<FunctionArg>,
        name: String,
        ret_type: Type,
        returns: bool,
        return_val: Box<Option<Spanned<Node>>>,
        vardaic: bool,
        mangled_name: String,
    },
    LiteralCh(char),
    LiteralInt(String),
    LiteralStr(String),
    MemberAccess {
        base: Box<Spanned<Node>>,
        field: String,
    },
    NameSpace {
        alias: String,
        body: Box<Spanned<Node>>,
    },
    Pair {
        field: String,
        value: Box<Spanned<Node>>,
    },
    PluginStatement {
        name: String,
        ret_val: Box<Option<Spanned<Node>>>,
        ret_type: Box<Type>,
        body: Box<Spanned<Node>>,
        targ_type: Type,
        args: Vec<FunctionArg>,
    },
    GPluginStatement {
        generics: Vec<String>,
        name: String,
        ret_val: Box<Option<Spanned<Node>>>,
        ret_type: Box<Type>,
        body: Box<Spanned<Node>>,
        targ_type: Type,
        args: Vec<FunctionArg>,
    },
    Program(Vec<Spanned<Node>>),
    ReVal {
        name: Box<Spanned<Node>>,
        value: Box<Spanned<Node>>,
    },
    RefExpr {
        expr: Box<Spanned<Node>>,
    },
    Ret(Box<Spanned<Node>>),
    Token(String, bool),
    UnpackStmt {
        alias: String,
        symbols: Vec<String>,
    },
    GenericStructStmt {
        name: String,
        generics: Vec<String>,
        fields: Vec<Spanned<Node>>,
        meths: Vec<Result<Spanned<Node>, String>>,
    },
    GenericFnCall {
        callee: Box<Spanned<Node>>,
        generics: Vec<Type>,
        args: Vec<Spanned<Node>>,
    },
    WhileStmt {
        cond: Box<Spanned<Node>>,
        body: Box<Spanned<Node>>,
    },
}
#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

#[derive(Clone)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    source_code: String,
    has_error: bool,
    is_inloop: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, input: String) -> Self {
        Self {
            tokens,
            current: 0,
            source_code: input,
            has_error: false,
            is_inloop: false,
        }
    }
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }
    fn advance(&mut self) -> Option<&Token> {
        self.current += 1;
        self.peek()
    }
    fn expect_keyword(&mut self, expected: &str) -> Result<(), String> {
        match self.peek().unwrap().kind.clone() {
            TokenType::Keyword(k) if k == expected => {
                self.advance();
                Ok(())
            }
            TokenType::EOF => Err(format!("Expected {:?}, found EOF", self.peek().unwrap())),
            tok => Err(format!("Expected Keyword '{}', got {:?}", expected, tok)),
        }
    }
    pub fn parse_program(&mut self) -> Vec<Spanned<Node>> {
        let mut program = Vec::new();
        while let Some(token) = self.peek() {
            match token.kind.clone() {
                TokenType::Comment(_comment) => {
                    self.advance();
                    continue;
                }
                TokenType::Keyword(k) if k == "for" => {
                    let forstmt = self.parse_forstmt();
                    program.push(forstmt);
                    continue;
                }
                TokenType::Keyword(k) if k == "fn" => {
                    let func = self.parse_func().unwrap();
                    program.push(func);
                    continue;
                }
                TokenType::Keyword(k) if k == "let" => {
                    let var_stmt = self.parse_mk_stmt();
                    program.push(var_stmt);
                    continue;
                }
                TokenType::Keyword(k) if k == "if" => {
                    let ifstmt = self.parse_ifstmt();
                    program.push(ifstmt);
                    continue;
                }
                TokenType::Keyword(k) if k == "extern" => {
                    let extrn_stmt = self.parse_extern();
                    program.push(extrn_stmt);
                    continue;
                }
                TokenType::Keyword(k) if k == "struct" => {
                    let struct_stmt = self.parse_struct();
                    program.push(struct_stmt);
                    continue;
                }
                TokenType::Keyword(k) if k == "unpack" => {
                    let unpack_stmt = self.parse_unpack();
                    program.push(unpack_stmt);
                    continue;
                }
                TokenType::Ident(_val)
                    if self.tokens.get(self.current + 1).unwrap().clone().kind
                        == TokenType::Separator("(".into()) =>
                {
                    let func_call = self.parse_func_call(true);
                    program.push(func_call);
                    continue;
                }
                TokenType::Ident(_val)
                    if self.tokens.get(self.current + 1).unwrap().clone().kind
                        == TokenType::Operator("=".to_owned()) =>
                {
                    let reval_stmt = self.parse_reval();
                    program.push(reval_stmt);
                    continue;
                }
                TokenType::Keyword(k) if k == "bundle" => {
                    let bundle_stmt = self.parse_bundle();
                    program.push(bundle_stmt);
                    continue;
                }
                _ => {
                    if !self
                        .is_expr_start(self.peek().unwrap().clone().kind)
                        .clone()
                    {
                        break;
                    }
                    let expr = self.parse_logic_or();
                    program.push(expr);
                }
            }
            self.advance();
        }
        program
    }
    fn parse_mk_stmt(&mut self) -> Spanned<Node> {
        let start = self.peek().unwrap().clone().span.start;
        self.advance(); // skip mk keyword
        let vname = {
            let name = self.expect_identifier();
            name.unwrap().clone()
        };
        self.expect_separator(":");
        let mut vtype = Type::Any;
        if self.next().kind != TokenType::Operator("=".to_owned()) {
            vtype = self.parse_type().unwrap();
            self.advance();
        }
        self.expect_operator("=");
        let value = { self.parse_logic_or() };
        self.expect_separator(";");
        let end = self.tokens.get(self.current - 1).unwrap().clone().span.end;
        let ast_node = Node::LetStmt {
            name: vname,
            type_hint: vtype,
            value: Box::new(value),
        };
        Spanned {
            node: ast_node,
            span: Span { start, end },
        }
    }
    fn parse_func(&mut self) -> Result<Spanned<Node>, String> {
        let start = self.peek().unwrap().span.start;
        self.advance(); // skip fn keyword
        let fname = {
            let name = self.expect_identifier();
            name.unwrap().clone()
        };

        let mut args = Vec::new();
        let mut vardaic: bool = false;
        if self.next().kind == TokenType::Separator("(".to_owned()) {
            self.expect_separator("(");
            while self.peek().unwrap().clone().kind != TokenType::Separator(')'.to_string()) {
                let mut type_hint = Type::NoType;
                let arg_name: String = self.expect_identifier().unwrap();
                if arg_name == "self" {
                    args.push(FunctionArg {
                        name: arg_name.clone(),
                        type_hint,
                        is_ref: true,
                    });
                    if self.next().kind == TokenType::Separator(",".to_owned()) {
                        self.advance();
                    }
                    continue;
                }
                self.expect_separator(":");
                let mut is_ref = false;
                if self.peek().unwrap().clone().kind == TokenType::Operator('%'.to_string()) {
                    is_ref = true;
                    self.advance();
                }
                type_hint = self.parse_type().unwrap();
                self.advance();
                let arg = FunctionArg {
                    name: arg_name,
                    type_hint,
                    is_ref,
                };
                args.push(arg);
                if self.peek().unwrap().clone().kind == TokenType::Separator(",".to_string()) {
                    self.expect_separator(",");
                    if self.peek().unwrap().clone().kind == TokenType::Vardaic {
                        vardaic = true;
                        self.advance();
                        break;
                    }
                }
            }
            self.expect_separator(")");
        }

        let mut ret_type = Type::NoType;
        if self.peek().unwrap().clone().kind == TokenType::Separator(":".to_owned()) {
            self.expect_separator(":");
            ret_type = self.parse_type().unwrap();
            self.advance();
        }
        self.expect_separator("{");

        let body = self.parse_body(false, false);
        let mut has_ret = false;
        let mut ret_val = None;
        if self.peek().unwrap().clone().kind == TokenType::Keyword("ret".to_string()) {
            self.expect_keyword(&"ret")?;
            has_ret = true;
            ret_val = Some(self.parse_logic_or());
            self.expect_separator(";");
        }
        self.expect_separator("}");
        let end = self.tokens.get(self.current - 1).cloned().unwrap().span.end;
        return Ok(Spanned {
            node: Node::FnStmt {
                name: fname.clone(),
                returns: has_ret,
                ret_type,
                body: Box::new(body),
                return_val: Box::new(ret_val),
                args,
                vardaic,
                mangled_name: fname.clone(),
            },
            span: Span { start, end },
        });
    }
    fn expect_identifier(&mut self) -> Option<String> {
        match self.peek().unwrap().clone().kind {
            TokenType::Ident(ident) => {
                self.advance();
                Some(ident.clone())
            }
            _ => {
                self.error(
                    format!("Expected an identifier, got {}", self.peek().unwrap()),
                    &self.peek().unwrap().clone().span,
                );
                exit(100);
            }
        }
    }
    fn expect_separator(&mut self, expected: &str) {
        match self.peek().unwrap().clone().kind {
            TokenType::Separator(sep) if sep == expected => {
                self.advance();
            }
            _ => {
                self.error(
                    format!("Expected {}, got {}", expected, self.peek().unwrap()),
                    &self.peek().unwrap().clone().span,
                );
                exit(100);
            }
        }
    }
    fn expect_operator(&mut self, expected: &str) {
        match self.peek().unwrap().clone().kind {
            TokenType::Operator(k) if k == expected => {
                self.advance();
            }
            _ => {
                self.error(
                    format!("Expected {} and got {}", expected, self.peek().unwrap()),
                    &self.peek().unwrap().clone().span,
                );
                exit(100);
            }
        }
    }
    fn parse_body(&mut self, take_rets: bool, is_loop: bool) -> Spanned<Node> {
        let start = self.peek().unwrap().clone().span.start;
        let mut stmts = Vec::new();
        loop {
            let token = self.peek();
            match token.unwrap().clone().kind {
                TokenType::Comment(_) => {
                    self.advance();
                }
                TokenType::Keyword(k) if k == "let" => {
                    let m = self.parse_mk_stmt();
                    stmts.push(m);
                    continue;
                }
                TokenType::Keyword(k) if k == "for" => {
                    let forstmt = self.parse_forstmt();
                    stmts.push(forstmt);
                    continue;
                }
                TokenType::Keyword(k) if k == "while" => {
                    let while_stmt = self.parse_while();
                    stmts.push(while_stmt);
                    continue;
                }
                TokenType::Keyword(k) if k == "fn" => {
                    let f = self.parse_func();
                    stmts.push(f.unwrap());
                    continue;
                }
                TokenType::Keyword(k) if k == "break" => {
                    let span = self.next().span;
                    if !self.is_inloop {
                        self.error(format!("Use of break outside of loop body"), &span);
                        exit(1);
                    }
                    self.advance();
                    self.expect_separator(";");
                    stmts.push(Spanned {
                        node: Node::BREAK,
                        span,
                    })
                }
                TokenType::Keyword(k) if k == "continue" => {
                    let span = self.next().span;
                    if !self.is_inloop {
                        self.error(format!("Use of Continue outside of loop body"), &span);
                        exit(1);
                    }
                    self.advance();
                    self.expect_separator(";");
                    stmts.push(Spanned {
                        node: Node::CONTINUE,
                        span,
                    })
                }
                TokenType::Keyword(k) if k == "if" => {
                    let f = self.parse_ifstmt();
                    stmts.push(f);
                    continue;
                }
                TokenType::Ident(val)
                    if self.tokens.get(self.current + 1).unwrap().clone().kind
                        == TokenType::Separator('('.to_string()) =>
                {
                    let func_call = self.parse_func_call(true);
                    stmts.push(func_call);
                    continue;
                }
                TokenType::Ident(val)
                    if self.tokens.get(self.current + 1).unwrap().clone().kind
                        == TokenType::Operator("=".to_owned()) =>
                {
                    let reval_stmt = self.parse_expr();
                    self.expect_separator(";");
                    stmts.push(reval_stmt);
                    continue;
                }
                TokenType::Keyword(k) if k == "ret" => {
                    self.advance();
                    let expr = self.parse_expr();
                    self.expect_separator(";");

                    stmts.push(Spanned {
                        node: Node::Ret(Box::new(expr)),
                        span: Span {
                            start,
                            end: self.before().span.end,
                        },
                    });
                    continue;
                }
                TokenType::EOF => {
                    self.error(
                        format!("Not a valid statment"),
                        &self.peek().unwrap().clone().span,
                    );
                    exit(100)
                }
                _ => {
                    if !self.is_expr_start(self.peek().unwrap().clone().kind) {
                        break;
                    }
                    // debug
                    let expr = self.parse_logic_or();
                    if self.next().kind == TokenType::Separator(';'.to_string()) {
                        self.expect_separator(";");
                    }
                    stmts.push(expr);
                    continue;
                }
            }
        }
        let end = self.tokens.get(self.current - 1).unwrap().clone().span.end;
        Spanned {
            node: Node::Program(stmts),
            span: Span { start, end },
        }
    }
    fn parse_type(&mut self) -> Option<Type> {
        match Some(self.peek().unwrap().clone().kind) {
            Some(TokenType::Ident(custom)) => {
                let first = custom.clone();
                match self.get(1).unwrap().kind {
                    TokenType::DCOLON => {
                        let pos = self.current - 1;
                        self.advance();
                        self.advance();
                        let second = self.parse_type();
                        if matches!(second, None) {
                            return None;
                        }
                        if self.get(1).unwrap().kind == TokenType::Separator("{".to_string())
                            || self.get(1).unwrap().kind == TokenType::Separator(",".to_owned())
                            || (self.get(1).unwrap().kind == TokenType::Operator("=".to_owned())
                                && self.tokens.get(pos).unwrap().kind
                                    == TokenType::Separator(":".to_owned()))
                            || !(self.tokens.get(pos).unwrap().kind
                                == TokenType::Operator("=".to_owned())
                                && self.get(1).unwrap().kind
                                    == TokenType::Separator("(".to_owned()))
                        {
                            return Some(Type::BundledType {
                                bundle: first,
                                ty: Box::new(second.unwrap()),
                            });
                        }
                        if self.before().kind == TokenType::Keyword("for".into()) {
                            return Some(Type::BundledType {
                                bundle: first,
                                ty: Box::new(second.unwrap()),
                            });
                        }
                        None
                    }
                    TokenType::Operator(v) if v == "<" => {
                        self.advance();
                        self.advance();
                        let mut generics = vec![];
                        while self.next().kind != TokenType::Operator(">".to_owned()) {
                            let mut state = self.clone();
                            let t = state.parse_type();
                            if !t.is_some() {
                                generics.push(Type::Custom(self.expect_identifier().unwrap()));
                            } else {
                                generics.push(t.unwrap());
                                self.advance();
                            }
                            if self.next().kind == TokenType::Separator(",".to_owned()) {
                                self.advance();
                            }
                        }
                        if self.next().kind != TokenType::Operator(">".to_owned()) {
                            self.todo_err("Expected closing delim: <");
                        }
                        Some(Type::Generic {
                            base: Box::new(Type::Custom(custom.clone())),
                            generics,
                        })
                    }
                    _ => {
                        if self.get(1).unwrap().kind == TokenType::Separator("{".to_string())
                            || (self.get(1).unwrap().kind == TokenType::Separator(",".to_owned())
                                && self.tokens.get(self.current - 2).unwrap().kind
                                    == TokenType::Separator(":".to_owned()))
                            || (self.get(1).unwrap().kind == TokenType::Operator("=".to_owned())
                                && self.before().kind == TokenType::Separator(":".to_owned()))
                            || (self.get(1).unwrap().kind == TokenType::Separator(")".to_owned())
                                && self.get(2).unwrap().kind
                                    == TokenType::Separator(":".to_owned()))
                            || (self.before().kind == TokenType::DCOLON
                                && (self.get(1).unwrap().kind
                                    != TokenType::Separator("(".to_owned())))
                            || self.get(1).unwrap().kind == TokenType::Operator(">".to_owned())
                            || !(self.get(1).unwrap().kind == TokenType::Separator(";".to_owned()))
                        {
                            return Some(Type::Custom(first));
                        }
                        if self.before().kind == TokenType::Keyword("for".into()) {
                            return Some(Type::Custom(first));
                        }
                        None
                    }
                }
            }
            Some(TokenType::Keyword(t)) if t == "int" => Some(Type::INT),
            Some(TokenType::Keyword(t)) if t == "str" => Some(Type::STR),
            Some(TokenType::Keyword(t)) if t == "i8" => Some(Type::I8),
            Some(TokenType::Keyword(t)) if t == "i16" => Some(Type::I16),
            Some(TokenType::Keyword(t)) if t == "u64" => Some(Type::U64),
            Some(TokenType::Keyword(t)) if t == "i32" => Some(Type::I32),
            Some(TokenType::Keyword(t)) if t == "i64" => Some(Type::I64),
            Some(TokenType::Keyword(t)) if t == "u8" => Some(Type::U8),
            Some(TokenType::Keyword(t)) if t == "u16" => Some(Type::U16),
            Some(TokenType::Keyword(t)) if t == "u32" => Some(Type::U32),
            Some(TokenType::Keyword(t)) if t == "char" => Some(Type::CHAR),
            Some(TokenType::Keyword(t)) if t == "void" => Some(Type::NoType),
            Some(TokenType::Keyword(t)) if t == "list" => {
                self.advance();
                self.expect_operator("<");
                let inner = self.parse_type();
                self.advance();
                self.expect_separator(",");
                if let TokenType::Number(size) = self.next().kind {
                    self.advance();
                    if let TokenType::Operator(_) = self.next().kind {
                        return Some(Type::list(Box::new(inner.unwrap()), size));
                    } else {
                        self.expect_operator(">");
                        exit(100);
                    }
                } else {
                    let span = self.next().span;
                    self.error(format!("Expected list size"), &span);
                    exit(100);
                }
            }
            Some(TokenType::Keyword(t)) if t == "ptr" => {
                self.advance();
                self.expect_operator("<");
                let inner = self.parse_type();
                self.advance();
                if self.next().kind == TokenType::Operator(">".into()) {
                    return Some(Type::PTR(Box::new(inner.unwrap())));
                }
                None
            }

            Some(_) => None,
            None => None,
        }
    }
    fn is_generic_context(&mut self) -> bool {
        let mut depth = 1;
        let mut i = 1;
        while let Some(token) = self.get(i) {
            match token.kind {
                TokenType::Operator(val) if val == ">" => {
                    depth -= 1;
                    if depth == 0 {
                        if let Some(next) = self.get(i + 1) {
                            match next.kind {
                                TokenType::Separator(v) if v == "(" => return true,
                                TokenType::DOT | TokenType::DCOLON => return true,
                                TokenType::Operator(v) if v == "<" => depth += 1,
                                TokenType::Operator(v) if v == "=" => return false,
                                TokenType::Separator(v) if v == ";" || v == "," => return false,
                                _ => return false,
                            }
                        } else {
                            return false;
                        }
                    }
                }
                TokenType::Operator(val) if val == "<" => depth += 1,
                TokenType::Operator(v)
                    if ["+", "-", "*", "/", "%", "==", "!="].contains(&v.as_str()) =>
                {
                    return false;
                }
                TokenType::Separator(v) if [";", "]", ")"].contains(&v.as_str()) => {
                    return false;
                }
                _ => {}
            }
            i += 1;
            if i > 32 {
                return false;
            }
        }
        false
    }
    fn parse_logic_or(&mut self) -> Spanned<Node> {
        let start = self.next().span.start;
        let mut left = self.parse_logic_and();
        if self.next().kind == TokenType::Operator("||".to_owned()) {
            let op = self.next().kind;
            self.advance();
            let right = self.parse_logic_and();
            let end = self.before().span.end;
            left = Spanned {
                node: Node::BinaryExpr {
                    lhs: Box::new(left),
                    opr: op,
                    rhs: Box::new(right),
                },
                span: Span { start, end },
            }
        }
        left
    }
    fn parse_logic_and(&mut self) -> Spanned<Node> {
        let start = self.next().span.start;
        let mut left = self.parse_equality();
        if self.next().kind == TokenType::Operator("&&".to_owned()) {
            let op = self.next().kind;
            self.advance();
            let right = self.parse_equality();
            let end = self.before().span.end;
            left = Spanned {
                node: Node::BinaryExpr {
                    lhs: Box::new(left),
                    opr: op,
                    rhs: Box::new(right),
                },
                span: Span { start, end },
            }
        }
        left
    }
    fn parse_equality(&mut self) -> Spanned<Node> {
        let start = self.next().span.start;
        let mut left = self.parse_comparison();
        if self.next().kind == TokenType::Operator("==".to_owned()) {
            let op = self.next().kind;
            self.advance();
            let right = self.parse_comparison();
            let end = self.before().span.end;
            left = Spanned {
                node: Node::BinaryExpr {
                    lhs: Box::new(left),
                    opr: op,
                    rhs: Box::new(right),
                },
                span: Span { start, end },
            }
        }
        left
    }

    fn parse_comparison(&mut self) -> Spanned<Node> {
        let start = self.next().span.start;
        let mut left = self.parse_expr();
        match self.next().kind {
            TokenType::Operator(v)
                if v == ">" || v == "<" || v == ">=" || v == "<=" || v == "!=" =>
            {
                let op = self.next().kind;
                self.advance();
                let right = self.parse_expr();
                let end = self.before().span.end;
                left = Spanned {
                    node: Node::BinaryExpr {
                        lhs: Box::new(left),
                        opr: op,
                        rhs: Box::new(right),
                    },
                    span: Span { start, end },
                };
                left
            }
            _ => left,
        }
    }
    fn parse_expr(&mut self) -> Spanned<Node> {
        let start = self.peek().unwrap().clone().span.start;
        match self.peek().cloned().unwrap().kind {
            TokenType::Operator(v) if v == "*" => {
                // dereference operation
                self.advance();
                let expr = self.parse_logic_or();
                let end = self.tokens.get(self.current - 1).unwrap().span.end;
                return Spanned {
                    node: Node::DeRefExpr {
                        expr: Box::new(expr),
                    },
                    span: Span { start, end },
                };
            }
            TokenType::Operator(v) if v == "&" => {
                // dereference operation
                self.advance();
                let expr = self.parse_logic_or();
                let end = self.tokens.get(self.current - 1).unwrap().span.end;
                return Spanned {
                    node: Node::RefExpr {
                        expr: Box::new(expr),
                    },
                    span: Span { start, end },
                };
            }

            TokenType::Ident(val) | TokenType::Number(val) => {
                // debug
                let mut left = self.parse_term();
                while matches!(
                    self.peek().cloned().unwrap().kind,
                    TokenType::Operator(v) | TokenType::Operator(v) if v == "+" || v == "-"
                ) {
                    let opr = self.peek().cloned().unwrap().kind;
                    self.advance();
                    let right = self.parse_term();
                    let end = self.tokens.get(self.current - 1).unwrap().span.end;
                    left = Spanned {
                        node: Node::BinaryExpr {
                            lhs: Box::new(left),
                            opr,
                            rhs: Box::new(right),
                        },
                        span: Span { start, end },
                    }
                }
                if self.next().kind == TokenType::Operator('='.to_string()) {
                    self.expect_operator("=");
                    let mut value = self.parse_expr();
                    left = Spanned {
                        node: Node::ReVal {
                            name: Box::new(left.clone()),
                            value: Box::new(value),
                        },
                        span: left.span,
                    }
                } else if self.next().kind == TokenType::Separator('['.to_string()) {
                    self.advance();
                    let index = self.parse_logic_or();
                    left = Spanned {
                        node: Node::ListAccess {
                            name: Box::new(left.clone()),
                            index: Box::new(index),
                        },
                        span: left.span.clone(),
                    };
                    self.expect_separator("]");
                    if self.next().kind == TokenType::Separator('.'.to_string()) {
                        left = self.parse_postfix(left.clone());
                    }
                }

                return left;
            }
            // TokenType::Ident(var) => {
            //     let c = self.parse_term();
            //     return c;
            // }
            TokenType::Separator(sep) if sep == "[" => {
                self.advance();
                let mut contents = Vec::new();
                loop {
                    let expr = self.parse_logic_or();
                    contents.push(Box::new(expr));
                    if self.next().kind != TokenType::Separator(",".to_owned()) {
                        break;
                    } else {
                        self.advance();
                    }
                }
                self.expect_separator("]");
                let end = self.before().span.end;
                Spanned {
                    node: Node::ListInit { content: contents },
                    span: Span { start, end },
                }
            }
            TokenType::Separator(sep) if sep == "(" => {
                let mut left = self.parse_term();

                while matches!(
                    self.peek().cloned().unwrap().kind,
                    TokenType::Operator(v) | TokenType::Operator(v) if v == "+" || v == "-"
                ) {
                    let opr = self.peek().cloned().unwrap().kind;
                    self.advance();
                    let right = self.parse_term();
                    let end = self.tokens.get(self.current - 1).unwrap().span.end;
                    left = Spanned {
                        node: Node::BinaryExpr {
                            lhs: Box::new(left),
                            opr,
                            rhs: Box::new(right),
                        },
                        span: Span { start, end },
                    }
                }
                if self.next().kind == TokenType::Operator('='.to_string()) {
                    self.expect_operator("=");
                    let value = self.parse_logic_or();
                    left = Spanned {
                        node: Node::ReVal {
                            name: Box::new(left.clone()),
                            value: Box::new(value),
                        },
                        span: left.span,
                    }
                }
                return left;
            }
            TokenType::Separator(val) if val == "{".to_owned() => {
                let start = self.peek().cloned().unwrap().span.start;
                self.expect_separator("{");
                let mut fields = Vec::new();
                while self.peek().cloned().unwrap().kind != TokenType::Separator('}'.to_string()) {
                    let pstart = self.peek().cloned().unwrap().span.start;
                    let pname = self.expect_identifier().clone().unwrap();
                    self.expect_separator(":");
                    let pvalue = self.parse_logic_or();
                    if self.next().kind == TokenType::Separator(','.to_string()) {
                        self.expect_separator(",");
                    }
                    let pend = self.before().span.end;
                    fields.push(Spanned {
                        node: Node::Pair {
                            field: pname.clone(),
                            value: Box::new(pvalue),
                        },
                        span: Span {
                            start: pstart,
                            end: pend,
                        },
                    });
                    let pend = self.before().span.end;
                }
                self.expect_separator("}");
                let end = self.before().span.end;
                return Spanned {
                    node: Node::StructInit { fields },
                    span: Span { start, end },
                };
            }
            TokenType::StrLit(var) => self.parse_primary(),
            TokenType::Char(c) => self.parse_primary(),

            _ => {
                self.error(
                    format!("Not an expression {}", self.peek().cloned().unwrap()),
                    &self.peek().cloned().unwrap().span,
                );
                exit(100)
            }
        }
    }
    fn parse_term(&mut self) -> Spanned<Node> {
        let start = self.peek().unwrap().clone().span.start;
        let mut lhs = { self.parse_primary() };
        lhs = self.parse_postfix(lhs.clone());
        while let TokenType::Operator(v) = self.peek().unwrap().clone().kind {
            if v != "*" && v != "/" {
                break;
            }
            let op = self.peek().unwrap().clone();
            self.advance();
            let mut rhs = { self.parse_primary() };
            rhs = self.parse_postfix(rhs.clone());
            let end = self.tokens.get(self.current - 1).unwrap().span.end;
            lhs = Spanned {
                node: Node::BinaryExpr {
                    lhs: Box::new(lhs),
                    opr: op.clone().kind,
                    rhs: Box::new(rhs),
                },
                span: Span { start, end },
            }
        }
        lhs
    }
    fn parse_postfix(&mut self, mut expr: Spanned<Node>) -> Spanned<Node> {
        let start = self.peek().cloned().unwrap().span.start;
        loop {
            match self.peek().cloned().unwrap().kind {
                TokenType::DOT => {
                    self.advance();
                    let field = self.expect_identifier();

                    let end = self.tokens.get(self.current - 1).cloned().unwrap().span.end;
                    expr = Spanned {
                        node: Node::MemberAccess {
                            base: Box::new(expr),
                            field: field.unwrap(),
                        },
                        span: Span { start, end },
                    };
                    continue;
                }
                TokenType::Separator(v) if v == "(" => {
                    self.expect_separator("(");
                    let args: Vec<Spanned<Node>> = self.parse_args();
                    self.expect_separator(")");
                    let end = self.tokens.get(self.current - 1).cloned().unwrap().span.end;
                    expr = Spanned {
                        node: Node::FcCall {
                            callee: Box::new(expr),
                            params: args,
                        },
                        span: Span { start, end },
                    };
                    continue;
                }
                TokenType::DCOLON => {
                    self.advance();
                    let field = self.parse_logic_or();
                    let end = self.tokens.get(self.current - 1).cloned().unwrap().span.end;
                    expr = Spanned {
                        node: Node::BundleAccess {
                            base: Box::new(expr),
                            field: Box::new(field),
                        },
                        span: Span { start, end },
                    };
                }
                TokenType::Separator(sep) if sep == "[" => {
                    self.advance();
                    let index = self.parse_logic_or();
                    self.expect_separator("]");
                    let end = self.before().span.end;
                    expr = Spanned {
                        node: Node::ListAccess {
                            name: Box::new(expr),
                            index: Box::new(index),
                        },
                        span: Span { start, end },
                    }
                }
                TokenType::Operator(v) if v == "<" => {
                    if !self.is_generic_context() {
                        return expr;
                    }
                    self.advance();
                    let mut g = vec![];
                    while self.next().kind != TokenType::Operator(">".to_owned()) {
                        let t = self.parse_type();
                        if t.is_none() {
                            break;
                        }
                        g.push(t.unwrap());
                        self.advance();
                        if self.next().kind == TokenType::Separator(",".to_owned()) {
                            self.advance();
                        }
                    }
                    self.expect_operator(">");
                    self.expect_separator("(");
                    let a = self.parse_args();
                    self.expect_separator(")");
                    let end = self.before().span.end;
                    expr = Spanned {
                        node: Node::GenericFnCall {
                            callee: Box::new(expr),
                            generics: g,
                            args: a,
                        },
                        span: Span { start, end },
                    };
                }
                TokenType::Keyword(v) if v == "as" => {
                    self.advance();
                    let ty = self.parse_type().unwrap();
                    self.advance();
                    let end = self.before().span.end;
                    expr = Spanned {
                        node: Node::Cast {
                            expr: Box::new(expr),
                            ty,
                        },
                        span: Span { start, end },
                    }
                }
                _ => break,
            }
        }
        let end = self.tokens.get(self.current - 1).cloned().unwrap().span.end;
        expr
    }
    fn parse_primary(&mut self) -> Spanned<Node> {
        let start = self.peek().unwrap().clone().span.start;
        let mut is_deref = false;
        let base = match self.peek().cloned().unwrap().kind {
            TokenType::Number(val) => {
                self.advance();
                let end = self.tokens.get(self.current - 1).unwrap().clone().span.end;
                Spanned {
                    node: Node::LiteralInt(val),
                    span: Span { start, end },
                }
            }
            TokenType::StrLit(val) => {
                self.advance();
                let end = self.tokens.get(self.current - 1).unwrap().clone().span.end;
                Spanned {
                    node: Node::LiteralStr(val),
                    span: Span { start, end },
                }
            }
            TokenType::Char(c) => {
                self.advance();
                let end = self.before().span.end;
                Spanned {
                    node: Node::LiteralCh(c),
                    span: Span { start, end },
                }
            }
            TokenType::Ident(ident)
                if self.tokens.get(self.current + 1).unwrap().clone().kind == TokenType::DOT =>
            {
                self.advance();
                let end = self.tokens.get(self.current - 1).unwrap().clone().span.end;
                Spanned {
                    node: Node::Token(ident, is_deref),
                    span: Span { start, end },
                }
            }
            TokenType::Ident(ident) => {
                self.advance();
                let end = self.tokens.get(self.current - 1).unwrap().clone().span.end;
                Spanned {
                    node: Node::Token(ident, is_deref),
                    span: Span { start, end },
                }
            }
            TokenType::Separator(val) if val == "(" => {
                self.advance();
                let m = self.parse_logic_or();
                if self.peek().unwrap().clone().kind != TokenType::Separator(")".to_owned()) {
                    panic!("Expected Matching Closing Parethesis");
                }
                self.advance(); // skip closing braket
                m
            }

            _ => {
                let tok = self.next().kind;
                if self.is_expr_start(tok) {
                    return self.parse_logic_or();
                }
                self.error(
                    format!("Expected an Term"),
                    &self.peek().unwrap().clone().span,
                );
                exit(100)
            }
        };
        self.parse_postfix(base)
    }

    fn parse_func_call(&mut self, check_semi: bool) -> Spanned<Node> {
        let start = self.peek().unwrap().clone().span.start;
        let callee = self.parse_logic_or();
        // debug
        if self.next().kind == TokenType::Separator(')'.to_string()) {
            self.expect_separator(")");
        }
        if check_semi {
            self.expect_separator(";");
        }
        let end = self.tokens.get(self.current - 1).unwrap().clone().span.end;
        callee
    }

    fn parse_extern(&mut self) -> Spanned<Node> {
        let start = self.peek().unwrap().span.start;
        self.advance(); // skip the keyword
        let name = self.expect_identifier().unwrap().clone();
        self.expect_separator("(");
        let mut args = Vec::new();
        let mut vardaic = false;
        while self.peek().unwrap().kind.clone() != TokenType::Separator(')'.to_string()) {
            let arg_name: String = {
                match self.peek().cloned().unwrap().kind {
                    TokenType::Ident(val) => {
                        self.advance();
                        val.clone()
                    }
                    _ => {
                        self.error(
                            format!(
                                "Expected an Identifier, got {:?}",
                                self.peek().cloned().unwrap().kind.clone()
                            ),
                            &self.peek().unwrap().span.clone(),
                        );
                        exit(100);
                    }
                }
            };
            self.expect_separator(":");
            let mut is_ref = false;
            if self.peek().unwrap().clone().kind == TokenType::Operator('%'.to_string()) {
                is_ref = true;
                self.advance();
            }
            let type_hint = self.parse_type();
            self.advance();
            let arg = FunctionArg {
                name: arg_name,
                type_hint: type_hint.unwrap(),
                is_ref,
            };
            args.push(arg);
            if self.peek().unwrap().clone().kind == TokenType::Separator(','.to_string()) {
                self.expect_separator(",");
                if self.peek().unwrap().clone().kind == TokenType::Vardaic {
                    self.advance();
                    vardaic = true;
                    break;
                }
            }
        }
        self.expect_separator(")");
        self.expect_separator(":");
        let type_hint = self.parse_type();
        self.advance();
        self.expect_separator(";");
        let end = self.tokens.get(self.current - 1).cloned().unwrap().span.end;
        Spanned {
            node: Node::ExTernStmt {
                name,
                args,
                return_type: type_hint.unwrap(),
                vardaic,
            },
            span: Span { start, end },
        }
    }

    fn parse_reval(&mut self) -> Spanned<Node> {
        let start = self.peek().unwrap().clone().span.start;
        let name = self.parse_expr();
        self.expect_operator("=");
        let value = self.parse_expr();
        self.expect_separator(";");
        let end = self.tokens.get(self.current - 1).unwrap().clone().span.end;
        Spanned {
            node: Node::ReVal {
                name: Box::new(name),
                value: Box::new(value),
            },
            span: Span { start, end },
        }
    }

    fn is_expr_start(&self, arg: TokenType) -> bool {
        match arg.clone() {
            TokenType::Number(_num) => true,
            TokenType::Operator(sep) if "*" == sep || "&" == sep => true,
            TokenType::Ident(val) => true,
            _ => false,
        }
    }
    fn error(&mut self, err_msg: String, spanned: &Span) {
        self.has_error = true;
        let span = spanned.clone();
        let source = self.source_code.clone();
        let line_start = source[..span.start].rfind('\n').map_or(0, |i| i + 1);
        let line_end = source[span.start..]
            .find('\n')
            .map(|i| span.start + i)
            .unwrap_or(source.len());
        let line = &source[line_start..line_end];
        let col_start = span.start - line_start;
        let col_end = span.end - line_start;
        let line_number = source[..span.start].chars().filter(|&c| c == '\n').count() + 1;
        eprintln!("{}", format!("[Tixie Error]: {err_msg}").red().bold());
        eprintln!(" ---> line : {line_number}");
        eprintln!("   |\n{: >2} | {}", line_number, line);
        let mut caret_line = String::new();
        for (i, c) in line.chars().enumerate() {
            if i >= col_start && i < col_end {
                caret_line.push('^');
            } else if i < col_start {
                caret_line.push(if c == '\t' { '\t' } else { ' ' });
            }
        }
        eprintln!("   | {}", format!("{caret_line}").as_str().yellow().bold());
    }

    fn warn<T: std::fmt::Debug>(&mut self, err_msg: String, spanned: &Spanned<T>) {
        let span = spanned.span.clone();
        let source = self.source_code.clone();
        let line_start = source[..span.start].rfind('\n').map_or(0, |i| i + 1);
        let line_end = source[span.start..]
            .find('\n')
            .map(|i| span.start + i)
            .unwrap_or(source.len());
        let line = &source[line_start..line_end];
        let col_start = span.start - line_start;
        let col_end = span.end - line_start;
        let line_number = source[..span.start].chars().filter(|&c| c == '\n').count() + 1;
        eprintln!("{}", format!("[Tixie Warning]: {err_msg}").yellow().bold());
        eprintln!(" ---> line : {line_number}");
        eprintln!("   |\n{: >2} | {}", line_number, line);
        let mut caret_line = String::new();
        for (i, c) in line.chars().enumerate() {
            if i >= col_start && i < col_end {
                caret_line.push('^');
            } else if i < col_start {
                caret_line.push(if c == '\t' { '\t' } else { ' ' });
            }
        }
        eprintln!("   | {}", format!("{caret_line}").as_str().yellow().bold());
    }

    fn parse_struct(&mut self) -> Spanned<Node> {
        let start = self.peek().unwrap().clone().span.start;
        self.advance(); // skip 'struct'
        let name = self.expect_identifier().unwrap().clone();

        self.expect_separator("{");
        let mut fields = Vec::new();
        let mut meths = Vec::new();
        while self.peek().unwrap().clone().kind != TokenType::Separator('}'.to_string()) {
            if self.next().kind == TokenType::Keyword("fn".to_owned()) {
                let method = self.parse_func();
                meths.push(method);
                continue;
            }
            let fe_start = self.peek().unwrap().clone().span.start;
            let feild_name = self.expect_identifier().unwrap();
            self.expect_separator(":");
            let fstart = self.peek().unwrap().clone().span.start;
            let feild_type = self.parse_type();
            let fend = self.tokens.get(self.current - 1).unwrap().clone().span.end;
            self.advance();
            if self.peek().unwrap().clone().kind == TokenType::Separator(','.to_string()) {
                self.expect_separator(",");
            } else if matches!(self.peek().unwrap().clone().kind, TokenType::Separator(v) if v == "}")
            {
                self.error(
                    format!("Missing comma?"),
                    &self.peek().unwrap().clone().span,
                );
            }
            let fe_end = self.tokens.get(self.current - 1).unwrap().clone().span.end;
            fields.push(Spanned {
                node: Node::Feilds {
                    name: feild_name,
                    type_hint: feild_type.unwrap(),
                },
                span: Span {
                    start: fe_start,
                    end: fe_end,
                },
            });
        }
        self.expect_separator("}");
        let node: Node = Node::StructStmt {
            name,
            fields,
            meths,
        };
        let end = self.tokens.get(self.current - 1).unwrap().clone().span.end;
        return Spanned {
            node,
            span: Span { start, end },
        };
    }

    fn parse_args(&mut self) -> Vec<Spanned<Node>> {
        let mut args = Vec::new();
        let start = self.peek().cloned().unwrap().span.start;
        if self.next().kind == TokenType::Separator(')'.to_string()) {
            return args;
        }
        loop {
            let arg = self.parse_logic_or();
            if self.peek().cloned().unwrap().kind == TokenType::Separator(",".to_string()) {
                self.expect_separator(",");
            } else if self.peek().cloned().unwrap().kind == TokenType::Separator(')'.to_string()) {
                args.push(arg);
                break;
            }
            args.push(arg);
        }
        let end = self.tokens.get(self.current - 1).unwrap().clone().span.end;
        args
    }
    fn before(&mut self) -> Token {
        return self.tokens.clone().get(self.current - 1).cloned().unwrap();
    }
    fn next(&mut self) -> Token {
        return self.peek().cloned().unwrap();
    }

    fn parse_bundle(&mut self) -> Spanned<Node> {
        let start = self.next().span.start;
        self.advance(); // skip 'bundle'
        let mut bpath = String::new();
        let mut balais = String::new();
        if let TokenType::StrLit(path) = self.peek().cloned().unwrap().kind {
            bpath = path;
            self.advance();
            self.expect_keyword("as").unwrap();
            balais = self.expect_identifier().unwrap();
            let end = self.before().span.end;
            self.expect_separator(";");
            return Spanned {
                node: Node::BundleStmt {
                    path: bpath,
                    alias: balais,
                },
                span: Span { start, end },
            };
        }
        balais = self.expect_identifier().unwrap();
        self.expect_separator("{");
        let body: Spanned<Node> = self.parse_bundle_body();
        self.expect_separator("}");
        let end = self.before().span.end;
        Spanned {
            node: Node::NameSpace {
                alias: balais,
                body: Box::new(body),
            },
            span: Span { start, end },
        }
    }

    fn get(&self, arg: i32) -> Option<Token> {
        if (self.current + arg as usize) < self.tokens.len() {
            Some(
                self.tokens
                    .get(self.current + arg as usize)
                    .unwrap()
                    .clone(),
            )
        } else {
            None
        }
    }

    fn is_type_ahead(&self) -> bool {
        let mut state = self.clone();
        if matches!(state.parse_type(), None) {
            return false;
        }
        true
    }

    fn parse_bundle_body(&mut self) -> Spanned<Node> {
        let start = self.next().span.start;
        let mut stmts = Vec::new();
        loop {
            match self.next().kind {
                TokenType::Comment(_) => {
                    self.advance();
                }
                TokenType::Keyword(k) if k == "let" => {
                    let m = self.parse_mk_stmt();
                    stmts.push(m);
                    continue;
                }
                TokenType::Keyword(k) if k == "fn" => {
                    let f = self.parse_func();
                    stmts.push(f.unwrap());
                    continue;
                }
                TokenType::Ident(val)
                    if self.tokens.get(self.current + 1).unwrap().clone().kind
                        == TokenType::Separator('('.to_string()) =>
                {
                    println!(
                        "Todo Err system: Cannot have a function call in a none executable scope"
                    );
                    exit(100);
                }
                TokenType::Keyword(val) if val == "struct" => {
                    let struct_stmt = self.parse_struct();
                    stmts.push(struct_stmt);
                    continue;
                }
                TokenType::Ident(val)
                    if self.tokens.get(self.current + 1).unwrap().clone().kind
                        == TokenType::Operator('='.to_string()) =>
                {
                    self.todo_err("Cannot have a re-assignment in a non exectable scope");
                }
                TokenType::EOF => {
                    self.error(
                        format!("Not a valid statment"),
                        &self.peek().unwrap().clone().span,
                    );
                    exit(100)
                }
                _ => {
                    break;
                }
            }
        }
        let end = self.tokens.get(self.current - 1).unwrap().clone().span.end;
        Spanned {
            node: Node::Program(stmts),
            span: Span { start, end },
        }
    }
    fn todo_err(&self, arg: &str) {
        println!("Todo Err system: {arg}");
        exit(100);
    }

    fn parse_ifstmt(&mut self) -> Spanned<Node> {
        let start = self.next().span.start;
        self.advance(); // skip if keyword
        let cond = self.parse_logic_or();
        self.expect_separator("{");
        let body = self.parse_body(true, false);
        self.expect_separator("}");
        let mut branches = vec![];
        if self.next().kind == TokenType::Keyword("else".to_owned()) {
            while self.next().kind == TokenType::Keyword("else".to_owned())
                && self.get(1).unwrap().kind == TokenType::Keyword("if".to_owned())
            {
                let _ = self.expect_keyword("else");
                branches.push(self.parse_ifstmt());
            }
        }
        let mut else_body = None;
        if self.next().kind == TokenType::Keyword("else".to_owned()) {
            self.advance();
            self.expect_separator("{");
            else_body = Some(Box::new(self.parse_body(false, false)));
            self.expect_separator("}");
        }
        let end = self.before().span.end;
        Spanned {
            node: Node::IfStmt {
                cond: Box::new(cond),
                body: Box::new(body),
                elseifs: Some(branches),
                elsestmt: else_body,
            },
            span: Span { start, end },
        }
    }

    fn parse_forstmt(&mut self) -> Spanned<Node> {
        self.is_inloop = true;
        let start = self.next().span.start;
        self.advance();
        self.expect_separator("(");
        let init = self.parse_expr();
        self.expect_separator(";");
        let cond = self.parse_logic_or();
        self.expect_separator(";");
        let inc = self.parse_logic_or();
        self.expect_separator(")");
        self.expect_separator("{");
        let body = self.parse_body(true, true);
        self.expect_separator("}");
        let end = self.before().span.end;
        self.is_inloop = false;
        Spanned {
            node: Node::ForStmt {
                init: Box::new(init),
                cond: Box::new(cond),
                inc: Box::new(inc),
                body: Box::new(body),
            },
            span: Span { start, end },
        }
    }

    fn parse_unpack(&mut self) -> Spanned<Node> {
        let start = self.next().span.start;
        self.advance();
        let mut symbols = vec![];
        let alias = self.expect_identifier().unwrap();
        self.expect_separator("{");
        while self.next().kind != TokenType::Separator("}".to_owned()) {
            symbols.push(self.expect_identifier().unwrap());
            if self.next().kind == TokenType::Separator(",".to_owned()) {
                self.expect_separator(",");
            }
        }
        self.expect_separator("}");
        self.expect_separator(";");
        let end = self.before().span.end;
        Spanned {
            node: Node::UnpackStmt { alias, symbols },
            span: Span { start, end },
        }
    }

    fn parse_while(&mut self) -> Spanned<Node> {
        let start = self.next().span.start;
        self.advance();
        let cond = self.parse_logic_or();
        self.expect_separator("{");
        let body = self.parse_body(false, true);
        self.expect_separator("}");
        let end = self.next().span.end;
        Spanned {
            node: Node::WhileStmt {
                cond: Box::new(cond),
                body: Box::new(body),
            },
            span: Span { start, end },
        }
    }
}
