//! <https://en.wikipedia.org/wiki/Transformation_(music)>
//! <https://en.wikipedia.org/wiki/Permutation_(music)>

use crate::prim::{duration::Dur, interval::Interval, pitch::AbsPitch};

use super::{Music, Primitive};

impl Music {
    // TODO: >> and << operator for transpose

    /// Exercise 2.5
    ///
    /// In contrast to the annotation of the `Music` with `Transpose`
    /// (as part of the `Control` data type, which in turn is part of the `Music`),
    /// this function actually changes each note in a `Music<Pitch>` value by
    /// transposing it by the interval specified.
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

    // TODO: operation -
    /// <https://en.wikipedia.org/wiki/Inversion_(music)#Melodies>
    pub fn invert(self) -> Self {
        let line = Vec::from(self.clone());
        if let Some(Self::Prim(Primitive::Note(_, first_pitch))) = line.first() {
            let first_pitch = *first_pitch;
            let inv = |m| {
                if let Self::Prim(Primitive::Note(d, p)) = m {
                    // prevent i8 overflow
                    let inverted_pitch = 2 * i16::from(first_pitch.abs().get_inner())
                        - i16::from(p.abs().get_inner());
                    let inverted_pitch = AbsPitch::from(inverted_pitch as i8);
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
    /// TODO: shorten or provide operator
    pub fn with_delay(self, dur: Dur) -> Self {
        Self::rest(dur) + self
    }

    pub fn retrograde(self) -> Self {
        Self::line(Vec::from(self).into_iter().rev().collect())
    }

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
    // TODO: implement operation * and / for tempo changing
    pub fn times(&self, n: usize) -> Self {
        // TODO: think about an 'infinite' Music
        std::iter::repeat(self.clone())
            .take(n)
            .fold(Self::rest(Dur::ZERO), |acc, m| acc + m)
    }
}
