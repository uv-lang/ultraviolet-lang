use std::{cell::RefCell, rc::Rc};

use crate::types::{EnvRef, frontend::Spanned};

pub mod backend;
pub mod ffi;
pub mod frontend;

pub trait EnvironmentTrait<T> {
    /// Find symbol by name
    fn find_var(&self, name: &[Spanned<String>]) -> Option<Rc<RefCell<T>>>;

    /// Define variable in current scope
    fn define_variable(&mut self, name: impl Into<String>, value: T) -> Rc<RefCell<T>>;

    /// Define variable from rc in current scope
    fn define_variable_rc(&mut self, name: impl Into<String>, value: Rc<RefCell<T>>);

    /// Remove symbol from CURRENT scope
    fn remove_symbol(&mut self, name: impl Into<String>) -> bool;
}

pub trait GetVariableContainedEnvironment {
    type Out: GetVariableContainedEnvironment;

    /// Get the contained environment from a variable
    fn get_variable_contained_env(&self) -> Option<EnvRef<Self::Out>>;
}
