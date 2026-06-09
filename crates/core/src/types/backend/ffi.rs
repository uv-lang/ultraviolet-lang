use crate::types::frontend::{Spanned, types::UVType};
use libffi::{low::CodePtr, middle::Cif};
use libloading::{Library, Symbol};
use std::sync::Arc;

#[derive(Clone)]
pub struct FFIFunction {
    pub _lib: Arc<Library>,

    pub func_symbol: Symbol<'static, unsafe extern "C" fn()>,
    pub func_ptr: CodePtr,
    pub cif: Cif,

    pub returns: Option<Spanned<UVType>>,
}
