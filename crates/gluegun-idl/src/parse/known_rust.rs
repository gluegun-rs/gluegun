use std::collections::BTreeMap;

use crate::{Error, Name, RustName, RustReprKind, Scalar, SourcePath, Ty, TypeKind};

/// A Rust path extracted from syn
pub struct RustPath {
    pub(super) idents: Vec<syn::Ident>,
    pub(super) tys: Vec<Ty>,
    pub(super) bindings: BTreeMap<Name, Ty>,
}

/// Defines a known Rust type that can be matched against.
/// See [`elaborate_rust_type`][].
pub(super) struct KnownRustType {
    /// A path, beginning with the crate name.
    name: &'static [&'static str],

    /// The Type Kind to produce from our IDL (given the type arguments),.
    type_kind: ArityFn<TypeKind>,

    /// The Rust name to use for the rust representation;
    /// the `Vec<Ty>` returned are extra arguments to add.
    rust_name: fn() -> (RustName, Vec<Ty>),
}

/// Defines the arity along with a function that constructs the type kind, given its arguments.
enum ArityFn<O> {
    /// No positional type arguments expected.
    Arity0(O),

    /// Exactly 1 positional type argument expected (`<X>`)
    Arity1(fn(Ty) -> crate::Result<O>),

    /// No positional type arguments expected but `Output = X` expected.
    Arity0Output(fn(Ty) -> crate::Result<O>),

    /// Exactly 2 positional type arguments expected (`<X, Y>`).
    Arity2(fn(Ty, Ty) -> crate::Result<O>),
}

/// Known Rust types that we recognize from the std library or elsewhere.
pub(super) const KNOWN_RUST_TYPES: &[KnownRustType] = &[
    KnownRustType {
        name: &["std", "result", "Result"],
        type_kind: ArityFn::Arity2(|ok, err| Ok(TypeKind::Result { ok, err })),
        rust_name: || (RustName::Result, vec![]),
    },
    KnownRustType {
        name: &["anyhow", "Result"],
        type_kind: ArityFn::Arity1(|ok| Ok(TypeKind::Result { ok, err: Ty::anyhow_error() })),
        rust_name: || (RustName::Result, vec![Ty::anyhow_error()]),
    },
    KnownRustType {
        name: &["std", "option", "Option"],
        type_kind: ArityFn::Arity1(|element| Ok(TypeKind::Option { element })),
        rust_name: || (RustName::Option, vec![]),
    },
    KnownRustType {
        name: &["std", "vec", "Vec"],
        type_kind: ArityFn::Arity1(|element| Ok(TypeKind::Vec { element })),
        rust_name: || (RustName::Vec, vec![]),
    },
    KnownRustType {
        name: &["std", "collections", "HashMap"],
        type_kind: ArityFn::Arity2(|key, value| Ok(TypeKind::Map { key, value })),
        rust_name: || (RustName::HashMap, vec![]),
    },
    KnownRustType {
        name: &["std", "collections", "BTreeMap"],
        type_kind: ArityFn::Arity2(|key, value| Ok(TypeKind::Map { key, value })),
        rust_name: || (RustName::BTreeMap, vec![]),
    },
    KnownRustType {
        name: &["std", "collections", "HashSet"],
        type_kind: ArityFn::Arity1(|element| Ok(TypeKind::Set { element })),
        rust_name: || (RustName::HashSet, vec![]),
    },
    KnownRustType {
        name: &["std", "collections", "BTreeSet"],
        type_kind: ArityFn::Arity1(|element| Ok(TypeKind::Set { element })),
        rust_name: || (RustName::BTreeSet, vec![]),
    },
    KnownRustType {
        name: &["str"],
        type_kind: ArityFn::Arity0(TypeKind::String),
        rust_name: || (RustName::Str, vec![]),
    },
    KnownRustType {
        name: &["std", "string", "String"],
        type_kind: ArityFn::Arity0(TypeKind::String),
        rust_name: || (RustName::String, vec![]),
    },
    KnownRustType {
        name: &["std", "path", "Path"],
        type_kind: ArityFn::Arity0(TypeKind::String),
        rust_name: || (RustName::Str, vec![]),
    },
    KnownRustType {
        name: &["std", "path", "PathBuf"],
        type_kind: ArityFn::Arity0(TypeKind::String),
        rust_name: || (RustName::String, vec![]),
    },
    
    KnownRustType {
        name: &["u8"],
        type_kind: ArityFn::Arity0(TypeKind::Scalar(Scalar::U8)),
        rust_name: || (RustName::Scalar(Scalar::U8), vec![]),
    },
    KnownRustType {
        name: &["u16"],
        type_kind: ArityFn::Arity0(TypeKind::Scalar(Scalar::U16)), 
        rust_name: || (RustName::Scalar(Scalar::U16), vec![]),
    },
    KnownRustType {
        name: &["u32"],
        type_kind: ArityFn::Arity0(TypeKind::Scalar(Scalar::U32)),
        rust_name: || (RustName::Scalar(Scalar::U32), vec![]),
    },
    KnownRustType {
        name: &["u64"],
        type_kind: ArityFn::Arity0(TypeKind::Scalar(Scalar::U64)),
        rust_name: || (RustName::Scalar(Scalar::U64), vec![]),
    },
    KnownRustType {
        name: &["i8"],
        type_kind: ArityFn::Arity0(TypeKind::Scalar(Scalar::I8)),
        rust_name: || (RustName::Scalar(Scalar::I8), vec![]),
    },
    KnownRustType {
        name: &["i16"],
        type_kind: ArityFn::Arity0(TypeKind::Scalar(Scalar::I16)),
        rust_name: || (RustName::Scalar(Scalar::I16), vec![]),
    },
    KnownRustType {
        name: &["i32"],
        type_kind: ArityFn::Arity0(TypeKind::Scalar(Scalar::I32)),
        rust_name: || (RustName::Scalar(Scalar::I32), vec![]),
    },
    KnownRustType {
        name: &["i64"],
        type_kind: ArityFn::Arity0(TypeKind::Scalar(Scalar::I64)),
        rust_name: || (RustName::Scalar(Scalar::I64), vec![]),
    },
    KnownRustType {
        name: &["f32"],
        type_kind: ArityFn::Arity0(TypeKind::Scalar(Scalar::F32)),
        rust_name: || (RustName::Scalar(Scalar::F32), vec![]),
    },
    KnownRustType {
        name: &["f64"],
        type_kind: ArityFn::Arity0(TypeKind::Scalar(Scalar::F64)),
        rust_name: || (RustName::Scalar(Scalar::F64), vec![]),
    },
];

