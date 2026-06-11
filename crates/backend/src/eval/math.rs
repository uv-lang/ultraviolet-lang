use crate::eval::ops::EvalOps;
use ultraviolet_core::{
    errors::SpannedError,
    types::{
        backend::UVRTValue,
        frontend::{
            Spanned,
            ast::{BuiltInOperation, MathOpType},
        },
    },
};

impl EvalOps for Spanned<BuiltInOperation<MathOpType>> {
    fn eval_expr(&self, values: &[UVRTValue]) -> Result<UVRTValue, SpannedError> {
        let mut iter = values.iter();

        let first = iter
            .next()
            .ok_or_else(|| SpannedError::new("empty operands", self.span))?
            .clone();

        let result = match self.op_type {
            MathOpType::Sum => iter.fold(first, |acc, v| &acc + v),
            MathOpType::Sub => iter.fold(first, |acc, v| &acc - v),
            MathOpType::Mul => iter.fold(first, |acc, v| &acc * v),
            MathOpType::Div => iter.fold(first, |acc, v| &acc / v),
            MathOpType::Mod => iter.fold(first, |acc, v| &acc % v),
        };

        Ok(result)
    }
}
