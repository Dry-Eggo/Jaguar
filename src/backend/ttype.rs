use std::{ops::Deref, str::FromStr};

use serde_json::ser::PrettyFormatter;

#[derive(Debug, Clone, PartialEq, Hash, std::cmp::Eq)]
pub enum Type {
    Any,
    Generic {
        base: Box<Type>,
        generics: Vec<Type>,
    },
    CHAR,
    Custom(String),
    GenericAtom(String),
    list(Box<Type>, String), /* list<T, N> T: Type N: size */
    I16,
    I32,
    I64,
    I8,
    INT,
    NoType,
    STR,
    U16,
    U32,
    U64,
    U8,
    BundledType {
        bundle: String,
        ty: Box<Type>,
    },
    PTR(Box<Type>),
}

impl Type {
    pub fn to_str(&self) -> String {
        match self {
            Type::Custom(value) => {
                return value.clone();
            }
            Type::list(t, n) => {
                return format!("jaguar_list_{}", t.to_str());
            }
            Type::STR => {
                return "jaguar_str".into();
            }
            Type::INT => {
                return "jaguar_i32".into();
            }
            Type::CHAR => {
                return "char".into();
            }
            Type::U8 => {
                return "jaguar_u8".into();
            }
            Type::U16 => {
                return "jaguar_u16".into();
            }
            Type::U32 => {
                return "jaguar_u32".into();
            }
            Type::U64 => {
                return "jaguar_u64".into();
            }
            Type::I8 => {
                return "jaguar_i8".into();
            }
            Type::I16 => {
                return "jaguar_i16".into();
            }
            Type::I32 => {
                return "jaguar_i32".into();
            }
            Type::I64 => {
                return "jaguar_i64".into();
            }
            Type::BundledType { bundle, ty } => {
                return format!("{}", ty.to_str());
            }
            Type::PTR(v) => {
                return format!("{}*", v.to_str());
            }
            Type::Generic { base, generics } => {
                // println!("Here{generics:?}");
                let mut g = base.to_str();
                for gn in generics {
                    // println!("Debug: Adding {gn:?}");
                    g += &format!("_{}", gn.to_str());
                }
                // println!("Debug: Final Generic name: {g}");
                return g;
            }
            Type::GenericAtom(v) => {
                return format!("_##{v}");
            }
            Type::NoType => return "void".into(),
            _ => {
                return format!("{:?}", self.clone());
            }
        }
    }
    pub fn debug(&self) -> String {
        match self {
            Type::Custom(value) => {
                return value.clone();
            }
            Type::list(t, n) => {
                return format!("list<{}, {n}>", t.debug());
            }
            Type::BundledType { bundle, ty } => {
                return format!("{bundle}::{}", ty.debug());
            }
            Type::CHAR => return "char".into(),
            Type::STR => return "imut string".into(),
            Type::INT => return "int".into(),
            Type::PTR(ty) => {
                return format!("*{}", ty.debug());
            }
            Self::Generic { base, generics } => {
                let mut g = format!("{}<", base.debug());
                for (i, t) in generics.iter().enumerate() {
                    g += &format!("{}", t.debug());
                    if i != generics.len() - 1 {
                        g += ",";
                    }
                }
                g += ">";
                g
            }
            Type::NoType => return "void".into(),
            _ => return format!("{:?}", self.clone()),
        }
    }
    pub fn genimpl(&self) -> String {
        match self {
            Type::Generic { base, generics } => {
                let mut g = base.to_str();
                for gn in generics {
                    g += &format!("_##{}", gn.to_str());
                }
                g
            }
            _ => self.to_str(),
        }
    }
}
