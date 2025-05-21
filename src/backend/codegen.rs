use crate::backend::var;
use crate::lexer::{self, Span, TokenType};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{self, Path, PathBuf};
use std::process::exit;
use std::str::FromStr;

use colored::Colorize;

use crate::parser::{FunctionArg, Node, Parser, Spanned};

use super::bundle::Bundle;
use super::compile_error::{CompileError, ErrLevel};
use super::context::Context;
use super::function::Function;
use super::ttype::Type;
use super::type_table::TTable;
use super::var::Var;
use super::var_table::VTable;

#[derive(Debug, Clone)]
struct FResult {
    stream: String,
    type_hint: Type,
    preamble: String,
}
#[derive(Debug, Clone)]
pub(crate) struct FieldLayout {
    ty: Type,
}
#[derive(Debug, Clone)]
pub struct StructLayout {
    pub name: Type,
    pub feilds: HashMap<String, FieldLayout>,
    pub methods: Vec<Function>,
    pub file: String,
}
#[derive(Debug, Clone)]
enum RefStyle {
    DEREF,
    REF,
    COPY,
}
#[derive(Debug, Clone)]
struct ExprResult {
    preamble: String,
    stream: String,
    is_ref: bool,
    refed_var: Option<String>,
    type_hint: Box<Type>,
    var: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Section {
    TEXT,
    FUNC,
    HEADER,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BodyType {
    GENERIC,
    NORMAL,
}

pub struct Generator {
    bss: String,
    pub(crate) buildpath: Box<PathBuf>,
    bundled: Vec<String>,
    bundles: Vec<Bundle>,
    cur_body_type: BodyType,
    cur_section: Section,
    current_context: Context,
    current_file: String,
    current_scope_return_type: Type,
    data: String,
    errorbox: Vec<CompileError>,
    func: String,
    func_table: Vec<Function>,
    global_context: Context,
    has_error: bool,
    header: String,
    immediate_counter: u8,
    inputpath: String,
    is_included: bool,
    pub(crate) outfilename: String,
    pub(crate) outfilepath: String,
    pub source: Vec<Spanned<Node>>,
    source_code: String,
    pub text: String,
    track_rsp: bool,
    types: TTable,
    var_table: VTable, // Validating return statements
}

impl Generator {
    pub fn new(
        source: Vec<Spanned<Node>>,
        path: &str,
        input: String,
        is_included: bool,
        inputpath: String,
        current_file: String,
        builddir: String,
    ) -> Self {
        /* global context for global variables */
        let global_context = Context::new("global".into(), None);
        /* base types: int, str, u8, ... , i64 */
        let base = TTable::new();
        Generator {
            cur_body_type: BodyType::NORMAL,
            source,
            outfilepath: path.to_string(),
            bss: String::new(),
            data: String::new(),
            text: String::new(),
            func: String::new(),
            header: String::new(),
            immediate_counter: 0,
            func_table: Vec::new(),
            var_table: VTable::new(),
            cur_section: Section::TEXT,
            track_rsp: false,
            errorbox: vec![],
            outfilename: Path::new(path)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            source_code: input.clone(),
            has_error: false,
            types: base,
            global_context: global_context.clone(),
            current_context: global_context,
            bundles: Vec::new(),
            is_included,
            inputpath,
            current_file,
            bundled: vec![],
            buildpath: Box::new(Path::new(&builddir).to_path_buf()),
            current_scope_return_type: Type::NoType,
        }
    }
    pub fn change_scope(&mut self, name: &str) {
        let context = Context::new(
            name.to_string(),
            Some(Box::new(self.current_context.clone())),
        );
        self.current_context = context;
    }
    pub fn exit_scope(&mut self) {
        let mut context = self.current_context.clone();
        let prev_context = context.get_parent();
        if !matches!(prev_context, None) {
            self.current_context = prev_context.unwrap();
            return;
        }
        self.current_context = self.global_context.clone();
    }
    pub fn init(&mut self) {
        self.cur_section = Section::HEADER;
        if self.is_included {
            self.emit("#pragma once\n");
        }
        /* Path to Runtime and generic helper header */
        self.emit(&format!(
            "\n#include \"/home/dry/Documents/Eggo/jaguar/std/claw.h\"",
        ));
        self.cur_section = Section::TEXT;
    }
    pub fn generate(&mut self, source: Vec<Spanned<Node>>) {
        for node in source.clone() {
            match node.clone().node {
                Node::LetStmt {
                    name: _,
                    type_hint,
                    value: _,
                    is_mut: _,
                } => {
                    let o = self.gen_expr(Box::new(node), type_hint, RefStyle::COPY);
                    self.emit(&format!("\n{};", o.stream));
                }
                Node::DeRefExpr { expr: _ } => {
                    let s = self.gen_expr(Box::new(node), Type::Any, RefStyle::DEREF);
                    self.emit(&format!("\n\t{};", s.stream));
                }
                Node::RefExpr { expr: _ } => {
                    let s = self.gen_expr(Box::new(node), Type::Any, RefStyle::REF);
                    self.emit(&format!("\n\t{};", s.stream));
                }

                Node::BundleAccess { base: _, field: _ } => {
                    let out = self.gen_expr(Box::new(node), Type::Any, RefStyle::COPY);
                    self.emit("\n\t");
                    self.emit(&out.clone().stream);
                    self.emit(";");
                }
                Node::BundleStmt { path, alias } => {
                    /* append var:path to the parent path of the main file */
                    let p_import_path = Path::new(&self.inputpath)
                        .parent()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();
                    let import_path = format!("{}/{}", p_import_path, path);
                    if !Path::exists(Path::new(import_path.as_str())) {
                        self.consume(CompileError::new(
                            format!("Could not resolve {}", path),
                            Some("Try Confirming Bundle Path or alias".to_string()),
                            node.span,
                            ErrLevel::ERROR,
                        ));
                        self.flush();
                        exit(1);
                    }
                    let source = std::fs::read_to_string(import_path.clone()).unwrap();
                    #[allow(unused_assignments)]
                    let mut output = import_path.clone();
                    let mut tokenizer = lexer::Tokenizer::new(&source);
                    let mut tokens = Vec::new();
                    loop {
                        let tok = tokenizer.next_token();
                        if let TokenType::Comment(_) = tok.kind {
                            continue;
                        }
                        tokens.push(tok.clone());
                        if tok.kind == lexer::TokenType::EOF {
                            break;
                        }
                    }
                    let mut parser = Parser::new(tokens, source.clone());
                    let final_output = format!("{}.h", import_path);
                    let p = std::path::Path::new(&import_path)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();
                    output = format!("{}/{}.h", self.buildpath.to_str().unwrap(), p);
                    let mut found = false;
                    for mut b in self.bundles.clone() {
                        if b.refuse_dup(output.clone()).is_some() {
                            let mut nb = b.refuse_dup(output.clone()).unwrap();
                            nb.name = alias.clone();
                            self.bundles.push(nb);
                            found = true;
                        }
                    }
                    if found {
                        continue;
                    }
                    match parser.parse_program() {
                        program => {
                            let mut cgen = Generator::new(
                                program,
                                &output,
                                source,
                                true,
                                path.clone(),
                                std::fs::canonicalize(path::Path::new(&import_path))
                                    .ok()
                                    .map(|p| p.to_string_lossy().to_string())
                                    .unwrap(),
                                self.buildpath.to_str().unwrap().to_string(),
                            );
                            cgen.init();
                            cgen.generate(cgen.source.clone());
                            cgen.rest();
                            self.bundled.append(&mut cgen.bundled.clone());
                            let mut new_bundle = Bundle::new(
                                alias.clone(),
                                cgen.var_table,
                                cgen.func_table,
                                cgen.types,
                                cgen.bundles,
                                final_output.clone(),
                            );
                            new_bundle.types.wrap(alias.clone());
                            new_bundle.wrap(&alias.clone());
                            self.bundles.push(new_bundle);
                            let sv = self.cur_section.clone();
                            self.cur_section = Section::HEADER;
                            self.emit(format!("\n#include \"{output}\"").as_str());
                            self.cur_section = sv;
                            self.bundled.push(output);
                        }
                    };
                }
                Node::MemberAccess { base: _, field: _ } => {
                    let out = self.gen_expr(Box::new(node), Type::Any, RefStyle::COPY);
                    self.emit("\n\t");
                    self.emit(&out.stream);
                    self.emit(";");
                }

                Node::StructStmt {
                    name,
                    fields,
                    meths,
                    statics,
                } => {
                    let layout: StructLayout;
                    let mut b_fields: HashMap<String, FieldLayout> = HashMap::new();
                    let save = self.cur_section.clone();
                    self.cur_section = Section::HEADER;
                    self.emit(format!("\ntypedef struct {name} {{\n").as_str());

                    let mut encountered_fields: Vec<(String, Span)> = vec![];
                    for (_i, field) in fields.clone().iter().enumerate() {
                        if let Node::Feilds {
                            name: fname,
                            type_hint,
                        } = field.node.clone()
                        {
                            if self.get_layout(type_hint.clone()).is_some() {
                                self.emit(format!("\n\t{} {fname};", type_hint.to_str()).as_str());
                                b_fields.insert(fname.clone(), FieldLayout { ty: type_hint });
                                encountered_fields.push((fname, field.span.clone()));
                            } else {
                                self.consume(CompileError::new(
                                    format!("Not a type, {}", type_hint.debug()),
                                    None,
                                    field.span.clone(),
                                    ErrLevel::ERROR,
                                ));
                                self.flush();
                            }
                        }
                    }
                    layout = StructLayout {
                        name: Type::Custom(name.clone()),
                        feilds: b_fields.clone(),
                        methods: Vec::new(),
                        file: self.current_file.clone(),
                    };
                    self.types.add_type(Type::Custom(name.clone()), layout);
                    self.emit(format!("}} {name};").as_str());
                    for m in meths.clone() {
                        if let Spanned {
                            node:
                                Node::FnStmt {
                                    body,
                                    args,
                                    name: fname,
                                    ret_type,
                                    returns,
                                    return_val: _,
                                    vardaic: _,
                                    mangled_name: _,
                                },
                            span: _,
                        } = m.clone().unwrap()
                        {
                            let targ_type = Type::Custom(name.clone());
                            let layout = self.get_layout(targ_type.clone());
                            if matches!(layout, None) {
                                println!(
                                    "ToDo Err system: Not a type {}",
                                    targ_type.debug().clone()
                                );
                                exit(1);
                            }
                            match layout
                                .as_ref()
                                .unwrap()
                                .clone()
                                .methods
                                .clone()
                                .iter()
                                .find(|m| m.get_name() == fname.clone())
                            {
                                Some(_plug) => {
                                    println!(
                                        "ToDo Err system: Plugin {} already exsist for type {}",
                                        fname.clone(),
                                        targ_type.debug().clone()
                                    );
                                    exit(1);
                                }
                                None => (),
                            }
                            let mut context = Context::new(
                                name.clone(),
                                Some(Box::new(self.current_context.clone())),
                            );
                            self.emit(&format!(
                                "\nextern inline {} {}_{}(",
                                ret_type.clone().c_impl(),
                                targ_type.c_impl(),
                                fname.clone()
                            ));
                            for (i, a) in args.clone().iter_mut().enumerate() {
                                let mut modif = "";
                                if a.name == "self" {
                                    if a.type_hint == Type::NoType {
                                        modif = "*";
                                        a.type_hint = targ_type.clone();
                                    }
                                }
                                self.emit(&format!(
                                    "{}{modif} {}",
                                    a.type_hint.clone().to_str(),
                                    a.name.clone()
                                ));
                                if i != args.clone().len() - 1 {
                                    self.emit(",");
                                }
                                context.add(Var::new(
                                    a.name.clone(),
                                    a.type_hint.clone(),
                                    a.is_ref,
                                    None,
                                    m.clone().unwrap().span,
                                ));
                            }
                            self.emit(");");
                            let mut f = Function::new(
                                fname.clone(),
                                context,
                                ret_type.clone(),
                                returns,
                                body.clone().node,
                            );
                            f.args = args;
                            self.register_plugin(Type::Custom(name.clone()), f);
                        }
                    }
                    for meth in meths.clone() {
                        if let Spanned {
                            node:
                                Node::FnStmt {
                                    body,
                                    args,
                                    name: fname,
                                    ret_type,
                                    returns: _,
                                    return_val,
                                    vardaic: _,
                                    mangled_name: _,
                                },
                            span,
                        } = meth.clone().unwrap().clone()
                        {
                            let p = Node::PluginStatement {
                                name: fname.clone(),
                                ret_val: return_val,
                                ret_type: Box::new(ret_type.clone()),
                                body: body.clone(),
                                targ_type: Type::Custom(name.clone()),
                                args: args.clone(),
                            };

                            self.generate(vec![Spanned { node: p, span }]);
                        }
                    }
                    if statics.is_some() {
                        let functions = statics.unwrap();
                        let body = self.convert_vecnode_nodeprogram(functions);
                        let type_bundle = Node::NameSpace { alias: name, body };
                        let o = self.gen_expr(
                            Box::new(Spanned {
                                node: type_bundle,
                                span: Span { start: 0, end: 0 },
                            }),
                            Type::Any,
                            RefStyle::COPY,
                        );
                        self.emit(&o.stream);
                    }
                    self.cur_section = save;
                }

                Node::ReVal { name: _, value: _ } => {
                    let s = self.gen_expr(Box::new(node), Type::Any, RefStyle::COPY);
                    self.emit(&format!("\n\t{};", s.stream));
                }
                Node::IfStmt {
                    cond: _,
                    body: _,
                    elseifs: _,
                    elsestmt: _,
                } => {
                    let i = self.gen_expr(Box::new(node), Type::Any, RefStyle::COPY);
                    self.emit(&i.stream);
                }
                Node::BREAK => {
                    self.emit(&format!("break;"));
                }
                Node::CONTINUE => {
                    self.emit("\ncontinue;");
                }

                Node::BinaryExpr {
                    lhs: _,
                    opr: _,
                    rhs: _,
                } => {}
                Node::ExTernStmt {
                    name,
                    args: params,
                    return_type,
                    vardaic,
                } => {
                    let mut stream = String::new();
                    let mut context = Context::new(name.clone(), None);
                    stream += format!("\nextern {} {name} (", return_type.to_str()).as_str();
                    for (i, arg) in params.clone().iter().enumerate() {
                        let v = Var::new(
                            arg.clone().name,
                            arg.type_hint.clone(),
                            false,
                            None,
                            node.clone().span,
                        );
                        stream +=
                            format!("{} {}", arg.type_hint.clone().to_str(), arg.name.clone())
                                .as_str();
                        context.add(v);
                        if i != (params.clone().len() - 1 as usize) as usize {
                            stream += ",";
                        }
                    }
                    if vardaic {
                        stream += ", ...";
                    }
                    stream += ");";
                    let mut extrnfunc = Function::new(
                        name.clone(),
                        context,
                        return_type,
                        true,
                        Node::Program(Vec::new()),
                    );
                    extrnfunc.gen_name = extrnfunc.name.clone();
                    extrnfunc.args = params;
                    extrnfunc.variadic = vardaic;
                    self.func_table.push(extrnfunc);
                    self.cur_section = Section::HEADER;
                    self.emit(&stream);
                    self.cur_section = Section::TEXT;
                    continue;
                }
                Node::FcCall {
                    params: _,
                    callee: _,
                } => {
                    self.emit("\n\t");
                    let out = self.gen_func_call(Box::new(node.clone()), Type::Any);
                    self.emit(&format!("{}\n{}", out.preamble, out.stream));
                    self.emit(";");
                }
                Node::FnStmt {
                    body: _,
                    args: _,
                    name: _,
                    ret_type: _,
                    returns: _,
                    return_val: _,
                    vardaic: _,
                    mangled_name: _,
                } => {
                    let out = self.gen_expr(Box::new(node.clone()), Type::Any, RefStyle::COPY);
                    self.emit(out.stream.as_str());
                }
                Node::PluginStatement {
                    name: _,
                    ret_val: _,
                    ret_type: _,
                    body: _,
                    targ_type: _,
                    args: _,
                } => {
                    let o = self.gen_expr(Box::new(node), Type::Any, RefStyle::COPY);
                    self.emit(&o.stream);
                }
                Node::UnpackStmt { alias, symbols } => {
                    /* pull symbols from 'symbols' into the global context */
                    let bndl = self.bundles.iter().find(|b| b.name == alias).cloned();
                    if !bndl.is_some() {
                        self.consume(CompileError::new(
                            format!("No Bundle named {alias}"),
                            None,
                            node.clone().span,
                            ErrLevel::ERROR,
                        ));
                        self.flush();
                    }
                    let mut b = bndl.clone().unwrap();
                    for sym in symbols.clone() {
                        let f = b.functions.iter().find(|f| f.get_name() == sym);
                        let tf = self.func_table.iter().find(|f| f.get_name() == sym);
                        if f.is_some() {
                            if tf.is_some() {
                                self.consume(CompileError::new(
                                    format!("Conflicting symbol {sym}. Function with this name already exists in the scope"), None, node.clone().span, ErrLevel::ERROR
                                ));
                                self.flush();
                            }
                            self.func_table.push(f.unwrap().clone());
                            continue;
                        }
                        let bn = b.bundles.iter().find(|b| b.name == sym);
                        let tbn = self.bundles.iter().find(|b| b.name == sym);
                        if bn.is_some() {
                            if tbn.is_some() {
                                self.consume(CompileError::new(
                                    format!("Conflicting symbol {sym}. Bundle with this alias already exists in the scope"), None, node.clone().span, ErrLevel::ERROR
                                ));
                                self.flush();
                            }
                            self.bundles.push(bn.unwrap().clone());
                            continue;
                        }
                        let v = b.vars.lookup(&sym);
                        let tv = self.var_table.lookup(&sym);
                        if v.is_some() {
                            if tv.is_some() {
                                self.consume(CompileError::new(
                                    format!("Conflicting symbol {sym}. Variable with this name already exists in the scope"), None, node.clone().span, ErrLevel::ERROR
                                ));
                                self.flush();
                            }
                            self.current_context.add(v.unwrap().clone());
                            continue;
                        }
                        let mut t = b.types.get_layout(Type::Custom(sym.clone()));
                        let tt = self.types.get_layout(Type::Custom(sym.clone()));
                        if t.is_some() {
                            if tt.is_some() {
                                self.consume(CompileError::new(
                                    format!("Conflicting symbol {sym}. Type with this name already exists in the scope"), None, node.clone().span, ErrLevel::ERROR
                                ));
                                self.flush();
                            }
                            t.as_mut().unwrap().name = Type::Custom(sym.clone());
                            self.types
                                .add_type(t.clone().unwrap().name, t.clone().unwrap());
                            continue;
                        }
                        self.consume(CompileError::new(
                            format!("Undefined symbol {sym}"),
                            None,
                            node.clone().span,
                            ErrLevel::ERROR,
                        ));
                        self.flush();
                    }
                }
                _ => {
                    let out = self
                        .gen_expr(Box::new(node), Type::Any, RefStyle::COPY)
                        .clone();
                    self.emit(&format!("{};", out.stream));
                    continue;
                }
            }
        }
    }
    pub fn rest(&mut self) {
        self.flush();
        if !self.is_included {
            self.outfilename = format!("{}.c", self.outfilename);
        } else {
            self.outfilename = format!("{}", self.outfilename);
        }
        self.outfilename = format!("{}/{}", self.buildpath.to_str().unwrap(), self.outfilename);
        let mut outfile = File::create(self.outfilename.clone()).unwrap();
        outfile.write(self.bss.as_bytes()).unwrap();
        outfile.write(self.data.as_bytes()).unwrap();
        outfile.write(self.header.as_bytes()).unwrap();
        outfile.write(self.text.as_bytes()).unwrap();
        outfile.write(self.func.as_bytes()).unwrap();
    }
    fn name_mangler(&mut self, input: String) -> String {
        let _ = input;
        let _prefix = "_Jaguar";
        let _namespace = self.current_context.name.clone();
        let _path = std::path::Path::new(&self.inputpath)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        format!("{_prefix}_{_path}_{_namespace}_{input}")
    }
    fn gen_expr(
        &mut self,
        expression: Box<Spanned<Node>>,
        target_type: Type,
        is_ref: RefStyle,
    ) -> ExprResult {
        let mut stream = String::new();
        let v_is_ref = false;
        let expr = expression.as_ref();
        match expr.node.clone() {
            Node::LiteralInt(num) => {
                if (is_int(target_type.clone())) || (target_type == Type::Any) {
                    match is_ref {
                        RefStyle::DEREF => {
                            self.consume(CompileError::new(
                                format!("Cannot De-reference Value of a literal"),
                                Some("Remove the * operator".into()),
                                expr.clone().span,
                                ErrLevel::ERROR,
                            ));
                            self.flush();
                        }
                        RefStyle::REF => {
                            self.consume(CompileError::new(
                                format!("Cannot reference a literal"),
                                Some("Remove the & operator".into()),
                                expr.clone().span,
                                ErrLevel::ERROR,
                            ));
                            self.flush();
                        }
                        _ => {}
                    }
                }
                self.check_overflow(num.parse::<i128>().unwrap(), target_type, expr.clone().span);
                stream.push_str(format!("{}", num).as_str());
                return ExprResult {
                    preamble: String::new(),
                    stream,
                    is_ref: false,
                    refed_var: None,
                    type_hint: Box::new(Type::INT),
                    var: None,
                };
            }
            Node::LiteralStr(value) => {
                if matches!(is_ref, RefStyle::REF) {
                    self.consume(CompileError::new(
                        format!("Cannot reference an immediate value"),
                        Some("Consider Removing the & operator".to_owned()),
                        expr.clone().span,
                        ErrLevel::ERROR,
                    ));
                } else if matches!(is_ref, RefStyle::DEREF) {
                    self.consume(CompileError::new(
                        format!("Cannot dereference an immediate value"),
                        Some("Consider Removing the * operator".to_owned()),
                        expr.clone().span,
                        ErrLevel::ERROR,
                    ));
                }
                let save = self.cur_section.clone();
                self.cur_section = save;
                return ExprResult {
                    preamble: String::new(),
                    stream: format!("\"{value}\""),
                    is_ref: false,
                    refed_var: None,
                    type_hint: Box::new(Type::MUT(Box::new(Type::STR))),
                    var: Some(format!("(const char*)\"{value}\"")),
                };
            }
            Node::LiteralCh(value) => {
                if matches!(is_ref, RefStyle::REF) {
                    self.consume(CompileError::new(
                        format!("Cannot reference an immediate value"),
                        Some("Consider Removing the & operator".to_owned()),
                        expr.clone().span,
                        ErrLevel::ERROR,
                    ));
                } else if matches!(is_ref, RefStyle::DEREF) {
                    self.consume(CompileError::new(
                        format!("Cannot dereference an immediate value"),
                        Some("Consider Removing the * operator".to_owned()),
                        expr.clone().span,
                        ErrLevel::ERROR,
                    ));
                }
                let save = self.cur_section.clone();
                self.cur_section = save;
                return ExprResult {
                    preamble: String::new(),
                    stream: format!("\'{value}\'"),
                    is_ref: false,
                    refed_var: None,
                    type_hint: Box::new(Type::CHAR),
                    var: Some(format!("(const char*)\"{value}\"")),
                };
            }

            Node::ListInit { content } => {
                let is_generic = self.cur_body_type == BodyType::GENERIC;
                let mut fix = "\\";
                if !is_generic {
                    fix = "";
                }
                let save = self.cur_section.clone();
                self.cur_section = Section::HEADER;
                let mut list_type = Box::new(Type::NoType);
                let mut list_size = String::new();
                if let Type::List(t, n) = target_type.clone() {
                    list_type = t.clone();
                    list_size = n.clone();
                    self.emit(format!("\njaguar_list({}, {});\n", t.to_str(), n).as_str());
                    self.cur_section = save;
                }
                stream += "{.data = {";
                for (i, expr) in content.iter().enumerate() {
                    let out = self.gen_expr(expr.clone(), Type::Any, is_ref.clone());
                    list_type = out.type_hint.clone();
                    stream += out.stream.clone().as_str();
                    if i != content.len() - 1 {
                        stream += ",";
                    }
                    if i >= (list_size.parse::<u32>().unwrap()) as usize {
                        self.consume(CompileError::new(format!("Excess elements passed to array initializer. expected only {} but got {}+", list_size.clone(), i), None,expr.clone().span , ErrLevel::ERROR));
                        self.flush();
                    }
                }
                stream += format!("}}, .len = {list_size}}}{fix}").as_str();

                return ExprResult {
                    stream: stream.clone(),
                    is_ref: false,
                    refed_var: None,
                    type_hint: Box::new(Type::List(list_type, list_size)),
                    var: Some(stream),
                    preamble: String::new(),
                };
            }
            Node::ListAccess { name, index } => {
                match name.node.clone() {
                    Node::MemberAccess { base, field } => {
                        let b = self.gen_expr(name.clone(), target_type, RefStyle::COPY);
                        let v = self
                            .lookup_variable(&b.var.clone().unwrap())
                            .unwrap()
                            .clone();
                        let layout = self.get_layout(v.clone().type_hint).clone();
                        if matches!(layout, None) {
                            self.consume(CompileError::new(
                                format!("Not a type, '{}'", b.clone().type_hint.debug()),
                                None,
                                base.span,
                                ErrLevel::ERROR,
                            ));
                            self.flush();
                        }
                        let mut l = layout.clone().unwrap();
                        let f = l.feilds.iter_mut().find(|p| p.0.to_string() == field);
                        if matches!(f, None) {
                            self.consume(CompileError::new(
                                format!(
                                    "Type '{}' has no field {field}",
                                    v.clone().type_hint.debug()
                                ),
                                None,
                                expr.clone().span,
                                ErrLevel::ERROR,
                            ));
                            self.flush();
                        }
                        let fl = f.unwrap().1.ty.clone();
                        let mut ty = Type::NoType;
                        if !self.is_iterable(fl.clone()) {}
                        let i = self.gen_expr(index, Type::Any, RefStyle::COPY);
                        if let Type::List(v, _c) = fl.clone() {
                            stream += &format!("jaguar_list_at({}, {})", b.stream, i.stream);
                            ty = *v.clone();
                        } else if let Type::PTR(v) = fl.clone() {
                            stream += &format!("{}[{}]", b.stream, i.stream);
                            ty = *v.clone();
                        } else if let Type::STR = fl.clone() {
                            stream += &format!("{}[{}]", b.stream, i.stream);
                            ty = Type::CHAR;
                        }
                        return ExprResult {
                            preamble: format!(""),
                            stream: stream.clone(),
                            is_ref: false,
                            refed_var: None,
                            type_hint: Box::new(ty),
                            var: Some("foo".to_owned()),
                        };
                    }
                    Node::Token(_v, _t) => {
                        let t = self.gen_expr(name.clone(), target_type, RefStyle::COPY);
                        let fl = *t.type_hint.clone();
                        let mut ty = Type::NoType;
                        if !self.is_iterable(fl.clone()) {}
                        let i = self.gen_expr(index, Type::Any, RefStyle::COPY);
                        if let Type::List(v, _c) = fl.clone() {
                            stream += &format!("jaguar_list_at({}, {})", t.stream, i.stream);
                            ty = *v.clone();
                        } else if let Type::PTR(v) = fl.clone() {
                            stream += &format!("{}[{}]", t.stream, i.stream);
                            ty = *v.clone();
                        } else if let Type::STR = fl.clone() {
                            stream += &format!("{}[{}]", t.stream, i.stream);
                            ty = Type::CHAR;
                        }
                        return ExprResult {
                            preamble: "".to_owned(),
                            stream: stream.clone(),
                            is_ref: false,
                            refed_var: None,
                            type_hint: Box::new(ty),
                            var: Some("foo".to_owned()),
                        };
                    }

                    _ => {}
                }
                return ExprResult {
                    preamble: "".to_owned(),
                    stream: stream.clone(),
                    is_ref: false,
                    refed_var: None,
                    type_hint: Box::new(Type::NoType),
                    var: Some("foo".to_owned()),
                };
            }
            Node::StructInit { fields } => {
                let block_name = target_type.clone();
                let block_fields = fields.clone();

                let mut t = self.resolve_type(block_name.clone());
                if let Type::PTR(v) = t {
                    t = *v;
                }
                let layout = self.get_layout(t);
                if layout.is_none() {
                    self.consume(CompileError::new(
                        format!("Not a type, {}", block_name.debug()),
                        None,
                        expr.clone().span,
                        ErrLevel::ERROR,
                    ));
                    self.flush();
                }
                stream += &format!("({}) {{", target_type.to_str());
                for (_i, b_field) in block_fields.clone().iter().enumerate() {
                    if let Node::Pair { field, value } = b_field.node.clone() {
                        if layout
                            .clone()
                            .unwrap()
                            .clone()
                            .feilds
                            .contains_key(field.clone().as_str())
                        {
                            let v_field =
                                layout.clone().unwrap().feilds.get(&field).unwrap().clone();
                            let out = self
                                .gen_expr(value.clone(), v_field.ty.clone(), RefStyle::COPY)
                                .clone();
                            if !self.type_match(*out.clone().type_hint, v_field.ty.clone()) {
                                self.consume(CompileError::new(
                                    format!(
                                        "Type Mismatch. Expected '{}' but got '{}' instead",
                                        v_field.ty.clone().debug(),
                                        out.type_hint.clone().debug()
                                    ),
                                    None,
                                    value.clone().span,
                                    ErrLevel::ERROR,
                                ));
                                self.flush();
                                exit(100);
                            }
                            stream += &format!(".{} = ", field);
                            stream += out.stream.as_str();
                            if _i != block_fields.len() - 1 {
                                stream += ",";
                            }
                        } else {
                            self.consume(CompileError::new(
                                format!("Type {} has no field {field}", block_name.debug()),
                                None,
                                b_field.clone().span,
                                ErrLevel::ERROR,
                            ));
                            self.flush();
                            exit(100);
                        }
                    }
                }
                stream += "}";
                return ExprResult {
                    preamble: "".to_owned(),
                    stream: stream.clone(),
                    is_ref: false,
                    refed_var: None,
                    type_hint: Box::new(block_name.clone()),
                    var: Some(stream),
                };
            }
            Node::RefExpr { expr } => {
                let out = self.gen_expr(expr.clone(), target_type, RefStyle::REF);
                if let Node::Token(name, _is_deref) = &expr.node.clone() {
                    if let Some(var) = self.lookup_variable(name).cloned() {
                        self.set_ref(var);
                    }
                }
                stream += &format!("&{}", out.stream);
                return ExprResult {
                    preamble: "".to_owned(),
                    stream,
                    is_ref: true,
                    refed_var: out.refed_var.clone(),
                    type_hint: Box::new(Type::PTR(out.type_hint)),
                    var: out.var,
                };
            }
            Node::DeRefExpr { expr } => {
                let out = self.gen_expr(expr.clone(), target_type, RefStyle::DEREF);
                let mut type_hint = Type::NoType;
                stream += &format!("*{}", out.stream);
                let mut v_var = None;
                if let Node::Token(name, _is_deref) = &expr.node {
                    v_var = Some(name.clone());

                    if let Some(var) = self.lookup_variable(name).cloned() {
                        type_hint = var.type_hint.clone();
                    }
                }
                if let Type::PTR(v) = *out.clone().type_hint {
                    type_hint = *v;
                }
                return ExprResult {
                    preamble: "".to_owned(),
                    stream,
                    is_ref: false,
                    refed_var: out.refed_var.clone(),
                    type_hint: Box::new(type_hint),
                    var: v_var,
                };
            }
            Node::Token(var, _is_deref) => {
                if let Some(val) = self.lookup_variable(var.as_str()).cloned() {
                    match is_ref {
                        RefStyle::DEREF => {
                            stream += format!("{}", val.name).as_str();
                            if !val.is_ref {
                                if let Type::PTR(ref _v) = val.type_hint {
                                } else {
                                    self.consume(CompileError::new(
                                        format!(
                                            "Cannot dereference value at '{var}'. Not a reference"
                                        ),
                                        None,
                                        expr.clone().span,
                                        ErrLevel::ERROR,
                                    ));
                                    self.flush();
                                }
                            }
                            return ExprResult {
                                preamble: String::new(),
                                stream,
                                is_ref: false,
                                refed_var: Some(var.clone()),
                                type_hint: Box::new(val.type_hint),
                                var: Some(var),
                            };
                        }
                        RefStyle::COPY => {
                            stream += format!("{}", val.name).as_str();
                            return ExprResult {
                                preamble: String::new(),
                                stream,
                                is_ref: val.is_ref,
                                refed_var: Some(var.clone()),
                                type_hint: Box::new(val.type_hint.clone()),
                                var: Some(var),
                            };
                        }
                        RefStyle::REF => {
                            stream += format!("{}", val.name).as_str();
                            self.set_ref(val.clone());
                            return ExprResult {
                                preamble: String::new(),
                                stream,
                                is_ref: true,
                                refed_var: Some(var.clone()),
                                type_hint: Box::new(val.clone().type_hint),
                                var: Some(var),
                            };
                        }
                    }
                } else {
                    self.consume(CompileError::new(
                        format!("Use of Undeclared Symbol '{var}'"),
                        None,
                        expr.clone().span,
                        ErrLevel::ERROR,
                    ));
                    self.flush();
                }
                return ExprResult {
                    stream,
                    is_ref: false,
                    refed_var: None,
                    type_hint: Box::new(Type::NoType),
                    var: None,
                    preamble: String::new(),
                };
            }
            Node::Cast { expr: ex, ty } => {
                let out = self.gen_expr(ex.clone(), target_type, RefStyle::COPY);
                if self.is_castable(*out.type_hint.clone(), ty.clone()) {
                    stream += &format!("({})({})", ty.to_str(), out.stream);
                } else {
                    self.consume(CompileError::new(
                        format!(
                            "Casting from '{}' to '{}' is invalid",
                            out.type_hint.debug(),
                            ty.debug()
                        ),
                        None,
                        expression.span,
                        ErrLevel::ERROR,
                    ));
                    self.flush();
                }
                return ExprResult {
                    preamble: String::new(),
                    stream,
                    is_ref: v_is_ref,
                    refed_var: None,
                    type_hint: Box::new(ty),
                    var: None,
                };
            }
            Node::Ret(v) => {
                let out = self.gen_expr(
                    v.clone(),
                    self.current_scope_return_type.clone(),
                    RefStyle::COPY,
                );
                if !self.type_match(
                    *out.type_hint.clone(),
                    self.current_scope_return_type.clone(),
                ) {
                    self.consume(CompileError::new(
                        format!(
                            "Returning '{}' when '{}' was expected",
                            out.type_hint.debug(),
                            self.current_scope_return_type.debug()
                        ),
                        None,
                        v.span.clone(),
                        ErrLevel::ERROR,
                    ));
                    self.flush();
                }
                stream += &format!("return {}", out.stream);
                return ExprResult {
                    preamble: String::new(),
                    stream,
                    is_ref: false,
                    refed_var: None,
                    type_hint: Box::new(Type::NoType),
                    var: None,
                };
            }
            Node::BREAK => {
                stream += &format!("break;");
                return ExprResult {
                    preamble: String::new(),
                    stream,
                    is_ref: false,
                    refed_var: None,
                    type_hint: Box::new(Type::NoType),
                    var: None,
                };
            }
            Node::BinaryExpr { lhs, opr, rhs } => {
                let mut out = self.gen_expr(lhs.clone(), target_type.clone(), RefStyle::COPY);
                if !is_int(*out.type_hint.clone())
                    && opr.clone() != TokenType::Operator("==".to_owned())
                {
                    self.consume(CompileError::new(
                        format!(
                            "Type {} does not support binary arithmetic'",
                            out.type_hint.debug()
                        ),
                        None,
                        lhs.clone().span,
                        ErrLevel::ERROR,
                    ));
                    self.flush();
                }
                stream += format!("( {}", out.stream.clone()).as_str();
                out = self.gen_expr(rhs.clone(), target_type.clone(), RefStyle::COPY);
                if !is_int(*out.type_hint.clone())
                    && opr.clone() != TokenType::Operator("==".to_owned())
                {
                    self.consume(CompileError::new(
                        format!(
                            "Type {} does not support binary arithmetic'",
                            out.type_hint.debug()
                        ),
                        None,
                        rhs.clone().span,
                        ErrLevel::ERROR,
                    ));
                    self.flush();
                }
                match opr {
                    crate::lexer::TokenType::Operator(val) if val == "+".to_owned() => {
                        stream += "+";
                    }
                    crate::lexer::TokenType::Operator(val) if val == "-".to_owned() => {
                        stream += "-";
                    }
                    crate::lexer::TokenType::Operator(val) if val == "*".to_owned() => {
                        stream += "*";
                    }
                    crate::lexer::TokenType::Operator(val) if val == "/".to_owned() => {
                        stream += "/";
                    }
                    crate::lexer::TokenType::Operator(val) => {
                        stream += &val;
                    }
                    _ => {
                        self.consume(CompileError::new(
                            format!("Not an Operator {:#?}", opr),
                            None,
                            expr.clone().span,
                            ErrLevel::ERROR,
                        ));
                        self.flush();
                        exit(1)
                    }
                }
                stream += format!(" {})", out.stream.clone()).as_str();
                return ExprResult {
                    preamble: String::new(),
                    stream: stream.clone(),
                    is_ref: false,
                    refed_var: None,
                    type_hint: Box::new(Type::INT),
                    var: Some(stream),
                };
            }
            Node::BundleAccess { base, field } => {
                if let Node::Token(var, _) = base.node.clone() {
                    let bndl = self.bundles.iter().find(|b| b.name == var);
                    if matches!(bndl, None) {
                        self.consume(CompileError::new(
                            format!("Could not resolve {var}"),
                            None,
                            base.clone().span,
                            ErrLevel::ERROR,
                        ));
                        self.flush();
                        exit(100);
                    }
                    if let Node::FcCall {
                        params: _,
                        callee: _,
                    } = field.node.clone()
                    {
                        let save = self.func_table.clone();
                        self.func_table = bndl.unwrap().functions.clone();
                        let res = self.gen_func_call(field, target_type);
                        self.func_table = save;
                        stream += format!("\n{}", res.stream).as_str();
                        return ExprResult {
                            preamble: res.preamble,
                            stream,
                            is_ref: false,
                            refed_var: None,
                            type_hint: Box::new(res.type_hint.clone()),
                            var: None,
                        };
                    } else if let Node::BundleAccess {
                        base: _b2,
                        field: _f2,
                    } = field.clone().node
                    {
                        let sb = self.bundles.clone();
                        self.bundles = bndl.clone().unwrap().bundles.clone();
                        let out2 = self.gen_expr(field, target_type, is_ref);
                        stream += &out2.stream;
                        self.bundles = sb.clone();
                        return ExprResult {
                            preamble: String::new(),
                            stream,
                            is_ref: false,
                            refed_var: None,
                            type_hint: Box::new(*out2.type_hint.clone()),
                            var: None,
                        };
                    } else {
                        let out = self.gen_expr(field, target_type, RefStyle::COPY);
                        stream += &out.stream;
                        return ExprResult {
                            preamble: String::new(),
                            stream,
                            is_ref: false,
                            refed_var: None,
                            type_hint: Box::new(*out.type_hint),
                            var: None,
                        };
                    }
                }
            }
            Node::FcCall {
                params: _,
                callee: _,
            } => {
                let out = self.gen_func_call(expression.clone(), target_type);
                stream += out.clone().stream.as_str();
                return ExprResult {
                    preamble: out.preamble,
                    stream,
                    is_ref: false,
                    refed_var: None,
                    type_hint: Box::new(out.type_hint.clone()),
                    var: None,
                };
            }
            Node::MemberAccess { base, field } => {
                let out = self.gen_expr(base.clone(), target_type, RefStyle::COPY);
                stream += out.stream.as_str();
                let layout = self.get_layout(*out.type_hint.clone());
                if matches!(layout, None) {
                    self.consume(CompileError::new(
                        format!("Not a type,  {}", out.type_hint.debug()),
                        None,
                        base.clone().span,
                        ErrLevel::ERROR,
                    ));
                    self.flush();
                    exit(100);
                }
                #[allow(unused_assignments)]
                let mut type_hint: Box<Type> = Box::new(Type::NoType);
                if !layout.as_ref().unwrap().feilds.contains_key(&field.clone()) {
                    self.consume(CompileError::new(
                        format!("Type {} has no field {field}", out.type_hint.debug()),
                        None,
                        base.clone().span,
                        ErrLevel::ERROR,
                    ));
                    self.flush();
                }
                let v_field = layout
                    .unwrap()
                    .clone()
                    .feilds
                    .get(&field.clone())
                    .unwrap()
                    .clone();
                if out.var.clone() == None {
                    self.consume(CompileError::new(
                        format!("Use of undeclared symbol {field}"),
                        None,
                        base.clone().span,
                        ErrLevel::ERROR,
                    ));
                    self.flush();
                }
                if let Some(var) = self.lookup_variable(out.var.clone().unwrap().as_str()) {
                    if var.type_hint.is_pointer() {
                        stream += format!("->{}", field.clone()).as_str();
                    } else {
                        stream += format!(".{}", field.clone()).as_str();
                    }
                    type_hint = Box::new(v_field.ty.clone());
                    if !var.type_hint.is_mutable() {
                        if let Type::MUT(ty) = v_field.ty.clone() {
                            type_hint = ty;
                        }
                    }
                } else {
                    self.consume(CompileError::new(
                        format!("Use of undeclared symbol {}", out.var.unwrap().clone()),
                        None,
                        base.clone().span,
                        ErrLevel::ERROR,
                    ));
                    self.flush();
                    exit(100);
                }
                return ExprResult {
                    preamble: String::new(),
                    stream,
                    is_ref: false,
                    refed_var: None,
                    type_hint,
                    var: out.var,
                };
            }
            Node::ReVal { name, value } => match name.node.clone() {
                Node::ListAccess {
                    name: _lname,
                    index: _,
                } => {
                    let out = self.gen_expr(name, Type::Any, RefStyle::COPY);
                    let v = self.gen_expr(value, Type::Any, RefStyle::COPY);
                    stream += &format!("{} = {}", out.stream, v.stream);
                }
                Node::MemberAccess { base, field } => {
                    let base_out = self
                        .gen_expr(base.clone(), Type::Any, RefStyle::COPY)
                        .clone();
                    let mut t = self.resolve_type(*base_out.type_hint.clone());
                    if let Type::PTR(v) = t {
                        t = *v;
                    }
                    let layout = self.get_layout(t);
                    if matches!(layout, None) {
                        self.consume(CompileError::new(
                            format!("Not a Type. '{}'", base_out.type_hint.debug()),
                            None,
                            base.span,
                            ErrLevel::ERROR,
                        ));
                        self.flush();
                        exit(1);
                    }
                    stream += &base_out.clone().stream;
                    let v_field = layout.unwrap().feilds.get(&field).cloned();
                    if v_field.is_some() {
                        if !v_field.unwrap().ty.is_mutable() {
                            self.consume(CompileError::new(
                                format!(
                                    "Cannot mutate a const value. '{}'",
                                    base_out.var.clone().unwrap()
                                ),
                                None,
                                base.span.clone(),
                                ErrLevel::ERROR,
                            ));
                            self.flush();
                        }
                        let out = self.gen_expr(value, Type::Any, RefStyle::COPY).clone();
                        let mut modifier = ".";
                        if base_out.type_hint.is_pointer() {
                            if let Type::PTR(ty) = *base_out.clone().type_hint {
                                if !ty.is_mutable() {
                                    self.consume(CompileError::new(
                                        format!(
                                            "Cannot mutate a const value. '{}'",
                                            base_out.var.clone().unwrap()
                                        ),
                                        Some(format!(
                                            "'{}' points to a const object of type '{}'",
                                            base_out.var.unwrap(),
                                            ty.c_impl()
                                        )),
                                        base.span.clone(),
                                        ErrLevel::ERROR,
                                    ));
                                    self.flush();
                                }
                            }
                            modifier = "->";
                        }
                        stream += &format!("{modifier}{} = {}", field, out.stream);
                    }
                }
                Node::Token(var, _d) => {
                    let val = self.lookup_variable(&var.clone()).unwrap().clone();
                    if !val.type_hint.is_mutable() {
                        self.consume(CompileError::new(
                            format!("Cannot mutate a const value '{}'.", val.name),
                            None,
                            value.clone().span,
                            ErrLevel::ERROR,
                        ));
                        self.flush();
                    }
                    let out = self.gen_expr(value.clone(), target_type.clone(), RefStyle::COPY);
                    if !self.type_match(target_type, *out.clone().type_hint) {
                        self.consume(CompileError::new(
                            format!(
                                "Type Mismatch. '{}' expected '{}' but got '{}' instead",
                                var.clone(),
                                val.clone().type_hint.debug(),
                                out.type_hint.clone().debug()
                            ),
                            None,
                            value.span,
                            ErrLevel::ERROR,
                        ));
                        self.flush();
                    }
                    stream += &format!("{var} = {}\n", out.stream);
                    return ExprResult {
                        preamble: String::new(),
                        stream,
                        is_ref: v_is_ref,
                        refed_var: None,
                        type_hint: Box::new(val.type_hint),
                        var: None,
                    };
                }
                _ => {
                    self.consume(CompileError::new(
                        format!("Not an Expression"),
                        None,
                        expression.span,
                        ErrLevel::ERROR,
                    ));
                    self.flush();
                }
            },
            Node::PluginStatement {
                name,
                ret_val: _,
                ret_type,
                body,
                targ_type,
                args,
            } => {
                let saved_type = self.current_scope_return_type.clone();
                self.current_scope_return_type = *ret_type.clone();
                let plugin_name = name.clone();
                let plugin_type = ret_type.clone();
                let mut oplugin_name = String::new();
                oplugin_name += &targ_type.c_impl();
                oplugin_name += &format!("_{}", plugin_name);
                self.change_scope(name.as_str());
                self.cur_section = Section::FUNC;
                stream.push_str(
                    format!(
                        "\nextern inline {} {}(",
                        plugin_type.c_impl().clone(),
                        oplugin_name.clone()
                    )
                    .as_str(),
                );
                for (index, arg) in args.clone().iter_mut().enumerate() {
                    let mut modifier = "";
                    if arg.name.clone() == "self" {
                        if arg.type_hint == Type::NoType {
                            modifier = "*";
                            self.current_context.add(Var::new(
                                "self".into(),
                                targ_type.clone(),
                                true,
                                None,
                                expr.clone().span,
                            ));
                            arg.type_hint = targ_type.clone();
                        } else {
                            self.current_context.add(Var::new(
                                "self".into(),
                                arg.type_hint.clone(),
                                match arg.type_hint.clone() {
                                    Type::PTR(_v) => true,
                                    _ => false,
                                },
                                None,
                                expr.clone().span,
                            ));
                        }
                    } else {
                        self.current_context.add(Var::new(
                            arg.name.clone(),
                            arg.type_hint.clone(),
                            arg.is_ref.clone(),
                            None,
                            expr.clone().span,
                        ));
                    }

                    stream.push_str(&format!(
                        "{} {}{}",
                        arg.type_hint.to_str(),
                        modifier,
                        arg.name
                    ));
                    if index != args.len() - 1 {
                        stream.push_str(",");
                    }
                }
                stream.push_str(&format!(") {{\n"));
                match body.clone().node.clone() {
                    Node::Program(k) => {
                        for n in k.clone().iter().enumerate() {
                            let o = self.gen_expr(Box::new(n.1.clone()), Type::Any, RefStyle::COPY);
                            stream += &o.stream;
                            stream += ";";
                            if n.0 != k.len() - 1 {
                                stream += "\\\n";
                            }
                        }
                    }
                    _ => {}
                }

                stream.push_str(&format!("}}"));
                let mut plugin = Function::new(
                    plugin_name.clone(),
                    self.current_context.clone(),
                    *ret_type.clone(),
                    true,
                    body.clone().node,
                );
                plugin.args = args;
                self.cur_section = Section::TEXT;
                self.exit_scope();
                self.current_scope_return_type = saved_type;
            }
            Node::IfStmt {
                cond,
                body,
                elseifs,
                elsestmt,
            } => {
                let is_generic = self.cur_body_type == BodyType::GENERIC;
                let mut fix = "\\";
                if !is_generic {
                    fix = "";
                }
                stream.push_str(&format!("if ("));
                let cout = self.gen_expr(cond, Type::Any, RefStyle::COPY);
                stream.push_str(&format!("{}", cout.stream));
                stream.push_str(&format!("){{{fix}\n"));
                match body.node {
                    Node::Program(k) => {
                        for node in k.clone() {
                            let o = self.gen_expr(Box::new(node), Type::Any, RefStyle::COPY);
                            stream += &o.stream;
                            stream += &format!(";{fix}");
                        }
                    }
                    _ => {}
                }
                stream.push_str(&format!("\n}}"));
                if elseifs.is_some() {
                    let branches = elseifs.unwrap();
                    for else_if_stmt in branches {
                        let s = self.gen_expr(Box::new(else_if_stmt), Type::Any, RefStyle::COPY);
                        stream += &format!("else {}", s.stream);
                    }
                }
                if let Some(elsestmt1) = elsestmt {
                    stream += "else {";
                    match elsestmt1.node {
                        Node::Program(k) => {
                            for node in k {
                                let n = self.gen_expr(Box::new(node), Type::Any, RefStyle::COPY);
                                stream += &format!("{};\n", n.stream);
                            }
                        }
                        _ => {}
                    }
                    stream += "}"
                }
            }
            Node::LetStmt {
                name,
                type_hint,
                value,
                is_mut,
            } => {
                if type_hint.clone() == Type::NoType {
                    self.consume(CompileError::new(
                        format!("Cannot assign to type void"),
                        None,
                        value.clone().span,
                        ErrLevel::ERROR,
                    ));
                    self.flush();
                }
                if let Some(v) = self.lookup_variable(&name).cloned() {
                    self.consume(CompileError::new(
                        format!("Redefinition of {name}"),
                        None,
                        expr.clone().span,
                        ErrLevel::ERROR,
                    ));
                    self.consume(CompileError::new(
                        format!("Previously defined here"),
                        None,
                        v.definition,
                        ErrLevel::WARNING,
                    ));
                    self.flush();
                }
                let l = self.get_layout(type_hint.clone());
                if !l.is_some() && type_hint != Type::Any {
                    self.consume(CompileError::new(
                        format!("Not a Type, '{}'", type_hint.debug()),
                        None,
                        expr.clone().span,
                        ErrLevel::ERROR,
                    ));
                    self.flush();
                }
                let mut temp_stream: String = String::new();
                let out = self.gen_expr(value.clone(), type_hint.clone(), RefStyle::COPY);
                if *out.type_hint.clone() == Type::NoType {
                    self.consume(CompileError::new(
                        format!("Expression does not return a value"),
                        None,
                        value.clone().span,
                        ErrLevel::ERROR,
                    ));
                    self.flush();
                }
                if !self.type_match(type_hint.clone(), *out.type_hint.clone()) {
                    self.consume(CompileError::new(
                        format!(
                            "Type Mismatch. '{}' expected '{}' but got '{}' instead",
                            name.clone(),
                            type_hint.clone().debug(),
                            out.type_hint.clone().debug(),
                        ),
                        None,
                        value.clone().span,
                        ErrLevel::ERROR,
                    ));
                    self.flush();
                }

                temp_stream += out.stream.as_str();
                let is_ref = {
                    match *out.type_hint.clone() {
                        Type::PTR(_v) => true,
                        _ => out.is_ref,
                    }
                };
                let mut new_var = var::Var {
                    name: name.clone(),
                    type_hint: type_hint.clone(),
                    is_ref,
                    references: None,
                    definition: expr.clone().span,
                };
                if type_hint == Type::Any {
                    if is_mut {
                        new_var.type_hint = Type::MUT(out.type_hint.clone());
                    } else {
                        new_var.type_hint = *out.type_hint.clone();
                    }
                }
                // ToDo: Implement Functionality => VTable
                if let Some(ref_name) = out.refed_var {
                    new_var.references = Some(Box::new(
                        self.lookup_variable(&ref_name.as_str()).unwrap().clone(),
                    ));
                    self.lookup_variable(&ref_name.as_str()).unwrap().is_ref = true;
                }
                stream += &out.preamble;
                stream.push_str(&format!(
                    "{} {} = {}",
                    new_var.clone().type_hint.to_str(),
                    name,
                    out.stream
                ));
                self.current_context.add(new_var.clone());
            }
            Node::ForStmt {
                init,
                cond,
                inc,
                body,
            } => {
                let is_generic = {
                    let v = self.cur_body_type == BodyType::GENERIC;
                    v
                };
                let mut fix = "\\";
                if !is_generic {
                    fix = "";
                }
                stream.push_str("\nfor (");
                if let Node::ReVal { name, value: _ } = init.clone().node {
                    if let Node::Token(v, _) = name.node.clone() {
                        self.current_context.add(Var::new(
                            v,
                            Type::INT,
                            false,
                            None,
                            name.clone().span,
                        ));
                    }
                }
                let iniout = self.gen_expr(init, Type::Any, RefStyle::COPY);
                stream.push_str("jaguar_int ");
                stream.push_str(&iniout.stream);
                stream.push_str(";");
                let condout = self.gen_expr(cond, Type::Any, RefStyle::COPY);
                stream.push_str(&condout.stream);
                stream.push_str(";");
                stream.push_str("(");
                let incout = self.gen_expr(inc, Type::Any, RefStyle::COPY);
                stream.push_str(&incout.stream);
                stream.push_str(")");
                stream.push_str(") {");
                match body.node.clone() {
                    Node::Program(k) => {
                        for node in k.clone() {
                            let s = self.gen_expr(Box::new(node), Type::Any, RefStyle::COPY);
                            stream += &format!("\n{};{fix}", s.stream);
                        }
                    }
                    _ => {}
                }
                stream.push_str("}\n");
            }
            Node::WhileStmt { cond, body } => {
                stream += "\nwhile (";
                let cond_stream = self.gen_expr(cond, Type::Any, RefStyle::COPY);
                stream += &format!("{}) {{\n", cond_stream.stream);
                match body.node {
                    Node::Program(k) => {
                        for node in k {
                            let o = self.gen_expr(Box::new(node), Type::Any, RefStyle::COPY);
                            stream += &format!("{};\n", o.stream);
                        }
                    }
                    _ => {}
                }
                stream += "}";
            }
            Node::NameSpace { alias, body } => {
                let sc = self.current_context.clone();
                let st = self.types.clone();
                let sf = self.func_table.clone();
                let sb = self.bundles.clone();
                self.current_context = Context::new(alias.clone(), Some(Box::new(sc.clone())));
                self.bundles = vec![];
                match body.clone().node {
                    Node::Program(k) => {
                        for node in k {
                            let o = self.gen_expr(Box::new(node), Type::Any, RefStyle::COPY);
                            stream += &format!("{};\n", o.stream);
                        }
                    }
                    _ => {}
                }
                let saved_bundle = self.bundles.clone();
                let b = self.bundles.iter_mut().find(|p| p.name == alias);
                if let Some(b1) = b {
                    b1.vars
                        .content
                        .append(&mut self.current_context.content.content.clone());
                    b1.functions.append(&mut self.func_table.clone());
                    b1.types.content.extend(self.types.content.clone());
                    b1.bundles.append(&mut saved_bundle.clone());
                    self.current_context = sc;
                    self.types = st;
                    self.func_table = sf;
                    self.bundles = sb;
                } else {
                    let bundle = Bundle::new(
                        alias,
                        self.current_context.get_content(),
                        self.func_table.clone(),
                        self.types.clone(),
                        self.bundles.clone(),
                        self.outfilepath.clone(),
                    );
                    self.current_context = sc;
                    self.types = st;
                    self.func_table = sf;
                    self.bundles = sb;
                    self.bundles.push(bundle);
                }

                return ExprResult {
                    preamble: String::new(),
                    stream,
                    is_ref: false,
                    refed_var: None,
                    type_hint: Box::new(Type::NoType),
                    var: None,
                };
            }
            Node::FnStmt {
                body,
                name,
                ret_type,
                returns,
                return_val: _,
                args,
                vardaic: _,
                mangled_name: _,
            } => {
                let saved_type = self.current_scope_return_type.clone();
                self.current_scope_return_type = ret_type.clone();
                let mut fn_ret = ret_type.clone();
                let mut fmangled_name = self.name_mangler(name.clone());
                if name == "main".to_string() {
                    fmangled_name = name.clone();
                    fn_ret = Type::INT;
                }
                stream.push_str("\n");
                self.change_scope(name.as_str());
                self.cur_section = Section::FUNC;
                stream.push_str("\n");
                if self.is_included {
                    stream.push_str("extern inline ");
                }
                if self.type_match(fn_ret.clone(), Type::NoType) && name != "main" {
                    stream.push_str(&format!("void {fmangled_name}("));
                } else {
                    if name == "main" {
                        stream += &format!("{} {fmangled_name} (", fn_ret.c_impl());
                    } else {
                        stream.push_str(&format!("{} {fmangled_name} (", fn_ret.to_str()));
                    }
                }
                let mut index: u16 = 0;
                for arg in args.clone() {
                    stream.push_str(format!("{} {}", arg.type_hint.to_str(), arg.name).as_str());
                    if index != (args.len() - 1 as usize) as u16 {
                        stream.push_str(",");
                    }
                    let v = Var::new(arg.name, arg.type_hint, false, None, expr.clone().span);
                    self.var_table.add(v.clone());
                    self.current_context.add(v);
                    index += 1;
                }
                stream.push_str(") {");
                self.track_rsp = true;
                match body.node.clone() {
                    Node::Program(k) => {
                        for node in k.clone() {
                            let o =
                                self.gen_expr(Box::new(node.clone()), Type::Any, RefStyle::COPY);
                            stream.push_str(&o.stream);
                            stream.push_str(";");
                        }
                    }
                    _ => {}
                }

                if name == "main" && fn_ret == Type::INT {
                    stream.push_str("\n\treturn 0;");
                }
                stream.push_str("\n}\n");
                let mut func = Function::new(
                    name,
                    self.current_context.clone(),
                    fn_ret.clone(),
                    returns,
                    body.node,
                );
                func.args = args.clone();
                func.gen_name = fmangled_name;
                self.func_table.push(func);
                self.exit_scope();
                self.cur_section = Section::TEXT;
                self.current_scope_return_type = saved_type;
                return ExprResult {
                    preamble: String::new(),
                    stream,
                    is_ref: false,
                    refed_var: None,
                    type_hint: Box::new(target_type),
                    var: None,
                };
            }
            Node::NULLPTR => match target_type.clone() {
                Type::PTR(_v) => {
                    return ExprResult {
                        preamble: String::new(),
                        stream: "NULL".to_owned(),
                        is_ref: true,
                        refed_var: None,
                        type_hint: Box::new(target_type),
                        var: None,
                    };
                }
                Type::STR => {
                    return ExprResult {
                        preamble: String::new(),
                        stream: "NULL".to_owned(),
                        is_ref: true,
                        refed_var: None,
                        type_hint: Box::new(target_type),
                        var: None,
                    };
                }
                _ => {
                    return ExprResult {
                        preamble: String::new(),
                        stream: "NULL".to_owned(),
                        is_ref: true,
                        refed_var: None,
                        type_hint: Box::new(target_type),
                        var: None,
                    };
                }
            },
            _ => {
                self.consume(CompileError::new(
                    format!("Not a Expression"),
                    None,
                    expr.clone().span,
                    ErrLevel::ERROR,
                ));
                self.flush();
            }
        }
        ExprResult {
            preamble: String::new(),
            stream,
            is_ref: v_is_ref,
            refed_var: None,
            type_hint: Box::new(Type::NoType),
            var: None,
        }
    }
    fn gen_func_call(&mut self, expression: Box<Spanned<Node>>, target_type: Type) -> FResult {
        let mut stream = String::new();
        let Node::FcCall { params, callee } = expression.node.clone() else {
            return FResult {
                stream,
                type_hint: Type::NoType,
                preamble: String::new(),
            };
        };
        let mut fcname = String::new();
        let mut fargs: Vec<FunctionArg> = Vec::new();
        let mut fret_type = Type::NoType;
        let mut variadic = false;
        match callee.clone().node {
            Node::MemberAccess { base, field } => {
                let mut is_f = false;
                if let Node::FcCall {
                    params: _,
                    callee: _,
                } = base.clone().node
                {
                    is_f = true;
                }
                let out = self.gen_expr(base.clone(), target_type.clone(), RefStyle::COPY);
                let layout = self.get_layout(*out.clone().type_hint).clone();
                #[allow(unused)]
                let mut base_type: Type = Type::NoType;
                if out.var.is_some() {
                    base_type = self
                        .lookup_variable(&out.var.clone().unwrap())
                        .unwrap()
                        .type_hint
                        .clone();
                } else {
                    base_type = *out.type_hint.clone();
                }
                let method = self.get_field_item(layout.unwrap(), &field, callee.clone().span);

                fargs = method.clone().args;
                #[allow(unused_assignments)]
                let mut modifier = "";
                let mut gmod = "*";
                #[allow(unused_assignments)]
                let mut gvalmod = "&";
                if out.type_hint.is_pointer() {
                    gmod = "*";
                    modifier = "";
                    gvalmod = "";
                } else {
                    gvalmod = "&";
                    modifier = "";
                }
                if is_f {
                    gvalmod = "";
                    gmod = "";
                }
                let g = self.gb();
                let mut preamble = String::new();
                preamble += &format!(
                    "{}{gmod} __{} = {gvalmod}{};\n",
                    base_type.c_impl(),
                    g.clone(),
                    out.stream
                );
                stream += &format!("{}_{}({modifier}__{g}", base_type.c_impl(), field);
                if !params.clone().is_empty() {
                    stream += ",";
                }
                let mut argsize = fargs.clone().len() as i128;
                #[allow(irrefutable_let_patterns)]
                if let _ = fargs.clone().iter().find(|p| p.name == "self") {
                    if argsize >= 1 {
                        argsize = argsize - 1;
                    }
                }
                if argsize as usize != params.clone().len() {
                    self.consume(CompileError::new(
                        format!(
                            "Invalid paramemter count. expected '{}' but got '{}' instead",
                            argsize,
                            params.clone().len()
                        ),
                        None,
                        expression.clone().span,
                        ErrLevel::ERROR,
                    ));
                    self.flush();
                }
                let mut i = 1;
                for arg in params.clone() {
                    let ar = fargs.get(i).clone();
                    let a = self.gen_expr(
                        Box::new(arg.clone()),
                        ar.unwrap().type_hint.clone(),
                        RefStyle::COPY,
                    );
                    if !self.type_match(*a.type_hint.clone(), ar.unwrap().type_hint.clone()) {
                        self.consume(CompileError::new(
                            format!(
                                "mismatched type. got {} instead of {}",
                                a.type_hint.debug(),
                                ar.unwrap().type_hint.debug()
                            ),
                            None,
                            arg.span.clone(),
                            ErrLevel::ERROR,
                        ));
                        self.flush();
                        exit(2);
                    }
                    if *a.type_hint.clone() == Type::NoType {
                        self.consume(CompileError::new(
                            format!("Expression does not return a value"),
                            None,
                            arg.span.clone(),
                            ErrLevel::ERROR,
                        ));
                        self.flush();
                    }
                    stream += &a.stream;
                    if i <= params.len() - 1 {
                        stream += ",";
                    }
                    i += 1;
                }
                stream += &format!(")");
                return FResult {
                    preamble,
                    stream,
                    type_hint: method.ty,
                };
            }
            Node::BundleAccess { base, field } => {
                if let Node::Token(var, _i) = base.node.clone() {
                    let bundle: Option<Bundle> =
                        self.bundles.clone().iter().find(|b| b.name == var).cloned();
                    if matches!(bundle, None) {
                        self.consume(CompileError::new(
                            format!("Could not resolve scope {var}"),
                            Some("Confirm Bundle path".into()),
                            base.span,
                            ErrLevel::ERROR,
                        ));
                        self.flush();
                        let _sf = self.func_table.clone();
                        self.func_table = bundle.clone().unwrap().functions;
                        let out = self.gen_expr(field.clone(), target_type, RefStyle::COPY);
                        stream += &out.stream;
                        exit(100); /* should never reach here */
                    }
                } else if let Node::BundleAccess {
                    base: _inner,
                    field: _d,
                } = base.node.clone()
                {
                } else if let Node::FcCall {
                    params: _,
                    callee: _,
                } = base.node.clone()
                {
                }
            }
            Node::Token(var, _deref) => {
                fcname = var.clone();
                let func = self
                    .func_table
                    .iter()
                    .find(|func| func.get_name() == fcname.clone());
                if matches!(func, None) {
                    self.consume(CompileError::new(
                        format!("Use of undeclared symbol {var}"),
                        None,
                        expression.span,
                        ErrLevel::ERROR,
                    ));
                    self.flush();
                    exit(100);
                }
                fargs = func.unwrap().args.clone();
                fret_type = func.unwrap().ty.clone();
                variadic = func.unwrap().variadic;
                stream += format!("{}(", func.unwrap().gen_name).as_str();
            }
            _ => {
                println!("TODO: ERR CALLEE");
                exit(100);
            }
        }
        let parlen = params.len();
        if (fargs.clone().len() != parlen) && !variadic {
            self.consume(CompileError::new(
                format!("Invalid argument count. {fcname} expected {} parameters but got {parlen} instead", fargs.len()),
                None,
                expression.span,
                ErrLevel::ERROR,
            ));
            self.flush();
        }
        for (index, param) in params.iter().enumerate() {
            let arg = fargs
                .get(index as usize)
                .unwrap_or(
                    &FunctionArg {
                        name: "".into(),
                        type_hint: Type::Any,
                        is_ref: false,
                    }
                    .clone(),
                )
                .clone();

            let expr_code = self.gen_expr(
                Box::new(param.clone()),
                arg.type_hint.clone(),
                RefStyle::COPY,
            );
            if *expr_code.type_hint.clone() == Type::NoType {
                self.consume(CompileError::new(
                    format!("Expression does not return a value"),
                    None,
                    param.span.clone(),
                    ErrLevel::ERROR,
                ));
                self.flush();
            }
            stream += expr_code.stream.as_str();
            if index != params.len() - 1 {
                stream += ",";
            }
            if !self.type_match(arg.type_hint.clone(), *expr_code.type_hint.clone()) {
                self.consume(CompileError::new(
                    format!(
                        "Type Mismatch. '{}' expected '{}' but got '{}' instead",
                        arg.name.clone(),
                        arg.type_hint.clone().debug(),
                        expr_code.type_hint.clone().debug()
                    ),
                    None,
                    param.clone().span,
                    ErrLevel::ERROR,
                ));
                self.flush();
            }
            if arg.type_hint.is_mutable() && !expr_code.type_hint.is_mutable() {
                self.consume(CompileError::new(
                    format!(
                        "Passing a const value '{}' to an argument that expects a mutable value is forbidden",
                        expr_code.type_hint.clone().debug()
                    ),
                    None,
                    param.clone().span,
                    ErrLevel::ERROR,
                ));
                self.flush();
            }
        }
        stream += &format!(")");
        let ty = fret_type.clone();
        return FResult {
            preamble: String::new(),
            stream,
            type_hint: ty,
        };
    }
    fn lookup_variable(&mut self, name: &str) -> Option<&mut Var> {
        self.current_context.look_up_var(name)
    }

    fn emit(&mut self, text: &str) {
        match self.cur_section {
            Section::TEXT => {
                self.text += text;
            }
            Section::FUNC => {
                self.func += text;
            }
            Section::HEADER => {
                self.header += text;
            }
        }
    }

    fn error(&mut self, err_msg: CompileError) {
        self.has_error = true;
        let span = err_msg.span.clone();
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
        eprintln!(
            "{}",
            format!("{} {}", "[Tixie Error] :".red().bold(), err_msg.errmsg)
        );
        let filename = self.inputpath.clone();
        eprintln!(" ---> line : [{filename}:{line_number}]");
        eprintln!("   |\n{: >2} | {}", line_number, line);
        let mut caret_line = String::new();
        for (i, c) in line.chars().enumerate() {
            if i >= col_start && i < col_end {
                caret_line.push('^');
            } else if i < col_start {
                caret_line.push(if c == '\t' { '\t' } else { ' ' });
            }
        }
        eprint!("   | {}", format!("{caret_line}").as_str().yellow().bold());
        if !matches!(err_msg.help, None) {
            eprint!(" help : {}", err_msg.help.unwrap().yellow().bold());
        }
        eprintln!("");
    }

    fn warn(&mut self, err_msg: CompileError) {
        let span = err_msg.span.clone();
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
        eprintln!(
            "{}",
            format!("[Tixie Warning]: {}", err_msg.errmsg)
                .yellow()
                .bold()
        );
        let filename = self.inputpath.clone();
        eprintln!(" ---> line : [{filename}:{line_number}]");
        eprintln!("   |\n{: >2} | {}", line_number, line);
        let mut caret_line = String::new();
        for (i, c) in line.chars().enumerate() {
            if i >= col_start && i < col_end {
                caret_line.push('^');
            } else if i < col_start {
                caret_line.push(if c == '\t' { '\t' } else { ' ' });
            }
        }
        eprint!("   | {}", format!("{caret_line}").as_str().yellow().bold());
        if !matches!(err_msg.help, None) {
            eprint!(" help : {}", err_msg.help.unwrap().blue())
        }
        eprintln!("");
    }

    fn get_layout(&mut self, type_hint: Type) -> Option<StructLayout> {
        if let Type::BundledType { bundle, ty } = type_hint {
            let bd = self.bundles.iter().find(|b| b.name == bundle).cloned();
            if matches!(bd, None) {
                return None;
            }
            let sb = self.bundles.clone();
            let st = self.types.clone();
            let sc = self.current_context.clone();
            let sv = self.var_table.clone();
            self.bundles = bd.clone().unwrap().bundles.clone();
            self.types = bd.clone().unwrap().types.clone();
            self.var_table = bd.unwrap().vars;
            let g = self.get_layout(*ty);
            self.bundles = sb.clone();
            self.types = st.clone();
            self.current_context = sc.clone();
            self.var_table = sv.clone();
            return g;
        } else if let Type::PTR(ty) = type_hint {
            return self.get_layout(*ty);
        } else if let Type::MUT(ty) = type_hint {
            return self.get_layout(*ty);
        }
        self.types.get_layout(type_hint)
    }

    fn consume(&mut self, err: CompileError) {
        self.errorbox.push(err);
    }
    fn resolve_type(&mut self, t: Type) -> Type {
        match t.clone() {
            Type::BundledType { bundle, ty } => {
                let bndl = self
                    .bundles
                    .clone()
                    .iter()
                    .find(|b| b.name == bundle)
                    .cloned();
                let sb = self.bundles.clone();
                if !bndl.is_some() {
                    return *ty;
                }
                self.bundles = bndl.clone().unwrap().bundles.clone();
                let t2 = self.resolve_type(*ty);
                self.bundles = sb.clone();
                t2
            }
            _ => t,
        }
    }
    fn type_match(&mut self, ty1: Type, ty2: Type) -> bool {
        let a = self.resolve_type(ty1);
        let b = self.resolve_type(ty2);
        match (a, b) {
            (
                Type::BundledType {
                    bundle: _b1,
                    ty: ty_1,
                },
                Type::BundledType {
                    bundle: _b2,
                    ty: ty_2,
                },
            ) => {
                let l1 = self.get_layout(*ty_1.clone()).clone();
                let l2 = self.get_layout(*ty_2.clone()).clone();
                l1.unwrap().file == l2.unwrap().file && self.type_match(*ty_1, *ty_2)
            }
            (Type::Any, _) => true,
            (_, Type::BundledType { bundle: _, ty: _ })
            | (Type::BundledType { bundle: _, ty: _ }, _) => false,
            (Type::MUT(t), other) => self.type_match(*t, other),
            (other, Type::MUT(t)) => self.type_match(other, *t),
            (Type::Custom(c1), Type::Custom(c2)) => c1 == c2,
            (Type::INT, other) => is_int(other),
            (other, Type::INT) => is_int(other),
            (Type::STR, Type::STR) => true,
            (Type::STR, Type::PTR(v)) | (Type::PTR(v), Type::STR) if *v == Type::CHAR => true,
            (Type::STR, Type::PTR(v)) if *v == Type::NoType => true,
            (Type::STR, _other) => false,
            (_other, Type::STR) => false,
            (Type::List(t, s), Type::List(t2, s2)) => self.type_match(*t, *t2) && s == s2,
            (Type::CHAR, Type::CHAR) => true,
            (Type::NoType, Type::NoType) => true,
            (Type::PTR(v), Type::PTR(c)) => self.type_match(*v, *c),
            (int1, int2) => is_int(int1) && is_int(int2),
        }
    }

    fn flush(&mut self) {
        let mut shld_err = false;
        for err in self.errorbox.clone() {
            match err.level {
                ErrLevel::WARNING => {
                    self.warn(err);
                }
                ErrLevel::ERROR => {
                    self.error(err);
                    shld_err = true;
                }
                ErrLevel::INFO => self.warn(err),
            }
        }
        if !self.errorbox.is_empty() && shld_err {
            exit(1);
        }
    }

    fn register_plugin(&mut self, ty: Type, plugin: Function) {
        if let Type::BundledType { bundle, ty } = ty {
            let bd = self.bundles.iter().find(|b| b.name == bundle).cloned();
            let sb = self.bundles.clone();
            let st = self.types.clone();
            let sc = self.current_context.clone();
            let sv = self.var_table.clone();
            self.bundles = bd.clone().unwrap().bundles.clone();
            self.types = bd.clone().unwrap().types.clone();
            self.var_table = bd.unwrap().vars;
            self.register_plugin(*ty, plugin);
            self.bundles = sb.clone();
            self.types = st.clone();
            self.current_context = sc.clone();
            self.var_table = sv.clone();
            return;
        }
        self.types.register_plugin(ty, plugin);
    }
    fn gb(&mut self) -> String {
        let mut name: String = String::from_str("gbval").unwrap();
        name += &format!("{}", self.immediate_counter);
        self.immediate_counter += 1;
        name.into()
    }

    fn check_overflow(&mut self, value: i128, target_type: Type, span: Span) {
        match target_type.clone() {
            Type::U8 => {
                if value < 0 || value > 255 {
                    let warp = value as u8;
                    self.consume(CompileError::new(
                        format!("u8 overflow. value {value} warped to {warp}"),
                        Some("Use from range 0..255 for type u8".into()),
                        span,
                        ErrLevel::WARNING,
                    ));
                }
            }
            Type::U16 => {
                if value < u16::MIN as i128 || value > u16::MAX as i128 {
                    let warp = value as u16;
                    self.consume(CompileError::new(
                        format!("u16 overflow. value {value} warped to {warp}"),
                        Some(format!(
                            "Use from range {}..{} for type u16",
                            u16::MIN,
                            u16::MAX
                        )),
                        span,
                        ErrLevel::WARNING,
                    ));
                }
            }
            Type::U32 => {
                if value < u32::MIN as i128 || value > u32::MAX as i128 {
                    let warp = value as u32;
                    self.consume(CompileError::new(
                        format!("u16 overflow. value {value} warped to {warp}"),
                        Some(format!(
                            "Use from range {}..{} for type u32",
                            u32::MIN,
                            u32::MAX
                        )),
                        span,
                        ErrLevel::WARNING,
                    ));
                }
            }
            Type::U64 => {
                if value < u64::MIN as i128 || value > u64::MAX as i128 {
                    let warp = value as u64;
                    self.consume(CompileError::new(
                        format!("u16 overflow. value {value} warped to {warp}"),
                        Some(format!(
                            "Use from range {}..{} for type u64",
                            u64::MIN,
                            u64::MAX
                        )),
                        span,
                        ErrLevel::WARNING,
                    ));
                }
            }
            Type::I8 => {
                if value < i8::MIN as i128 || value > i8::MAX as i128 {
                    let warp = value as i8;
                    self.consume(CompileError::new(
                        format!("u16 overflow. value {value} warped to {warp}"),
                        Some(format!(
                            "Use from range {}..{} for type i8",
                            i8::MIN,
                            i8::MAX
                        )),
                        span,
                        ErrLevel::WARNING,
                    ));
                }
            }
            Type::I16 => {
                if value < i16::MIN as i128 || value > i16::MAX as i128 {
                    let warp = value as i16;
                    self.consume(CompileError::new(
                        format!("u16 overflow. value {value} warped to {warp}"),
                        Some(format!(
                            "Use from range {}..{} for type i16",
                            i16::MIN,
                            i16::MAX
                        )),
                        span,
                        ErrLevel::WARNING,
                    ));
                }
            }
            Type::I32 => {
                if value < i32::MIN as i128 || value > i32::MAX as i128 {
                    let warp = value as i32;
                    self.consume(CompileError::new(
                        format!("u16 overflow. value {value} warped to {warp}"),
                        Some(format!(
                            "Use from range {}..{} for type i32",
                            i32::MIN,
                            i32::MAX
                        )),
                        span,
                        ErrLevel::WARNING,
                    ));
                }
            }
            Type::I64 => {
                if value < i64::MIN as i128 || value > i64::MAX as i128 {
                    let warp = value as i64;
                    self.consume(CompileError::new(
                        format!("u16 overflow. value {value} warped to {warp}"),
                        Some(format!(
                            "Use from range {}..{} for type i64",
                            i64::MIN,
                            i64::MAX
                        )),
                        span,
                        ErrLevel::WARNING,
                    ));
                }
            }
            Type::INT => {
                if value < i64::MIN as i128 || value > i64::MAX as i128 {
                    let warp = value as i64;
                    self.consume(CompileError::new(
                        format!("u16 overflow. value {value} warped to {warp}"),
                        Some(format!(
                            "Use from range {}..{} for type int a.k.a i64",
                            i64::MIN,
                            i64::MAX
                        )),
                        span,
                        ErrLevel::WARNING,
                    ));
                }
            }
            _ => {}
        }
    }

    fn set_ref(&mut self, val: Var) {
        self.current_context.set_ref(val);
    }

    fn is_castable(&mut self, ty: Type, type_hint: Type) -> bool {
        let a = self.resolve_type(ty);
        let b = self.resolve_type(type_hint);
        match (a, b) {
            (Type::PTR(_v), Type::PTR(_f)) => true,
            (Type::STR, Type::PTR(_v)) => true,
            (Type::PTR(_v), Type::STR) => true,
            (v, Type::PTR(_v)) => is_int(v),
            (Type::PTR(_v), v) => is_int(v),
            (int1, int2) => is_int(int1.clone()) && is_int(int2.clone()) || int1 == int2,
        }
    }

    fn is_iterable(&mut self, clone: Type) -> bool {
        match clone.clone() {
            Type::PTR(_v) => true,
            Type::List(_, _) => true,
            Type::STR => true,
            _ => false,
        }
    }

    fn get_field_item(&mut self, layout: StructLayout, name: &str, span: Span) -> Function {
        let m = layout.methods.iter().find(|p| p.get_name() == name);
        if m.is_none() {
            self.consume(CompileError::new(
                format!("Type '{}' has no method '{}'", layout.name.debug(), name),
                None,
                span,
                ErrLevel::ERROR,
            ));
            self.flush();
        }
        m.unwrap().clone()
    }

    fn convert_vecnode_nodeprogram(&self, functions: Vec<Spanned<Node>>) -> Box<Spanned<Node>> {
        return Box::new(Spanned {
            node: Node::Program(functions),
            span: Span { start: 0, end: 0 },
        });
    }
}

pub fn is_builtin(clone: Type) -> bool {
    match clone {
        Type::Custom(_) => false,
        Type::BundledType { bundle: _, ty: _ } => false,
        _ => true,
    }
}

fn is_int(target_type: Type) -> bool {
    use crate::backend::ttype::Type::*;
    match target_type {
        INT | U8 | U64 | U32 | U16 | I8 | I16 | I32 | I64 | CHAR => true,
        _ => false,
    }
}
