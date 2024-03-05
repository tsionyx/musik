//! <https://en.wikipedia.org/wiki/General_MIDI>

#![cfg_attr(not(feature = "play-midi"), allow(dead_code))]
use std::{borrow::Cow, collections::HashMap, iter, time::Duration};

use itertools::Itertools as _;
use midly::{
    num::u15, Format, Fps, Header, MetaMessage, MidiMessage, Smf, Timing, Track, TrackEvent,
    TrackEventKind,
};

use crate::{
    instruments::InstrumentName,
    music::perf::{Event, Performance},
    prim::volume::Volume,
};

use super::{Channel, UserPatchMap};

pub(super) fn into_relative_time(track: AbsTimeTrack) -> Track<'static> {
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
    pub fn into_midi(self, user_patch: Option<&UserPatchMap>) -> Result<Smf, String> {
        let split = self.split_by_instruments();
        let instruments: Vec<_> = split.keys().cloned().collect();
        let user_patch = user_patch.and_then(|user_patch| {
            user_patch
                .contains_all(&instruments)
                .then_some(Cow::Borrowed(user_patch))
        });

        let user_patch = match user_patch {
            Some(x) => x,
            None => Cow::Owned(UserPatchMap::with_instruments(instruments)?),
        };

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

    fn split_by_instruments(self) -> HashMap<InstrumentName, Self> {
        self.into_events()
            .into_iter()
            .map(|e| (e.instrument.clone(), e))
            .into_group_map()
            .into_iter()
            .map(|(k, v)| (k, Self::with_events(v)))
            .collect()
    }

    fn as_midi_track(
        &self,
        instrument: &InstrumentName,
        user_patch: &UserPatchMap,
    ) -> Result<AbsTimeTrack, String> {
        let (channel, program) = user_patch
            .lookup(instrument)
            .ok_or_else(|| format!("Not found instrument {:?}", instrument))?;

        let tempo = 1_000_000 / BEATS_PER_SECOND;
        let set_tempo = TrackEventKind::Meta(MetaMessage::Tempo(tempo.into()));
        let setup_instrument = TrackEventKind::Midi {
            channel: channel.into(),
            message: MidiMessage::ProgramChange {
                program: program.into(),
            },
        };

        let messages = self
            .iter()
            .flat_map(|e| {
                // TODO: sort the NoteOff more effective for the infinite `Performance`
                let (on, off) = e.as_midi(channel);
                iter::once(on).chain(iter::once(off))
            })
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

type TimedMessage<'a, T = u32> = (T, TrackEventKind<'a>);
pub(super) type AbsTimeTrack<'a, T = u32> = Vec<TimedMessage<'a, T>>;
type Pair<T> = (T, T);

impl Event {
    fn as_midi(&self, channel: Channel) -> Pair<TimedMessage> {
        let ticks_per_second = u32::from(u16::from(DEFAULT_TIME_DIV)) * BEATS_PER_SECOND;

        let start = (self.start_time * ticks_per_second).to_integer();
        let end = ((self.start_time + self.duration) * ticks_per_second).to_integer();
        let key = u8::from(self.pitch.get_inner());
        let vel = self.volume.clamp(Volume::softest(), Volume::loudest());
        (
            (
                start,
                TrackEventKind::Midi {
                    channel: channel.into(),
                    message: MidiMessage::NoteOn {
                        key: key.into(),
                        vel: vel.0.into(),
                    },
                },
            ),
            (
                end,
                TrackEventKind::Midi {
                    channel: channel.into(),
                    message: MidiMessage::NoteOff {
                        key: key.into(),
                        vel: vel.0.into(),
                    },
                },
            ),
        )
    }
}

fn to_absolute(track: Track, drop_track_end: bool) -> AbsTimeTrack {
    track
        .into_iter()
        .filter(|t| !(drop_track_end && t.kind == TrackEventKind::Meta(MetaMessage::EndOfTrack)))
        .scan(0, |acc, t| {
            let abs_time = u32::from(t.delta) + *acc;
            *acc = abs_time;
            Some((abs_time, t.kind))
        })
        .collect()
}

/// Join all tracks into a single track with absolute time:
/// - convert to absolute time  (remove TrackEnd)
/// - merge (https://hackage.haskell.org/package/HCodecs-0.5.2/docs/src/Codec.Midi.html#merge)
/// - add TrackEnd
pub fn merge_tracks(mut tracks: Vec<Track>) -> AbsTimeTrack {
    if tracks.is_empty() {
        return AbsTimeTrack::new();
    }
    let first = tracks.remove(0);
    let first = to_absolute(first, true);

    let mut single = tracks
        .into_iter()
        .map(|t| to_absolute(t, true))
        .fold(first, |acc, track| {
            acc.into_iter()
                .merge_by(track, |(t1, _), (t2, _)| t1 < t2)
                .collect()
        });

    if let Some((last, _)) = single.last() {
        single.push((*last, TrackEventKind::Meta(MetaMessage::EndOfTrack)))
    }
    single
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
