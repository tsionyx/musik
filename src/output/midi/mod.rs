mod convert;
pub(crate) mod instruments;

#[cfg(feature = "play-midi")]
mod io;
#[cfg(feature = "play-midi")]
mod player;

use std::{collections::HashMap, path::Path};

use enum_map::Enum;

use crate::{instruments::InstrumentName, music::perf::Performance};

pub use self::instruments::{Instrument, PercussionSound};

type AnyError = Box<dyn std::error::Error>;

impl Performance {
    pub fn save_to_file<P: AsRef<Path>>(self, path: P) -> Result<(), AnyError> {
        let midi = self.into_midi(None)?;
        midi.save(path)?;
        Ok(())
    }

    #[cfg(feature = "play-midi")]
    pub fn play(self) -> Result<(), AnyError> {
        use self::{convert::merge_tracks, player::MidiPlayer};
        use midly::Smf;

        let mut player = MidiPlayer::make_default()?;
        let Smf { header, tracks } = self.into_midi(None)?;
        let single_track = merge_tracks(tracks);

        player.play(single_track, header.timing)?;
        Ok(())
    }
}

// up to 16 channels
type Channel = u8;

// up to 128 instruments
type ProgNum = u8;

#[derive(Debug, Clone)]
pub struct UserPatchMap {
    repr: HashMap<InstrumentName, Channel>,
}

impl UserPatchMap {
    const PERCUSSION: Channel = 9;
    // all but Percussion
    const AVAILABLE_CHANNELS: [Channel; 15] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12, 13, 14, 15];

    pub fn with_instruments(instruments: Vec<InstrumentName>) -> Result<Self, String> {
        // TODO: extend the range of instruments by combining non-overlapping tracks
        if instruments.len() > Self::AVAILABLE_CHANNELS.len() {
            return Err(format!("Too many instruments: {}", instruments.len()));
        }

        let map = instruments.into_iter().scan(0, |idx, instrument| {
            if instrument == InstrumentName::Percussion {
                Some((instrument, Self::PERCUSSION))
            } else {
                let channel = Self::AVAILABLE_CHANNELS[*idx];
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
                InstrumentName::Midi(i) => i.into_usize(),
                InstrumentName::Percussion => 0,
                InstrumentName::Custom(_) => 0,
            };
            (*x, u8::try_from(prog_num).expect("128 instruments only"))
        })
    }

    pub fn contains_all(&self, instruments: &[InstrumentName]) -> bool {
        instruments.iter().all(|i| self.lookup(i).is_some())
    }
}
