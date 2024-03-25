//! Defines abstract [`Performance`] which
//! is a time-ordered sequence of musical [`Event`]s.
use std::{borrow::Cow, collections::HashMap, fmt, iter, sync::Arc};

use itertools::Itertools as _;
use num_rational::Ratio;
use ordered_float::OrderedFloat;
use ux2::u4;

use crate::{
    instruments::InstrumentName,
    music::{AttrNote, MusicAttr},
    prim::{duration::Dur, interval::Interval, pitch::AbsPitch, scale::KeySig, volume::Volume},
};

use super::{
    control::{Control, PlayerName},
    phrase::PhraseAttribute,
    Music, Primitive,
};

#[derive(Debug, Clone)]
/// [`Performance`] is a time-ordered sequence
/// of musical [`events`][Event].
pub struct Performance {
    repr: Vec<Event>,
}

impl Performance {
    /// Create a [`Performance`] from a number of [`Event`]s.
    pub fn with_events(events: Vec<Event>) -> Self {
        Self { repr: events }
    }

    #[allow(clippy::missing_const_for_fn)] // for 1.63
    /// Convert the [`Performance`] into a number of [`Event`]s.
    pub fn into_events(self) -> Vec<Event> {
        self.repr
    }

    /// Iterate over the [`Event`]s of the [`Performance`].
    pub(crate) fn iter(&self) -> impl Iterator<Item = &Event> {
        self.repr.iter()
    }
}

/// Allows some form of [`Music`]al value to be performed,
/// i.e. converted to the abstract [`Performance`].
pub trait Performable<P> {
    /// Create a [`Performance`] using the default [`Context`]
    /// and the default [`Player`]s mapping.
    fn perform(self) -> Performance
    where
        Player<P>: Default,
        Self: Sized,
    {
        self.perform_with_context(Context::with_player(Cow::Owned(Player::default())))
    }

    /// Create a [`Performance`] using the custom [`Context`]
    /// and the default [`Player`]s mapping.
    fn perform_with_context(self, ctx: Context<'_, P>) -> Performance;

    /// Create a [`Performance`] using the custom [`Context`]
    /// and [`Player`]s mapping.
    fn perform_with<'p>(self, players: &'p PlayerMap<P>, ctx: Context<'p, P>) -> Performance;
}

impl<P> Performable<P> for &Music<P>
where
    Player<P>: Default,
{
    fn perform_with_context(self, ctx: Context<'_, P>) -> Performance {
        let def_name = Player::default().name;

        let players: PlayerMap<_> = iter::once(Player::default())
            .map(|p| (p.name.clone(), p))
            .collect();

        let def_player = players.get(&def_name).expect("Just inserted");
        let player = Cow::Borrowed(def_player);
        self.perform_with(&players, Context { player, ..ctx })
    }

    fn perform_with<'p>(self, players: &'p PlayerMap<P>, ctx: Context<'p, P>) -> Performance {
        self.perf(players, ctx).0
    }
}

impl<P> Performable<AttrNote> for Music<P>
where
    MusicAttr: From<Self>,
{
    fn perform_with_context(self, ctx: Context<'_, AttrNote>) -> Performance {
        let def_name = Player::<AttrNote>::fancy().name;

        let players: PlayerMap<_> = [Player::default(), Player::fancy()]
            .into_iter()
            .map(|p| (p.name.clone(), p))
            .collect();

        let def_player = players.get(&def_name).expect("Just inserted");
        let player = Cow::Borrowed(def_player);
        self.perform_with(&players, Context { player, ..ctx })
    }

    fn perform_with<'p>(
        self,
        players: &'p PlayerMap<AttrNote>,
        ctx: Context<'p, AttrNote>,
    ) -> Performance {
        MusicAttr::from(self).perf(players, ctx).0
    }
}

