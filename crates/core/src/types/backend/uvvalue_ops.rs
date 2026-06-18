use crate::errors::CommonError;
use crate::traits::ffi::FromFFI;
use crate::types::frontend::{
    number::{Number, UVNumberType},
    types::UVType,
};
use crate::{number_variants, types::backend::UVRTValue};
use std::ffi::CStr;
use std::ops::{Add, Div, Mul, Rem, Sub};

macro_rules! impl_RTVal_op {
    (
        $($trait_name:ident($method:ident)), *$(,)?
    ) => {
        $(
            impl<'a> $trait_name<&'a UVRTValue> for &'a UVRTValue {
                type Output = UVRTValue;

                fn $method(self, rhs: &'a UVRTValue) -> Self::Output {
                    match (self, rhs) {
                        (Self::Output::Number(value), Self::Output::Number(rhs)) => {
                            Self::Output::Number(value.$method(rhs))
                        },
                        _ => unreachable!("type mismatch for Number"),
                    }
                }
            }
        )*
    };
}

// Impl base math operations for UVRTValue
impl_RTVal_op!(Add(add), Sub(sub), Mul(mul), Div(div), Rem(rem));

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

impl PartialOrd for UVRTValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (UVRTValue::Number(a), UVRTValue::Number(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

macro_rules! gen_ffi_to_num {
    ($($variant:ident($ty:ty, $ffi:ident)),* $(,)?) => {
        impl FromFFI for u64 {
            fn to_uv_value(&self, exp: UVType) -> Result<UVRTValue, CommonError> {
                unsafe {
                    Ok(match exp {
                        UVType::Number(t) => match t {
                            $(
                                UVNumberType::$variant => {
                                    let val = *( self as *const u64 as *const $ty );
                                    UVRTValue::Number(Number::$variant(val))
                            },)*
                            UVNumberType::AnyNumber => {
                                // This hand is unreachable, but for compatibility I'll leave this here
                                let val = *( self as *const u64 as *const f64 );
                                UVRTValue::Number(Number::F64(val))
                            }
                        },
                        UVType::String => {
                            let char_ptr = *( self as *const u64 as *const *const i8 );
                            if char_ptr.is_null() {
                                return Ok(UVRTValue::Null);
                            }

                            let c_str = CStr::from_ptr(char_ptr);
                            let rust_str = c_str.to_string_lossy().into_owned();
                            UVRTValue::String(rust_str)

                            // FIXME:? Should code free the string memory
                        },
                        UVType::Boolean => {
                            let val = *(self as *const u64 as *const u8);
                            UVRTValue::Boolean(val != 0)
                        },
                        UVType::Null => UVRTValue::Null,
                        UVType::Void => UVRTValue::Void,
                        _ => return Err(CommonError::new("Cannot convert this type")),
                    })
                }
            }
        }
    };
}

number_variants!(gen_ffi_to_num);
