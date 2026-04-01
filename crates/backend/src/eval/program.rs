use crate::eval::eval;
use ultraviolet_core::{
    errors::SpannedError,
    types::{
        backend::{ControlFlow, EnvRef},
        frontend::ast::ProgramBlock,
    },
};

/// Evaluate program block
pub fn eval_program(program: &ProgramBlock, env: EnvRef) -> Result<ControlFlow, SpannedError> {
    if let Some(head) = &program.head {
        eval(head, env.clone())?;
    }

    // Eval main program block
    eval(&program.main, env.clone())
}
