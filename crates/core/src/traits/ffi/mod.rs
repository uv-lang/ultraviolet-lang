use crate::types::{backend::UVRTValue, ffi::FFIData, frontend::types::UVType};
use anyhow::Result;
use libffi::middle::{Arg, Type};

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
    fn to_ffi_data(&self) -> Result<FFIData<'_>>;
}

pub trait FromFFI {
    /// Converts ffi data to a Ultraviolet val
    fn from_ffi(&self, exp: UVType) -> Result<UVRTValue>;
}
