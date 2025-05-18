use super::var::Var;

#[derive(Debug, Clone)]
pub struct VTable {
    pub content: Vec<Var>,
}

impl VTable {
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
        }
    }

    pub fn add(&mut self, v: Var) {
        self.content.push(v);
    }

    pub fn lookup(&mut self, _name: &str) -> Option<&mut Var> {
        self.content.iter_mut().find(|var| var.name == _name)
    }

    pub(crate) fn set_ref(&mut self, val: Var) {
        let var = self.content.iter_mut().find(|var| var.name == val.name);
        var.unwrap().is_ref = true;
    }
}
