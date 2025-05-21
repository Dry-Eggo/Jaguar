#[derive(Debug, Clone, PartialEq, Hash, std::cmp::Eq)]
pub enum Type {
    Any,
    CHAR,
    Custom(String),
    List(Box<Type>, String), /* list<T, N> T: Type N: size */
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
    BundledType { bundle: String, ty: Box<Type> },
    PTR(Box<Type>),
    MUT(Box<Type>),
}

impl Type {
    pub fn to_str(&self) -> String {
        match self {
            Type::Custom(value) => {
                return format!("const {}", value.clone());
            }
            Type::List(t, n) => {
                return format!("const jaguar_list_{}", t.to_str());
            }
            Type::STR => {
                return "const jaguar_str".into();
            }
            Type::INT => {
                return "const jaguar_i32".into();
            }
            Type::CHAR => {
                return "const char".into();
            }
            Type::U8 => {
                return "const jaguar_u8".into();
            }
            Type::U16 => {
                return "const jaguar_u16".into();
            }
            Type::U32 => {
                return "const jaguar_u32".into();
            }
            Type::U64 => {
                return "const jaguar_u64".into();
            }
            Type::I8 => {
                return "const jaguar_i8".into();
            }
            Type::I16 => {
                return "const jaguar_i16".into();
            }
            Type::I32 => {
                return "const jaguar_i32".into();
            }
            Type::I64 => {
                return "const jaguar_i64".into();
            }
            Type::BundledType { bundle, ty } => {
                return format!("const {}", ty.to_str());
            }
            Type::PTR(v) => {
                return format!("{}* const", v.to_str());
            }
            Type::MUT(v) => {
                return v.c_impl();
            }
            Type::NoType => return "void".into(),
            _ => {
                return format!("{:?}", self.clone());
            }
        }
    }
    pub fn c_impl(&self) -> String {
        match self {
            Type::Custom(value) => {
                return format!("{}", value.clone());
            }
            Type::List(t, n) => {
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
            Type::MUT(v) => {
                return v.c_impl();
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
            Type::List(t, n) => {
                return format!("list<{}, {n}>", t.debug());
            }
            Type::BundledType { bundle, ty } => {
                return format!("{bundle}::{}", ty.debug());
            }
            Type::CHAR => return "char".into(),
            Type::STR => return "str".into(),
            Type::INT => return "int".into(),
            Type::PTR(ty) => {
                return format!("*{}", ty.debug());
            }
            Type::NoType => return "void".into(),
            _ => return format!("{:?}", self.clone()),
        }
    }

    pub(crate) fn is_mutable(&self) -> bool {
        if let Type::MUT(t) = self {
            return true;
        }
        false
    }
    pub(crate) fn is_pointer(&self) -> bool {
        if let Type::PTR(_) = self {
            return true;
        } else if let Type::MUT(t) = self {
            return t.is_pointer();
        }
        false
    }
}
