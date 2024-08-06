use std::{
    cmp::Ordering,
    ops::{Add, Div, Mul, Sub},
};

use num_rational::Ratio;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// [Duration][Dur] is the length of time a pitch, or tone, is sounded.
///
/// See more: <https://en.wikipedia.org/wiki/Duration_(music)>
pub struct Dur(u8, u8);

impl PartialOrd for Dur {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Dur {
    fn cmp(&self, other: &Self) -> Ordering {
        self.into_ratio::<u8>().cmp(&other.into_ratio())
    }
}

#[macro_export]
/// Define a [duration][Dur]
/// using a division syntax to express fractions:
///
/// ```
/// # use musik::{dur, Dur};
/// assert_eq!(dur!(1/16), Dur::recip(16));
/// assert_eq!(dur!(3:32), Dur::DOTTED_SIXTEENTH);
/// ```
macro_rules! dur {
    ($x:literal / $y:expr) => {
        Dur::new($x, $y)
    };
    ($x:literal : $y:expr) => {
        Dur::new($x, $y)
    };
}

impl Dur {
    const fn from_integer(i: u8) -> Self {
        Self(i, 1)
    }

    /// Low level constructor for [`Dur`].
    ///
    /// It is almost always better to use predefined constants,
    /// or functions. Also the [handy macro][crate::dur] is available.
    pub const fn new(num: u8, denom: u8) -> Self {
        Self(num, denom)
    }

    /// Convert a [`Dur`] into a [`Ratio`]
    /// of any type `T` that can be constructed from `u8`.
    pub fn into_ratio<T>(self) -> Ratio<T>
    where
        T: From<u8> + Clone + num_integer::Integer,
    {
        Ratio::new(T::from(self.0), T::from(self.1))
    }

    /// Zero [`Dur`] is a no-op.
    /// Not very useful in real composition, but used intensively in functions.
    pub const ZERO: Self = Self::from_integer(0);

    // see all the constants here: <https://en.wikipedia.org/wiki/Note_value>

    /// Quadruple whole note. Lasts four times as long as a [whole note][Self::WHOLE].
    ///
    /// See more: <https://en.wikipedia.org/wiki/Longa_(music)>
    pub const LONGA: Self = Self::from_integer(4);

    /// Double whole note. Lasts two times as long as a [whole note][Self::WHOLE].
    ///
    /// See more: <https://en.wikipedia.org/wiki/Double_whole_note>
    pub const BREVIS: Self = Self::from_integer(2);

    /// Single note duration.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Whole_note>
    pub const WHOLE: Self = Self::from_integer(1);

    /// Half the duration of a [whole note][Self::WHOLE].
    ///
    /// Aka _minim_.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Half_note>
    pub const HALF: Self = Self::new(1, 2);

    /// One quarter of the duration of a [whole note][Self::WHOLE].
    ///
    /// Aka _crotchet_.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Quarter_note>
    pub const QUARTER: Self = Self::new(1, 4);

    /// One eight of the duration of a [whole note][Self::WHOLE].
    ///
    /// Aka _quaver_.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Eighth_note>
    pub const EIGHTH: Self = Self::new(1, 8);

    /// One half of the duration of a [eighth note][Self::EIGHTH].
    ///
    /// Aka _semiquaver_.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Sixteenth_note>
    pub const SIXTEENTH: Self = Self::new(1, 16);

    /// One quarter of the duration of a [eighth note][Self::EIGHTH].
    ///
    /// Aka _demisemiquaver_.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Thirty-second_note>
    pub const THIRTY_SECOND: Self = Self::new(1, 32);

    /// One eight of the duration of a [eighth note][Self::EIGHTH].
    ///
    /// Aka _hemidemisemiquaver_.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Sixty-fourth_note>
    pub const SIXTY_FOURTH: Self = Self::new(1, 64);

    // dotted notes are simply increased on a half of its duration
    // and double dotted on a half plus a quarter of its duration
    // See more: <https://en.wikipedia.org/wiki/Dotted_note>

