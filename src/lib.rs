pub mod instruments;
pub mod music;

pub use music::{
    duration::Dur,
    interval::{AbsPitch, Interval, Octave},
    pitch::{Pitch, PitchClass},
    rests, Music,
};
