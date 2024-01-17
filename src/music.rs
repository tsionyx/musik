use std::{
    convert::TryFrom,
    ops::{Add, AddAssign, BitAnd, BitOr, Sub},
};

use num_rational::Ratio;

use super::instruments::StandartMidiInstrument;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// 0..8 on piano
pub struct Octave(i8);

impl Octave {
    pub const OCTO_CONTRA: Self = Self(-1);
    pub const SUB_CONTRA: Self = Self(0);
    pub const CONTRA: Self = Self(1);
    pub const GREAT: Self = Self(2);
    pub const SMALL: Self = Self(3);
    pub const ONE_LINED: Self = Self(4);
    pub const TWO_LINED: Self = Self(5);
    pub const THREE_LINED: Self = Self(6);
    pub const FOUR_LINED: Self = Self(7);
    pub const FIVE_LINED: Self = Self(8);
    pub const SIX_LINED: Self = Self(9);
    pub const SEVEN_LINED: Self = Self(10);
}

impl From<i8> for Octave {
    fn from(val: i8) -> Self {
        Self(val)
    }
}

#[derive(Debug, Clone, Copy, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct Interval(i8);

impl Interval {
    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn semi_tone() -> Self {
        Self(1)
    }

    pub const fn tone() -> Self {
        Self(2)
    }
}

impl From<i8> for Interval {
    fn from(val: i8) -> Self {
        Self(val)
    }
}

impl Add for Interval {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Interval {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PitchClass {
    // https://en.wikipedia.org/wiki/Enharmonic
    Bs , C , Dff,
    Bss, Cs, Df,
    Css, D , Eff,
    Ds , Ef, Fff,
    Dss, E , Ff,
    Es , F , Gff,
    Ess, Fs, Gf,
    Fss, G , Aff,
    Gs , Af,
    Gss, A , Bff,
    As , Bf, Cff,
    Ass, B , Cf,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AbsPitch(i8);

impl AbsPitch {
    pub const fn get_inner(self) -> i8 {
        self.0
    }
}

impl From<i8> for AbsPitch {
    fn from(x: i8) -> Self {
        Self(x)
    }
}

impl From<AbsPitch> for Interval {
    fn from(abs_pitch: AbsPitch) -> Self {
        Self(abs_pitch.0)
    }
}

impl From<AbsPitch> for (Octave, u8) {
    fn from(abs_pitch: AbsPitch) -> Self {
        let octave_size = Octave::semitones_number().0;
        let (octave, n) = (abs_pitch.0 / octave_size, abs_pitch.0 % octave_size);

        let (octave, n) = if n < 0 {
            (octave - 1, (n + octave_size))
        } else {
            (octave, n)
        };

        (
            Octave(octave),
            u8::try_from(n).expect("Negative interval found"),
        )
    }
}

impl From<Octave> for AbsPitch {
    fn from(octave: Octave) -> Self {
        let octave_size = Octave::semitones_number().0;
        Self(octave.0 * octave_size)
    }
}

impl Add<Interval> for AbsPitch {
    type Output = Self;

