use crate::output::midi::instruments::Instrument;

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Hash)]
#[allow(variant_size_differences)]
pub enum InstrumentName {
    Midi(Instrument),
    /// Marks the pitches in the [`Music`] as the specific [`PercussionSound`].
    Percussion,
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
