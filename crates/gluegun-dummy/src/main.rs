use gluegun_core::cli::{GenerateCx, GlueGunHelper};

pub fn main() -> anyhow::Result<()> {
    gluegun_core::cli::run(GlueGunDummy)
}

struct GlueGunDummy;

impl GlueGunHelper for GlueGunDummy {
    type Metadata = serde_json::Value;

    fn name(&self) -> String {
        "dummy".to_string()
    }

    fn generate(self, cx: &mut GenerateCx, metadata: &Self::Metadata) -> anyhow::Result<()> {
        eprintln!("gluegun-dummy: dest_crate = {:#?}", cx.dest_crate());
        eprintln!("gluegun-dummy: metadata = {:#?}", metadata);
        eprintln!("gluegun-dummy: idl = {:#?}", cx.idl());
        Ok(())
    }
}
