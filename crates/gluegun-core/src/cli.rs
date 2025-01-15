//! A "GlueGun CLI" is a Rust crate that creates the glue between the Rust code and
//! some other language. Most GlueGun CLI crates can use the Clap structs defined
//! in this file.

use std::{path::PathBuf, str::FromStr};

use clap::Parser;

use crate::idl::Idl;

/// Trait implemented by gluegun helper applications.
/// Your `main` function should invoke [`run`][].
/// By convention, types that implement this trait should be named `GlueGunX` where `X` is the name of your helper.
pub trait GlueGunHelper {
    /// Returns the helper name that users provide to invoke this, e.g., for `gluegun-java`, returns `"java"`.
    fn name(&self) -> String;

    /// Generate a helper crate `dest_crate` given the `idl`
    fn generate(self, idl: Idl, dest_crate: GlueGunDestinationCrate) -> anyhow::Result<()>;
}

/// The "main" function for a gluegun helper. Defines standard argument parsing.
pub fn run(helper: impl GlueGunHelper) -> anyhow::Result<()> {
    let args = Cli::try_parse()?;
    match args.command {
        GlueGunCommand::Generate { idl, crate_args } => helper.generate(idl.into(), crate_args),
    }
}

/// A simple Cli you can use for your own parser.
#[derive(clap::Parser)]
struct Cli {
    #[command(subcommand)]
    command: GlueGunCommand,
}

/// These are the subcommands executed by our system.
/// Your extension should be able to respond to them.
#[derive(clap::Subcommand)]
enum GlueGunCommand {
    Generate {
        idl: IdlArg,

        #[command(flatten)]
        crate_args: GlueGunDestinationCrate,
    },
}

impl AsRef<GlueGunDestinationCrate> for GlueGunDestinationCrate {
    fn as_ref(&self) -> &GlueGunDestinationCrate {
        self
    }
}

/// The arguments that identify where the crate should be generated.
/// You don't normally need to inspect the fields of this struct,
/// instead just invoke [`LibraryCrate::from_args`](`crate::codegen::LibraryCrate::from_args`).
#[derive(clap::Args, Debug)]
#[non_exhaustive]
pub struct GlueGunDestinationCrate {
    /// Path at which to create the crate
    pub path: PathBuf,

    /// Name to give the crate; if `None`, then just let `cargo` pick a name.
    pub crate_name: Option<String>,
}

/// A wrapper around the GlueGun [`Idl`] that implements [`FromStr`][],
/// permitting it to be used from the CLI.
#[derive(Clone, Debug)]
struct IdlArg {
    idl: Idl,
}

impl FromStr for IdlArg {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(IdlArg {
            idl: serde_json::from_str(s)?,
        })
    }
}

impl std::ops::Deref for IdlArg {
    type Target = Idl;

    fn deref(&self) -> &Self::Target {
        &self.idl
    }
}

impl AsRef<Idl> for IdlArg {
    fn as_ref(&self) -> &Idl {
        &self.idl
    }
}

impl From<IdlArg> for Idl {
    fn from(val: IdlArg) -> Self {
        val.idl
    }
}
