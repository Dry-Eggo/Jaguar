use super::{
    codegen::is_builtin, function::Function, ttype::Type, type_table::TTable, var_table::VTable,
};
#[derive(Debug, Clone)]
pub struct Bundle {
    pub name: String,
    pub vars: VTable,
    pub functions: Vec<Function>,
    pub types: TTable,
    pub bundles: Vec<Bundle>,
    pub path: String,
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
        }
    }
    pub fn refuse_dup(&mut self, subject: String) -> Option<Bundle> {
        if self.path == subject {
            return Some(self.clone());
        }
        let o = self.bundles.iter().find(|b| b.path == subject).cloned();
        o
    }
    pub fn wrap(&mut self, alias: &str) {
        for f in &mut self.functions {
            for arg in &mut f.args {
                if !is_builtin(arg.type_hint.clone()) {
                    arg.type_hint = Type::BundledType {
                        bundle: alias.clone().to_owned(),
                        ty: Box::new(arg.type_hint.clone()),
                    }
                }
            }
            if !is_builtin(f.ty.clone()) {
                f.ty = Type::BundledType {
                    bundle: alias.clone().to_owned(),
                    ty: Box::new(f.ty.clone()),
                };
            }
        }
        for b in &mut self.bundles {
            b.wrap(alias);
        }
    }
}
