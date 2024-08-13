//! Musical theory and audio signals concepts expressed in Rust

// `use super::* and Enum::*` in tests
#![cfg_attr(test, allow(clippy::wildcard_imports, clippy::enum_glob_use))]
// using `expect` is almost always better, but `unwrap` still allowed in tests
#![cfg_attr(not(test), warn(clippy::unwrap_used))]

mod instruments;
pub mod music;
mod output;
mod prim;

pub use self::{
    instruments::InstrumentName,
    music::{
        perf::{self, metro, Performable, Performance, Player},
        phrase::{self as attributes, PhraseAttribute},
        Music, NoteAttribute, Temporal,
    },
    output::midi,
    prim::{
        duration::Dur,
        helpers::{self, pitch_class::accidentals},
        interval::{ErrorOctaveTryFromNum, Interval, Octave},
        pitch::{AbsPitch, ErrorPitchClipping, Pitch, PitchClass},
        scale::KeySig,
        volume::Volume,
    },
};
