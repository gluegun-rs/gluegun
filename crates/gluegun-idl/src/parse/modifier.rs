use crate::RefKind;



#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(super) enum Modifier {
    /// `&T`,  `impl AsRef<T>`
    Ref(RefKind),
}