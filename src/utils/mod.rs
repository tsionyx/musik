//! Utilities independent from music domain.
mod iter;
mod measure;
mod r#ref;

pub use self::{
    iter::{CloneableIterator, LazyList},
    measure::Measure,
};

pub(crate) use self::r#ref::to_static;
