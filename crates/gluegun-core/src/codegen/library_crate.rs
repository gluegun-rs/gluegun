use super::CodeWriter;
use crate::cli::GlueGunDestinationCrate;
use accessors_rs::Accessors;
use anyhow::Context;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
};

/// Type to create a GlueGun adapter crate.
#[derive(Debug, Accessors)]
pub struct LibraryCrate {
    /// The Rust name of the crate being generated (may include e.g., `-`)
    #[accessors(get)]
    crate_name: String,

    /// The path where the crate will be generated (directory name)
    #[accessors(get)]
    crate_path: PathBuf,

    cargo_command: Command,
    dependencies: Vec<Dependency>,
    directories: Vec<PathBuf>,
    files: BTreeMap<PathBuf, Vec<u8>>,
}

impl LibraryCrate {
    /// Create an instance from a [`GlueGunDestinationCrate`][].
    /// This has no immediate effect.
    /// You can use the various methods on this returned value to configure files that should be present.
    /// Once everything is ready, you can invoke [`Self::execute`][] to make changes on disk.
    pub(crate) fn from_args(args: &GlueGunDestinationCrate) -> Self {
        let mut cargo_command = std::process::Command::new("cargo");
        cargo_command.arg("new");
        // cargo_command.arg("-q");
        cargo_command.arg("--lib");
        cargo_command.arg(&args.path);
        cargo_command.arg("--name");
        cargo_command.arg(&args.crate_name);

        Self {
            crate_name: args.crate_name.clone(),
            crate_path: args.path.clone(),
            cargo_command,
            directories: Default::default(),
            files: Default::default(),
            dependencies: Default::default(),
        }
    }

    /// Generate the crate on disk. May fail.
    pub fn generate(mut self) -> anyhow::Result<()> {
        // FIXME: we shouldn't just delete the old thing
        if self.crate_path.exists() {
            std::fs::remove_dir_all(&self.crate_path)
                .with_context(|| format!("removing {}", self.crate_path.display()))?;
        }

        self.execute()
            .with_context(|| format!("generating crate at path {}", self.crate_path.display()))
    }

    /// Internal method to generate code.
    fn execute(&mut self) -> anyhow::Result<()> {
        self.ensure_workspace()?;

        eprintln!("cargo_command: {:?}", self.cargo_command);
        let status = self.cargo_command.status()?;
        if !status.success() {
            anyhow::bail!(
                "cargo command `{:?}` failed with exit status `{}`",
                self.cargo_command,
                status,
            );
        }

        for dependency in &self.dependencies {
            eprintln!("adding {dependency:?}");
            dependency.execute_cargo_add(&self.crate_name)?;
        }

        for directory in &self.directories {
            let crate_directory = self.crate_path.join(directory);
            eprintln!("creating {crate_directory:?}");
            std::fs::create_dir_all(&crate_directory).with_context(|| {
                format!("creating directory at `{}`", crate_directory.display())
            })?;
        }

        for (path, data) in &self.files {
            let file_path = self.crate_path.join(path);
            eprintln!("writing to {file_path:?}");

            if let Some(dir_path) = file_path.parent() {
                std::fs::create_dir_all(dir_path)
                    .with_context(|| format!("creating directory at `{}`", dir_path.display()))?;
            }

            std::fs::write(&file_path, data)
                .with_context(|| format!("writing to file at `{}`", file_path.display()))?;
        }

        Ok(())
    }

    /// Identifies the surrounding cargo.toml and ensures that it is setup to act as a workspace.
    fn ensure_workspace(&self) -> anyhow::Result<()> {
        let workspace_path = self.locate_workspace()?;

        // Read the contents of the workspace cargo.toml
        let contents = std::fs::read_to_string(&workspace_path)
            .context("failed to read workspace cargo.toml")?;

        // Check if [workspace] section exists
        if !contents.contains("[workspace]") {
            // Append [workspace] section if it doesn't exist
            std::fs::write(&workspace_path, format!("{contents}\n\n[workspace]\n"))
                .context("failed to update workspace cargo.toml")?;
        }

        Ok(())
    }

    fn locate_workspace(&self) -> anyhow::Result<PathBuf> {
        #[derive(Deserialize)]
        struct CargoLocateProjectOutput {
            root: PathBuf,
        }

        let output = Command::new("cargo")
            .args(["locate-project", "--workspace"])
            .output()
            .context("failed to execute cargo locate-project")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("cargo locate-project failed: {}", stderr);
        }

        let json = String::from_utf8(output.stdout)
            .context("cargo locate-project output was not valid UTF-8")?;

        let project_info: CargoLocateProjectOutput =
            serde_json::from_str(&json).context("failed to parse cargo locate-project output")?;

