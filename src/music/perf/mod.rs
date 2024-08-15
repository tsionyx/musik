//! Defines abstract [`Performance`] which
//! is a time-ordered sequence of musical [`Event`]s.
use std::{iter, ops::Deref};

use itertools::Itertools as _;
use log::{debug, info};
use num_rational::Ratio;
use ordered_float::OrderedFloat;

use crate::{
    instruments::InstrumentName,
    midi::Instrument,
    music::{AttrNote, MusicAttr},
    prim::{duration::Dur, interval::Interval, pitch::AbsPitch, scale::KeySig, volume::Volume},
    utils::{CloneableIterator, LazyList, Measure},
};

use super::{control::Control, Music, Primitive};

pub use self::{
    interpretations::{DefaultPlayer, EventAnnotator, FancyPlayer},
    player::{DynPlayer, Player},
};

mod interpretations;
mod player;

#[derive(Debug, Clone)]
/// [`Performance`] is a time-ordered sequence
/// of musical [`events`][Event].
pub struct Performance {
    repr: LazyList<Event>,
}

impl Performance {
    /// Create a [`Performance`] from a number of [`Event`]s.
    pub fn with_events<I>(events: I) -> Self
    where
        I: CloneableIterator<Item = Event> + 'static,
    {
        Self {
            repr: LazyList(Box::new(events)),
        }
    }

    /// Iterate over the [`Event`]s of the [`Performance`].
    pub fn iter(&self) -> LazyList<Event> {
        self.repr.clone()
    }

    /// Checks whether the given performance is infinite
    /// by calling [`Iterator::size_hint`].
    pub fn is_probably_infinite(&self) -> bool {
        is_probably_infinite(&self.repr)
    }
}

fn is_probably_infinite<T>(it: &impl Iterator<Item = T>) -> bool {
    let (_lower, upper) = it.size_hint();
    upper.is_none()
}

impl IntoIterator for &Performance {
    type Item = Event;
    type IntoIter = LazyList<Event>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Allows some form of [`Music`]al value to be performed,
/// i.e. converted to the abstract [`Performance`].
pub trait Performable<P> {
    /// Create a [`Performance`] using the default [`Context`]
    /// and the default [`Player`]s mapping.
    fn perform(self) -> Performance;

    /// Create a [`Performance`] using the custom [`Context`].
    fn perform_with_context(self, ctx: Context<P>) -> Performance;
}

impl<P> Performable<AttrNote> for Music<P>
where
    MusicAttr: From<Self>,
{
    fn perform(self) -> Performance {
        let def_ctx = Context::with_default_player::<DefaultPlayer>();
        self.perform_with_context(def_ctx)
    }

    fn perform_with_context(self, ctx: Context<AttrNote>) -> Performance {
        let (perf, dur) = MusicAttr::from(self).perf(ctx);
        info!("Produced a performance of {:?} seconds long", dur);
        perf
    }
}

impl<P: 'static> Music<P> {
    fn perf(&self, ctx: Context<P>) -> (Performance, Measure<Duration>) {
        let ctx = Context {
            depth: ctx.depth + 1,
            ..ctx
        };
        match self {
            Self::Prim(Primitive::Note(d, p)) => {
                let dur = d.into_ratio() * ctx.whole_note;
                (ctx.player.clone().play_note((*d, p), ctx), dur.into())
            }
            Self::Prim(Primitive::Rest(d)) => (
                Performance::with_events(iter::empty()),
                (d.into_ratio() * ctx.whole_note).into(),
            ),
            Self::Sequential(m1, m2) => Self::perf_seq_pair(m1, m2, ctx),
            Self::Lazy(it) => Self::perf_seq(it, ctx),
            Self::Parallel(m1, m2) => Self::perf_par(m1, m2, ctx),
            Self::Modify(ctrl, m) => m.perf_control(ctrl, ctx),
        }
    }

