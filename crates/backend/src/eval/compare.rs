use crate::eval::ops::EvalOps;
use ultraviolet_core::{
    errors::SpannedError,
    types::{
        backend::UVRTValue,
        frontend::{
            Spanned,
            ast::{BuiltInOperation, CompareOpType},
        },
    },
};

impl EvalOps for Spanned<BuiltInOperation<CompareOpType>> {
    fn eval_expr(&self, values: &[UVRTValue]) -> Result<UVRTValue, SpannedError> {
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

        Ok(UVRTValue::Boolean(result))
    }
}
