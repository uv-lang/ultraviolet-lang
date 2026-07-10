use crate::{
    traits::EnvironmentTrait,
    types::{
        EnvRef, Environment,
        frontend::{number::UVNumberType, typechecker::UVTypeVariable, types::UVType},
    },
};

pub fn init_builtin_types_constants(env: EnvRef<UVTypeVariable>) {
    let mut borrowed_env = env.borrow_mut();

    borrowed_env.define_variable("endl", UVTypeVariable::new_from(UVType::String, true));
    borrowed_env.define_variable("tab", UVTypeVariable::new_from(UVType::String, true));
    borrowed_env.define_variable("space", UVTypeVariable::new_from(UVType::String, true));

    borrowed_env.define_variable(
        "math.pi",
        UVTypeVariable::new_from(UVType::Number(UVNumberType::F64), true),
    );
    borrowed_env.define_variable(
        "math.exp",
        UVTypeVariable::new_from(UVType::Number(UVNumberType::F64), true),
    );

    borrowed_env.define_variable("os.name", UVTypeVariable::new_from(UVType::String, true));
    borrowed_env.define_variable("os.arch", UVTypeVariable::new_from(UVType::String, true));
    borrowed_env.define_variable("os.family", UVTypeVariable::new_from(UVType::String, true));

    let dll_env = Environment::<UVTypeVariable>::new();
    {
        let mut borrowed_dll_env = dll_env.borrow_mut();
        borrowed_dll_env.define_variable("prefix", UVTypeVariable::new_from(UVType::String, true));
        borrowed_dll_env.define_variable("suffix", UVTypeVariable::new_from(UVType::String, true));
    }

    borrowed_env.define_variable("dll", UVTypeVariable::new_environmental(dll_env));
}
