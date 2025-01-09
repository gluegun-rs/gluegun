use crate::{Error, RustName, RustReprKind, Scalar, SourcePath, Ty, TypeKind};

/// Defines a known Rust type that can be matched against.
/// See [`elaborate_rust_type`][].
pub(super) struct KnownRustType {
    /// A path, beginning with the crate name.
    name: &'static [&'static str],
    type_kind: TypeKindFn,
    rust_name: RustName,
}

/// Defines the arity along with a function that constructs the type kind, given its arguments.
enum TypeKindFn {
    Arity0(fn() -> TypeKind),
    Arity1(fn(Ty) -> TypeKind),
    Arity2(fn(Ty, Ty) -> TypeKind),
}

/// Known Rust types that we recognize from the std library or elsewhere.
pub(super) const KNOWN_RUST_TYPES: &[KnownRustType] = &[
    KnownRustType {
        name: &["std", "vec", "Vec"],
        type_kind: TypeKindFn::Arity1(|element| TypeKind::Vec { element }),
        rust_name: RustName::Vec,
    },
    KnownRustType {
        name: &["std", "collections", "HashMap"],
        type_kind: TypeKindFn::Arity2(|key, value| TypeKind::Map { key, value }),
        rust_name: RustName::HashMap,
    },
    KnownRustType {
        name: &["std", "collections", "BTreeMap"],
        type_kind: TypeKindFn::Arity2(|key, value| TypeKind::Map { key, value }),
        rust_name: RustName::BTreeMap,
    },
    KnownRustType {
        name: &["std", "collections", "HashSet"],
        type_kind: TypeKindFn::Arity1(|element| TypeKind::Set { element }),
        rust_name: RustName::HashSet,
    },
    KnownRustType {
        name: &["std", "collections", "BTreeSet"],
        type_kind: TypeKindFn::Arity1(|element| TypeKind::Set { element }),
        rust_name: RustName::BTreeSet,
    },
    KnownRustType {
        name: &["str"],
        type_kind: TypeKindFn::Arity0(|| TypeKind::String),
        rust_name: RustName::Str,
    },
    KnownRustType {
        name: &["std", "string", "String"],
        type_kind: TypeKindFn::Arity0(|| TypeKind::String),
        rust_name: RustName::String,
    },
    KnownRustType {
        name: &["std", "path", "Path"],
        type_kind: TypeKindFn::Arity0(|| TypeKind::String),
        rust_name: RustName::Str,
    },
    KnownRustType {
        name: &["std", "path", "PathBuf"],
        type_kind: TypeKindFn::Arity0(|| TypeKind::String),
        rust_name: RustName::String,
    },
    
    KnownRustType {
        name: &["u8"],
        type_kind: TypeKindFn::Arity0(|| TypeKind::Scalar(Scalar::U8)),
        rust_name: RustName::Scalar(Scalar::U8),
    },
    KnownRustType {
        name: &["u16"],
        type_kind: TypeKindFn::Arity0(|| TypeKind::Scalar(Scalar::U16)), 
        rust_name: RustName::Scalar(Scalar::U16),
    },
    KnownRustType {
        name: &["u32"],
        type_kind: TypeKindFn::Arity0(|| TypeKind::Scalar(Scalar::U32)),
        rust_name: RustName::Scalar(Scalar::U32),
    },
    KnownRustType {
        name: &["u64"],
        type_kind: TypeKindFn::Arity0(|| TypeKind::Scalar(Scalar::U64)),
        rust_name: RustName::Scalar(Scalar::U64),
    },
    KnownRustType {
        name: &["i8"],
        type_kind: TypeKindFn::Arity0(|| TypeKind::Scalar(Scalar::I8)),
        rust_name: RustName::Scalar(Scalar::I8),
    },
    KnownRustType {
        name: &["i16"],
        type_kind: TypeKindFn::Arity0(|| TypeKind::Scalar(Scalar::I16)),
        rust_name: RustName::Scalar(Scalar::I16),
    },
    KnownRustType {
        name: &["i32"],
        type_kind: TypeKindFn::Arity0(|| TypeKind::Scalar(Scalar::I32)),
        rust_name: RustName::Scalar(Scalar::I32),
    },
    KnownRustType {
        name: &["i64"],
        type_kind: TypeKindFn::Arity0(|| TypeKind::Scalar(Scalar::I64)),
        rust_name: RustName::Scalar(Scalar::I64),
    },
    KnownRustType {
        name: &["f32"],
        type_kind: TypeKindFn::Arity0(|| TypeKind::Scalar(Scalar::F32)),
        rust_name: RustName::Scalar(Scalar::F32),
    },
    KnownRustType {
        name: &["f64"],
        type_kind: TypeKindFn::Arity0(|| TypeKind::Scalar(Scalar::F64)),
        rust_name: RustName::Scalar(Scalar::F64),
    },
];

