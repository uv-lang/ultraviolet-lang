use crate::types::frontend::ast::UVType;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
pub type EnvRef = Rc<RefCell<Environment>>;

/// Scope-based environment
#[derive(Default)]
pub struct Environment {
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<EnvRef>,
}

impl Environment {
    /// Create new children environment from parent
    pub fn new_child(parent: EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Self {
            symbols: HashMap::new(),
            parent: Some(parent),
        }))
    }
}

pub enum ControlFlow {
    Return(UVType),
    Simple(UVType),
}

pub enum Symbol {
    Variable(UVType),
}
