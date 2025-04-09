use std::{
    collections::HashSet,
    io::Write as _,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::sleep,
    time::{Duration, Instant},
};

use log::{info, trace, warn};
use midly::{
    live::LiveEvent,
    num::{u4, u7},
    MidiMessage, Timing, TrackEventKind,
};

#[rustversion::before(1.80)]
use once_cell::sync::Lazy;
#[rustversion::since(1.80)]
use std::sync::LazyLock as Lazy;

use super::{
    convert::{tick_size, TimedMessage},
    io::Connection,
};

#[derive(Debug)]
/// Holds the connection to MIDI device
/// and handles the playback of provided MIDI track
/// through the former.
pub struct MidiPlayer {
    conn: Connection,
    currently_played: HashSet<(u4, u7, u7)>,
    config: Config,
}

#[derive(Debug, Copy, Clone)]
/// Configuration for a [`MidiPlayer`].
///
/// Use it in [`MidiPlayer::with_config`].
pub struct Config {
    /// Check the SIGINT to correctly stop the performance.
    ///
    /// Default: true.
    pub check_ctrl_c: bool,

    /// Worst allowed latency.
    ///
    /// The lower the value, the more CPU will it take to play.
    ///
    /// Default: 1ms.
    pub max_latency: Duration,

    /// Best allowed latency.
    ///
    /// Lowering the latency too much will likely be useless
    /// due to spending some time for handling a single note.
    ///
    /// Default: 50mcs.
    pub min_latency: Duration,
    // TODO: allow pause (see https://github.com/insomnimus/nodi/blob/main/src/player.rs)
}

impl Default for Config {
    fn default() -> Self {
        Self {
            check_ctrl_c: true,
            max_latency: Duration::from_millis(1),
            min_latency: Duration::from_micros(50),
        }
    }
}

type AnyError = Box<dyn std::error::Error>;

impl MidiPlayer {
    /// Create a [MIDI player][Self] by choosing
    /// the most appropriate MIDI device.
    ///
    /// To choose the device manually, use the [`Self::with_port`].
    pub fn make_default() -> Result<Self, AnyError> {
        let conn = Connection::get_default()?;
        Ok(Self {
            conn,
            currently_played: HashSet::new(),
            config: Config::default(),
        })
    }

    // TODO: provide the port here
    //  pub fn with_port() -> Result<Self, AnyError> {}

    /// Create a [MIDI player][Self] with [`Config`] by choosing
    /// the most appropriate MIDI device.
    ///
    /// To choose the device manually, use the [`Self::with_port`].
    pub fn with_config(config: Config) -> Result<Self, AnyError> {
        let conn = Connection::get_default()?;
        Ok(Self {
            conn,
            currently_played: HashSet::new(),
            config,
        })
    }

    /// Play the series of [MIDI events][midly::TrackEventKind]
    /// by adjusting the playback speed with [`Timing`].
    #[allow(single_use_lifetimes)] // false positive
    pub fn play<'t>(
        &mut self,
        track: impl Iterator<Item = TimedMessage<'t>>,
        timing: Timing,
    ) -> std::io::Result<()> {
        let sec_per_tick = tick_size(timing);
        let real_time = track.map(|(ticks, msg)| (ticks * sec_per_tick, msg));

        let start = Instant::now();
        for (t, msg) in real_time {
            if !self.continue_play() {
                break;
            }
            while self.continue_play() {
                let elapsed = start.elapsed();
                // wait for the right time of the event
                if elapsed >= t {
                    self.sync_currently_played(&msg);
                    if let Some(live) = msg.as_live_event() {
                        self.play_event(live)?;
                    }
                    break;
                }

                sleep(self.latency(timing));
            }
        }
        Ok(())
    }

    fn continue_play(&self) -> bool {
        if self.config.check_ctrl_c {
            IS_RUNNING.load(Ordering::SeqCst)
        } else {
            true
        }
    }

    /// The interval of time to check the stream of events.
    fn latency(&self, timing: Timing) -> Duration {
        let sec_per_tick = tick_size(timing);
        // do not allow the precision to be worse than `self.config.max_latency`
        sec_per_tick.clamp(self.config.min_latency, self.config.max_latency)
    }

    fn play_event(&mut self, event: LiveEvent<'_>) -> std::io::Result<()> {
        if let LiveEvent::Midi { channel, message } = event {
            match message {
                MidiMessage::NoteOn { key, vel } => {
                    trace!("Playing MIDI #{key} on channel {channel} with volume {vel}");
                }
                MidiMessage::NoteOff { key, vel } => {
                    trace!("Stopping MIDI #{key} on channel {channel} with volume {vel}");
                }
                _ => {}
            }
        }

        event.write_std(&mut self.conn)?;
        // flush immediately to prevent buffering
        self.conn.flush()
    }

    fn sync_currently_played(&mut self, msg: &TrackEventKind<'_>) {
        if let TrackEventKind::Midi { channel, message } = msg {
            match message {
                MidiMessage::NoteOn { key, vel } => {
                    let note = (*channel, *key, *vel);
                    if !self.currently_played.insert(note) {
                        warn!("Repeating note: {note:?}");
                    }
                }
                MidiMessage::NoteOff { key, vel } => {
                    let note = (*channel, *key, *vel);
                    if !self.currently_played.remove(&(*channel, *key, *vel)) {
                        warn!("Stopping the note that was not started: {note:?}");
                    }
                }
                _ => {}
            }
        }
    }

    fn stop_all(&mut self) -> std::io::Result<()> {
        let mut played = std::mem::take(&mut self.currently_played);

        let notes_left = played.len();
        if notes_left > 0 {
            info!(
                "Stopping the {:?}: {} notes unfinished",
                std::any::type_name::<Self>(),
                notes_left
            );
            for (channel, key, vel) in played.drain() {
                let msg = LiveEvent::Midi {
                    channel,
                    message: MidiMessage::NoteOff { key, vel },
                };
                self.play_event(msg)?;
            }
        }
        Ok(())
    }
}

impl Drop for MidiPlayer {
    fn drop(&mut self) {
        if let Err(err) = self.stop_all() {
            warn!("Stopping the player failed: {err:?}");
        }
    }
}

static IS_RUNNING: Lazy<Arc<AtomicBool>> = Lazy::new(|| {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        info!("Received Ctrl-C");
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    running
});
