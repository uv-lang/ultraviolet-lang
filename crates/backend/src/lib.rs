use ultraviolet_core::{
    errors::SpannedError,
    types::{Environment, backend::ControlFlow, frontend::SourceFileParsed},
};

use crate::builtins::DefineBuiltinsRT;

mod builtins;
mod eval;

/** Evaluate code */
pub fn eval(source: &SourceFileParsed) -> Result<ControlFlow, SpannedError> {
    let env = Environment::new();
    env.define_builtins();

    eval::eval(&source.ast, env)
}
