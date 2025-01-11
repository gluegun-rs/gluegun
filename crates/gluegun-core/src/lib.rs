/// The gluegun Interface Description Language (IDL).
pub use gluegun_idl as idl;

/// Re-exporting the clap version used for `cli`.
///
/// We recommend you depend on this rather than adding an explicit dependency on `clap`
/// when authoring GlueGun extensions.
pub use clap as clap;

/// Utility structs and things for GlueGun CLIs.
pub mod cli;

/// Utility structs for generating "vaguely well formatted" code.
pub mod codegen;
