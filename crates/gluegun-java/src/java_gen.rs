use std::{collections::BTreeMap, path::PathBuf};

use gluegun_core::{
    codegen::{CodeWriter, DirBuilder},
    idl::{
        Enum, Field, Function, FunctionInput, Idl, Item, Method, MethodCategory, Name,
        QualifiedName, Record, Resource, Scalar, SelfKind, Signature, Ty, TypeKind, Variant,
    },
};

use crate::util;

pub(crate) struct JavaCodeGenerator<'idl> {
    idl: &'idl Idl,
}

impl<'idl> JavaCodeGenerator<'idl> {
    pub(crate) fn new(idl: &'idl Idl) -> Self {
        Self { idl }
    }

    pub(crate) fn generate(mut self, mut dir: DirBuilder<'_>) -> anyhow::Result<()> {
        let mut functions: BTreeMap<QualifiedName, Vec<&'idl Function>> = Default::default();

        for (qname, item) in self.idl.definitions() {
            self.generate_item(&mut dir, qname, item, &mut functions)?;
        }

        for (module_qname, functions) in &functions {
            self.generate_functions(&mut dir, module_qname, functions)?;
        }

        Ok(())
    }

    fn generate_java_file(
        &mut self,
        dir: &mut DirBuilder<'_>,
        java_type: &str,
        qname: &QualifiedName,
        body: impl FnOnce(&mut Self, &mut CodeWriter<'_>) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        let mut file = dir.add_file(util::class_file_name(qname))?;
        let (package, name) = qname.split_module_name();
        let package = package.camel_case().dotted();
        write!(file, "package {package}")?;
        write!(file, "")?;
        write!(file, "public {java_type} {name} {{",)?;

        body(self, &mut file)?;

        write!(file, "}}")?;

        Ok(())
    }

    fn generate_item(
        &mut self,
        dir: &mut DirBuilder<'_>,
        qname: &QualifiedName,
        item: &'idl Item,
        functions: &mut BTreeMap<QualifiedName, Vec<&'idl Function>>,
    ) -> anyhow::Result<()> {
        match item {
            Item::Resource(resource) => self.generate_resource(dir, qname, resource),
            Item::Record(record) => self.generate_record(dir, qname, record),
            Item::Variant(variant) => self.generate_variant(dir, qname, variant),
            Item::Enum(an_enum) => self.generate_enum(dir, qname, an_enum),
            Item::Function(function) => {
                // Collect functons, grouped by module. We will generate them later.
                functions
                    .entry(qname.module_name())
                    .or_insert(Default::default())
                    .push(function);
                Ok(())
            }
            _ => anyhow::bail!("unsupported item: "),
        }
    }

    fn generate_functions(
        &mut self,
        dir: &mut DirBuilder<'_>,
        module_qname: &QualifiedName,
        functions: &[&Function],
    ) -> anyhow::Result<()> {
        let functions_class = module_qname.join("Functions");
        self.generate_java_file(dir, "class", &functions_class, |this, file| {
            for function in functions {
                this.generate_regular_method(file, None, function.name(), function.signature())?;
            }
            Ok(())
        })
    }

    fn generate_resource(
        &mut self,
        dir: &mut DirBuilder<'_>,
        qname: &QualifiedName,
        resource: &Resource,
    ) -> anyhow::Result<()> {
        self.generate_java_file(dir, "class", qname, |this, file| {
            write!(file, "private long pointer;")?;
            this.generate_methods(file, resource.methods())?;
            Ok(())
        })
    }

    fn generate_record(
        &mut self,
        dir: &mut DirBuilder<'_>,
        qname: &QualifiedName,
        record: &Record,
    ) -> anyhow::Result<()> {
        self.generate_java_file(dir, "class", qname, |this, file| {
            this.generate_fields(file, record.fields())?;

            // FIXME: make a constructor?

            this.generate_methods(file, record.methods())?;
            Ok(())
        })
    }

    fn generate_variant(
        &mut self,
        dir: &mut DirBuilder<'_>,
        qname: &QualifiedName,
        variant: &Variant,
    ) -> anyhow::Result<()> {
        self.generate_java_file(dir, "abstract class", qname, |this, file| {
            this.generate_methods(file, variant.methods())?;
            Ok(())
        })?;

        for variant_arm in variant.arms() {
            let variant_qname = qname.module_name().join(variant_arm.name());
            self.generate_java_file(dir, "abstract class", &variant_qname, |this, file| {
                this.generate_fields(file, variant_arm.fields())?;
                Ok(())
            })?;
        }

        Ok(())
    }

    fn generate_enum(
        &mut self,
        dir: &mut DirBuilder<'_>,
        qname: &QualifiedName,
        an_enum: &Enum,
    ) -> anyhow::Result<()> {
        self.generate_java_file(dir, "enum", qname, |this, file| {
            for arm in an_enum.arms() {
                write!(file, "{},", arm.name().upper_camel_case())?;
            }
            this.generate_methods(file, an_enum.methods())?;
            Ok(())
        })
    }

    fn generate_fields(&self, file: &mut CodeWriter<'_>, fields: &[Field]) -> anyhow::Result<()> {
        for field in fields {
            write!(
                file,
                "public {ty} {name};",
                ty = self.write_ty(field.ty())?,
                name = field.name().camel_case()
            )?;
        }
        Ok(())
    }

    fn generate_methods(
        &self,
        file: &mut CodeWriter<'_>,
        methods: &[Method],
    ) -> anyhow::Result<()> {
        for method in methods {
            self.generate_method(file, method)?;
        }
        Ok(())
    }

    fn generate_method(&self, file: &mut CodeWriter<'_>, method: &Method) -> anyhow::Result<()> {
        write!(file, "")?;

        match method.category() {
            MethodCategory::Constructor => todo!(),

            MethodCategory::InstanceMethod(self_kind)
            | MethodCategory::BuilderMethod(self_kind) => self.generate_regular_method(
                file,
                Some(self_kind),
                method.name(),
                method.signature(),
            ),

            MethodCategory::StaticMethod => {
                self.generate_regular_method(file, None, method.name(), method.signature())
            }

            _ => anyhow::bail!("unsupported method category: `{:?}`", method.category()),
        }
    }

    fn generate_regular_method(
        &self,
        file: &mut CodeWriter<'_>,
        self_kind: Option<&SelfKind>,
        name: &Name,
        signature: &Signature,
    ) -> anyhow::Result<()> {
        let native_name = self.generate_native_counterpart(file, self_kind, name, signature)?;

        write!(file, "")?;

        let static_kw = if self_kind.is_none() { "static" } else { "" };

        let return_ty = signature.output_ty().main_ty();
        write!(
            file,
            "public {static_kw} {ret} {name}(",
            ret = self.write_ty(return_ty)?,
            name = name
        )?;
        self.generate_function_inputs(file, signature.inputs())?;
        write!(file, ") {{")?;
        write!(file, "return {native_name}(")?;
        for input in signature.inputs() {
            write!(file, "{input_name},", input_name = input.name())?;
        }
        write!(file, ");");
        write!(file, "}}")?;

        Ok(())
    }

    fn generate_function_inputs(
        &self,
        file: &mut CodeWriter<'_>,
        inputs: &[FunctionInput],
    ) -> anyhow::Result<()> {
        for input in inputs {
            write!(
                file,
                "{ty} {name},",
                ty = self.write_ty(input.ty())?,
                name = input.name()
            )?;
        }
        Ok(())
    }

    fn generate_native_counterpart(
        &self,
        file: &mut CodeWriter<'_>,
        self_kind: Option<&SelfKind>,
        name: &Name,
        signature: &Signature,
    ) -> anyhow::Result<String> {
        let native_name = format!("native${name}");

        write!(file, "")?;

        let static_kw = if self_kind.is_none() { "static" } else { "" };

        let return_ty = signature.output_ty().main_ty();
        write!(
            file,
            "public {static_kw} native {ret} {native_name}(",
            ret = self.write_ty(return_ty)?,
        )?;
        self.generate_function_inputs(file, signature.inputs())?;
        write!(file, ");")?;

        Ok(native_name)
    }

    fn write_ty(&self, ty: &Ty) -> anyhow::Result<String> {
        match ty.kind() {
            TypeKind::Scalar(scalar) => match scalar {
                Scalar::Char => Ok("int".to_string()),
                Scalar::Boolean => Ok("boolean".to_string()),
                Scalar::I8 | Scalar::U8 => Ok("byte".to_string()),
                Scalar::I16 | Scalar::U16 => Ok("short".to_string()),
                Scalar::I32 | Scalar::U32 => Ok("int".to_string()),
                Scalar::I64 | Scalar::U64 => Ok("long".to_string()),
                Scalar::F32 => Ok("float".to_string()),
                Scalar::F64 => Ok("double".to_string()),
                _ => anyhow::bail!("unsupported scalar type: `{scalar}`"),
            },
            _ => self.write_objectified_ty(ty),
        }
    }

    fn write_objectified_ty(&self, ty: &Ty) -> anyhow::Result<String> {
        match ty.kind() {
            TypeKind::Map { key, value } => Ok(format!(
                "java.util.Map<{K}, {V}>",
                K = self.write_objectified_ty(key)?,
                V = self.write_objectified_ty(value)?,
            )),
            TypeKind::Vec { element } => Ok(format!(
                "java.util.List<{E}>",
                E = self.write_objectified_ty(element)?,
            )),
            TypeKind::Set { element } => Ok(format!(
                "java.util.Set<{E}>",
                E = self.write_objectified_ty(element)?,
            )),
            TypeKind::Path => Ok("String".to_string()),
            TypeKind::String => Ok("String".to_string()),
            TypeKind::Option { element } => self.write_objectified_ty(element),

            // This is pretty bad, but the expectation is that people don't pass `Result`
            // around most of the time, they should up in return types where they
            // are specially handled.
            TypeKind::Result { ok: _, err: _ } => Ok("Object".to_string()),

            // FIXME: I think tuple arguments should be flattened,
            // since Java has no native concept of them.
            TypeKind::Tuple { elements: _ } => Ok("Object[]".to_string()),

            TypeKind::Scalar(scalar) => match scalar {
                Scalar::Char => Ok("Integer".to_string()),
                Scalar::Boolean => Ok("Boolean".to_string()),
                Scalar::I8 | Scalar::U8 => Ok("Byte".to_string()),
                Scalar::I16 | Scalar::U16 => Ok("Short".to_string()),
                Scalar::I32 | Scalar::U32 => Ok("Integer".to_string()),
                Scalar::I64 | Scalar::U64 => Ok("Long".to_string()),
                Scalar::F32 => Ok("Float".to_string()),
                Scalar::F64 => Ok("Double".to_string()),
                _ => anyhow::bail!("unsupported scalar type: `{scalar}`"),
            },
            TypeKind::Future { output } => Ok(format!(
                "java.util.concurrent.Future<{V}>",
                V = self.write_objectified_ty(output)?
            )),
            TypeKind::Error => todo!(),
            TypeKind::UserType { qname } => Ok(util::class_dot_name(qname)),
            _ => anyhow::bail!("unsupported type: `{ty}`"),
        }
    }
}
