use std::path::Path;
use ultraviolet_core::{
    errors::{SpannedError, error_renderer::ErrorRenderer},
    types::{
        backend::{ControlFlow, EnvRef},
        frontend::SourceFile,
    },
};

fn main() {
    let source = SourceFile::load(Path::new("./examples/file.uv")).unwrap();

    let ret = run(&source).map_err(|err| {
        eprintln!("{}", err.display_with_source(&source));
    });

    println!("{:?}", ret);
}

fn run(source: &SourceFile) -> Result<ControlFlow, SpannedError> {
    let ast = frontend::process(source)?;
    backend::eval::eval(&ast, EnvRef::default())
}
