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
        frontend::token_parser::UnwrapOptionError,
    },
    types::{
        EnvRef,
        backend::{ControlFlow, RTVariable, UVRTValue, ffi::FFIFunction},
        ffi::FFIData,
        frontend::{
            ast::{FFIDefinition, FunctionCall},
            types::UVType,
        },
    },
};

use crate::eval::eval;

static DLLS: OnceLock<RwLock<HashMap<String, Arc<Library>>>> = OnceLock::new();
static DLL_SYMBOLS: OnceLock<RwLock<HashMap<String, Arc<FFIFunction>>>> = OnceLock::new();

/// All loaded dlls
pub fn libraries() -> &'static RwLock<HashMap<String, Arc<Library>>> {
    DLLS.get_or_init(|| RwLock::new(HashMap::new()))
}

/// All loaded dll symbols
pub fn dll_symbols() -> &'static RwLock<HashMap<String, Arc<FFIFunction>>> {
    DLL_SYMBOLS.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Load DLL into memory
pub fn load_dll(
    ffi_def: &FFIDefinition,
    env: EnvRef<RTVariable>,
) -> Result<ControlFlow, SpannedError> {
    let lib_path = match eval(ffi_def.dll.deref(), env.clone())? {
        // Other types in this hand is unreachable due typecheck
        ControlFlow::Simple(UVRTValue::String(v)) => v,
        cf => return Ok(cf),
    };

    let lib = libraries()
        .try_read()
        .map_err(|e| SpannedError::new(format!("Cannot lock dll store: {e}"), ffi_def.span))?
        .get(&lib_path)
        .cloned();

    let lib = match lib {
        Some(lib) => lib,
        None => {
            let lib = unsafe {
                Arc::new(Library::new(&lib_path).map_err(|e| {
                    SpannedError::new(format!("Cannot load DLL {}: {}", lib_path, e), ffi_def.span)
                })?)
            };

            libraries()
                .try_write()
                .map_err(|e| {
                    SpannedError::new(
                        format!("Cannot save DLL reference for {}: {}", lib_path, e),
                        ffi_def.span,
                    )
                })?
                .insert(lib_path, lib.clone());

            lib
        },
    };

    let func_name = match eval(ffi_def.func.deref(), env.clone())? {
        // Other types in this hand is unreachable due typecheck
        ControlFlow::Simple(UVRTValue::String(v)) => v,
        cf => return Ok(cf),
    };

    let cname = CString::new(func_name.clone())
        .map_err(|e| SpannedError::new(format!("Invalid function name: {}", e), ffi_def.span))?;

    let func_symbol: Symbol<unsafe extern "C" fn()> = unsafe {
        let sym: Symbol<unsafe extern "C" fn()> =
            lib.get(cname.as_bytes_with_nul()).map_err(|e| {
                SpannedError::new(format!("Can't get symbol from dll: {}", e), ffi_def.span)
            })?;
        std::mem::transmute(sym)
    };
    let func_ptr = CodePtr::from_ptr(*func_symbol as *const c_void);

    let arg_types = ffi_def
        .arguments
        .iter()
        .map(|arg| arg.value.to_ffi_type().unwrap_or_spanned(ffi_def.span))
        .collect::<Result<Vec<Type>, SpannedError>>()?;

    let returns = match &ffi_def.return_type {
        Some(t) => t.deref().clone(),
        None => UVType::Void,
    }
    .to_ffi_type()
    .unwrap_or_spanned(ffi_def.span)?;

    let cif = Cif::new(arg_types, returns);

    dll_symbols()
        .try_write()
        .map_err(|e| {
            SpannedError::new(format!("Cannot acquire internal writer: {e}"), ffi_def.span)
        })?
        .insert(
            ffi_def.name.deref().clone(),
            Arc::new(FFIFunction {
                _lib: lib.clone(),
                func_symbol,
                func_ptr,
                cif,
                returns: ffi_def.return_type.clone(),
            }),
        );

    env.borrow_mut().define_variable(
        ffi_def.name.deref(),
        RTVariable::new_from(UVRTValue::FFIFunction, true),
    );

    Ok(ControlFlow::Simple(UVRTValue::Void))
}

/// Call already loaded DLL function
pub fn call_dll(call: &FunctionCall, args: Vec<UVRTValue>) -> Result<ControlFlow, SpannedError> {
    let binding = dll_symbols()
        .try_read()
        .map_err(|e| SpannedError::new(format!("Cannot lock dll symbols store: {e}"), call.span))?;

    let f = binding.get(&call.name).ok_or(SpannedError::new(
        format!("{} not loaded", call.name),
        call.span,
    ))?;

    let args_data = args
        .iter()
        .map(|a| a.to_ffi_data())
        .collect::<Result<Vec<FFIData>>>()
        .map_err(|e| {
            SpannedError::new(
                format!("Cannot convert ultraviolet value to a C-like: {e}"),
                call.span,
            )
        })?;

    let args_data_ffi = args_data.iter().map(|a| a.as_arg()).collect::<Vec<Arg>>();
    unsafe {
        let ret: u64 = f.cif.call::<u64>(f.func_ptr, &args_data_ffi);

        let result = if let Some(rt) = &f.returns {
            ret.to_uv_value(rt.value.clone())
                .map_err(|e| SpannedError::new(format!("{e}"), rt.span))?
        } else {
            UVRTValue::Void
        };

        Ok(ControlFlow::Simple(result))
    }
}
