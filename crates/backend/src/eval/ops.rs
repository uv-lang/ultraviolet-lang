use crate::{Evaluator, eval::functions::EvalArgsResult};
use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::ast::GetOperands,
    types::{
        EnvRef,
        backend::{ControlFlow, RTVariable, UVRTValue},
    },
};

pub trait EvalOps: GetOperands {
    /// Evaluate operands and expr
    fn eval(
        &self,
        env: EnvRef<RTVariable>,
        evaluator: &Evaluator,
    ) -> Result<ControlFlow, SpannedError> {
        let values = match evaluator.eval_args(self.get_operands(), env.clone())? {
            EvalArgsResult::Values(v) => v,
            EvalArgsResult::Flow(cf) => return Ok(cf),
        };

        Ok(ControlFlow::Simple(self.eval_expr(values.as_slice())?))
    }

    fn eval_expr(&self, values: &[UVRTValue]) -> Result<UVRTValue, SpannedError>;
}
