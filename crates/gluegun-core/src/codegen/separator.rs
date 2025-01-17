/// Utility class for getting interspersed commas and other separators.
pub trait Separator: Iterator {
    /// Returns an iterator over pairs `(item, sep)` where `sep` is either `","`
    /// (for every item but the last) or `""` (for the last item).
    fn comma_separated(self) -> impl Iterator<Item = (Self::Item, &'static str)>;
}

impl<T: Iterator> Separator for T {
    fn comma_separated(self) -> impl Iterator<Item = (Self::Item, &'static str)> {
        with_separator(",", "", self)
    }
}

/// Returns an iterator over pairs `(item, s)` where `s` is either `sep`
/// (for every item but the last) or `last_sep` (for the last item).
pub fn with_separator<S, I>(
    sep: S,
    last_sep: S,
    iter: impl Iterator<Item = I>,
) -> impl Iterator<Item = (I, S)> 
where 
    S: Clone,
{
    let mut p = iter.peekable();
    std::iter::from_fn(move || {
        let item = p.next()?;

        if p.peek().is_none() {
            Some((item, last_sep.clone()))
        } else {
            Some((item, sep.clone()))
        }
    })    
}