use crate::{
    traits::frontend::ast::GetType,
    types::frontend::ast::{ASTBlockType, UVType, UVValue},
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

    /// Find variable by name
    pub fn find_var(&self, name: impl Into<String>) -> Option<Rc<RefCell<RTVariable>>> {
        if let Some(Symbol::Variable(var)) = self.find(name) {
            Some(var)
        } else {
            None
        }
    }

    /// Find function by name
    pub fn find_func(&self, name: impl Into<String>) -> Option<Rc<RefCell<RTFunction>>> {
        if let Some(Symbol::Function(var)) = self.find(name) {
            Some(var)
        } else {
            None
        }
    }

    /// Define variable in current scope
    pub fn define_variable(&mut self, name: impl Into<String>, value: UVValue, constant: bool) {
        self.symbols.insert(
            name.into(),
            Symbol::Variable(Rc::new(RefCell::new(RTVariable::new_from(value, constant)))),
        );
    }

    /// Define function in current scope
    pub fn define_function(&mut self, name: impl Into<String>, f: RTFunction) {
        self.symbols
            .insert(name.into(), Symbol::Function(Rc::new(RefCell::new(f))));
    }

    /// Remove symbol from CURRENT scope
    pub fn remove_symbol(&mut self, name: impl Into<String>) -> bool {
        self.symbols.remove(&name.into()).is_some()
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
    Function(Rc<RefCell<RTFunction>>),
}

impl GetType for Symbol {
    fn get_type(&self) -> UVType {
        match self {
            Symbol::Variable(rc) => rc.borrow().value.get_type(),
            Symbol::Function(_) => todo!(),
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
        Self {
            value: val,
            constant,
        }
    }
}

#[derive(Debug)]
pub struct RTFunction {
    pub args_names_order: Vec<String>,
    pub body: Rc<Vec<ASTBlockType>>,
    pub lexical_env: EnvRef,
}

/// Indicates, when block ended with return, break, etc...
#[derive(Debug)]
pub enum ControlFlow {
    Simple(UVValue),

    /// Return propagates upstream
    Return(UVValue),

    Break,
    Continue,
}

impl ControlFlow {
    // FIXME: Should interpreter flat a ControlFlow?
    pub fn flatten(&self) -> &UVValue {
        match self {
            ControlFlow::Simple(uvvalue) => uvvalue,
            ControlFlow::Return(uvvalue) => uvvalue,
            ControlFlow::Continue | ControlFlow::Break => &UVValue::Void,
        }
    }
}
