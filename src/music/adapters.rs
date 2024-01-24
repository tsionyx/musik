use num_rational::Ratio;

use super::{
    duration::Dur,
    interval::{AbsPitch, Interval},
    Control, Music, Primitive,
};

impl Music {
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
            Self::Modify(control, m) => Self::Modify(control, Box::new(m.trans(delta))),
        }
    }

    pub fn grace_note(&self, offset: AbsPitch, grace_fraction: Ratio<u8>) -> Result<Self, String> {
        if let Self::Prim(Primitive::Note(d, p)) = self {
            Ok(Self::note(*d * grace_fraction, p.trans(offset.into()))
                + Self::note(*d * (Ratio::from_integer(1) - grace_fraction), *p))
        } else {
            Err("Can only add a grace note to a note".into())
        }
    }

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
    pub fn line(musics: Vec<Self>) -> Self {
        musics
            .into_iter()
            .fold(Self::rest(Dur::ZERO), |acc, m| acc + m)
    }

    pub fn chord(musics: Vec<Self>) -> Self {
        musics
            .into_iter()
            .fold(Self::rest(Dur::ZERO), |acc, m| acc | m)
    }

    pub fn with_dur(pitches: Vec<P>, dur: Dur) -> Self {
        Self::line(
            pitches
                .into_iter()
                .map(|pitch| Self::note(dur, pitch))
                .collect(),
        )
    }

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
            Self::Modify(c, m) => Self::Modify(c, Box::new(m.reverse())),
        }
    }

    /// Take the first N whole beats and drop the other
    pub fn take(self, n: Dur) -> Self {
        if n == Dur::ZERO {
            return Self::rest(Dur::ZERO);
        }

        match self {
            Self::Prim(Primitive::Note(d, p)) => Self::note(d.min(n), p),
            Self::Prim(Primitive::Rest(d)) => Self::rest(d.min(n)),
            Self::Sequential(m1, m2) => {
                let m1 = m1.take(n);
                let m2 = m2.take(n - m1.duration());
                m1 + m2
            }
            Self::Parallel(m1, m2) => m1.take(n) | m2.take(n),
            Self::Modify(Control::Tempo(r), m) => m.take(n * r).with_tempo(r),
            Self::Modify(c, m) => Self::Modify(c, Box::new(m.take(n))),
        }
    }

    /// Drop the first N whole beats and take the other
    pub fn drop(self, n: Dur) -> Self {
        if n == Dur::ZERO {
            return self;
        }

        match self {
            Self::Prim(Primitive::Note(d, p)) => Self::note(d.saturating_sub(n), p),
            Self::Prim(Primitive::Rest(d)) => Self::rest(d.saturating_sub(n)),
            Self::Sequential(m1, m2) => {
                let m2 = (*m2).drop(n.saturating_sub(m1.duration()));
                (*m1).drop(n) + m2
            }
            Self::Parallel(m1, m2) => (*m1).drop(n) | (*m2).drop(n),
            Self::Modify(Control::Tempo(r), m) => (*m).drop(n * r).with_tempo(r),
            Self::Modify(c, m) => Self::Modify(c, Box::new((*m).drop(n))),
        }
    }

    pub fn remove_zeros(self) -> Self {
        match self {
            n @ Self::Prim(_) => n,
            Self::Sequential(m1, m2) => match (m1.remove_zeros(), m2.remove_zeros()) {
                (Self::Prim(Primitive::Note(Dur::ZERO, _)), m) => m,
                (Self::Prim(Primitive::Rest(Dur::ZERO)), m) => m,
                (m, Self::Prim(Primitive::Note(Dur::ZERO, _))) => m,
                (m, Self::Prim(Primitive::Rest(Dur::ZERO))) => m,
                (m1, m2) => m1 + m2,
            },
            Self::Parallel(m1, m2) => match (m1.remove_zeros(), m2.remove_zeros()) {
                (Self::Prim(Primitive::Note(Dur::ZERO, _)), m) => m,
                (Self::Prim(Primitive::Rest(Dur::ZERO)), m) => m,
                (m, Self::Prim(Primitive::Note(Dur::ZERO, _))) => m,
                (m, Self::Prim(Primitive::Rest(Dur::ZERO))) => m,
                (m1, m2) => m1 | m2,
            },
            Self::Modify(c, m) => Self::Modify(c, Box::new(m.remove_zeros())),
        }
    }
}

impl<P: Clone> Music<P> {
    pub fn times(&self, n: usize) -> Self {
        // TODO: think about an 'infinite' Music
        std::iter::repeat(self.clone())
            .take(n)
            .fold(Self::rest(Dur::ZERO), |acc, m| acc + m)
    }
}

impl<P> From<Music<P>> for Vec<Music<P>> {
    fn from(value: Music<P>) -> Self {
        match value {
            Music::Prim(Primitive::Rest(Dur::ZERO)) => vec![],
            Music::Sequential(m1, m2) => {
                Self::from(*m1).into_iter().chain(Self::from(*m2)).collect()
            }
            other => vec![other],
        }
    }
}
