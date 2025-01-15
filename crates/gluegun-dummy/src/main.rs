use gluegun_core::cli::{GenerateCx, GlueGunHelper};

pub fn main() -> anyhow::Result<()> {
    gluegun_core::cli::run(GlueGunDummy)
}

struct GlueGunDummy;

impl GlueGunHelper for GlueGunDummy {
    fn name(&self) -> String {
        "dummy".to_string()
    }

    fn generate(self, cx: &mut GenerateCx) -> anyhow::Result<()> {
        eprintln!("gluegun-dummy: dest_crate = {:#?}", cx.dest_crate());
        eprintln!("gluegun-dummy: idl = {:#?}", cx.idl());
        Ok(())
    }
}
