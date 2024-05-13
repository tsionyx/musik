use num_rational::Ratio;

use crate::{
    instruments::InstrumentName,
    prim::{interval::Interval, scale::KeySig},
};

use super::{
    combinators::MapToOther,
    perf::{DynPlayer, Player},
    phrase::PhraseAttribute,
    Music,
};

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
/// A set of modifiers to change the [`Music`]'s performance.
pub enum Control<P: 'static> {
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
    Player(DynPlayer<P>),

    /// Specify the key signature for a piece,
    /// which could be useful while interpreting
    /// [phrase attributes][PhraseAttribute].
    KeySig(KeySig),
}

impl<P> Music<P> {
    /// Annotate the [`Music`] with one of [modifiers][Control].
    ///
    /// Also could be used in the form `Music & control`
    pub fn with(self, control: Control<P>) -> Self {
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
    pub fn with_player<Pl>(self, player: Pl) -> Self
    where
        Pl: Player<P> + 'static,
    {
        self.with(Control::Player(DynPlayer::from_player(player)))
    }

    /// Specify the key signature for a piece,
    /// which could be useful while interpreting
    /// [phrase attributes][Self::with_phrase].
    pub fn with_key_sig(self, key_signature: KeySig) -> Self {
        self.with(Control::KeySig(key_signature))
    }
}

impl<T, U> MapToOther<Control<U>> for Control<T>
where
    DynPlayer<T>: MapToOther<DynPlayer<U>>,
{
    fn into_other(self) -> Option<Control<U>> {
        match self {
            Self::Tempo(x) => Some(Control::Tempo(x)),
            Self::Transpose(x) => Some(Control::Transpose(x)),
            Self::Instrument(x) => Some(Control::Instrument(x)),
            Self::Phrase(x) => Some(Control::Phrase(x)),
            Self::Player(x) => x.into_other().map(Control::Player),
            Self::KeySig(x) => Some(Control::KeySig(x)),
        }
    }
}
