use crate::typechecker::typecheck;
use std::ops::Deref;
use ultraviolet_core::{
    errors::SpannedError,
    traits::ffi::ToTypeFFI,
    types::{
        EnvRef,
        frontend::{
            Spanned,
            ast::FFIDefinition,
            typechecker::{ControlFlow, UVTypeVariable},
            types::{UVFunctionType, UVType},
        },
    },
};

pub fn check_ffi_definition(
    ffi_d: &Spanned<Box<FFIDefinition>>,
    env: EnvRef<UVTypeVariable>,
) -> Result<ControlFlow, SpannedError> {
    if env.borrow().find_var(ffi_d.name.deref()).is_some() {
        return Err(SpannedError::new(
            format!("`{}` already defined", *ffi_d.name),
            ffi_d.span,
        ));
    }

    // ------------------------------ <dll> ------------------------------
    let dll_type = match typecheck(&ffi_d.dll, env.clone())? {
        ControlFlow::Simple(t) => t,
        cf => return Ok(cf),
    };

    if !matches!(dll_type, UVType::String) {
        return Err(SpannedError::new(
            format!(
                "Type for <dll> mismatch. Expected <str />, found {}",
                dll_type
            ),
            ffi_d.dll.span,
        ));
    }

    // ------------------------------ <func> -----------------------------

    let func_type = match typecheck(&ffi_d.func, env.clone())? {
        ControlFlow::Simple(t) => t,
        cf => return Ok(cf),
    };

    if !matches!(func_type, UVType::String) {
        return Err(SpannedError::new(
            format!(
                "Type for <func> mismatch. Expected <str />, found {}",
                func_type
            ),
            ffi_d.func.span,
        ));
    }

    // ----------------------------- <arg> -------------------------------
    for arg in &ffi_d.arguments {
        if arg.to_ffi_type().is_none() {
            return Err(SpannedError::new(
                format!("Type {} cannot be used as ffi argument", arg.deref()),
                arg.span,
            ));
        }
    }

    // ----------------------------- <returns> -------------------------------
    if let Some(t) = &ffi_d.return_type
        && t.to_ffi_type().is_none()
    {
        return Err(SpannedError::new(
            format!("Type {} cannot be used as ffi returns type", t.deref()),
            t.span,
        ));
    }

    let exp = ffi_d
        .return_type
        .clone()
        .map(|t| t.value)
        .unwrap_or(UVType::Void);

    env.borrow_mut().define_variable(
        ffi_d.name.deref(),
        UVTypeVariable::new_from(
            UVType::Function(Box::new(UVFunctionType {
                args: ffi_d.arguments.iter().map(|a| a.value.clone()).collect(),
                returns: exp,
            })),
            true,
        ),
    );

    Ok(ControlFlow::Simple(UVType::Void))
}
