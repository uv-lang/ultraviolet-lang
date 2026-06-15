use std::{env::current_dir, io, ops::Deref, path::PathBuf};

use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::Positional,
    types::frontend::{SourceFileParsed, Spanned, ast::ModuleImport},
};

/// Get path relative to current
fn get_relative_path(path: &PathBuf) -> Result<PathBuf, io::Error> {
    let mut p = current_dir()?.join(path);
    p.set_extension("uv");
    Ok(p)
}

/// Get path of relative uvmodules folder
fn get_modules_path(path: &mut PathBuf) -> Result<PathBuf, io::Error> {
    let mut dp = path.clone();
    dp.set_extension("");
    path.set_file_name("mod.uv");
    let p = current_dir()?.join("uvmodules").join(dp).join(path);
    Ok(p)
}

/// Check if path is file and file exists
fn exists_file(path: &PathBuf) -> bool {
    path.is_file() && path.exists()
}

/// TODO: IMPLEMENT THIS
#[allow(dead_code)]
fn get_global_modules_path(_path: PathBuf) -> Result<PathBuf, io::Error> {
    todo!()
}

/// Resolving a relative path along a chain of paths
fn resolve_by_path(module: &Spanned<ModuleImport>) -> Result<(), SpannedError> {
    let mut path = PathBuf::from(module.name.as_str());

    if let Ok(p) = get_relative_path(&path)
        && exists_file(&p)
    {
        path = p;
    } else if let Ok(p) = get_modules_path(&mut path)
        && exists_file(&p)
    {
        path = p;
    } else {
        return Err(SpannedError::new(
            format!("Could not load module `{}`", module.name.deref()),
            module.get_span(),
        ));
    }

    println!("{:?}", path);

    todo!()
}

pub fn resolve_modules(
    modules: &Vec<Spanned<ModuleImport>>,
) -> Result<Vec<SourceFileParsed>, SpannedError> {
    modules
        .iter()
        .map(|m| resolve_by_path(m))
        .collect::<Result<(), SpannedError>>()?;

    Ok(Vec::new())
}
