use std::path::Path;

use crate::{Error, Idl, Name, QualifiedName, SourcePath};

pub struct Parser {
}

impl Parser {
    pub fn new() -> Self {
        Self {
        }
    }

    /// Parse the crate with the given name and the path to its `lib.rs`.
    pub fn parse_crate_named(
        &mut self,
        crate_name: impl Into<Name>,
        crate_path: impl AsRef<Path>,
    ) -> crate::Result<Idl> {
        let crate_name: Name = crate_name.into();
        let crate_path: &Path = crate_path.as_ref();
        let arena = AstArena::default();
        let ast = arena.parse_file(crate_path)?;
        let crate_qname = QualifiedName::from(&crate_name);
        let source = SourcePath::new(crate_path);
        let recognized = pass1::Recognizer::new(&source, crate_qname, ast).into_recognized()?;
        let elaborated = pass2::Elaborator::new(recognized).into_elaborated_items()?;
        Ok(Idl {
            crate_name,
            definitions: elaborated,
        })
    }

    /// Convenient function to add the crate at `rs_path`, inferring the crate name,
    /// and then invoke [`Self::parse_crate_named`][].
    pub fn parse_crate(&mut self, crate_path: impl AsRef<Path>) -> crate::Result<Idl> {
        let crate_path: &Path = crate_path.as_ref();
        let crate_name = extract_crate_name(crate_path)?;
        self.parse_crate_named(crate_name, crate_path)
    }
}

/// We deduce the crate name based on the directory.
/// We expect `path` to be a `.rs` file found in some `src` directory;
/// the parent of the src is the crate name.
///
/// Really we should look at the toml file.
fn extract_crate_name(rs_path: &Path) -> crate::Result<Name> {
    if rs_path.extension().is_none() || rs_path.extension().unwrap() != "rs" {
        return Err(Error::InvalidPath(rs_path.to_owned()));
    }

    if !rs_path.is_file() {
        return Err(Error::InvalidPath(rs_path.to_owned()));
    }

    let mut parents = std::iter::from_fn({
        let mut p = rs_path;
        move || {
            p = p.parent()?;
            Some(p)
        }
    });

    while let Some(parent) = parents.next() {
        if let Some(f) = parent.file_name() {
            if f == "src" {
                break;
            }
        }
    }

    let Some(crate_path) = parents.next() else {
        return Err(Error::InvalidPath(rs_path.to_owned()));
    };

    let Some(crate_name) = crate_path.file_name() else {
        return Err(Error::InvalidPath(rs_path.to_owned()));
    };

    Ok(Name::try_from(crate_name)?)
}

#[derive(Default)]
struct AstArena {
    files: typed_arena::Arena<syn::File>,
}

impl AstArena {
    fn parse_file(&self, path: &Path) -> crate::Result<&syn::File> {
        let contents = std::fs::read_to_string(path)?;
        let file = syn::parse_file(&contents)?;
        Ok(self.files.alloc(file))
    }
}

/// Internal intermediate structure representing some kind of public user-visible definition.
struct Definition<'p> {
    /// The syn module from which this was parsed.
    module: &'p syn::File,

    /// The path which the definition was parsed from.
    source: SourcePath,

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
    FileModule,
}

/// Pass 1: Recognize types, imports, and things. Don't fill out the details (fields, methods).
mod pass1;

/// Pass 2: Fill out the fields, methods, etc. In this pass we resolve types.
mod pass2;

mod known_rust;

mod util;
