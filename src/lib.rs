pub mod instruments;
pub mod music;
mod output;
mod prim;

pub use self::{
    music::{
        perf::{self, Performable, Performance, Player},
        phrase::{PhraseAttribute, TrillOptions},
        Music,
    },
    output::midi,
    prim::{
        duration::Dur,
        interval::{ErrorOctaveFromNum, Interval, Octave},
        pitch::{AbsPitch, ErrorPitchClipping, Pitch, PitchClass},
        volume::Volume,
    },
};
