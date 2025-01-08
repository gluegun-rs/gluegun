use std::{ffi::OsString, path::PathBuf};

use thiserror::Error;

use crate::ErrorSpan;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("{0}: generics not permitted")]
    GenericsNotPermitted(ErrorSpan),

    #[error("{0}: fields must either be all public or all crate-private")]
    MixedPublicPrivateFields(ErrorSpan),

    #[error("{0}: unrecognized Rust item")]
    UnrecognizedItem(ErrorSpan),

    #[error("{0}: unsupported Rust item; consider using `#[squared::ignore]`")]
    UnsupportedItem(ErrorSpan),

    #[error("{0}: only `self`, `&self`, and `&mut self` are supported")]
    ExplicitSelfNotSupported(ErrorSpan),

    #[error("{0}: macro invocations not supported")]
    MacroNotSupported(ErrorSpan),

    #[error("{0}: unsupported Rust type")]
    UnsupportedType(ErrorSpan),

    #[error("{0}: cannot resolve name (it must be public)")]
    UnresolvedName(ErrorSpan),

    #[error("{0}: expected a Rust type, not this")]
    NotType(ErrorSpan),

    #[error("{0}: anonymous fields unsupported")]
    AnonymousField(ErrorSpan),

    #[error("{0}: variants must have anonymous fields")]
    AnonymousFieldRequired(ErrorSpan),

    #[error("{0}: unsupported function input pattern, must be a single identifier")]
    UnsupportedInputPattern(ErrorSpan),

    #[error("{0}: expected to be invoked with a path like `foo/src/../*.rs`, found")]
    InvalidPath(PathBuf),

    #[error("path component could not be converted to a string")]
    NotUtf8(OsString),
}

impl From<syn::Error> for Error {
    fn from(value: syn::Error) -> Self {
        Error::Parse(value.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
