use crate::types::{
    EnvRef,
    frontend::{
        ast::{UVNumberType, UVType},
        typechecker::UVTypeVariable,
    },
};

pub fn init_builtin_types_constants(env: EnvRef<UVTypeVariable>) {
    let mut borrowed_env = env.borrow_mut();

    borrowed_env.define_variable("endl", UVTypeVariable::new_from(UVType::String, true));
    borrowed_env.define_variable("tab", UVTypeVariable::new_from(UVType::String, true));
    borrowed_env.define_variable("space", UVTypeVariable::new_from(UVType::String, true));

    borrowed_env.define_variable(
        "math.pi",
        UVTypeVariable::new_from(UVType::Number(UVNumberType::Float), true),
    );
    borrowed_env.define_variable(
        "math.exp",
        UVTypeVariable::new_from(UVType::Number(UVNumberType::Float), true),
    );

    borrowed_env.define_variable("os.name", UVTypeVariable::new_from(UVType::String, true));
    borrowed_env.define_variable("os.arch", UVTypeVariable::new_from(UVType::String, true));
    borrowed_env.define_variable("os.family", UVTypeVariable::new_from(UVType::String, true));

    borrowed_env.define_variable("dll.prefix", UVTypeVariable::new_from(UVType::String, true));
    borrowed_env.define_variable("dll.suffix", UVTypeVariable::new_from(UVType::String, true));
}
