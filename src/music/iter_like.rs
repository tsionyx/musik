use crate::{prim::duration::Dur, utils::LazyList};

use super::{control::Control, Music, Primitive};

impl<P> Music<P> {
    /// Linear succession of musical parts.
    /// One of the most basic form of composition.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Melody>
    pub fn line(musics: Vec<Self>) -> Self {
        musics
            .into_iter()
            .rfold(Self::rest(Dur::ZERO), |acc, m| m + acc)
    }

    /// Lazy linear succession of musical parts (could be infinite).
    /// One of the most basic form of composition.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Melody>
    pub fn lazy_line<I>(musics: I) -> Self
    where
        I: Iterator<Item = Self> + Clone + 'static,
    {
        Self::Lazy(LazyList::new(musics))
    }

    /// A set of musical parts that are supposed to play simultaneously.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Chord_(music)>
    pub fn chord(musics: Vec<Self>) -> Self {
        musics
            .into_iter()
            .rfold(Self::rest(Dur::ZERO), |acc, m| acc | m)
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
            Self::Lazy(it) => Self::lazy_line(it.map(Self::remove_zeros).filter(|m| {
                !matches!(
                    m,
                    Self::Prim(Primitive::Note(Dur::ZERO, _) | Primitive::Rest(Dur::ZERO))
                )
            })),
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

    /// Skip the given [`Dur`] from the beginning and take the other.
    fn skip(self, dur: Dur) -> Self;
}

impl<P> Temporal for Music<P> {
    fn duration(&self) -> Dur {
        self.fold_by_ref(
            |prim| match prim {
                Primitive::Note(d, _) | Primitive::Rest(d) => *d,
            },
            |d1, d2| d1 + d2,
            (Dur::ZERO, |d, md| d + md),
            Dur::max,
            |ctrl, d| {
                if let Control::Tempo(r) = ctrl {
                    d / *r
                } else {
                    d
                }
            },
        )
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
            Self::Lazy(it) => {
                let it = it.scan(Dur::ZERO, move |total_dur, m| {
                    let left_to_take = n.saturating_sub(*total_dur);
                    if left_to_take == Dur::ZERO {
                        return None;
                    }
                    let m = m.take(left_to_take);
                    *total_dur = *total_dur + m.duration();
                    Some(m)
                });
                Self::lazy_line(it)
            }
            Self::Parallel(m1, m2) => m1.take(n) | m2.take(n),
            Self::Modify(Control::Tempo(r), m) => m.take(n * r).with_tempo(r),
            Self::Modify(c, m) => m.take(n).with(c),
        }
    }

    /// Drop the first N whole beats and take the other
    fn skip(self, n: Dur) -> Self {
        if n == Dur::ZERO {
            return self;
        }

        match self {
            Self::Prim(Primitive::Note(d, p)) => Self::note(d.saturating_sub(n), p),
            Self::Prim(Primitive::Rest(d)) => Self::rest(d.saturating_sub(n)),
            Self::Sequential(m1, m2) => {
                let m2 = (*m2).skip(n.saturating_sub(m1.duration()));
                (*m1).skip(n) + m2
            }
            Self::Lazy(it) => {
                let it = it.scan(Dur::ZERO, move |total_dur, m| {
                    let left_to_skip = n.saturating_sub(*total_dur);
                    if left_to_skip == Dur::ZERO {
                        return Some(m);
                    }
                    *total_dur = *total_dur + m.duration();
                    let m = m.skip(left_to_skip);
                    Some(m)
                });
                Self::lazy_line(it)
            }
            Self::Parallel(m1, m2) => (*m1).skip(n) | (*m2).skip(n),
            Self::Modify(Control::Tempo(r), m) => (*m).skip(n * r).with_tempo(r),
            Self::Modify(c, m) => (*m).skip(n).with(c),
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
            Music::Lazy(it) => it.flat_map(Self::from).collect(),
            other => vec![other],
        }
    }
}
