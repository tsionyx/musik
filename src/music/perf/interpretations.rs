//!  Defines default interpreters for the [`Player`]s.
use std::{iter, sync::Arc};

use itertools::Itertools as _;
use num_rational::Ratio;
use num_traits::{ops::checked::CheckedSub as _, One as _, Zero as _};

use crate::{
    music::{
        phrase::{Articulation, Dynamic, Ornament, PhraseAttribute, Tempo, TrillOptions},
        AttrNote, Music, NoteAttribute,
    },
    prim::{
        duration::Dur,
        pitch::{AbsPitch, Pitch},
        scale::KeySig,
        volume::Volume,
    },
};

use super::{
    player::{PhraseFun, Player},
    Context, Duration, Event, Performance, PlayerMap, TimePoint,
};

pub fn default_play_note<A, NoteWithAttributeHandler>(
    ctx: Context<'_, (Pitch, Vec<A>)>,
    dur: Dur,
    (note_pitch, attrs): &(Pitch, Vec<A>),
    attr_modifier: NoteWithAttributeHandler,
) -> Performance
where
    NoteWithAttributeHandler: Fn(Event, &A, &Context<'_, (Pitch, Vec<A>)>) -> Event,
{
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
        .fold(init, |acc, attr| attr_modifier(acc, attr, &ctx));
    Performance::with_events(vec![event])
}

pub trait NoteAttributeHandler<A> {
    fn handle<P>(self, attr: &A, ctx: &Context<'_, (P, Vec<A>)>) -> Self;
}

impl NoteAttributeHandler<NoteAttribute> for Event {
    fn handle<P>(self, attr: &NoteAttribute, _: &Context<'_, (P, Vec<NoteAttribute>)>) -> Self {
        match attr {
            NoteAttribute::Volume(vol) => Self {
                volume: *vol,
                ..self
            },
            NoteAttribute::Params(params) => Self {
                params: params.clone(),
                ..self
            },
            NoteAttribute::Fingering(_) | NoteAttribute::Dynamics(_) => self,
        }
    }
}

// Transform the event according to [`Context`] and Attribute.
// type NoteWithAttributeHandler<P, Attr> =
//     Box<dyn Fn(&Context<'_, (P, Vec<Attr>)>, &Attr, Event) -> Event>;

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

pub fn default_phrase_attribute_handler(perf: Performance, attr: &PhraseAttribute) -> Performance {
    match attr {
        PhraseAttribute::Dyn(Dynamic::Accent(x)) => perf.map(|event| Event {
            volume: Volume::from((x * Ratio::from_integer(u8::from(event.volume.0))).to_integer()),
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
            play_note: Arc::new(move |ctx, dur, note| {
                default_play_note(ctx, dur, note, Event::handle)
            }),
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

fn trill(event: Event, opts: TrillOptions<Ratio<u32>>, key: KeySig) -> impl Iterator<Item = Event> {
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

fn mordent(event: Event, upper: bool, double: bool, key: KeySig) -> impl Iterator<Item = Event> {
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
            volume: Volume::from((x * Ratio::from_integer(u8::from(event.volume.0))).to_integer()),
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
                    if let Some(lengthened_duration) = last_event_end.checked_sub(&event.start_time)
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