impl<P> Music<P>
where
    Player<P>: Default,
{
    fn perf<'p>(
        &self,
        players: &'p PlayerMap<P>,
        mut ctx: Context<'p, P>,
    ) -> (Performance, Duration) {
        match self {
            Self::Prim(Primitive::Note(d, p)) => {
                let dur = d.into_ratio() * ctx.whole_note;
                ((ctx.player.play_note.clone())(ctx, *d, p), dur)
            }
            Self::Prim(Primitive::Rest(d)) => (
                Performance::with_events(vec![]),
                d.into_ratio() * ctx.whole_note,
            ),
            Self::Sequential(m1, m2) => {
                let (mut p1, d1) = m1.perf(players, ctx.clone());
                ctx.start_time += d1;
                let (p2, d2) = m2.perf(players, ctx);
                p1.repr.extend(p2.repr);
                (p1, d1 + d2)
            }
            Self::Parallel(m1, m2) => {
                let (p1, d1) = m1.perf(players, ctx.clone());
                let (p2, d2) = m2.perf(players, ctx);
                (
                    Performance::with_events(
                        p1.repr
                            .into_iter()
                            // use simple `.merge()` for perfectly commutative `Self::Parallel`
                            .merge_by(p2.repr, |x, y| x.start_time < y.start_time)
                            .collect(),
                    ),
                    d1.max(d2),
                )
            }
            Self::Modify(Control::Tempo(t), m) => {
                ctx.whole_note /= convert_ratio(*t);
                m.perf(players, ctx)
            }
            Self::Modify(Control::Transpose(p), m) => {
                ctx.transpose_interval += *p;
                m.perf(players, ctx)
            }
            Self::Modify(Control::Instrument(i), m) => {
                ctx.instrument = i.clone();
                m.perf(players, ctx)
            }
            Self::Modify(Control::Phrase(phrases), m) => {
                (ctx.player.interpret_phrase.clone())(m, players, ctx, phrases)
            }
            Self::Modify(Control::Player(p), m) => {
                let player = players
                    .get(p)
                    .map_or_else(|| Cow::Owned(Player::default()), Cow::Borrowed);
                ctx.player = player;
                m.perf(players, ctx)
            }
            Self::Modify(Control::KeySig(ks), m) => {
                ctx.key = *ks;
                m.perf(players, ctx)
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
pub struct Context<'p, P> {
    start_time: TimePoint,
    player: Cow<'p, Player<P>>,
    instrument: InstrumentName,
    whole_note: Duration,
    transpose_interval: Interval,
    volume: Volume,
    key: KeySig,
}

impl<P> Clone for Context<'_, P> {
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

#[derive(Debug, Clone, Eq, PartialEq)]
/// Attributes that can be attached to each individual note.
pub enum NoteAttribute {
    /// How loud to play the note.
    Volume(Volume),

    /// Which finger to use while playing.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Fingering_(music)>.
    Fingering(u4),

    /// Individual note dynamics.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Accent_(music)#Marks>
    ///
    /// TODO: fill more from <https://www.musictheoryacademy.com/how-to-read-sheet-music/dynamics/>
    Dynamics(String),

    /// Additional parameters to customize the note's performance.
    ///
    /// Used for instruments [other than MIDI][InstrumentName::Custom].
    /// It is up to the instrument designer to decide how these parameters are used.
    Params(Vec<OrderedFloat<f64>>),
}

type PlayerMap<P> = HashMap<PlayerName, Player<P>>;

pub struct Player<P> {
    pub name: String,
    pub play_note: NoteFun<P>,
    pub interpret_phrase: PhraseFun<P>,
    pub notate_player: NotateFun<P>,
}

impl<P> Clone for Player<P> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            play_note: self.play_note.clone(),
            interpret_phrase: self.interpret_phrase.clone(),
            notate_player: self.notate_player,
        }
    }
}

impl<P> fmt::Debug for Player<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Player {}", self.name)
    }
}

