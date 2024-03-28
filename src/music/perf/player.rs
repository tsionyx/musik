//! Defines players structure responsible for interpretation details.
use std::{fmt, sync::Arc};

use crate::{
    music::{phrase::PhraseAttribute, Music},
    prim::duration::Dur,
};

use super::{Context, Duration, Performance, PlayerMap};

pub(super) type NoteFun<P> = Arc<dyn Fn(Context<'_, P>, Dur, &P) -> Performance>;
pub(super) type PhraseFun<P> = Arc<
    dyn Fn(&Music<P>, &PlayerMap<P>, Context<'_, P>, &[PhraseAttribute]) -> (Performance, Duration),
>;
// TODO: producing a properly notated score is not defined yet
pub(super) type NotateFun<P> = std::marker::PhantomData<P>;

/// Defines the way to interpret different parts of [`Music`]
/// to produce the [`Performance`]:
/// - playing individual notes with the [`NoteAttribute`]s;
/// - playing phrases with the [`PhraseAttribute`]s;
/// - notating a musical score in its own unique way (**not implemented**).
pub struct Player<P> {
    pub(super) name: String,
    pub(super) play_note: NoteFun<P>,
    pub(super) interpret_phrase: PhraseFun<P>,
    pub(super) notate_player: NotateFun<P>,
}

impl<P> Clone for Player<P> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            play_note: self.play_note.clone(),
            interpret_phrase: self.interpret_phrase.clone(),
            notate_player: self.notate_player,
        }
    }
}

impl<P> Player<P> {
    /// Get player's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Change the name of the [`Player`].
    pub fn with_name(self, name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..self
        }
    }

    /// Set the interpreter of the individual notes.
    pub fn with_play_note(self, play_note: NoteFun<P>) -> Self {
        Self { play_note, ..self }
    }

    /// Set the interpreter of the phrases.
    pub fn with_interpret_phrase(self, interpret_phrase: PhraseFun<P>) -> Self {
        Self {
            interpret_phrase,
            ..self
        }
    }
}

impl<P> fmt::Debug for Player<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Player {}", self.name)
    }
}
