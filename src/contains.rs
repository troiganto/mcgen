use std::cmp::PartialOrd;

/// Helper trait that makes range checks beautiful.
///
/// It is implemented for tuples of types that are `PartialOrd`.
///
/// # Examples
/// ```
/// assert!((0, 10).contains(5));
/// assert!(!(0, 10).contains(12));
/// ```
pub trait Contains<T> {
    /// Returns `true` if `value` lies in this range.
    ///
    /// The exact range check is `low <= x && x <= high`.
    fn contains(&self, value: T) -> bool;
}

impl<T> Contains<T> for (T, T)
where
    for<'a, 'b> &'a T: PartialOrd<&'b T>,
{
    fn contains(&self, value: T) -> bool {
        &self.0 <= &value && &value <= &self.1
    }
}

impl<'t, T> Contains<&'t T> for (T, T)
where
    for<'a, 'b> &'a T: PartialOrd<&'b T>,
{
    fn contains(&self, value: &'t T) -> bool {
        &self.0 <= value && value <= &self.1
    }
}
