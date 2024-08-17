use std::{cmp::Ordering, fmt, ops::Deref};

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
    pub(crate) fn extend<I>(&mut self, iter: I)
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

/// Clone-able [Iterator] which can be used in dyn context.
pub trait CloneableIterator: Iterator + DynClone {}

impl<I: Iterator + DynClone> CloneableIterator for I {}

clone_trait_object!(<T> CloneableIterator<Item = T>);

pub const fn append_with_last<I, F>(iter: I, f: F) -> AppendWithLast<I, F>
where
    I: Iterator,
    I::Item: Clone,
    F: FnMut(I::Item) -> Option<I::Item>,
{
    AppendWithLast {
        iter,
        last_item: None,
        f,
    }
}

pub struct AppendWithLast<I, F>
where
    I: Iterator,
{
    iter: I,
    last_item: Option<I::Item>,
    f: F,
}

impl<I, F> Iterator for AppendWithLast<I, F>
where
    I: Iterator,
    I::Item: Clone,
    F: FnMut(I::Item) -> Option<I::Item>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.iter.next() {
            self.last_item = Some(item.clone());
            Some(item)
        } else {
            self.last_item.take().and_then(|last| (self.f)(last))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lo, hi) = self.iter.size_hint();
        (lo, hi.map(|hi| hi + 1))
    }
}
