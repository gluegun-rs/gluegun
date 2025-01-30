use gluegun_core::{
    codegen::{CodeWriter, LibraryCrate},
    idl::{
        Function, FunctionInput, FutureRepr, Idl, Item, MapSetRepr, OptionRepr,
        PathRepr, QualifiedName, RefdTy, ResultRepr, StringRepr, TupleRepr, Ty, TypeKind,
        VecRepr,
    },
};

pub(crate) struct RustCodeGenerator<'idl> {
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

        self.generate_python_items(&mut lib_rs)?;

        Ok(())
    }

    fn generate_python_items(&mut self, lib_rs: &mut CodeWriter<'_>) -> anyhow::Result<()> {
        for (qname, item) in self.idl.definitions() {
            self.generate_python_item(lib_rs, qname, item)?;
        }

        Ok(())
    }

    fn generate_python_item(
        &mut self,
        lib_rs: &mut CodeWriter<'_>,
        qname: &QualifiedName,
        item: &Item,
    ) -> anyhow::Result<()> {
        match item {
            Item::Resource(_resource) => {
                todo!()
            }
            Item::Enum(_enum_) => {
                todo!()
            }
            Item::Record(_record) => {
                todo!()
            }
            Item::Variant(_variant) => {
                todo!()
            }
            Item::Function(function) => {
                self.generate_python_function(lib_rs, qname, function)?;
            }
            _ => todo!(),
        }

        Ok(())
    }

    fn generate_python_function(
        &mut self,
        lib_rs: &mut CodeWriter<'_>,
        qname: &QualifiedName,
        function: &Function,
    ) -> anyhow::Result<()> {
        // Write function definition with #[pyfunction] attribute
        write!(lib_rs, "#[pyo3::pyfunction]")?;
        write!(lib_rs, "fn {}(", function.name())?;

        // Write function parameters
        for input in function.signature().inputs() {
            let input_type = self.rust_argument_ty(input)?;
            write!(lib_rs, "{}: {},", input.name(), input_type)?;
        }

        // Write return type if function has output
        let main_ty =
            self.generic_ty(function.signature().output_ty().main_ty().owned_or_err()?)?;
        write!(lib_rs, ") -> {main_ty} {{")?;

        // Write function body. Arguments will a suitable Rust owned type
        // but they may need to be borrowed or adapted to fit what the callee function
        // expects.
        write!(lib_rs, "{}(", qname.colon_colon())?;
        for input in function.signature().inputs() {
            let name = input.name();
            match input.refd_ty() {
                RefdTy::Owned(..) => write!(lib_rs, "&{name}, ")?,
                RefdTy::Ref(..) => write!(lib_rs, "&{name}, ")?,
            }
        }
        write!(lib_rs, ")")?;
        write!(lib_rs, "}}")?;

        Ok(())
    }

    /// Invoked with a function argument. Returns a pair `(ty, expr)` of a
    /// Rust type (`ty`) that will be provided by pyo3 and an `expr` that will adapt
    /// this value to what the wrapped Rust function requires.
    ///
    /// So, for example, imagine that we have an input like `Map { ..., repr: HashMap }`.
    /// This means that (a) the Python code will provide a map; (b) the Rust code expects a `HashMap`.
    /// We need to pick a good type to use with pyo3 to make that efficient. In this case, we ought to
    /// prefer a `HashMap` so that we can just directly pass it in (and rely on pyo3 to efficiently and
    /// correctly handle creating a Rust `HashMap` from a Python map).
    ///
    /// General rule:
    ///
    /// * Where possible, use the same type for the pyo3 argument as the Rust code wants.
    /// * Otherwise, use a generic pyo3 argument and some form of interconversion.
    fn rust_argument_ty(&mut self, input: &FunctionInput) -> anyhow::Result<String> {
        let input_ty = input.refd_ty().ty();
        match input_ty.kind() {
            TypeKind::Map { key, value, repr } => {
                let name = self.map_name(repr)?;
                Ok(format!(
                    "{name}<{}, {}>",
                    self.generic_ty(key)?,
                    self.generic_ty(value)?
                ))
            }

            TypeKind::Set { element, repr } => {
                let name = self.map_name(repr)?;
                Ok(format!("{name}<{}>", self.generic_ty(element)?))
            }

            TypeKind::Vec {
                element,
                repr: VecRepr::Vec,
            } => Ok(format!("Vec<{}>", self.generic_ty(element)?)),

            TypeKind::Vec {
                element,
                repr: VecRepr::SliceRef,
            } => Ok(format!("Vec<{}>", self.generic_ty(element)?)),

            TypeKind::Path {
                repr: PathRepr::PathBuf,
            } => Ok(format!("PathBuf")),

            TypeKind::Path {
                repr: PathRepr::PathRef,
            } => Ok(format!("&Path")),

            TypeKind::String {
                repr: StringRepr::String,
            } => Ok(format!("String")),

            TypeKind::String {
                repr: StringRepr::StrRef,
            } => Ok(format!("&str")),

            TypeKind::Option {
                element,
                repr: OptionRepr::Option,
            } => Ok(format!("Option<{}>", self.generic_ty(element)?)),

            TypeKind::Result {
                ok,
                err,
                repr: ResultRepr::Result,
            } => Ok(format!(
                "Result<{}, {}>",
                self.generic_ty(ok)?,
                self.generic_ty(err)?
            )),

            TypeKind::Tuple { .. } => Ok(self.generic_ty(input_ty)?),

            TypeKind::Scalar(scalar) => Ok(scalar.to_string()),

            TypeKind::Future { .. } => Ok(self.generic_ty(input_ty)?),

            TypeKind::Error { .. } => Ok(self.generic_ty(input_ty)?),

            TypeKind::UserType { .. } => Ok(self.generic_ty(input_ty)?),

            _ => anyhow::bail!(
                "{span}: unsupported type for `{name}`: {ty} (`{ty:?}`)",
                span = input.span(),
                name = input.name(),
                ty = input_ty,
            ),
        }
    }

    fn map_name(&mut self, v: &MapSetRepr) -> anyhow::Result<String> {
        match v {
            MapSetRepr::Hash => Ok(format!("HashMap")),
            MapSetRepr::BTree => Ok(format!("BTreeMap")),
            MapSetRepr::Index => {
                self.features.push("indexmap");
                Ok(format!("IndexMap"))
            }
            _ => anyhow::bail!("unknown map representation: `{v:?}`"),
        }
    }

    fn set_name(&mut self, v: &MapSetRepr) -> anyhow::Result<String> {
        match v {
            MapSetRepr::Hash => Ok(format!("HashSet")),
            MapSetRepr::BTree => Ok(format!("BTreeSet")),
            MapSetRepr::Index => {
                self.features.push("indexmap");
                Ok(format!("IndexSet"))
            }
            _ => anyhow::bail!("unknown set representation: `{v:?}`"),
        }
    }

    /// Convert a type into a Rust type. This is used for type arguments that are generic
    /// arguments of other types, so they are more limited.
    ///
    /// Will only returned owned values.
    fn generic_ty(&mut self, ty: &Ty) -> anyhow::Result<String> {
        match ty.kind() {
            TypeKind::Map { key, value, repr } => Ok(format!(
                "{}<{}, {}>",
                self.map_name(repr)?,
                self.generic_ty(key)?,
                self.generic_ty(value)?
            )),
            TypeKind::Vec { element, repr } => match repr {
                VecRepr::Vec | VecRepr::SliceRef => Ok(format!("Vec<{}>", self.generic_ty(element)?)),
                _ => anyhow::bail!("unsupported: {repr:?}"),
            },
            TypeKind::Set { element, repr } => Ok(format!(
                "{}<{}>",
                self.set_name(repr)?,
                self.generic_ty(element)?,
            )),
            TypeKind::Path { repr } => match repr {
                gluegun_core::idl::PathRepr::PathBuf => Ok(format!("PathBuf")),
                _ => anyhow::bail!("unsupported: {repr:?}"),
            },
            TypeKind::String { repr } => match repr {
                StringRepr::String => Ok(format!("String")),
                _ => anyhow::bail!("unsupported: {repr:?}"),
            },
            TypeKind::Option { element, repr } => match repr {
                OptionRepr::Option => Ok(format!("Option<{}>", self.generic_ty(element)?)),
                _ => anyhow::bail!("unsupported: {repr:?}"),
            },
            TypeKind::Result { ok, err, repr } => match repr {
                ResultRepr::Result => Ok(format!(
                    "Result<{}, {}>",
                    self.generic_ty(ok)?,
                    self.generic_ty(err)?
                )),
                _ => anyhow::bail!("unsupported: {repr:?}"),
            },
            TypeKind::Tuple { elements, repr } => match repr {
                TupleRepr::Tuple(_) => Ok(format!(
                    "({})",
                    elements
                        .iter()
                        .map(|e| self.generic_ty(e))
                        .collect::<anyhow::Result<Vec<_>>>()?
                        .join(", ")
                )),
                _ => anyhow::bail!("unsupported: {repr:?}"),
            },
            TypeKind::Scalar(scalar) => Ok(scalar.to_string()),
            TypeKind::Future { output, repr } => match repr {
                FutureRepr::PinBoxDynFuture(_auto_traits) => Ok(format!(
                    "Pin<Box<dyn Future<Output = {}>>>",
                    self.generic_ty(output)?
                )),
                _ => anyhow::bail!("unsupported: {repr:?}"),
            },
            TypeKind::Error { repr } => anyhow::bail!("unsupported: {repr:?}"),
            TypeKind::UserType { qname } => Ok(format!("{}", qname.dotted())),
            _ => todo!(),
        }
    }
}
