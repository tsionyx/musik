use std::ops::{Add, BitOr, Div};

use num_rational::Ratio;

use super::instruments::InstrumentName;

pub(crate) mod adapters;
pub(crate) mod duration;
pub(crate) mod interval;
pub mod performance;
pub(crate) mod phrases;
pub(crate) mod pitch;

use self::{
    duration::Dur,
    interval::{Interval, Octave},
    performance::NoteAttribute,
    phrases::PhraseAttribute,
    pitch::{Pitch, PitchClass},
};

#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub enum Primitive<P> {
    Note(Dur, P),
    Rest(Dur),
}

impl<P> Primitive<P> {
    pub fn map<U, F>(self, mut f: F) -> Primitive<U>
    where
        F: FnMut(P) -> U,
    {
        match self {
            Self::Note(d, p) => Primitive::Note(d, f(p)),
            Self::Rest(d) => Primitive::Rest(d),
        }
    }
}

pub type PlayerName = String;

#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub enum KeySig {
    Major(PitchClass),
    Minor(PitchClass),
}

impl Default for KeySig {
    fn default() -> Self {
        // the white piano keys
        Self::Major(PitchClass::C)
    }
}

impl KeySig {
    pub fn get_scale(self) -> impl Iterator<Item = PitchClass> {
        let oc4 = Octave::ONE_LINED;
        let with_octave: Box<dyn Iterator<Item = Pitch>> = match self {
            Self::Major(pc) => Box::new(Pitch::new(pc, oc4).major_scale()),
            Self::Minor(pc) => Box::new(Pitch::new(pc, oc4).natural_minor_scale()),
        };
        with_octave.map(Pitch::class)
    }

    const fn pitch_class(self) -> PitchClass {
        match self {
            Self::Major(pc) | Self::Minor(pc) => pc,
        }
    }

    pub fn get_intervals_scale(self) -> impl Iterator<Item = Interval> {
        let scale = match self {
            Self::Major(_) => Interval::major_scale(),
            Self::Minor(_) => Interval::natural_minor_scale(),
        };
        let tonic = self.pitch_class().into();
        scale.into_iter().scan(tonic, |state, p| {
            *state += p;
            Some(*state)
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub enum Control {
    Tempo(Ratio<u8>), // scale the tempo
    Transpose(Interval),
    Instrument(InstrumentName),
    Phrase(Vec<PhraseAttribute>),
    Player(PlayerName),
    KeySig(KeySig),
}

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

impl<P> Music<P> {
    pub fn map<U, F>(self, f: F) -> Music<U>
    where
        F: FnMut(P) -> U + Clone,
    {
        match self {
            Self::Prim(p) => Music::Prim(p.map(f)),
            Self::Sequential(m1, m2) => m1.map(f.clone()) + m2.map(f),
            Self::Parallel(m1, m2) => m1.map(f.clone()) | m2.map(f),
            Self::Modify(c, m) => m.map(f).with(c),
        }
    }

    pub fn fold<U, Prim, Seq, Par, Mod>(
        self,
        mut prim: Prim,
        mut seq: Seq,
        mut par: Par,
        modify: Mod,
    ) -> U
    where
        Prim: FnMut(Primitive<P>) -> U + Clone,
        Seq: FnMut(U, U) -> U + Clone,
        Par: FnMut(U, U) -> U + Clone,
        Mod: FnMut(Control, U) -> U + Clone,
    {
        match self {
            Self::Prim(p) => prim(p),
            Self::Sequential(m1, m2) => {
                let u1 = m1.fold(prim.clone(), seq.clone(), par.clone(), modify.clone());
                let u2 = m2.fold(prim, seq.clone(), par, modify);
                seq(u1, u2)
            }
            Self::Parallel(m1, m2) => {
                let u1 = m1.fold(prim.clone(), seq.clone(), par.clone(), modify.clone());
                let u2 = m2.fold(prim, seq, par.clone(), modify);
                par(u1, u2)
            }
            Self::Modify(c, m) => modify.clone()(c, m.fold(prim, seq, par, modify)),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Volume(pub u8);

impl Volume {
    pub const fn softest() -> Self {
        Self(0)
    }

    pub const fn loudest() -> Self {
        Self(127)
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

    #[test]
    fn key_sig_c_major_scale() {
        let scale: Vec<_> = KeySig::Major(PitchClass::C).get_scale().collect();
        assert_eq!(
            scale,
            [
                PitchClass::C,
                PitchClass::D,
                PitchClass::E,
                PitchClass::F,
                PitchClass::G,
                PitchClass::A,
                PitchClass::B,
                PitchClass::C,
            ]
        );

        let i_scale: Vec<_> = KeySig::Major(PitchClass::C).get_intervals_scale().collect();
        assert_eq!(
            i_scale,
            [
                Interval::from(0),
                Interval::from(2),
                Interval::from(4),
                Interval::from(5),
                Interval::from(7),
                Interval::from(9),
                Interval::from(11),
                Interval::from(12),
            ]
        );
    }

    #[test]
    fn key_sig_g_major_scale() {
        let scale: Vec<_> = KeySig::Major(PitchClass::G).get_scale().collect();
        assert_eq!(
            scale,
            [
                PitchClass::G,
                PitchClass::A,
                PitchClass::B,
                PitchClass::C,
                PitchClass::D,
                PitchClass::E,
                PitchClass::Fs,
                PitchClass::G,
            ]
        );

        let i_scale: Vec<_> = KeySig::Major(PitchClass::G).get_intervals_scale().collect();
        assert_eq!(
            i_scale,
            [
                Interval::from(7),
                Interval::from(9),
                Interval::from(11),
                Interval::from(12),
                Interval::from(14),
                Interval::from(16),
                Interval::from(18),
                Interval::from(19),
            ]
        );
    }
}
