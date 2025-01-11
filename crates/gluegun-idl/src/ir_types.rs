use std::{collections::BTreeMap, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{Name, QualifiedName};

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

    pub(crate) fn anyhow_error() -> Self {
        Ty::new(TypeKind::Error, RustReprKind::Named(RustName::AnyhowError, Default::default(), Default::default()))
    }

    /// Returns the unit type. Used for a dummy value in early phases.
    pub fn unit() -> Self {
        Self::new(
            TypeKind::Tuple { elements: vec![] },
            RustReprKind::Tuple(vec![]),
        )
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
        Self {
            rust_repr: RustRepr::new(kind),
            ..self
        }
    }

    pub fn rust_repr(&self) -> &RustRepr {
        &self.rust_repr
    }
}

impl std::fmt::Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &*self.kind {
            TypeKind::Map { key, value } => write!(f, "Map<{}, {}>", key, value),
            TypeKind::Vec { element } => write!(f, "Vec<{}>", element),
            TypeKind::Set { element } => write!(f, "Set<{}>", element),
            TypeKind::Path => write!(f, "Path"),
            TypeKind::String => write!(f, "String"),
            TypeKind::Option { element } => write!(f, "Option<{}>", element),
            TypeKind::Result { ok, err } => write!(f, "Result<{}, {}>", ok, err),
            TypeKind::Tuple { elements } => {
                let mut s = String::new();
                s.push('(');
                for (i, e) in elements.iter().enumerate() {
                    if i > 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&e.to_string());
                }
                s.push(')');
                write!(f, "{}", s)
            }
            TypeKind::Scalar(s) => write!(f, "{}", s),
            TypeKind::Future { output } => write!(f, "impl Future<Output = {}>", output),
            TypeKind::Error => write!(f, "Error"),
            TypeKind::UserType { qname } => write!(f, "{}", qname.to_string("::")),
        }
    }
}

#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum TypeKind {
    Map {
        key: Ty,
        value: Ty,
    },
    Vec {
        element: Ty,
    },
    Set {
        element: Ty,
    },
    Path,
    String,
    Option {
        element: Ty,
    },
    Result {
        ok: Ty,
        err: Ty,
    },
    Tuple {
        elements: Vec<Ty>,
    },
    Scalar(Scalar),
    Future {
        output: Ty,
    },

    // Represents a generic exception/error type.
    Error,

    /// Type defined by the user
    UserType {
        qname: QualifiedName,
    },
}

/// Recognized scalar types
#[non_exhaustive]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Scalar {
    Boolean,
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

impl std::fmt::Display for Scalar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scalar::Boolean => write!(f, "bool"),
            Scalar::Char => write!(f, "char"),
            Scalar::I8 => write!(f, "i8"),
            Scalar::I16 => write!(f, "i16"),
            Scalar::I32 => write!(f, "i32"),
            Scalar::I64 => write!(f, "i64"),
            Scalar::U8 => write!(f, "u8"),
            Scalar::U16 => write!(f, "u16"),
            Scalar::U32 => write!(f, "u32"),
            Scalar::U64 => write!(f, "u64"),
            Scalar::F32 => write!(f, "f32"),
            Scalar::F64 => write!(f, "f64"),
        }
    }
}


/// Recognized scalar types
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct RustRepr {
    kind: Arc<RustReprKind>,
}

impl RustRepr {
    pub(crate) fn new(kind: RustReprKind) -> Self {
        Self {
            kind: Arc::new(kind),
        }
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
    Named(RustName, Vec<Ty>, BTreeMap<Name, Ty>),
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
    Scalar(Scalar),
    Result,
    Option,
    AnyhowError,
    Future,
}
