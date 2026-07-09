use std::{cell::RefCell, rc::Rc};

pub mod backend;
pub mod ffi;
pub mod frontend;

pub trait EnvironmentTrait<T> {
    /// Find symbol by name
    fn find_var(&self, name: impl Into<String>) -> Option<Rc<RefCell<T>>>;

    /// Define variable in current scope
    fn define_variable(&mut self, name: impl Into<String>, value: T) -> Rc<RefCell<T>>;

    /// Remove symbol from CURRENT scope
    fn remove_symbol(&mut self, name: impl Into<String>) -> bool;
}
