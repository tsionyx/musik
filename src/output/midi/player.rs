use std::{
    collections::HashSet,
    io::Write as _,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::sleep,
    time::Instant,
};

use midly::{
    live::LiveEvent,
    num::{u4, u7},
    MidiMessage, Timing, TrackEventKind,
};
use once_cell::sync::Lazy;

use super::{
    convert::{tick_size, AbsTimeTrack},
    io::Connection,
};

#[derive(Debug)]
pub struct MidiPlayer {
    // TODO: allow pause (see https://github.com/insomnimus/nodi/blob/main/src/player.rs)
    conn: Connection,
    currently_played: HashSet<(u4, u7, u7)>,
}

type AnyError = Box<dyn std::error::Error>;

impl MidiPlayer {
    // TODO: provide the port here
    pub fn make_default() -> Result<Self, AnyError> {
        let conn = Connection::get_default()?;
        Ok(Self {
            conn,
            currently_played: HashSet::new(),
        })
    }

    pub fn play(&mut self, track: AbsTimeTrack<'_>, timing: Timing) -> std::io::Result<()> {
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
                }

                sleep(sec_per_tick);
            }
        }
        Ok(())
    }

    fn sync_currently_played(&mut self, msg: &TrackEventKind<'_>) {
        if let TrackEventKind::Midi { channel, message } = msg {
            match message {
                MidiMessage::NoteOn { key, vel } => {
                    if !self.currently_played.insert((*channel, *key, *vel)) {
                        // TODO: warn on repeating note
                    }
                }
                MidiMessage::NoteOff { key, vel } => {
                    if !self.currently_played.remove(&(*channel, *key, *vel)) {
                        // TODO: warn on stopping the note that was not started
                    }
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
            // TODO: log
            // println!(
            //     "Dropping the {:?}: {} notes unfinished",
            //     std::any::type_name::<Self>(),
            //     notes_left
            // );
            if let Err(_err) = self.stop_all() {
                // TODO: also log here on err
            }
        }
    }
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
