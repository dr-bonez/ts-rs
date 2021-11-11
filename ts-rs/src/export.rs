use std::{
    any::TypeId,
    collections::BTreeSet,
    fmt::Write,
    path::{Component, Path, PathBuf},
};

pub use dprint_plugin_typescript::{configuration::ConfigurationBuilder, format_text};
use thiserror::Error;

use crate::TS;

/// An error which may occur when exporting a type
#[derive(Error, Debug)]
pub enum ExportError {
    #[error("this type cannot be exported")]
    CannotBeExported,
    #[error("an error occurred while formatting the generated typescript output")]
    Formatting(String),
    #[error("an error occurred while performing IO")]
    Io(#[from] std::io::Error),
}

/// Export `T` to the file specified by the `#[ts(export = ..)]` attribute and/or the `out_dir`
/// setting in the `ts.toml` config file.
pub(crate) fn export_type<T: TS + ?Sized>() -> Result<(), ExportError> {
    let path = Path::new(T::EXPORT_TO.ok_or(ExportError::CannotBeExported)?);

    let mut buffer = String::with_capacity(1024);
    generate_imports::<T>(&mut buffer)?;
    generate_decl::<T>(&mut buffer);

    // format output
    let fmt_cfg = ConfigurationBuilder::new().deno().build();
    let buffer = format_text(path, &buffer, &fmt_cfg).map_err(ExportError::Formatting)?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, &buffer)?;
    Ok(())
}

/// Push the declaration of `T`
fn generate_decl<T: TS + ?Sized>(out: &mut String) {
    out.push_str("export ");
    out.push_str(&T::decl());
}

/// Push an import statement for all dependencies of `T`
fn generate_imports<T: TS + ?Sized>(out: &mut String) -> Result<(), ExportError> {
    let path = Path::new(T::EXPORT_TO.ok_or(ExportError::CannotBeExported)?);

    let deps = T::dependencies()
        .into_iter()
        .filter(|dep| dep.type_id != TypeId::of::<T>())
        .collect::<BTreeSet<_>>();

    for dep in deps {
        let rel_path = import_path(path, Path::new(dep.exported_to));
        writeln!(out, "import {{ {} }} from {:?};", &dep.ts_name, rel_path).unwrap();
    }
    writeln!(out).unwrap();
    Ok(())
}

/// Returns the required import path for importing `import` from the file `from`
fn import_path(from: &Path, import: &Path) -> String {
    let rel_path =
        diff_paths(import, from.parent().unwrap()).expect("failed to calculate import path");
    match rel_path.components().next() {
        Some(Component::Normal(_)) => format!("./{}", rel_path.to_string_lossy()),
        _ => rel_path.to_string_lossy().into(),
    }
    .trim_end_matches(".ts")
    .to_owned()
}

// Construct a relative path from a provided base directory path to the provided path.
//
// Copyright 2012-2015 The Rust Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Adapted from rustc's path_relative_from
// https://github.com/rust-lang/rust/blob/e1d0de82cc40b666b88d4a6d2c9dcbc81d7ed27f/src/librustc_back/rpath.rs#L116-L158
fn diff_paths<P, B>(path: P, base: B) -> Option<PathBuf>
where
    P: AsRef<Path>,
    B: AsRef<Path>,
{
    let path = path.as_ref();
    let base = base.as_ref();

    if path.is_absolute() != base.is_absolute() {
        if path.is_absolute() {
            Some(PathBuf::from(path))
        } else {
            None
        }
    } else {
        let mut ita = path.components();
        let mut itb = base.components();
        let mut comps: Vec<Component> = vec![];
        loop {
            match (ita.next(), itb.next()) {
                (None, None) => break,
                (Some(a), None) => {
                    comps.push(a);
                    comps.extend(ita.by_ref());
                    break;
                }
                (None, _) => comps.push(Component::ParentDir),
                (Some(a), Some(b)) if comps.is_empty() && a == b => (),
                (Some(a), Some(b)) if b == Component::CurDir => comps.push(a),
                (Some(_), Some(b)) if b == Component::ParentDir => return None,
                (Some(a), Some(_)) => {
                    comps.push(Component::ParentDir);
                    for _ in itb {
                        comps.push(Component::ParentDir);
                    }
                    comps.push(a);
                    comps.extend(ita.by_ref());
                    break;
                }
            }
        }
        Some(comps.iter().map(|c| c.as_os_str()).collect())
    }
}
