use std::{cmp::Ordering, fmt, ops::Deref};

use super::CloneableIterator;

/// Wrapper around an iterator with additional abilities like cloning.
pub struct LazyList<T>(Box<dyn CloneableIterator<Item = T>>);

impl<T> LazyList<T> {
    /// Create a new [`LazyList`].
    pub fn new(it: impl Iterator<Item = T> + Clone + 'static) -> Self {
        Self(Box::new(it))
    }
}

impl<T> Clone for LazyList<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> PartialEq for LazyList<T> {
    fn eq(&self, _other: &Self) -> bool {
        // TODO: define rules for partial equality
        false
    }
}

impl<T> PartialOrd for LazyList<T> {
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        // TODO: define rules for partial cmp
        None
    }
}

impl<T> Deref for LazyList<T> {
    type Target = Box<dyn CloneableIterator<Item = T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> fmt::Debug for LazyList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (lower_bound, upper_bound) = self.size_hint();
        let size: Box<dyn fmt::Debug> = if upper_bound == Some(lower_bound) {
            Box::new(lower_bound)
        } else if let Some(upper) = upper_bound {
            Box::new(lower_bound..=upper)
        } else {
            Box::new(lower_bound..)
        };

        let name = format!("LazyList<{}>", std::any::type_name::<T>());
        f.debug_struct(&name).field("size", &size).finish()
    }
}

impl<T> Iterator for LazyList<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<T> LazyList<T> {
    /// Extend the [`LazyList`] with items from [`IntoIterator`] type.
    ///
    /// Cannot `impl<T> std::iter::Extend<T>`
    /// because of the additional requirement below
    /// `where I::IntoIter: Clone + 'static`
    pub fn extend<I>(&mut self, iter: I)
    where
        T: 'static,
        I: IntoIterator<Item = T>,
        I::IntoIter: Clone + 'static,
    {
        let mut content: Box<dyn CloneableIterator<Item = T>> = Box::new(std::iter::empty());
        std::mem::swap(&mut self.0, &mut content);
        self.0 = Box::new(content.chain(iter));
    }
}
