use ultraviolet_core::{
    errors::SpannedError,
    types::{
        EnvRef,
        backend::{ControlFlow, RTVariable, UVRTValue},
        frontend::ast::{MathOp, MathOpType},
    },
};

use crate::EvalOps;

impl EvalOps for MathOp {
    fn eval(&self, env: EnvRef<RTVariable>) -> Result<ControlFlow, SpannedError> {
        self._eval_with_operands(&self.operands, env)
    }

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
