/**
* Constants built into the language
*
* Contains text, mathematical and system constants,
* derived from the Rust standard library
*/
use std::f64::consts::{E, PI};
use ultraviolet_core::types::{
    backend::{EnvRef, UVRTValue},
    frontend::ast::Number,
};

/// Initialize built-in constants
///
/// Inserts constants into the provided environment
pub fn init_builtin_constants(env: EnvRef) {
    let mut borrowed_env = env.borrow_mut();

    borrowed_env.define_variable("endl", UVRTValue::String("\n".to_string()), true);
    borrowed_env.define_variable("tab", UVRTValue::String("\t".to_string()), true);
    borrowed_env.define_variable("space", UVRTValue::String(" ".to_string()), true);

    borrowed_env.define_variable("math.pi", UVRTValue::Number(Number::Float(PI)), true);
    borrowed_env.define_variable("math.exp", UVRTValue::Number(Number::Float(E)), true);

    borrowed_env.define_variable(
        "os.name",
        UVRTValue::String(std::env::consts::OS.to_string()),
        true,
    );
    borrowed_env.define_variable(
        "os.arch",
        UVRTValue::String(std::env::consts::ARCH.to_string()),
        true,
    );
    borrowed_env.define_variable(
        "os.family",
        UVRTValue::String(std::env::consts::FAMILY.to_string()),
        true,
    );

    borrowed_env.define_variable(
        "dll.prefix",
        UVRTValue::String(std::env::consts::DLL_PREFIX.to_string()),
        true,
    );
    borrowed_env.define_variable(
        "dll.suffix",
        UVRTValue::String(std::env::consts::DLL_SUFFIX.to_string()),
        true,
    );
}