type NoteFun<P> = Arc<dyn Fn(Context<'_, P>, Dur, &P) -> Performance>;
type PhraseFun<P> = Arc<
    dyn Fn(&Music<P>, &PlayerMap<P>, Context<'_, P>, &[PhraseAttribute]) -> (Performance, Duration),
>;
// TODO: producing a properly notated score is not defined yet
type NotateFun<P> = std::marker::PhantomData<P>;

pub mod defaults {
    use std::iter;

    use num_traits::{ops::checked::CheckedSub as _, One as _, Zero as _};

    use crate::{output::midi::instruments::Instrument, prim::pitch::Pitch};

    use super::{
        super::{
            phrase::{Articulation, Dynamic, Ornament, PhraseAttribute, Tempo, TrillOptions},
            AttrNote,
        },
        *,
    };

    pub fn default_play_note<Attr>(
        attr_modifier: NoteWithAttributeHandler<Pitch, Attr>,
    ) -> NoteFun<(Pitch, Vec<Attr>)>
    where
        Attr: 'static,
    {
        Arc::new(move |ctx, dur, (note_pitch, attrs)| {
            let Context {
                start_time,
                player: _ignore_player,
                instrument,
                whole_note,
                transpose_interval,
                volume,
                key: _ignore_key,
            } = ctx.clone();
            let init = Event {
                start_time,
                instrument,
                pitch: note_pitch.abs() + transpose_interval,
                duration: dur.into_ratio() * whole_note,
                volume,
                params: vec![],
            };

            let event = attrs
                .iter()
                .fold(init, |acc, attr| attr_modifier(&ctx, attr, acc));
            Performance::with_events(vec![event])
        })
    }

    pub fn default_note_attribute_handler<P>() -> NoteWithAttributeHandler<P, NoteAttribute> {
        Box::new(|_ignore_context, attr, event| match attr {
            NoteAttribute::Volume(vol) => Event {
                volume: *vol,
                ..event
            },
            NoteAttribute::Params(params) => Event {
                params: params.clone(),
                ..event
            },
            NoteAttribute::Fingering(_) | NoteAttribute::Dynamics(_) => event,
        })
    }

    /// Transform the event according to [`Context`] and Attribute.
    type NoteWithAttributeHandler<P, Attr> =
        Box<dyn Fn(&Context<'_, (P, Vec<Attr>)>, &Attr, Event) -> Event>;

    // Transform the whole performance according to [`Context`] and [`PhraseAttribute`].
    // type PhraseAttributeHandler = Box<dyn Fn(Performance, &PhraseAttribute) -> Performance>;

    pub fn default_interpret_phrase<P, PhraseF>(attr_modifier: PhraseF) -> PhraseFun<P>
    where
        Player<P>: Default,
        PhraseF: Fn(Performance, &PhraseAttribute) -> Performance + 'static,
    {
        Arc::new(move |music, players, ctx, attrs| {
            let (perf, dur) = music.perf(players, ctx);
            let perf = attrs.iter().fold(perf, &attr_modifier);
            (perf, dur)
        })
    }

    pub fn default_phrase_attribute_handler(
        perf: Performance,
        attr: &PhraseAttribute,
    ) -> Performance {
        match attr {
            PhraseAttribute::Dyn(Dynamic::Accent(x)) => perf.map(|event| Event {
                volume: Volume::from(
                    (x * Ratio::from_integer(u8::from(event.volume.0))).to_integer(),
                ),
                ..event
            }),
            PhraseAttribute::Art(Articulation::Staccato(x)) => perf.map(|event| Event {
                duration: x * event.duration,
                ..event
            }),
            PhraseAttribute::Art(Articulation::Legato(x)) => perf.map(|event| Event {
                duration: x * event.duration,
                ..event
            }),

            PhraseAttribute::Dyn(_)
            | PhraseAttribute::Tmp(_)
            | PhraseAttribute::Art(_)
            | PhraseAttribute::Orn(_) => perf,
        }
    }

    impl Performance {
        fn map<F>(self, f: F) -> Self
        where
            F: FnMut(Event) -> Event,
        {
            Self::with_events(self.repr.into_iter().map(f).collect())
        }
    }

    impl Default for Player<AttrNote> {
        fn default() -> Self {
            Self {
                name: "Default".to_string(),
                play_note: default_play_note(default_note_attribute_handler()),
                interpret_phrase: default_interpret_phrase(default_phrase_attribute_handler),
                notate_player: Default::default(),
            }
        }
    }

    pub fn fancy_interpret_phrase<P>(
        music: &Music<P>,
        players: &PlayerMap<P>,
        mut ctx: Context<'_, P>,
        attrs: &[PhraseAttribute],
    ) -> (Performance, Duration)
    where
        Player<P>: Default,
    {
        let key = ctx.key;

        let last_volume_phrase = attrs.iter().fold(None, |found, pa| match pa {
            // ignore the previous volume if found new one
            PhraseAttribute::Dyn(Dynamic::StdLoudness(std_loud)) => Some(std_loud.get_volume()),
            PhraseAttribute::Dyn(Dynamic::Loudness(vol)) => Some(*vol),
            _ => found,
        });

        if let Some(volume) = last_volume_phrase {
            ctx.volume = volume;
        }

        let (perf, dur) =
            default_interpret_phrase(fancy_phrase_attribute_handler)(music, players, ctx, attrs);

        let t0 = match perf.repr.first().map(|e| e.start_time) {
            Some(t) => t,
            None => {
                return (perf, dur);
            }
        };

        let inflate = |event: Event, coef: Ratio<u32>, sign: bool| {
            let r = coef / dur;
            let dt = event.start_time - t0;
            let coef_event = dt * r;
            let shift = if sign {
                Ratio::one() + coef_event
            } else {
                // for `sign=false`, the `coef` should belong
                // to the range `[0 (no changes)..1 (fade out to zero)]`
                Ratio::one().checked_sub(&coef_event).unwrap_or_default()
            };

            let new_volume = Ratio::from(u32::from(event.volume.0)) * shift;
            Event {
                volume: Volume::from(u8::try_from(new_volume.to_integer()).unwrap_or(u8::MAX)),
                ..event
            }
        };

        let stretch = |event: Event, coef: Ratio<u32>, sign: bool| {
            let r = coef / dur;
            let dt = event.start_time - t0;
            let time_coef_event = dt * r;
            let dur_coef_event = (Ratio::from(2) * dt + event.duration) * r;

            let (time_shift, dur_shift) = if sign {
                (
                    Ratio::one() + time_coef_event,
                    Ratio::one() + dur_coef_event,
                )
            } else {
                (
                    // for `sign=false`, the `coef` should belong
                    // to the range `[0 (no changes)..1 (shrink to point)]`
                    Ratio::one()
                        .checked_sub(&time_coef_event)
                        .unwrap_or_default(),
                    // for `sign=false`, the `coef` should belong
                    // to the range `[0 (no changes)..0.5 (shrink to point)]`
                    Ratio::one()
                        .checked_sub(&dur_coef_event)
                        .unwrap_or_default(),
                )
            };

            Event {
                start_time: time_shift * dt + t0,
                duration: dur_shift * event.duration,
                ..event
            }
        };

        attrs
            .iter()
            .fold((perf, dur), |(perf, dur), attr| match attr {
                PhraseAttribute::Dyn(Dynamic::Crescendo(x)) => {
                    let perf = perf.map(|e| inflate(e, *x, true));
                    (perf, dur)
                }
                PhraseAttribute::Dyn(Dynamic::Diminuendo(x)) => {
                    let perf = perf.map(|e| inflate(e, *x, false));
                    (perf, dur)
                }
                PhraseAttribute::Tmp(Tempo::Ritardando(x)) => {
                    let perf = perf.map(|e| stretch(e, *x, true));
                    let dur = (Ratio::one() + *x) * dur;
                    (perf, dur)
                }
                PhraseAttribute::Tmp(Tempo::Accelerando(x)) => {
                    let perf = perf.map(|e| stretch(e, *x, false));
                    let dur = Ratio::one().checked_sub(x).unwrap_or_default() * dur;
                    (perf, dur)
                }
                PhraseAttribute::Orn(Ornament::Trill(opts)) => {
                    // exercise 8.2.1
                    let events = perf
                        .into_events()
                        .into_iter()
                        .flat_map(|e| trill(e, *opts, key))
                        .collect();
                    (Performance::with_events(events), dur)
                }
                PhraseAttribute::Orn(Ornament::Mordent) => {
                    // exercise 8.2.2
                    let events = perf
                        .into_events()
                        .into_iter()
                        .flat_map(|e| mordent(e, true, false, key))
                        .collect();
                    (Performance::with_events(events), dur)
                }
                PhraseAttribute::Orn(Ornament::InvMordent) => {
                    // exercise 8.2.3
                    let events = perf
                        .into_events()
                        .into_iter()
                        .flat_map(|e| mordent(e, false, false, key))
                        .collect();
                    (Performance::with_events(events), dur)
                }
                PhraseAttribute::Orn(Ornament::DoubleMordent) => {
                    // exercise 8.2.4
                    let events = perf
                        .into_events()
                        .into_iter()
                        .flat_map(|e| mordent(e, true, true, key))
                        .collect();
                    (Performance::with_events(events), dur)
                }
                PhraseAttribute::Orn(Ornament::DiatonicTrans(i)) => {
                    // exercise 8.5
                    let perf = perf.map(|e| Event {
                        pitch: e.pitch.diatonic_trans(key, *i),
                        ..e
                    });
                    (perf, dur)
                }
                _ => (perf, dur),
            })
    }

    fn trill(
        event: Event,
        opts: TrillOptions<Ratio<u32>>,
        key: KeySig,
    ) -> impl Iterator<Item = Event> {
        let main_pitch = event.pitch;
        let mut trill_pitch = main_pitch.diatonic_trans(key, 1);
        if trill_pitch == main_pitch {
            // pitch is out of defined key
            trill_pitch = main_pitch.diatonic_trans(key, 2);
        }
        assert!(trill_pitch > main_pitch);

        let d = event.duration;
        let dur_seq: Box<dyn Iterator<Item = Duration>> = match opts {
            TrillOptions::Duration(single) => {
                let n = (d / single).to_integer();
                let last_dur = d
                    .checked_sub(&(Ratio::from(n) * single))
                    .expect("Parts total duration should not be bigger than the whole");

                Box::new(
                    iter::repeat(single)
                        .take(n as usize)
                        .chain((!last_dur.is_zero()).then_some(last_dur)),
                )
            }
            TrillOptions::Count(n) => {
                let single = d / Ratio::from(u32::from(n));
                Box::new(iter::repeat(single).take(usize::from(n)))
            }
        };

        alternate_pitch(event, trill_pitch, dur_seq)
    }

    fn alternate_pitch(
        event: Event,
        auxiliary: AbsPitch,
        durations: impl Iterator<Item = Duration>,
    ) -> impl Iterator<Item = Event> {
        let principal = event.pitch;
        durations
            .enumerate()
            .scan(TimePoint::zero(), move |start, (i, duration)| {
                // odd are alternate
                let pitch = if i % 2 == 1 { auxiliary } else { principal };
                let prev_start = *start;
                *start += duration;
                Some(Event {
                    start_time: prev_start,
                    pitch,
                    duration,
                    ..event.clone()
                })
            })
    }

    fn mordent(
        event: Event,
        upper: bool,
        double: bool,
        key: KeySig,
    ) -> impl Iterator<Item = Event> {
        let main_pitch = event.pitch;
        let aux_pitch = if upper {
            let mut pitch = main_pitch.diatonic_trans(key, 1);
            if pitch == main_pitch {
                // pitch is out of defined key
                pitch = main_pitch.diatonic_trans(key, 2);
            }
            assert!(pitch > main_pitch);
            pitch
        } else {
            let mut pitch = main_pitch.diatonic_trans(key, -1);
            if pitch == main_pitch {
                // pitch is out of defined key
                pitch = main_pitch.diatonic_trans(key, -2);
            }
            assert!(pitch < main_pitch);
            pitch
        };

        let d = event.duration;
        let mordent = d / 8;
        let dur_seq: Box<dyn Iterator<Item = Duration>> = if double {
            Box::new(
                iter::repeat(mordent)
                    .take(4)
                    .chain(Some(d * Ratio::new(1, 2))),
            )
        } else {
            Box::new(
                iter::repeat(mordent)
                    .take(2)
                    .chain(Some(d * Ratio::new(3, 4))),
            )
        };
        alternate_pitch(event, aux_pitch, dur_seq)
    }

    fn arpeggio(events: Vec<Event>, up: bool) -> Vec<Event> {
        let chord_groups = events.into_iter().group_by(|e| (e.start_time, e.duration));
        chord_groups
            .into_iter()
            .flat_map(|(_, chord)| arpeggio_chord(chord.collect(), up))
            .collect()
    }

    fn arpeggio_chord(mut events: Vec<Event>, up: bool) -> Box<dyn Iterator<Item = Event>> {
        let (s, d) = if let Some(first) = events.first() {
            (first.start_time, first.duration)
        } else {
            return Box::new(iter::empty());
        };

        assert!(events
            .iter()
            .all(|e| (e.start_time == s) && (e.duration == d)));

        if up {
            events.sort_by_key(|e| e.pitch);
        } else {
            events.sort_by_key(|e| std::cmp::Reverse(e.pitch));
        }

        let size = events.len() as u32;
        match size {
            3 | 5 | 6 | 7 if d.numer() % size == 0 => {
                if d.numer() % size == 0 {
                    // could split into equal intervals
                    let short_dur = d / size;
                    Box::new(events.into_iter().enumerate().map(move |(i, e)| Event {
                        start_time: s + short_dur * (i as u32),
                        duration: short_dur,
                        ..e
                    }))
                } else {
                    // split into 1/4 or 1/8 intervals, with the last note longer
                    let short_dur = if size <= 4 {
                        d / 4
                    } else {
                        assert!(size <= 8);
                        d / 8
                    };

                    let equal_dur_notes = size - 1;
                    Box::new(events.into_iter().enumerate().map(move |(i, e)| {
                        // the last is longer
                        let duration = if i as u32 == equal_dur_notes {
                            d - (short_dur * equal_dur_notes)
                        } else {
                            short_dur
                        };

                        Event {
                            start_time: s + short_dur * (i as u32),
                            duration,
                            ..e
                        }
                    }))
                }
            }
            4 | 8 => {
                let short_dur = d / size;
                Box::new(events.into_iter().enumerate().map(move |(i, e)| Event {
                    start_time: s + short_dur * (i as u32),
                    duration: short_dur,
                    ..e
                }))
            }
            _ => Box::new(events.into_iter()),
        }
    }

    fn fancy_phrase_attribute_handler(perf: Performance, attr: &PhraseAttribute) -> Performance {
        match attr {
            PhraseAttribute::Dyn(Dynamic::Accent(x)) => perf.map(|event| Event {
                volume: Volume::from(
                    (x * Ratio::from_integer(u8::from(event.volume.0))).to_integer(),
                ),
                ..event
            }),
            PhraseAttribute::Dyn(_) | PhraseAttribute::Tmp(_) => {
                // already handled in the fancy_interpret_phrase
                perf
            }
            PhraseAttribute::Art(Articulation::Staccato(x)) => perf.map(|event| Event {
                duration: x * event.duration,
                ..event
            }),
            PhraseAttribute::Art(Articulation::Legato(x)) => perf.map(|event| Event {
                duration: x * event.duration,
                ..event
            }),
            PhraseAttribute::Art(Articulation::Slurred(x)) => {
                // the same as Legato, but do not extend the duration of the last note(s)
                let last_start_time = perf.repr.iter().map(|e| e.start_time).max();
                if let Some(last_start_time) = last_start_time {
                    perf.map(|event| {
                        if event.start_time < last_start_time {
                            Event {
                                duration: x * event.duration,
                                ..event
                            }
                        } else {
                            event
                        }
                    })
                } else {
                    perf
                }
            }
            PhraseAttribute::Art(Articulation::Pedal) => {
                // exercise 8.2.1
                // all the notes will sustain until the end of the phrase
                let end_of_the_phrase = perf.repr.iter().map(|e| e.start_time + e.duration).max();
                if let Some(last_event_end) = end_of_the_phrase {
                    perf.map(|event| {
                        if let Some(lengthened_duration) =
                            last_event_end.checked_sub(&event.start_time)
                        {
                            assert!(lengthened_duration >= event.duration);
                            Event {
                                duration: lengthened_duration,
                                ..event
                            }
                        } else {
                            event
                        }
                    })
                } else {
                    perf
                }
            }
            PhraseAttribute::Orn(Ornament::ArpeggioUp) => {
                Performance::with_events(arpeggio(perf.into_events(), true))
            }
            PhraseAttribute::Orn(Ornament::ArpeggioDown) => {
                Performance::with_events(arpeggio(perf.into_events(), false))
            }
            PhraseAttribute::Art(_) | PhraseAttribute::Orn(_) => perf,
        }
    }

    impl<P> Player<P>
    where
        P: 'static,
        Self: Default,
    {
        /// Slightly customized [default][Self::default] player with changed
        /// interpretations of some of the [phrases][PhraseAttribute].
        pub fn fancy() -> Self {
            Self {
                name: "Fancy".to_string(),
                interpret_phrase: Arc::new(fancy_interpret_phrase),
                ..Self::default()
            }
        }
    }

    impl<'p, P> Context<'p, P> {
        /// Defines the default [`Context`] with the given [`Player`].
        ///
        /// All the other fields could be changed using
        /// the family of other `with_*` methods.
        ///
        /// The [player][Player] could be changed during performance
        /// for the [`Music`] value itself by using [`Music::with_player`].
        pub fn with_player(player: Cow<'p, Player<P>>) -> Self {
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
        pub fn player(&self) -> &Player<P> {
            self.player.as_ref()
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

    impl<P> Default for Context<'_, P>
    where
        P: 'static,
        Player<P>: Default,
    {
        /// Defines the default [`Context`] with
        /// the [`fancy`][Player::fancy] player.
        fn default() -> Self {
            Self::with_player(Cow::Owned(Player::fancy()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn john_cage() {
        // 136.5 whole notes with tempo (120 QN/min)
        // will last exactly 4'33"
        let m: Music = Music::line(
            [Dur::from(136), Dur::HALF]
                .into_iter()
                .map(Music::rest)
                .collect(),
        );

        let perf = m.perform();
        assert!(perf.repr.is_empty());
    }
}
