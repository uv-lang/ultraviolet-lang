use crate::help::print_help;
use std::{env::args, path::Path, rc::Rc};
use ultraviolet_core::{
    errors::SpannedError,
    types::{
        backend::{ControlFlow, UVRTValue},
        frontend::{SourceFile, number::Number},
    },
};
mod help;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = args().collect();

    let path = match args.get(1) {
        Some(path) => path,
        None => print_help(),
    };

    let source = match SourceFile::load(Path::new(path)) {
        Ok(s) => Rc::new(s),
        Err(err) => {
            eprintln!("Can't open source file: {}", err);
            std::process::exit(-1);
        },
    };

    let ret = run(source).unwrap_or_else(|err| {
        eprintln!("{err}");
        ControlFlow::Simple(UVRTValue::Number(Number::I8(-1)))
    });

    let return_code = match ret {
        ControlFlow::Simple(UVRTValue::Number(Number::I8(v)))
        | ControlFlow::Return(UVRTValue::Number(Number::I8(v))) => v,
        _ => 0,
    };

    std::process::exit(return_code.into());
}

fn run(source: Rc<SourceFile>) -> Result<ControlFlow, SpannedError> {
    let ast = frontend::process_file(source, "", false)?;
    backend::eval(&ast)
}
