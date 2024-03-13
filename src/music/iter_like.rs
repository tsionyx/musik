use crate::prim::duration::Dur;

use super::{control::Control, Music, Primitive};

impl<P> Music<P> {
    /// Linear succession of musical parts.
    /// One of the most basic form of composition.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Melody>
    pub fn line(musics: Vec<Self>) -> Self {
        musics
            .into_iter()
            .fold(Self::rest(Dur::ZERO), |acc, m| acc + m)
    }

    /// A set of musical parts that are supposed to play simultaneously.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Chord_(music)>
    pub fn chord(musics: Vec<Self>) -> Self {
        musics
            .into_iter()
            .fold(Self::rest(Dur::ZERO), |acc, m| acc | m)
    }

    /// Strip away the [`Dur::ZERO`] occurrences that could appear
    /// during composition and [transformations][super::transform].
    pub fn remove_zeros(self) -> Self {
        match self {
            n @ Self::Prim(_) => n,
            Self::Sequential(m1, m2) => match (m1.remove_zeros(), m2.remove_zeros()) {
                (Self::Prim(Primitive::Note(Dur::ZERO, _) | Primitive::Rest(Dur::ZERO)), m)
                | (m, Self::Prim(Primitive::Note(Dur::ZERO, _) | Primitive::Rest(Dur::ZERO))) => m,
                (m1, m2) => m1 + m2,
            },
            Self::Parallel(m1, m2) => match (m1.remove_zeros(), m2.remove_zeros()) {
                (Self::Prim(Primitive::Note(Dur::ZERO, _) | Primitive::Rest(Dur::ZERO)), m)
                | (m, Self::Prim(Primitive::Note(Dur::ZERO, _) | Primitive::Rest(Dur::ZERO))) => m,
                (m1, m2) => m1 | m2,
            },
            Self::Modify(c, m) => m.remove_zeros().with(c),
        }
    }
}

/// Entity that have a temporal duration.
pub trait Temporal {
    /// Get the temporal size.
    fn duration(&self) -> Dur;

    /// Take the given [`Dur`] from the beginning and drop the other.
    fn take(self, dur: Dur) -> Self;

    /// Drop the given [`Dur`] from the beginning and take the other.
    fn drop(self, dur: Dur) -> Self;
}

impl<P> Temporal for Music<P> {
    fn duration(&self) -> Dur {
        match self {
            Self::Prim(Primitive::Note(d, _) | Primitive::Rest(d)) => *d,
            Self::Sequential(m1, m2) => m1.duration() + m2.duration(),
            Self::Parallel(m1, m2) => m1.duration().max(m2.duration()),
            Self::Modify(Control::Tempo(r), m) => m.duration() / *r,
            Self::Modify(_, m) => m.duration(),
        }
    }

    /// Take the first N whole beats and drop the other
    fn take(self, n: Dur) -> Self {
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

    /// Drop the first N whole beats and take the other
    fn drop(self, n: Dur) -> Self {
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
