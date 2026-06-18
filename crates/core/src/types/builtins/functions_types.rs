use crate::types::{
    EnvRef,
    frontend::{
        number::UVNumberType,
        typechecker::UVTypeVariable,
        types::{ReferenceType, UVBuiltinFunctionArguments, UVBuiltinFunctionType, UVType},
    },
};

/// Initializes builtin function types into the passed environment
pub fn init_builtin_types_functions(env: EnvRef<UVTypeVariable>) {
    let mut borrowed_env = env.borrow_mut();

    borrowed_env.define_variable(
        "println",
        UVTypeVariable::new_from(
            UVType::BuiltInFunction(Box::new(UVBuiltinFunctionType {
                args: UVBuiltinFunctionArguments::Any,
                returns: UVType::Void,
            })),
            true,
        ),
    );

    borrowed_env.define_variable(
        "print",
        UVTypeVariable::new_from(
            UVType::BuiltInFunction(Box::new(UVBuiltinFunctionType {
                args: UVBuiltinFunctionArguments::Any,
                returns: UVType::Void,
            })),
            true,
        ),
    );

    borrowed_env.define_variable(
        "read",
        UVTypeVariable::new_from(
            UVType::BuiltInFunction(Box::new(UVBuiltinFunctionType {
                args: UVBuiltinFunctionArguments::Args(vec![UVType::Optional(Box::new(
                    UVType::String,
                ))]),
                returns: UVType::String,
            })),
            true,
        ),
    );

    borrowed_env.define_variable(
        "typeof",
        UVTypeVariable::new_from(
            UVType::BuiltInFunction(Box::new(UVBuiltinFunctionType {
                args: UVBuiltinFunctionArguments::Args(vec![UVType::Any]),
                returns: UVType::String,
            })),
            true,
        ),
    );

    borrowed_env.define_variable(
        "concat",
        UVTypeVariable::new_from(
            UVType::BuiltInFunction(Box::new(UVBuiltinFunctionType {
                args: UVBuiltinFunctionArguments::AllOf(UVType::String),
                returns: UVType::String,
            })),
            true,
        ),
    );

    borrowed_env.define_variable(
        "inc",
        UVTypeVariable::new_from(
            UVType::BuiltInFunction(Box::new(UVBuiltinFunctionType {
                args: UVBuiltinFunctionArguments::Args(vec![UVType::Reference(Box::new(
                    ReferenceType::new(UVType::Number(UVNumberType::AnyNumber)),
                ))]),
                returns: UVType::String,
            })),
            true,
        ),
    );

    borrowed_env.define_variable(
        "dec",
        UVTypeVariable::new_from(
            UVType::BuiltInFunction(Box::new(UVBuiltinFunctionType {
                args: UVBuiltinFunctionArguments::Args(vec![UVType::Reference(Box::new(
                    ReferenceType::new(UVType::Number(UVNumberType::AnyNumber)),
                ))]),
                returns: UVType::String,
            })),
            true,
        ),
    );
}
