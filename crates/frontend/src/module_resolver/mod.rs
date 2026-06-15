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
    module: &Spanned<ModuleImport>,
) -> Result<(String, Rc<SourceFileParsed>), SpannedError> {
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

    let source = SourceFile::load(&path).map_err(|e| {
        SpannedError::new(
            format!("Could not load module file: {e}"),
            module.get_span(),
        )
    })?;

    let name = if let Some(al) = &module.alias {
        al.clone().unwrap()
    } else {
        path.file_stem().unwrap().to_string_lossy().to_string()
    };

    Ok((name, process_file(Rc::new(source), true)?))
}

pub fn resolve_modules(
    modules: &[Spanned<ModuleImport>],
) -> Result<HashMap<String, Rc<SourceFileParsed>>, SpannedError> {
    let mut hm = HashMap::new();
    for module in modules.iter() {
        let (s, f) = resolve_by_path(module)?;
        hm.insert(s, f);
    }

    Ok(hm)
}
