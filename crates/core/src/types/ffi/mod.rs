use anyhow::{Result, anyhow};
use libffi::middle::{Arg, Type};
use std::{
    ffi::{CString, c_void},
    ptr,
};

use crate::{
    traits::ffi::{AsArg, ToFFIData, ToTypeFFI},
    types::{backend::UVRTValue, frontend::types::UVType},
};

impl ToTypeFFI for UVType {
    fn to_ffi_type(&self) -> Option<Type> {
        match self {
            UVType::Number(n) => n.to_ffi_type(),
            UVType::String => Some(Type::pointer()),
            UVType::Boolean => Some(Type::u8()),
            UVType::Void => Some(Type::void()),
            UVType::Null => Some(Type::pointer()),
            _ => None,
        }
    }
}

pub enum FFIData<'a> {
    Number(Arg<'a>),
    String(CString),
    Boolean(u8),
    Null,
}

impl ToFFIData for UVRTValue {
    fn to_ffi_data(&'_ self) -> Result<FFIData<'_>> {
        Ok(match self {
            UVRTValue::Number(n) => FFIData::Number(n.as_arg()),
            UVRTValue::String(s) => FFIData::String(
                CString::new(s.clone()).map_err(|_| anyhow!("Found zero byte in string"))?,
            ),
            UVRTValue::Boolean(b) => FFIData::Boolean(if *b { 1 } else { 0 }),
            UVRTValue::Null => FFIData::Null,
            _ => return Err(anyhow!("Cannot create C pointer to this value")),
        })
    }
}

impl<'a> AsArg for FFIData<'a> {
    fn as_arg(&self) -> Arg<'_> {
        match self {
            FFIData::Number(ptr) => ptr.clone(),
            FFIData::String(c_str) => Arg::new(c_str),
            FFIData::Boolean(b) => Arg::new(b),

            // TODO: A null pointer must not be passed as a value
            // In most FFIs, a null pointer is used to pass a value back
            // to the calling code. However, at the time of implementation of FFI in Ultraviolet there are no pointers
            FFIData::Null => Arg::new(&(ptr::null::<c_void>())),
        }
    }
}
