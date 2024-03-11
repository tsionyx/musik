mod convert;
pub(crate) mod instruments;

#[cfg(feature = "play-midi")]
mod io;
#[cfg(feature = "play-midi")]
mod player;

use std::{collections::HashMap, path::Path};

use enum_map::Enum;
use log::info;
use midly::num::{u4, u7};

use crate::{instruments::InstrumentName, music::perf::Performance};

pub use self::instruments::{Instrument, PercussionSound};
#[cfg(feature = "play-midi")]
pub use self::player::MidiPlayer;

type AnyError = Box<dyn std::error::Error>;

impl Performance {
    pub fn save_to_file<P: AsRef<Path>>(self, path: P) -> Result<(), AnyError> {
        let midi = self.into_midi(None)?;
        info!("Saving to MIDI file {}", path.as_ref().display());
        midi.save(path)?;
        Ok(())
    }

    #[cfg(feature = "play-midi")]
    pub fn play(self) -> Result<(), AnyError> {
        use self::convert::merge_tracks;

        let mut player = MidiPlayer::make_default()?;
        let midly::Smf { header, tracks } = self.into_midi(None)?;
        let single_track = merge_tracks(tracks);

        info!("Playing MIDI with {} events", single_track.len());
        player.play(single_track, header.timing)?;
        Ok(())
    }
}

// up to 16 channels
type Channel = u4;

// up to 128 instruments
type ProgNum = u7;

#[derive(Debug, Clone)]
pub struct UserPatchMap {
    repr: HashMap<InstrumentName, Channel>,
}

impl UserPatchMap {
    const PERCUSSION: Channel = Channel::new(9);

    /// All but Percussion
    fn available_channels() -> [Channel; 15] {
        (0..16)
            .filter_map(|i| (i != Self::PERCUSSION.as_int()).then_some(Channel::new(i)))
            .collect::<Vec<_>>()
            .try_into()
            .expect("Should contains exactly 15 Channels")
    }

    pub fn with_instruments(instruments: Vec<InstrumentName>) -> Result<Self, String> {
        let available_channels = Self::available_channels();
        if instruments.len() > available_channels.len() {
            // TODO: extend the range of instruments by combining non-overlapping tracks
            return Err(format!("Too many instruments: {}", instruments.len()));
        }

        let map = instruments.into_iter().scan(0, |idx, instrument| {
            if instrument == InstrumentName::Percussion {
                Some((instrument, Self::PERCUSSION))
            } else {
                let channel = available_channels[*idx];
                *idx += 1;
                Some((instrument, channel))
            }
        });

        Ok(Self {
            repr: map.collect(),
        })
    }

    pub fn lookup(&self, instrument: &InstrumentName) -> Option<(Channel, ProgNum)> {
        self.repr.get(instrument).map(|x| {
            let prog_num = match instrument {
                InstrumentName::Midi(i) => i
                    .into_usize()
                    .try_into()
                    .expect("MIDI instruments should be less than 256"),
                InstrumentName::Percussion | InstrumentName::Custom(_) => 0,
            };
            (
                *x,
                ProgNum::try_from(prog_num).expect("exactly 128 instruments"),
            )
        })
    }

    pub fn contains_all(&self, instruments: &[InstrumentName]) -> bool {
        instruments.iter().all(|i| self.lookup(i).is_some())
    }
}
