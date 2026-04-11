use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    io::{self, BufWriter, Write},
};
use ultraviolet_core::{
    errors::SpannedError,
    types::{
        backend::{ControlFlow, EnvRef},
        frontend::ast::{FunctionCall, UVValue},
    },
};

use crate::eval::eval;

type BuiltinFunctionSignature =
    fn(args: &[UVValue], env: EnvRef) -> Result<ControlFlow, SpannedError>;

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
pub fn execute_builtin_function(
    fc: &FunctionCall,
    env: EnvRef,
) -> Result<ControlFlow, SpannedError> {
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
///
/// Each argument is printed without delimiters or newlines.
///
/// Example:
/// ```xml
/// <call print>
///    <int>3<int>
///    <int>4<int>
/// </call>
/// ```
///
/// Will output `34`
fn print(args: &[UVValue], _env: EnvRef) -> Result<ControlFlow, SpannedError> {
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    for arg in args {
        let _ = write!(out, "{arg}");
    }

    let _ = out.flush();
    Ok(ControlFlow::Simple(UVValue::Void))
}

/// Built-in `println` function
///
/// A newline character is added after each argument is printed.
///
/// Example:
/// ```xml
/// <call println>
///    <int>3<int>
///    <int>4<int>
/// </call>
/// ```
///
/// Will output
/// ```
/// 3
/// 4
/// ```
fn println(args: &[UVValue], _env: EnvRef) -> Result<ControlFlow, SpannedError> {
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    for arg in args {
        let _ = writeln!(out, "{arg}");
    }

    let _ = out.flush();
    Ok(ControlFlow::Simple(UVValue::Void))
}

/// Built-in function for reading from stdin
///
/// Prints the value of the first argument (if passed) to stdout
/// as an input prompt and then waits for input from stdin
///
/// If the string is successfully received, returns UVValue::String,
/// If getting the string failed, returns UVValue::Null
fn read(args: &[UVValue], _env: EnvRef) -> Result<ControlFlow, SpannedError> {
    // Print an initial input prompt if provided
    if let Some(arg) = args.first() {
        print!("{arg}");
        let _ = io::stdout().flush();
    }

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => Ok(ControlFlow::Simple(UVValue::String(
            input.trim_end().to_owned(),
        ))),
        Err(_) => Ok(ControlFlow::Simple(UVValue::Null)),
    }
}
