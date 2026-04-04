use std::ops::{Add, Div, Mul, Rem, Sub};

use crate::types::frontend::ast::{Number, UVValue};

impl Number {
    // FIXME: Is this correct?
    /// Cast number to f64
    fn to_f64(&self) -> f64 {
        match self {
            Number::Int(v) => *v as f64,
            Number::Float(v) => *v,
        }
    }
}

impl<'b> Add<&'b UVValue> for &UVValue {
    type Output = UVValue;

    fn add(self, rhs: &'b UVValue) -> Self::Output {
        match (self, rhs) {
            (UVValue::Number(lhs), UVValue::Number(rhs)) => {
                UVValue::Number(Number::Float(lhs.to_f64().add(rhs.to_f64())))
            },
            _ => unreachable!("Typechecker bug"),
        }
    }
}

impl<'b> Sub<&'b UVValue> for &UVValue {
    type Output = UVValue;

    fn sub(self, rhs: &'b UVValue) -> Self::Output {
        match (self, rhs) {
            (UVValue::Number(lhs), UVValue::Number(rhs)) => {
                UVValue::Number(Number::Float(lhs.to_f64().sub(rhs.to_f64())))
            },
            _ => unreachable!("Typechecker bug"),
        }
    }
}

impl<'b> Mul<&'b UVValue> for &UVValue {
    type Output = UVValue;

    fn mul(self, rhs: &'b UVValue) -> Self::Output {
        match (self, rhs) {
            (UVValue::Number(lhs), UVValue::Number(rhs)) => {
                UVValue::Number(Number::Float(lhs.to_f64().mul(rhs.to_f64())))
            },
            _ => unreachable!("Typechecker bug"),
        }
    }
}

impl<'b> Div<&'b UVValue> for &UVValue {
    type Output = UVValue;

    fn div(self, rhs: &'b UVValue) -> Self::Output {
        match (self, rhs) {
            (UVValue::Number(lhs), UVValue::Number(rhs)) => {
                UVValue::Number(Number::Float(lhs.to_f64().div(rhs.to_f64())))
            },
            _ => unreachable!("Typechecker bug"),
        }
    }
}

impl<'b> Rem<&'b UVValue> for &UVValue {
    type Output = UVValue;

    fn rem(self, rhs: &'b UVValue) -> Self::Output {
        match (self, rhs) {
            (UVValue::Number(lhs), UVValue::Number(rhs)) => {
                UVValue::Number(Number::Float(lhs.to_f64().rem(rhs.to_f64())))
            },
            _ => unreachable!("Typechecker bug"),
        }
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Number::Float(a), Number::Float(b)) => a == b,
            (Number::Int(a), Number::Int(b)) => a == b,

            (Number::Int(a), Number::Float(b)) => (*a as f64) == *b,
            (Number::Float(a), Number::Int(b)) => *a == (*b as f64),
        }
    }
}

impl PartialEq for UVValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (UVValue::Number(a), UVValue::Number(b)) => a == b,

            (UVValue::String(a), UVValue::String(b)) => a == b,
            (UVValue::Boolean(a), UVValue::Boolean(b)) => a == b,

            (UVValue::Null, UVValue::Null) => true,
            (UVValue::Void, UVValue::Void) => true,

            _ => false,
        }
    }
}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Number::Int(a), Number::Int(b)) => a.partial_cmp(b),
            (Number::Float(a), Number::Float(b)) => a.partial_cmp(b),

            (Number::Int(a), Number::Float(b)) => (*a as f64).partial_cmp(b),
            (Number::Float(a), Number::Int(b)) => a.partial_cmp(&(*b as f64)),
        }
    }
}

impl PartialOrd for UVValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (UVValue::Number(a), UVValue::Number(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}
