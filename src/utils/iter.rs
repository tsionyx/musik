use std::{cmp::Ordering, fmt};

use dyn_clone::{clone_trait_object, DynClone};

/// Wrapper around an iterator with additional abilities like cloning.
pub struct LazyList<T>(pub(crate) Box<dyn CloneableIterator<Item = T>>);

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

impl<T> Eq for LazyList<T> {}

impl<T> PartialOrd for LazyList<T> {
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        // TODO: define rules for partial cmp
        None
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

/// Clone-able [Iterator] which can be used in dyn context.
pub trait CloneableIterator: Iterator + DynClone {}

impl<I: Iterator + DynClone> CloneableIterator for I {}

clone_trait_object!(<T> CloneableIterator<Item = T>);
