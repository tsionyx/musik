use num_rational::Ratio;

use crate::{
    instruments::InstrumentName,
    prim::{interval::Interval, scale::KeySig},
};

use super::{phrase::PhraseAttribute, Music, PlayerName};

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub enum Control {
    Tempo(Ratio<u8>), // scale the tempo
    Transpose(Interval),
    Instrument(InstrumentName),
    Phrase(Vec<PhraseAttribute>),
    Player(PlayerName),
    KeySig(KeySig),
}

impl<P> Music<P> {
    // TODO: make & operation
    pub fn with(self, control: Control) -> Self {
        Self::Modify(control, Box::new(self))
    }

    pub fn with_tempo(self, tempo: impl Into<Ratio<u8>>) -> Self {
        self.with(Control::Tempo(tempo.into()))
    }

    pub fn with_transpose(self, delta: Interval) -> Self {
        self.with(Control::Transpose(delta))
    }

    pub fn with_instrument(self, name: impl Into<InstrumentName>) -> Self {
        self.with(Control::Instrument(name.into()))
    }

    pub fn with_phrase(self, attributes: Vec<PhraseAttribute>) -> Self {
        self.with(Control::Phrase(attributes))
    }

    pub fn with_player(self, name: PlayerName) -> Self {
        self.with(Control::Player(name))
    }

    pub fn with_key_sig(self, key_signature: KeySig) -> Self {
        self.with(Control::KeySig(key_signature))
    }
}