    fn perf_seq_pair(m1: &Self, m2: &Self, ctx: Context<P>) -> (Performance, Measure<Duration>) {
        let (mut p1, d1) = m1.perf(ctx.clone());
        debug!("The duration of sum's LHS: {d1:?}");
        if let Measure::Finite(d) = d1 {
            let ctx = Context {
                start_time: ctx.start_time + d,
                ..ctx
            };
            let (p2, d2) = m2.perf(ctx);
            debug!("The duration of sum's RHS: {d2:?}");
            p1.repr.extend(p2.repr);
            (p1, d1 + d2)
        } else {
            info!(
                "Skipping the performing of RHS of the Music::Sequential, because LHS is infinite"
            );
            (p1, d1)
        }
    }

    #[allow(clippy::borrowed_box)]
    fn perf_seq(
        it: &Box<dyn CloneableIterator<Item = Self>>,
        ctx: Context<P>,
    ) -> (Performance, Measure<Duration>) {
        let is_infinite = is_probably_infinite(it);

        let events_with_max_dur = it.clone().enumerate()
            .scan((ctx, Measure::default()), |(ctx, total_dur), (i, m)| {
                if ctx.start_time == Measure::Infinite {
                    info!("Ignoring the performance of the rest of Music::Lazy, because the last item is infinite");
                    return None;
                }
                let (p, d) = m.perf(ctx.clone());
                debug!("The duration of Lazy item #{i}: {d:?}");
                debug!("Ctx start time #{i}: {:?}. Depth={}", ctx.start_time, ctx.depth);
                ctx.start_time = ctx.start_time + d;
                *total_dur = *total_dur + d;
                Some((p.repr, *total_dur))
            });

        if is_infinite {
            debug!("The Music::Lazy has infinite items");
            let perf = Performance::with_events(events_with_max_dur.flat_map(|(e, _)| e));
            (perf, Measure::Infinite)
        } else {
            debug!("The Music::Lazy has finite items: {:?}", it.size_hint());
            // TODO: calculate the duration more intelligently (maybe some `Measure::Lazy`)
            let d = Measure::max_in_iter(events_with_max_dur.clone().map(|(_, d)| d));
            let perf = Performance::with_events(events_with_max_dur.flat_map(|(e, _)| e));
            (perf, d.unwrap_or_default())
        }
    }

    fn perf_par(m1: &Self, m2: &Self, ctx: Context<P>) -> (Performance, Measure<Duration>) {
        let (p1, d1) = m1.perf(ctx.clone());
        debug!("The duration of parallel's LHS: {d1:?}");
        let (p2, d2) = m2.perf(ctx);
        debug!("The duration of parallel's RHS: {d2:?}");
        (
            Performance::with_events(
                p1.iter()
                    // use simple `.merge()` for perfectly commutative `Self::Parallel`
                    .merge_by(p2.iter(), |x, y| x.start_time < y.start_time),
            ),
            d1.max(d2),
        )
    }

    fn perf_control(
        &self,
        control: &Control<P>,
        ctx: Context<P>,
    ) -> (Performance, Measure<Duration>) {
        let ctx = match control {
            Control::Tempo(t) => Context {
                whole_note: ctx.whole_note / convert_ratio(*t),
                ..ctx
            },
            Control::Transpose(p) => Context {
                transpose_interval: ctx.transpose_interval + *p,
                ..ctx
            },
            Control::Instrument(i) => Context {
                instrument: i.clone(),
                ..ctx
            },
            Control::Phrase(phrases) => {
                return ctx.player.clone().interpret_phrases(self, phrases, ctx);
            }
            Control::Player(p) => {
                info!("Overwriting player during `perform`: {}", p.name());
                Context {
                    player: p.clone(),
                    ..ctx
                }
            }
            Control::KeySig(ks) => Context { key: *ks, ..ctx },
        };
        self.perf(ctx)
    }
}

