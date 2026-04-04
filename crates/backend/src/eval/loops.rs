use ultraviolet_core::{
    errors::SpannedError,
    types::{
        backend::{ControlFlow, EnvRef, Environment},
        frontend::ast::{ForLoop, Number, UVValue},
    },
};

use crate::eval::{eval, eval_block};

/// Evaluate for loop
pub fn eval_for_loop(for_node: &ForLoop, env: EnvRef) -> Result<ControlFlow, SpannedError> {
    let start_flow = eval(&for_node.start, env.clone())?;
    let ControlFlow::Simple(start) = start_flow else {
        return Ok(start_flow);
    };

    let end_flow = eval(&for_node.end, env.clone())?;
    let ControlFlow::Simple(end) = end_flow else {
        return Ok(end_flow);
    };

    let step = if let Some(step) = &for_node.step {
        let step_flow = eval(step, env.clone())?;
        let ControlFlow::Simple(step) = step_flow else {
            return Ok(step_flow);
        };

        step
    } else {
        UVValue::Number(Number::Int(1))
    };

    let loop_env = Environment::new_child(env.clone());
    loop_env
        .borrow_mut()
        .define_variable(for_node.iterator.value.clone(), start.clone(), false);

    loop {
        let current = loop_env
            .borrow()
            .find_var(&for_node.iterator.value)
            .unwrap(); // Lol, is this even panicable?

        if current.borrow().value >= end {
            break;
        }

        let result = eval_block(&for_node.body, loop_env.clone())?;

        match result {
            ControlFlow::Simple(_) | ControlFlow::Continue => {},
            ControlFlow::Break => break,
            _ => return Ok(result),
        }

        let new_val = &current.borrow().value + &step;

        (*loop_env
            .borrow_mut()
            .find_var(&for_node.iterator.value)
            .unwrap()
            .borrow_mut())
        .value = new_val;
    }

    Ok(ControlFlow::Simple(UVValue::Null))
}
