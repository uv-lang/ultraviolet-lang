use crate::types::frontend::ast::UVType;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
pub type EnvRef = Rc<RefCell<Environment>>;

/// Scope-based environment
#[derive(Default)]
pub struct Environment {
    pub symbols: HashMap<String, Rc<RefCell<Variable>>>,
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

    /// Define variable type in current scope
    pub fn define_variable(&mut self, name: impl Into<String>, t: UVType, constant: bool) {
        self.symbols.insert(
            name.into(),
            Rc::new(RefCell::new(Variable::new_from(t, constant))),
        );
    }
}

pub enum ControlFlow {
    Return(UVType),
    Simple(UVType),
}

/// Typecheck variable struct
#[derive(Debug, Clone)]
pub struct Variable {
    pub value: UVType,
    pub constant: bool,
}

impl Variable {
    /// Create new variable from value
    pub fn new_from(val: UVType, constant: bool) -> Self {
        Self {
            value: val,
            constant,
        }
    }
}
