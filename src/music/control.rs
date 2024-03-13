use num_rational::Ratio;

use crate::{
    instruments::InstrumentName,
    prim::{interval::Interval, scale::KeySig},
};

use super::{phrase::PhraseAttribute, Music};

/// Identifier for a player.  // TODO: make it a pointer to the `Player` instance.
pub type PlayerName = String;

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
/// A set of modifiers to change the [`Music`]'s performance.
pub enum Control {
    /// Scale the tempo.
    Tempo(Ratio<u8>),

    /// Transpose all pitches while performing.
    Transpose(Interval),

    /// Perform with a specific instrument.
    Instrument(InstrumentName),

    /// Apply one or more of [phrase attributes][PhraseAttribute].
    Phrase(Vec<PhraseAttribute>),

    /// Set-up the [player][super::perf::Player] which defines
    /// more fine-grained control over the performance details.
    Player(PlayerName),

    /// Specify the key signature for a piece,
    /// which could be useful while interpreting
    /// [phrase attributes][PhraseAttribute].
    KeySig(KeySig),
}

impl<P> Music<P> {
    /// Annotate the [`Music`] with one of [modifiers][Control].
    ///
    /// Also could be used in the form `Music & control`
    pub fn with(self, control: Control) -> Self {
        Self::Modify(control, Box::new(self))
    }

    /// Annotate the [`Music`] to change its tempo while performing:
    /// - accelerate if `tempo` > 1;
    /// - decelerate, otherwise.
    pub fn with_tempo(self, tempo: impl Into<Ratio<u8>>) -> Self {
        self.with(Control::Tempo(tempo.into()))
    }

    /// Annotate the [`Music`] to transpose all its pitches while performing.
    pub fn with_transpose(self, delta: Interval) -> Self {
        self.with(Control::Transpose(delta))
    }

    /// Annotate the [`Music`] to use the given [`InstrumentName`] while performing.
    pub fn with_instrument(self, name: impl Into<InstrumentName>) -> Self {
        self.with(Control::Instrument(name.into()))
    }

    /// Annotate the [`Music`] with a set of [`PhraseAttribute`]s
    /// to extend the performance techniques.
    pub fn with_phrase(self, attributes: Vec<PhraseAttribute>) -> Self {
        self.with(Control::Phrase(attributes))
    }

    /// Specify which [player][super::perf::Player] should be used for performing.
    pub fn with_player(self, name: PlayerName) -> Self {
        self.with(Control::Player(name))
    }

    /// Specify the key signature for a piece,
    /// which could be useful while interpreting
    /// [phrase attributes][Self::with_phrase].
    pub fn with_key_sig(self, key_signature: KeySig) -> Self {
        self.with(Control::KeySig(key_signature))
    }
}
