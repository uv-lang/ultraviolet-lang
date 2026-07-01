use crate::{
    errors::CommonError,
    types::{backend::UVRTValue, ffi::FFIData, frontend::types::UVType},
};
use libffi::middle::{Arg, Type};
use std::{ffi::c_void};
pub mod implementations;

pub trait ToTypeFFI {
    /// Convert UV Type to a Middle FFI type
    fn to_ffi_type(&self) -> Option<Type>;
}

pub trait AsArg {
    /// Convert value to a pointer to void
    fn as_arg(&self) -> Arg<'_>;
}

pub trait ToFFIData {
    /// Convert value to a owned struct
    fn to_ffi_data(&self) -> Result<FFIData<'_>, CommonError>;
}

pub trait FromFFI {
    /// Converts ffi data to a Ultraviolet val
    fn to_uv_value(&self, exp: UVType) -> Result<UVRTValue, CommonError>;
}

pub trait EnumPayloadPtr {
    /// Returns a pointer to data.
    ///
    /// # Safety
    /// The caller must ensure that enum is located
    /// in the appropriate version.
    unsafe fn payload_ptr(ptr: *mut Self) -> *mut c_void;
}
