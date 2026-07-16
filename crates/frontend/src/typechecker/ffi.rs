use std::{ops::Deref, slice};
use ultraviolet_core::{
    errors::SpannedError,
    traits::{EnvironmentTrait, ffi::ToTypeFFI, frontend::Positional},
    types::{
        EnvRef,
        frontend::{
            Spanned,
            ast::FFIDefinition,
            typechecker::{TControlFlow, UVTypeVariable},
            types::{UVFunctionType, UVType},
        },
    },
};

use crate::typechecker::Typechecker;

impl Typechecker {
    pub fn check_ffi_definition(
        &self,
        ffi_d: &Spanned<Box<FFIDefinition>>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<TControlFlow, SpannedError> {
        let mut cf = TControlFlow::new_void(ffi_d.get_span());
        if env
            .borrow()
            .find_var(slice::from_ref(&ffi_d.name.clone()))
            .is_ok()
        {
            return Err(SpannedError::new(
                format!("`{}` already defined", *ffi_d.name),
                ffi_d.get_span(),
            ));
        }

        // ------------------------------ <dll> ------------------------------
        let cfi = self.typecheck(&ffi_d.dll, env.clone())?;
        let dll_type = cfi.ty;
        cf.extend_returns(cfi.returns);

        UVType::String
            .is_assignable_from_many(&dll_type)
            .map_err(|t| {
                SpannedError::new(
                    format!("Type for <dll> mismatch. Expected <str />, found {}", t),
                    t.get_span(),
                )
            })?;

        // ------------------------------ <func> -----------------------------

        let cfi = self.typecheck(&ffi_d.func, env.clone())?;
        let func_type = cfi.ty;
        cf.extend_returns(cfi.returns);

        UVType::String
            .is_assignable_from_many(&func_type)
            .map_err(|t| {
                SpannedError::new(
                    format!("Type for <func> mismatch. Expected <str />, found {}", t),
                    t.get_span(),
                )
            })?;

        // ----------------------------- <arg> -------------------------------
        for arg in &ffi_d.arguments {
            if arg.to_ffi_type().is_none() {
                return Err(SpannedError::new(
                    format!("Type {} cannot be used as ffi argument", arg.deref()),
                    arg.get_span(),
                ));
            }
        }

        // ----------------------------- <returns> -------------------------------
        if let Some(t) = &ffi_d.return_type
            && t.to_ffi_type().is_none()
        {
            return Err(SpannedError::new(
                format!("Type {} cannot be used as ffi returns type", t.deref()),
                t.get_span(),
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

        Ok(cf)
    }
}
