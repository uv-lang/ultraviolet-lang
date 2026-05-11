use crate::{
    errors::SpannedError,
    types::{
        EnvRef, Environment,
        frontend::ast::{ASTBlockType, Number, UVValue},
    },
};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
pub mod uvvalue_ops;

#[derive(Clone)]
pub struct RTFunction {
    pub args_names_order: Vec<String>,
    pub body: Rc<Vec<ASTBlockType>>,

    // FIXME:! The function should take a snapshot of the environment, not a link to it
    pub lexical_env: Weak<RefCell<Environment<RTVariable>>>,
}

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
    pub fn new_from(f: BuiltinFunctionSignature) -> Self {
        Self { f }
    }
}

#[derive(Clone)]
pub enum UVRTValue {
    Number(Number),
    String(String),
    Boolean(bool),
    Null,
    Void,

    Function(RTFunction),
    BuiltInFunction(BuiltInFunction),
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
