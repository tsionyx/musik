pub mod instruments;
mod io;
pub mod music;

pub use self::{
    io::midi,
    music::{
        duration::Dur,
        interval::{AbsPitch, Interval, Octave},
        performance::{self, Performable, Performance, Player},
        phrases::{PhraseAttribute, TrillOptions},
        pitch::{Pitch, PitchClass},
        rests, Music,
    },
};
