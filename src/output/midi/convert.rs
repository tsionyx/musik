//! <https://en.wikipedia.org/wiki/General_MIDI>

#![cfg_attr(not(feature = "play-midi"), allow(dead_code))]

use std::{borrow::Cow, collections::BTreeMap as Map, fmt, iter, time::Duration};

use itertools::Itertools as _;
use midly::{
    num::u15, Format, Fps, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent,
    TrackEventKind,
};
use num_traits::{CheckedAdd, CheckedMul};

use crate::{
    instruments::InstrumentName,
    music::perf::{Event, Performance},
    prim::volume::Volume,
    utils::{append_with_last, merge_pairs_by},
};

use super::{Channel, UserPatchMap};

pub(super) fn into_relative_time<'t>(
    track: impl Iterator<Item = TimedMessage<'t, u32>>,
) -> impl Iterator<Item = TrackEvent<'t>> {
    track.scan(0, |acc, (t, kind)| {
        let delta = t - *acc;
        *acc = t;
        Some(TrackEvent {
            delta: delta.into(),
            kind,
        })
    })
}

impl Performance {
    /// Convert the [`Performance`] into MIDI representation
    /// to [save it into file][Self::save_to_file] or play.
    ///
    /// Optionally, the [patch map][UserPatchMap] could be provided to
    /// explicitly assign MIDI channels to instruments.
    pub fn into_midi(self, user_patch: Option<&UserPatchMap>) -> Result<Smf<'_>, Error> {
        let split = self.split_by_instruments();
        let user_patch = user_patch.and_then(|user_patch| {
            let instruments = split.keys();
            user_patch
                .contains_all(instruments)
                .then_some(Cow::Borrowed(user_patch))
        });

        let user_patch = user_patch.map_or_else(
            || {
                let instruments = split.keys().cloned().collect();
                UserPatchMap::with_instruments(instruments).map(Cow::Owned)
            },
            Ok,
        )?;

        let file_type = if split.len() == 1 {
            Format::SingleTrack
        } else {
            Format::Parallel
        };

        let tracks: Result<Vec<_>, Error> = split
            .into_iter()
            .map(move |(i, p)| {
                let mut track: Vec<_> =
                    into_relative_time(p.as_midi_track(&i, &user_patch)?).collect();
                track.push(TrackEvent {
                    delta: 0.into(),
                    kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
                });
                Ok(track)
            })
            .collect();

        tracks.map(|tracks| Smf {
            header: Header::new(file_type, Timing::Metrical(DEFAULT_TIME_DIV)),
            tracks,
        })
    }

    fn split_by_instruments(self) -> Map<InstrumentName, Self> {
        self.iter()
            .map(|e| (e.instrument.clone(), e))
            .into_group_map()
            .into_iter()
            .map(|(k, v)| (k, Self::with_events(v.into_iter())))
            .collect()
    }

    fn as_midi_track(
        &self,
        instrument: &InstrumentName,
        user_patch: &UserPatchMap,
    ) -> Result<impl Iterator<Item = TimedMessage<'static>>, Error> {
        let (channel, program) = user_patch
            .lookup(instrument)
            .ok_or_else(|| Error::NotFoundInstrument(instrument.clone()))?;

        let tempo = 1_000_000 / BEATS_PER_SECOND;
        let set_tempo = TrackEventKind::Meta(MetaMessage::Tempo(tempo.into()));
        let setup_instrument = TrackEventKind::Midi {
            channel,
            message: MidiMessage::ProgramChange { program },
        };

        let pairs = self.iter().filter_map(move |e| e.as_midi(channel));

        let sorted = merge_pairs_by(pairs, |e1, e2| e1.0 < e2.0);
        Ok(iter::once((0, set_tempo))
            .chain(iter::once((0, setup_instrument)))
            .chain(sorted))
    }
}

