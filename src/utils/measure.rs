use std::ops::{Add, Mul};

use num_traits::{CheckedAdd, CheckedMul};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
/// Wraps the numeric category in terms of measure
/// which can be finite or infinite.
pub enum Measure<T> {
    /// The finite value.
    Finite(T),

    // infinite should be on the second place
    // in the list of enum variants
    // to automatically derive `Ord`:
    // `Measure::Infinite > Measure::Finite(x)` for all `x` from `T`
    /// Infinite value which absorbs any [`Finite`][Self::Finite] ones.
    Infinite,
}

impl<T: Default> Default for Measure<T> {
    fn default() -> Self {
        Self::Finite(T::default())
    }
}

impl<T> Add<T> for Measure<T>
where
    T: CheckedAdd<Output = T>,
{
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        if let Self::Finite(x) = self {
            if let Some(sum) = x.checked_add(&rhs) {
                return Self::Finite(sum);
            }
        }
        Self::Infinite
    }
}

impl<T> Add for Measure<T>
where
    T: CheckedAdd<Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if let Self::Finite(y) = rhs {
            self + y
        } else {
            Self::Infinite
        }
    }
}

impl<T> Mul<T> for Measure<T>
where
    T: CheckedMul<Output = T>,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        if let Self::Finite(x) = self {
            if let Some(mul) = x.checked_mul(&rhs) {
                return Self::Finite(mul);
            }
        }
        Self::Infinite
    }
}

impl<T> Mul for Measure<T>
where
    T: CheckedMul<Output = T>,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        if let Self::Finite(y) = rhs {
            self * y
        } else {
            Self::Infinite
        }
    }
}

impl<T> From<T> for Measure<T> {
    fn from(value: T) -> Self {
        Self::Finite(value)
    }
}

impl<T: PartialOrd> Measure<T> {
    pub(crate) fn max_in_iter(iter: impl Iterator<Item = Self>) -> Option<Self> {
        let mut max = None;
        for x in iter {
            if let Self::Finite(x) = x {
                if let Some(current_max) = max.as_ref() {
                    if &x > current_max {
                        max = Some(x);
                    }
                } else {
                    max = Some(x);
                }
            } else {
                return Some(x);
            }
        }

        max.map(Self::Finite)
    }
}