/// Known trait paths that can appear after `impl Trait`.
/// These still resolve to types.
pub(super) const KNOWN_RUST_IMPL_TRAIT_TYPES: &[KnownRustType] = &[
    KnownRustType {
        name: &["std", "convert", "AsRef"],
        type_kind: TypeKindFn::Arity1(|element| element.kind().clone()),
        rust_name: RustName::ImplAsRef,
    },
    KnownRustType {
        name: &["std", "convert", "Into"],
        type_kind: TypeKindFn::Arity1(|element| element.kind().clone()),
        rust_name: RustName::ImplInto,
    },
    KnownRustType {
        name: &["gluegun", "MapLike"],
        type_kind: TypeKindFn::Arity2(|key, value| TypeKind::Map { key, value }),
        rust_name: RustName::ImplMapLike,
    },
    KnownRustType {
        name: &["gluegun", "VecLike"],
        type_kind: TypeKindFn::Arity1(|element| TypeKind::Vec { element }),
        rust_name: RustName::ImplVecLike,
    },
    KnownRustType {
        name: &["gluegun", "SetLike"],
        type_kind: TypeKindFn::Arity1(|element| TypeKind::Set { element }),
        rust_name: RustName::ImplSetLike,
    },
];

/// Match the path, deconstructed into `idents` and `tys`, that appears in `ty` against the list `krts` of known Rust types.
/// Returns `Ok(Some(ty))` if the match is successful or `Ok(None)` if there is no match.
/// Returns an error if there is a match for the name but the arity is wrong or some other similar situation.
pub(super) fn elaborate_rust_type(
    source: &SourcePath,
    ty: &syn::Type,
    idents: &[syn::Ident],
    tys: &[Ty],
    krts: &[KnownRustType],
) -> crate::Result<Option<Ty>> {
    let krt = if idents.len() == 1 {
        // If the user just wrote `Foo`, search just the last identifier.
        // We just assume all std Rust types are either in the prelude or are imported by some `use`.
        // This is a bit of a hack because the user may have shadowed e.g. `HashMap` with their own `HashMap`
        // and we won't notice. Oh well, I'm lazy.
        krts.iter()
            .find(|krt| idents[0] == *krt.name.last().unwrap())
    } else {
        krts.iter().find(|krt| {
            idents.len() == krt.name.len()
                && idents.iter().zip(krt.name.iter()).all(|(a, b)| a == b)
        })
    };

    // Did we find an entry?
    let Some(krt) = krt else {
        return Ok(None);
    };

    // Construct the type kind.
    let type_kind = match krt.type_kind {
        TypeKindFn::Arity0(f) => {
            if tys.len() != 0 {
                return Err(Error::GenericsNotPermitted(source.span(ty)));
            } else {
                f()
            }
        }
        TypeKindFn::Arity1(f) => {
            if tys.len() != 1 {
                return Err(Error::GenericsNotPermitted(source.span(ty)));
            } else {
                f(tys[0].clone())
            }
        }
        TypeKindFn::Arity2(f) => {
            if tys.len() != 2 {
                return Err(Error::GenericsNotPermitted(source.span(ty)));
            } else {
                f(tys[0].clone(), tys[1].clone())
            }
        }
    };

    // The Rust repr will be a "named" type.
    let rust_repr = RustReprKind::Named(krt.rust_name.clone(), tys.to_vec());

    Ok(Some(Ty::new(type_kind, rust_repr)))
}
