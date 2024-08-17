//! <https://en.wikipedia.org/wiki/General_MIDI>

#![cfg_attr(not(feature = "play-midi"), allow(dead_code))]

use std::{borrow::Cow, collections::BTreeMap as Map, iter, time::Duration};

use itertools::Itertools as _;
use midly::{
    num::u15, Format, Fps, Header, MetaMessage, MidiMessage, Smf, Timing, Track, TrackEvent,
    TrackEventKind,
};
use num_traits::{CheckedAdd, CheckedMul};

use crate::{
    instruments::InstrumentName,
    music::perf::{Event, Performance},
    prim::volume::Volume,
    utils::append_with_last,
};

use super::{Channel, UserPatchMap};

pub(super) fn into_relative_time(track: AbsTimeTrack<'_>) -> Track<'static> {
    track
        .into_iter()
        .scan(0, |acc, (t, kind)| {
            let delta = t - *acc;
            *acc = t;
            Some(
                TrackEvent {
                    delta: delta.into(),
                    kind,
                }
                .to_static(),
            )
        })
        .collect()
}

impl Performance {
    /// Convert the [`Performance`] into MIDI representation
    /// to [save it into file][Self::save_to_file] or play.
    ///
    /// Optionally, the [patch map][UserPatchMap] could be provided to
    /// explicitly assign MIDI channels to instruments.
    pub fn into_midi(self, user_patch: Option<&UserPatchMap>) -> Result<Smf<'_>, String> {
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

        let tracks: Result<Vec<_>, String> = split
            .iter()
            .map(|(i, p)| {
                let mut track = into_relative_time(p.as_midi_track(i, &user_patch)?);
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
    ) -> Result<AbsTimeTrack<'_>, String> {
        let (channel, program) = user_patch
            .lookup(instrument)
            .ok_or_else(|| format!("Not found instrument {instrument:?}"))?;

        let tempo = 1_000_000 / BEATS_PER_SECOND;
        let set_tempo = TrackEventKind::Meta(MetaMessage::Tempo(tempo.into()));
        let setup_instrument = TrackEventKind::Midi {
            channel,
            message: MidiMessage::ProgramChange { program },
        };

        let messages = self
            .iter()
            .filter_map(|e| e.as_midi(channel))
            // TODO: sort the NoteOff more effective for the infinite `Performance`
            .flat_map(|(on, off)| iter::once(on).chain(iter::once(off)))
            .sorted_by_key(|(t, _)| *t);
        Ok(iter::once((0, set_tempo))
            .chain(iter::once((0, setup_instrument)))
            .chain(messages)
            .collect())
    }
}

const DEFAULT_TIME_DIV: u15 = u15::new(96);

// beat is a quarter note
const BEATS_PER_SECOND: u32 = 2;

pub(super) type TimedMessage<'a, T = u32> = (T, TrackEventKind<'a>);
pub(super) type AbsTimeTrack<'a, T = u32> = Vec<TimedMessage<'a, T>>;
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

fn to_absolute(
    track: Track<'_>,
    drop_track_end: bool,
) -> impl Iterator<Item = TimedMessage<'_>> + '_ {
    track
        .into_iter()
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
pub fn merge_tracks<'t>(
    tracks: impl Iterator<Item = Track<'t>>,
) -> impl Iterator<Item = TimedMessage<'t>> {
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
