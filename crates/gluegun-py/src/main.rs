use gluegun_core::{
    cli::{GenerateCx, GlueGunHelper},
    codegen::LibraryCrate,
};
use rs_gen::RustCodeGenerator;

pub fn main() -> anyhow::Result<()> {
    gluegun_core::cli::run(GlueGunPython)
}

mod rs_gen;

struct GlueGunPython;

impl GlueGunHelper for GlueGunPython {
    type Metadata = ();

    fn name(&self) -> String {
        format!("py")
    }

    fn generate(
        self,
        cx: &mut GenerateCx,
        _metadata: &Self::Metadata,
        output: &mut LibraryCrate,
    ) -> anyhow::Result<()> {
        let features = RustCodeGenerator::new(cx.idl()).generate(output)?;

        let mut dep = output.add_dependency("pyo3").version("0.23");
        for feature in features {
            dep = dep.feature(feature);
        }

        Ok(())
    }
}