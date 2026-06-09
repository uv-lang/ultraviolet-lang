use anyhow::Result;
use libffi::middle::Type;
use std::ffi::c_void;

use crate::types::ffi::FFIData;

pub trait ToTypeFFI {
    /// Convert UV Type to a Middle FFI type
    fn to_ffi_type(&self) -> Option<Type>;
}

pub trait AsVoidPtr {
    /// Convert value to a pointer to void
    fn as_void_ptr(&self) -> Result<*const c_void>;
}

pub trait ToFFIData {
    /// Convert value to a owned struct
    fn to_ffi_data(&self) -> Result<FFIData>;
}
