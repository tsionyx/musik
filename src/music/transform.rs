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
        self.map(move |pitch| pitch.trans(delta))
    }

    /// Get the inverted [musical line][Self::line]
    /// where all the pitch intervals _from the first note_
    /// replaced with their simple arithmetic inverses (-).
    ///
    /// It resembles in some sense the playing of the 'upside-down' music.
    /// Also could be used in the form `!Music`
    ///
    /// See more: <https://en.wikipedia.org/wiki/Inversion_(music)#Melodies>
    pub fn invert(self) -> Self {
        let line = Vec::from(self.clone());
        if let Some(Self::Prim(Primitive::Note(_, first_pitch))) = line.first() {
            let first_pitch = *first_pitch;
            let inv = move |m| {
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
            Self::lazy_line(line.into_iter().map(inv))
        } else {
            self
        }
    }

    /// [Playing the reversed version][Self::retrograde]
    /// of the [inverted][Self::invert] [musical line][Self::line].
    ///
    /// In other words, it is a function composition `retrograde * invert`.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Retrograde_inversion>
    pub fn retro_invert(self) -> Self {
        self.invert().retrograde()
    }

    /// [Invert][Self::invert] the [reversed][Self::retrograde] [musical line][Self::line].
    ///
    /// In other words, it is a function composition `invert * retrograde`.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Retrograde_inversion>
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

    /// Playing the reversed version of a simple [musical line][Self::line].
    ///
    /// It is more simple version of the more exact [`Self::reverse`].
    pub fn retrograde(self) -> Self
    where
        P: Clone,
    {
        Self::lazy_line(Vec::from(self).into_iter().rev())
    }

    /// Play the [`Music`] backwards.
    ///
    /// In contrast with the [`Self::retrograde`]
    /// it supports reversing the [`Music`] with arbitrary nesting,
    /// not just the simple [succession of values][Self::line].
    ///
    /// Also could be used in the form `-Music`
    pub fn reverse(self) -> Self
    where
        P: Clone,
    {
        self.fold(
            Self::Prim,
            |m1, m2| m2 + m1,
            (Self::rest(Dur::ZERO), |acc, m| m + acc),
            |m1, m2| {
                let d1 = m1.duration();
                let d2 = m2.duration();
                if d1 > d2 {
                    m1 | (Self::rest(d1 - d2) + m2)
                } else {
                    (Self::rest(d2 - d1) + m1) | m2
                }
            },
            |c, m| m.with(c),
        )
    }
}

impl<P: Clone> Music<P> {
    /// Repeats the [`Music`] the given amount of times.
    ///
    /// Also could be used in the form `Music * n`.
    pub fn times(&self, n: usize) -> Self {
        Self::lazy_line(std::iter::repeat(self.clone()).take(n))
    }
}
