use std::rc::Rc;

use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::Positional,
    types::{
        EnvRef, Environment,
        backend::{ControlFlow, RTFunction, RTVariable, UVRTValue},
        frontend::{
            Spanned,
            ast::{ASTBlockType, FunctionCall, FunctionDefinition},
        },
    },
};

use crate::eval::{eval, eval_block, ffi::call_dll};

/// Defines function in provided scope
pub fn define_function(
    def: &Spanned<FunctionDefinition>,
    env: EnvRef<RTVariable>,
) -> Result<ControlFlow, SpannedError> {
    let args: Vec<String> = def.arguments.iter().map(|e| e.name.value.clone()).collect();

    let f = UVRTValue::Function(RTFunction {
        args_names_order: args,
        body: def.value.body.clone(),
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
    call: &Spanned<FunctionCall>,
    env: EnvRef<RTVariable>,
) -> Result<ControlFlow, SpannedError> {
    let Some(f) = env.borrow().find_var(call.name.clone()) else {
        return Err(SpannedError::new(
            format!("`{}` not found", call.name),
            call.get_span(),
        ));
    };

    if let UVRTValue::BuiltInFunction(f) = &f.borrow().value {
        let evaluated_args = match eval_args(&call.args, env.clone())? {
            EvalArgsResult::Values(v) => v,
            EvalArgsResult::Flow(cf) => return Ok(cf),
        };

        return (f.f)(&evaluated_args, env);
    }

    if let UVRTValue::FFIFunction(f) = &f.borrow().value {
        let evaluated_args = match eval_args(&call.args, env.clone())? {
            EvalArgsResult::Values(v) => v,
            EvalArgsResult::Flow(cf) => return Ok(cf),
        };

        return call_dll(call, evaluated_args, f);
    }

    let UVRTValue::Function(f_struct) = &f.borrow().value else {
        return Err(SpannedError::new(
            format!("`{}` is not callable", call.name),
            call.get_span(),
        ));
    };

    let evaluated_args = match eval_args(&call.args, env.clone())? {
        EvalArgsResult::Values(v) => v,
        EvalArgsResult::Flow(cf) => return Ok(cf),
    };

    let call_env = Environment::new_child(f_struct.lexical_env.upgrade().ok_or_else(|| {
        SpannedError::new("Lexical environment no longer exists", call.get_span())
    })?);
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
pub enum EvalArgsResult {
    Values(Vec<UVRTValue>),
    Flow(ControlFlow),
}

/// Evaluate function args
pub fn eval_args(
    args: &Vec<Spanned<ASTBlockType>>,
    env: EnvRef<RTVariable>,
) -> Result<EvalArgsResult, SpannedError> {
    let mut evaluated_args: Vec<UVRTValue> = Vec::new();
    for arg in args {
        let v = eval(&arg.value, env.clone())?;
        evaluated_args.push(match v {
            ControlFlow::Simple(v) => v,
            cf => return Ok(EvalArgsResult::Flow(cf)),
        });
    }

    Ok(EvalArgsResult::Values(evaluated_args))
}
