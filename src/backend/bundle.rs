use super::{
    function::{Function, Generic_Function},
    type_table::TTable,
    var_table::VTable,
};
#[derive(Debug, Clone)]
pub struct Bundle {
    pub name: String,
    pub vars: VTable,
    pub functions: Vec<Function>,
    pub types: TTable,
    pub bundles: Vec<Bundle>,
    pub path: String,
    pub gfunctions: Vec<Generic_Function>,
}

impl Bundle {
    pub fn new(
        name: String,
        vars: VTable,
        funcs: Vec<Function>,
        types: TTable,
        bundles: Vec<Bundle>,
        p: String,
    ) -> Self {
        Self {
            name,
            vars,
            functions: funcs,
            types,
            bundles,
            path: p,
            gfunctions: vec![],
        }
    }
    pub fn refuse_dup(&mut self, subject: String) -> Option<Bundle> {
        if self.path == subject {
            return Some(self.clone());
        }
        let o = self.bundles.iter().find(|b| b.path == subject).cloned();
        o
    }
}
