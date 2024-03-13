//! Defines some ways to change the given [`Music`] value
//! turning it into another [`Music`] value.
//!
//! See more:
//! - <https://en.wikipedia.org/wiki/Transformation_(music)>
//! - <https://en.wikipedia.org/wiki/Permutation_(music)>

use crate::prim::{duration::Dur, interval::Interval, pitch::AbsPitch};

use super::{Music, Primitive, Temporal as _};

impl Music {
    /// In contrast to the annotation of the [`Music`] with [`Transpose`][`Self::with_transpose`]
    /// this function actually changes each note in a [`Music`] value by
    /// transposing it by the interval specified.
    ///
    /// This function also has its operations counterparts:
    /// - shift right (`>>`) for the positive transposition (the given function alias);
    /// - shift left (`<<`) for the negative transposition.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Transposition_(music)>
    pub fn trans(self, delta: Interval) -> Self {
        match self {
            Self::Prim(Primitive::Note(duration, pitch)) => {
                Self::note(duration, pitch.trans(delta))
            }
            Self::Prim(Primitive::Rest(duration)) => Self::rest(duration),
            Self::Sequential(m1, m2) => m1.trans(delta) + m2.trans(delta),
            Self::Parallel(m1, m2) => m1.trans(delta) | m2.trans(delta),
            Self::Modify(control, m) => m.trans(delta).with(control),
        }
    }

    /// Get the inverted [`Music`].
    ///
    /// Also could be used in the form `!Music`
    ///
    /// See more: <https://en.wikipedia.org/wiki/Inversion_(music)#Melodies>
    pub fn invert(self) -> Self {
        let line = Vec::from(self.clone());
        if let Some(Self::Prim(Primitive::Note(_, first_pitch))) = line.first() {
            let first_pitch = *first_pitch;
            let inv = |m| {
                if let Self::Prim(Primitive::Note(d, p)) = m {
                    let inverted_pitch = 2 * u16::from(first_pitch.abs().get_inner())
                        - u16::from(p.abs().get_inner());
                    // TODO: prevent u8 overflow, and then u7 overflow
                    let inverted_pitch =
                        u8::try_from(inverted_pitch).expect("TODO: take highest pitch on overflow");
                    let inverted_pitch = AbsPitch::from(ux2::u7::new(inverted_pitch));
                    Self::note(d, inverted_pitch.into())
                } else {
                    m
                }
            };
            Self::line(line.into_iter().map(inv).collect())
        } else {
            self
        }
    }

    pub fn retro_invert(self) -> Self {
        self.invert().retrograde()
    }

    pub fn invert_retro(self) -> Self {
        self.retrograde().invert()
    }
}

impl<P> Music<P> {
    /// Prepends the [`Music`] with the rest of given [`Dur`].
    ///
    /// Also could be used in the form `dur + Music`
    pub fn with_delay(self, dur: Dur) -> Self {
        Self::rest(dur) + self
    }

    pub fn retrograde(self) -> Self {
        Self::line(Vec::from(self).into_iter().rev().collect())
    }

    /// Play the [`Music`] backwards.
    ///
    /// Also could be used in the form `-Music`
    pub fn reverse(self) -> Self {
        match self {
            n @ Self::Prim(_) => n,
            Self::Sequential(m1, m2) => m2.reverse() + m1.reverse(),
            Self::Parallel(m1, m2) => {
                let d1 = m1.duration();
                let d2 = m2.duration();
                if d1 > d2 {
                    m1.reverse() | (Self::rest(d1 - d2) + m2.reverse())
                } else {
                    (Self::rest(d2 - d1) + m1.reverse()) | m2.reverse()
                }
            }
            Self::Modify(c, m) => m.reverse().with(c),
        }
    }
}

impl<P: Clone> Music<P> {
    /// Repeats the [`Music`] the given amount of times.
    ///
    /// Also could be used in the form `Music * n`.
    pub fn times(&self, n: usize) -> Self {
        // TODO: think about an 'infinite' Music
        std::iter::repeat(self.clone())
            .take(n)
            .fold(Self::rest(Dur::ZERO), |acc, m| acc + m)
    }
}
