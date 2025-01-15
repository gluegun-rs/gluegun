use gluegun_core::{cli::GlueGunHelper, codegen::LibraryCrate};

mod java_gen;
mod rs_gen;
mod util;

pub fn main() -> anyhow::Result<()> {
    gluegun_core::cli::run(GlueGunJava)
}

struct GlueGunJava;

impl GlueGunHelper for GlueGunJava {
    fn name(&self) -> String {
        "java".to_string()
    }

    fn generate(
        self,
        idl: gluegun_core::idl::Idl,
        crate_args: gluegun_core::cli::GlueGunDestinationCrate,
    ) -> anyhow::Result<()> {
        let mut lib = LibraryCrate::from_args(crate_args);

        lib.add_dependency("duchess");

        java_gen::JavaCodeGenerator::new(&idl).generate(lib.add_dir("java_src")?)?;

        lib.generate()
    }
}
