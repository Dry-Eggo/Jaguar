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

#[derive(Debug, Clone)]
pub struct Generic_Function {
    pub name: String,
    pub(crate) context: Context,
    pub ty: Type, /* ToDo : Implement getters and setters */
    pub(crate) returns: bool,
    pub(crate) body: Node,
    pub variadic: bool,
    pub args: Vec<FunctionArg>,
    pub gen_name: String,
    pub generics: Vec<Type>,
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

    pub(crate) fn to_generic(&self, generics: Vec<Type>) -> Generic_Function {
        return Generic_Function {
            name: self.name.clone(),
            context: self.context.clone(),
            ty: self.ty.clone(),
            returns: self.returns.clone(),
            body: self.body.clone(),
            variadic: self.variadic.clone(),
            args: self.args.clone(),
            gen_name: self.gen_name.clone(),
            generics: generics.clone(),
        };
    }
}
impl Generic_Function {
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
            generics: vec![],
        }
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// Returns the get context content of this [`Function`].
    pub(crate) fn get_context_content(&self) -> VTable {
        self.context.get_content()
    }

    pub(crate) fn to_function(mut self) -> Function {
        Function::new(self.name, self.context, self.ty, self.returns, self.body)
    }
}
