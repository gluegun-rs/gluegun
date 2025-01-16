//! A "GlueGun CLI" is a Rust crate that creates the glue between the Rust code and
//! some other language. Most GlueGun CLI crates can use the Clap structs defined
//! in this file.

use std::path::PathBuf;

use accessors_rs::Accessors;
use serde::{de::DeserializeOwned, Deserialize};

use crate::{codegen::LibraryCrate, idl::Idl};

/// Trait implemented by gluegun helper applications.
/// Your `main` function should invoke [`run`][].
/// By convention, types that implement this trait should be named `GlueGunX` where `X` is the name of your helper.
pub trait GlueGunHelper {
    /// The metadata type used by this helper.
    /// This metadata will be extracted from the `Cargo.toml``.
    /// You can use `serde_json::Value` if you would like to just capture free-form.
    type Metadata: DeserializeOwned;

    /// Returns the helper name that users provide to invoke this, e.g., for `gluegun-java`, returns `"java"`.
    fn name(&self) -> String;

    /// Generate a helper crate `dest_crate` given the `idl`
    fn generate(self, cx: &mut GenerateCx, metadata: &Self::Metadata) -> anyhow::Result<()>;
}

/// The "main" function for a gluegun helper. Defines standard argument parsing.
pub fn run<G>(helper: G) -> anyhow::Result<()>
where
    G: GlueGunHelper,
{
    // cargo-gluegun will invoke us with `gg` as argument and a JSON doc on stdin.
    let mut args = std::env::args();
    let Some(_arg0) = args.next() else {
        anyhow::bail!("expected to give given an argument");
    };
    let Some(arg1) = args.next() else {
        anyhow::bail!("expected to give given an argument");
    };
    if arg1 != format!("gg-{}", helper.name()) {
        anyhow::bail!("expected to be invoked by `cargo gluegun`");
    }

    // Parse the input from stdin
    let stdin = std::io::stdin();
    let input: GlueGunInput<G::Metadata> = serde_json::from_reader(stdin.lock())?;

    // Invoke the user's code
    let mut cx = GenerateCx {
        idl: input.idl,
        dest_crate: input.dest_crate,
    };
    helper.generate(&mut cx, &input.metadata)
}

/// These are the subcommands executed by our system.
/// Your extension should be able to respond to them.
#[derive(Deserialize)]
struct GlueGunInput<M> {
    idl: Idl,
    metadata: M,
    dest_crate: GlueGunDestinationCrate,
}

/// Context provided to the [`GlueGunHelper::generate`][] implementation.
#[derive(Accessors)]
#[accessors(get)]
pub struct GenerateCx {
    /// The IDL from the source crate
    idl: Idl,

    /// Informaton about the destination crate
    dest_crate: GlueGunDestinationCrate,
}

impl GenerateCx {
    /// Create a [`LibraryCrate`][] instance.
    pub fn create_library_crate(&mut self) -> LibraryCrate {
        LibraryCrate::from_args(&self.dest_crate)
    }
}

impl AsRef<GlueGunDestinationCrate> for GlueGunDestinationCrate {
    fn as_ref(&self) -> &GlueGunDestinationCrate {
        self
    }
}

/// The arguments that identify where the crate should be generated.
/// You don't normally need to inspect the fields of this struct,
/// instead just invoke [`LibraryCrate::from_args`](`crate::codegen::LibraryCrate::from_args`).
#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct GlueGunDestinationCrate {
    /// Path at which to create the crate
    pub path: PathBuf,

    /// Name to give the crate; if `None`, then just let `cargo` pick a name.
    pub crate_name: Option<String>,
}
