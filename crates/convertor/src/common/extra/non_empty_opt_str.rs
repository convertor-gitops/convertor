use std::ops::Not;

pub trait NonEmptyOptStr<T> {
    fn filter_non_empty(&self) -> Option<&str>;
}

impl<T: AsRef<str>> NonEmptyOptStr<Option<T>> for Option<T> {
    fn filter_non_empty(&self) -> Option<&str> {
        self.as_ref().and_then(|s| {
            let s = s.as_ref().trim();
            s.is_empty().not().then_some(s)
        })
    }
}
