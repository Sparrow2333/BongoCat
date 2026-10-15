//! Owns the extracted models until every consumer has shut down.

use std::{
    fs::OpenOptions,
    io::{self, Write},
    path::{Component, Path},
};

include!(concat!(env!("OUT_DIR"), "/embedded-presets.rs"));

#[cfg(test)]
#[path = "../standalone_build.rs"]
mod standalone_build;

pub(crate) struct PresetResources {
    directory: tempfile::TempDir,
}

impl PresetResources {
    pub(crate) fn prepare() -> io::Result<Self> {
        let directory = tempfile::Builder::new()
            .prefix("bongocat-presets-")
            .tempdir()?;
        Self::extract(directory, EMBEDDED_FILES)
    }

    fn extract(directory: tempfile::TempDir, files: &[(&str, &[u8])]) -> io::Result<Self> {
        let resources = Self { directory };
        bongocat_storage::set_private_directory(resources.path())?;
        extract_files(resources.path(), files)?;
        Ok(resources)
    }

    pub(crate) fn path(&self) -> &Path {
        self.directory.path()
    }
}

fn extract_files(root: &Path, files: &[(&str, &[u8])]) -> io::Result<()> {
    for (name, contents) in files {
        let relative = Path::new(name);
        if name.contains(['\\', ':'])
            || relative.as_os_str().is_empty()
            || !relative
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
        {
            return Err(io::Error::other("invalid embedded resource path"));
        }
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            bongocat_storage::create_private_dir_all(parent)?;
        }
        let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
        bongocat_storage::set_private_file(&file)?;
        file.write_all(contents)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bongocat_model::{ModelPackageLimits, PresetModelCatalog};
    use std::fs;

    #[test]
    fn embedding_excludes_desktop_metadata_and_records_provenance() {
        let root = tempfile::tempdir().expect("build fixture");
        let models = root.path().join("models");
        fs::create_dir_all(models.join("standard")).expect("model directory");
        fs::write(models.join("standard/cat.model3.json"), b"{}").expect("model");
        fs::write(models.join("standard/.DS_Store"), b"junk").expect("metadata");
        fs::write(models.join("Thumbs.db"), b"junk").expect("metadata");
        let provenance = root.path().join("provenance.json");
        fs::write(&provenance, b"{}").expect("provenance");
        let output = root.path().join("embedded.rs");
        standalone_build::generate(&models, &output, Some(&provenance)).expect("generate");
        let generated = fs::read_to_string(output).expect("generated source");
        assert!(generated.contains("standard/cat.model3.json"));
        assert!(generated.contains("build-provenance.json"));
        assert!(!generated.contains("DS_Store"));
        assert!(!generated.contains("Thumbs.db"));
    }

    #[cfg(unix)]
    #[test]
    fn embedding_rejects_symbolic_links() {
        let root = tempfile::tempdir().expect("build fixture");
        let models = root.path().join("models");
        fs::create_dir(&models).expect("model directory");
        let outside = root.path().join("outside");
        fs::write(&outside, b"private").expect("outside file");
        std::os::unix::fs::symlink(outside, models.join("link")).expect("symlink");
        assert!(
            standalone_build::generate(&models, &root.path().join("embedded.rs"), None).is_err()
        );
    }

    #[test]
    fn embedded_models_load_without_sidecar_files_and_are_removed_on_drop() {
        let resources = PresetResources::prepare().expect("extract embedded models");
        let root = resources.path().to_owned();
        PresetModelCatalog::open(&root, ModelPackageLimits::default())
            .expect("all embedded preset models are valid");
        assert!(root.join("standard").is_dir());
        assert!(root.join("keyboard").is_dir());
        assert!(root.join("gamepad").is_dir());
        drop(resources);
        assert!(
            !root.exists(),
            "resources must be cleaned after their owner exits"
        );
    }

    #[test]
    fn extraction_rejects_escaping_paths_and_existing_files() {
        let root = tempfile::tempdir().expect("private destination");
        for name in [
            "../escape",
            "/absolute",
            "C:\\escape",
            "",
            "model/../escape",
        ] {
            assert!(
                extract_files(root.path(), &[(name, b"bad")]).is_err(),
                "{name}"
            );
        }
        let target = root.path().join("model.json");
        fs::write(&target, b"original").expect("existing resource");
        assert!(extract_files(root.path(), &[("model.json", b"replacement")]).is_err());
        assert_eq!(fs::read(target).expect("preserved file"), b"original");
    }

    #[test]
    fn failed_extraction_removes_its_partial_directory() {
        let root = tempfile::tempdir().expect("private destination");
        let path = root.path().to_owned();
        assert!(
            PresetResources::extract(root, &[("first", b"ok"), ("../escape", b"bad")]).is_err()
        );
        assert!(!path.exists());
    }
}
