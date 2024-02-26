use std::ops::{Add, BitOr, Div};

use super::Music;

/// Sequential composition
impl<P> Add for Music<P> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Sequential(Box::new(self), Box::new(rhs))
    }
}

/// Parallel composition
impl<P> BitOr for Music<P> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::Parallel(Box::new(self), Box::new(rhs))
    }
}

/// Truncating parallel composition
impl<P> Div for Music<P> {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, rhs: Self) -> Self::Output {
        let d1 = self.duration();
        let d2 = rhs.duration();
        self.take(d2) | rhs.take(d1)
    }
}
