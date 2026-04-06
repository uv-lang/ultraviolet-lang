use ultraviolet_core::{
    errors::SpannedError,
    types::{
        backend::{ControlFlow, EnvRef},
        frontend::ast::{CompareOp, CompareOpType, UVValue},
    },
};

use crate::EvalOps;

impl EvalOps for CompareOp {
    fn eval(&self, env: EnvRef) -> Result<ControlFlow, SpannedError> {
        self._eval_with_operands(&self.operands, env)
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
