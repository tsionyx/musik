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
