use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::ast::IsAssignable,
    types::{
        EnvRef,
        frontend::{
            ast::{VariableAccess, VariableAssign, VariableDefinition},
            typechecker::{ControlFlow, UVTypeVariable},
            types::UVType,
        },
    },
};

use crate::typechecker::typecheck;

/// Definition and checking of variable types
pub fn check_variable_definition(
    vd: &VariableDefinition,
    env: EnvRef<UVTypeVariable>,
) -> Result<ControlFlow, SpannedError> {
    let mut val = match typecheck(&vd.value.value, env.clone())? {
        ControlFlow::Simple(uvtype) => uvtype,
        cf => return Ok(cf),
    };

    if let Some(expected) = &vd.expected_type {
        if !expected.value.is_assignable_from(&val) {
            return Err(SpannedError::new(
                format!(
                    "Expected type `{}`, got `{}` for variable `{}`",
                    expected.value, val, vd.name.value
                ),
                vd.value.span,
            ));
        }

        val = expected.value.clone();
    }

    if env.borrow().find_var(&vd.name.value).is_some() {
        return Err(SpannedError::new(
            format!("Variable with name {} already defined", vd.name.value),
            vd.span,
        ));
    }

    env.borrow_mut()
        .define_variable(&vd.name.value, UVTypeVariable::new_from(val, vd.is_const));

    Ok(ControlFlow::Simple(UVType::Void))
}

/// Check variable assignment
pub fn check_variable_assign(
    va: &VariableAssign,
    env: EnvRef<UVTypeVariable>,
) -> Result<ControlFlow, SpannedError> {
    let Some(var_rc) = env.borrow().find_var(&va.name) else {
        return Err(SpannedError::new(
            format!("Variable `{}` not defined", va.name),
            va.span,
        ));
    };

    let var = var_rc.borrow();
    if var.constant {
        return Err(SpannedError::new(
            format!("Cannot assign to a constant `{}` variable", va.name),
            va.span,
        ));
    }

    let t = match typecheck(&va.value.value, env.clone())? {
        ControlFlow::Simple(uvtype) => uvtype,
        cf => return Ok(cf),
    };

    if !var.value.is_assignable_from(&t) {
        return Err(SpannedError::new(
            format!(
                "Expected type `{}`, got `{}` for variable `{}`",
                var.value, t, va.name
            ),
            va.value.span,
        ));
    }

    Ok(ControlFlow::Simple(UVType::Void))
}

/// Check variable is defined and get its type
pub fn check_variable_access(
    va: &VariableAccess,
    env: EnvRef<UVTypeVariable>,
) -> Result<ControlFlow, SpannedError> {
    let Some(var_rc) = env.borrow().find_var(&va.name) else {
        return Err(SpannedError::new(
            format!("Variable `{}` not defined", va.name),
            va.span,
        ));
    };

    Ok(ControlFlow::Simple(var_rc.borrow().value.clone()))
}
