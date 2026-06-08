use colored::Colorize;
use std::ops::Deref;

use crate::{
    traits::frontend::ast::{IsAssignable, StringToUVNumberType, StringToUVType},
    types::frontend::number::UVNumberType,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UVFunctionType {
    pub args: Vec<UVType>,
    pub returns: UVType,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UVBuiltinFunctionArguments {
    Any,
    Args(Vec<UVType>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UVBuiltinFunctionType {
    pub args: UVBuiltinFunctionArguments,
    pub returns: UVType,
}

// ------------------------------------------------------------------------

/// Ultraviolet primitive types
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UVType {
    Number(UVNumberType),
    String,
    Boolean,
    Null,
    Void,
    Function(Box<UVFunctionType>),
    BuiltInFunction(Box<UVBuiltinFunctionType>),

    Any,

    Union(Vec<UVType>),
    Optional(Box<UVType>),
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
            UVType::Union(u) => {
                write!(
                    f,
                    "<union>{}</union>",
                    u.iter()
                        .map(|i| i.to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            },
            UVType::Optional(t) => {
                write!(f, "<optional>{}</optional>", t.to_string().green().bold())
            },
        }
    }
}

impl UVType {
    /// Create new union type
    pub fn new_union(types: Vec<UVType>) -> UVType {
        let mut flat = Vec::new();

        for t in types {
            t.flatten_into(&mut flat);
        }

        flat.sort();
        flat.dedup();

        if flat.len() == 1 {
            flat.into_iter().next().unwrap()
        } else {
            UVType::Union(flat)
        }
    }

    /// Flat Union type to provided output vector
    pub fn flatten_into(&self, out: &mut Vec<Self>) {
        match self {
            Self::Union(types) => {
                types.iter().for_each(|t| t.flatten_into(out));
            },
            t => out.push(t.clone()),
        }
    }

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
            (_, UVType::Union(types)) => types.iter().all(|t| self.is_assignable_from(t)),
            (UVType::Union(types), _) => types.iter().any(|t| t.is_assignable_from(other)),

            (UVType::Optional(lv), rv) => lv.deref() == rv,

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