        Ok(project_info.root)
    }

    /// Add a dependency to the crate with the given name.
    /// Returns a builder that can be used to configure additional options.
    pub fn add_dependency(&mut self, crate_name: &str) -> AddDependency<'_> {
        AddDependency {
            krate: self,
            dependency: Dependency {
                crate_name: crate_name.to_string(),
                kind: None,
                path: None,
                version: None,
                features: Default::default(),
                no_default_features: Default::default(),
            },
        }
    }

    /// Create a directory (and all required parent directories)
    /// within the crate. Returns a builder which can be used to populate
    /// that directory with files.
    ///
    /// No changes on disk occur until [`Self::generate`][] is called.
    ///
    /// # Parameters
    ///
    /// * `path`, path for source file relative to the root of crate
    pub fn add_dir(&mut self, path: impl AsRef<Path>) -> anyhow::Result<DirBuilder<'_>> {
        let dir_path = path.as_ref().to_path_buf();
        self.directories.push(dir_path.clone());
        Ok(DirBuilder {
            dir_path,
            krate: self,
        })
    }

    /// Return a [`CodeWriter`][] for the contents of a file in the crate.
    ///
    /// No changes on disk occur until [`Self::generate`][] is called.
    ///
    /// # Parameters
    ///
    /// * `path`, path for source file relative to the root of crate
    pub fn add_file(&mut self, path: impl AsRef<Path>) -> anyhow::Result<CodeWriter<'_>> {
        let path = path.as_ref();

        if self.files.contains_key(path) {
            anyhow::bail!("duplicate path: `{}`", path.display());
        }

        Ok(CodeWriter::new(LibraryFileWriter {
            krate: self,
            path: path.to_path_buf(),
            contents: Default::default(),
        }))
    }
}

pub struct DirBuilder<'w> {
    dir_path: PathBuf,
    krate: &'w mut LibraryCrate,
}

impl DirBuilder<'_> {
    /// Return a [`CodeWriter`][] for the contents of a file in the crate.
    ///
    /// No changes on disk occur until [`Self::generate`][] is called.
    ///
    /// # Parameters
    ///
    /// * `path`, path for source file relative to the root of crate
    pub fn add_file(&mut self, path: impl AsRef<Path>) -> anyhow::Result<CodeWriter<'_>> {
        let path = self.dir_path.join(path);
        self.krate.add_file(path)
    }
}

struct LibraryFileWriter<'w> {
    krate: &'w mut LibraryCrate,
    path: PathBuf,
    contents: Vec<u8>,
}

impl std::io::Write for LibraryFileWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.contents.extend(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for LibraryFileWriter<'_> {
    fn drop(&mut self) {
        self.krate
            .files
            .insert(self.path.clone(), self.contents.clone());
    }
}

/// Record of a dependency to add
#[derive(Debug, Default)]
struct Dependency {
    crate_name: String,
    kind: Option<DependencyKind>,
    path: Option<PathBuf>,
    version: Option<String>,
    features: Vec<String>,
    no_default_features: bool,
}

#[derive(Debug)]
enum DependencyKind {
    Build,
    Dev,
}

impl Dependency {
    fn execute_cargo_add(&self, crate_name: &str) -> anyhow::Result<()> {
        let mut command = std::process::Command::new("cargo");
        command.arg("add");

        command.arg("-p");
        command.arg(crate_name);

        if let Some(path) = &self.path {
            command.arg("--path").arg(path);
        } else if let Some(version) = &self.version {
            command.arg(&format!("{}@{}", self.crate_name, version));
        } else {
            panic!("dependency `{crate_name}` needs either a path or a version");
        }

        if !self.features.is_empty() {
            command.arg("--features");
            command.arg(self.features.join(","));
        }

        if self.no_default_features {
            command.arg("--no-default-features");
        }

        if let Some(kind) = &self.kind {
            match kind {
                DependencyKind::Build => command.arg("--build"),
                DependencyKind::Dev => command.arg("--dev"),
            };
        }


        let status = command.status()?;
        if !status.success() {
            anyhow::bail!(
                "cargo command `{:?}` failed with exit status `{}`",
                command,
                status,
            );
        }
        Ok(())
    }
}

/// Builder returned by [`LibraryCrate::add_dependency`][].
/// Allows configuring the version and required features.
pub struct AddDependency<'w> {
    krate: &'w mut LibraryCrate,
    dependency: Dependency,
}

impl AddDependency<'_> {
    /// Add a required feature for the dependency
    pub fn feature(mut self, feature: impl ToString) -> Self {
        self.dependency.features.push(feature.to_string());
        self
    }

    /// Add a required feature for the dependency
    pub fn no_default_features(mut self) -> Self {
        self.dependency.no_default_features = true;
        self
    }

    /// Mark this as a build dependency
    pub fn build(mut self) -> Self {
        self.dependency.kind = Some(DependencyKind::Build);
        self
    }

    /// Mark this as a dev dependency
    pub fn dev(mut self) -> Self {
        self.dependency.kind = Some(DependencyKind::Dev);
        self
    }

    /// Mark this as a dev dependency
    pub fn path(mut self, path: impl Into<PathBuf>) -> Self {
        self.dependency.path = Some(path.into());
        self
    }

    /// Version to request
    pub fn version(mut self, path: impl ToString) -> Self {
        self.dependency.version = Some(path.to_string());
        self
    }
}

impl Drop for AddDependency<'_> {
    fn drop(&mut self) {
        self.krate
            .dependencies
            .push(std::mem::replace(&mut self.dependency, Default::default()));
    }
}
