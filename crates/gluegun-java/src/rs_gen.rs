use std::collections::BTreeMap;

use gluegun_core::{
    codegen::{CodeWriter, LibraryCrate},
    idl::{
        Enum, FunctionInput, FunctionOutput, Idl, Item, Method, MethodCategory, Name, QualifiedName, Record, Resource, RustReprKind, Signature, Ty, TypeKind, Variant
    },
};

use crate::util::{self, JavaQName};

pub(crate) struct RustCodeGenerator<'idl> {
    idl: &'idl Idl,
}

impl<'idl> RustCodeGenerator<'idl> {
    pub(crate) fn new(idl: &'idl Idl) -> Self {
        Self { idl }
    }

    pub(crate) fn generate(mut self, lib: &mut LibraryCrate) -> anyhow::Result<()> {
        let mut lib_rs = lib.add_file("src/lib.rs")?;

        write!(lib_rs, "#![allow(non_snake_case)]")?; // FIXME: bug in duchess

        self.generate_java_classes(&mut lib_rs)?;

        for (qname, item) in self.idl.definitions() {
            self.generate_item(&mut lib_rs, qname, item)?;
        }
        std::mem::drop(lib_rs);

        let mut build_rs = lib.add_file("build.rs")?;
        self.generate_build_rs(&mut build_rs)?;
        std::mem::drop(build_rs);

        Ok(())
    }

    fn generate_build_rs(&mut self, build_rs: &mut CodeWriter<'_>) -> anyhow::Result<()> {
        write!(
            build_rs,
            "fn main() -> anyhow::Result<()> {{ gluegun_java_util::main() }}"
        )?;
        Ok(())
    }

    fn generate_java_classes(&self, lib_rs: &mut CodeWriter<'_>) -> anyhow::Result<()> {
        let mut map = BTreeMap::default();

        for (qname, item) in self.idl.definitions() {
            let java_qname = self.java_class(qname, item)?;
            map.entry(java_qname).or_insert(vec![]).push(item);
        }

        for (java_qname, _items) in map {
            // FIXME: Do we want to generate items or Java-based members in any of these classes?
            
            write!(lib_rs, "duchess::java_package! {{")?;
            write!(lib_rs, "package {};", java_qname.package.dotted())?;
            write!(lib_rs, "class {} {{ }}", java_qname.class_name)?;
            write!(lib_rs, "}}")?;
        }

        Ok(())
    }

    fn java_class(&self, qname: &QualifiedName, item: &Item) -> anyhow::Result<JavaQName> {
        match item {
            Item::Resource(_) | Item::Record(_) | Item::Variant(_) | Item::Enum(_) => {
                Ok(util::class_package_and_name(qname))
            }
            Item::Function(_) => {
                let package = qname.module_name().camel_case();
                Ok(JavaQName {
                    package,
                    class_name: Name::from("Functions"),
                })
            }
            _ => {
                anyhow::bail!("unsupported item: {item:?}")
            }
        }
    }

    fn generate_item(
        &mut self,
        lib_rs: &mut CodeWriter<'_>,
        qname: &QualifiedName,
        item: &Item,
    ) -> anyhow::Result<()> {
        match item {
            Item::Resource(resource) => self.generate_resource(lib_rs, qname, resource),
            Item::Record(record) => self.generate_record(lib_rs, qname, record),
            Item::Variant(variant) => self.generate_variant(lib_rs, qname, variant),
            Item::Enum(an_enum) => self.generate_enum(lib_rs, qname, an_enum),
            Item::Function(f) => {
                let module_name = qname.module_name();
                let java_qname = module_name.join("Functions");
                self.generate_native_function(
                    lib_rs,
                    &module_name,
                    &java_qname,
                    f.name(),
                    &MethodCategory::StaticMethod,
                    f.signature(),
                )?;
                Ok(())
            }
            _ => anyhow::bail!("unsupported item: {item:?}"),
        }
    }

    fn generate_resource(
        &self,
        lib_rs: &mut CodeWriter<'_>,
        qname: &QualifiedName,
        resource: &Resource,
    ) -> Result<(), anyhow::Error> {
        for method in resource.methods() {
            self.generate_method(lib_rs, qname, method)?;
        }
        Ok(())
    }

