use std::{any::Any, collections::HashMap};

use super::{
    codegen::BlockLayout, context::Context, function::Function, parser::Node, ttype::Type,
};
#[derive(Debug, Clone)]
pub struct TTable {
    content: HashMap<Type, BlockLayout>,
}

impl TTable {
    pub fn new() -> Self {
        let mut content: HashMap<Type, BlockLayout> = HashMap::new();
        content.insert(
            Type::INT,
            BlockLayout {
                name: Type::INT,
                feilds: HashMap::new(),
                methods: vec![],
                file: "".to_owned(),
            },
        );
        content.insert(
            Type::STR,
            BlockLayout {
                name: Type::STR,
                feilds: HashMap::new(),
                methods: vec![],
                file: String::new(),
            },
        );
        content.insert(
            Type::CHAR,
            BlockLayout {
                name: Type::CHAR,
                feilds: HashMap::new(),
                methods: vec![
                    Function::new(
                        "to_upper".into(),
                        Context::new("to_upper".into(), None),
                        Type::CHAR,
                        true,
                        Node::Program(vec![]),
                    ),
                    Function::new(
                        "to_lower".into(),
                        Context::new("to_lower".into(), None),
                        Type::CHAR,
                        true,
                        Node::Program(vec![]),
                    ),
                ],
                file: String::new(),
            },
        );
        content.insert(
            Type::U8,
            BlockLayout {
                name: Type::U8,
                feilds: HashMap::new(),
                methods: vec![],
                file: String::new(),
            },
        );
        content.insert(
            Type::U16,
            BlockLayout {
                name: Type::U16,
                feilds: HashMap::new(),
                methods: vec![],
                file: String::new(),
            },
        );
        content.insert(
            Type::U32,
            BlockLayout {
                name: Type::U32,
                feilds: HashMap::new(),
                methods: vec![],
                file: String::new(),
            },
        );
        content.insert(
            Type::U64,
            BlockLayout {
                name: Type::U64,
                feilds: HashMap::new(),
                methods: vec![],
                file: String::new(),
            },
        );
        content.insert(
            Type::I8,
            BlockLayout {
                name: Type::U8,
                feilds: HashMap::new(),
                methods: vec![],
                file: String::new(),
            },
        );
        content.insert(
            Type::I16,
            BlockLayout {
                name: Type::U16,
                feilds: HashMap::new(),
                methods: vec![],
                file: String::new(),
            },
        );
        content.insert(
            Type::I32,
            BlockLayout {
                name: Type::U32,
                feilds: HashMap::new(),
                methods: vec![],
                file: String::new(),
            },
        );
        content.insert(
            Type::I64,
            BlockLayout {
                name: Type::U64,
                feilds: HashMap::new(),
                methods: vec![],
                file: String::new(),
            },
        );
        content.insert(
            Type::NoType,
            BlockLayout {
                name: Type::NoType,
                feilds: HashMap::new(),
                methods: vec![],
                file: String::new(),
            },
        );
        Self { content }
    }

    pub fn verify(&mut self, ty: Type) -> Option<Type> {
        self.content.keys().into_iter().find(|t| **t == ty).cloned()
    }
    pub fn getLayout(&mut self, ty: Type) -> Option<BlockLayout> {
        self.content.get(&ty).cloned()
    }
    pub fn add_type(&mut self, ty: Type, b: BlockLayout) -> () {
        self.content.insert(ty, b);
    }
    pub fn register_plugin(&mut self, ty: Type, plugin: Function) {
        let l = self.content.get_mut(&ty);
        l.unwrap().methods.push(plugin);
    }

    pub(crate) fn wrap(&mut self, clone: String) {
        for mut ty in &mut self.content {
            for meth in &mut ty.1.methods {
                for f in &mut meth.args {
                    match f.type_hint.clone() {
                        Type::Custom(_) => {
                            f.type_hint = Type::BundledType {
                                bundle: clone.clone(),
                                ty: Box::new(f.type_hint.clone()),
                            }
                        }
                        Type::BundledType { bundle, ty } => {
                            f.type_hint = Type::BundledType {
                                bundle: clone.clone(),
                                ty: Box::new(f.type_hint.clone()),
                            }
                        }
                        _ => (),
                    }
                }
                match meth.ty.clone() {
                    Type::Custom(_) => {
                        meth.ty = Type::BundledType {
                            bundle: clone.clone(),
                            ty: Box::new(meth.ty.clone()),
                        }
                    }
                    Type::BundledType { bundle, ty } => {
                        meth.ty = Type::BundledType {
                            bundle: clone.clone(),
                            ty: Box::new(meth.ty.clone()),
                        }
                    }
                    _ => {}
                }
            }
            ty.1.name = Type::BundledType {
                bundle: clone.clone(),
                ty: Box::new(ty.1.clone().name.clone()),
            };
            ty.0 = &Type::BundledType {
                bundle: clone.clone(),
                ty: Box::new(ty.0.clone()),
            };
        }
    }
}
