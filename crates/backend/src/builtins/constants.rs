/**
* Constants built into the language
*
* Contains text, mathematical and system constants,
* derived from the Rust standard library
*/
use std::f64::consts::{E, PI};
use ultraviolet_core::{
    traits::EnvironmentTrait,
    types::{
        EnvRef, Environment,
        backend::{RTVariable, UVRTValue},
        frontend::number::Number,
    },
};

/// Initialize built-in constants
///
/// Inserts constants into the provided environment
pub fn init_builtin_constants(env: EnvRef<RTVariable>) {
    let mut borrowed_env = env.borrow_mut();

    borrowed_env.define_variable(
        "endl",
        RTVariable::new_from(UVRTValue::String("\n".to_string()), true),
    );
    borrowed_env.define_variable(
        "tab",
        RTVariable::new_from(UVRTValue::String("\t".to_string()), true),
    );
    borrowed_env.define_variable(
        "space",
        RTVariable::new_from(UVRTValue::String(" ".to_string()), true),
    );

    borrowed_env.define_variable(
        "math.pi",
        RTVariable::new_from(UVRTValue::Number(Number::F64(PI)), true),
    );
    borrowed_env.define_variable(
        "math.exp",
        RTVariable::new_from(UVRTValue::Number(Number::F64(E)), true),
    );

    borrowed_env.define_variable(
        "os.name",
        RTVariable::new_from(UVRTValue::String(std::env::consts::OS.to_string()), true),
    );
    borrowed_env.define_variable(
        "os.arch",
        RTVariable::new_from(UVRTValue::String(std::env::consts::ARCH.to_string()), true),
    );
    borrowed_env.define_variable(
        "os.family",
        RTVariable::new_from(
            UVRTValue::String(std::env::consts::FAMILY.to_string()),
            true,
        ),
    );

    let dll_env = Environment::<RTVariable>::new();
    {
        let mut borrowed_dll_env = dll_env.borrow_mut();
        borrowed_dll_env.define_variable(
            "prefix",
            RTVariable::new_from(
                UVRTValue::String(std::env::consts::DLL_PREFIX.to_string()),
                true,
            ),
        );
        borrowed_dll_env.define_variable(
            "suffix",
            RTVariable::new_from(
                UVRTValue::String(std::env::consts::DLL_SUFFIX.to_string()),
                true,
            ),
        );
    }

    borrowed_env.define_variable(
        "dll",
        RTVariable::new_from(UVRTValue::Module(dll_env), true),
    );
}
