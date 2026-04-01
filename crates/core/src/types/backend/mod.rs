use crate::{
    traits::frontend::ast::GetType,
    types::frontend::ast::{UVType, UVValue},
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub type EnvRef = Rc<RefCell<Environment>>;

/// Scope-based environment
#[derive(Default, Debug)]
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

    /// Find symbol by name
    pub fn find(&self, name: impl Into<String>) -> Option<Symbol> {
        let n = name.into();
        if let Some(sym) = self.symbols.get(&n) {
            return Some(sym.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().find(&n);
        }

        None
    }

    /// Define variable in current scope
    pub fn define_variable(&mut self, name: String, value: UVValue, constant: bool) {
        self.symbols.insert(
            name,
            Symbol::Variable(Rc::new(RefCell::new(RTVariable::new_from(value, constant)))),
        );
    }
}

/*
impl Drop for Environment {
    fn drop(&mut self) {
        println!("Environment dropped");
    }
}
*/

#[derive(Debug, Clone)]
pub enum Symbol {
    Variable(Rc<RefCell<RTVariable>>),
    Function(),
}

impl GetType for Symbol {
    fn get_type(&self) -> UVType {
        match self {
            Symbol::Variable(rc) => rc.borrow().value.get_type(),
            Symbol::Function() => todo!(),
        }
    }
}

/// Runtime variable struct'
#[derive(Debug, Clone)]
pub struct RTVariable {
    pub value: UVValue,
    pub constant: bool,
}

impl RTVariable {
    /// Create new variable from value
    pub fn new_from(val: UVValue, constant: bool) -> Self {
        Self { value: val, constant }
    }
}

/// Indicates, when block ended with return, break, etc...
#[derive(Debug)]
pub enum ControlFlow {
    Simple(UVValue),

    /// Return propagates upstream
    Return(UVValue),
}

impl ControlFlow {
    // FIXME: Should interpreter flat a ControlFlow?
    pub fn flatten(&self) -> &UVValue {
        match self {
            ControlFlow::Simple(uvvalue) => uvvalue,
            ControlFlow::Return(uvvalue) => uvvalue,
        }
    }
}
