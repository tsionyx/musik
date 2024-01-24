use std::ops::{Add, BitOr, Div};

use num_rational::Ratio;

use super::instruments::StandartMidiInstrument;

mod adapters;
pub(crate) mod duration;
pub(crate) mod interval;
pub(crate) mod pitch;

use self::{
    duration::Dur,
    interval::{AbsPitch, Octave},
    pitch::{Pitch, PitchClass},
};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Primitive<P> {
    Note(Dur, P),
    Rest(Dur),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PlayerName(String);

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Mode {
    Major,
    Minor,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Control {
    Tempo(Ratio<u8>), // scale the tempo
    Transpose(AbsPitch),
    Instrument(StandartMidiInstrument),
    //TODO: Phrase(Vec<PhraseAttribute>),
    Player(PlayerName),
    KeySig(PitchClass, Mode), // key signature and mode
}

#[derive(Debug, PartialEq, Eq, Clone)]
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

/// Sequential composition
impl<P> Add for Music<P> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Sequential(Box::new(self), Box::new(rhs))
    }
}

/// Parallel composition
impl<P> BitOr for Music<P> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::Parallel(Box::new(self), Box::new(rhs))
    }
}

/// Truncating parallel composition
impl<P> Div for Music<P> {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, rhs: Self) -> Self::Output {
        let d1 = self.duration();
        let d2 = rhs.duration();
        self.take(d2) | rhs.take(d1)
    }
}

macro_rules! def_note_constructor {
    ($pitch: ident) => {
        #[allow(non_snake_case)]
        pub const fn $pitch(octave: Octave, duration: Dur) -> Self {
            Self::note(duration, Pitch::$pitch(octave))
        }
    };

    ( $( $pitch: ident ),+ $(,)? ) => {
        $(
            def_note_constructor!($pitch);
        )+
    }
}

impl<P> Music<P> {
    pub const fn note(duration: Dur, pitch: P) -> Self {
        Self::Prim(Primitive::Note(duration, pitch))
    }

    pub const fn rest(duration: Dur) -> Self {
        Self::Prim(Primitive::Rest(duration))
    }

    pub fn with_tempo(self, tempo: Ratio<u8>) -> Self {
        Self::Modify(Control::Tempo(tempo), Box::new(self))
    }

    pub fn with_transpose(self, abs_pitch: AbsPitch) -> Self {
        Self::Modify(Control::Transpose(abs_pitch), Box::new(self))
    }

    pub fn with_instrument(self, name: StandartMidiInstrument) -> Self {
        Self::Modify(Control::Instrument(name), Box::new(self))
    }

    //fn with_phrase(self, attributes: Vec<PhraseAttribute>) -> Self {
    //    Music::Modify(Control::Phrase(attributes), Box::new(self))
    //}

    pub fn with_player(self, name: PlayerName) -> Self {
        Self::Modify(Control::Player(name), Box::new(self))
    }

    pub fn with_key_sig(self, pitch_class: PitchClass, mode: Mode) -> Self {
        Self::Modify(Control::KeySig(pitch_class, mode), Box::new(self))
    }

    pub fn duration(&self) -> Dur {
        match self {
            Self::Prim(Primitive::Note(d, _)) => *d,
            Self::Prim(Primitive::Rest(d)) => *d,
            Self::Sequential(m1, m2) => m1.duration() + m2.duration(),
            Self::Parallel(m1, m2) => m1.duration().max(m2.duration()),
            Self::Modify(Control::Tempo(r), m) => m.duration() / *r,
            Self::Modify(_, m) => m.duration(),
        }
    }
}

impl Music {
    def_note_constructor![Aff, Af, A, As, Ass];
    def_note_constructor![Bff, Bf, B, Bs, Bss];
    def_note_constructor![Cff, Cf, C, Cs, Css];
    def_note_constructor![Dff, Df, D, Ds, Dss];
    def_note_constructor![Eff, Ef, E, Es, Ess];
    def_note_constructor![Fff, Ff, F, Fs, Fss];
    def_note_constructor![Gff, Gf, G, Gs, Gss];
}

