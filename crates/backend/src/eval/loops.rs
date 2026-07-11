use core::slice;

use crate::Evaluator;
use ultraviolet_core::{
    errors::SpannedError,
    traits::{
        EnvironmentTrait,
        frontend::{Positional, ast::GetType},
    },
    types::{
        EnvRef, Environment,
        backend::{ControlFlow, RTVariable, UVRTValue},
        frontend::{
            Spanned,
            ast::{ForLoop, WhileLoop},
            number::Number,
        },
    },
};

impl Evaluator {
    /// Evaluate for loop
    pub fn eval_for_loop(
        &self,
        for_node: &Spanned<ForLoop>,
        env: EnvRef<RTVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let start_flow = self.eval_single(&for_node.start, env.clone())?;
        let ControlFlow::Simple(start) = start_flow else {
            return Ok(start_flow);
        };

        let end_flow = self.eval_single(&for_node.end, env.clone())?;
        let ControlFlow::Simple(end) = end_flow else {
            return Ok(end_flow);
        };

        let step = if let Some(step) = &for_node.step {
            let step_flow = self.eval_single(step, env.clone())?;
            let ControlFlow::Simple(step) = step_flow else {
                return Ok(step_flow);
            };

            step
        } else {
            UVRTValue::Number(Number::auto(1, start.get_type()).map_err(|e| {
                SpannedError::new(
                    format!("Runtime Error: Cannot cast step for `for` loop: {}", e),
                    for_node.get_span(),
                )
            })?)
        };

        env.borrow_mut().define_variable(
            for_node.iterator.value.clone(),
            RTVariable::new_from(start.clone(), false),
        );

        let loop_env = Environment::new_child(env.clone());

        loop {
            let current = env.borrow().find_var(slice::from_ref(&for_node.iterator))?;
            if current.borrow().value > end {
                break;
            }

            let result = self.eval_block(&for_node.body, loop_env.clone())?;

            match result {
                ControlFlow::Simple(_) | ControlFlow::Continue => {},
                ControlFlow::Break => break,
                _ => return Ok(result),
            }

            let new_val = &current.borrow().value + &step;

            (*env
                .borrow_mut()
                .find_var(slice::from_ref(&for_node.iterator))?
                .borrow_mut())
            .value = new_val;

            // Clearing current scope for next iteration
            loop_env.borrow_mut().symbols.clear();
        }

        env.borrow_mut().remove_symbol(&for_node.iterator.value);
        Ok(ControlFlow::Simple(UVRTValue::Void))
    }

    /// Eval while loop
    pub fn eval_while_loop(
        &self,
        while_node: &WhileLoop,
        env: EnvRef<RTVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        loop {
            let test = self.eval_single(&while_node.test, env.clone())?;
            let test_res = match test {
                ControlFlow::Simple(UVRTValue::Boolean(t)) => t,
                ControlFlow::Simple(_) => unreachable!("Typechecker bug"),
                _ => return Ok(test),
            };

            if !test_res {
                break;
            }

            let loop_env = Environment::new_child(env.clone());
            let result = self.eval_block(&while_node.body, loop_env)?;
            match result {
                ControlFlow::Simple(_) | ControlFlow::Continue => {},
                ControlFlow::Break => break,
                _ => return Ok(result),
            }
        }

        Ok(ControlFlow::Simple(UVRTValue::Void))
    }
}
