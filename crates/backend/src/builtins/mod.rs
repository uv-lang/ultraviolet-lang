use crate::builtins::{constants::init_builtin_constants, functions::init_builtin_functions};
use std::{cell::RefCell, rc::Rc};
use ultraviolet_core::types::{Environment, backend::RTVariable};

pub mod constants;
pub mod functions;

/// Trait, that defines behavior for defining runtime builtins
pub trait DefineBuiltinsRT {
    fn define_builtins(&self);
}

impl DefineBuiltinsRT for Rc<RefCell<Environment<RTVariable>>> {
    fn define_builtins(&self) {
        init_builtin_constants(self.clone());
        init_builtin_functions(self.clone());
    }
}
