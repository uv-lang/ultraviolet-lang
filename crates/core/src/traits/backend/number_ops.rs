use crate::number_variants;
use crate::types::frontend::number::Number;
use std::ops::{Add, Div, Mul, Rem, Sub};

macro_rules! impl_number_op {
    (
        $trait_name:ident,
        $method:ident,
        $($variant:ident($ty:ty,$a:ident)),* $(,)?
    ) => {
        impl<'a> $trait_name<&'a Number> for &'a Number {
            type Output = Number;

            fn $method(self, rhs: &'a Number) -> Self::Output {

                match (self, rhs) {
                    $((Number::$variant(value), Number::$variant(rhs)) => Number::$variant(value.$method(rhs)),)*
                    _ => unreachable!(
                        "type mismatch for Number"
                    ),
                }
            }
        }
    };
}

// Impl base math operations for number
number_variants!(impl_number_op, Add, add);
number_variants!(impl_number_op, Sub, sub);
number_variants!(impl_number_op, Mul, mul);
number_variants!(impl_number_op, Div, div);
number_variants!(impl_number_op, Rem, rem);

macro_rules! impl_number_cmp_op {
    (
        $trait_name:ident,
        $method:ident,
        $out_t:ident,
        $($variant:ident($ty:ty,$a:ident)),* $(,)?
    ) => {
        impl $trait_name for Number {
            fn $method(&self, rhs: &Number) -> $out_t {
                match (self, rhs) {
                    $((Number::$variant(value), Number::$variant(rhs)) => value.$method(rhs),)*
                    _ => unreachable!(
                        "type mismatch for Number"
                    ),
                }
            }
        }
    };
}

number_variants!(impl_number_cmp_op, PartialEq, eq, bool);

type OrdOutType = Option<std::cmp::Ordering>;
number_variants!(impl_number_cmp_op, PartialOrd, partial_cmp, OrdOutType);
