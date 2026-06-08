use crate::types::{
    EnvRef,
    frontend::{
        number::UVNumberType,
        typechecker::UVTypeVariable,
        types::{UVBuiltinFunctionArguments, UVBuiltinFunctionType, UVType},
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
        "sin",
        UVTypeVariable::new_from(
            UVType::BuiltInFunction(Box::new(UVBuiltinFunctionType {
                args: UVBuiltinFunctionArguments::Args(vec![UVType::Number(UVNumberType::F64)]),
                returns: UVType::Number(UVNumberType::F64),
            })),
            true,
        ),
    );
}
