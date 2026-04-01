use crate::eval::{eval, eval_block};
use ultraviolet_core::{
    errors::SpannedError,
    types::{
        backend::{ControlFlow, EnvRef, Environment},
        frontend::ast::{ConditionalOperator, UVValue},
    },
};

/// Evaluate conditional operator
pub fn eval_conditional_op(co: &ConditionalOperator, env: EnvRef) -> Result<ControlFlow, SpannedError> {
    let cf = eval(&co.test, env.clone())?;
    let ControlFlow::Simple(evaluated_test) = cf else {
        return Ok(cf);
    };

    let test_result = match evaluated_test {
        UVValue::Boolean(b) => b,
        _ => {
            return Err(SpannedError::new(
                "Unexpected type for `test` expression. Expected `bool`",
                co.span,
            ));
        },
    };

    let branch = if test_result { &co.then_body } else { &co.else_body };

    if let Some(body) = branch {
        let new_env = Environment::new_child(env.clone());
        return eval_block(&body.value, new_env);
    }

    Ok(ControlFlow::Simple(UVValue::Void))
}
