use gluegun_core::{
    clap::{self, Parser},
    cli::GlueGunCommand,
    codegen::LibraryCrate,
};

mod java_gen;
mod rs_gen;

/// A simple Cli you can use for your own parser.
#[derive(clap::Parser)]
pub struct GlueGunJava {
    #[command(subcommand)]
    command: GlueGunCommand,
}

pub fn main() -> anyhow::Result<()> {
    let cli = GlueGunJava::parse();

    let GlueGunCommand::Generate { idl, crate_args } = cli.command;

    let mut lib = LibraryCrate::from_args(crate_args);

    lib.add_dependency("duchess");

    java_gen::JavaCodeGenerator::new(&idl).generate(lib.add_dir("java_src")?)?;

    lib.generate()
}
