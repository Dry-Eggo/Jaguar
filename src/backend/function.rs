use super::{
    context::Context,
    parser::{FunctionArg, Node},
    ttype::Type,
    var_table::VTable,
};
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub(crate) context: Context,
    pub ty: Type, /* ToDo : Implement getters and setters */
    pub(crate) returns: bool,
    pub(crate) body: Node,
    pub variadic: bool,
    pub args: Vec<FunctionArg>,
    pub gen_name: String,
}

impl Function {
    pub fn new(name: String, context: Context, ty: Type, returns: bool, body: Node) -> Self {
        Self {
            name,
            context,
            ty,
            returns,
            body,
            args: Vec::new(),
            variadic: false,
            gen_name: "d".to_owned(),
        }
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// Returns the get context content of this [`Function`].
    pub(crate) fn get_context_content(&self) -> VTable {
        self.context.get_content()
    }
}
