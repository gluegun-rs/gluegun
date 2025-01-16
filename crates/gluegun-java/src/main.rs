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

        lib.add_dependency("duchess");

        java_gen::JavaCodeGenerator::new(cx.idl()).generate(lib.add_dir("java_src")?)?;
        rs_gen::RustCodeGenerator::new(cx.idl()).generate(lib.add_dir("src")?)?;

        lib.generate()
    }
}
