use std::{
    collections::HashMap,
    env::current_dir,
    io,
    ops::Deref,
    path::{Path, PathBuf},
    rc::Rc,
};

use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::Positional,
    types::frontend::{SourceFile, SourceFileParsed, Spanned, ast::ModuleImport},
};

use crate::process_file;

/// Get path relative to current
fn get_relative_path(mod_path: &Path, path: &PathBuf) -> Result<PathBuf, io::Error> {
    let mut p = mod_path.parent().unwrap_or(Path::new("")).join(path);
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
fn exists_file(path: &Path) -> bool {
    path.is_file() && path.exists()
}

/// TODO: IMPLEMENT THIS
#[allow(dead_code)]
fn get_global_modules_path(_path: PathBuf) -> Result<PathBuf, io::Error> {
    todo!()
}

/// Resolving a relative path along a chain of paths
fn resolve_by_path(
    current_file: Rc<SourceFile>,
    module: &Spanned<ModuleImport>,
) -> Result<(String, Rc<SourceFileParsed>), SpannedError> {
    let mut path = PathBuf::from(module.path.as_str());

    if let Ok(p) = get_relative_path(&current_file.path, &path)
        && exists_file(&p)
    {
        path = p;
    } else if let Ok(p) = get_modules_path(&mut path)
        && exists_file(&p)
    {
        path = p;
    } else {
        return Err(SpannedError::new(
            format!("Could not load module `{}`", module.path.deref()),
            module.get_span(),
        ));
    }

    let source = SourceFile::load(&path).map_err(|e| {
        SpannedError::new(
            format!("Could not load module file: {e}"),
            module.get_span(),
        )
    })?;

    Ok((
        module.name.clone().unwrap(),
        process_file(Rc::new(source), true)?,
    ))
}

pub fn resolve_modules(
    current_file: Rc<SourceFile>,
    modules: &[Spanned<ModuleImport>],
) -> Result<HashMap<String, Rc<SourceFileParsed>>, SpannedError> {
    let mut hm = HashMap::new();
    for module in modules.iter() {
        let (s, f) = resolve_by_path(current_file.clone(), module)?;
        hm.insert(s, f);
    }

    Ok(hm)
}
