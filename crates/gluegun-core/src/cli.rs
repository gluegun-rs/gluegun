//! A "GlueGun CLI" is a Rust crate that creates the glue between the Rust code and
//! some other language. Most GlueGun CLI crates can use the Clap structs defined
//! in this file.

use std::{path::PathBuf, str::FromStr};

use crate::idl::Idl;

/// These are the subcommands executed by our system.
/// Your extension should be able to respond to them.
#[derive(clap::Subcommand)]
pub enum GlueGunCommand {
    Generate {
        idl: IdlArg,

        #[command(flatten)]
        crate_args: GlueGunCrateArgs,
    },
}

impl AsRef<GlueGunCrateArgs> for GlueGunCrateArgs {
    fn as_ref(&self) -> &GlueGunCrateArgs {
        self
    }
}

#[derive(clap::Args)]
#[non_exhaustive]
pub struct GlueGunCrateArgs {
    /// Path at which to create the crate
    pub path: PathBuf,

    /// Name to give the crate; if `None`, then just let `cargo` pick a name.
    pub crate_name: Option<String>,
}

/// A wrapper around the GlueGun [`Idl`] that implements [`FromStr`][],
/// permitting it to be used from the CLI.
#[derive(Clone, Debug)]
pub struct IdlArg {
    pub idl: Idl,
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
