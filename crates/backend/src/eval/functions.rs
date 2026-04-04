use ultraviolet_core::{
    errors::SpannedError,
    types::{
        backend::{ControlFlow, EnvRef, Environment, RTFunction},
        frontend::ast::{FunctionCall, FunctionDefinition, UVValue},
    },
};

use crate::eval::{eval, eval_block};

/// Defines function in provided scope
pub fn define_function(def: &FunctionDefinition, env: EnvRef) -> Result<ControlFlow, SpannedError> {
    let args: Vec<String> = def.arguments.iter().map(|e| e.name.value.clone()).collect();

    env.borrow_mut().define_function(
        def.name.value.clone(),
        RTFunction {
            args_names_order: args,
            body: def.body.clone(),
            lexical_env: env.clone(),
        },
    );

    Ok(ControlFlow::Simple(UVValue::Void))
}

/// Call function
pub fn call_function(call: &FunctionCall, env: EnvRef) -> Result<ControlFlow, SpannedError> {
    let Some(f) = env.borrow().find_func(call.name.clone()) else {
        return Err(SpannedError::new(
            format!("Function `{}` not defined", call.name),
            call.span,
        ));
    };

    let f_struct = f.borrow();
    let call_env = Environment::new_child(f_struct.lexical_env.clone());

    if f_struct.args_names_order.len() != call.args.len() {
        return Err(SpannedError::new(
            format!(
                "Function `{}` accepts {} arguments, but {} are passed",
                call.name,
                f_struct.args_names_order.len(),
                call.args.len()
            ),
            call.span,
        ));
    }

    let mut evaluated_args = Vec::new();
    for arg in &call.args {
        let v = eval(&arg.value, env.clone())?;
        evaluated_args.push(match v {
            ControlFlow::Simple(v) => v,
            fc => return Ok(fc),
        });
    }

    for (name, value) in f_struct.args_names_order.iter().zip(evaluated_args) {
        call_env.borrow_mut().define_variable(name, value, true);
    }

    let body_res = eval_block(&f_struct.body, call_env)?;
    let result = match body_res {
        ControlFlow::Return(t) => t,
        ControlFlow::Simple(_) => UVValue::Void,
        cf => return Ok(cf),
    };

    Ok(ControlFlow::Simple(result))
}
