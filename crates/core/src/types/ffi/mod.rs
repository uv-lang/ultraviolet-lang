use std::ffi::{CString, c_void};

use anyhow::{Result, anyhow};
use libffi::middle::Type;

use crate::{
    traits::ffi::{AsVoidPtr, ToFFIData, ToTypeFFI},
    types::{backend::UVRTValue, frontend::types::UVType},
};

impl ToTypeFFI for UVType {
    fn to_ffi_type(&self) -> Option<Type> {
        match self {
            UVType::Number(n) => n.to_ffi_type(),
            UVType::String => Some(Type::pointer()),
            UVType::Boolean => Some(Type::u8()),
            UVType::Void => Some(Type::void()),
            _ => None,
        }
    }
}

pub enum FFIData {
    Number(*const c_void),
    String(CString),
    Boolean(u8),
}

impl ToFFIData for UVRTValue {
    fn to_ffi_data(&self) -> Result<FFIData> {
        Ok(match self {
            UVRTValue::Number(n) => FFIData::Number(n.as_void_ptr()?),
            UVRTValue::String(s) => FFIData::String(
                CString::new(s.clone()).map_err(|_| anyhow!("Found zero byte in string"))?,
            ),
            UVRTValue::Boolean(b) => FFIData::Boolean(if *b { 1 } else { 0 }),
            _ => return Err(anyhow!("Cannot create C pointer to this value")),
        })
    }
}

impl AsVoidPtr for FFIData {
    fn as_void_ptr(&self) -> Result<*const c_void> {
        Ok(match self {
            FFIData::Number(ptr) => *ptr,
            FFIData::String(c_str) => c_str.as_ptr() as *const c_void,
            FFIData::Boolean(b) => b as *const u8 as *const c_void,
        })
    }
}
