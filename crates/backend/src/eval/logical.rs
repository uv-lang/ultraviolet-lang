use ultraviolet_core::{
    errors::SpannedError,
    types::{
        EnvRef,
        backend::{ControlFlow, RTVariable, UVRTValue},
        frontend::ast::{LogicalOp, LogicalOpType},
    },
};

use crate::EvalOps;

impl EvalOps for LogicalOp {
    fn eval(&self, env: EnvRef<RTVariable>) -> Result<ControlFlow, SpannedError> {
        self._eval_with_operands(&self.operands, env)
    }

    fn eval_expr(&self, values: &[UVRTValue]) -> Result<UVRTValue, SpannedError> {
        let mut iter = values.iter();

        let result = match self.op_type {
            LogicalOpType::And => iter.all(|op| matches!(op, UVRTValue::Boolean(true))),
            LogicalOpType::Or => iter.any(|op| matches!(op, UVRTValue::Boolean(true))),
            LogicalOpType::Not => {
                let first = iter
                    .next()
                    .ok_or_else(|| SpannedError::new("empty operands", self.span))?
                    .clone();
                match first {
                    UVRTValue::Boolean(v) => !v,
                    _ => unreachable!(),
                }
            },
        };

        Ok(UVRTValue::Boolean(result))
    }
}
