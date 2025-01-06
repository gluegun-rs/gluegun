use std::sync::Arc;

use accessors_rs::Accessors;
use serde::{Deserialize, Serialize};

use crate::QualifiedName;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
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

    /// Returns the unit type. Used for a dummy value in early phases.
    pub fn unit() -> Self {
        Self::new(TypeKind::Tuple { elements: vec![] }, RustReprKind::Tuple(vec![]))
    }

    pub fn kind(&self) -> &TypeKind {
        &self.kind
    }

    pub(crate) fn user(qname: &QualifiedName) -> Self {
        Ty::new(
            TypeKind::UserType {
                qname: qname.clone(),
            },
            RustReprKind::User(qname.clone()),
        )
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
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum TypeKind {
    Map { key: Ty, value: Ty },
    Vec { element: Ty },
    Set { element: Ty },
    Path,
    String,
    Option { element: Ty },
    Result { ok: Ty, err: Ty },
    Tuple { elements: Vec<Ty> },
    Scalar(Scalar),

    /// Type defined by the user 
    UserType { qname: QualifiedName },
}

/// Recognized scalar types
#[non_exhaustive]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
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
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
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
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum RustReprKind {
    Scalar(Scalar),
    Ref(RustRepr),
    Slice(Ty),
    Named(RustName, Vec<Ty>),
    /// A type defined in this library
    User(QualifiedName),
    Tuple(Vec<Ty>),
}

/// Well known Rust types.
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum RustName {
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
    ImplInto,
    Path,
    Vec,
}
