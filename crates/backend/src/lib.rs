use ultraviolet_core::{
    errors::SpannedError,
    types::{
        EnvRef, Environment,
        backend::{ControlFlow, RTVariable, UVRTValue},
        frontend::ast::ASTBlockType,
    },
};

use crate::builtins::DefineBuiltinsRT;

mod builtins;
mod eval;
mod ffi;

/** Evaluate code */
pub fn eval(node: &ASTBlockType) -> Result<ControlFlow, SpannedError> {
    let env = Environment::new();
    env.define_builtins();

    eval::eval(node, env)
}

pub trait EvalOps {
    /// Evaluate operation
    fn eval(&self, env: EnvRef<RTVariable>) -> Result<ControlFlow, SpannedError>;

    /// Evaluate operands and eval expr
    fn _eval_with_operands(
        &self,
        ops: &Vec<ASTBlockType>,
        env: EnvRef<RTVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let mut values = Vec::new();

        for op in ops {
            let e_r = eval::eval(op, env.clone())?;
            let v = match e_r {
                ControlFlow::Simple(v) => v,
                _ => return Ok(e_r),
            };
            values.push(v);
        }

        Ok(ControlFlow::Simple(self.eval_expr(values.as_slice())?))
    }

    fn eval_expr(&self, values: &[UVRTValue]) -> Result<UVRTValue, SpannedError>;
}
