use dyn_clone::{clone_trait_object, DynClone};

mod append_single;
mod lazy_list;
mod lazy_sort;
mod partition;

pub use self::{
    append_single::append_with_last, lazy_list::LazyList, lazy_sort::merge_pairs_by,
    partition::partition,
};

/// Clone-able [Iterator] which can be used in dyn context.
pub trait CloneableIterator: Iterator + DynClone {}

impl<I: Iterator + DynClone> CloneableIterator for I {}

clone_trait_object!(<T> CloneableIterator<Item = T>);
