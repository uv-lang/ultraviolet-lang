use std::ops::{Add, Div, Mul, Rem, Sub};

use crate::types::{backend::UVRTValue, frontend::ast::Number};

// FIXME: FUCK TYPECAST TO FLOAT!!!

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

impl<'b> Add<&'b UVRTValue> for &UVRTValue {
    type Output = UVRTValue;

    fn add(self, rhs: &'b UVRTValue) -> Self::Output {
        match (self, rhs) {
            (UVRTValue::Number(lhs), UVRTValue::Number(rhs)) => {
                UVRTValue::Number(Number::Float(lhs.to_f64().add(rhs.to_f64())))
            },
            _ => unreachable!("Typechecker bug"),
        }
    }
}

impl<'b> Sub<&'b UVRTValue> for &UVRTValue {
    type Output = UVRTValue;

    fn sub(self, rhs: &'b UVRTValue) -> Self::Output {
        match (self, rhs) {
            (UVRTValue::Number(lhs), UVRTValue::Number(rhs)) => {
                UVRTValue::Number(Number::Float(lhs.to_f64().sub(rhs.to_f64())))
            },
            _ => unreachable!("Typechecker bug"),
        }
    }
}

impl<'b> Mul<&'b UVRTValue> for &UVRTValue {
    type Output = UVRTValue;

    fn mul(self, rhs: &'b UVRTValue) -> Self::Output {
        match (self, rhs) {
            (UVRTValue::Number(lhs), UVRTValue::Number(rhs)) => {
                UVRTValue::Number(Number::Float(lhs.to_f64().mul(rhs.to_f64())))
            },
            _ => unreachable!("Typechecker bug"),
        }
    }
}

impl<'b> Div<&'b UVRTValue> for &UVRTValue {
    type Output = UVRTValue;

    fn div(self, rhs: &'b UVRTValue) -> Self::Output {
        match (self, rhs) {
            (UVRTValue::Number(lhs), UVRTValue::Number(rhs)) => {
                UVRTValue::Number(Number::Float(lhs.to_f64().div(rhs.to_f64())))
            },
            _ => unreachable!("Typechecker bug"),
        }
    }
}

impl<'b> Rem<&'b UVRTValue> for &UVRTValue {
    type Output = UVRTValue;

    fn rem(self, rhs: &'b UVRTValue) -> Self::Output {
        match (self, rhs) {
            (UVRTValue::Number(lhs), UVRTValue::Number(rhs)) => {
                UVRTValue::Number(Number::Float(lhs.to_f64().rem(rhs.to_f64())))
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

impl PartialEq for UVRTValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (UVRTValue::Number(a), UVRTValue::Number(b)) => a == b,

            (UVRTValue::String(a), UVRTValue::String(b)) => a == b,
            (UVRTValue::Boolean(a), UVRTValue::Boolean(b)) => a == b,

            (UVRTValue::Null, UVRTValue::Null) => true,
            (UVRTValue::Void, UVRTValue::Void) => true,

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

impl PartialOrd for UVRTValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (UVRTValue::Number(a), UVRTValue::Number(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}
