use std::rc::Rc;

use crate::builtins::DefineBuiltinsRT;
use ultraviolet_core::{
    errors::SpannedError,
    types::{
        EnvRef, Environment,
        backend::{ControlFlow, RTVariable},
        frontend::SourceFileParsed,
    },
};

mod builtins;
mod eval;

pub struct Evaluator {
    pub source: Rc<SourceFileParsed>,
    pub exports: EnvRef<RTVariable>,
    pub current_name: String,
}

/** Evaluate code */
impl Evaluator {
    pub fn new(source: Rc<SourceFileParsed>, name: impl Into<String>) -> Self {
        Self {
            source,
            exports: Environment::new(),
            current_name: name.into(),
        }
    }

    pub fn eval(&self) -> Result<ControlFlow, SpannedError> {
        let env = Environment::new();
        env.define_builtins();

        self.eval_single(&self.source.ast, env)
    }
}
