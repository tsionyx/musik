use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    io::Write as _,
    iter,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::sleep,
    time::{Duration, Instant},
};

use itertools::Itertools as _;
use midly::{
    live::LiveEvent,
    num::{u15, u4, u7},
    Format, Fps, Header, MetaMessage, MidiMessage, Smf, Timing, Track, TrackEvent, TrackEventKind,
};
use once_cell::sync::Lazy;

use crate::{
    instruments::InstrumentName,
    music::{
        performance::{Event, Performance},
        Volume,
    },
};

use super::{
    transport::{get_default_connection, Connection},
    Channel, UserPatchMap,
};

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

    pub fn play(self) -> Result<(), AnyError> {
        let mut player = MidiPlayer::make_default()?;
        let Smf { header, tracks } = self.into_midi(None)?;
        let single_track = merge_tracks(tracks);

        player.play(single_track, header.timing)?;
        Ok(())
    }
}

struct MidiPlayer {
    // TODO: allow pause (see https://github.com/insomnimus/nodi/blob/main/src/player.rs)
    conn: Connection,
    currently_played: HashSet<(u4, u7, u7)>,
}

impl MidiPlayer {
    // TODO: provide the port here
    fn make_default() -> Result<Self, AnyError> {
        let conn = get_default_connection()?;
        Ok(Self {
            conn,
            currently_played: HashSet::new(),
        })
    }

    fn play(&mut self, track: AbsTimeTrack, timing: Timing) -> std::io::Result<()> {
        let sec_per_tick = tick_size(timing);
        let real_time = track
            .into_iter()
            .map(|(ticks, msg)| (ticks * sec_per_tick, msg));

        let start = Instant::now();
        for (t, msg) in real_time {
            while IS_RUNNING.load(Ordering::SeqCst) {
                let elapsed = start.elapsed();
                if elapsed >= t {
                    self.sync_currently_played(&msg);
                    if let Some(live) = msg.as_live_event() {
                        live.write_std(&mut self.conn)?;
                        self.conn.flush()?;
                    }
                    break;
                } else {
                    sleep(sec_per_tick);
                }
            }
        }
        Ok(())
    }

    fn sync_currently_played(&mut self, msg: &TrackEventKind) {
        if let TrackEventKind::Midi { channel, message } = msg {
            match message {
                MidiMessage::NoteOn { key, vel } => {
                    self.currently_played.insert((*channel, *key, *vel));
                }
                MidiMessage::NoteOff { key, vel } => {
                    self.currently_played.remove(&(*channel, *key, *vel));
                }
                _ => {}
            }
        }
    }

    fn stop_all(&mut self) -> std::io::Result<()> {
        for (channel, key, vel) in self.currently_played.drain() {
            let msg = LiveEvent::Midi {
                channel,
                message: MidiMessage::NoteOff { key, vel },
            };
            msg.write_std(&mut self.conn)?;
            self.conn.flush()?;
        }
        Ok(())
    }
}

impl Drop for MidiPlayer {
    fn drop(&mut self) {
        let notes_left = self.currently_played.len();
        if notes_left > 0 {
            println!(
                "Dropping the {:?}: {} notes unfinished",
                std::any::type_name::<Self>(),
                notes_left
            );
            let _ = self.stop_all();
        }
    }
}

fn tick_size(timing: Timing) -> Duration {
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
fn merge_tracks(mut tracks: Vec<Track>) -> AbsTimeTrack {
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

static IS_RUNNING: Lazy<Arc<AtomicBool>> = Lazy::new(|| {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    running
});
