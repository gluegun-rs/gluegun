use std::{
    io::Write,
    process::{Command, Stdio},
};

use anyhow::Context;
use clap::Parser;

/// A simple Cli you can use for your own parser.
#[derive(clap::Parser)]
struct Cli {
    #[command(flatten)]
    manifest: clap_cargo::Manifest,

    #[command(flatten)]
    workspace: clap_cargo::Workspace,

    /// Specify a list of plugins to use.
    plugins: Vec<String>,
}

/// Main function for the gluegun CLI.
pub fn cli_main() -> anyhow::Result<()> {
    let cli = Cli::try_parse()?;

    let metadata = cli.manifest.metadata().exec()?;
    let (selected, _excluded) = cli.workspace.partition_packages(&metadata);

    if selected.is_empty() {
        anyhow::bail!("no packages selected -- you may have misspelled the package name?");
    }

    if cli.plugins.is_empty() {
        anyhow::bail!("no plugins specified");
    }

    for package in selected {
        for plugin in &cli.plugins {
            apply_plugin(plugin, package)?;
        }
    }

    Ok(())
}

fn apply_plugin(plugin: &str, package: &cargo_metadata::Package) -> anyhow::Result<()> {
    if let Some(_) = package.source {
        anyhow::bail!("{pkg}: can only process local packages", pkg = package.name);
    }

    // FIXME: Don't be so hacky. My god Niko, you should be ashamed of yourself.
    let cargo_toml_path = &package.manifest_path;
    let src_lib_rs = cargo_toml_path.parent().unwrap().join("src/lib.rs");

    let idl = gluegun_idl::Parser::new()
        .parse_crate_named(&package.name, &src_lib_rs)
        .with_context(|| format!("extracting interface from `{src_lib_rs}`"))?;

    // Search for `workspace.metadata.gluegun.tool_name` and
    // `package.metadata.gluegun.tool_name`.
    let workspace_metadata = extract_metadata(plugin, &package.metadata);
    let package_metdata = extract_metadata(plugin, &package.metadata);
    let metadata = merge_metadata(workspace_metadata, package_metdata)
        .with_context(|| format!("merging workspace and package metadata"))?;

    let mut child = Command::new(format!("gluegun-{plugin}"))
        .arg("gg")
        .stdin(Stdio::piped()) // Configure stdin
        .spawn()
        .with_context(|| format!("spawning gluegun-{plugin}"))?;

    // Write the data to the child's stdin.
    // This has to be kept in sync with the definition from `gluegun_core::cli``
    let Some(mut stdin) = child.stdin.take() else {
        anyhow::bail!("failed to take stdin");
    };
    writeln!(stdin, "{{")?;
    writeln!(stdin, "  idl: {},", serde_json::to_string(&idl)?)?;
    writeln!(stdin, "  metadata: {}", serde_json::to_string(&metadata)?)?;
    writeln!(stdin, "}}")?;
    std::mem::drop(stdin);

    let exit_status = child
        .wait()
        .with_context(|| format!("waiting for gluegun-{plugin}"))?;

    if exit_status.success() {
        Ok(())
    } else {
        anyhow::bail!("gluegun-{plugin} failed with code {exit_status}");
    }
}

/// Given a root object, exact `{metadata}.gluegun.{plugin}`:
fn extract_metadata<'r>(
    plugin: &str,
    metadata: &'r serde_json::Value,
) -> Option<&'r serde_json::Value> {
    Some(metadata.get("gluegun")?.get(plugin)?)
}

/// Merge metadata from workspace/package
fn merge_metadata(
    workspace_metadata: Option<&serde_json::Value>,
    package_metadata: Option<&serde_json::Value>,
) -> anyhow::Result<serde_json::Value> {
    match (workspace_metadata, package_metadata) {
        (Some(workspace), Some(package)) => merge_values(workspace, package),
        (Some(workspace), None) => Ok(workspace.clone()),
        (None, Some(package)) => Ok(package.clone()),
        (None, None) => Ok(serde_json::Value::Null),
    }
}

/// Merge metadata values from workspace/package.
///
/// Generally speaking, package wins, but for maps we take the keys from workspace that are not present in package.
fn merge_values(
    workspace_value: &serde_json::Value,
    package_value: &serde_json::Value,
) -> anyhow::Result<serde_json::Value> {
    match (workspace_value, package_value) {
        (serde_json::Value::Null, serde_json::Value::Null) => Ok(serde_json::Value::Null),

        (serde_json::Value::Number(_), serde_json::Value::Number(_))
        | (serde_json::Value::Bool(_), serde_json::Value::Bool(_))
        | (serde_json::Value::String(_), serde_json::Value::String(_))
        | (serde_json::Value::Array(_), serde_json::Value::Array(_)) => Ok(package_value.clone()),

        (serde_json::Value::Object(workspace_map), serde_json::Value::Object(package_map)) => {
            let mut merged = workspace_map.clone();

            for (key, value) in package_map {
                merged.insert(key.clone(), value.clone());
            }

            Ok(serde_json::Value::Object(merged))
        }

        (serde_json::Value::Null, _)
        | (serde_json::Value::Number(_), _)
        | (serde_json::Value::Bool(_), _)
        | (serde_json::Value::String(_), _)
        | (serde_json::Value::Array(_), _)
        | (_, serde_json::Value::Null)
        | (_, serde_json::Value::Number(_))
        | (_, serde_json::Value::Bool(_))
        | (_, serde_json::Value::String(_))
        | (_, serde_json::Value::Array(_)) => anyhow::bail!(
            "cannot merge workspace/package configuration:\
            \n    workspace: {workspace_value}\
            \n    package: {package_value}"
        ),
    }
}
