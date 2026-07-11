use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    ops::Deref,
    rc::Rc,
};

use crate::{
    errors::SpannedError,
    traits::{
        EnvironmentTrait, GetVariableContainedEnvironment,
        frontend::{Positional, UVDisplay},
    },
    types::{
        backend::{RTVariable, UVRTValue},
        frontend::{Spanned, ast::SymbolName},
    },
};

pub mod backend;
pub mod builtins;
pub mod ffi;
pub mod frontend;

pub type EnvRef<T> = Rc<RefCell<Environment<T>>>;

#[derive(Default, Debug)]
pub struct SymbolsUseInterceptor {
    pub intercepted_names: RefCell<HashSet<SymbolName>>,
}

#[derive(Debug)]
pub struct Environment<T> {
    pub symbols: HashMap<String, Rc<RefCell<T>>>,
    pub parent: Option<EnvRef<T>>,

    /// Used for intercept inner names, that been accessed
    pub interceptor: Option<Rc<SymbolsUseInterceptor>>,
}

impl<T> Environment<T> {
    /// Create new empty env
    pub fn new() -> EnvRef<T> {
        Rc::new(RefCell::new(Self {
            symbols: HashMap::new(),
            parent: None,

            interceptor: None,
        }))
    }

    /// Create new empty env
    pub fn new_from(sym: HashMap<String, Rc<RefCell<T>>>) -> EnvRef<T> {
        Rc::new(RefCell::new(Self {
            symbols: sym,
            parent: None,

            interceptor: None,
        }))
    }

    /// Create new children environment from parent
    pub fn new_child(parent: EnvRef<T>) -> EnvRef<T> {
        Rc::new(RefCell::new(Self {
            symbols: HashMap::new(),
            parent: Some(parent.clone()),

            interceptor: parent.borrow().interceptor.clone(),
        }))
    }

    /// Enable interception of accessed symbols
    pub fn enable_interception(&mut self) {
        self.interceptor = Some(Rc::new(SymbolsUseInterceptor::default()))
    }

    /// Intercept symbol usage
    pub fn intercept(&self, name: &[Spanned<String>]) {
        if let Some(i) = &self.interceptor {
            i.intercepted_names.borrow_mut().insert(name.to_vec());
        }
    }
}

impl<T> EnvironmentTrait<T> for Environment<T>
where
    T: GetVariableContainedEnvironment<Out = T>,
{
    fn find_var(&self, name: &[Spanned<String>]) -> Result<Rc<RefCell<T>>, SpannedError> {
        let (first, rest) = name.split_first().ok_or(SpannedError::new(
            format!("Invalid name: `{}`", name.to_vec().join(".")),
            name.to_vec().get_span(),
        ))?;

        let found = if let Some(sym) = self.symbols.get(&first.value) {
            self.intercept(name);
            sym.clone()
        } else {
            return self
                .parent
                .as_ref()
                .ok_or(SpannedError::new(
                    format!("Name `{}` not defined", first),
                    first.get_span(),
                ))?
                .borrow()
                .find_var(name);
        };

        if rest.is_empty() {
            Ok(found)
        } else {
            found
                .borrow()
                .get_variable_contained_env()
                .ok_or(SpannedError::new(
                    format!("Name `{}` not defined", first),
                    first.get_span(),
                ))?
                .borrow()
                .find_var(rest)
        }
    }

    fn define_variable(&mut self, name: impl Into<String>, value: T) -> Rc<RefCell<T>> {
        let rc = Rc::new(RefCell::new(value));
        self.symbols.insert(name.into(), rc.clone());
        rc
    }

    fn remove_symbol(&mut self, name: impl Into<String>) -> bool {
        self.symbols.remove(&name.into()).is_some()
    }

    fn define_variable_rc(&mut self, name: impl Into<String>, value: Rc<RefCell<T>>) {
        self.symbols.insert(name.into(), value);
    }
}

impl Environment<RTVariable> {
    /// Artificially creates a tree of names in the current environment to simulate captured names
    pub fn define_intercepted_name(
        &mut self,
        name: SymbolName,
        val: Rc<RefCell<RTVariable>>,
    ) -> Result<(), SpannedError> {
        for (i, part) in name.iter().enumerate() {
            if i + 1 == name.len() {
                self.define_variable_rc(part.deref(), val);
                break;
            }

            if let Some(child_env) = self.symbols.get(&part.value) {
                let env = child_env.borrow_mut().get_variable_contained_env().ok_or(
                    SpannedError::new(
                        format!("Name conflict: {} already defined", part.value),
                        part.get_span(),
                    ),
                )?;
                env.borrow_mut()
                    .define_intercepted_name(name[i..].to_vec(), val.clone())?;

                continue;
            }

            let rc = self.define_variable(
                part.value.clone(),
                RTVariable::new_from(UVRTValue::Module(Environment::new()), true),
            );
            rc.borrow_mut()
                .get_variable_contained_env()
                .unwrap()
                .borrow_mut()
                .define_intercepted_name(name[i..].to_vec(), val.clone())?;
        }

        Ok(())
    }
}
