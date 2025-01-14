use std::path::PathBuf;

use gluegun_core::{
    codegen::{CodeWriter, DirBuilder},
    idl::{
        Enum, Field, FunctionInput, Idl, Item, Method, MethodCategory, Name, QualifiedName, Record,
        Resource, Scalar, SelfKind, Signature, Ty, TypeKind, Variant,
    },
};

use crate::util;

pub(crate) struct RustCodeGenerator<'idl> {
    idl: &'idl Idl,
}

impl<'idl> RustCodeGenerator<'idl> {
    pub(crate) fn new(idl: &'idl Idl) -> Self {
        Self { idl }
    }

    pub(crate) fn generate(mut self, mut dir: DirBuilder<'_>) -> anyhow::Result<()> {
        let mut lib_rs = dir.add_file("src/lib.rs")?;
        for (qname, item) in self.idl.definitions() {
            self.generate_item(&mut lib_rs, qname, item)?;
        }
        Ok(())
    }

    fn generate_item(
        &mut self,
        lib_rs: &mut CodeWriter<'_>,
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
        lib_rs: &mut CodeWriter<'_>,
        qname: &QualifiedName,
        resource: &Resource,
    ) -> Result<(), anyhow::Error> {
        for method in resource.methods() {
            self.generate_method(dir, qname, method)?;
        }
        Ok(())
    }

    fn generate_record(
        &self,
        lib_rs: &mut CodeWriter<'_>,
        qname: &QualifiedName,
        record: &Record,
    ) -> Result<(), anyhow::Error> {
        todo!()
    }

    fn generate_variant(
        &self,
        lib_rs: &mut CodeWriter<'_>,
        qname: &QualifiedName,
        variant: &Variant,
    ) -> Result<(), anyhow::Error> {
        todo!()
    }

    fn generate_enum(
        &self,
        lib_rs: &mut CodeWriter<'_>,
        qname: &QualifiedName,
        an_enum: &Enum,
    ) -> Result<(), anyhow::Error> {
        todo!()
    }

    fn generate_method(
        &self,
        lib_rs: &mut CodeWriter<'_>,
        qname: &QualifiedName,
        method: &Method,
    ) -> anyhow::Result<()> {
    }

    fn generate_native_function(
        &self,
        lib_rs: &mut CodeWriter<'_>,
        class_qname: &QualifiedName,
        method_name: &Name,
        method_category: &MethodCategory,
        signature: &Signature,
    ) -> anyhow::Result<()> {
        write!(lib_rs, "const _: () = {{")?;
        write!(
            lib_rs,
            "#[duchess::java_function({class_dot_name}::{method_name})]",
            class_dot_name = util::class_dot_name(class_qname)
        )?;
        write!(lib_rs, "fn {method_name}(")?;

        match method_category {
            MethodCategory::Constructor => todo!(),
            MethodCategory::BuilderMethod(self_kind) => todo!(),
            MethodCategory::InstanceMethod(self_kind) => todo!(),
            MethodCategory::StaticMethod => todo!(),
            _ => anyhow::bail!("unsupported method category: {method_category:?}"),
        }

        for input in signature.inputs() {
            let name = input.name();
            let ty = input.ty();
        }

        write!(lib_rs, ") -> duchess::Result<> {{")?;
        write!(lib_rs, "}}")?;
        write!(lib_rs, "}}")?;
        Ok(())
    }

    fn rust_parameter_ty(&self, ty: &Ty) -> String {
        match ty.kind() {
            TypeKind::Map { key, value } => todo!(),
            TypeKind::Vec { element } => todo!(),
            TypeKind::Set { element } => todo!(),
            TypeKind::Path => todo!(),
            TypeKind::String => todo!(),
            TypeKind::Option { element } => todo!(),
            TypeKind::Result { ok, err } => todo!(),
            TypeKind::Tuple { elements } => todo!(),
            TypeKind::Scalar(scalar) => scalar.to_string(),
            TypeKind::Future { output } => todo!(),
            TypeKind::Error => todo!(),
            TypeKind::UserType { qname } => todo!(),
            _ => todo!(),
        }
    }
}
