use libffi::middle::{Arg, Type};
use std::ffi::{CString, c_void};

use core::mem::offset_of;

use crate::{
    errors::CommonError,
    traits::ffi::{AsArg, EnumPayloadPtr, ToFFIData, ToTypeFFI},
    types::{
        backend::{RTVariable, UVRTValue},
        frontend::types::UVType,
    },
};

impl ToTypeFFI for UVType {
    fn to_ffi_type(&self) -> Option<Type> {
        match self {
            UVType::Number(n) => n.to_ffi_type(),
            UVType::String => Some(Type::pointer()),
            UVType::Boolean => Some(Type::u8()),
            UVType::Void => Some(Type::void()),
            UVType::Null => Some(Type::pointer()),
            UVType::Reference(_) => Some(Type::pointer()),
            _ => None,
        }
    }
}

pub enum FFIData<'a> {
    Arg(Arg<'a>),
    String(CString),
    Boolean(u8),
    Reference(*const c_void),
}

impl ToFFIData for UVRTValue {
    fn to_ffi_data(&'_ self) -> Result<FFIData<'_>, CommonError> {
        Ok(match self {
            UVRTValue::Number(n) => FFIData::Arg(n.as_arg()),
            UVRTValue::String(s) => FFIData::String(
                CString::new(s.clone())
                    .map_err(|_| CommonError::new("Found zero byte in string"))?,
            ),
            UVRTValue::Boolean(b) => FFIData::Boolean(if *b { 1 } else { 0 }),
            UVRTValue::Reference(data) => unsafe {
                // create an owned pointer boxed inside the enum so its address lives
                // for the lifetime of the FFIData value
                let strong = data
                    .upgrade()
                    .ok_or_else(|| CommonError::new("Reference dropped"))?;

                let base = strong.as_ptr() as *const c_void;
                let field = base.add(offset_of!(RTVariable, value));
                FFIData::Reference(UVRTValue::payload_ptr(field as *mut UVRTValue))
            },
            _ => return Err(CommonError::new("Cannot create C pointer to this value")),
        })
    }
}

impl<'a> AsArg for FFIData<'a> {
    fn as_arg(&self) -> Arg<'_> {
        match self {
            FFIData::Arg(ptr) => ptr.clone(),
            FFIData::String(c_str) => Arg::new(c_str),
            FFIData::Boolean(b) => Arg::new(b),
            FFIData::Reference(ptr) => Arg::new(ptr),
        }
    }
}
