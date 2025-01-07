use std::{
    collections::BTreeMap, path::{Path, PathBuf}, str::FromStr
};

use syn::{spanned::Spanned, ImplItemFn};

use crate::{Enum, Error, Field, Function, IsAsync, Item, Method, Module, Name, QualifiedName, Record, Resource, RustRepr, RustReprKind, SelfKind, Ty, Universe, Variant, VariantArm};

pub fn parse_path(path: &Path) -> crate::Result<Universe> {
    let text = std::fs::read_to_string(path)?;
    let tokens = proc_macro2::TokenStream::from_str(&text)?;
    let ast: syn::File = syn::parse2(tokens)?;

    todo!()
}

#[derive(Default)]
pub struct ParserArena {
    files: typed_arena::Arena<syn::File>,
}

struct Parser<'arena> {
    crate_name: Name,
    arena: &'arena ParserArena,
    definitions: BTreeMap<QualifiedName, Definition<'arena>>,
}

/// Internal intermediate structure representing some kind of public user-visible definition.
struct Definition<'p> {
    /// The syn module from which this was parsed.
    module: &'p syn::File, 

    /// The kind of definition.
    kind: DefinitionKind<'p>,
}

/// Internal intermediate structure representing kind of some public user-visible definition.
/// The names reference [WIT](https://component-model.bytecodealliance.org/design/wit.html).
enum DefinitionKind<'p> {
    /// *Resources* are "class-like" structures defined by their methods.
    /// In Rust, they are represented by a struct with private fields or a `#[non_exhaustive]` attribute.
    Resource(&'p syn::ItemStruct),

    /// *Records* are "struct-like" structures defined by their fields.
    /// In Rust, they are represented by a struct with public fields and no `#[non_exhaustive]` attribute.
    Record(&'p syn::ItemStruct),

    /// *Variants* are "enum-like" structures with data-carrying fields.
    /// In Rust, they are represented by a non-C-like enum that is not marked with `#[non_exhaustive]`.
    Variant(&'p syn::ItemEnum, Vec<&'p syn::Variant>),

    /// *Enums* are "enum-like" structures with no data-carrying fields.
    /// In Rust, they are represented by a C-like enum that is not marked with `#[non_exhaustive]`.
    Enum(&'p syn::ItemEnum, Vec<&'p syn::Variant>),

    /// *Functions* are top-level, callable functions (!).
    Function(&'p syn::ItemFn),

    /// *Modules* are public Rust modules; unlike the other variants, these are not mapped to output items,
    /// but they are used in name resolution.
    FileModule(&'p syn::File),
}

impl<'p> Parser<'p> {
    pub fn new(arena: &'p ParserArena, crate_name: Name) -> Self {
        Self {
            arena,
            crate_name,
            definitions: BTreeMap::new(),
        }
    }

    /// The path of the root crate file (typically something like `src/lib.rs`)
    pub fn parse(&mut self, crate_root_path: impl AsRef<Path>) -> crate::Result<()> {
        self.parse_path(crate_root_path.as_ref())
    }

}

/// Pass 1: Recognize types, imports, and things. Don't fill out the details (fields, methods).
mod pass1;

/// Pass 2: Fill out the fields, methods, etc. In this pass we resolve types.
mod pass2;

mod known_rust;

mod util;