use ultraviolet_core::{
    errors::SpannedError,
    types::{
        backend::{ControlFlow, EnvRef, Environment},
        frontend::ast::{ForLoop, Number, UVValue, WhileLoop},
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

    // FIXME: Должен ли интерпретатор создавать итератор в родительском скоупе для
    // снижения количества аллокаций для нового скоупа?
    env.borrow_mut()
        .define_variable(for_node.iterator.value.clone(), start.clone(), false);

    let loop_env = Environment::new_child(env.clone());

    loop {
        let current = env.borrow().find_var(&for_node.iterator.value).unwrap(); // Lol, is this even panicable?

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

        (*env
            .borrow_mut()
            .find_var(&for_node.iterator.value)
            .unwrap()
            .borrow_mut())
        .value = new_val;

        // Clearing current scope for next iteration
        loop_env.borrow_mut().symbols.clear();
    }

    env.borrow_mut().remove_symbol(&for_node.iterator.value);
    Ok(ControlFlow::Simple(UVValue::Void))
}

/// Eval while loop
pub fn eval_while_loop(while_node: &WhileLoop, env: EnvRef) -> Result<ControlFlow, SpannedError> {
    loop {
        let test = eval(&while_node.test, env.clone())?;
        let test_res = match test {
            ControlFlow::Simple(UVValue::Boolean(t)) => t,
            ControlFlow::Simple(_) => unreachable!("Typechecker bug"),
            _ => return Ok(test),
        };

        if !test_res {
            break;
        }

        let loop_env = Environment::new_child(env.clone());
        let result = eval_block(&while_node.body, loop_env.clone())?;
        match result {
            ControlFlow::Simple(_) | ControlFlow::Continue => {},
            ControlFlow::Break => break,
            _ => return Ok(result),
        }
    }

    Ok(ControlFlow::Simple(UVValue::Void))
}
