use std::collections::BTreeMap;

use crate::{AutoTraits, Error, Name, RefdTy, Scalar, Span, StringRepr, Ty, TypeKind};

use super::modifier::Modifier;

/// A Rust path extracted from syn
pub struct RustPath<'s> {
    pub(super) idents: Vec<syn::Ident>,
    pub(super) tys: Vec<&'s syn::Type>,
    pub(super) bindings: BTreeMap<Name, Ty>,
}

/// Defines a known Rust type that can be matched against.
/// See [`elaborate_rust_type`][].
pub(super) struct KnownRustType {
    /// A path, beginning with the crate name.
    pub(super) name: &'static [&'static str],

    /// The Type Kind to produce from our IDL (given the type arguments),.
    pub(super) kr_fn: KnownRustFn,
}

pub(super) enum KnownRustFn {
    MakeType(fn(Span, &[Modifier], &[Ty], &BTreeMap<Name, Ty>) -> crate::Result<RefdTy>),
    Modifier(Modifier),
}

// Macro for creating the "known rust type" table
macro_rules! known_rust_types {
    (
        $(
            [$($m:pat)*] $ident0:ident $(:: $ident1:ident)* [ $($ty:ident),* ] [$($binding:ident = $tyb:ident),* ] @ $s:pat => $e:expr,
        )*

        ---

        $(
            $mod_ident0:ident $(:: $mod_ident1:ident)* => $mod_value:expr,
        )*
    ) => {
        &[
            $(
                KnownRustType {
                    name: &[stringify!($ident0) $(, stringify!($ident1))*],
                    kr_fn: KnownRustFn::MakeType(|span, modifiers, tys, bindings| {
                        // Check whether modifier(s) meets the expected modifier(s)
                        #[allow(unused_mut)]
                        let mut expected_modifiers = 0;
                        $(
                            let Some($m) = modifiers.get(expected_modifiers).cloned() else {
                                return Err($crate::Error::UnsupportedUseOfType(span));
                            };
                            expected_modifiers += 1;
                        )*
                        if modifiers.len() != expected_modifiers {
                            return Err($crate::Error::UnsupportedUseOfType(span.clone()));
                        }

                        // Extract the arguments, erroring if the number provided doesn't match expectations
                        #[allow(unused_mut)]
                        let mut expected_args = 0;
                        $(
                            let Some($ty) = tys.get(expected_args).cloned() else {
                                return Err($crate::Error::UnsupportedUseOfType(span));
                            };
                            expected_args += 1;
                        )*
                        if tys.len() != expected_args {
                            return Err($crate::Error::UnsupportedUseOfType(span.clone()));
                        }

                        // Same for bindings.
                        #[allow(unused_mut)]
                        let mut expected_bindings = 0;
                        $(
                            let binding_name = $crate::Name::from(stringify!($binding));
                            let Some($tyb) = bindings.get(&binding_name).cloned() else {
                                return Err(Error::BindingNotFound(span.clone(), binding_name));
                            };
                            expected_bindings += 1;
                        )*
                        if bindings.len() != expected_bindings {
                            return Err($crate::Error::UnsupportedUseOfType(span.clone()));
                        }

                        // Assign span to the user's variable
                        let $s = span;

                        // Execute user expression
                        Ok($e)
                    }),
                },
            )*

            $(
                KnownRustType {
                    name: &[stringify!($mod_ident0) $(, stringify!($mod_ident1))*],
                    kr_fn: KnownRustFn::Modifier($mod_value),
                },
            )*
        ]
    };
}

/// Known Rust types that we recognize from the std library or elsewhere.
pub(super) const KNOWN_RUST_TYPES: &[KnownRustType] = known_rust_types! {
    [] std::result::Result[ok, err][] @ span => TypeKind::Result { ok, err, repr: crate::ResultRepr::Result }.not_refd(span),
    [] anyhow::Result[ok][] @ span => TypeKind::Result { ok, err: Ty::anyhow_error(span.clone()), repr: crate::ResultRepr::Result }.not_refd(span),
    [] std::option::Option[element][] @ span => TypeKind::Option { element, repr: crate::OptionRepr::Option }.not_refd(span),

    [] std::string::String[][] @ span => TypeKind::String { repr: StringRepr::String }.not_refd(span),
    [Modifier::Ref(r)] str[][] @ span => TypeKind::String { repr: StringRepr::StrRef }.refd(span, r),

    [] std::vec::Vec[element][] @ span => TypeKind::Vec { element, repr: crate::VecRepr::Vec, }.not_refd(span),
    [] std::collections::HashMap[key, value][] @ span =>TypeKind::Map { key, value, repr: crate::MapSetRepr::BTree }.not_refd(span),
    [] std::collections::BTreeMap[key, value][] @ span => TypeKind::Map { key, value, repr: crate::MapSetRepr::BTree }.not_refd(span),
    [] std::collections::HashSet[element][] @ span =>TypeKind::Set { element, repr: crate::MapSetRepr::BTree }.not_refd(span),
    [] std::collections::BTreeSet[element][] @ span => TypeKind::Set { element, repr: crate::MapSetRepr::BTree }.not_refd(span),
    [Modifier::Ref(r)] std::path::Path[][] @ span => TypeKind::Path { repr: crate::PathRepr::PathRef }.refd(span, r),
    [] std::path::PathBuf[][] @ span => TypeKind::Path { repr: crate::PathRepr::PathBuf }.not_refd(span),

    [] u16[][] @ span => TypeKind::Scalar(Scalar::U16).not_refd(span),
    [] u32[][] @ span => TypeKind::Scalar(Scalar::U32).not_refd(span),
    [] u64[][] @ span => TypeKind::Scalar(Scalar::U64).not_refd(span),
    [] i8[][] @ span => TypeKind::Scalar(Scalar::I8).not_refd(span),
    [] i16[][] @ span => TypeKind::Scalar(Scalar::I16).not_refd(span),
    [] i32[][] @ span => TypeKind::Scalar(Scalar::I32).not_refd(span),
    [] i64[][] @ span => TypeKind::Scalar(Scalar::I64).not_refd(span),
    [] f32[][] @ span => TypeKind::Scalar(Scalar::F32).not_refd(span),
    [] f64[][] @ span => TypeKind::Scalar(Scalar::F64).not_refd(span),

    ---
    
};


/// Known Rust types that we recognize from the std library or elsewhere.
pub(super) const KNOWN_RUST_IMPL_TRAIT_TYPES: &[KnownRustType] = known_rust_types! {
    [] std::string::ToString[][] @ span => TypeKind::String { repr: StringRepr::ImplToString }.not_refd(span),
    [] std::task::Future[][Output = output] @ span => TypeKind::Future { output, repr: crate::FutureRepr::ImplFuture(AutoTraits::default()) }.not_refd(span),

    ---
    
    std::convert::AsRef => Modifier::Ref(crate::RefKind::ImplAsRef),
};
