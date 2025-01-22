use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Context;

/// build-rs helper: compile all `java` files in `java_src` and
/// store into `$OUT_DIR/java_class_files`.
///
/// Adjust `CLASSPATH` and set the variable for rustc.
///
/// Meant to be invoked from the `build.rs` of a gluegun-java-generated crate.
pub fn build_rs_main() -> anyhow::Result<()> {
    let java_class_files = make_java_class_files_directory()?;
    let new_classpath = init_classpath(&java_class_files);
    for java_path in java_files("java_src".as_ref()) {
        compile_java(&java_path, &java_class_files, &new_classpath)?;
    }
    Ok(())
}

fn make_java_class_files_directory() -> Result<PathBuf, anyhow::Error> {
    let java_class_files = out_dir()?.join("java_class_files");
    std::fs::create_dir_all(&java_class_files).with_context(|| {
        format!(
            "failed to create java directory: {}",
            java_class_files.display()
        )
    })?;
    Ok(java_class_files)
}

fn init_classpath(java_class_files: &Path) -> String {
    let existing_classpath = std::env::var("CLASSPATH").unwrap_or_default();
    println!("cargo::rerun-if-env-changed=CLASSPATH");
    let new_classpath = format!("{}:{existing_classpath}", java_class_files.display());
    println!("cargo::rustc-env=CLASSPATH={new_classpath}");
    new_classpath
}

fn out_dir() -> anyhow::Result<PathBuf> {
    Ok(PathBuf::from(
        std::env::var("OUT_DIR").map_err(|_| anyhow::anyhow!("OUT_DIR not set"))?,
    ))
}

fn java_files(java_src: &Path) -> impl Iterator<Item = PathBuf> {
    walkdir::WalkDir::new(java_src)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "java")
                .unwrap_or(false)
        })
        .map(|e| e.into_path())
}

fn compile_java(
    java_path: &Path,
    java_class_files: &Path,
    new_classpath: &str,
) -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed={}", java_path.display());

    Command::new("javac")
        .arg("-d")
        .arg(&java_class_files)
        .arg("-cp")
        .arg(&new_classpath)
        .arg(&java_path)
        .output()
        .with_context(|| format!("invoking `javac` on `{}`", java_path.display()))?;

    Ok(())
}

/// Main function from the binary
pub fn bin_main() -> anyhow::Result<()> {
    anyhow::bail!("TODO")
}