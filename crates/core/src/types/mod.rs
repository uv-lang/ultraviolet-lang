use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub mod backend;
pub mod builtins;
pub mod ffi;
pub mod frontend;

pub type EnvRef<T> = Rc<RefCell<Environment<T>>>;

#[derive(Default, Debug)]
pub struct SymbolsUseInterceptor {
    pub intercepted_names: RefCell<HashSet<String>>,
}

pub struct Environment<T> {
    pub symbols: HashMap<String, Rc<RefCell<T>>>,
    pub parent: Option<EnvRef<T>>,
    pub neighbor_envs: HashMap<String, EnvRef<T>>,

    /// Used for intercept inner names, that been accessed
    pub interceptor: Option<Rc<SymbolsUseInterceptor>>,
}

impl<T> Environment<T> {
    /// Create new empty env
    pub fn new() -> EnvRef<T> {
        Rc::new(RefCell::new(Self {
            symbols: HashMap::new(),
            parent: None,
            neighbor_envs: HashMap::new(),

            interceptor: None,
        }))
    }

    /// Create new empty env
    pub fn new_from(sym: HashMap<String, Rc<RefCell<T>>>) -> EnvRef<T> {
        Rc::new(RefCell::new(Self {
            symbols: sym,
            parent: None,
            neighbor_envs: HashMap::new(),

            interceptor: None,
        }))
    }

    /// Create new children environment from parent
    pub fn new_child(parent: EnvRef<T>) -> EnvRef<T> {
        Rc::new(RefCell::new(Self {
            symbols: HashMap::new(),
            parent: Some(parent.clone()),
            neighbor_envs: HashMap::new(),

            interceptor: parent.borrow().interceptor.clone(),
        }))
    }

    /// Find symbol by name
    pub fn find_var(&self, name: impl Into<String>) -> Option<Rc<RefCell<T>>> {
        let n = name.into();
        if let Some(sym) = self.symbols.get(&n) {
            self.intercept(n);
            return Some(sym.clone());
        }

        if let Some(parent) = &self.parent {
            self.intercept(n.clone());
            return parent.borrow().find_var(&n);
        }

        None
    }

    /// Define variable in current scope
    pub fn define_variable(&mut self, name: impl Into<String>, value: T) -> Rc<RefCell<T>> {
        let rc = Rc::new(RefCell::new(value));
        self.symbols.insert(name.into(), rc.clone());
        rc
    }

    /// Remove symbol from CURRENT scope
    pub fn remove_symbol(&mut self, name: impl Into<String>) -> bool {
        self.symbols.remove(&name.into()).is_some()
    }

    /// Enable interception of accessed symbols
    pub fn enable_interception(&mut self) {
        self.interceptor = Some(Rc::new(SymbolsUseInterceptor::default()))
    }

    /// Intercept symbol usage
    pub fn intercept(&self, name: String) {
        if let Some(i) = &self.interceptor {
            i.intercepted_names.borrow_mut().insert(name);
        }
    }
}
