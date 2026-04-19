use crate::eval::eval;
use ultraviolet_core::{
    errors::SpannedError,
    types::{
        backend::{ControlFlow, EnvRef, Environment, UVRTValue},
        frontend::ast::{VariableAccess, VariableAssign, VariableDefinition},
    },
};

/// Define variable
pub fn define_variable(
    var_def: &VariableDefinition,
    env: EnvRef,
) -> Result<ControlFlow, SpannedError> {
    if env.borrow().find_var(var_def.name.value.clone()).is_some() {
        return Err(SpannedError::new(
            format!("Variable `{}` already defined", var_def.name.value),
            var_def.span,
        ));
    }

    match eval(&var_def.value.value, env.clone())? {
        ControlFlow::Simple(value) => {
            env.borrow_mut().define_variable(
                var_def.name.value.clone(),
                value.clone(),
                var_def.is_const,
            );
            Ok(ControlFlow::Simple(UVRTValue::Void))
        },
        cf => Ok(cf),
    }
}

/// Access variable by value
pub fn access_variable(var_acc: &VariableAccess, env: EnvRef) -> Result<ControlFlow, SpannedError> {
    match env.borrow().find_var(var_acc.name.clone()) {
        Some(sym) => Ok(ControlFlow::Simple(sym.borrow().clone().value)),
        None => Err(SpannedError::new(
            format!("Name `{}` not defined", var_acc.name),
            var_acc.span,
        )),
    }
}

/// Assign to a variable
pub fn assign_variable(
    assign_var: &VariableAssign,
    env: EnvRef,
) -> Result<ControlFlow, SpannedError> {
    let sym = env
        .borrow()
        .find_var(assign_var.name.clone())
        .ok_or_else(|| {
            SpannedError::new(
                format!("Variable `{}` not defined", assign_var.name),
                assign_var.span,
            )
        })?;

    if sym.borrow().constant {
        return Err(SpannedError::new(
            "Cannot assign to a constant variable",
            assign_var.span,
        ));
    }

    let new_env = Environment::new_child(env);
    let result = eval(&assign_var.value, new_env)?;

    if let ControlFlow::Simple(uvvalue) = result {
        (*sym.borrow_mut()).value = uvvalue;
        Ok(ControlFlow::Simple(UVRTValue::Void))
    } else {
        Ok(result)
    }
}
