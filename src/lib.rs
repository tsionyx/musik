pub mod instruments;
pub mod music;

pub use music::{
    duration::Dur,
    interval::{AbsPitch, Interval, Octave},
    performance::{self, Performable, Performance, Player},
    phrases::{PhraseAttribute, TrillOptions},
    pitch::{Pitch, PitchClass},
    rests, Music,
};
