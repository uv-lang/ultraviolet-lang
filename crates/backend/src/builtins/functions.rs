use std::io::{self, Write};
use ultraviolet_core::{
    errors::SpannedError,
    traits::backend::TypeOf,
    types::{
        EnvRef,
        backend::{BuiltInFunction, ControlFlow, RTVariable, UVRTValue},
    },
};

/// Initialize built-in functions into environ
pub fn init_builtin_functions(env: EnvRef<RTVariable>) {
    let mut borrowed_env = env.borrow_mut();

    borrowed_env.define_variable(
        "print",
        RTVariable::new_from(
            UVRTValue::BuiltInFunction(BuiltInFunction::new_from(print)),
            true,
        ),
    );
    borrowed_env.define_variable(
        "println",
        RTVariable::new_from(
            UVRTValue::BuiltInFunction(BuiltInFunction::new_from(println)),
            true,
        ),
    );
    borrowed_env.define_variable(
        "read",
        RTVariable::new_from(
            UVRTValue::BuiltInFunction(BuiltInFunction::new_from(read)),
            true,
        ),
    );

    borrowed_env.define_variable(
        "typeof",
        RTVariable::new_from(
            UVRTValue::BuiltInFunction(BuiltInFunction::new_from(tof)),
            true,
        ),
    );

    borrowed_env.define_variable(
        "concat",
        RTVariable::new_from(
            UVRTValue::BuiltInFunction(BuiltInFunction::new_from(concat)),
            true,
        ),
    );
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
fn print(args: &[UVRTValue], _env: EnvRef<RTVariable>) -> Result<ControlFlow, SpannedError> {
    let mut out = io::stdout().lock();

    for arg in args {
        let _ = write!(out, "{arg}");
    }

    let _ = out.flush();
    Ok(ControlFlow::Simple(UVRTValue::Void))
}

/// Built-in `println` function
///
/// A newline character is added after last argument.
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
/// ```plain
/// 3 4
/// ```
fn println(args: &[UVRTValue], _env: EnvRef<RTVariable>) -> Result<ControlFlow, SpannedError> {
    let mut out = io::stdout().lock();

    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            let _ = write!(out, " ");
        }
        let _ = write!(out, "{arg}");
    }
    let _ = writeln!(out);

    let _ = out.flush();
    Ok(ControlFlow::Simple(UVRTValue::Void))
}

/// Built-in function for reading from stdin
///
/// Prints the value of the first argument (if passed) to stdout
/// as an input prompt and then waits for input from stdin
///
/// If the string is successfully received, returns UVValue::String,
/// If getting the string failed, returns UVValue::Null
fn read(args: &[UVRTValue], _env: EnvRef<RTVariable>) -> Result<ControlFlow, SpannedError> {
    // Print an initial input prompt if provided
    if let Some(arg) = args.first() {
        print!("{arg}");
        let _ = io::stdout().flush();
    }

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => Ok(ControlFlow::Simple(UVRTValue::String(
            input.trim_end().to_owned(),
        ))),
        Err(_) => Ok(ControlFlow::Simple(UVRTValue::Null)),
    }
}

/// Built-in `typeof`` function
///
/// Returns string-representation of provided value
fn tof(args: &[UVRTValue], _env: EnvRef<RTVariable>) -> Result<ControlFlow, SpannedError> {
    let Some(v) = args.first() else {
        unreachable!()
    };

    Ok(ControlFlow::Simple(UVRTValue::String(v.typeof_str())))
}

/// Built-in string `concat` function
///
/// Returns concatenated strings
fn concat(args: &[UVRTValue], _env: EnvRef<RTVariable>) -> Result<ControlFlow, SpannedError> {
    let str = args.iter().fold(String::default(), |mut acc, arg| {
        let UVRTValue::String(s) = arg else {
            unreachable!()
        };
        acc.push_str(s);
        acc
    });

    Ok(ControlFlow::Simple(UVRTValue::String(str)))
}
