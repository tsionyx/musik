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
        interval::{Interval, Octave},
        note::Note,
        pitch::{AbsPitch, Pitch, PitchClass},
        volume::Volume,
    },
};
