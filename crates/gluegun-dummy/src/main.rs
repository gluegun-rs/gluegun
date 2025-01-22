use gluegun_core::{cli::{GenerateCx, GlueGunHelper}, codegen::LibraryCrate};

pub fn main() -> anyhow::Result<()> {
    gluegun_core::cli::run(GlueGunDummy)
}

struct GlueGunDummy;

impl GlueGunHelper for GlueGunDummy {
    type Metadata = serde_json::Value;

    fn name(&self) -> String {
        "dummy".to_string()
    }

    fn generate(self, cx: &mut GenerateCx, metadata: &Self::Metadata, output: &mut LibraryCrate) -> anyhow::Result<()> {
        let mut f = output.add_file("README.md")?;
        write!(f, "# Dummy GlueGun crate generator")?;
        write!(f, "")?;
        write!(f, "This demo just exists to show you how to implement a generator.")?;
        write!(f, "")?;
        write!(f, "## Input metadata")?;
        write!(f, "```")?;
        write!(f, "metadata = {:#?}", metadata)?;
        write!(f, "```")?;
        write!(f, "")?;
        write!(f, "## Input IDL")?;
        write!(f, "```")?;
        write!(f, "idl = {:#?}", cx.idl())?;
        write!(f, "```")?;
        Ok(())
    }
}
