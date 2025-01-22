use std::path::PathBuf;

use anyhow::Context;

pub(crate) fn make_java_class_files_directory() -> Result<PathBuf, anyhow::Error> {
    let java_class_files = out_dir()?.join("java_class_files");
    std::fs::create_dir_all(&java_class_files).with_context(|| {
        format!(
            "failed to create java directory: {}",
            java_class_files.display()
        )
    })?;
    Ok(java_class_files)
}

pub(crate) fn out_dir() -> anyhow::Result<PathBuf> {
    Ok(PathBuf::from(
        std::env::var("OUT_DIR").map_err(|_| anyhow::anyhow!("OUT_DIR not set"))?,
    ))
}