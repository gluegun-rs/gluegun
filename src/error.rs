use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("parse error: {0}")]
    Parse(#[from] syn::Error),

    #[error("lex error: {0}")]
    LexError(#[from] proc_macro2::LexError),

    #[error("generics not permitted")]
    GenericsNotPermitted(proc_macro2::Span),
}

pub type Result<T> = std::result::Result<T, Error>;