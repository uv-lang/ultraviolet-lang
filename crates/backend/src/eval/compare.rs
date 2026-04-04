use ultraviolet_core::{
    errors::SpannedError,
    types::{
        backend::{ControlFlow, EnvRef},
        frontend::ast::{CompareOp, CompareOpType, UVValue},
    },
};

use crate::{EvalOps, eval::eval};

impl EvalOps for CompareOp {
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

        let result = match &self.op_type {
            CompareOpType::Equality => iter.all(|op| op == &first),

            op => {
                let a = &values[0];
                let b = &values[1];

                match op {
                    CompareOpType::NotEquality => a != b,
                    CompareOpType::Greater => a > b,
                    CompareOpType::GreaterEquals => a >= b,
                    CompareOpType::Less => a < b,
                    CompareOpType::LessEquals => a <= b,
                    _ => unreachable!(),
                }
            },
        };

        Ok(UVValue::Boolean(result))
    }
}