    fn generate_record(
        &self,
        lib_rs: &mut CodeWriter<'_>,
        qname: &QualifiedName,
        record: &Record,
    ) -> Result<(), anyhow::Error> {
        for method in record.methods() {
            self.generate_method(lib_rs, qname, method)?;
        }
        Ok(())
    }

    fn generate_variant(
        &self,
        lib_rs: &mut CodeWriter<'_>,
        qname: &QualifiedName,
        variant: &Variant,
    ) -> Result<(), anyhow::Error> {
        for method in variant.methods() {
            self.generate_method(lib_rs, qname, method)?;
        }
        Ok(())
    }

    fn generate_enum(
        &self,
        lib_rs: &mut CodeWriter<'_>,
        qname: &QualifiedName,
        an_enum: &Enum,
    ) -> Result<(), anyhow::Error> {
        for method in an_enum.methods() {
            self.generate_method(lib_rs, qname, method)?;
        }
        Ok(())
    }

    fn generate_method(
        &self,
        lib_rs: &mut CodeWriter<'_>,
        qname: &QualifiedName,
        method: &Method,
    ) -> anyhow::Result<()> {
        self.generate_native_function(
            lib_rs,
            qname,
            qname,
            method.name(),
            method.category(),
            method.signature(),
        )
    }

    /// Generate a native function definition that will be the backing function for a Java method.
    ///
    /// # Parameters
    ///
    /// * `lib_rs`, write-stream for the `lib.rs` file
    /// * `rust_qname`, qname of the `Resource` type or, for free functions, the containing module
    /// * `java_qname`, the qname of the Java class containing the method; often the same as `rust_qname` but (e.g. for free functions) not always
    /// * `fn_name`, the name of the method/function
    /// * `method_category`, the category of method (e.g., static etc). Static for free functions.
    /// * `signature`, types of inputs/outputs apart from `self`
    fn generate_native_function(
        &self,
        lib_rs: &mut CodeWriter<'_>,
        rust_qname: &QualifiedName,
        java_qname: &QualifiedName,
        fn_name: &Name,
        method_category: &MethodCategory,
        signature: &Signature,
    ) -> anyhow::Result<()> {
        write!(lib_rs, "const _: () = {{")?;

        write!(lib_rs, "use duchess::java;")?; // FIXME: duchess bug, this should not be needed

        write!(
            lib_rs,
            "#[duchess::java_function({class_dot_name}::{fn_name})]",
            class_dot_name = util::class_dot_name(java_qname)
        )?;
        write!(lib_rs, "fn {fn_name}(")?;

        match method_category {
            MethodCategory::Constructor => {}
            MethodCategory::BuilderMethod(_self_kind)
            | MethodCategory::InstanceMethod(_self_kind) => {
                write!(lib_rs, "_self: &duchess::JavaObject")?; // FIXME
            }
            MethodCategory::StaticMethod => {}
            _ => anyhow::bail!("unsupported method category: {method_category:?}"),
        }

        for input in signature.inputs() {
            let name = input.name();
            let ty = input.ty();
            write!(lib_rs, "{name}: {ty},", ty = self.java_parameter_ty(ty)?)?;
        }

        let output = signature.output_ty();
        write!(lib_rs, ") -> {} {{", self.rust_return_ty(output))?;

        self.generate_fn_body(lib_rs, fn_name, rust_qname, signature, output)?;

        write!(lib_rs, "}}")?;
        write!(lib_rs, "}};")?;
        Ok(())
    }

    fn rust_return_ty(&self, output: &FunctionOutput) -> String {
        let main_ty = output.main_ty();
        let main_str = self.rust_owned_ty(main_ty);

        let Some(_err_ty) = output.error_ty() else {
            return format!("duchess::Result<{main_str}>");
        };

        // FIXME: fix the `err_ty` handling

        format!("duchess::Result<{main_str}>")
    }

