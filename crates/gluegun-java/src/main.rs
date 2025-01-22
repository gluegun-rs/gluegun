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

    fn generate(self, cx: &mut GenerateCx, &(): &()) -> anyhow::Result<()> {
        let mut lib = cx.create_library_crate();
        lib.add_dependency(cx.idl().crate_name().text()).path(cx.idl().crate_path());
        lib.add_dependency("duchess").version("0.3");
        lib.add_dependency("anyhow").version("1").build();
        self.add_gluegun_java_util(&mut lib)?;

        let java_src_dir = lib
            .add_dir("java_src")
            .with_context(|| format!("adding `java_src` dir"))?;
        java_gen::JavaCodeGenerator::new(cx.idl())
            .generate(java_src_dir)
            .with_context(|| format!("generaring Java sources"))?;

        rs_gen::RustCodeGenerator::new(cx.idl())
            .generate(&mut lib)
            .with_context(|| format!("generaring Rust sources"))?;

        lib.generate()
            .with_context(|| format!("emitting data to disk"))?;

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
