use crate::{
    errors::SpannedError,
    traits::{backend::TypeOf, frontend::ast::GetType},
    types::{
        EnvRef, Environment,
        backend::ffi::FFIFunction,
        frontend::{
            Spanned,
            ast::{ASTBlockType, UVValue},
            number::Number,
            types::UVType,
        },
    },
};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
pub mod ffi;
pub mod uvvalue_ops;

#[derive(Clone)]
pub struct RTFunction {
    pub args_names_order: Vec<String>,
    pub body: Rc<Vec<Spanned<ASTBlockType>>>,

    // FIXME:! The function should take a snapshot of the environment, not a link to it
    pub lexical_env: Weak<RefCell<Environment<RTVariable>>>,
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
        }
    }
}

/// Runtime variable struct
#[derive(Clone)]
pub struct RTVariable {
    pub value: UVRTValue,
    pub constant: bool,
}

impl RTVariable {
    /// Create new variable from value
    pub fn new_from(val: UVRTValue, constant: bool) -> Self {
        Self {
            value: val,
            constant,
        }
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
        }
    }
}