    /// Return the type we should expect to receive from Java.
    fn java_parameter_ty(&self, ty: &Ty) -> anyhow::Result<String> {
        // FIXME: Duchess's macro has bugs but these work more-or-less for now.
        match ty.kind() {
            TypeKind::Map { key, value } => {
                Ok(format!(
                    "&duchess::java::util::Map<{}, {}>",
                    self.java_parameter_ty(key)?,
                    self.java_parameter_ty(value)?,
                ))
            }
            TypeKind::Vec { element } => {
                Ok(format!("&duchess::java::util::List<{}>", self.java_parameter_ty(element)?))
            }
            TypeKind::Set { element } => {
                Ok(format!("&duchess::java::util::Set<{}>", self.java_parameter_ty(element)?))
            }
            TypeKind::Path => {
                Ok(format!("&duchess::java::lang::String"))
            }
            TypeKind::String => {
                Ok(format!("&duchess::java::lang::String"))
            }
            TypeKind::Option { element } => {
                // in practice everything in Java is nullable...
                self.java_parameter_ty(element)
            }
            TypeKind::Result { ok: _, err: _ } => {
                Ok(format!("&duchess::java::lang::Object"))
            }
            TypeKind::Tuple { elements: _ } => {
                Ok(format!(
                    "&[&duchess::lang::Object]",
                ))
            }
            TypeKind::Scalar(scalar) => Ok(scalar.to_string()),
            TypeKind::Future { output: _ } => todo!(),
            TypeKind::Error => {
                Ok(format!("&duchess::java::lang::Exception"))
            }
            TypeKind::UserType { qname: _ } => {
                anyhow::bail!("user types not supported currently")
            }
            _ => todo!(),
        }
    }

    fn rust_owned_ty(&self, ty: &Ty) -> String {
        // FIXME: We really ought to be taking the Rust representation into account.
        match ty.kind() {
            TypeKind::Map { key, value } => {
                format!(
                    "HashMap<{}, {}>",
                    self.rust_owned_ty(key),
                    self.rust_owned_ty(value),
                )
            }
            TypeKind::Vec { element } => {
                format!("Vec<{}>", self.rust_owned_ty(element))
            }
            TypeKind::Set { element } => {
                format!("HashSet<{}>", self.rust_owned_ty(element),)
            }
            TypeKind::Path => {
                format!("PathBuf")
            }
            TypeKind::String => {
                format!("String")
            }
            TypeKind::Option { element } => {
                format!("Option<{}>", self.rust_owned_ty(element))
            }
            TypeKind::Result { ok, err } => {
                format!(
                    "Result<{}, {}>",
                    self.rust_owned_ty(ok),
                    self.rust_owned_ty(err)
                )
            }
            TypeKind::Tuple { elements } => {
                format!(
                    "({})",
                    elements
                        .iter()
                        .map(|ty| self.rust_owned_ty(ty))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            TypeKind::Scalar(scalar) => scalar.to_string(),
            TypeKind::Future { output: _ } => todo!(),
            TypeKind::Error => format!("anyhow::Error"),
            TypeKind::UserType { qname } => qname.colon_colon(),
            _ => todo!(),
        }
    }

    /// Generate a call to the underlying Rust function.
    /// 
    /// Adapt from Java arguments to the Rust argument.
    /// 
    /// If the result is an error, use `?` to adapt it.
    fn generate_fn_body(
        &self,
        lib_rs: &mut CodeWriter<'_>,
        fn_name: &Name,
        rust_qname: &QualifiedName,
        signature: &Signature,
        output: &FunctionOutput,
    ) -> anyhow::Result<()> {
        for input in signature.inputs() {
            let name = input.name();
            let ty = input.ty();
            write!(
                lib_rs, 
                "let {name}: {ty} = duchess::JvmOp::execute({name})?;",
                ty = self.rust_owned_ty(ty),
            )?;
        }

        write!(lib_rs, "Ok({m}::{fn_name}(", m = rust_qname.colon_colon())?;

        for input in signature.inputs() {
            self.generate_rust_argument(lib_rs, input)?;
        }

        let qmark = if output.error_ty().is_some() {
            "?"
        } else {
            ""
        };

        write!(lib_rs, "){qmark})")?;
        Ok(())
    }

    fn generate_rust_argument(&self,
        lib_rs: &mut CodeWriter<'_>,
        input: &FunctionInput,
    ) -> anyhow::Result<()> {
        let name = input.name();

        let rust_repr = input.ty().rust_repr();
        match rust_repr.kind() {
            RustReprKind::Ref(_) => {
                write!(lib_rs, "&{name},")?;
            }
            _ => {
                write!(lib_rs, "{name},")?;
            }
        }

        Ok(())
    }
}
