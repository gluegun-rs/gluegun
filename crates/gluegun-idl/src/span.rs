use std::{path::PathBuf, sync::Arc};

use accessors_rs::Accessors;
use serde::{Deserialize, Serialize};
use syn::spanned::Spanned;

#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct Span {
    pub(crate) path: PathBuf,
    pub(crate) start: ErrorLocation,
    pub(crate) end: ErrorLocation,
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}:{}:{}",
            self.path.display(),
            self.start.line,
            self.start.column,
            self.end.line,
            self.end.column
        )
    }
}

#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct ErrorLocation {
    /// Byte index since start of file
    pub(crate) byte: usize,

    /// Line number (1-indexed)
    pub(crate) line: usize,

    /// Column number on line (1-indexed in utf-8 characters)
    pub(crate) column: usize,
}

/// Wrapper around a source path for constructing ErrorSpans.
#[derive(Clone, Debug)]
pub(crate) struct SourcePath {
    path: Arc<PathBuf>,
}

impl SourcePath {
    pub(crate) fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: Arc::new(path.into()),
        }
    }

    /// Create an error span from a [`Span`][].
    /// There is no stable API to access path info, so pass that separately.
    pub(crate) fn span(&self, span: impl Spanned) -> Span {
        let span = span.span();
        let byte_range = span.byte_range();
        let start = span.start();
        let end = span.end();
        Span {
            path: self.path.to_path_buf(),
            start: ErrorLocation {
                byte: byte_range.start,
                line: start.line,
                column: start.column + 1,
            },
            end: ErrorLocation {
                byte: byte_range.end,
                line: end.line,
                column: end.column + 1,
            },
        }
    }
}
