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
use super::function::{Function, Generic_Function};
use super::ttype::Type;
use super::type_table::TTable;
use super::var::Var;
use super::var_table::VTable;

#[derive(Debug, Clone)]
struct FResult {
    stream: String,
    type_hint: Type,
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
    bundled: Vec<String>,
    bundles: Vec<Bundle>,
    cur_body_type: BodyType,
    cur_offset: u16,
    cur_section: Section,
    current_context: Context,
    current_file: String,
    data: String,
    errorbox: Vec<CompileError>,
    func: String,
    func_table: Vec<Function>,
    generics: Vec<Generic>,
    gfunc_table: Vec<Generic_Function>,
    global_context: Context,
    has_error: bool,
    header: String,
    immediate_counter: u8,
    inputpath: String,
    is_included: bool,
    outfilename: String,
    outfilepath: String,
    pub source: Vec<Spanned<Node>>,
    source_code: String,
    pub text: String,
    track_rsp: bool,
    types: TTable,
    var_table: VTable,
    buildpath: Box<PathBuf>,
    generic_stream: String,
    generic_headers: Vec<(String, String)>, // path, content
    current_scope_return_type: Type,        // Validating return statements
}
#[derive(Debug, Clone)]
pub struct Generic {
    name: Type,
    generics: Vec<Type>,
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
            cur_offset: 1,
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
            generics: vec![],
            gfunc_table: vec![],
            buildpath: Box::new(Path::new(&builddir).to_path_buf()),
            generic_stream: String::new(),
            generic_headers: vec![],
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
            "\n#include \"/home/dry/Documents/Eggo/jaguar/std/claw.h\"\n#include \"{}\"",
            format!("{}/__generics__.h", self.buildpath.to_str().unwrap())
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
                Node::NameSpace { alias, body } => {
                    let sc = self.current_context.clone();
                    let st = self.types.clone();
                    let sf = self.func_table.clone();
                    let sb = self.bundles.clone();
                    self.current_context = Context::new(alias.clone(), Some(Box::new(sc.clone())));
                    self.types = TTable::new();
                    self.bundles = vec![];
                    self.generate(match body.clone().node {
                        Node::Program(k) => k.clone(),
                        _ => {
                            self.consume(CompileError::new(
                                "Fatal Error".into(),
                                None,
                                body.clone().span,
                                ErrLevel::ERROR,
                            ));
                            self.flush();
                            exit(1); /* should not reach here */
                        }
                    });
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
                    let ext = path.split_at(path.len() - 3).1;
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
                            new_bundle.gfunctions = cgen.gfunc_table.clone();
                            for f in &mut new_bundle.functions {
                                for arg in &mut f.args {
                                    if !is_builtin(arg.type_hint.clone()) {
                                        arg.type_hint = Type::BundledType {
                                            bundle: alias.clone(),
                                            ty: Box::new(arg.type_hint.clone()),
                                        }
                                    }
                                }
                                if !is_builtin(f.ty.clone()) {
                                    f.ty = Type::BundledType {
                                        bundle: alias.clone(),
                                        ty: Box::new(f.ty.clone()),
                                    };
                                }
                            }
                            for gf in &mut new_bundle.gfunctions {
                                let g = gf.clone();
                                for arg in &mut gf.args {
                                    println!("Wrapping ({}, {:?})", arg.name.clone(), g);
                                    if !is_builtin(arg.type_hint.clone())
                                        && !gf.generics.contains(&arg.type_hint)
                                    {
                                        arg.type_hint = Type::BundledType {
                                            bundle: alias.clone(),
                                            ty: Box::new(arg.type_hint.clone()),
                                        };
                                    }
                                }
                                if !is_builtin(gf.ty.clone()) {
                                    gf.ty = Type::BundledType {
                                        bundle: alias.clone(),
                                        ty: Box::new(gf.ty.clone()),
                                    }
                                }
                            }
                            new_bundle.types.wrap(alias.clone());
                            self.bundles.push(new_bundle);
                            self.generics.append(&mut cgen.generics);
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
                            if self.verify_type(type_hint.clone()) != None {
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
                                Some(plug) => {
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
                                ret_type.clone().to_str(),
                                targ_type.to_str(),
                                fname.clone()
                            ));
                            for (i, a) in args.clone().iter_mut().enumerate() {
                                let mut modif = "";
                                if a.name == "self" {
                                    modif = "*";
                                    a.type_hint = targ_type.clone();
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
                                    returns,
                                    return_val,
                                    vardaic,
                                    mangled_name,
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
                    self.cur_section = save;
                }
                Node::GenericStructStmt {
                    name,
                    fields,
                    mut meths,
                    generics,
                } => {
                    self.cur_body_type = BodyType::GENERIC;
                    let layout: StructLayout;
                    let mut b_fields: HashMap<String, FieldLayout> = HashMap::new();
                    let save = self.cur_section.clone();
                    self.cur_section = Section::HEADER;
                    let generic_header_path = format!(
                        "{}/generic_{}.h",
                        self.buildpath.to_str().unwrap(),
                        self.outfilename
                    );
                    let mut generic_entry = self
                        .generic_headers
                        .iter_mut()
                        .find(|p| p.0 == generic_header_path)
                        .cloned();
                    if generic_entry.is_none() {
                        self.generic_headers.push((generic_header_path.clone(), format!("#pragma once\n\n#include \"/home/dry/Documents/Eggo/jaguar/std/claw.h\"\n")));
                    }
                    let mut stream = String::new();
                    stream.push_str(&format!(
                        "\n\n#define GENERIC_BLOCK_{}(",
                        name.to_uppercase()
                    ));
                    let mut gname = name.clone();
                    for (i, generic) in generics.clone().iter().enumerate() {
                        stream.push_str(&format!("{}", generic));
                        gname += &format!("_##{generic}");
                        if i != generics.len() - 1 {
                            stream.push_str(",");
                            gname += "##";
                        }
                    }
                    stream.push_str(") ");
                    stream.push_str(format!("typedef struct {gname} {{\\").as_str());
                    let mut encountered_fields: Vec<(String, Span)> = vec![];
                    for (_i, field) in fields.clone().iter().enumerate() {
                        if let Node::Feilds {
                            name: fname,
                            type_hint,
                        } = field.node.clone()
                        {
                            if let Some(find) = encountered_fields.iter().find(|p| p.0 == fname) {
                                self.consume(CompileError::new(
                                    format!("Redefinition of field '{fname}'"),
                                    None,
                                    field.span.clone(),
                                    ErrLevel::ERROR,
                                ));
                                self.consume(CompileError::new(
                                    format!("Previously Defined here"),
                                    None,
                                    find.1.clone(),
                                    ErrLevel::WARNING,
                                ));
                                self.flush();
                            }
                            stream.push_str(
                                format!("\n\t{} {fname}; \\", type_hint.to_str()).as_str(),
                            );
                            encountered_fields.push((fname.clone(), field.span.clone()));
                            b_fields.insert(fname.clone(), FieldLayout { ty: type_hint });
                        }
                    }
                    let g = self.gfromstr(generics.clone());
                    layout = StructLayout {
                        name: Type::Custom(name.clone()),
                        feilds: b_fields.clone(),
                        methods: Vec::new(),
                        file: self.current_file.clone(),
                    };
                    self.types.add_type(Type::Custom(name.clone()), layout);
                    stream.push_str(format!("\n}} {gname}; \\").as_str());
                    let type_generics = self.gfromstr(generics.clone());
                    for mut m in &mut meths {
                        if let Ok(Spanned {
                            node:
                                Node::FnStmt {
                                    body: _,
                                    args,
                                    name: _,
                                    ret_type: _,
                                    returns,
                                    return_val,
                                    vardaic: _,
                                    mangled_name: _,
                                },
                            span: _,
                        }) = m
                        {
                            for a in args {
                                if type_generics.contains(&a.type_hint) {
                                    a.type_hint = Type::GenericAtom {
                                        ty: Box::new(a.type_hint.clone()),
                                    };
                                }
                            }
                        }
                    }
                    for m in meths.clone() {
                        if let Spanned {
                            node:
                                Node::FnStmt {
                                    body,
                                    args,
                                    name: fname,
                                    ret_type,
                                    returns,
                                    return_val,
                                    vardaic,
                                    mangled_name,
                                },
                            span,
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
                                Some(plug) => {
                                    self.consume(CompileError::new(
                                        format!(
                                            "Method {} already exsist for type {}",
                                            fname, name
                                        ),
                                        None,
                                        span.clone(),
                                        ErrLevel::ERROR,
                                    ));
                                }
                                None => (),
                            }
                            let mut context = Context::new(
                                name.clone(),
                                Some(Box::new(self.current_context.clone())),
                            );
                            let mut oplugin_name = String::new();
                            let v = Some(g.clone());
                            if v.is_some() {
                                oplugin_name += &targ_type.to_str();
                                for t in v.clone().unwrap().iter().enumerate() {
                                    oplugin_name += &format!("_##{}", t.1.to_str());
                                    if t.0 == v.clone().unwrap().len() - 1 {
                                        oplugin_name += "##_";
                                    }
                                }
                            }
                            oplugin_name += &fname.clone();
                            if let Type::Generic {
                                base: _,
                                generics: _,
                            } = ret_type.clone()
                            {
                                stream += &format!(
                                    "\n{}\\",
                                    self.generate_generic_block_instance(ret_type.clone())
                                );
                            }
                            stream.push_str(&format!(
                                "\nextern inline {} {}(",
                                ret_type.clone().genimpl(),
                                oplugin_name.clone()
                            ));
                            for (i, a) in args.clone().iter_mut().enumerate() {
                                let mut modif = "";
                                if a.name == "self" {
                                    modif = "*";
                                    a.type_hint = targ_type.clone();
                                    stream.push_str(&format!("{gname}{modif} {}", a.name.clone()));
                                } else {
                                    stream.push_str(&format!(
                                        "{}{modif} {}",
                                        a.type_hint.clone().to_str(),
                                        a.name.clone()
                                    ));
                                }
                                if i != args.clone().len() - 1 {
                                    stream.push_str(",");
                                }
                                context.add(Var::new(
                                    a.name.clone(),
                                    a.type_hint.clone(),
                                    a.is_ref,
                                    None,
                                    m.clone().unwrap().span,
                                ));
                            }
                            stream.push_str("); \\");
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
                    self.generics.push(Generic {
                        name: Type::Custom(name.clone()),
                        generics: g,
                    });
                    for meth in meths.clone() {
                        if let Spanned {
                            node:
                                Node::FnStmt {
                                    body,
                                    args,
                                    name: fname,
                                    ret_type,
                                    returns,
                                    return_val,
                                    vardaic,
                                    mangled_name,
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

                            let o = self.gen_expr(
                                Box::new(Spanned { node: p, span }),
                                Type::Any,
                                RefStyle::COPY,
                            );
                            stream += &o.stream;
                        }
                    }
                    self.cur_section = save;
                    let mut g = vec![];
                    for s in generics {
                        g.push(Type::Custom(s));
                    }
                    self.generics.push(Generic {
                        name: Type::Custom(name),
                        generics: g,
                    });
                    let mut fgeneric_entry = self
                        .generic_headers
                        .iter_mut()
                        .find(|p| p.0 == generic_header_path);
                    if fgeneric_entry.is_some() {
                        fgeneric_entry.unwrap().1 += &stream.clone();
                    }
                    self.cur_body_type = BodyType::NORMAL;
                }
                Node::ReVal { name, value } => {
                    let s = self.gen_expr(Box::new(node), Type::Any, RefStyle::COPY);
                    self.emit(&format!("\n\t{};", s.stream));
                }
                Node::IfStmt {
                    cond,
                    body,
                    elseifs,
                    elsestmt,
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
                Node::FnStmt {
                    body,
                    name,
                    ret_type,
                    returns,
                    return_val,
                    args,
                    vardaic,
                    mangled_name,
                } => {
                    let saved_type = self.current_scope_return_type.clone();
                    self.current_scope_return_type = ret_type.clone();
                    let mut fn_ret = ret_type.clone();
                    let mut fmangled_name = name.clone();
                    if name == "main".to_string() {
                        fmangled_name = name.clone();
                        fn_ret = Type::INT;
                    }
                    self.emit("\n");
                    self.change_scope(name.as_str());
                    self.cur_section = Section::FUNC;
                    self.emit("\n");
                    if self.is_included {
                        self.emit("extern inline ");
                    }
                    if self.type_match(fn_ret.clone(), Type::NoType) && name != "main" {
                        self.emit(format!("void {name}(").as_str());
                    } else {
                        self.emit(format!("{} {name} (", fn_ret.to_str()).as_str());
                    }
                    let mut index: u16 = 0;
                    for arg in args.clone() {
                        self.emit(format!("{} {}", arg.type_hint.to_str(), arg.name).as_str());
                        if index != (args.len() - 1 as usize) as u16 {
                            self.emit(",");
                        }
                        let v = Var::new(arg.name, arg.type_hint, false, None, node.clone().span);
                        self.var_table.add(v.clone());
                        self.current_context.add(v);
                        index += 1;
                    }
                    self.emit(") {");
                    self.track_rsp = true;
                    match body.node.clone() {
                        Node::Program(k) => {
                            for node in k.clone() {
                                let o = self.gen_expr(
                                    Box::new(node.clone()),
                                    Type::Any,
                                    RefStyle::COPY,
                                );
                                self.emit(&o.stream);
                                if let Node::IfStmt {
                                    cond,
                                    body,
                                    elseifs,
                                    elsestmt,
                                } = node.node
                                {
                                } else {
                                    self.emit(";");
                                }
                            }
                        }
                        _ => {}
                    }

                    if name == "main" && fn_ret == Type::INT {
                        self.emit("\n\treturn 0;");
                    }
                    self.emit("\n}\n");
                    let mut func = Function::new(
                        name,
                        self.current_context.clone(),
                        fn_ret.clone(),
                        returns,
                        body.node,
                    );
                    func.args = args.clone();
                    self.func_table.push(func);
                    self.exit_scope();
                    self.cur_section = Section::TEXT;
                    self.current_scope_return_type = saved_type;
                    continue;
                }
                Node::GenericFnCall {
                    callee,
                    generics,
                    args,
                } => {
                    let t = self.gen_expr(Box::new(node), Type::Any, RefStyle::COPY);
                    self.emit(&format!("{};", t.stream));
                }
                Node::GenericFnStmt {
                    generics,
                    body,
                    args,
                    name,
                    ret_type,
                    returns,
                    return_val,
                    vardaic,
                    mangled_name,
                } => {
                    let saved_type = self.current_scope_return_type.clone();
                    self.current_scope_return_type = ret_type.clone();
                    self.cur_body_type = BodyType::GENERIC;
                    let is_generic = self.cur_body_type == BodyType::GENERIC;
                    let mut fix = "\\";
                    if !is_generic {
                        fix = "";
                    }
                    self.cur_section = Section::FUNC;
                    let mut stream = String::new();
                    let instantiator_name = format!("GENERIC_FN_{name}");
                    let generic_header_path = format!(
                        "{}/generic_{}.h",
                        self.buildpath.to_str().unwrap(),
                        self.outfilename
                    );
                    let mut generic_entry = self
                        .generic_headers
                        .iter_mut()
                        .find(|p| p.0 == generic_header_path)
                        .cloned();
                    if generic_entry.is_none() {
                        self.generic_headers.push((generic_header_path.clone(), format!("#pragma once\n\n#include \"/home/dry/Documents/Eggo/jaguar/std/claw.h\"\n")));
                    }
                    let mut stream = String::new();
                    stream.push_str(&format!("\n\n#define {instantiator_name}("));
                    for (i, g) in generics.clone().iter().enumerate() {
                        stream.push_str(&format!("{g}"));
                        if i != generics.len() - 1 {
                            stream.push_str(",");
                        }
                    }
                    stream.push_str(") ");
                    let mut fn_ret = ret_type.clone();
                    let mut fmangled_name = name.clone();
                    self.change_scope(name.as_str());
                    self.cur_section = Section::FUNC;
                    let saved_offset: u16 = self.cur_offset;
                    self.cur_offset = 1;
                    if self.is_included {
                        stream.push_str(&format!("extern inline "));
                    }
                    let mut gname = name.clone();
                    for (i, g) in generics.iter().enumerate() {
                        gname += &format!("_##{g}");
                    }
                    stream.push_str(&format!("{} {gname} (", fn_ret.genimpl()));
                    let mut index: u16 = 0;
                    for arg in args.clone() {
                        stream.push_str(&format!("{} {}", arg.type_hint.to_str(), arg.name));
                        if index != (args.len() - 1 as usize) as u16 {
                            stream.push_str(&format!(","));
                        }
                        let v = Var::new(arg.name, arg.type_hint, false, None, node.clone().span);
                        self.var_table.add(v.clone());
                        self.current_context.add(v);
                        index += 1;
                    }
                    stream.push_str(&format!(") {{{fix}"));
                    match body.node.clone() {
                        Node::Program(k) => {
                            for node in k.clone() {
                                let s = self.gen_expr(Box::new(node), Type::Any, RefStyle::COPY);
                                stream += &format!("\n{};{fix}", s.stream);
                            }
                        }
                        _ => {}
                    }

                    stream.push_str(&format!("\n}}\n"));
                    let mut func = Function::new(
                        name.clone(),
                        self.current_context.clone(),
                        fn_ret.clone(),
                        returns,
                        body.node,
                    );
                    func.gen_name = gname;
                    func.args = args.clone();
                    self.gfunc_table
                        .push(func.to_generic(self.gfromstr(generics.clone())));
                    self.exit_scope();
                    self.cur_offset = saved_offset;
                    self.cur_section = Section::TEXT;
                    let mut g = vec![];
                    for s in generics {
                        g.push(Type::Custom(s));
                    }
                    let fgeneric_entry = self
                        .generic_headers
                        .iter_mut()
                        .find(|p| p.0 == generic_header_path);
                    if fgeneric_entry.is_some() {
                        fgeneric_entry.unwrap().1.push_str(&stream.clone());
                    }
                    self.generics.push(Generic {
                        name: Type::Custom(name),
                        generics: g,
                    });
                    self.cur_body_type = BodyType::NORMAL;
                    self.current_scope_return_type = saved_type;
                    continue;
                }
                Node::BinaryExpr { lhs, opr, rhs } => {}
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
                    extrnfunc.args = params;
                    extrnfunc.variadic = vardaic;
                    self.func_table.push(extrnfunc);
                    self.cur_section = Section::HEADER;
                    self.emit(&stream);
                    self.cur_section = Section::TEXT;
                    continue;
                }
                Node::FcCall { params, callee } => {
                    self.emit("\n\t");
                    let out = self.gen_func_call(Box::new(node.clone()), Type::Any);
                    self.emit(out.stream.as_str());
                    self.emit(";");
                }
                Node::PluginStatement {
                    name,
                    ret_val,
                    ret_type,
                    body,
                    targ_type,
                    args,
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
                        let mut t = b.types.getLayout(Type::Custom(sym.clone()));
                        let tt = self.types.getLayout(Type::Custom(sym.clone()));
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
                        let g = b.gfunctions.iter().find(|f| f.get_name() == sym);
                        let gtf = self.func_table.iter().find(|f| f.get_name() == sym);
                        if g.is_some() {
                            if gtf.is_some() {
                                self.consume(CompileError::new(
                                    format!("Conflicting symbol {sym}. Function with this name already exists in the scope"), None, node.clone().span, ErrLevel::ERROR
                                ));
                                self.flush();
                            }
                            self.gfunc_table.push(g.unwrap().clone());
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
        for (path, content) in self.generic_headers.clone() {
            let mut f = File::create(path.clone()).unwrap();
            f.write(content.as_bytes()).unwrap();
            self.generic_stream
                .insert_str(0, &format!("#pragma once\n\n#include \"{path}\""));
        }
        let mut f = File::create(format!(
            "{}/__generics__.h",
            self.buildpath.to_str().unwrap()
        ))
        .unwrap();
        f.write(self.generic_stream.as_bytes()).unwrap();
    }
    fn name_mangler(&mut self, input: String) -> String {
        let _ = input;
        let new_name: String = String::new();
        let _prefix = "_Tixie";
        let _path = self.outfilename.clone();
        new_name
    }
    fn gen_expr(
        &mut self,
        expression: Box<Spanned<Node>>,
        target_type: Type,
        is_ref: RefStyle,
    ) -> ExprResult {
        let mut stream = String::new();
        let v_is_ref = false;
        let v_is_deref = false;
        let v_is_moved = false;
        let expr = expression.as_ref();
        match expr.node.clone() {
            Node::LiteralInt(num) => {
                let is_generic = self.cur_body_type == BodyType::GENERIC;
                let mut fix = "\\";
                if !is_generic {
                    fix = "";
                }

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
                    stream,
                    is_ref: false,
                    refed_var: None,
                    type_hint: Box::new(Type::INT),
                    var: None,
                };
            }
            Node::LiteralStr(value) => {
                let is_generic = self.cur_body_type == BodyType::GENERIC;
                let mut fix = "\\";
                if !is_generic {
                    fix = "";
                }
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
                    stream: format!("\"{value}\""),
                    is_ref: false,
                    refed_var: None,
                    type_hint: Box::new(Type::STR),
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
                    stream: format!("\'{value}\'"),
                    is_ref: false,
                    refed_var: None,
                    type_hint: Box::new(Type::CHAR),
                    var: Some(format!("(const char*)\"{value}\"")),
                };
            }
            Node::GenericFnCall {
                callee,
                generics,
                args,
            } => {
                let is_generic = self.cur_body_type == BodyType::GENERIC;
                let mut fix = "\\";
                if !is_generic {
                    fix = "";
                }
                match callee.clone().node {
                    Node::Token(v, d) => {
                        let fnc = self.gfunc_table.iter().find(|p| p.get_name() == v).cloned();
                        if fnc.is_none() {
                            self.consume(CompileError::new(
                                format!("Use of undeclared symbol '{v}'"),
                                None,
                                callee.clone().span,
                                ErrLevel::ERROR,
                            ));
                            self.flush();
                            exit(1);
                        }
                        let fnm = fnc.unwrap().clone();
                        let mut new_func = fnm.clone().to_function();
                        match fnm.ty.clone() {
                            Type::Generic {
                                base,
                                generics: grcs,
                            } => {
                                new_func.ty = Type::Generic {
                                    base: base.clone(),
                                    generics: generics.clone(),
                                };
                                if self
                                    .generics
                                    .iter()
                                    .find(|p| p.name == fnm.ty.clone())
                                    .is_none()
                                {
                                    let mut l = self.get_layout(fnm.ty);
                                    let mut nl = l.clone().unwrap();
                                    for f in l.unwrap().feilds.iter().enumerate() {
                                        if grcs.iter().find(|p| **p == f.1.1.ty).is_some() {
                                            nl.feilds.get_mut(f.1.0).unwrap().ty =
                                                generics.get(f.0).unwrap_or(&Type::Any).clone();
                                        }
                                    }
                                    nl.name = new_func.ty.clone();
                                    self.types.add_type(nl.name.clone(), nl);
                                }
                                for (i, arg) in new_func.clone().args.iter().enumerate() {
                                    if grcs.iter().find(|p| **p == arg.type_hint).is_some() {
                                        new_func.args.get_mut(i).unwrap().type_hint =
                                            generics.get(i).unwrap().clone();
                                    }
                                }
                                let mut gname = new_func.get_name();
                                stream += &format!("\nGENERIC_FN_{v}(");
                                for t in generics.iter().enumerate() {
                                    stream += &format!("{}", t.1.to_str());
                                    gname += &format!("_{}", t.1.to_str());
                                    if t.0 != generics.len() - 1 {
                                        stream += &format!(",");
                                    }
                                }
                                new_func.name = gname.clone();
                                self.func_table.push(new_func.clone());
                                stream += &format!("){fix}\n");
                                let sv = self.cur_section.clone();
                                self.cur_section = Section::FUNC;
                                let n = Spanned {
                                    node: Node::FcCall {
                                        params: args,
                                        callee: Box::new(Spanned {
                                            node: Node::Token(gname, false),
                                            span: callee.span,
                                        }),
                                    },
                                    span: expr.span.clone(),
                                };
                                let f = self.gen_func_call(Box::new(n), target_type);
                                self.generic_stream.push_str(&stream);
                                stream = f.stream;
                                self.cur_section = sv;
                                return ExprResult {
                                    stream,
                                    is_ref: false,
                                    refed_var: None,
                                    type_hint: Box::new(f.type_hint),
                                    var: None,
                                };
                            }
                            t => {
                                println!("{generics:?}");
                                new_func.ty = t.clone();
                                let mut gname = new_func.get_name();
                                stream += &format!("\nGENERIC_FN_{v}(");
                                for t in generics.iter().enumerate() {
                                    stream += &format!("{}", t.1.to_str());
                                    gname += &format!("_{}", t.1.to_str());
                                    if t.0 != generics.len() - 1 {
                                        stream += &format!(",");
                                    }
                                }
                                new_func.name = gname.clone();
                                let idx = 0;
                                for a in new_func.args.clone() {}
                                self.func_table.push(new_func.clone());
                                stream += &format!("){fix}\n");
                                let sv = self.cur_section.clone();
                                self.cur_section = Section::FUNC;
                                let n = Spanned {
                                    node: Node::FcCall {
                                        params: args,
                                        callee: Box::new(Spanned {
                                            node: Node::Token(gname, false),
                                            span: callee.span,
                                        }),
                                    },
                                    span: expr.span.clone(),
                                };
                                let f = self.gen_func_call(Box::new(n), target_type);
                                self.generic_stream.push_str(&stream);
                                stream = f.stream;
                                self.cur_section = sv;
                                return ExprResult {
                                    stream,
                                    is_ref: false,
                                    refed_var: None,
                                    type_hint: Box::new(f.type_hint),
                                    var: None,
                                };
                            }
                        }
                    }
                    _ => {}
                }
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
                if let Type::list(t, n) = target_type.clone() {
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
                    type_hint: Box::new(Type::list(list_type, list_size)),
                    var: Some(stream),
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
                        if let Type::list(v, c) = fl.clone() {
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
                            stream: stream.clone(),
                            is_ref: false,
                            refed_var: None,
                            type_hint: Box::new(ty),
                            var: Some("foo".to_owned()),
                        };
                    }
                    Node::Token(v, t) => {
                        let t = self.gen_expr(name.clone(), target_type, RefStyle::COPY);
                        let fl = *t.type_hint.clone();
                        let mut ty = Type::NoType;
                        if !self.is_iterable(fl.clone()) {}
                        let i = self.gen_expr(index, Type::Any, RefStyle::COPY);
                        if let Type::list(v, c) = fl.clone() {
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
                if let Type::Generic { base, generics } = target_type.clone() {
                    let l = self.get_layout(*base.clone());
                    if !l.is_some() {
                        self.consume(CompileError::new(
                            format!("Not a type, {}", block_name.debug()),
                            None,
                            expr.clone().span,
                            ErrLevel::ERROR,
                        ));
                        self.flush();
                    }
                    let v_fields = l.clone().unwrap().feilds;
                    if self.cur_body_type == BodyType::GENERIC {
                        stream += &format!("({}) {{", block_name.genimpl());
                    } else {
                        stream += &format!("({}) {{", block_name.to_str());
                    }
                    for (i, b) in block_fields.clone().iter().enumerate() {
                        if let Node::Pair { field, value } = b.clone().node {
                            if v_fields.contains_key(&field) {
                                let f = v_fields.get(&field).unwrap();
                                let o = self.gen_expr(value, Type::Any, RefStyle::COPY);
                                stream += &format!(".{field} = {}", o.stream);
                                if i != block_fields.len() - 1 {
                                    stream += ","
                                }
                            }
                        }
                    }
                    stream += "}";
                    // Swapping all generic types in old layout and creating a new layout
                    let mut new_layout = l.clone().unwrap();
                    let g = self.generics.iter().find(|p| p.name == *base);
                    let mut gidx = 0;
                    for field in &mut new_layout.feilds {
                        if g.unwrap().generics.contains(&field.1.ty) {
                            field.1.ty = generics.get(gidx).unwrap().clone();
                            gidx += 1;
                        }
                    }
                    self.types.add_type(target_type.clone(), new_layout); // adding new layout to type registary
                    return ExprResult {
                        stream: stream.clone(),
                        is_ref: false,
                        refed_var: None,
                        type_hint: Box::new(target_type),
                        var: Some(stream),
                    };
                }
                if self.verify_type(block_name.clone()) == None {
                    self.consume(CompileError::new(
                        format!("Not a type, {}", block_name.debug()),
                        None,
                        expr.clone().span,
                        ErrLevel::ERROR,
                    ));
                    self.flush();
                }
                let _block_size = self.verify_type(block_name.clone());
                let mut t = self.resolve_type(block_name.clone());
                if let Type::PTR(v) = t {
                    t = *v;
                }
                let layout = self.get_layout(t);
                if matches!(layout, None) {
                    self.consume(CompileError::new(
                        format!("Not a type, {}", block_name.debug()),
                        None,
                        expr.clone().span,
                        ErrLevel::ERROR,
                    ));
                    self.flush();
                }
                stream += "{";
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
                    stream: stream.clone(),
                    is_ref: false,
                    refed_var: None,
                    type_hint: Box::new(block_name.clone()),
                    var: Some(stream),
                };
            }
            Node::RefExpr { expr } => {
                let out = self.gen_expr(expr.clone(), target_type, RefStyle::REF);
                if let Node::Token(name, is_deref) = &expr.node.clone() {
                    if let Some(var) = self.lookup_variable(name).cloned() {
                        self.set_ref(var);
                    }
                }
                stream += &format!("&{}", out.stream);
                return ExprResult {
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
                if let Node::Token(name, is_deref) = &expr.node {
                    v_var = Some(name.clone());

                    if let Some(var) = self.lookup_variable(name).cloned() {
                        type_hint = var.type_hint.clone();
                    }
                }
                if let Type::PTR(v) = *out.clone().type_hint {
                    type_hint = *v;
                }
                return ExprResult {
                    stream,
                    is_ref: false,
                    refed_var: out.refed_var.clone(),
                    type_hint: Box::new(type_hint),
                    var: v_var,
                };
            }
            Node::Token(var, is_deref) => {
                if let Some(val) = self.lookup_variable(var.as_str()).cloned() {
                    match is_ref {
                        RefStyle::DEREF => {
                            stream += format!("{}", val.name).as_str();
                            if !val.is_ref {
                                self.consume(CompileError::new(
                                    format!("Cannot dereference value at '{var}'. Not a reference"),
                                    None,
                                    expr.clone().span,
                                    ErrLevel::ERROR,
                                ));
                                self.flush();
                            }
                            let mut type_hint = val.type_hint.clone();

                            if let Type::PTR(ty) = val.type_hint.clone() {
                                type_hint = *ty.clone();
                            }
                            return ExprResult {
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
                                stream,
                                is_ref: true,
                                refed_var: Some(var.clone()),
                                type_hint: Box::new(val.clone().type_hint),
                                var: Some(var),
                            };
                        }
                        _ => (),
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
                    stream,
                    is_ref: v_is_ref,
                    refed_var: None,
                    type_hint: Box::new(ty),
                    var: None,
                };
            }
            Node::Ret(v) => {
                let ig = self.cur_body_type == BodyType::GENERIC;
                let mut fix = "";
                if ig {
                    fix == "\\";
                }
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
                stream += &format!("{fix}return {}", out.stream);
                return ExprResult {
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
                    if let Node::FcCall { params, callee } = field.node.clone() {
                        let save = self.func_table.clone();
                        self.func_table = bndl.unwrap().functions.clone();
                        let res = self.gen_func_call(field, target_type);
                        self.func_table = save;
                        stream += format!("\n\t{}", res.stream).as_str();
                        return ExprResult {
                            stream,
                            is_ref: false,
                            refed_var: None,
                            type_hint: Box::new(res.type_hint.clone()),
                            var: None,
                        };
                    } else if let Node::BundleAccess {
                        base: b2,
                        field: f2,
                    } = field.clone().node
                    {
                        let sb = self.bundles.clone();
                        self.bundles = bndl.clone().unwrap().bundles.clone();
                        let out2 = self.gen_expr(field, target_type, is_ref);
                        stream += &out2.stream;
                        self.bundles = sb.clone();
                        return ExprResult {
                            stream,
                            is_ref: false,
                            refed_var: None,
                            type_hint: Box::new(*out2.type_hint.clone()),
                            var: None,
                        };
                    } else if let Node::GenericFnCall {
                        callee,
                        generics,
                        args,
                    } = field.node.clone()
                    {
                        let save_function_table = self.func_table.clone();
                        let save_gfunction_table = self.gfunc_table.clone();
                        let save_types = self.types.clone();
                        self.func_table = bndl.unwrap().functions.clone();
                        self.gfunc_table = bndl.unwrap().gfunctions.clone();
                        self.types = bndl.unwrap().types.clone();
                        let res = self.gen_expr(field, target_type, RefStyle::COPY);
                        self.func_table = save_function_table;
                        self.gfunc_table = save_gfunction_table;
                        self.types = save_types;
                        stream += format!("\n\t{}", res.stream).as_str();
                        return ExprResult {
                            stream,
                            is_ref: false,
                            refed_var: None,
                            type_hint: res.type_hint.clone(),
                            var: None,
                        };
                    } else {
                        let out = self.gen_expr(field, target_type, RefStyle::COPY);
                        stream += &out.stream;
                        return ExprResult {
                            stream,
                            is_ref: false,
                            refed_var: None,
                            type_hint: Box::new(*out.type_hint),
                            var: None,
                        };
                    }
                }
            }
            Node::FcCall { params, callee } => {
                let out = self.gen_func_call(expression.clone(), target_type);
                stream += out.clone().stream.as_str();
                return ExprResult {
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
                let mut t = self.resolve_type(*out.type_hint.clone());
                if let Type::PTR(v) = t {
                    t = *v;
                }
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
                let mut type_hint: Box<Type> = Box::new(Type::NoType);
                if !layout.as_ref().unwrap().feilds.contains_key(&field.clone()) {
                    self.consume(CompileError::new(
                        format!("Type {} has no field {field}", out.type_hint.debug()),
                        None,
                        base.clone().span,
                        ErrLevel::ERROR,
                    ));
                    self.flush();
                    exit(100);
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
                    if var.is_ref {
                        stream += format!("->{}", field.clone()).as_str();
                    } else {
                        stream += format!(".{}", field.clone()).as_str();
                    }
                    type_hint = Box::new(v_field.ty);
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
                    stream,
                    is_ref: false,
                    refed_var: None,
                    type_hint,
                    var: out.var,
                };
            }
            Node::ReVal { name, value } => match name.node.clone() {
                Node::ListAccess { name: lname, index } => {
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
                    if layout.clone().unwrap().clone().feilds.contains_key(&field) {
                        let v_field = layout.clone().unwrap().feilds.get(&field).unwrap().clone();
                        let out = self.gen_expr(value, Type::Any, RefStyle::COPY).clone();
                        let mut modifier = ".";
                        if base_out.is_ref {
                            modifier = "->";
                        }
                        stream += &format!("{modifier}{} = {}", field, out.stream);
                    }
                }
                Node::Token(var, d) => {
                    let val = self.lookup_variable(&var.clone()).unwrap().clone();
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
                ret_val,
                ret_type,
                body,
                mut targ_type,
                args,
            } => {
                let saved_type = self.current_scope_return_type.clone();
                self.current_scope_return_type = *ret_type.clone();
                let plugin_name = name.clone();
                let plugin_type = ret_type.clone();
                let plugin_val = ret_val.clone();
                let is_generic = {
                    let v = self.cur_body_type == BodyType::GENERIC;
                    v
                };
                let mut fix = "\\";
                if !is_generic {
                    fix = "";
                }
                let mut self_type = Type::NoType;
                let mut oplugin_name = String::new();
                oplugin_name += &targ_type.to_str();
                self_type = if is_generic {
                    let v = self.generics.iter().find(|p| p.name == targ_type);
                    if v.is_some() {
                        for t in v.unwrap().generics.iter().enumerate() {
                            oplugin_name += &format!("_##{}", t.1.to_str());
                            if t.0 == v.unwrap().generics.len() - 1 {
                                oplugin_name += "##";
                            }
                        }
                    }
                    Type::Generic {
                        base: Box::new(targ_type.clone()),
                        generics: v.unwrap().generics.clone(),
                    }
                } else {
                    targ_type.clone()
                };
                oplugin_name += &format!("_{}", plugin_name);
                self.change_scope(name.as_str());
                self.cur_section = Section::FUNC;
                stream.push_str(
                    format!(
                        "\nextern inline {} {}(",
                        plugin_type.genimpl().clone(),
                        oplugin_name.clone()
                    )
                    .as_str(),
                );
                for (index, arg) in args.clone().iter_mut().enumerate() {
                    let mut modifier = "";
                    if arg.name.clone() == "self" {
                        modifier = "*";
                        self.current_context.add(Var::new(
                            "self".into(),
                            targ_type.clone(),
                            true,
                            None,
                            expr.clone().span,
                        ));
                        arg.type_hint = self_type.clone();
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
                        arg.type_hint.genimpl(),
                        modifier,
                        arg.name
                    ));
                    if index != args.len() - 1 {
                        stream.push_str(",");
                    }
                }
                stream.push_str(&format!(") {{ {fix}\n"));
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

                stream.push_str(&format!("}}{fix}"));
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
            }
            Node::LetStmt {
                name,
                type_hint,
                value,
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
                let is_generic = self.cur_body_type == BodyType::GENERIC;
                let mut fix = "\\";
                if !is_generic {
                    fix = "";
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
                let mut generic_exists = false;
                let l = self.get_layout(type_hint.clone());
                if !l.is_some() && type_hint != Type::Any {
                    if let Type::Generic { base, generics } = type_hint.clone() {
                        generic_exists = true;
                        let t = self.generics.iter().find(|p| p.name == type_hint);
                        let sv = self.cur_section.clone();
                        self.cur_section = Section::HEADER;
                        let mut hd = String::new();
                        hd += &format!("\nGENERIC_BLOCK_{}(", base.to_str().to_uppercase());
                        for (i, g) in generics.iter().enumerate() {
                            hd += &format!("{}", g.to_str());
                            if i != generics.len() - 1 {
                                hd += ","
                            }
                        }
                        hd += ");\n";
                        self.generic_stream += &hd;
                        self.cur_section = sv;
                    } else {
                        self.consume(CompileError::new(
                            format!("Not a Type, '{}'", type_hint.debug()),
                            None,
                            expr.clone().span,
                            ErrLevel::ERROR,
                        ));
                        self.flush();
                    }
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
                if !generic_exists {
                    if let Type::Generic { base, generics } = *out.type_hint.clone() {
                        generic_exists = true;
                        let t = self.generics.iter().find(|p| p.name == type_hint);
                        let sv = self.cur_section.clone();
                        self.cur_section = Section::HEADER;
                        let mut hd = String::new();
                        hd += &format!("\nGENERIC_BLOCK_{}(", base.to_str().to_uppercase());
                        for (i, g) in generics.iter().enumerate() {
                            hd += &format!("{}", g.to_str());
                            if i != generics.len() - 1 {
                                hd += ","
                            }
                        }
                        hd += ");\n";
                        self.generic_stream.insert_str(0, &hd);
                        self.cur_section = sv;
                    }
                }
                temp_stream += out.stream.as_str();
                let mut new_var = var::Var {
                    name: name.clone(),
                    type_hint: type_hint.clone(),
                    is_ref: out.is_ref
                        || if let Type::PTR(_v) = type_hint.clone() {
                            true
                        } else {
                            false
                        },
                    references: None,
                    definition: expr.clone().span,
                };
                if type_hint == Type::Any {
                    new_var.type_hint = *out.type_hint.clone();
                }
                // ToDo: Implement Functionality => VTable
                if let Some(ref_name) = out.refed_var {
                    new_var.references = Some(Box::new(
                        self.lookup_variable(&ref_name.as_str()).unwrap().clone(),
                    ));
                    self.lookup_variable(&ref_name.as_str()).unwrap().is_ref = true;
                }
                if self.cur_body_type != BodyType::GENERIC {
                    stream.push_str(&format!(
                        "{} {} = {}",
                        new_var.clone().type_hint.to_str(),
                        name,
                        out.stream
                    ));
                } else {
                    stream.push_str(&format!(
                        "{} {} = {}",
                        new_var.clone().type_hint.genimpl(),
                        name,
                        out.stream
                    ));
                }
                self.current_context.add(new_var);
                self.cur_offset += 1;
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
                if let Node::ReVal { name, value } = init.clone().node {
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
            stream,
            is_ref: v_is_ref,
            refed_var: None,
            type_hint: Box::new(Type::NoType),
            var: None,
        }
    }
    fn gen_func_call(&mut self, expression: Box<Spanned<Node>>, target_type: Type) -> FResult {
        let is_generic = self.cur_body_type == BodyType::GENERIC;
        let mut fix = "\\";
        if !is_generic {
            fix = "";
        }
        let mut stream = String::new();
        let Node::FcCall { params, callee } = expression.node.clone() else {
            return FResult {
                stream,
                type_hint: Type::NoType,
            };
        };
        let mut fcname = String::new();
        let mut fargs: Vec<FunctionArg> = Vec::new();
        let mut fret_type = Type::NoType;
        let mut variadic = false;
        match callee.clone().node {
            Node::MemberAccess { base, field } => {
                let mut is_f = false;
                if let Node::FcCall { params, callee } = base.clone().node {
                    is_f = true;
                }
                let out = self.gen_expr(base.clone(), target_type.clone(), RefStyle::COPY);
                let mut layout = self.get_layout(*out.clone().type_hint).clone();
                let mut base_type = Type::NoType;
                if layout.is_none() {
                    if let Type::Generic { base, generics } = *out.type_hint.clone() {
                        layout = self.get_layout(*base.clone());
                        base_type = *base;
                    }
                } else {
                    base_type = self
                        .lookup_variable(&out.var.clone().unwrap())
                        .unwrap()
                        .type_hint
                        .clone();
                }
                let mut method = self.get_field_item(layout.unwrap(), &field, callee.clone().span);
                let mut ig = false;
                let mut gnrcs = vec![];

                if let Type::Generic { base, generics } = base_type.clone() {
                    method = self.create_concrete_from_generic(method.clone(), generics, *base);
                } else if let Some(Generic { name, generics }) = self
                    .generics
                    .clone()
                    .iter()
                    .find(|p| p.name == *out.type_hint)
                {
                    base_type = Type::Generic {
                        base: out.type_hint.clone(),
                        generics: generics.clone(),
                    };
                    ig = true;
                    gnrcs = generics.clone();
                }
                println!("\n\n{method:#?}\n{base_type:#?}");

                fargs = method.clone().args;
                let mut modifier = "";
                let mut gmod = "*";
                let mut gvalmod = "&";
                if out.clone().is_ref {
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
                    modifier = "&";
                }
                let g = self.gb();
                if !is_generic {
                    stream += &format!(
                        "({{{fix}\n\t{}{gmod} __{} = {gvalmod}{};{fix}\n",
                        base_type.to_str(),
                        g.clone(),
                        out.stream
                    );
                    stream += &format!("{}_{}({modifier}__{g}", base_type.to_str(), field);
                } else {
                    stream += &format!(
                        "({{{fix}\n\t{}{gmod} __{} = {gvalmod}{};{fix}\n",
                        base_type.genimpl(),
                        g.clone(),
                        out.stream
                    );
                    stream += &format!("{}##_{}({modifier}__{g}", base_type.genimpl(), field);
                }
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
                stream += &format!(");{fix}\n}})");
                return FResult {
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
                // println!("here ({fcname}, {:?})", func.unwrap().ty.clone());
                fargs = func.unwrap().args.clone();
                fret_type = func.unwrap().ty.clone();
                variadic = func.unwrap().variadic;
                stream += format!("{fcname}(").as_str();
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
        }
        stream += &format!(")");
        let ty = fret_type.clone();
        return FResult {
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

    fn verify_type(&mut self, type_hint: Type) -> Option<Type> {
        if let Type::PTR(_) = type_hint {
            return Some(type_hint);
        }
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
            let g = self.verify_type(*ty);
            self.bundles = sb.clone();
            self.types = st.clone();
            self.current_context = sc.clone();
            self.var_table = sv.clone();
            return g;
        }
        self.types.verify(type_hint)
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
        }
        self.types.getLayout(type_hint)
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
            (Type::Custom(c1), Type::Custom(c2)) => c1 == c2,
            (Type::INT, other) => is_int(other),
            (other, Type::INT) => is_int(other),
            (Type::STR, Type::STR) => true,
            (Type::STR, _other) => false,
            (_other, Type::STR) => false,
            (Type::list(t, s), Type::list(t2, s2)) => self.type_match(*t, *t2) && s == s2,
            (Type::CHAR, Type::CHAR) => true,
            (Type::NoType, Type::NoType) => true,
            (Type::PTR(v), Type::PTR(c)) => self.type_match(*v, *c),
            (
                Type::Generic {
                    base: b1,
                    generics: g1,
                },
                Type::Generic {
                    base: b2,
                    generics: g2,
                },
            ) => self.type_match(*b1, *b2) && g1 == g2,
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
            (int1, int2) => is_int(int1.clone()) && is_int(int2.clone()) || int1 == int2,
        }
    }

    fn is_iterable(&mut self, clone: Type) -> bool {
        match clone.clone() {
            Type::PTR(_v) => true,
            Type::list(_, _) => true,
            Type::STR => true,
            _ => false,
        }
    }

    fn gfromstr(&self, sub: Vec<String>) -> Vec<Type> {
        let mut g = vec![];
        for t in sub {
            g.push(Type::Custom(t));
        }
        g
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

    fn create_concrete_from_generic(
        &mut self,
        method: Function,
        generics: Vec<Type>,
        ty: Type,
    ) -> Function {
        let mut idx = 0;
        let mut ret = Function::new(
            method.name.clone(),
            method.context.clone(),
            method.ty.clone(),
            method.returns,
            method.body.clone(),
        );
        let mut new_context =
            Context::new(method.context.name.clone(), method.context.parent.clone());
        let generic_entry = self.generics.iter().find(|p| p.name == ty).unwrap();
        for mut v in method.context.content.content.clone() {
            if generic_entry.generics.contains(&v.type_hint) {
                v.type_hint = generics.get(idx).unwrap().clone();
                new_context.add(v);
                idx += 1;
            }
        }
        let mut new_ty = method.clone().ty;
        if let Type::Generic { base, generics: _g } = method.ty.clone() {
            new_ty = Type::Generic {
                base,
                generics: generics.clone(),
            };
        } else if generic_entry.generics.contains(&method.ty) {
            let mut idx = 0;
            for t in generic_entry.generics.clone() {
                if t == method.ty {
                    new_ty = generics.get(idx).unwrap().clone();
                    idx += 1;
                }
            }
        }
        idx = 0;
        let mut new_args = vec![];
        for mut a in method.clone().args {
            if generic_entry.generics.contains(&a.type_hint) {
                a.type_hint = self.type_replace(a.type_hint, generics.get(idx).unwrap());
                idx += 1;
            }
            new_args.push(a);
        }
        ret.context = new_context;
        ret.args = new_args;
        ret.ty = new_ty;
        ret
    }

    fn generate_generic_block_instance(&mut self, ty: Type) -> String {
        let mut stream = String::new();
        if let Type::Generic { base, generics } = ty.clone() {
            stream += &format!("GENERIC_BLOCK_{}(", base.to_str().to_uppercase());
            for (i, g) in generics.iter().enumerate() {
                stream += &g.to_str();
                if i != generics.len() - 1 {
                    stream += ",";
                }
            }
            stream += ")";
        }
        stream
    }

    fn type_replace(&self, ty: Type, t: &Type) -> Type {
        t.clone()
    }
}

fn is_builtin(clone: Type) -> bool {
    match clone {
        Type::Custom(_) => false,
        Type::BundledType { bundle: _, ty: _ } => false,
        Type::Generic {
            base: _,
            generics: _,
        } => false,
        _ => true,
    }
}

fn is_int(target_type: Type) -> bool {
    use crate::backend::ttype::Type::*;
    matches!(
        target_type,
        INT | U8 | U64 | U32 | U16 | I8 | I16 | I32 | I64 | CHAR
    )
}
