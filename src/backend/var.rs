use crate::lexer::Span;

use super::ttype::Type;

#[derive(Debug, Clone)]
pub struct Var {
    pub name: String,
    pub type_hint: Type,
    pub is_ref: bool,
    pub references: Option<Box<Var>>,
    pub definition: Span,
}
impl Var {
    pub fn new(name: String, ty: Type, is_ref: bool, refs: Option<Box<Var>>, span: Span) -> Self {
        Self {
            name,
            type_hint: ty,
            is_ref,
            references: refs,
            definition: span,
        }
    }
}
