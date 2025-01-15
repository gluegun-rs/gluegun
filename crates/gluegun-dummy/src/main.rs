use gluegun_core::cli::GlueGunHelper;

pub fn main() -> anyhow::Result<()> {
    gluegun_core::cli::run(GlueGunDummy)
}

struct GlueGunDummy;

impl GlueGunHelper for GlueGunDummy {
    fn name(&self) -> String {
        "dummy".to_string()
    }

    fn generate(
        self,
        idl: gluegun_core::idl::Idl,
        dest_crate: gluegun_core::cli::GlueGunDestinationCrate,
    ) -> anyhow::Result<()> {
        eprintln!("gluegun-dummy: dest_crate = {dest_crate:#?}");
        eprintln!("gluegun-dummy: idl = {idl:#?}");
        Ok(())
    }
}
