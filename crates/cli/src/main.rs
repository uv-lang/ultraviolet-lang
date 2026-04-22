use std::{env::args, path::Path};
use ultraviolet_core::{
    errors::{SpannedError, error_renderer::ErrorRenderer},
    types::{
        backend::{ControlFlow, UVRTValue},
        frontend::{SourceFile, ast::Number},
    },
};

use crate::help::print_help;

mod help;

fn main() {
    let args: Vec<String> = args().collect();

    let path = match args.get(1) {
        Some(path) => path,
        None => print_help(),
    };

    let source = match SourceFile::load(Path::new(path)) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("Can't open source file: {}", err);
            std::process::exit(-1);
        },
    };

    let ret = run(&source).unwrap_or_else(|err| {
        eprintln!("{}", err.display_with_source(&source));
        ControlFlow::Simple(UVRTValue::Number(Number::Int(-1)))
    });

    let return_code = match ret {
        ControlFlow::Simple(UVRTValue::Number(Number::Int(v)))
        | ControlFlow::Return(UVRTValue::Number(Number::Int(v))) => v,
        _ => 0,
    };

    std::process::exit(return_code.try_into().unwrap_or(0));
}

fn run(source: &SourceFile) -> Result<ControlFlow, SpannedError> {
    let ast = frontend::process(source)?;
    backend::eval(&ast)
}
