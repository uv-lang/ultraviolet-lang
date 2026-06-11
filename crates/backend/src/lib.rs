use ultraviolet_core::{
    errors::SpannedError,
    types::{Environment, backend::ControlFlow, frontend::ast::ASTBlockType},
};

use crate::builtins::DefineBuiltinsRT;

mod builtins;
mod eval;

/** Evaluate code */
pub fn eval(node: &ASTBlockType) -> Result<ControlFlow, SpannedError> {
    let env = Environment::new();
    env.define_builtins();

    eval::eval(node, env)
}
