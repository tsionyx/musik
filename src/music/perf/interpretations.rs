//!  Defines default interpreters for the [`Player`]s.
use std::iter;

use intertrait::{cast_to, castable_to};
use itertools::Itertools as _;
use num_rational::Ratio;
use num_traits::{ops::checked::CheckedSub as _, One as _, Zero as _};

use crate::{
    music::{
        phrase::{Articulation, Dynamic, Ornament, PhraseAttribute, Tempo, TrillOptions},
        Music, NoteAttribute,
    },
    prim::{
        duration::Dur,
        pitch::{AbsPitch, Pitch},
        scale::KeySig,
        volume::Volume,
    },
    utils::CloneableIterator,
};

use super::{player::Player, Context, Duration, Event, Performance, TimePoint};

/// Annotate [`Event`] with attributes.
pub trait EventAnnotator<P, A> {
    /// Transform the event according to attribute and [`Context`].
    fn modify_event_with_attr(&self, event: Event, attr: &A, ctx: &Context<(P, Vec<A>)>) -> Event;
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
/// Most basic interpretation of [Player]'s capabilities.
pub struct DefaultPlayer {}

#[cast_to]
impl Player<Pitch> for DefaultPlayer {
    fn name(&self) -> &'static str {
        "Default (Pitch)"
    }

    fn play_note(&self, (dur, note_pitch): (Dur, &Pitch), ctx: Context<Pitch>) -> Performance {
        let event = default_event_from_note((dur, *note_pitch), ctx);
        Performance::with_events(iter::once(event))
    }

    fn interpret_phrase(&self, perf: Performance, attr: &PhraseAttribute) -> Performance {
        default_interpret_phrase(perf, attr)
    }
}

#[cast_to]
impl Player<(Pitch, Volume)> for DefaultPlayer {
    fn name(&self) -> &'static str {
        "Default (Pitch + Volume)"
    }

    fn play_note(
        &self,
        (dur, &(note_pitch, volume)): (Dur, &(Pitch, Volume)),
        ctx: Context<(Pitch, Volume)>,
    ) -> Performance {
        let event = default_event_from_note((dur, note_pitch), ctx);
        let event = Event { volume, ..event };
        Performance::with_events(iter::once(event))
    }

    fn interpret_phrase(&self, perf: Performance, attr: &PhraseAttribute) -> Performance {
        default_interpret_phrase(perf, attr)
    }
}

fn default_event_from_note<P>(note: (Dur, Pitch), ctx: Context<P>) -> Event {
    let Context {
        start_time,
        player: _ignore_player,
        instrument,
        whole_note,
        transpose_interval,
        volume,
        key: _ignore_key,
    } = ctx;
    Event {
        start_time,
        instrument,
        pitch: note.1.abs() + transpose_interval,
        duration: note.0.into_ratio() * whole_note,
        volume,
        params: vec![],
    }
}

fn default_interpret_phrase(perf: Performance, attr: &PhraseAttribute) -> Performance {
    let attr = *attr;
    match attr {
        PhraseAttribute::Dyn(Dynamic::Accent(x)) => perf.map(move |event| Event {
            volume: Volume::from((x * Ratio::from_integer(u8::from(event.volume.0))).to_integer()),
            ..event
        }),
        PhraseAttribute::Art(Articulation::Staccato(x)) => perf.map(move |event| Event {
            duration: x * event.duration,
            ..event
        }),
        PhraseAttribute::Art(Articulation::Legato(x)) => perf.map(move |event| Event {
            duration: x * event.duration,
            ..event
        }),

        PhraseAttribute::Dyn(_)
        | PhraseAttribute::Tmp(_)
        | PhraseAttribute::Art(_)
        | PhraseAttribute::Orn(_) => perf,
    }
}

castable_to!(DefaultPlayer => Player<(Pitch, Vec<NoteAttribute>)>);

impl<A> Player<(Pitch, Vec<A>)> for DefaultPlayer
where
    Self: EventAnnotator<Pitch, A>,
    A: Clone,
{
    fn name(&self) -> &'static str {
        "Default (Pitch with attributes)"
    }

    fn play_note(
        &self,
        (dur, (note_pitch, attrs)): (Dur, &(Pitch, Vec<A>)),
        ctx: Context<(Pitch, Vec<A>)>,
    ) -> Performance {
        let init = default_event_from_note((dur, *note_pitch), ctx.clone());
        let event = attrs.iter().fold(init, |acc, attr| {
            self.modify_event_with_attr(acc, attr, &ctx)
        });
        Performance::with_events(iter::once(event))
    }

    fn interpret_phrase(&self, perf: Performance, attr: &PhraseAttribute) -> Performance {
        default_interpret_phrase(perf, attr)
    }
}

