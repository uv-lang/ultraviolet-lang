use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::ast::IsAssignable,
    types::frontend::{
        ast::{UVType, VariableDefinition},
        typechecker::{ControlFlow, EnvRef},
    },
};

use crate::typechecker::typecheck;

/// Definition and checking of variable types
pub fn check_variable_definition(
    vd: &VariableDefinition,
    env: EnvRef,
) -> Result<ControlFlow, SpannedError> {
    let val = match typecheck(&vd.value.value, env.clone())? {
        ControlFlow::Simple(uvtype) => uvtype,
        cf => return Ok(cf),
    };

    if let Some(expected) = &vd.expected_type {
        if !expected.value.is_assignable_from(&val) {
            return Err(SpannedError::new(
                format!(
                    "Mismatched types: variable `{}` expected type `{}` but type `{}` are provided",
                    vd.name.value, expected.value, val
                ),
                vd.value.span,
            ));
        }
    }

    env.borrow_mut()
        .define_variable(vd.name.value.clone(), val, vd.is_const);

    Ok(ControlFlow::Simple(UVType::Void))
}
