use std::{env::args, path::Path};
use ultraviolet_core::{
    errors::{SpannedError, error_renderer::ErrorRenderer},
    types::{backend::ControlFlow, frontend::SourceFile},
};

use crate::help::print_help;

mod help;

fn main() {
    let args: Vec<String> = args().collect();

    let path = match args.get(1) {
        Some(path) => path,
        None => print_help(),
    };

    let source = SourceFile::load(Path::new(path)).unwrap();
    let ret = run(&source).map_err(|err| {
        eprintln!("{}", err.display_with_source(&source));
    });

    println!("{ret:?}");
}

fn run(source: &SourceFile) -> Result<ControlFlow, SpannedError> {
    let ast = frontend::process(source)?;
    backend::eval(&ast)
}
