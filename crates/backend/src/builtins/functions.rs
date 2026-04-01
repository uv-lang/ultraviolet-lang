use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    io::{self, Write},
};
use ultraviolet_core::{
    errors::SpannedError,
    types::{
        backend::{ControlFlow, EnvRef},
        frontend::ast::{FunctionCall, UVValue},
    },
};

use crate::eval::eval;

type BuiltinFunctionSignature = fn(args: &[UVValue], env: EnvRef) -> Result<ControlFlow, SpannedError>;

lazy_static! {
    static ref BUILTIN_FUNCTIONS: HashMap<&'static str, BuiltinFunctionSignature> = {
        let mut m = HashMap::new();
        m.insert("print", print as BuiltinFunctionSignature);
        m.insert("println", println as BuiltinFunctionSignature);
        m.insert("read", read as BuiltinFunctionSignature);
        m
    };
}

/// Check if provided function name is built-in function
pub fn is_builtin_function(name: &str) -> bool {
    BUILTIN_FUNCTIONS.contains_key(name)
}

/// Execute builtin function by signature
pub fn execute_builtin_function(fc: &FunctionCall, env: EnvRef) -> Result<ControlFlow, SpannedError> {
    match BUILTIN_FUNCTIONS.get(fc.name.as_str()) {
        Some(f) => {
            let mut av = Vec::new();

            for arg in &fc.args {
                match eval(&arg.value, env.clone())? {
                    // FIXME: Value should be passed by reference, not value!
                    ControlFlow::Simple(v) => av.push(v),
                    other => return Ok(other),
                }
            }

            f(&av, env)
        },
        None => unreachable!(),
    }
}

/// Built-in `print` function
fn print(args: &[UVValue], _env: EnvRef) -> Result<ControlFlow, SpannedError> {
    for arg in args {
        print!("{}", arg);
    }

    Ok(ControlFlow::Simple(UVValue::Void))
}

/// Built-in `println` function
fn println(args: &[UVValue], _env: EnvRef) -> Result<ControlFlow, SpannedError> {
    for arg in args {
        println!("{}", arg);
    }

    Ok(ControlFlow::Simple(UVValue::Void))
}

/// Built-in function for reading from stdin
///
/// Returns String on success and Null on failure
fn read(args: &[UVValue], _env: EnvRef) -> Result<ControlFlow, SpannedError> {
    // Print an initial input prompt if provided
    if let Some(arg) = args.first() {
        print!("{}", arg);
        let _ = io::stdout().flush();
    }

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => Ok(ControlFlow::Simple(UVValue::String(input.trim_end().to_owned()))),
        Err(_) => Ok(ControlFlow::Simple(UVValue::Null)),
    }
}
