use crate::eval::eval;
use ultraviolet_core::{
    errors::SpannedError,
    types::{
        EnvRef,
        backend::{ControlFlow, RTVariable},
        frontend::ast::ProgramBlock,
    },
};

/// Evaluate program block
pub fn eval_program(
    program: &ProgramBlock,
    env: EnvRef<RTVariable>,
) -> Result<ControlFlow, SpannedError> {
    if let Some(head) = &program.head {
        eval(head, env.clone())?;
    }

    // Eval main program block
    eval(&program.main, env.clone())
}