impl<P> EventAnnotator<P, NoteAttribute> for DefaultPlayer {
    fn modify_event_with_attr(
        &self,
        event: Event,
        attr: &NoteAttribute,
        _: &Context<(P, Vec<NoteAttribute>)>,
    ) -> Event {
        match attr {
            NoteAttribute::Volume(vol) => Event {
                volume: *vol,
                ..event
            },
            NoteAttribute::Params(params) => Event {
                params: params.clone(),
                ..event
            },
            NoteAttribute::Fingering(_) | NoteAttribute::Dynamics(_) => event,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
/// Slightly customized [`DefaultPlayer`] with changed
/// interpretations of some of the [phrases][PhraseAttribute].
pub struct FancyPlayer {
    inner: DefaultPlayer,
}

// TODO: more impls for `FancyPlayer`

impl<A> Player<(Pitch, Vec<A>)> for FancyPlayer
where
    Self: EventAnnotator<Pitch, A>,
    DefaultPlayer: EventAnnotator<Pitch, A>,
    A: Clone,
{
    fn name(&self) -> &'static str {
        "Fancy"
    }

    fn play_note(
        &self,
        note: (Dur, &(Pitch, Vec<A>)),
        ctx: Context<(Pitch, Vec<A>)>,
    ) -> Performance {
        self.inner.play_note(note, ctx)
    }

    #[allow(clippy::too_many_lines, clippy::manual_let_else)]
    fn interpret_phrases(
        &self,
        music: &Music<(Pitch, Vec<A>)>,
        attrs: &[PhraseAttribute],
        mut ctx: Context<(Pitch, Vec<A>)>,
    ) -> (Performance, Duration) {
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

        let (perf, dur) = {
            let (perf, dur) = music.perf(ctx);
            let perf = attrs
                .iter()
                .fold(perf, |perf, attr| self.interpret_phrase(perf, attr));
            (perf, dur)
        };

        let e = perf.iter().next();
        let t0 = if let Some(e) = e {
            e.start_time
        } else {
            return (perf, dur);
        };

        let inflate = move |event: Event, coef: Ratio<u32>, sign: bool| {
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

        let stretch = move |event: Event, coef: Ratio<u32>, sign: bool| {
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
            .fold((perf, dur), move |(perf, dur), attr| match *attr {
                PhraseAttribute::Dyn(Dynamic::Crescendo(x)) => {
                    let perf = perf.map(move |e| inflate(e, x, true));
                    (perf, dur)
                }
                PhraseAttribute::Dyn(Dynamic::Diminuendo(x)) => {
                    let perf = perf.map(move |e| inflate(e, x, false));
                    (perf, dur)
                }
                PhraseAttribute::Tmp(Tempo::Ritardando(x)) => {
                    let perf = perf.map(move |e| stretch(e, x, true));
                    let dur = (Ratio::one() + x) * dur;
                    (perf, dur)
                }
                PhraseAttribute::Tmp(Tempo::Accelerando(x)) => {
                    let perf = perf.map(move |e| stretch(e, x, false));
                    let dur = Ratio::one().checked_sub(&x).unwrap_or_default() * dur;
                    (perf, dur)
                }
                PhraseAttribute::Orn(Ornament::Trill(opts)) => {
                    // exercise 8.2.1
                    let events = perf.iter().flat_map(move |e| trill(e, opts, key));
                    (Performance::with_events(events), dur)
                }
                PhraseAttribute::Orn(Ornament::Mordent) => {
                    // exercise 8.2.2
                    let events = perf.iter().flat_map(move |e| mordent(e, true, false, key));
                    (Performance::with_events(events), dur)
                }
                PhraseAttribute::Orn(Ornament::InvMordent) => {
                    // exercise 8.2.3
                    let events = perf.iter().flat_map(move |e| mordent(e, false, false, key));
                    (Performance::with_events(events), dur)
                }
                PhraseAttribute::Orn(Ornament::DoubleMordent) => {
                    // exercise 8.2.4
                    let events = perf.iter().flat_map(move |e| mordent(e, true, true, key));
                    (Performance::with_events(events), dur)
                }
                PhraseAttribute::Orn(Ornament::DiatonicTrans(i)) => {
                    // exercise 8.5
                    let perf = perf.map(move |e| Event {
                        pitch: e.pitch.diatonic_trans(key, i),
                        ..e
                    });
                    (perf, dur)
                }
                _ => (perf, dur),
            })
    }

    fn interpret_phrase(&self, perf: Performance, attr: &PhraseAttribute) -> Performance {
        match *attr {
            PhraseAttribute::Dyn(Dynamic::Accent(x)) => perf.map(move |event| Event {
                volume: Volume::from(
                    (x * Ratio::from_integer(u8::from(event.volume.0))).to_integer(),
                ),
                ..event
            }),
            PhraseAttribute::Dyn(_) | PhraseAttribute::Tmp(_) => {
                // already handled in the `self.interpret_phrases`
                perf
            }
            PhraseAttribute::Art(Articulation::Staccato(x)) => perf.map(move |event| Event {
                duration: x * event.duration,
                ..event
            }),
            PhraseAttribute::Art(Articulation::Legato(x)) => perf.map(move |event| Event {
                duration: x * event.duration,
                ..event
            }),
            PhraseAttribute::Art(Articulation::Slurred(x)) => {
                // the same as Legato, but do not extend the duration of the last note(s)
                let last_start_time = perf.iter().map(|e| e.start_time).max();
                if let Some(last_start_time) = last_start_time {
                    perf.map(move |event| {
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
                let end_of_the_phrase = perf.iter().map(|e| e.start_time + e.duration).max();
                if let Some(last_event_end) = end_of_the_phrase {
                    perf.map(move |event| {
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
                Performance::with_events(arpeggio(perf.iter(), true).into_iter())
            }
            PhraseAttribute::Orn(Ornament::ArpeggioDown) => {
                Performance::with_events(arpeggio(perf.iter(), false).into_iter())
            }
            PhraseAttribute::Art(_) | PhraseAttribute::Orn(_) => perf,
        }
    }
}

impl Performance {
    fn map<F>(self, f: F) -> Self
    where
        F: FnMut(Event) -> Event + Clone + 'static,
    {
        Self::with_events(self.iter().map(f))
    }
}

fn trill(
    event: Event,
    opts: TrillOptions<Ratio<u32>>,
    key: KeySig,
) -> impl Iterator<Item = Event> + Clone {
    let main_pitch = event.pitch;
    let mut trill_pitch = main_pitch.diatonic_trans(key, 1);
    if trill_pitch == main_pitch {
        // pitch is out of defined key
        trill_pitch = main_pitch.diatonic_trans(key, 2);
    }
    assert!(trill_pitch > main_pitch);

    let d = event.duration;
    let dur_seq: Box<dyn CloneableIterator<Item = Duration>> = match opts {
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
    durations: impl Iterator<Item = Duration> + Clone,
) -> impl Iterator<Item = Event> + Clone {
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
) -> impl Iterator<Item = Event> + Clone {
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
    let dur_seq: Box<dyn CloneableIterator<Item = Duration>> = if double {
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

fn arpeggio(events: impl Iterator<Item = Event>, up: bool) -> Vec<Event> {
    let chord_groups = events.group_by(|e| (e.start_time, e.duration));
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

    let size = u32::try_from(events.len()).expect("len is not low enough");
    match size {
        3 | 5 | 6 | 7 if d.numer() % size == 0 => {
            if d.numer() % size == 0 {
                // could split into equal intervals
                let short_dur = d / size;
                Box::new(events.into_iter().enumerate().map(move |(i, e)| Event {
                    start_time: s + short_dur * u32::try_from(i).expect("i is not low enough"),
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
                    let i = u32::try_from(i).expect("i is not low enough");
                    let duration = if i == equal_dur_notes {
                        d - (short_dur * equal_dur_notes)
                    } else {
                        short_dur
                    };

                    Event {
                        start_time: s + short_dur * i,
                        duration,
                        ..e
                    }
                }))
            }
        }
        4 | 8 => {
            let short_dur = d / size;
            Box::new(events.into_iter().enumerate().map(move |(i, e)| Event {
                start_time: s + short_dur * u32::try_from(i).expect("i is not low enough"),
                duration: short_dur,
                ..e
            }))
        }
        _ => Box::new(events.into_iter()),
    }
}
