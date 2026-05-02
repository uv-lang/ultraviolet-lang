pub mod constants_types;
pub mod functions_types;

use crate::types::{
    Environment,
    builtins::{
        constants_types::init_builtin_types_constants,
        functions_types::init_builtin_types_functions,
    },
    frontend::typechecker::UVTypeVariable,
};
use std::{cell::RefCell, rc::Rc};

/// Trait, that defines behavior for defining builtins types
pub trait DefineBuiltinsType {
    fn define_builtins(&self);
}

impl DefineBuiltinsType for Rc<RefCell<Environment<UVTypeVariable>>> {
    fn define_builtins(&self) {
        init_builtin_types_constants(self.clone());
        init_builtin_types_functions(self.clone());
    }
}
