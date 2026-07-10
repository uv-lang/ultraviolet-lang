use crate::{
    errors::SpannedError,
    traits::{
        GetVariableContainedEnvironment, backend::TypeOf, ffi::EnumPayloadPtr,
        frontend::ast::GetType,
    },
    types::{
        EnvRef,
        backend::ffi::FFIFunction,
        frontend::{
            Spanned,
            ast::{ASTBlockType, SymbolName, UVValue},
            number::Number,
            types::UVType,
        },
    },
};
use std::{
    cell::RefCell,
    collections::HashMap,
    ffi::c_void,
    mem::offset_of,
    rc::{Rc, Weak},
};
pub mod ffi;
pub mod uvvalue_ops;

#[derive(Clone)]
pub struct RTFunction {
    pub args_names_order: Vec<String>,
    pub body: Rc<Vec<Spanned<ASTBlockType>>>,
    pub definition_name: Option<String>,
    pub moved_symbols: HashMap<SymbolName, Rc<RefCell<RTVariable>>>,
}

/// Call signature for built-in function
pub type BuiltinFunctionSignature =
    fn(args: &[UVRTValue], env: EnvRef<RTVariable>) -> Result<ControlFlow, SpannedError>;

#[derive(Debug, Clone)]
/// Function built into the interpreter
///
/// Contains a reference to a function
pub struct BuiltInFunction {
    pub f: BuiltinFunctionSignature,
}

impl BuiltInFunction {
    /// Create new built-in function from existing function
    pub fn new_from(f: BuiltinFunctionSignature) -> Self {
        Self { f }
    }
}

#[derive(Clone)]
/// Runtime value enum
pub enum UVRTValue {
    Number(Number),
    String(String),
    Boolean(bool),
    Null,
    Void,

    Function(RTFunction),
    BuiltInFunction(BuiltInFunction),
    FFIFunction(FFIFunction),

    Reference(Weak<RefCell<RTVariable>>),
}

impl std::fmt::Display for UVRTValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UVRTValue::Number(n) => n.fmt(f),
            UVRTValue::String(s) => write!(f, "{s}"),
            UVRTValue::Boolean(b) => write!(f, "{b}"),
            UVRTValue::Null => write!(f, "null"),
            UVRTValue::Void => write!(f, "void"),
            UVRTValue::Function(_) => write!(f, "<function>"),
            UVRTValue::BuiltInFunction(_) => write!(f, "<built-in function>"),
            UVRTValue::FFIFunction(_) => write!(f, "<ffi function>"),
            UVRTValue::Reference(r) => {
                let Some(val) = r.upgrade() else {
                    return write!(f, "NULL");
                };
                write!(f, "{}", val.borrow())
            },
        }
    }
}

impl UVRTValue {
    /** Converts frontend UVValue to UVRTValue */
    pub fn from_uvvalue(val: UVValue) -> Self {
        match val {
            UVValue::Number(number) => Self::Number(number),
            UVValue::String(s) => Self::String(s),
            UVValue::Boolean(b) => Self::Boolean(b),
            UVValue::Null => Self::Null,
            UVValue::Void => Self::Void,
            UVValue::Reference(_) => unreachable!(),
        }
    }
}

impl EnumPayloadPtr for UVRTValue {
    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe fn payload_ptr(ptr: *mut Self) -> *mut c_void {
        match &mut *ptr {
            UVRTValue::Number(v) => Number::payload_ptr(v),
            UVRTValue::String(v) => v as *const _ as *mut c_void,
            UVRTValue::Boolean(v) => v as *const _ as *mut c_void,

            UVRTValue::Reference(v) => {
                let Some(strong) = v.upgrade() else {
                    // # Safety
                    // This hand is unreachable due typecheck
                    unreachable!()
                };

                let base = strong.as_ptr() as *const c_void;
                let field = base.add(offset_of!(RTVariable, value));
                UVRTValue::payload_ptr(field as *mut UVRTValue)
            },

            _ => unreachable!(),
        }
    }
}

/// Runtime variable struct
#[derive(Clone)]
pub struct RTVariable {
    pub value: UVRTValue,
    pub constant: bool,

    pub contained_env: Option<EnvRef<RTVariable>>,
}

impl std::fmt::Display for RTVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl RTVariable {
    /// Create new variable from value
    pub fn new_from(val: UVRTValue, constant: bool) -> Self {
        Self {
            value: val,
            constant,
            contained_env: None,
        }
    }

    /// Create new contained env
    pub fn new_environmental(env: EnvRef<RTVariable>) -> Self {
        Self {
            value: UVRTValue::Void,
            constant: true,
            contained_env: Some(env),
        }
    }
}

impl GetVariableContainedEnvironment for RTVariable {
    type Out = RTVariable;
    fn get_variable_contained_env(&self) -> Option<EnvRef<Self::Out>> {
        self.contained_env.clone()
    }
}

/// Indicates, when block ended with return, break, etc...
pub enum ControlFlow {
    Simple(UVRTValue),

    /// Return propagates upstream
    Return(UVRTValue),

    Break,
    Continue,
}

impl GetType for UVRTValue {
    fn get_type(&self) -> UVType {
        match self {
            UVRTValue::Number(number) => number.get_type(),
            UVRTValue::String(_) => UVType::String,
            UVRTValue::Boolean(_) => UVType::Boolean,
            UVRTValue::Null => UVType::Null,
            UVRTValue::Void => UVType::Void,
            _ => unimplemented!(),
        }
    }
}

impl TypeOf for UVRTValue {
    fn typeof_str(&self) -> String {
        match self {
            UVRTValue::Number(n) => n.typeof_str(),
            UVRTValue::String(_) => String::from("string"),
            UVRTValue::Boolean(_) => String::from("boolean"),
            UVRTValue::Null => String::from("null"),
            UVRTValue::Void => String::from("void"),
            UVRTValue::Function(_) | UVRTValue::BuiltInFunction(_) | UVRTValue::FFIFunction(_) => {
                String::from("function")
            },
            UVRTValue::Reference(r) => {
                let Some(val) = r.upgrade() else {
                    return String::from("NULL");
                };
                val.borrow().value.typeof_str()
            },
        }
    }
}
