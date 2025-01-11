use std::path::PathBuf;

use gluegun_core::{
    codegen::{CodeWriter, DirBuilder},
    idl::{
        Enum, Field, FunctionInput, Idl, Item, Method, MethodCategory, QualifiedName, Record,
        Resource, Scalar, SelfKind, Ty, TypeKind, Variant,
    },
};

pub(crate) struct RustCodeGenerator<'idl> {
    idl: &'idl Idl,
}

impl<'idl> RustCodeGenerator<'idl> {
    pub(crate) fn new(idl: &'idl Idl) -> Self {
        Self { idl }
    }

    pub(crate) fn generate(mut self, mut dir: DirBuilder<'_>) -> anyhow::Result<()> {
        for (qname, item) in self.idl.definitions() {
            self.generate_item(&mut dir, qname, item)?;
        }
        Ok(())
    }

    fn generate_item(
        &mut self,
        dir: &mut DirBuilder<'_>,
        qname: &QualifiedName,
        item: &Item,
    ) -> anyhow::Result<()> {
        match item {
            Item::Resource(resource) => self.generate_resource(dir, qname, resource),
            Item::Record(record) => self.generate_record(dir, qname, record),
            Item::Variant(variant) => self.generate_variant(dir, qname, variant),
            Item::Enum(an_enum) => self.generate_enum(dir, qname, an_enum),
            Item::Function(_) => {
                // Skip functions for now. We will collect them and generate them as static methods.
                Ok(())
            }
            _ => anyhow::bail!("unsupported item: "),
        }
    }

    fn generate_resource(
        &self,
        dir: &mut DirBuilder<'_>,
        qname: &QualifiedName,
        resource: &Resource,
    ) -> Result<(), anyhow::Error> {
        todo!()
    }

    fn generate_record(
        &self,
        dir: &mut DirBuilder<'_>,
        qname: &QualifiedName,
        record: &Record,
    ) -> Result<(), anyhow::Error> {
        todo!()
    }

    fn generate_variant(
        &self,
        dir: &mut DirBuilder<'_>,
        qname: &QualifiedName,
        variant: &Variant,
    ) -> Result<(), anyhow::Error> {
        todo!()
    }

    fn generate_enum(
        &self,
        dir: &mut DirBuilder<'_>,
        qname: &QualifiedName,
        an_enum: &Enum,
    ) -> Result<(), anyhow::Error> {
        todo!()
    }
}
