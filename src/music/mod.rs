mod adapters;
mod combinators;
mod constructors;
mod control;
mod iter_like;
mod ops;
pub mod perf;
pub(crate) mod phrase;
mod transform;

use crate::prim::{duration::Dur, pitch::Pitch, volume::Volume};

pub use self::{
    constructors::{rests, A440},
    control::Control,
    iter_like::Temporal,
    perf::NoteAttribute,
};

#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub enum Primitive<P> {
    Note(Dur, P),
    Rest(Dur),
}

pub type PlayerName = String;

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub enum Music<P = Pitch> {
    Prim(Primitive<P>),
    Sequential(Box<Self>, Box<Self>),
    Parallel(Box<Self>, Box<Self>),
    Modify(Control, Box<Self>),
}

impl<P> From<Primitive<P>> for Music<P> {
    fn from(value: Primitive<P>) -> Self {
        Self::Prim(value)
    }
}

impl Music {
    pub fn with_volume(self, vol: Volume) -> Music<(Pitch, Volume)> {
        self.map(|p| (p, vol))
    }
}

pub type AttrNote = (Pitch, Vec<NoteAttribute>);

pub type MusicAttr = Music<AttrNote>;

impl From<Music> for MusicAttr {
    fn from(value: Music) -> Self {
        value.map(|pitch| (pitch, vec![]))
    }
}

impl From<Music<(Pitch, Volume)>> for MusicAttr {
    fn from(value: Music<(Pitch, Volume)>) -> Self {
        value.map(|(pitch, vol)| (pitch, vec![NoteAttribute::Volume(vol)]))
    }
}
