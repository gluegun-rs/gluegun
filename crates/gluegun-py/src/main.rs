use gluegun_core::{
    cli::{GenerateCx, GlueGunHelper},
    codegen::LibraryCrate,
};

pub fn main() -> anyhow::Result<()> {
    gluegun_core::cli::run(GlueGunPython)
}

struct GlueGunPython;

impl GlueGunHelper for GlueGunPython {
    type Metadata = ();

    fn name(&self) -> String {
        format!("py")
    }

    fn generate(
        self,
        _cx: &mut GenerateCx,
        _metadata: &Self::Metadata,
        output: &mut LibraryCrate,
    ) -> anyhow::Result<()> {
        output.add_dependency("pyo3").version("0.23");

        Ok(())
    }
}