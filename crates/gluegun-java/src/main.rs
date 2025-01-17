use anyhow::Context;
use gluegun_core::cli::{GenerateCx, GlueGunHelper};

mod java_gen;
mod rs_gen;
mod util;

pub fn main() -> anyhow::Result<()> {
    gluegun_core::cli::run(GlueGunJava)
}

struct GlueGunJava;

impl GlueGunHelper for GlueGunJava {
    type Metadata = ();

    fn name(&self) -> String {
        "java".to_string()
    }

    fn generate(self, cx: &mut GenerateCx, &(): &()) -> anyhow::Result<()> {
        let mut lib = cx.create_library_crate();
        lib.add_dependency("duchess", "0.3");

        let java_src_dir = lib.add_dir("java_src").with_context(||format!("adding `java_src` dir"))?;
        java_gen::JavaCodeGenerator::new(cx.idl()).generate(java_src_dir).with_context(|| format!("generaring Java sources"))?;

        let rs_src_dir = lib.add_dir("src").with_context(||format!("adding `java_src` dir"))?;
        rs_gen::RustCodeGenerator::new(cx.idl()).generate(rs_src_dir).with_context(|| format!("generaring Rust sources"))?;

        lib.generate().with_context(|| format!("emitting data to disk"))?;

        Ok(())
    }
}