    /// [BREVIS][Self::BREVIS] lengthened by 50% (multiplied by 3/2) to make `Dur(3)`.
    pub const DOTTED_BREVIS: Self = Self::from_integer(3);
    /// [WHOLE][Self::WHOLE] lengthened by 50% (multiplied by 3/2) to make `Dur(3/2)`.
    pub const DOTTED_WHOLE: Self = Self::new(3, 2);
    /// [HALF][Self::HALF] lengthened by 50% (multiplied by 3/2) to make `Dur(3/4)`.
    pub const DOTTED_HALF: Self = Self::new(3, 4);
    /// [QUARTER][Self::QUARTER] lengthened by 50% (multiplied by 3/2) to make `Dur(3/8)`.
    pub const DOTTED_QUARTER: Self = Self::new(3, 8);
    /// [EIGHTH][Self::EIGHTH] lengthened by 50% (multiplied by 3/2) to make `Dur(3/16)`.
    pub const DOTTED_EIGHTH: Self = Self::new(3, 16);
    /// [SIXTEENTH][Self::SIXTEENTH] lengthened by 50% (multiplied by 3/2) to make `Dur(3/32)`.
    pub const DOTTED_SIXTEENTH: Self = Self::new(3, 32);
    /// [THIRTY_SECOND][Self::THIRTY_SECOND] lengthened by 50% (multiplied by 3/2) to make `Dur(3/64)`.
    pub const DOTTED_THIRTY_SECOND: Self = Self::new(3, 64);

    /// [WHOLE][Self::WHOLE] lengthened by 50% twice
    /// (a half and a quarter, which is equivalent to multiplying by 7/4) to make `Dur(7/4)`.
    pub const DOUBLE_DOTTED_WHOLE: Self = Self::new(7, 4);
    /// [HALF][Self::HALF] lengthened by 50% twice
    /// (a half and a quarter, which is equivalent to multiplying by 7/4) to make `Dur(7/8)`.
    pub const DOUBLE_DOTTED_HALF: Self = Self::new(7, 8);
    /// [QUARTER][Self::QUARTER] lengthened by 50% twice
    /// (a half and a quarter, which is equivalent to multiplying by 7/4) to make `Dur(7/16)`.
    pub const DOUBLE_DOTTED_QUARTER: Self = Self::new(7, 16);
    /// [EIGHTH][Self::EIGHTH] lengthened by 50% twice
    /// (a half and a quarter, which is equivalent to multiplying by 7/4) to make `Dur(7/32)`.
    pub const DOUBLE_DOTTED_EIGHTH: Self = Self::new(7, 32);

    /// Get the [`Dur`] corresponding to `1/fraction` of note size.
    ///
    /// As the special case, the `Dur:recip(0)` is simply [`Dur::ZERO`].
    pub const fn recip(fraction: u8) -> Self {
        if fraction == 0 {
            Self::ZERO
        } else {
            Self::new(1, fraction)
        }
    }

    /// Double the duration.
    pub const fn double(self) -> Self {
        if self.1 & 1 == 0 {
            Self::new(self.0, self.1 >> 1)
        } else {
            Self::new(self.0 << 1, self.1)
        }
    }

    /// Halve the duration.
    pub const fn halve(self) -> Self {
        if self.0 & 1 == 0 {
            Self::new(self.0 >> 1, self.1)
        } else {
            Self::new(self.0, self.1 << 1)
        }
    }

    /// Increase the duration on a half (* 3/2).
    pub const fn dotted(self) -> Self {
        let self_ = self.halve();
        Self::new(self_.0 * 3, self_.1)
    }

    /// Find the difference of two [durations][Dur].
    /// If the second one is bigger, simply yield [`Self::ZERO`].
    pub fn saturating_sub(self, rhs: Self) -> Self {
        if self > rhs {
            self - rhs
        } else {
            Self::ZERO
        }
    }
}

impl From<u8> for Dur {
    fn from(value: u8) -> Self {
        Self::from_integer(value)
    }
}

impl From<Ratio<u8>> for Dur {
    fn from(value: Ratio<u8>) -> Self {
        Self::new(*value.numer(), *value.denom())
    }
}

impl Add for Dur {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        (self.into_ratio() + rhs.into_ratio()).into()
    }
}

impl Sub for Dur {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        (self.into_ratio() - rhs.into_ratio()).into()
    }
}

impl Mul<u8> for Dur {
    type Output = Self;

