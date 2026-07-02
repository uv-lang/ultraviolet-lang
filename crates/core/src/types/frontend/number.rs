use crate::traits::ffi::EnumPayloadPtr;
use crate::types::frontend::CommonError;
use crate::{
    traits::{
        backend::TypeOf,
        ffi::{AsArg, ToTypeFFI},
        frontend::ast::{GetType, StringToUVNumberType},
    },
    types::frontend::types::UVType,
};
use core::ffi::c_void;
use libffi::middle::{Arg, Type};
use num_traits::NumCast;

/// Variants of number
#[macro_export]
macro_rules! number_variants {
    ($m:ident) => {
        $m!(
            I8(i8, i8),
            I16(i16, i16),
            I32(i32, i32),
            I64(i64, i64),
            U8(u8, u8),
            U16(u16, u16),
            U32(u32, u32),
            U64(u64, u64),
            F32(f32, f32),
            F64(f64, f64),
        );
    };

    (
        $m:ident,
        $($args:ident),* $(,)?
    ) => {
        $m!(
            $($args),*,
            I8(i8, i8),
            I16(i16, i16),
            I32(i32, i32),
            I64(i64, i64),
            U8(u8, u8),
            U16(u16, u16),
            U32(u32, u32),
            U64(u64, u64),
            F32(f32, f32),
            F64(f64, f64),
        );
    };
}

macro_rules! define_number {
    ($($variant:ident($ty:ty, $ffi:ident)),* $(,)?) => {
        /// Number-like value
        #[derive(Debug, Clone)]
        pub enum Number {
            $(
                $variant($ty),
            )*
        }

        impl std::fmt::Display for Number {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Number::$variant(n) => write!(f, "{n}"),)*
                }
            }
        }

        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
        /// Ultraviolet number types
        ///
        /// Must be ordered by type width
        pub enum UVNumberType {
            $(
                $variant,
            )*
            AnyNumber
        }

        impl std::fmt::Display for UVNumberType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        UVNumberType::$variant=> write!(f, "<{} />", stringify!($ty)),
                    )*
                    UVNumberType::AnyNumber =>  write!(f, "any number")
                }
            }
        }


        impl GetType for Number {
            fn get_type(&self) -> UVType {
                match self {
                    $(
                        Self::$variant(_) => UVType::Number(UVNumberType::$variant),
                    )*
                }
            }
        }

        impl StringToUVNumberType for str {
            fn to_uv_number_type(&self) -> Option<UVNumberType> {
                match self {
                    $(
                        stringify!($ty) => Some(UVNumberType::$variant),
                    )*
                    _ => None
                }
            }
        }

        impl Number {
            /// Create number from number type and value
            pub fn auto<T: NumCast>(v: T, t: UVType) -> Result<Self, CommonError> {
                    match t {
                    UVType::Number(t) => Ok(
                        match t {
                            $(
                                UVNumberType::$variant =>
                                    Self::$variant(
                                        NumCast::from(v)
                                            .ok_or(CommonError::new("Cannot create number with non-number type"))?
                                    ),
                            )*
                            UVNumberType::AnyNumber => unreachable!()
                        }
                    ),
                    _ => Err(CommonError::new("Cannot create number with non-number type")),
                }

            }
        }

        impl TypeOf for Number {
            fn typeof_str(&self) -> String {
                match self {
                    $(Self::$variant(_) => stringify!($ty).to_owned(),)*
                }
            }
        }

        impl ToTypeFFI for UVNumberType {
            fn to_ffi_type(&self) -> Option<Type> {
                Some(
                    match self {
                        $(Self::$variant => Type::$ffi(),)*
                        UVNumberType::AnyNumber => unreachable!()
                    }
                )
             }
        }

        impl AsArg for Number {
            fn as_arg(&'_ self) -> Arg<'_> {
                match self {
                    $(Self::$variant(v) => Arg::new(v),)*
                }
            }
        }


        impl EnumPayloadPtr for Number {
            #[allow(unsafe_op_in_unsafe_fn)]
            unsafe fn payload_ptr(ptr: *mut Self) -> *mut c_void {
                match &mut *ptr {
                    $(Self::$variant(v) => v as *const _ as *mut c_void,)*
                }
            }
        }
    };
}

number_variants!(define_number);

impl UVNumberType {
    /// Checks if all provided types is eq
    pub fn all_eq(vec: &[&Self]) -> bool {
        let mut i = vec.iter();
        let f = i.next().unwrap();

        for el in i {
            if f != el {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use crate::types::frontend::number::UVNumberType;

    #[test]
    fn display_type() {
        assert_eq!(format!("{}", UVNumberType::F64), "<f64 />");
    }
}
