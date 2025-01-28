use std::{borrow::Cow, sync::Arc};

use accessors_rs::Accessors;
use serde::{Deserialize, Serialize};

use crate::{QualifiedName, Span};

#[derive(Accessors, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct Ty {
    #[accessors(get)]
    span: Span,

    kind: Arc<TypeKind>,
}

impl Ty {
    pub(crate) fn new(span: Span, kind: TypeKind) -> Self {
        Self {
            span,
            kind: Arc::new(kind),
        }
    }

    pub(crate) fn anyhow_error(span: Span) -> Self {
        Ty::new(
            span,
            TypeKind::Error {
                repr: ErrorRepr::AnyhowError,
            },
        )
    }

    pub fn kind(&self) -> &TypeKind {
        &*self.kind
    }

    /// Returns the unit type. Used for a dummy value in early phases.
    pub fn unit(span: Span) -> Self {
        Self::new(
            span,
            TypeKind::Tuple { elements: vec![], repr: TupleRepr::Tuple(0) },
        )
    }

    pub(crate) fn user(span: Span, qname: &QualifiedName) -> Self {
        Ty::new(
            span,
            TypeKind::UserType {
                qname: qname.clone(),
                repr: UserTypeRepr::Owned,
            },
        )
    }
}

impl std::fmt::Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &*self.kind {
            TypeKind::Map { key, value, repr: _ } => write!(f, "Map<{}, {}>", key, value),
            TypeKind::Vec { element, repr: _ } => write!(f, "Vec<{}>", element),
            TypeKind::Set { element , repr: _} => write!(f, "Set<{}>", element),
            TypeKind::Path { repr: _ } => write!(f, "Path"),
            TypeKind::String { repr: _ } => write!(f, "String"),
            TypeKind::Option { element, repr: _ } => write!(f, "Option<{}>", element),
            TypeKind::Result { ok, err, repr: _ } => write!(f, "Result<{}, {}>", ok, err),
            TypeKind::Tuple { elements, repr: _ } => {
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
            TypeKind::Future { output, repr: _ } => write!(f, "impl Future<Output = {}>", output),
            TypeKind::Error { repr: _ } => write!(f, "Error"),
            TypeKind::UserType { qname, repr: _ } => write!(f, "{}", qname.to_string("::")),
        }
    }
}

#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum TypeKind {
    Map {
        key: Ty,
        value: Ty,
        repr: MapSetRepr,
    },
    Vec {
        element: Ty,
        repr: VecRepr,
    },
    Set {
        element: Ty,
        repr: MapSetRepr,
    },
    Path {
        repr: PathRepr,
    },
    String {
        repr: StringRepr,
    },
    Option {
        element: Ty,
        repr: OptionRepr,
    },
    Result {
        ok: Ty,
        err: Ty,
        repr: ResultRepr,
    },
    Tuple {
        elements: Vec<Ty>,
        repr: TupleRepr,
    },

    Scalar(Scalar),
    
    Future {
        output: Ty,
        repr: FutureRepr,
    },

    // Represents a generic exception/error type.
    Error {
        repr: ErrorRepr,
    },

    /// Type defined by the user
    UserType {
        qname: QualifiedName,

        repr: UserTypeRepr,
    },
}

impl std::fmt::Display for TypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeKind::Map { key, value, repr: _ } => write!(f, "Map<{}, {}>", key, value)?,
            TypeKind::Vec { element, repr: _ } => write!(f, "Vec<{}>", element)?,
            TypeKind::Set { element, repr: _ } => write!(f, "Set<{}>", element)?,
            TypeKind::Path { repr: _ } => write!(f, "Path")?,
            TypeKind::String { repr: _ } => write!(f, "String")?, 
            TypeKind::Option { element, repr: _ } => write!(f, "Option<{}>", element)?,
            TypeKind::Result { ok, err, repr: _ } => write!(f, "Result<{}, {}>", ok, err)?,
            TypeKind::Tuple { elements, repr: _ } => {
                let mut s = String::new();
                s.push('(');
                for (i, e) in elements.iter().enumerate() {
                    if i > 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&e.to_string());
                }
                s.push(')');
                write!(f, "{}", s)?
            },
            TypeKind::Scalar(scalar) => write!(f, "{}", scalar)?,
            TypeKind::Future { output, repr: _ } => write!(f, "impl Future<Output = {}>", output)?,
            TypeKind::Error { repr: _ } => write!(f, "Error")?,
            TypeKind::UserType { qname, repr: _ } => write!(f, "{}", qname.to_string("::"))?,
        }
        Ok(())
    }
}

/// Different patterns that we recognize as being a "string" in Rust code.
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum StringRepr {
    /// String
    String,

    /// &str
    Str(RefKind),
    
    /// impl ToString
    ImplToString,
}

/// Different patterns that we recognize as being a "Vec" in Rust code.
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum VecRepr {
    /// Vec
    Vec,

    /// [T]
    Slice(RefKind),
}

/// Different patterns that we recognize as being a "Map" in Rust code.
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum MapSetRepr {
    Owned(MapVariant),

    Ref(MapVariant, RefKind),
}

#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum MapVariant {
    Hash,
    BTree,
    Index,
}
/// Different patterns that we recognize as being a "Path" in Rust code.
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum PathRepr {
    /// Path,
    Path(RefKind),

    /// PathBuf
    PathBuf,
}

/// Different patterns that we recognize as being a "Option" in Rust code.
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum OptionRepr {
    /// Option<E>
    Option,
}

/// Different patterns that we recognize as being a "Result" in Rust code.
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum ResultRepr {
    /// Result<E>
    Result,
}

/// Different patterns that we recognize as being a "Tuple" in Rust code.
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum TupleRepr {
    /// (...) of arity N
    Tuple(usize),
}

/// Different patterns that we recognize as being a "Future" in Rust code.
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum FutureRepr {
    /// `impl Future<Output = T>`
    ImplFuture(AutoTraits),

    /// `Pin<Box<dyn Future<Output = T>`
    PinBoxDynFuture(AutoTraits),
}

/// Different patterns that we recognize as being an "Error" in Rust code.
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum ErrorRepr {
    /// `anyhow::Error`
    AnyhowError,

    /// `Box<dyn Error>`
    BoxDynError(AutoTraits),
}

#[non_exhaustive]
#[derive(Accessors, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[accessors(get_copy)]
pub struct AutoTraits {
    send: bool,
    sync: bool,
    unpin: bool,
}

/// Kinds of references that can be provided
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum RefKind {
    /// `&T``
    AnonRef,

    /// `impl AsRef<T>`
    ImplAsRef,
}

/// Kinds of references that can be provided
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum UserTypeRepr {
    /// `T``
    Owned,

    /// `&T`
    Ref(RefKind),
}

/// Recognized scalar types.
///
/// The `Display` impl gives their Rust names.
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

impl Scalar {
    pub fn as_str(&self) -> Cow<'_, str> {
        Cow::Borrowed(match self {
            Scalar::Boolean => "bool",
            Scalar::Char => "char",
            Scalar::I8 => "i8",
            Scalar::I16 => "i16",
            Scalar::I32 => "i32",
            Scalar::I64 => "i64",
            Scalar::U8 => "u8",
            Scalar::U16 => "u16",
            Scalar::U32 => "u32",
            Scalar::U64 => "u64",
            Scalar::F32 => "f32",
            Scalar::F64 => "f64",
        })
    }
}

impl std::fmt::Display for Scalar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

