use crate::types::frontend::{Spanned, types::UVType};
use libffi::{low::CodePtr, middle::Cif};
use libloading::{Library, Symbol};
use std::sync::Arc;

pub struct FFIFunction {
    pub _lib: Arc<Library>,

    pub func_symbol: Symbol<'static, unsafe extern "C" fn()>,
    pub func_ptr: CodePtr,
    pub cif: Cif,

    pub returns: Option<Spanned<UVType>>,
}

// FIXME:? Is this correct?
unsafe impl Send for FFIFunction {}
unsafe impl Sync for FFIFunction {}
