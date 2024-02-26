use super::{duration::Dur, pitch::Pitch};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Note {
    pitch: Pitch,
    dur: Dur,
}