    fn mul(self, rhs: u8) -> Self::Output {
        (self.into_ratio() * rhs).into()
    }
}

impl Mul<Ratio<u8>> for Dur {
    type Output = Self;

    fn mul(self, rhs: Ratio<u8>) -> Self::Output {
        (self.into_ratio() * rhs).into()
    }
}

impl Div<u8> for Dur {
    type Output = Self;

    fn div(self, rhs: u8) -> Self::Output {
        (self.into_ratio() / rhs).into()
    }
}

impl Div<Ratio<u8>> for Dur {
    type Output = Self;

    fn div(self, rhs: Ratio<u8>) -> Self::Output {
        (self.into_ratio() / rhs).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn double_duration() {
        assert_eq!(Dur::BREVIS.double(), Dur::LONGA);
        assert_eq!(Dur::WHOLE.double(), Dur::BREVIS);
        assert_eq!(Dur::HALF.double(), Dur::WHOLE);
        assert_eq!(Dur::QUARTER.double(), Dur::HALF);
        assert_eq!(Dur::EIGHTH.double(), Dur::QUARTER);
        assert_eq!(Dur::SIXTEENTH.double(), Dur::EIGHTH);
        assert_eq!(Dur::THIRTY_SECOND.double(), Dur::SIXTEENTH);
        assert_eq!(Dur::SIXTY_FOURTH.double(), Dur::THIRTY_SECOND);

        assert_eq!(Dur::DOTTED_WHOLE.double(), Dur::DOTTED_BREVIS);
        assert_eq!(Dur::DOTTED_HALF.double(), Dur::DOTTED_WHOLE);
        assert_eq!(Dur::DOTTED_QUARTER.double(), Dur::DOTTED_HALF);
        assert_eq!(Dur::DOTTED_EIGHTH.double(), Dur::DOTTED_QUARTER);
        assert_eq!(Dur::DOTTED_SIXTEENTH.double(), Dur::DOTTED_EIGHTH);
        assert_eq!(Dur::DOTTED_THIRTY_SECOND.double(), Dur::DOTTED_SIXTEENTH);

        assert_eq!(Dur::DOUBLE_DOTTED_HALF.double(), Dur::DOUBLE_DOTTED_WHOLE);
        assert_eq!(Dur::DOUBLE_DOTTED_QUARTER.double(), Dur::DOUBLE_DOTTED_HALF);
        assert_eq!(
            Dur::DOUBLE_DOTTED_EIGHTH.double(),
            Dur::DOUBLE_DOTTED_QUARTER
        );
    }

    #[test]
    fn test_macro() {
        assert_eq!(dur!(1:1), Dur::WHOLE);
        assert_eq!(dur!(1:2), Dur::HALF);
        assert_eq!(dur!(1:4), Dur::QUARTER);
        assert_eq!(dur!(1:8), Dur::EIGHTH);
        assert_eq!(dur!(1:16), Dur::SIXTEENTH);
        assert_eq!(dur!(1:32), Dur::THIRTY_SECOND);
        assert_eq!(dur!(1:64), Dur::SIXTY_FOURTH);

        assert_eq!(dur!(3 / 2), Dur::DOTTED_WHOLE);
        assert_eq!(dur!(3 / 4), Dur::DOTTED_HALF);
        assert_eq!(dur!(3 / 8), Dur::DOTTED_QUARTER);
        assert_eq!(dur!(3 / 16), Dur::DOTTED_EIGHTH);
        assert_eq!(dur!(3 / 32), Dur::DOTTED_SIXTEENTH);
        assert_eq!(dur!(3 / 64), Dur::DOTTED_THIRTY_SECOND);
    }

    #[test]
    fn recip() {
        assert_eq!(Dur::recip(0), Dur::ZERO);
        assert_eq!(Dur::recip(1), Dur::WHOLE);
        assert_eq!(Dur::recip(2), Dur::HALF);
        assert_eq!(Dur::recip(4), Dur::QUARTER);
        assert_eq!(Dur::recip(8), Dur::EIGHTH);
        assert_eq!(Dur::recip(16), Dur::SIXTEENTH);
        assert_eq!(Dur::recip(32), Dur::THIRTY_SECOND);
    }
}
