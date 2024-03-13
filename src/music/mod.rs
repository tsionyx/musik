//! The module defines central notion of [`Music`]
//! which is the high-level representation of music
//! kinda musical score. In fact, it resembles
//! most of the score's functionality but rather
//! representing the music with declarative syntax,
//! instead of fancy musical symbols.
//!
//! Also, a number of high-level abstractions are defined
//! to reduce the burden of repetitions.
mod combinators;
mod constructors;
mod control;
mod iter_like;
mod r#macro;
mod ops;
mod ornaments;
pub mod perf;
pub mod phrase;
mod transform;

use crate::prim::{duration::Dur, pitch::Pitch, volume::Volume};

pub use self::{
    constructors::{rests, A440},
    control::Control,
    iter_like::Temporal,
    perf::NoteAttribute,
};

#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
/// 'Atomic' musical values.
pub enum Primitive<P> {
    /// The note key and its [value](https://en.wikipedia.org/wiki/Note_value).
    Note(Dur, P),

    /// Absence of sound for a defined period of time.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Rest_(music)>
    Rest(Dur),
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
/// High-level representation of music.
pub enum Music<P = Pitch> {
    /// Single atomic building block of music,
    /// usually a [note][Primitive::Note] or a [rest][Primitive::Rest].
    Prim(Primitive<P>),

    /// Sequentially composed two pieces.
    /// Could be combined to create arbitrarily
    /// long series resembling a complex linked list.
    Sequential(Box<Self>, Box<Self>),

    /// The polyphonic composition of two parts
    /// which should be played simultaneously.
    /// Allows to play different lines for different
    /// melodies and/or instruments.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Polyphony>
    Parallel(Box<Self>, Box<Self>),

    /// Annotate the music with one of [modifiers][Control].
    Modify(Control, Box<Self>),
}

impl<P> From<Primitive<P>> for Music<P> {
    fn from(value: Primitive<P>) -> Self {
        Self::Prim(value)
    }
}

impl Music {
    /// Assign [`Volume`] to every note of [`Music`].
    pub fn with_volume(self, vol: Volume) -> Music<(Pitch, Volume)> {
        self.map(|p| (p, vol))
    }
}

/// Pitch with Attributes
pub type AttrNote = (Pitch, Vec<NoteAttribute>);

/// Music with [attributed pitches][AttrNote].
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
