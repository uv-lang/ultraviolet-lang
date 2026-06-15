use crate::eval::ops::EvalOps;
use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::Positional,
    types::{
        backend::UVRTValue,
        frontend::{
            Spanned,
            ast::{BuiltInOperation, LogicalOpType},
        },
    },
};

impl EvalOps for Spanned<BuiltInOperation<LogicalOpType>> {
    fn eval_expr(&self, values: &[UVRTValue]) -> Result<UVRTValue, SpannedError> {
        let mut iter = values.iter();

        let result = match self.op_type {
            LogicalOpType::And => iter.all(|op| matches!(op, UVRTValue::Boolean(true))),
            LogicalOpType::Or => iter.any(|op| matches!(op, UVRTValue::Boolean(true))),
            LogicalOpType::Not => {
                let first = iter
                    .next()
                    .ok_or_else(|| SpannedError::new("empty operands", self.get_span()))?
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
