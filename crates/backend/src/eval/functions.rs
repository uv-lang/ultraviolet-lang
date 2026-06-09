use std::rc::Rc;

use ultraviolet_core::{
    errors::SpannedError,
    types::{
        EnvRef, Environment,
        backend::{ControlFlow, RTFunction, RTVariable, UVRTValue},
        frontend::ast::{FunctionCall, FunctionCallArg, FunctionDefinition},
    },
};

use crate::{
    eval::{eval, eval_block},
    ffi::call_dll,
};

/// Defines function in provided scope
pub fn define_function(
    def: &FunctionDefinition,
    env: EnvRef<RTVariable>,
) -> Result<ControlFlow, SpannedError> {
    let args: Vec<String> = def.arguments.iter().map(|e| e.name.value.clone()).collect();

    let f = UVRTValue::Function(RTFunction {
        args_names_order: args,
        body: def.body.clone(),
        lexical_env: Rc::downgrade(&env),
    });

    if let Some(name) = &def.name {
        env.borrow_mut()
            .define_variable(name.value.clone(), RTVariable::new_from(f, true));
        return Ok(ControlFlow::Simple(UVRTValue::Void));
    }

    Ok(ControlFlow::Simple(f))
}

/// Call function
pub fn call_function(
    call: &FunctionCall,
    env: EnvRef<RTVariable>,
) -> Result<ControlFlow, SpannedError> {
    let Some(f) = env.borrow().find_var(call.name.clone()) else {
        return Err(SpannedError::new(
            format!("`{}` not found", call.name),
            call.span,
        ));
    };

    if let UVRTValue::BuiltInFunction(f) = &f.borrow().value {
        let evaluated_args = match eval_args(&call.args, env.clone())? {
            EvalArgsResult::Values(v) => v,
            EvalArgsResult::Flow(cf) => return Ok(cf),
        };

        return (f.f)(&evaluated_args, env);
    }

    if let UVRTValue::FFIFunction = &f.borrow().value {
        let evaluated_args = match eval_args(&call.args, env.clone())? {
            EvalArgsResult::Values(v) => v,
            EvalArgsResult::Flow(cf) => return Ok(cf),
        };

        return call_dll(call, evaluated_args);
    }

    let UVRTValue::Function(f_struct) = &f.borrow().value else {
        return Err(SpannedError::new(
            format!("`{}` is not callable", call.name),
            call.span,
        ));
    };

    // This check is performed by typechecker
    /*
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
    */

    let evaluated_args = match eval_args(&call.args, env.clone())? {
        EvalArgsResult::Values(v) => v,
        EvalArgsResult::Flow(cf) => return Ok(cf),
    };

    let call_env = Environment::new_child(
        f_struct
            .lexical_env
            .upgrade()
            .ok_or_else(|| SpannedError::new("Lexical environment no longer exists", call.span))?,
    );
    for (name, value) in f_struct.args_names_order.iter().zip(evaluated_args) {
        call_env
            .borrow_mut()
            .define_variable(name, RTVariable::new_from(value, true));
    }

    let body_res = eval_block(&f_struct.body, call_env)?;
    let result = match body_res {
        ControlFlow::Return(t) => t,
        ControlFlow::Simple(_) => UVRTValue::Void,
        cf => return Ok(cf),
    };

    Ok(ControlFlow::Simple(result))
}

/// Possible return types for calculating arguments
///
/// The value inside Flow is propagated upstream
enum EvalArgsResult {
    Values(Vec<UVRTValue>),
    Flow(ControlFlow),
}

/// Evaluate function args
fn eval_args(
    args: &Vec<FunctionCallArg>,
    env: EnvRef<RTVariable>,
) -> Result<EvalArgsResult, SpannedError> {
    let mut evaluated_args: Vec<UVRTValue> = Vec::new();
    for arg in args {
        let v = eval(&arg.value, env.clone())?;
        evaluated_args.push(match v {
            ControlFlow::Simple(v) => v,
            fc => return Ok(EvalArgsResult::Flow(fc)),
        });
    }

    Ok(EvalArgsResult::Values(evaluated_args))
}
