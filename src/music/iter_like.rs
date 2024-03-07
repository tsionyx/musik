//! <https://en.wikipedia.org/wiki/Transformation_(music)>
//! <https://en.wikipedia.org/wiki/Permutation_(music)>
use crate::prim::duration::Dur;

use super::{control::Control, Music, Primitive};

impl<P> Music<P> {
    /// <https://en.wikipedia.org/wiki/Melody>
    pub fn line(musics: Vec<Self>) -> Self {
        musics
            .into_iter()
            .fold(Self::rest(Dur::ZERO), |acc, m| acc + m)
    }

    /// <https://en.wikipedia.org/wiki/Chord_progression>
    pub fn chord(musics: Vec<Self>) -> Self {
        musics
            .into_iter()
            .fold(Self::rest(Dur::ZERO), |acc, m| acc | m)
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
                let m2 = m2.take(n.saturating_sub(m1.duration()));
                m1 + m2
            }
            Self::Parallel(m1, m2) => m1.take(n) | m2.take(n),
            Self::Modify(Control::Tempo(r), m) => m.take(n * r).with_tempo(r),
            Self::Modify(c, m) => m.take(n).with(c),
        }
    }

    // TODO: trait Temporal
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
            Self::Modify(c, m) => (*m).drop(n).with(c),
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
            Self::Modify(c, m) => m.remove_zeros().with(c),
        }
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
