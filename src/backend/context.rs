use super::{var::Var, var_table::VTable};

#[derive(Debug, Clone)]
pub struct Context {
    pub name: String,
    pub(crate) content: VTable,
    pub(crate) parent: Option<Box<Context>>,
}

impl Context {
    pub fn new(name: String, parent: Option<Box<Context>>) -> Self {
        Self {
            name,
            content: VTable::new(),
            parent,
        }
    }

    pub(crate) fn get_parent(&mut self) -> Option<Context> {
        if matches!(self.parent.clone(), None) {
            Some(*self.parent.clone().unwrap());
        }
        None
    }

    pub(crate) fn add(&mut self, v: Var) {
        self.content.add(v);
    }

    pub(crate) fn get_content(&self) -> VTable {
        self.content.clone()
    }

    pub fn look_up_var(&mut self, name: &str) -> Option<&mut Var> {
        if let Some(var) = self.content.lookup(name) {
            return Some(var);
        }

        if let Some(parent) = &mut self.parent {
            return parent.look_up_var(name);
        }
        None
    }

    pub(crate) fn set_ref(&mut self, val: Var) {
        self.content.set_ref(val);
    }
}
