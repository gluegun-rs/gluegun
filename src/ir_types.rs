use std::sync::Arc;

use accessors_rs::Accessors;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Ty {
    kind: Arc<TypeKind>,
    rust_repr: RustRepr,
}

impl Ty {
    pub(crate) fn new(kind: TypeKind, repr: RustReprKind) -> Self {
        Self {
            kind: Arc::new(kind),
            rust_repr: RustRepr::new(repr),
        }
    }

    pub fn kind(&self) -> &TypeKind {
        &self.kind
    }

    pub(crate) fn with_repr(self, r: impl FnOnce(RustRepr) -> RustReprKind) -> Self {
        let kind = r(self.rust_repr);
        Self { rust_repr: RustRepr::new(kind), ..self }
    }

    pub fn rust_repr(&self) -> &RustRepr {
        &self.rust_repr
    }
}


#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TypeKind {
    Map { key: Ty, value: Ty },
    Vec { element: Ty },
    Set { element: Ty },
    Path,
    Option { element: Ty },
    Result { ok: Ty, err: Ty },
    Tuple { elements: Vec<Ty> },
    Scalar(Scalar),
}

/// Recognized scalar types
#[non_exhaustive]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Scalar {
    Char,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
}

/// Recognized scalar types
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RustRepr {
    kind: Arc<RustReprKind>,
}

impl RustRepr {
    pub(crate) fn new(kind: RustReprKind) -> Self {
        Self { kind: Arc::new(kind) }
    }

    pub fn kind(&self) -> &RustReprKind {
        &self.kind
    }
}

#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RustReprKind {
    Scalar(Scalar),
    Ref(RustRepr),
    Slice(Ty),
    Named(Name, Vec<Ty>),
    /// A "struct" defined in this library
    Struct(UserTypeName),
    Tuple(Vec<Ty>),
}

/// Name of a struct or enum defined in this library.
#[derive(Accessors, Clone, PartialEq, Eq, Debug)]
#[accessors(get)]
pub struct UserTypeName {
    name: String,
}

/// Well known Rust types.
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Name {
    String,
    Str,
    HashMap,
    IndexMap,
    BTreeMap,
    HashSet,
    IndexSet,
    BTreeSet,
    ImplMapLike,
    ImplVecLike,
    ImplSetLike,
    PathBuf,
    ImplAsRef,
    Path,
}
