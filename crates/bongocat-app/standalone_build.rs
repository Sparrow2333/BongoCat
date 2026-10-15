//! Compile-time resource embedding for the single executable distribution.

#![allow(clippy::print_stdout)]

use std::{
    fs,
    io::{self, Write},
    path::Path,
};

pub fn generate(models: &Path, output: &Path, provenance: Option<&Path>) -> io::Result<()> {
    let mut generated = fs::File::create(output)?;
    writeln!(generated, "static EMBEDDED_FILES: &[(&str, &[u8])] = &[")?;
    append_files(models, models, &mut generated)?;
    if let Some(path) = provenance {
        println!("cargo::rerun-if-changed={}", path.display());
        writeln!(
            generated,
            "(\"build-provenance.json\", include_bytes!({:?})),",
            path.to_str()
                .ok_or_else(|| io::Error::other("non-UTF-8 provenance path"))?
        )?;
    }
    writeln!(generated, "];")
}

fn append_files(root: &Path, directory: &Path, generated: &mut fs::File) -> io::Result<()> {
    println!("cargo::rerun-if-changed={}", directory.display());
    let mut entries = fs::read_dir(directory)?.collect::<io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let Some(name) = entry.file_name().into_string().ok() else {
            continue;
        };
        if name.starts_with('.')
            || matches!(name.as_str(), "__MACOSX" | "Thumbs.db" | "desktop.ini")
        {
            continue;
        }
        let path = entry.path();
        let kind = entry.file_type()?;
        if kind.is_dir() {
            append_files(root, &path, generated)?;
        } else if kind.is_file() {
            let relative = path.strip_prefix(root).map_err(io::Error::other)?;
            let name = relative
                .to_str()
                .ok_or_else(|| io::Error::other("non-UTF-8 resource"))?
                .replace('\\', "/");
            writeln!(
                generated,
                "({name:?}, include_bytes!({:?})),",
                path.to_str()
                    .ok_or_else(|| io::Error::other("non-UTF-8 resource path"))?
            )?;
        } else {
            return Err(io::Error::other(
                "embedded resources must be regular files or directories",
            ));
        }
    }
    Ok(())
}
