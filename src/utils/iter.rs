use std::{cmp::Ordering, collections::BinaryHeap, fmt, iter::Peekable, ops::Deref};

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

/// Produce sorted iterator by zipping the iterator of pairs `(a, b)`.
///
/// The precondition is that the pairs' 'minimum' elements
/// taken alone should be sorted: `for all i: (min(a[i], b[i]) <= min(a[i+1], b[i+1]))`.
///
/// So, the main challenge is to ensure the stream of second elements (`max(a, b)`)
/// gets inserted in the (already sorted) stream of (`min(a, b)`).
///
/// Elements in pairs could also be internally sorted `for all i: (a[i] <= b[i])`,
/// however this is not required, because it is a cheap operation
/// and it is made if necessary.
///
/// # Attention
/// If the precondition of sorted min elements does not hold,
/// it is impossible to forecast
/// whether less-y value occurred before fully consuming the iterator,
/// so the iterator's order will be incorect.
pub fn merge_pairs_by<I, T, F>(iter: I, is_first: F) -> MergePairsBy<I, T, F>
where
    I: Iterator<Item = (T, T)>,
    F: Fn(&T, &T) -> bool,
{
    MergePairsBy::new(iter, is_first)
}

pub struct MergePairsBy<I, T, F>
where
    I: Iterator<Item = (T, T)>,
{
    iter: Peekable<I>,
    pending: BinaryHeap<OrdFromKeyWrapper<T>>,
    is_first_fn: F,
}

type IsFirstFn<T> = Box<dyn Fn(&T, &T) -> bool>;

struct OrdFromKeyWrapper<T> {
    item: T,
    less_fn: IsFirstFn<T>
}

impl<T> PartialEq for OrdFromKeyWrapper<T> {
    fn eq(&self, other: &Self) -> bool {
        !(self.less_fn)(&self.item, &other.item)
            && !(self.less_fn)(&other.item, &self.item)
            && !(other.less_fn)(&self.item, &other.item)
            && !(other.less_fn)(&other.item, &self.item)
    }
}

impl<T> Eq for OrdFromKeyWrapper<T> {}

impl<T> PartialOrd for OrdFromKeyWrapper<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for OrdFromKeyWrapper<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        if (self.less_fn)(&self.item, &other.item) {
            // the first value for our condition
            // should be the greatest to be on top of the queue
            Ordering::Greater
        } else if (self.less_fn)(&other.item, &self.item) {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}

impl<I, T, F> MergePairsBy<I, T, F>
where
    I: Iterator<Item = (T, T)>,
{
    fn new(iter: I, is_first_fn: F) -> Self {
        Self {
            iter: iter.peekable(),
            pending: BinaryHeap::new(),
            is_first_fn,
        }
    }
}

impl<I, T, F> Iterator for MergePairsBy<I, T, F>
where
    I: Iterator<Item = (T, T)>,
    F: Fn(&T, &T) -> bool + Clone + 'static,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next_item) = self.iter.peek() {
            let (a, b) = (&next_item.0, &next_item.1);
            let min = if (self.is_first_fn)(b, a) { b } else { a };

            let min_pending = self.pending.peek();
            if let Some(pending) = min_pending {
                // return from pending while it is less than current pair's min
                if !(self.is_first_fn)(min, &pending.item) {
                    return self.pending.pop().map(|x| x.item);
                }
            }
        }

        if let Some((mut a, mut b)) = self.iter.next() {
            if (self.is_first_fn)(&b, &a) {
                std::mem::swap(&mut a, &mut b);
            }

            self.pending.push(OrdFromKeyWrapper {
                item: b,
                less_fn: Box::new(self.is_first_fn.clone()),
            });
            Some(a)
        } else {
            self.pending.pop().map(|x| x.item)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lo, hi) = self.iter.size_hint();
        (lo * 2, hi.map(|hi| hi * 2))
    }
}

pub fn partition<I, T, F, TakeF>(
    iter: I,
    predicate: F,
    take_only: Option<TakeF>,
) -> (
    impl CloneableIterator<Item = T>,
    impl CloneableIterator<Item = T>,
)
where
    I: Iterator<Item = T> + Clone,
    F: Fn(&T) -> bool + Clone + 'static,
    TakeF: Fn(&T) -> bool + Clone + 'static,
{
    let take = move |x: &T| take_only.as_ref().map_or(true, |take_only| take_only(x));
    let filter = move |x: &T| predicate(x);

    let left = iter.clone().take_while(take.clone()).filter(filter.clone());
    let right = iter.take_while(take).filter(move |x| !filter(x));
    (left, right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_pairs() {
        #[derive(Debug, Clone)]
        struct X {
            order: u32,
            id: char,
        }

        impl X {
            const fn new(order: u32, id: char) -> Self {
                Self { order, id }
            }
        }

        let pairs = vec![
            (X::new(0, 'a'), X::new(0, 'b')),
            (X::new(0, 'c'), X::new(4, 'd')),
            (X::new(0, 'e'), X::new(2, 'f')),
            (X::new(0, 'g'), X::new(0, 'h')),
            (X::new(3, 'i'), X::new(5, 'j')),
            (X::new(3, 'k'), X::new(4, 'l')),
            (X::new(8, 'm'), X::new(5, 'n')),
            (X::new(8, 'o'), X::new(8, 'p')),
            (X::new(8, 'q'), X::new(9, 'r')),
        ];

        let (orders, ids): (Vec<_>, String) =
            merge_pairs_by(pairs.iter().cloned(), |x1, x2| x1.order < x2.order)
                .map(|X { order, id }| (order, id))
                .unzip();

        assert_eq!(
            orders,
            [0, 0, 0, 0, 0, 0, 2, 3, 3, 4, 4, 5, 5, 8, 8, 8, 8, 9]
        );
        // the ids are: "000000233445588889"
        assert_eq!(ids, "abceghfikdljnmopqr");
        // or (in case of non-stable insert into the internal priority queue)
        // assert_eq!(ids, "abceghfikldjnmopqr");

        let mut flat: Vec<_> = pairs.into_iter().flat_map(<[X; 2]>::from).collect();
        flat.sort_by_key(|x| x.order);
        let (orders2, ids2): (Vec<_>, String) =
            flat.into_iter().map(|X { order, id }| (order, id)).unzip();
        assert_eq!(orders, orders2);
        assert_eq!(ids, ids2);
    }
}
