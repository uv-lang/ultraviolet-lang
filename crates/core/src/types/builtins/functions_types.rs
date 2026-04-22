use crate::types::{
    EnvRef,
    frontend::{
        ast::{UVBuiltinFunctionArguments, UVBuiltinFunctionType, UVType},
        typechecker::UVTypeVariable,
    },
};

/// Initializes builtin function types into the passed environment
pub fn init_builtins_types(env: EnvRef<UVTypeVariable>) {
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
                args: UVBuiltinFunctionArguments::Args(vec![UVType::Any]),
                returns: UVType::String,
            })),
            true,
        ),
    );
}
