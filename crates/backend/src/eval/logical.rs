use ultraviolet_core::{
    errors::SpannedError,
    types::{
        backend::{ControlFlow, EnvRef},
        frontend::ast::{LogicalOp, LogicalOpType, UVValue},
    },
};

use crate::EvalOps;

impl EvalOps for LogicalOp {
    fn eval(&self, env: EnvRef) -> Result<ControlFlow, SpannedError> {
        self._eval_with_operands(&self.operands, env)
    }

    fn eval_expr(&self, values: &[UVValue]) -> Result<UVValue, SpannedError> {
        let mut iter = values.iter();

        let result = match self.op_type {
            LogicalOpType::And => iter.all(|op| matches!(op, UVValue::Boolean(true))),
            LogicalOpType::Or => iter.any(|op| matches!(op, UVValue::Boolean(true))),
            LogicalOpType::Not => {
                let first = iter
                    .next()
                    .ok_or_else(|| SpannedError::new("empty operands", self.span))?
                    .clone();
                match first {
                    UVValue::Boolean(v) => !v,
                    _ => unreachable!(),
                }
            },
        };

        Ok(UVValue::Boolean(result))
    }
}
