use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::Positional,
    types::{
        EnvRef, Environment,
        frontend::{
            Spanned,
            ast::{ForLoop, WhileLoop},
            number::UVNumberType,
            typechecker::{ControlFlow, UVTypeVariable},
            types::UVType,
        },
    },
};

use crate::typechecker::{analyze_group, typecheck};

/// Typecheck while loop
pub fn check_while_loop(
    wl: &Spanned<Box<WhileLoop>>,
    env: EnvRef<UVTypeVariable>,
) -> Result<ControlFlow, SpannedError> {
    let test = match typecheck(&wl.test, env.clone())? {
        ControlFlow::Simple(t) => t,
        cf => return Ok(cf),
    };

    if !matches!(test, UVType::Boolean) {
        return Err(SpannedError::new(
            format!(
                "While loop allows only `bool` for test block, but `{}` provided",
                test
            ),
            wl.span,
        ));
    }

    analyze_group(&wl.body, Environment::new_child(env.clone()))?;
    Ok(ControlFlow::Simple(UVType::Void))
}

/// Typecheck for loop
pub fn check_for_loop(
    fl: &Spanned<ForLoop>,
    env: EnvRef<UVTypeVariable>,
) -> Result<ControlFlow, SpannedError> {
    let child_env = Environment::new_child(env.clone());

    let start = match typecheck(&fl.start, env.clone())? {
        ControlFlow::Simple(s) => match s {
            UVType::Number(n) => n,
            _ => {
                return Err(SpannedError::new(
                    "Type mismatch for `for` start. Expected number",
                    fl.start.get_span(),
                ));
            },
        },
        cf => return Ok(cf),
    };

    let end = match typecheck(&fl.end, env.clone())? {
        ControlFlow::Simple(s) => match s {
            UVType::Number(n) => n,
            _ => {
                return Err(SpannedError::new(
                    "Type mismatch for `for` end. Expected number",
                    fl.start.get_span(),
                ));
            },
        },
        cf => return Ok(cf),
    };

    let step = if let Some(s) = &fl.step {
        match typecheck(s, env.clone())? {
            ControlFlow::Simple(s) => match s {
                UVType::Number(n) => n,
                _ => {
                    return Err(SpannedError::new(
                        "Type mismatch for `for` step. Expected number",
                        fl.start.get_span(),
                    ));
                },
            },
            cf => return Ok(cf),
        }
    } else {
        start.clone()
    };

    if !UVNumberType::all_eq(&[&start, &end, &step]) {
        return Err(SpannedError::new(
            "All loop parameters should be same type",
            fl.span,
        ));
    }

    child_env.borrow_mut().define_variable(
        &fl.iterator.value,
        UVTypeVariable::new_from(UVType::Number(start), true),
    );

    analyze_group(&fl.body, child_env)?;
    Ok(ControlFlow::Simple(UVType::Void))
}
