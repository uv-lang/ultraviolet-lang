use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::ast::IsAssignable,
    types::{
        EnvRef,
        frontend::{
            ast::{UVType, VariableAssign, VariableDefinition},
            typechecker::{ControlFlow, UVTypeVariable},
        },
    },
};

use crate::typechecker::typecheck;

/// Definition and checking of variable types
pub fn check_variable_definition(
    vd: &VariableDefinition,
    env: EnvRef<UVTypeVariable>,
) -> Result<ControlFlow, SpannedError> {
    let val = match typecheck(&vd.value.value, env.clone())? {
        ControlFlow::Simple(uvtype) => uvtype,
        cf => return Ok(cf),
    };

    if let Some(expected) = &vd.expected_type
        && !expected.value.is_assignable_from(&val)
    {
        return Err(SpannedError::new(
            format!(
                "Mismatched types: variable `{}` expected type `{}` but type `{}` is provided",
                vd.name.value, expected.value, val
            ),
            vd.value.span,
        ));
    }

    env.borrow_mut().define_variable(
        vd.name.value.clone(),
        UVTypeVariable::new_from(val, vd.is_const),
    );

    Ok(ControlFlow::Simple(UVType::Void))
}

/// Check variable assignment
pub fn check_variable_assign(
    va: &VariableAssign,
    env: EnvRef<UVTypeVariable>,
) -> Result<ControlFlow, SpannedError> {
    let Some(var_rc) = env.borrow().find_var(va.name.clone()) else {
        return Err(SpannedError::new(
            format!("Variable `{}` not found", va.name),
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
                "Mismatched types: variable `{}` expected type `{}` but type `{}` is provided",
                va.name, var.value, t
            ),
            va.value.span,
        ));
    }

    Ok(ControlFlow::Simple(UVType::Void))
}