    fn add(self, rhs: Interval) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for AbsPitch {
    type Output = Interval;

    fn sub(self, rhs: Self) -> Self::Output {
        Interval(self.0 - rhs.0)
    }
}

impl From<Interval> for AbsPitch {
    fn from(int: Interval) -> Self {
        Self(int.0)
    }
}

impl From<PitchClass> for Interval {
    fn from(pc: PitchClass) -> Self {
        let val = match pc {
            PitchClass::Cff => -2,
            PitchClass::Cf => -1,
            PitchClass::C => 0,
            PitchClass::Cs => 1,
            PitchClass::Css => 2,
            PitchClass::Dff => 0,
            PitchClass::Df => 1,
            PitchClass::D => 2,
            PitchClass::Ds => 3,
            PitchClass::Dss => 4,
            PitchClass::Eff => 2,
            PitchClass::Ef => 3,
            PitchClass::E => 4,
            PitchClass::Es => 5,
            PitchClass::Ess => 6,
            PitchClass::Fff => 3,
            PitchClass::Ff => 4,
            PitchClass::F => 5,
            PitchClass::Fs => 6,
            PitchClass::Fss => 7,
            PitchClass::Gff => 5,
            PitchClass::Gf => 6,
            PitchClass::G => 7,
            PitchClass::Gs => 8,
            PitchClass::Gss => 9,
            PitchClass::Aff => 7,
            PitchClass::Af => 8,
            PitchClass::A => 9,
            PitchClass::As => 10,
            PitchClass::Ass => 11,
            PitchClass::Bff => 9,
            PitchClass::Bf => 10,
            PitchClass::B => 11,
            PitchClass::Bs => 12,
            PitchClass::Bss => 13,
        };

        Self(val)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pitch {
    class: PitchClass,
    octave: Octave,
}

macro_rules! def_pitch_constructor {
    ($pitch: ident) => {
        #[allow(non_snake_case)]
        pub const fn $pitch(octave: Octave) -> Self {
            Self::new(PitchClass::$pitch, octave)
        }
    };


    ( $( $pitch: ident ),+ $(,)? ) => {
        $(
            def_pitch_constructor!($pitch);
        )+
    }
}

impl Pitch {
    pub const fn new(class: PitchClass, octave: Octave) -> Self {
        Self { class, octave }
    }

    def_pitch_constructor![Aff, Af, A, As, Ass];
    def_pitch_constructor![Bff, Bf, B, Bs, Bss];
    def_pitch_constructor![Cff, Cf, C, Cs, Css];
    def_pitch_constructor![Dff, Df, D, Ds, Dss];
    def_pitch_constructor![Eff, Ef, E, Es, Ess];
    def_pitch_constructor![Fff, Ff, F, Fs, Fss];
    def_pitch_constructor![Gff, Gf, G, Gs, Gss];

    pub fn abs(self) -> AbsPitch {
        AbsPitch::from(self.octave) + Interval::from(self.class)
    }

    pub const CONCERT_A_FREQUENCY: f64 = 440.0;

    pub fn get_frequency(self) -> f64 {
        let a4 = Self::A(Octave::ONE_LINED);
        let interval_to_a4 = self.abs() - a4.abs();
        let octaves_from_a4 = f64::from(interval_to_a4.0) / f64::from(Octave::semitones_number().0);
        octaves_from_a4.exp2() * Self::CONCERT_A_FREQUENCY
    }
}

impl Octave {
    const MINIMAL_PITCHES: [PitchClass; 12] = [
        PitchClass::C,
        PitchClass::Cs,
        PitchClass::D,
        PitchClass::Ds,
        PitchClass::E,
        PitchClass::F,
        PitchClass::Fs,
        PitchClass::G,
        PitchClass::Gs,
        PitchClass::A,
        PitchClass::As,
        PitchClass::B,
    ];

    fn semitones_number() -> Interval {
        let len = i8::try_from(Self::MINIMAL_PITCHES.len()).unwrap();
        Interval(len)
    }
}

impl Pitch {
    pub fn trans(self, delta: Interval) -> Self {
        let abs = self.abs() + delta;
        Self::from(abs)
    }

    pub fn next(self) -> Self {
        self.trans(Interval::semi_tone())
    }

    pub fn prev(self) -> Self {
        self.trans(Interval::from(-1))
    }

    pub fn get_scale<I>(self, intervals: &[I]) -> impl Iterator<Item = Self> + '_
    where
        I: Copy + Into<Interval>,
    {
        intervals
            .iter()
            .scan(Interval::zero(), |tonic_distance, &interval| {
                *tonic_distance += interval.into();
                Some(*tonic_distance)
            })
            .map(move |distance| self.trans(distance))
    }

    pub fn major_scale(self) -> impl Iterator<Item = Self> {
        self.get_scale(&[0, 2, 2, 1, 2, 2, 2, 1])
    }

    pub fn natural_minor_scale(self) -> impl Iterator<Item = Self> {
        self.get_scale(&[0, 2, 1, 2, 2, 1, 2, 2])
    }
}

impl From<AbsPitch> for Pitch {
    fn from(abs_pitch: AbsPitch) -> Self {
        let (octave, interval) = abs_pitch.into();
        let n = usize::from(interval);

        Self {
            class: Octave::MINIMAL_PITCHES[n],
            octave,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dur(u8, u8);

impl Dur {
    const fn from_integer(i: u8) -> Self {
        Self(i, 1)
    }

    const fn new(num: u8, denom: u8) -> Self {
        Self(num, denom)
    }

    pub fn into_rational(self) -> Ratio<u8> {
        Ratio::new(self.0, self.1)
    }

    pub const ZERO: Self = Self::from_integer(0);
    pub const BN: Self = Self::from_integer(2); // brevis note
    pub const WN: Self = Self::from_integer(1); // whole note
    pub const HN: Self = Self::new(1, 2); // half note
    pub const QN: Self = Self::new(1, 4); // quarter note
    pub const EN: Self = Self::new(1, 8); // eighth note
    pub const SN: Self = Self::new(1, 16); // sixteenth note
    pub const TN: Self = Self::new(1, 32); // thirty-second note
    pub const SFN: Self = Self::new(1, 64); // sixty-fourth note

    pub const DWN: Self = Self::new(3, 2); // dotted whole note
    pub const DHN: Self = Self::new(3, 4); // dotted half note
    pub const DQN: Self = Self::new(3, 8); // dotted quarter note
    pub const DEN: Self = Self::new(3, 16); // dotted eighth note
    pub const DSN: Self = Self::new(3, 32); // dotted sixteenth note
    pub const DTN: Self = Self::new(3, 64); // dotted thirty-second note

    pub const DDHN: Self = Self::new(7, 8); // double-dotted half note
    pub const DDQN: Self = Self::new(7, 16); // double-dotted quarter note
    pub const DDEN: Self = Self::new(7, 32); // double-dotted eighth note

    pub const fn double(self) -> Self {
        if self.1 & 1 == 0 {
            //&Self::WN => Self::BN,
            //&Self::HN => Self::WN,
            //&Self::QN => Self::HN,
            //&Self::EN => Self::QN,
            //&Self::SN => Self::EN,
            //&Self::TN => Self::SN,
            //&Self::SFN => Self::TN,
            //
            //&Self::DHN => Self::DWN,
            //&Self::DQN => Self::DHN,
            //&Self::DEN => Self::DQN,
            //&Self::DSN => Self::DEN,
            //&Self::DTN => Self::DSN,
            //
            //&Self::DDQN => Self::DDHN,
            //&Self::DDEN => Self::DDQN,

            Self::new(self.0, self.1 >> 1)
        } else {
            Self::new(self.0 << 1, self.1)
        }
    }

    pub const fn halve(self) -> Self {
        if self.0 & 1 == 0 {
            Self::new(self.0 >> 1, self.1)
        } else {
            Self::new(self.0, self.1 << 1)
        }
    }
}

impl From<Ratio<u8>> for Dur {
    fn from(value: Ratio<u8>) -> Self {
        Self::new(*value.numer(), *value.denom())
    }
}

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
impl<P> BitAnd for Music<P> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
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
}

impl Music {
    def_note_constructor![Aff, Af, A, As, Ass];
    def_note_constructor![Bff, Bf, B, Bs, Bss];
    def_note_constructor![Cff, Cf, C, Cs, Css];
    def_note_constructor![Dff, Df, D, Ds, Dss];
    def_note_constructor![Eff, Ef, E, Es, Ess];
    def_note_constructor![Fff, Ff, F, Fs, Fss];
    def_note_constructor![Gff, Gf, G, Gs, Gss];

    /// Exercise 2.5
    ///
    /// In contrast to the annotation of the `Music` with `Transpose`
    /// (as part of the `Control` data type, which in turn is part of the `Music`),
    /// this function actually changes each note in a `Music<Pitch>` value by
    /// transposing it by the interval specified.
    pub fn trans(self, delta: Interval) -> Self {
        match self {
            Self::Prim(Primitive::Note(duration, pitch)) => {
                Self::note(duration, pitch.trans(delta))
            }
            Self::Prim(Primitive::Rest(duration)) => Self::rest(duration),
            Self::Sequential(m1, m2) => m1.trans(delta) & m2.trans(delta),
            Self::Parallel(m1, m2) => m1.trans(delta) | m2.trans(delta),
            Self::Modify(control, m) => Self::Modify(control, Box::new(m.trans(delta))),
        }
    }

    pub fn grace_note(&self, offset: AbsPitch, grace_fraction: Ratio<u8>) -> Result<Self, String> {
        if let Self::Prim(Primitive::Note(d, p)) = self {
            Ok(Self::note(
                (grace_fraction * d.into_rational()).into(),
                p.trans(offset.into()),
            ) & Self::note(
                ((Ratio::from_integer(1) - grace_fraction) * d.into_rational()).into(),
                *p,
            ))
        } else {
            Err("Can only add a grace note to a note".into())
        }
    }

    pub fn invert(self) -> Self {
        let line = Vec::from(self.clone());
        if let Some(Self::Prim(Primitive::Note(_, first_pitch))) = line.first() {
            let first_pitch = *first_pitch;
            let inv = |m| {
                if let Self::Prim(Primitive::Note(d, p)) = m {
                    // prevent i8 overflow
                    let inverted_pitch = 2 * i16::from(first_pitch.abs().get_inner())
                        - i16::from(p.abs().get_inner());
                    let inverted_pitch = AbsPitch::from(inverted_pitch as i8);
                    Self::note(d, inverted_pitch.into())
                } else {
                    m
                }
            };
            Self::line(line.into_iter().map(inv).collect())
        } else {
            self
        }
    }

    pub fn retro_invert(self) -> Self {
        self.invert().retrograde()
    }

    pub fn invert_retro(self) -> Self {
        self.retrograde().invert()
    }
}

impl<P> Music<P> {
    pub fn line(musics: Vec<Self>) -> Self {
        musics
            .into_iter()
            .fold(Self::rest(Dur::ZERO), |acc, m| acc & m)
    }

    pub fn chord(musics: Vec<Self>) -> Self {
        musics
            .into_iter()
            .fold(Self::rest(Dur::ZERO), |acc, m| acc | m)
    }

    pub fn with_dur(pitches: Vec<P>, dur: Dur) -> Self {
        Self::line(
            pitches
                .into_iter()
                .map(|pitch| Self::note(dur, pitch))
                .collect(),
        )
    }

    pub fn with_delay(self, dur: Dur) -> Self {
        Self::rest(dur) & self
    }

    pub fn retrograde(self) -> Self {
        Self::line(Vec::from(self).into_iter().rev().collect())
    }
}

impl<P: Clone> Music<P> {
    pub fn times(&self, n: usize) -> Self {
        std::iter::repeat(self.clone())
            .take(n)
            .fold(Self::rest(Dur::ZERO), |acc, m| acc & m)
    }
}

impl<P> From<Music<P>> for Vec<Music<P>> {
    fn from(value: Music<P>) -> Self {
        match value {
            Music::Prim(Primitive::Rest(Dur::ZERO)) => vec![],
            Music::Sequential(m1, m2) => {
                Self::from(*m1).into_iter().chain(Self::from(*m2)).collect()
            }
            other => vec![other],
        }
    }
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

#[allow(dead_code)]
const A440: Music = Music::A(Octave(4), Dur::WN);

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
