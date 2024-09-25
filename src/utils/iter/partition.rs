use super::CloneableIterator;

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