/// Known trait paths that can appear after `impl Trait`.
/// These still resolve to types.
pub(super) const KNOWN_RUST_IMPL_TRAIT_TYPES: &[KnownRustType] = &[
    KnownRustType {
        name: &["std", "future", "Future"],
        type_kind: ArityFn::Arity0Output(|output| Ok(TypeKind::Future { output })),
        rust_name: || (RustName::Future, vec![]),
    },
    KnownRustType {
        name: &["std", "convert", "AsRef"],
        type_kind: ArityFn::Arity1(|element| Ok(element.kind().clone())),
        rust_name: || (RustName::ImplAsRef, vec![]),
    },
    KnownRustType {
        name: &["std", "convert", "Into"],
        type_kind: ArityFn::Arity1(|element| Ok(element.kind().clone())),
        rust_name: || (RustName::ImplInto, vec![]),
    },
    KnownRustType {
        name: &["gluegun", "MapLike"],
        type_kind: ArityFn::Arity2(|key, value| Ok(TypeKind::Map { key, value })),
        rust_name: || (RustName::ImplMapLike, vec![]),
    },
    KnownRustType {
        name: &["gluegun", "VecLike"],
        type_kind: ArityFn::Arity1(|element| Ok(TypeKind::Vec { element })),
        rust_name: || (RustName::ImplVecLike, vec![]),
    },
    KnownRustType {
        name: &["gluegun", "SetLike"],
        type_kind: ArityFn::Arity1(|element| Ok(TypeKind::Set { element })),
        rust_name: || (RustName::ImplSetLike, vec![]),
    },
];

/// Match the path, deconstructed into `idents` and `tys`, that appears in `ty` against the list `krts` of known Rust types.
/// Returns `Ok(Some(ty))` if the match is successful or `Ok(None)` if there is no match.
/// Returns an error if there is a match for the name but the arity is wrong or some other similar situation.
pub(super) fn elaborate_rust_type(
    source: &SourcePath,
    ty: &syn::Type,
    path: RustPath,
    krts: &[KnownRustType],
) -> crate::Result<Option<Ty>> {
    let krt = if path.idents.len() == 1 {
        // If the user just wrote `Foo`, search just the last identifier.
        // We just assume all std Rust types are either in the prelude or are imported by some `use`.
        // This is a bit of a hack because the user may have shadowed e.g. `HashMap` with their own `HashMap`
        // and we won't notice. Oh well, I'm lazy.
        krts.iter()
            .find(|krt| path.idents[0] == *krt.name.last().unwrap())
    } else {
        krts.iter().find(|krt| {
            path.idents.len() == krt.name.len()
                && path.idents.iter().zip(krt.name.iter()).all(|(a, b)| a == b)
        })
    };

    // Did we find an entry?
    let Some(krt) = krt else {
        return Ok(None);
    };

    // Construct the type kind.
    let type_kind = match &krt.type_kind {
        ArityFn::Arity0(f) => {
            if path.tys.len() != 0 {
                return Err(Error::GenericsNotPermitted(source.span(ty)));
            } else {
                f.clone()
            }
        }
        ArityFn::Arity1(f) => {
            if path.tys.len() != 1 {
                return Err(Error::GenericsNotPermitted(source.span(ty)));
            } else {
                f(path.tys[0].clone())?
            }
        }
        ArityFn::Arity0Output(f) => {
            let output = Name::from("Output");
            
            let Some(output_ty) = path.bindings.get(&output) else {
                return Err(Error::BindingNotFound(source.span(ty), output));
            };
            
            if path.tys.len() != 0 {
                return Err(Error::GenericsNotPermitted(source.span(ty)));
            } else {
                f(output_ty.clone())?
            }
        }
        ArityFn::Arity2(f) => {
            if path.tys.len() != 2 {
                return Err(Error::GenericsNotPermitted(source.span(ty)));
            } else {
                f(path.tys[0].clone(), path.tys[1].clone())?
            }
        }
    };

    // The Rust repr will be a "named" type.
    let (rust_name, extra_tys) = (krt.rust_name)();
    let mut rust_repr_tys = path.tys;
    rust_repr_tys.extend(extra_tys);
    let rust_repr = RustReprKind::Named(rust_name, rust_repr_tys, path.bindings);

    Ok(Some(Ty::new(type_kind, rust_repr)))
}
