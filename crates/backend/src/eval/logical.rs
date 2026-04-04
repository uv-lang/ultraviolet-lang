use ultraviolet_core::{
    errors::SpannedError,
    types::{
        backend::{ControlFlow, EnvRef},
        frontend::ast::{LogicalOp, LogicalOpType, UVValue},
    },
};

use crate::{EvalOps, eval::eval};

impl EvalOps for LogicalOp {
    fn eval(&self, env: EnvRef) -> Result<ControlFlow, SpannedError> {
        let mut values = Vec::new();

        for op in &self.operands {
            let e_r = eval(op, env.clone())?;
            let v = match e_r {
                ControlFlow::Simple(v) => v,
                _ => return Ok(e_r),
            };
            values.push(v);
        }

        Ok(ControlFlow::Simple(self.eval_expr(values.as_slice())?))
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
