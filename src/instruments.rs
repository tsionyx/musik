//! Define operations with instruments.
//! Currently, the only available are [MIDI instruments][Instrument],
//! so here only the wrapping structure provided.
use crate::output::midi::instruments::Instrument;

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Hash)]
#[allow(variant_size_differences)]
/// Wrapper for the Instrument to use with [`Music`][crate::Music].
pub enum InstrumentName {
    /// On of the predefined [MIDI instrument][Instrument].
    Midi(Instrument),

    /// Marks the pitches in the [`Music`][crate::Music] as the specific [`PercussionSound`].
    Percussion,

    /// Custom non-MIDI instrument that could be identified by its name.
    Custom(String),
}

impl From<Instrument> for InstrumentName {
    fn from(value: Instrument) -> Self {
        Self::Midi(value)
    }
}

impl From<String> for InstrumentName {
    fn from(value: String) -> Self {
        Self::Custom(value)
    }
}
