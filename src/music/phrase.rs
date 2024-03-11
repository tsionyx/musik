use enum_iterator::Sequence;
use enum_map::Enum;
use num_rational::Ratio;

use crate::prim::volume::Volume;

type Rational = Ratio<u32>;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
/// <https://en.wikipedia.org/wiki/Musical_note>
pub enum PhraseAttribute {
    Dyn(Dynamic),
    Tmp(Tempo),
    Art(Articulation),
    Orn(Ornament),
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
/// <https://en.wikipedia.org/wiki/Dynamics_(music)>
pub enum Dynamic {
    Accent(Ratio<u8>),
    Crescendo(Rational),
    Diminuendo(Rational),
    StdLoudness(StdLoudness),
    Loudness(Volume),
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Enum, Sequence)]
pub enum StdLoudness {
    PianoPianissimo,
    Pianissimo,
    Piano,
    MezzoPiano,
    Sforzato,
    MezzoForte,
    Nf,
    Fortissimo,
    ForteFortissimo,
}

impl StdLoudness {
    pub fn get_volume(self) -> Volume {
        let vol: u8 = match self {
            Self::PianoPianissimo => 40,
            Self::Pianissimo => 50,
            Self::Piano => 60,
            Self::MezzoPiano => 70,
            Self::Sforzato => 80,
            Self::MezzoForte => 90,
            Self::Nf => 100,
            Self::Fortissimo => 110,
            Self::ForteFortissimo => 120,
        };
        Volume(vol.try_into().expect("< 127 is low enough"))
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Tempo {
    Ritardando(Rational),
    Accelerando(Rational),
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
/// <https://en.wikipedia.org/wiki/Articulation_(music)>
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

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Ornament {
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
    Instruction(String),
    Head(NoteHead),
    DiatonicTrans(i8),
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum TrillOptions<D> {
    Duration(D),
    Count(u8),
}

impl<D> From<D> for TrillOptions<D> {
    fn from(value: D) -> Self {
        Self::Duration(value)
    }
}

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