fn convert_ratio<T, U>(x: Ratio<T>) -> Ratio<U>
where
    U: From<T> + Clone + num_integer::Integer,
{
    let (num, denom) = x.into();
    Ratio::new(U::from(num), U::from(denom))
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
/// The playing of one individual note.
pub struct Event {
    /// The start time of the [`Event`] in seconds since
    /// the start of the whole performance.
    pub start_time: TimePoint,

    /// The instrument to play the [`Event`]'s note.
    pub instrument: InstrumentName,

    /// The note's pitch.
    pub pitch: AbsPitch,

    /// The duration of the [`Event`]'s note in seconds.
    pub duration: Duration,

    /// The note's volume.
    pub volume: Volume,

    /// Additional parameters to customize the note's performance.
    ///
    /// Used for instruments [other than MIDI][InstrumentName::Custom].
    pub params: Vec<OrderedFloat<f64>>,
}

/// Point on the time line to identify start of the event. Measured in seconds.
pub type TimePoint = Ratio<u32>;

/// Distance on the time line to identify length of the event. Measured in seconds.
pub type Duration = Ratio<u32>;

#[derive(Debug)]
/// The state of the [`Performance`] that changes
/// as we go through the interpretation.
pub struct Context<P: 'static> {
    start_time: Measure<TimePoint>,
    player: DynPlayer<P>,
    instrument: InstrumentName,
    whole_note: Duration,
    transpose_interval: Interval,
    volume: Volume,
    key: KeySig,
    depth: usize,
}

impl<P: 'static> Clone for Context<P> {
    fn clone(&self) -> Self {
        let Self {
            start_time,
            player,
            instrument,
            whole_note,
            transpose_interval,
            volume,
            key,
            depth,
        } = self;
        Self {
            start_time: *start_time,
            player: player.clone(),
            instrument: instrument.clone(),
            whole_note: *whole_note,
            transpose_interval: *transpose_interval,
            volume: *volume,
            key: *key,
            depth: *depth,
        }
    }
}

/// Defines a tempo of X beats per minute
/// using the size of a single beat for reference
/// (common value for a beat is [quarter note][Dur::QUARTER]).
///
/// E.g. default tempo of 120 bpm defined as
/// ```
/// # use musik::{perf::metro, Dur};
/// # use num_rational::Ratio;
///
/// let tempo = metro(120, Dur::QUARTER);
///
/// // the whole note lasts exactly 2 seconds with this tempo.
/// assert_eq!(tempo, Ratio::from_integer(2));
/// ```
///
/// This function should be used as a helper for [`Context::with_tempo`].
pub fn metro(setting: u32, note_dur: Dur) -> Duration {
    Ratio::from_integer(60) / (Ratio::from_integer(setting) * note_dur.into_ratio())
}

