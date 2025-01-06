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

    #[error("fields must either be all public or all crate-private")]
    MixedPublicPrivateFields(proc_macro2::Span),

    #[error("unrecognized Rust item")]
    UnrecognizedItem(proc_macro2::Span),

    #[error("unsupported Rust item; consider using `#[squared::ignore]`")]
    UnsupportedItem(proc_macro2::Span),

    #[error("only `self`, `&self`, and `&mut self` are supported")]
    ExplicitSelfNotSupported(proc_macro2::Span),

    #[error("macro invocations not supported")]
    MacroNotSupported(proc_macro2::Span),

    #[error("unsupported Rust type")]
    UnsupportedType(proc_macro2::Span),

    #[error("cannot resolve name (it must be public)")]
    UnresolvedName(proc_macro2::Span),

    #[error("expected a Rust type, not this")]
    NotType(proc_macro2::Span),

    #[error("anonymous fields unsupported")]
    AnonymousField(proc_macro2::Span),

    #[error("variants must have anonymous fields")]
    AnonymousFieldRequired(proc_macro2::Span),

    #[error("unsupported function input pattern, must be a single identifier")]
    UnsupportedInputPattern(proc_macro2::Span),
}

pub type Result<T> = std::result::Result<T, Error>;
