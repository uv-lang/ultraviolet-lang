use std::ffi::c_void;

use crate::{
    traits::ffi::EnumPayloadPtr,
    types::{backend::UVRTValue, frontend::number::Number},
};

impl EnumPayloadPtr for UVRTValue {
    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe fn payload_ptr(ptr: *mut Self) -> *mut c_void {
        match &mut *ptr {
            UVRTValue::Number(v) => Number::payload_ptr(v),
            UVRTValue::String(v) => v as *const _ as *mut c_void,
            UVRTValue::Boolean(v) => v as *const _ as *mut c_void,

            // FIXME: Is we allow nested references in ffi?
            UVRTValue::Reference(_) => todo!(),

            _ => unreachable!(),
        }
    }
}