impl<P: 'static> Context<P> {
    /// Defines the default [`Context`] with the given [`Player`].
    ///
    /// All the other fields could be changed using
    /// the family of other `with_*` methods.
    ///
    /// The [player][Player] could be changed during performance
    /// for the [`Music`] value itself by using [`Music::with_player`].
    pub fn with_player(player: DynPlayer<P>) -> Self {
        Self {
            start_time: Measure::default(),
            player,
            instrument: Instrument::AcousticGrandPiano.into(),
            whole_note: metro(120, Dur::QUARTER),
            transpose_interval: Interval::default(),
            volume: Volume::loudest(),
            key: KeySig::default(),
            depth: 0,
        }
    }

    /// Defines the default [`Context`] with the given type of [`Player`].
    pub fn with_default_player<Pl>() -> Self
    where
        Pl: Player<P> + Default + 'static,
    {
        Self::with_player(DynPlayer::from_player(Pl::default()))
    }

    /// Changes the default tempo for the performance.
    ///
    /// The provided value should define the number of seconds
    /// the [`whole note`][`Dur::WHOLE`] lasts.
    ///
    /// Use the [`metro`] helper function to define the tempo
    /// using standard metronome markings.
    pub fn with_tempo(self, whole_note: Duration) -> Self {
        Self { whole_note, ..self }
    }

    /// Changes the default volume for the performance.
    ///
    /// You could provide the explicit [`Volume`] value or use the
    /// [`StdLoudness::get_volume`][crate::attributes::StdLoudness::get_volume] here.
    pub fn with_volume(self, volume: Volume) -> Self {
        Self { volume, ..self }
    }

    /// Changes the default instrument for the performance.
    ///
    /// It is better to express the same more explicitly
    /// for the [`Music`] value itself by using [`Music::with_instrument`].
    pub fn with_instrument(self, instrument: impl Into<InstrumentName>) -> Self {
        Self {
            instrument: instrument.into(),
            ..self
        }
    }

    /// Changes the default transpose interval for the performance.
    ///
    /// It is better to express the same more explicitly
    /// for the [`Music`] value itself by using [`Music::with_transpose`].
    pub fn with_transpose(self, transpose_interval: Interval) -> Self {
        Self {
            transpose_interval,
            ..self
        }
    }

    /// Changes the default tonality for the performance.
    /// which could be useful while interpreting
    /// [phrase attributes][Self::with_phrase].
    ///
    /// It is better to express the same more explicitly
    /// for the [`Music`] value itself by using [`Music::with_key_sig`].
    pub fn with_key_sig(self, key: KeySig) -> Self {
        Self { key, ..self }
    }

    /// Current start time of the [`Context`] in seconds since
    /// the start of the whole performance.
    pub fn start_time(&self) -> TimePoint {
        match self.start_time {
            Measure::Finite(x) => x,
            Measure::Infinite => TimePoint::from_integer(u32::MAX),
        }
    }

    /// Current [`Player`] of the [`Context`].
    pub fn player(&self) -> &dyn Player<P> {
        self.player.deref().as_ref()
    }

    /// Current instrument of the [`Context`].
    pub const fn instrument(&self) -> &InstrumentName {
        &self.instrument
    }

    /// Current tempo of the context
    /// in terms of seconds per [`whole note`][Dur::WHOLE].
    pub const fn whole_note(&self) -> Duration {
        self.whole_note
    }

    /// Current transpose setting of the [`Context`].
    pub const fn transpose_interval(&self) -> Interval {
        self.transpose_interval
    }

    /// Current volume of the [`Context`].
    pub const fn volume(&self) -> Volume {
        self.volume
    }

    /// Current tonality of the [`Context`].
    pub const fn key(&self) -> KeySig {
        self.key
    }
}

#[cfg(test)]
mod tests {
    use std::iter::once;

    use super::*;

    use crate::{n, Octave, Pitch};

    #[test]
    fn john_cage() {
        // 136.5 whole notes with tempo (120 QN/min)
        // will last exactly 4'33"
        let m: Music = Music::lazy_line([Dur::from(136), Dur::HALF].into_iter().map(Music::rest));

        let mut perf = m.perform();
        assert!(perf.repr.next().is_none());
    }

    #[test]
    fn line_and_lazy_line_performed_the_same() {
        let it = once(n!(C 4 / 4))
            .chain(once(n!(G 5 / 8)))
            .chain(once(n!(B 3 / 8)));

        let m_lazy = Music::lazy_line(it.cycle().take(8).map(|n| Music::Prim(n.into())));

        let v = vec![
            n!(C 4 / 4),
            n!(G 5 / 8),
            n!(B 3 / 8),
            n!(C 4 / 4),
            n!(G 5 / 8),
            n!(B 3 / 8),
            n!(C 4 / 4),
            n!(G 5 / 8),
        ];
        let m_eager = Music::line(v.into_iter().map(|n| Music::Prim(n.into())).collect());

        let perf_lazy: Vec<Event> = m_lazy.perform().iter().collect();
        let perf_eager: Vec<Event> = m_eager.perform().iter().collect();
        assert_eq!(perf_lazy, perf_eager);
    }

