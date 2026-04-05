use ultraviolet_core::{
    errors::SpannedError,
    types::{
        backend::{ControlFlow, EnvRef},
        frontend::ast::{ASTBlockType, UVValue},
    },
};

mod builtins;
pub mod eval;

pub trait EvalOps {
    /// Evaluate operation
    fn eval(&self, env: EnvRef) -> Result<ControlFlow, SpannedError>;

    /// Evaluate operands and eval expr
    fn _eval_with_operands(
        &self,
        ops: &Vec<ASTBlockType>,
        env: EnvRef,
    ) -> Result<ControlFlow, SpannedError> {
        let mut values = Vec::new();

        for op in ops {
            let e_r = eval::eval(op, env.clone())?;
            let v = match e_r {
                ControlFlow::Simple(v) => v,
                _ => return Ok(e_r),
            };
            values.push(v);
        }

        Ok(ControlFlow::Simple(self.eval_expr(values.as_slice())?))
    }

    fn eval_expr(&self, values: &[UVValue]) -> Result<UVValue, SpannedError>;
}
