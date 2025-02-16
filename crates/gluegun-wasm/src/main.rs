use gluegun_core::{
    cli::{GenerateCx, GlueGunHelper},
    codegen::LibraryCrate,
};
use rs_gen::RustCodeGenerator;

pub fn main() -> anyhow::Result<()> {
    gluegun_core::cli::run(GlueGunWasm)
}

mod rs_gen;

struct GlueGunWasm;

impl GlueGunHelper for GlueGunWasm {
    type Metadata = ();

    fn name(&self) -> String {
        format!("wasm")
    }

    fn generate(
        self,
        cx: &mut GenerateCx,
        _metadata: &Self::Metadata,
        output: &mut LibraryCrate,
    ) -> anyhow::Result<()> {
        output.require_helper_command("cargo-component").or_run_cargo_install("cargo-component");

        RustCodeGenerator::new(cx.idl()).generate(output)?;
        output.add_dependency("wasm-bindgen").version("0.2");

        Ok(())
    }
}