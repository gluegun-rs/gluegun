use gluegun_core::{
    codegen::LibraryCrate,
    idl::Idl,
};

pub(crate) struct RustCodeGenerator<'idl> {
    #[expect(dead_code)]
    idl: &'idl Idl,
    features: Vec<&'static str>,
}

impl<'idl> RustCodeGenerator<'idl> {
    pub(crate) fn new(idl: &'idl Idl) -> Self {
        Self {
            idl,
            features: Default::default(),
        }
    }

    pub(crate) fn generate(mut self, lib: &mut LibraryCrate) -> anyhow::Result<Vec<&'static str>> {
        self.generate_lib_rs(lib)?;
        Ok(self.features)
    }

    fn generate_lib_rs(&mut self, lib: &mut LibraryCrate) -> anyhow::Result<()> {
        let mut lib_rs = lib.add_file("src/lib.rs")?;

        write!(lib_rs, "#![allow(non_snake_case)]")?; // FIXME: bug in duchess

        Ok(())
    }
}
