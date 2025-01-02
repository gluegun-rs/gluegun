use std::sync::Arc;
pub struct Ty {
    kind: Arc<TypeKind>,
    rust_repr: Arc<RustRepr>,
}

impl Ty {
    pub fn kind(&self) -> &TypeKind { &self.kind }
}

#[non_exhaustive]
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
pub enum Scalar {
    Char,
    I8, I16, I32, I64,
    U8, U16, U32, U64,
    F32, F64,
}

/// Recognized scalar types
pub struct RustRepr {
    kind: Arc<RustReprKind>,
}

#[non_exhaustive]
pub enum RustReprKind {
    Scalar(Scalar),
    Ref(RustRepr),
    Slice(RustRepr),
    Named(Name, Vec<RustRepr>),
}

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