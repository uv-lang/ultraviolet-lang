use ultraviolet_core::{
    errors::SpannedError,
    types::{
        backend::{ControlFlow, EnvRef},
        frontend::ast::UVValue,
    },
};

mod builtins;
pub mod eval;

pub trait EvalOps {
    /// Evaluate math operation
    fn eval(&self, env: EnvRef) -> Result<ControlFlow, SpannedError>;

    fn eval_expr(&self, values: &[UVValue]) -> Result<UVValue, SpannedError>;
}