#[derive(Debug, Clone)]
/// Error while converting to MIDI.
pub enum Error {
    /// Instrument cannot be found in the [`UserPatchMap`] provided.
    NotFoundInstrument(InstrumentName),
    /// Too many instruments provided to create the [`UserPatchMap`].
    TooManyInstruments(usize),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFoundInstrument(instrument) => {
                write!(f, "Not found instrument {instrument:?}")
            }
            Self::TooManyInstruments(n) => {
                write!(f, "Too many instruments: {n}")
            }
        }
    }
}

impl std::error::Error for Error {}

const DEFAULT_TIME_DIV: u15 = u15::new(96);

// beat is a quarter note
const BEATS_PER_SECOND: u32 = 2;

pub(super) type TimedMessage<'a, T = u32> = (T, TrackEventKind<'a>);
type Pair<T> = (T, T);

impl Event {
    fn as_midi(&self, channel: Channel) -> Option<Pair<TimedMessage<'static>>> {
        let ticks_per_second = u32::from(u16::from(DEFAULT_TIME_DIV)) * BEATS_PER_SECOND;

        let start = (self.start_time.checked_mul(&ticks_per_second.into())?).to_integer();
        let end = self
            .start_time
            .checked_add(&self.duration)?
            .checked_mul(&ticks_per_second.into())?
            .to_integer();
        let key = u8::from(self.pitch.get_inner());
        let vel = self.volume.clamp(Volume::softest(), Volume::loudest());

        let event_on = TrackEventKind::Midi {
            channel,
            message: MidiMessage::NoteOn {
                key: key.into(),
                vel: u8::from(vel.0).into(),
            },
        };

        let event_off = TrackEventKind::Midi {
            channel,
            message: MidiMessage::NoteOff {
                key: key.into(),
                vel: u8::from(vel.0).into(),
            },
        };
        Some(((start, event_on), (end, event_off)))
    }
}

fn to_absolute<'t>(
    track: impl Iterator<Item = TrackEvent<'t>> + 't,
    drop_track_end: bool,
) -> impl Iterator<Item = TimedMessage<'t>> + 't {
    track
        .filter(move |t| {
            !(drop_track_end && t.kind == TrackEventKind::Meta(MetaMessage::EndOfTrack))
        })
        .scan(0, |acc, t| {
            let abs_time = u32::from(t.delta) + *acc;
            *acc = abs_time;
            Some((abs_time, t.kind))
        })
}

/// Join all tracks into a single track with absolute time:
/// - convert to absolute time  (remove `TrackEnd`)
/// - merge (<https://hackage.haskell.org/package/HCodecs-0.5.2/docs/src/Codec.Midi.html#merge>)
/// - add `TrackEnd`
pub fn merge_tracks<'t, Track>(
    tracks: impl Iterator<Item = Track>,
) -> impl Iterator<Item = TimedMessage<'t>>
where
    Track: Iterator<Item = TrackEvent<'t>> + 't,
{
    let init: Box<dyn Iterator<Item = (u32, TrackEventKind<'t>)>> = Box::new(iter::empty());
    let single = tracks
        .map(|t| to_absolute(t, true))
        .fold(init, |acc, track| {
            let acc = acc.merge_by(track, |(t1, _), (t2, _)| t1 < t2);
            Box::new(acc)
        });

    append_with_last(single, |(last, _)| {
        Some((last, TrackEventKind::Meta(MetaMessage::EndOfTrack)))
    })
}

pub(super) fn tick_size(timing: Timing) -> Duration {
    let ticks_per_second = match timing {
        Timing::Metrical(tick) => u32::from(u16::from(tick)) * BEATS_PER_SECOND,
        Timing::Timecode(fps, sub) => {
            let fps: u32 = match fps {
                Fps::Fps24 => 24,
                Fps::Fps25 => 25,
                Fps::Fps29 => 29,
                Fps::Fps30 => 30,
            };
            fps * u32::from(sub)
        }
    };

    Duration::from_secs_f64(f64::from(ticks_per_second).recip())
}
