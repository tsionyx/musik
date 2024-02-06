pub mod instruments;
pub mod music;

pub use music::{
    adapters::TrillOptions,
    duration::Dur,
    interval::{AbsPitch, Interval, Octave},
    performance::{self, Performance, Player},
    phrases::PhraseAttribute,
    pitch::{Pitch, PitchClass},
    rests, Music,
};
