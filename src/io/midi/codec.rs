use std::{borrow::Cow, collections::HashMap, iter, path::Path};

use itertools::Itertools as _;
use midly::{
    num::u15, Format, Header, MetaMessage, MidiMessage, Smf, Timing, Track, TrackEvent,
    TrackEventKind,
};

use crate::{
    instruments::InstrumentName,
    music::{
        performance::{Event, Performance},
        Volume,
    },
};

use super::{Channel, UserPatchMap};

fn into_relative_time(track: AbsTimeTrack) -> Track<'static> {
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
type AbsTimeTrack<'a, T = u32> = Vec<TimedMessage<'a, T>>;
type Pair<T> = (T, T);

impl Event {
    fn as_midi(&self, channel: Channel) -> Pair<TimedMessage> {
        let ticks_per_second = u32::from(u16::from(DEFAULT_TIME_DIV)) * BEATS_PER_SECOND;

        let start = (self.start_time * ticks_per_second).to_integer();
        let end = ((self.start_time + self.duration) * ticks_per_second).to_integer();
        let key: u8 = self.pitch.get_inner().try_into().expect("Bad pitch");
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

type AnyError = Box<dyn std::error::Error>;

impl Performance {
    pub fn save_to_file<P: AsRef<Path>>(self, path: P) -> Result<(), AnyError> {
        let midi = self.into_midi(None)?;
        midi.save(path)?;
        Ok(())
    }

    pub fn play() -> Result<(), AnyError> {
        // TODO:
        //  1. join all tracks into a single track with absolute time:
        //    a. convert to absolute time  (remove TrackEnd)
        //    b. merge (https://hackage.haskell.org/package/HCodecs-0.5.2/docs/src/Codec.Midi.html#merge)
        //    d. add TrackEnd
        //  2. convert absolute time into Instant points using the predefined TimeDiv
        //     https://hackage.haskell.org/package/HCodecs-0.5.2/docs/src/Codec.Midi.html#toRealTime
        //  3. run a thread by sleeping in a loop until the new event is received
        //     https://github.com/insomnimus/nodi/blob/main/src/player.rs#L40
        //  4. take care of stopping all notes on Ctrl-C:
        //     https://github.com/insomnimus/nodi/blob/main/src/player.rs#L91
        Ok(())
    }
}
