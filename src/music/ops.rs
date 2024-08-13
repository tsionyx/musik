use std::ops::{Add, BitAnd, BitOr, Div, Mul, Neg, Not, Rem, Shl, Shr};

use num_rational::Ratio;

use crate::prim::{
    duration::{Dur, DurT},
    interval::Interval,
};

use super::{Control, Music, Temporal as _};

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

impl<P> BitAnd<Control<P>> for Music<P> {
    type Output = Self;

    fn bitand(self, rhs: Control<P>) -> Self::Output {
        self.with(rhs)
    }
}

impl<P> Div for Music<P> {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    /// Truncating parallel composition
    fn div(self, rhs: Self) -> Self::Output {
        let d1 = self.duration();
        let d2 = rhs.duration();
        self.take(d2) | rhs.take(d1)
    }
}

impl Shr<Interval> for Music {
    type Output = Self;

    /// Transpose the [`Music`] by a specific number of semitones up
    /// to get all *higher* pitches.
    ///
    /// If the negative [`Interval`] provided, the effective transposition
    /// will result in a *lower* pitches instead.
    fn shr(self, rhs: Interval) -> Self::Output {
        self.trans(rhs)
    }
}

impl Shl<Interval> for Music {
    type Output = Self;

    /// Transpose the [`Music`] by a specific number of semitones down
    /// to get all *lower* pitches.
    ///
    /// If the negative [`Interval`] provided, the effective transposition
    /// will result in a *higher* pitches instead.
    fn shl(self, rhs: Interval) -> Self::Output {
        self.trans(-rhs)
    }
}

impl<P> Add<Music<P>> for Dur {
    type Output = Music<P>;

    /// Prepends the [`Music`] with the rest of given [`Dur`].
    fn add(self, rhs: Music<P>) -> Self::Output {
        rhs.with_delay(self)
    }
}

impl<P: Clone> Neg for Music<P> {
    type Output = Self;

    /// Play the [`Music`] backwards.
    fn neg(self) -> Self::Output {
        self.reverse()
    }
}

impl Not for Music {
    type Output = Self;

    /// Get the [inverted](https://en.wikipedia.org/wiki/Inversion_(music))
    /// [`Music`].
    fn not(self) -> Self::Output {
        self.invert()
    }
}

impl<P> Rem<Ratio<DurT>> for Music<P> {
    type Output = Self;

    /// Annotate the [`Music`] to change its tempo:
    /// - accelerate if `tempo` > 1;
    /// - decelerate, otherwise.
    fn rem(self, tempo: Ratio<DurT>) -> Self::Output {
        self.with_tempo(tempo)
    }
}

impl<P: Clone> Mul<usize> for Music<P> {
    type Output = Self;

    /// Repeats the [`Music`] the given amount of times.
    fn mul(self, rhs: usize) -> Self::Output {
        self.times(rhs)
    }
}
