use std::{ffi::OsString, path::PathBuf};

use thiserror::Error;

use crate::{Name, RefKind, Span};

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("{0}: generics not permitted")]
    GenericsNotPermitted(Span),

    #[error("{0}: expected associated type binding `{1}=T` not found")]
    BindingNotFound(Span, Name),

    #[error("{0}: unexpected associated type binding")]
    BindingNotExpected(Span),

    #[error("{0}: fields must either be all public or all crate-private")]
    MixedPublicPrivateFields(Span),

    #[error("{0}: unrecognized Rust item")]
    UnrecognizedItem(Span),

    #[error("{0}: recognised this type but not number of arguments (expected {1}, found {2}")]
    UnsupportedNumberOfArguments(Span, usize, usize),

    #[error("{0}: unsupported Rust item; consider using `#[gluegun::ignore]`")]
    UnsupportedItem(Span),

    #[error("{0}: only `self`, `&self`, and `&mut self` are supported")]
    ExplicitSelfNotSupported(Span),

    #[error("{0}: macro invocations not supported")]
    MacroNotSupported(Span),

    #[error("{0}: unsupported Rust type")]
    UnsupportedType(Span),

    #[error("{0}: Rust type recognized but not used in expected way or with expected arguments")]
    UnsupportedUseOfType(Span),

    #[error("{0}: cannot resolve name (it must be public)")]
    UnresolvedName(Span),

    #[error("{0}: expected a Rust type, not this")]
    NotType(Span),

    #[error("{0}: anonymous fields unsupported")]
    AnonymousField(Span),

    #[error("{0}: unsupported function input pattern, must be a single identifier")]
    UnsupportedInputPattern(Span),

    #[error("{0}: expected to be invoked with a path like `foo/src/../*.rs`, found")]
    InvalidPath(PathBuf),

    #[error("path component could not be converted to a string")]
    NotUtf8(OsString),

    #[error("async functions cannot return `impl Future`")]
    DoubleAsync(Span),

    #[error("{0}: only owned types are permitted here, not `{1}`-types")]
    ReferenceType(Span, RefKind),
}

impl From<syn::Error> for Error {
    fn from(value: syn::Error) -> Self {
        Error::Parse(value.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
