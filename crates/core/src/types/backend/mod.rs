use crate::{
    errors::SpannedError,
    types::frontend::ast::{ASTBlockType, Number, UVValue},
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
pub mod uvvalue_ops;

pub type EnvRef = Rc<RefCell<Environment>>;

/// Scope-based environment
#[derive(Default, Debug)]
pub struct Environment {
    pub symbols: HashMap<String, Rc<RefCell<RTVariable>>>,
    pub parent: Option<EnvRef>,
}

impl Environment {
    /// Create new children environment from parent
    pub fn new_child(parent: EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Self {
            symbols: HashMap::new(),
            parent: Some(parent),
        }))
    }

    /// Find symbol by name
    pub fn find_var(&self, name: impl Into<String>) -> Option<Rc<RefCell<RTVariable>>> {
        let n = name.into();
        if let Some(sym) = self.symbols.get(&n) {
            return Some(sym.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().find_var(&n);
        }

        None
    }

    /// Define variable in current scope
    pub fn define_variable(&mut self, name: impl Into<String>, value: UVRTValue, constant: bool) {
        self.symbols.insert(
            name.into(),
            Rc::new(RefCell::new(RTVariable::new_from(value, constant))),
        );
    }

    /// Remove symbol from CURRENT scope
    pub fn remove_symbol(&mut self, name: impl Into<String>) -> bool {
        self.symbols.remove(&name.into()).is_some()
    }
}

#[derive(Debug, Clone)]
pub struct RTFunction {
    pub args_names_order: Vec<String>,
    pub body: Rc<Vec<ASTBlockType>>,
    pub lexical_env: EnvRef,
}

pub type BuiltinFunctionSignature =
    fn(args: &[UVRTValue], env: EnvRef) -> Result<ControlFlow, SpannedError>;

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

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug)]
pub enum ControlFlow {
    Simple(UVRTValue),

    /// Return propagates upstream
    Return(UVRTValue),

    Break,
    Continue,
}
