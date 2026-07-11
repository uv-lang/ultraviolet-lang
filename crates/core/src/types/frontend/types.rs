use colored::Colorize;
use std::{cell::RefCell, ops::Deref, rc::Weak};

use crate::{
    traits::frontend::ast::{IsAssignable, StringToUVNumberType, StringToUVType},
    types::{
        EnvRef, Environment,
        frontend::{number::UVNumberType, typechecker::UVTypeVariable},
    },
};

#[derive(Clone, PartialEq, Eq)]
/// User environment function type
pub struct UVFunctionType {
    pub args: Vec<UVType>,
    pub returns: UVType,
}

// ---------------------- Builtin functions -------------------------------

#[derive(Clone, PartialEq, Eq)]
pub enum UVBuiltinFunctionArguments {
    /// Arguments of any type and quantity
    Any,
    /// Fixed number and type of arguments
    Args(Vec<UVType>),
    /// ALL arguments of the given type and number
    AllOf(UVType),
}

#[derive(Clone, PartialEq, Eq)]
pub struct UVBuiltinFunctionType {
    pub args: UVBuiltinFunctionArguments,
    pub returns: UVType,
}

// ------------------------------------------------------------------------

/// Ultraviolet types
#[derive(Clone, PartialEq, Eq)]
pub enum UVType {
    Number(UVNumberType),
    String,
    Boolean,
    Null,
    Void,
    Function(Box<UVFunctionType>),
    BuiltInFunction(Box<UVBuiltinFunctionType>),

    /// Unreachable from user env
    Any,

    Reference(Box<ReferenceType>),

    Optional(Box<UVType>),

    Module(EnvRef<UVTypeVariable>),
    Namespace(EnvRef<UVTypeVariable>),
}

#[derive(Clone)]
pub struct ReferenceType {
    pub t: UVType,
    pub reference: Option<Weak<RefCell<UVTypeVariable>>>,
}

impl ReferenceType {
    /// Create new reference with empty reference field
    ///
    /// Used e.g. `<int ref />`
    pub fn new(t: UVType) -> Self {
        Self { t, reference: None }
    }

    /// Create new reference with non-empty reference field
    ///
    /// Used for real references to a variables
    pub fn new_referenced(t: UVType, reference: Weak<RefCell<UVTypeVariable>>) -> Self {
        Self {
            t,
            reference: Some(reference),
        }
    }
}

impl std::fmt::Display for UVType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UVType::Number(n) => n.fmt(f),
            UVType::String => write!(f, "<str />"),
            UVType::Boolean => write!(f, "<bool />"),
            UVType::Null => write!(f, "<null />"),
            UVType::Void => write!(f, "<void />"),
            UVType::Function(func) => {
                write!(
                    f,
                    "({}) -> {}",
                    func.args
                        .iter()
                        .map(|i| i.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    func.returns
                )
            },
            UVType::BuiltInFunction(_) => write!(f, "<built-in function>"),
            UVType::Any => write!(f, "<any />"),
            UVType::Optional(t) => {
                write!(f, "<optional>{}</optional>", t.to_string().green().bold())
            },
            UVType::Reference(r) => write!(f, "reference to {}", r.t),
            UVType::Module(_) => write!(f, "<module>"),
            UVType::Namespace(_) => writeln!(f, "<namespace>"),
        }
    }
}

impl UVType {
    /// Flatten optional type
    pub fn flat_optional(self) -> UVType {
        match self {
            UVType::Optional(t) => t.flat_optional(),
            t => t,
        }
    }

    /// Checks if all provided types is eq
    pub fn all_eq(vec: &[Self]) -> bool {
        let mut i = vec.iter();
        let f = i.next().unwrap();

        for el in i {
            if !f.is_assignable_from(el) {
                return false;
            }
        }

        true
    }
}

impl IsAssignable for UVType {
    fn is_assignable_from(&self, other: &UVType) -> bool {
        if self == other {
            return true;
        }

        match (self, other) {
            (UVType::Optional(lv), rv) => lv.deref() == rv,

            (UVType::Reference(lr), UVType::Reference(rr)) => lr.t.is_assignable_from(&rr.t),
            (UVType::Number(UVNumberType::AnyNumber), UVType::Number(_)) => true,

            (UVType::Any, _) => true,
            (_, UVType::Any) => false,

            _ => false,
        }
    }
}

// -------------------- String-Type conversion --------------

impl StringToUVType for str {
    fn to_uvtype(&self) -> Option<UVType> {
        if let Some(n) = self.to_uv_number_type() {
            return Some(UVType::Number(n));
        }

        match self {
            "str" => Some(UVType::String),
            "bool" => Some(UVType::Boolean),
            "null" => Some(UVType::Null),
            "void" => Some(UVType::Void),
            _ => None,
        }
    }
}

impl PartialEq for ReferenceType {
    fn eq(&self, other: &Self) -> bool {
        self.t == other.t
    }
}

impl Eq for ReferenceType {}

impl PartialEq for Environment<UVTypeVariable> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl Eq for Environment<UVTypeVariable> {}
