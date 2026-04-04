use ultraviolet_core::{
    errors::SpannedError,
    types::{
        backend::{ControlFlow, EnvRef},
        frontend::ast::{MathOp, MathOpType, UVValue},
    },
};

use crate::eval::eval;

pub trait EvalMath {
    /// Evaluate math operation
    fn eval(&self, env: EnvRef) -> Result<ControlFlow, SpannedError>;

    fn eval_expr(&self, values: &[UVValue]) -> Result<UVValue, SpannedError>;
}

impl EvalMath for MathOp {
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