    #[test]
    fn concat_lazy() {
        let _ = env_logger::try_init();

        let m_lazy = {
            let a_it = once(n!(C 4 / 8));
            let b_it = once(n!(C 5 / 8));

            let m_a = Music::lazy_line(a_it.map(|n| Music::Prim(n.into())));
            let m_b = Music::lazy_line(b_it.map(|n| Music::Prim(n.into())));
            m_a + m_b
        };

        let m_eager = Music::Prim(Primitive::Note(Dur::EIGHTH, Pitch::C(Octave::OneLined)))
            + Music::Prim(Primitive::Note(Dur::EIGHTH, Pitch::C(Octave::TwoLined)));

        let perf_lazy: Vec<Event> = m_lazy.perform().iter().collect();
        dbg!(&perf_lazy);
        let perf_eager: Vec<Event> = m_eager.perform().iter().collect();
        dbg!(&perf_eager);
        assert_eq!(perf_lazy, perf_eager);
    }

    #[test]
    fn lazy_of_lazy() {
        let _ = env_logger::try_init();

        let m_lazy = {
            let a_it = once(n!(C 4 / 8));
            let b_it = once(n!(C 5 / 8));

            let m = Music::lazy_line(
                a_it.chain(b_it)
                    .map(|n| Music::Prim(n.into()))
                    .chain(once(Music::rest(Dur::QUARTER))),
            );
            Music::lazy_line(iter::repeat(m).take(3))
        };

        let m_eager = {
            let m = Music::Prim(Primitive::Note(Dur::EIGHTH, Pitch::C(Octave::OneLined)))
                + Music::Prim(Primitive::Note(Dur::EIGHTH, Pitch::C(Octave::TwoLined)))
                + Music::Prim(Primitive::Rest(Dur::QUARTER));
            Music::line(vec![m.clone(), m.clone(), m])
        };

        let perf_lazy: Vec<Event> = m_lazy.perform().iter().collect();
        dbg!(&perf_lazy);
        let perf_eager: Vec<Event> = m_eager.perform().iter().collect();
        dbg!(&perf_eager);
        assert_eq!(perf_lazy, perf_eager);
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn complex_music_with_lazy_line_performed_the_same_as_eager() {
        let _ = env_logger::try_init();

        let m_lazy = {
            let a_it = once(n!(F 3 / 8))
                .chain(once(n!(Gs 3 / 8)))
                .chain(once(n!(C 4 / 8)))
                .chain(once(n!(C 3 / 8)));

            let b_it = once(n!(C 4 / 4))
                .chain(once(n!(G 5 / 8)))
                .chain(once(n!(B 3 / 8)));

            let m_a = Music::lazy_line(a_it.map(|n| Music::Prim(n.into())));
            let m_b = Music::lazy_line(b_it.cycle().take(8).map(|n| Music::Prim(n.into())));

            (m_a.clone().with_instrument(Instrument::Contrabass) + m_b.clone())
                | (m_a + m_b.with_instrument(Instrument::ElectricGuitarClean))
        };

        let m_eager = {
            let a_vec = vec![n!(F 3 / 8), n!(Gs 3 / 8), n!(C 4 / 8), n!(C 3 / 8)];
            let b_vec = vec![
                n!(C 4 / 4),
                n!(G 5 / 8),
                n!(B 3 / 8),
                n!(C 4 / 4),
                n!(G 5 / 8),
                n!(B 3 / 8),
                n!(C 4 / 4),
                n!(G 5 / 8),
            ];

            let m_a = Music::line(a_vec.into_iter().map(|n| Music::Prim(n.into())).collect());
            let m_b = Music::line(b_vec.into_iter().map(|n| Music::Prim(n.into())).collect());

            (m_a.clone().with_instrument(Instrument::Contrabass) + m_b.clone())
                | (m_a + m_b.with_instrument(Instrument::ElectricGuitarClean))
        };

        // m_lazy.clone().perform().save_to_file("lazy.mid").unwrap();
        // m_eager.clone().perform().save_to_file("eager.mid").unwrap();

        let perf_lazy: Vec<Event> = m_lazy.perform().iter().collect();
        dbg!(&perf_lazy);
        let perf_eager: Vec<Event> = m_eager.perform().iter().collect();
        dbg!(&perf_eager);
        assert_eq!(perf_lazy, perf_eager);
    }
}
