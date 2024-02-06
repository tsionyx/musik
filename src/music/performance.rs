use std::{collections::HashMap, fmt};

use itertools::Itertools as _;
use num_rational::Ratio;
use ordered_float::OrderedFloat;

use crate::instruments::InstrumentName;

use super::{
    duration::Dur, interval::AbsPitch, phrases::PhraseAttribute, pitch::PitchClass, Control, Mode,
    Music, PlayerName, Primitive, Volume,
};

#[derive(Debug)]
/// [`Performance`] is a time-ordered sequence
/// of musical [`events`][Event].
pub struct Performance {
    repr: Vec<Event>,
}

impl Performance {
    pub fn with_events(evs: Vec<Event>) -> Self {
        Self { repr: evs }
    }

    #[allow(clippy::missing_const_for_fn)] // for 1.63
    pub fn into_events(self) -> Vec<Event> {
        self.repr
    }
}

impl<P> Music<P> {
    pub fn perform<'p>(&self, players: &'p PlayerMap<P>, ctx: Context<'p, P>) -> Performance {
        self.perf(players, ctx).0
    }

    fn perf<'p>(
        &self,
        players: &'p PlayerMap<P>,
        mut ctx: Context<'p, P>,
    ) -> (Performance, Duration) {
        match self {
            Self::Prim(Primitive::Note(d, p)) => {
                let dur = d.into_ratio() * ctx.whole_note;
                ((ctx.player.play_note)(ctx, *d, p), dur)
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
                ctx.pitch = ctx.pitch + *p;
                m.perf(players, ctx)
            }
            Self::Modify(Control::Instrument(i), m) => {
                ctx.instrument = i.clone();
                m.perf(players, ctx)
            }
            Self::Modify(Control::Phrase(phrases), m) => {
                (ctx.player.interpret_phrase)(m, players, ctx, phrases)
            }
            Self::Modify(Control::Player(p), m) => {
                // TODO
                let player = players.get(p).expect("not found player");
                ctx.player = player;
                m.perf(players, ctx)
            }
            Self::Modify(Control::KeySig(pc, mode), m) => {
                ctx.key = (*pc, *mode);
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
    pub start_time: TimePoint,
    pub instrument: InstrumentName,
    pub pitch: AbsPitch,
    pub duration: Duration,
    pub volume: Volume,
    /// Used for instruments [other than MIDI][InstrumentName::Custom].
    pub params: Vec<OrderedFloat<f64>>,
}

/// Measured in seconds both.
pub type TimePoint = Ratio<u32>;
pub type Duration = Ratio<u32>;

#[derive(Debug)]
/// The state of the [`Performance`] that changes
/// as we go through the interpretation.
pub struct Context<'p, P> {
    pub start_time: TimePoint,
    pub player: &'p Player<P>,
    pub instrument: InstrumentName,
    pub whole_note: Duration,
    pub pitch: AbsPitch,
    pub volume: Volume,
    pub key: (PitchClass, Mode),
}

impl<'p, P> Clone for Context<'p, P> {
    fn clone(&self) -> Self {
        let Self {
            start_time,
            player,
            instrument,
            whole_note,
            pitch,
            volume,
            key,
        } = self;
        Self {
            start_time: *start_time,
            player: *player,
            instrument: instrument.clone(),
            whole_note: *whole_note,
            pitch: *pitch,
            volume: *volume,
            key: *key,
        }
    }
}

/// Defines a tempo of X notes per minute
fn metro(setting: u32, note_dur: Dur) -> Duration {
    Ratio::from_integer(60) / (Ratio::from_integer(setting) * note_dur.into_ratio())
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NoteAttribute {
    Volume(Volume),
    Fingering(u32),
    Dynamics(String),
    Params(Vec<OrderedFloat<f64>>),
}

type PlayerMap<P> = HashMap<PlayerName, Player<P>>;

pub struct Player<P> {
    pub name: String,
    pub play_note: NoteFun<P>,
    pub interpret_phrase: PhraseFun<P>,
    pub notate_player: NotateFun<P>,
}

impl<P> fmt::Debug for Player<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Player {}", self.name)
    }
}

type NoteFun<P> = Box<dyn Fn(Context<P>, Dur, &P) -> Performance>;
type PhraseFun<P> = Box<
    dyn Fn(&Music<P>, &PlayerMap<P>, Context<P>, &[PhraseAttribute]) -> (Performance, Duration),
>;
// TODO: producing a properly notated score is not defined yet
type NotateFun<P> = std::marker::PhantomData<P>;

pub mod defaults {
    use num_traits::{ops::checked::CheckedSub as _, One as _};

    use crate::instruments::StandardMidiInstrument;

    use super::{
        super::{
            phrases::{Articulation, Dynamic, Tempo},
            pitch::Pitch,
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
        Box::new(move |ctx, dur, (note_pitch, attrs)| {
            let Context {
                start_time,
                player: _ignore_player,
                instrument,
                whole_note,
                pitch,
                volume,
                key: _ignore_key,
            } = ctx.clone();
            let init = Event {
                start_time,
                instrument,
                pitch: note_pitch.abs() + pitch,
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
        Box<dyn Fn(&Context<(P, Vec<Attr>)>, &Attr, Event) -> Event>;

    // Transform the whole performance according to [`Context`] and [`PhraseAttribute`].
    // type PhraseAttributeHandler = Box<dyn Fn(Performance, &PhraseAttribute) -> Performance>;

    pub fn default_interpret_phrase<P, PhraseF>(attr_modifier: PhraseF) -> PhraseFun<P>
    where
        Player<P>: Default,
        PhraseF: Fn(Performance, &PhraseAttribute) -> Performance + 'static,
    {
        Box::new(move |music, players, ctx, attrs| {
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
                volume: Volume((x * Ratio::from_integer(event.volume.0)).to_integer()),
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
        mut ctx: Context<P>,
        attrs: &[PhraseAttribute],
    ) -> (Performance, Duration)
    where
        Player<P>: Default,
    {
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
                volume: Volume(new_volume.to_integer() as u8),
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
                _ => (perf, dur),
            })
    }

    fn fancy_phrase_attribute_handler(perf: Performance, attr: &PhraseAttribute) -> Performance {
        match attr {
            PhraseAttribute::Dyn(Dynamic::Accent(x)) => perf.map(|event| Event {
                volume: Volume((x * Ratio::from_integer(event.volume.0)).to_integer()),
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
            PhraseAttribute::Art(_) | PhraseAttribute::Orn(_) => perf,
        }
    }

    impl<P> Player<P>
    where
        P: 'static,
        Self: Default,
    {
        /// All like the [default][Self::default] one but
        /// with changed interpretations of the [phrases][PhraseAttribute].
        pub fn fancy() -> Self {
            Self {
                interpret_phrase: Box::new(fancy_interpret_phrase),
                ..Self::default()
            }
        }
    }

    impl<'p, P> Context<'p, P> {
        pub fn with_player(player: &'p Player<P>) -> Self {
            Self {
                start_time: TimePoint::from_integer(0),
                player,
                instrument: StandardMidiInstrument::AcousticGrandPiano.into(),
                whole_note: metro(120, Dur::QN),
                pitch: AbsPitch::from(0),
                volume: Volume::loudest(),
                key: (PitchClass::C, Mode::Major),
            }
        }
    }
}
