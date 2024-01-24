pub mod instruments;
pub mod music;

pub use music::{
    adapters::TrillOptions,
    duration::Dur,
    interval::{AbsPitch, Interval, Octave},
    pitch::{Pitch, PitchClass},
    rests, Music,
};
