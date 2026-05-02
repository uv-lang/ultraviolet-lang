use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub mod backend;
pub mod builtins;
pub mod frontend;

pub type EnvRef<T> = Rc<RefCell<Environment<T>>>;

#[derive(Debug)]
pub struct Environment<T> {
    pub symbols: HashMap<String, Rc<RefCell<T>>>,
    pub parent: Option<EnvRef<T>>,
}

impl<T> Environment<T> {
    /// Create new empty env
    pub fn new() -> EnvRef<T> {
        Rc::new(RefCell::new(Self {
            symbols: HashMap::new(),
            parent: None,
        }))
    }

    /// Create new children environment from parent
    pub fn new_child(parent: EnvRef<T>) -> EnvRef<T> {
        Rc::new(RefCell::new(Self {
            symbols: HashMap::new(),
            parent: Some(parent),
        }))
    }

    /// Find symbol by name
    pub fn find_var(&self, name: impl Into<String>) -> Option<Rc<RefCell<T>>> {
        let n = name.into();
        if let Some(sym) = self.symbols.get(&n) {
            return Some(sym.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().find_var(&n);
        }

        None
    }

    /// Define variable in current scope
    pub fn define_variable(&mut self, name: impl Into<String>, value: T) {
        self.symbols
            .insert(name.into(), Rc::new(RefCell::new(value)));
    }

    /// Remove symbol from CURRENT scope
    pub fn remove_symbol(&mut self, name: impl Into<String>) -> bool {
        self.symbols.remove(&name.into()).is_some()
    }
}

/// Splits provided name by `.` delimiter
pub fn process_sym_name(name: String) -> Vec<String> {
    name.split('.').map(str::to_string).collect()
}

/// Resolve symbol from provided environment, and modules (unimplemented)
pub fn resolve_sym<T>(name: Vec<String>, env: EnvRef<T>) -> Option<Rc<RefCell<T>>> {
    match name.len() {
        1 => env.borrow().find_var(name.first().unwrap()),
        _ => todo!(),
    }
}
