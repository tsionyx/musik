//! Additional attribute for a [`Music`][super::Music] to extend
//! the expressive power for composers and performers.
//!
//! See more: <https://en.wikipedia.org/wiki/Musical_phrasing>

use enum_iterator::Sequence;
use enum_map::Enum;
use num_rational::Ratio;

use crate::prim::volume::Volume;

type Rational = Ratio<u32>;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
/// A number of characteristics to shape
/// the various aspects of the musical phrase.
pub enum PhraseAttribute {
    /// How loud to play.
    Dyn(Dynamic),

    /// Gradual tempo change.
    Tmp(Tempo),

    /// Single note performance:
    /// - its length;
    /// - the snape of attack and decay;
    Art(Articulation),

    /// Additional notes not essential
    /// to the main melodic line.
    Orn(Ornament),
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
/// Indications of how loud to play.
///
/// See more: <https://en.wikipedia.org/wiki/Dynamics_(music)>
pub enum Dynamic {
    /// Stronger attack placed on a particular note.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Accent_(music)>
    Accent(Ratio<u8>),
    /// Gradually increasing volume.
    Crescendo(Rational),
    /// Gradually decreasing volume.
    Diminuendo(Rational),
    /// Choose from one of the standard Volume presets.
    StdLoudness(StdLoudness),
    /// Explicitly specify [`Volume`].
    Loudness(Volume),
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Enum, Sequence)]
/// Standard Volume presets.
///
/// See more: <https://en.wikipedia.org/wiki/Dynamics_(music)#Dynamic_markings>
pub enum StdLoudness {
    /// Very very quiet.
    PianoPianissimo,
    /// Very quiet.
    Pianissimo,
    /// Quiet
    Piano,
    /// Moderately quiet.
    MezzoPiano,
    /// Suddenly forceful. // TODO: ?
    Sforzato,
    /// Moderately loud.
    MezzoForte,
    /// Loud.
    Forte,
    /// Very loud.
    Fortissimo,
    /// Very very loud.
    ForteFortissimo,
}

impl StdLoudness {
    /// Get the numeric [`Volume`]
    /// from standard names using one of the predefined scales.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Dynamics_(music)#Interpretation_by_notation_programs>
    pub fn get_volume(self) -> Volume {
        let vol: u8 = match self {
            Self::PianoPianissimo => 40,
            Self::Pianissimo => 50,
            Self::Piano => 60,
            Self::MezzoPiano => 70,
            Self::Sforzato => 80,
            Self::MezzoForte => 90,
            Self::Forte => 100,
            Self::Fortissimo => 110,
            Self::ForteFortissimo => 120,
        };
        Volume(vol.try_into().expect("< 127 is low enough"))
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
/// Indicate the gradual change in tempo.
///
/// See more: <https://en.wikipedia.org/wiki/Tempo#Variation_through_a_piece>
pub enum Tempo {
    /// Gradually speeding up the tempo, opposite of [`Self::Ritardando`].
    Accelerando(Rational),
    /// Slowing down gradually, opposite of [`Self::Accelerando`].
    Ritardando(Rational),
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
/// Articulation is a musical parameter that determines how a single note
/// or other discrete event is sounded. Articulations primarily structure
/// an event's start and end, determining the length of its sound
/// and the shape of its attack and decay.
///
/// See more: <https://en.wikipedia.org/wiki/Articulation_(music)>
pub enum Articulation {
    Staccato(Rational),
    Legato(Rational),
    Slurred(Rational),
    Tenuto,
    Marcato,
    Pedal,
    Fermata,
    FermataDown,
    Breath,
    DownBow,
    UpBow,
    Harmonic,
    Pizzicato,
    LeftPizz,
    BartokPizz,
    Swell,
    Wedge,
    Thumb,
    Stopped,
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
/// [`Ornament`] is typically added notes
/// that are not essential to the main melody
/// but decorates the phrase.
///
/// See more: <https://en.wikipedia.org/wiki/Ornament_(music)>
pub enum Ornament {
    /// See more: <https://en.wikipedia.org/wiki/Trill_(music)>
    Trill(TrillOptions<Ratio<u32>>),
    Mordent,
    InvMordent,
    DoubleMordent,
    Turn,
    TrilledTurn,
    ShortTrill,
    Arpeggio,
    ArpeggioUp,
    ArpeggioDown,
    // TODO: it was in the original HSoM. What is it about?
    // Instruction(String),
    Head(NoteHead),
    DiatonicTrans(i8),
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
/// Defines performance parameter for a [`Trill`][Ornament::Trill].
pub enum TrillOptions<D> {
    /// How long should last every single trilled note.
    Duration(D),
    /// How many trilled notes will be in ornament.
    Count(u8),
}

impl<D> From<D> for TrillOptions<D> {
    fn from(value: D) -> Self {
        Self::Duration(value)
    }
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum NoteHead {
    DiamondHead,
    SquareHead,
    XHead,
    TriangleHead,
    TremoloHead,
    SlashHead,
    ArtHarmonic,
    NoHead,
}
