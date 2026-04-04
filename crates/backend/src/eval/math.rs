use ultraviolet_core::{
    errors::SpannedError,
    types::{
        backend::{ControlFlow, EnvRef},
        frontend::ast::{MathOp, MathOpType},
    },
};

use crate::eval::eval;

pub trait EvalMath {
    /// Evaluate math operation
    fn eval(&self, env: EnvRef) -> Result<ControlFlow, SpannedError>;
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

        match self.op_type {
            MathOpType::Sum => todo!(),
            MathOpType::Sub => todo!(),
            MathOpType::Mul => todo!(),
            MathOpType::Div => todo!(),
            MathOpType::Mod => todo!(),
        }

        todo!()
    }
}