pub mod rests {
    use super::{Dur, Music};

    macro_rules! def_rest {
        ($rest_name: ident) => {
            pub const $rest_name: Music = Music::rest(Dur::$rest_name);
        };
    }

    def_rest!(BN);
    def_rest!(WN);
    def_rest!(HN);
    def_rest!(QN);
    def_rest!(EN);
    def_rest!(SN);
    def_rest!(TN);
    def_rest!(SFN);
    def_rest!(DWN);
    def_rest!(DHN);
    def_rest!(DQN);
    def_rest!(DEN);
    def_rest!(DSN);
    def_rest!(DTN);
    def_rest!(DDHN);
    def_rest!(DDQN);
    def_rest!(DDEN);
}

pub const A440: Music = Music::A(Octave(4), Dur::WN);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn double_duration() {
        assert_eq!(Dur::WN.double(), Dur::BN);
        assert_eq!(Dur::HN.double(), Dur::WN);
        assert_eq!(Dur::QN.double(), Dur::HN);
        assert_eq!(Dur::EN.double(), Dur::QN);
        assert_eq!(Dur::SN.double(), Dur::EN);
        assert_eq!(Dur::TN.double(), Dur::SN);
        assert_eq!(Dur::SFN.double(), Dur::TN);

        assert_eq!(Dur::DHN.double(), Dur::DWN);
        assert_eq!(Dur::DQN.double(), Dur::DHN);
        assert_eq!(Dur::DEN.double(), Dur::DQN);
        assert_eq!(Dur::DSN.double(), Dur::DEN);
        assert_eq!(Dur::DTN.double(), Dur::DSN);

        assert_eq!(Dur::DDQN.double(), Dur::DDHN);
        assert_eq!(Dur::DDEN.double(), Dur::DDQN);
    }

    #[test]
    fn major() {
        let oc3 = Octave(3);
        let middle_c = Pitch::C(oc3);
        let major: Vec<_> = middle_c.major_scale().collect();

        assert_eq!(
            major,
            vec![
                Pitch::C(oc3),
                Pitch::D(oc3),
                Pitch::E(oc3),
                Pitch::F(oc3),
                Pitch::G(oc3),
                Pitch::A(oc3),
                Pitch::B(oc3),
                Pitch::C(Octave(4)),
            ]
        );
    }

    #[test]
    fn minor() {
        let oc4 = Octave(4);
        let oc5 = Octave(5);

        let concert_a = Pitch::A(oc4);
        let minor: Vec<_> = concert_a.natural_minor_scale().collect();

        assert_eq!(
            minor,
            vec![
                Pitch::A(oc4),
                Pitch::B(oc4),
                Pitch::C(oc5),
                Pitch::D(oc5),
                Pitch::E(oc5),
                Pitch::F(oc5),
                Pitch::G(oc5),
                Pitch::A(oc5),
            ]
        );
    }

    #[test]
    fn get_a440_freq() {
        let pitch = Pitch::A(Octave::ONE_LINED);
        assert!((pitch.get_frequency() - 440.0).abs() < f64::EPSILON);
    }

    fn assert_is_close_freq(f1: f64, f2: f64) {
        assert!(
            (f1 - f2).abs() < 0.001,
            "The {} and {} are definitely not the same frequencies",
            f1,
            f2
        );
    }

    #[test]
    fn get_middle_c_freq() {
        let pitch = Pitch::C(Octave::ONE_LINED);
        assert_is_close_freq(pitch.get_frequency(), 261.626);
    }

    #[test]
    fn get_1_herz_freq() {
        let pitch = Pitch::C(Octave(-4));
        assert_is_close_freq(pitch.get_frequency(), 1.022);
    }
}
