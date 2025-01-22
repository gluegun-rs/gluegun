use anyhow::Context;
use camino::Utf8PathBuf;
use gluegun_core::{
    cli::{GenerateCx, GlueGunHelper},
    codegen::LibraryCrate,
};

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

    fn generate(self, cx: &mut GenerateCx, &(): &(), output: &mut LibraryCrate) -> anyhow::Result<()> {
        output.add_dependency(cx.idl().crate_name().text()).path(cx.idl().crate_path());
        output.add_dependency("duchess").version("0.3");
        output.add_dependency("anyhow").version("1").build();
        self.add_gluegun_java_util(output)?;

        let java_src_dir = output
            .add_dir("java_src")
            .with_context(|| format!("adding `java_src` dir"))?;
        java_gen::JavaCodeGenerator::new(cx.idl())
            .generate(java_src_dir)
            .with_context(|| format!("generaring Java sources"))?;

        rs_gen::RustCodeGenerator::new(cx.idl())
            .generate(output)
            .with_context(|| format!("generaring Rust sources"))?;

        Ok(())
    }
}

impl GlueGunJava {
    fn add_gluegun_java_util(&self, lib: &mut LibraryCrate) -> anyhow::Result<()> {
        let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") else {
            anyhow::bail!("no CARGO_MANIFEST_DIR variable set")
        };
        let mut manifest_path = Utf8PathBuf::from(manifest_dir);
        manifest_path.pop();
        manifest_path.push("gluegun-java-util");

        // FIXME: we should eventually get this from crates.io, at least when not testing
        lib.add_dependency("gluegun_java_util")
            .build()
            .path(manifest_path);

        Ok(())
    }
}
