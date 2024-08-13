//! Defines abstract [`Performance`] which
//! is a time-ordered sequence of musical [`Event`]s.
use std::{borrow::Cow, iter, ops::Deref};

use itertools::Itertools as _;
use log::info;
use num_rational::Ratio;
use ordered_float::OrderedFloat;

use crate::{
    instruments::InstrumentName,
    midi::Instrument,
    music::{AttrNote, MusicAttr},
    prim::{duration::Dur, interval::Interval, pitch::AbsPitch, scale::KeySig, volume::Volume},
    utils::{CloneableIterator, LazyList},
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
    fn perform_with_context(self, ctx: Context<'_, P>) -> Performance;
}

impl<P> Performable<AttrNote> for Music<P>
where
    MusicAttr: From<Self>,
{
    fn perform(self) -> Performance {
        let def_ctx = Context::with_default_player::<DefaultPlayer>();
        self.perform_with_context(def_ctx)
    }

    fn perform_with_context(self, ctx: Context<'_, AttrNote>) -> Performance {
        MusicAttr::from(self).perf(ctx).0
    }
}

impl<P: 'static> Music<P> {
    fn perf<'s, 'ctx>(&'s self, mut ctx: Context<'ctx, P>) -> (Performance, Duration)
    where
        's: 'ctx,
    {
        match self {
            Self::Prim(Primitive::Note(d, p)) => {
                let dur = d.into_ratio() * ctx.whole_note;
                (ctx.player.clone().play_note((*d, p), ctx), dur)
            }
            Self::Prim(Primitive::Rest(d)) => (
                Performance::with_events(iter::empty()),
                d.into_ratio() * ctx.whole_note,
            ),
            Self::Sequential(m1, m2) => {
                let (mut p1, d1) = m1.perf(ctx.clone());
                ctx.start_time += d1;
                let (p2, d2) = m2.perf(ctx);
                p1.repr.extend(p2.repr);
                (p1, d1 + d2)
            }
            Self::Lazy(it) => {
                let mut total_perf = Performance::with_events(iter::empty());

                for m in it.clone() {
                    let (p, d) = m.perf(ctx.clone());
                    ctx.start_time += d;
                    total_perf.repr.extend(p.repr);
                }

                (total_perf, ctx.start_time)
            }
            Self::Parallel(m1, m2) => {
                let (p1, d1) = m1.perf(ctx.clone());
                let (p2, d2) = m2.perf(ctx);
                (
                    Performance::with_events(
                        p1.iter()
                            // use simple `.merge()` for perfectly commutative `Self::Parallel`
                            .merge_by(p2.iter(), |x, y| x.start_time < y.start_time),
                    ),
                    d1.max(d2),
                )
            }
            Self::Modify(Control::Tempo(t), m) => {
                ctx.whole_note /= convert_ratio(*t);
                m.perf(ctx)
            }
            Self::Modify(Control::Transpose(p), m) => {
                ctx.transpose_interval += *p;
                m.perf(ctx)
            }
            Self::Modify(Control::Instrument(i), m) => {
                ctx.instrument = i.clone();
                m.perf(ctx)
            }
            Self::Modify(Control::Phrase(phrases), m) => {
                ctx.player.clone().interpret_phrases(m, phrases, ctx)
            }
            Self::Modify(Control::Player(p), m) => {
                info!("Overwriting player during `perform`: {}", p.name());
                ctx.player = Cow::Borrowed(p);
                m.perf(ctx)
            }
            Self::Modify(Control::KeySig(ks), m) => {
                ctx.key = *ks;
                m.perf(ctx)
            }
        }
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
pub struct Context<'p, P: 'static> {
    start_time: TimePoint,
    player: Cow<'p, DynPlayer<P>>,
    instrument: InstrumentName,
    whole_note: Duration,
    transpose_interval: Interval,
    volume: Volume,
    key: KeySig,
}

impl<P: 'static> Clone for Context<'_, P> {
    fn clone(&self) -> Self {
        let Self {
            start_time,
            player,
            instrument,
            whole_note,
            transpose_interval,
            volume,
            key,
        } = self;
        Self {
            start_time: *start_time,
            player: player.clone(),
            instrument: instrument.clone(),
            whole_note: *whole_note,
            transpose_interval: *transpose_interval,
            volume: *volume,
            key: *key,
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

impl<'p, P: 'static> Context<'p, P> {
    /// Defines the default [`Context`] with the given [`Player`].
    ///
    /// All the other fields could be changed using
    /// the family of other `with_*` methods.
    ///
    /// The [player][Player] could be changed during performance
    /// for the [`Music`] value itself by using [`Music::with_player`].
    pub fn with_player(player: Cow<'p, DynPlayer<P>>) -> Self {
        Self {
            start_time: TimePoint::from_integer(0),
            player,
            instrument: Instrument::AcousticGrandPiano.into(),
            whole_note: metro(120, Dur::QUARTER),
            transpose_interval: Interval::default(),
            volume: Volume::loudest(),
            key: KeySig::default(),
        }
    }

    /// Defines the default [`Context`] with the given type of [`Player`].
    pub fn with_default_player<Pl>() -> Self
    where
        Pl: Player<P> + Default + 'static,
    {
        Self::with_player(Cow::Owned(DynPlayer::from_player(Pl::default())))
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
    pub const fn start_time(&self) -> TimePoint {
        self.start_time
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
    use super::*;

    #[test]
    fn john_cage() {
        // 136.5 whole notes with tempo (120 QN/min)
        // will last exactly 4'33"
        let m: Music = Music::lazy_line([Dur::from(136), Dur::HALF].into_iter().map(Music::rest));

        let mut perf = m.perform();
        assert!(perf.repr.next().is_none());
    }
}
