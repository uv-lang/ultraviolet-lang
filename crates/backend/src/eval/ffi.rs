use anyhow::Result;
use libffi::{
    low::CodePtr,
    middle::{Arg, Cif, Type},
};
use libloading::{Library, Symbol};
use std::{
    collections::HashMap,
    ffi::{CString, c_void},
    ops::Deref,
    sync::{Arc, OnceLock, RwLock},
};
use ultraviolet_core::{
    errors::SpannedError,
    traits::{
        ffi::{AsArg, FromFFI, ToFFIData, ToTypeFFI},
        frontend::{Positional, token_parser::UnwrapOptionError},
    },
    types::{
        EnvRef,
        backend::{ControlFlow, RTVariable, UVRTValue, ffi::FFIFunction},
        ffi::FFIData,
        frontend::{
            Spanned,
            ast::{FFIDefinition, FunctionCall},
            types::UVType,
        },
    },
};

use crate::Evaluator;

static DLLS: OnceLock<RwLock<HashMap<String, Arc<Library>>>> = OnceLock::new();

/// All loaded dlls
pub fn libraries() -> &'static RwLock<HashMap<String, Arc<Library>>> {
    DLLS.get_or_init(|| RwLock::new(HashMap::new()))
}

impl Evaluator {
    /// Load DLL into memory
    pub fn load_dll(
        &self,
        ffi_def: &Spanned<Box<FFIDefinition>>,
        env: EnvRef<RTVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let lib_path = match self.eval_single(ffi_def.dll.deref(), env.clone())? {
            // Other types in this hand is unreachable due typecheck
            ControlFlow::Simple(UVRTValue::String(v)) => v,
            cf => return Ok(cf),
        };

        let lib = libraries()
            .try_read()
            .map_err(|e| {
                SpannedError::new(format!("Cannot lock dll store: {e}"), ffi_def.get_span())
            })?
            .get(&lib_path)
            .cloned();

        let lib = match lib {
            Some(lib) => lib,
            None => {
                let lib = unsafe {
                    Arc::new(Library::new(&lib_path).map_err(|e| {
                        SpannedError::new(
                            format!("Cannot load DLL {}: {}", lib_path, e),
                            ffi_def.get_span(),
                        )
                    })?)
                };

                libraries()
                    .try_write()
                    .map_err(|e| {
                        SpannedError::new(
                            format!("Cannot save DLL reference for {}: {}", lib_path, e),
                            ffi_def.get_span(),
                        )
                    })?
                    .insert(lib_path, lib.clone());

                lib
            },
        };

        let func_name = match self.eval_single(ffi_def.func.deref(), env.clone())? {
            // Other types in this hand is unreachable due typecheck
            ControlFlow::Simple(UVRTValue::String(v)) => v,
            cf => return Ok(cf),
        };

        let cname = CString::new(func_name.clone()).map_err(|e| {
            SpannedError::new(format!("Invalid function name: {}", e), ffi_def.get_span())
        })?;

        let func_symbol: Symbol<unsafe extern "C" fn()> = unsafe {
            lib.get(cname.as_bytes_with_nul()).map_err(|e| {
                SpannedError::new(
                    format!("Can't get symbol from dll: {}", e),
                    ffi_def.get_span(),
                )
            })?
        };

        let func_ptr = CodePtr::from_ptr(*func_symbol as *const c_void);

        let arg_types = ffi_def
            .arguments
            .iter()
            .map(|arg| {
                arg.value
                    .to_ffi_type()
                    .unwrap_or_spanned(ffi_def.get_span())
            })
            .collect::<Result<Vec<Type>, SpannedError>>()?;

        let returns = match &ffi_def.return_type {
            Some(t) => t.deref().clone(),
            None => UVType::Void,
        }
        .to_ffi_type()
        .unwrap_or_spanned(ffi_def.get_span())?;

        let cif = Cif::new(arg_types, returns);

        env.borrow_mut().define_variable(
            ffi_def.name.deref(),
            RTVariable::new_from(
                UVRTValue::FFIFunction(FFIFunction {
                    _lib: lib.clone(),
                    func_ptr,
                    cif,
                    returns: ffi_def.return_type.clone(),
                }),
                true,
            ),
        );

        Ok(ControlFlow::Simple(UVRTValue::Void))
    }

    /// Call already loaded DLL function
    pub fn call_dll(
        call: &Spanned<FunctionCall>,
        args: Vec<UVRTValue>,
        f: &FFIFunction,
    ) -> Result<ControlFlow, SpannedError> {
        let args_data = args
            .iter()
            .map(|a| a.to_ffi_data())
            .collect::<Result<Vec<FFIData>>>()
            .map_err(|e| {
                SpannedError::new(
                    format!("Cannot convert ultraviolet value to a C-like: {e}"),
                    call.get_span(),
                )
            })?;

        let args_data_ffi = args_data.iter().map(|a| a.as_arg()).collect::<Vec<Arg>>();
        unsafe {
            let ret: u64 = f.cif.call::<u64>(f.func_ptr, &args_data_ffi);

            let result = if let Some(rt) = &f.returns {
                ret.to_uv_value(rt.value.clone())
                    .map_err(|e| SpannedError::new(format!("{e}"), rt.get_span()))?
            } else {
                UVRTValue::Void
            };

            Ok(ControlFlow::Simple(result))
        }
    }
}
