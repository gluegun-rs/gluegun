use std::collections::BTreeMap;

use crate::{AutoTraits, Error, Name, Scalar, Span, StringRepr, Ty, TypeKind};

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
    MakeType(fn(Span, &[Modifier], &[Ty], &BTreeMap<Name, Ty>) -> crate::Result<TypeKind>),
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
    [] std::result::Result[ok, err][] @ _span => TypeKind::Result { ok, err, repr: crate::ResultRepr::Result },
    [] anyhow::Result[ok][] @ span => TypeKind::Result { ok, err: Ty::anyhow_error(span), repr: crate::ResultRepr::Result },
    [] std::option::Option[element][] @ _span => TypeKind::Option { element, repr: crate::OptionRepr::Option },

    [] std::string::String[][] @ _span => TypeKind::String { repr: StringRepr::String },
    [Modifier::Ref(r)] str[][] @ _span => TypeKind::String { repr: StringRepr::Str(r) },

    [] std::vec::Vec[element][] @ _span => TypeKind::Vec { element, repr: crate::VecRepr::Vec, },
    [] std::collections::HashMap[key, value][] @ _span => TypeKind::Map { key, value, repr: crate::MapSetRepr::Owned(crate::MapVariant::BTree) },
    [] std::collections::BTreeMap[key, value][] @ _span => TypeKind::Map { key, value, repr: crate::MapSetRepr::Owned(crate::MapVariant::BTree) },
    [] std::collections::HashSet[element][] @ _span => TypeKind::Set { element, repr: crate::MapSetRepr::Owned(crate::MapVariant::BTree) },
    [] std::collections::BTreeSet[element][] @ _span => TypeKind::Set { element, repr: crate::MapSetRepr::Owned(crate::MapVariant::BTree) },
    [Modifier::Ref(r)] std::path::Path[][] @ _span => TypeKind::Path { repr: crate::PathRepr::Path(r) },
    [] std::path::PathBuf[][] @ _span => TypeKind::Path { repr: crate::PathRepr::PathBuf },

    [] u8[][] @ _span => TypeKind::Scalar(Scalar::U8),
    [] u16[][] @ _span => TypeKind::Scalar(Scalar::U16),
    [] u32[][] @ _span => TypeKind::Scalar(Scalar::U32),
    [] u64[][] @ _span => TypeKind::Scalar(Scalar::U64),
    [] i8[][] @ _span => TypeKind::Scalar(Scalar::I8),
    [] i16[][] @ _span => TypeKind::Scalar(Scalar::I16),
    [] i32[][] @ _span => TypeKind::Scalar(Scalar::I32),
    [] i64[][] @ _span => TypeKind::Scalar(Scalar::I64),
    [] f32[][] @ _span => TypeKind::Scalar(Scalar::F32),
    [] f64[][] @ _span => TypeKind::Scalar(Scalar::F64),

    ---
    
};


/// Known Rust types that we recognize from the std library or elsewhere.
pub(super) const KNOWN_RUST_IMPL_TRAIT_TYPES: &[KnownRustType] = known_rust_types! {
    [] std::string::ToString[][] @ _span => TypeKind::String { repr: StringRepr::ImplToString },
    [] std::task::Future[][Output = output] @ _span => TypeKind::Future { output, repr: crate::FutureRepr::ImplFuture(AutoTraits::default()) },

    ---
    
    std::convert::AsRef => Modifier::Ref(crate::RefKind::ImplAsRef),
};
