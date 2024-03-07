use std::{
    ops::{Add, Sub},
    str::FromStr,
};

use enum_iterator::Sequence;
use enum_map::Enum;
use ux2::u7;

use super::interval::{Interval, Octave};

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Enum, Sequence)]
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

macro_rules! match_str_to_pitch_class {
    ($test_var:ident: $($pc:ident),+ $(,)? ; otherwise $capture:ident => $other:expr) => {
        match $test_var {
            $(stringify!($pc) => Ok(Self::$pc),)+
            $capture => Err($other)
        }
    };
}

impl FromStr for PitchClass {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match_str_to_pitch_class!(s:
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
        Ass, B , Cf;
        otherwise other => format!(
            "{:?} is not a valid {}",
            other,
            std::any::type_name::<Self>()
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

    pub const fn octave(self) -> Octave {
        self.octave
    }

    pub const fn class(self) -> PitchClass {
        self.class
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
        let a4 = Self::A(Octave::OneLined);
        let interval_to_a4 = self.abs() - a4.abs();
        let octaves_from_a4 =
            f64::from(interval_to_a4.get_inner()) / f64::from(u8::from(Octave::semitones_number()));
        octaves_from_a4.exp2() * Self::CONCERT_A_FREQUENCY
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AbsPitch(pub(crate) u7);

impl AbsPitch {
    pub const fn get_inner(self) -> u7 {
        self.0
    }

    pub fn get_u8(self) -> u8 {
        u8::from(self.get_inner())
    }
}

impl From<u7> for AbsPitch {
    fn from(x: u7) -> Self {
        Self(x)
    }
}

impl From<AbsPitch> for (Octave, ux2::u4) {
    fn from(abs_pitch: AbsPitch) -> Self {
        let octave_size = Octave::semitones_number();
        let (octave, n) = (
            abs_pitch.0 / u7::from(octave_size),
            abs_pitch.0 % octave_size,
        );
        let octave = u8::try_from(octave).expect("u8 / 12 is low enough for i8");

        // TODO: make roundtrip tests for every value in 0..127
        (
            Octave::from_i8(octave as i8 - 1).expect("Abs pitch conversion is always valid"),
            n,
        )
    }
}

impl From<Octave> for AbsPitch {
    fn from(octave: Octave) -> Self {
        let octave_size = Octave::semitones_number();
        let octave = u8::try_from(octave as isize + 1).expect("Invalid octave");
        let val = octave * u8::from(octave_size);
        Self(u7::try_from(val).unwrap())
    }
}

impl Add<Interval> for AbsPitch {
    type Output = Self;

    fn add(self, rhs: Interval) -> Self::Output {
        let a = i8::try_from(u8::from(self.0)).expect("u7 should convert to i8 seamlessly");
        let b = rhs.0;
        if let Ok(val) = u8::try_from(a + b) {
            let val = u7::try_from(val).expect("i8 > 0 should convert to u7 seamlessly");
            Self(val)
        } else {
            // TODO: Err
            Self(u7::MIN) // saturating sub
        }
    }
}

impl Sub for AbsPitch {
    type Output = Interval;

    fn sub(self, rhs: Self) -> Self::Output {
        let a = i8::try_from(u8::from(self.0)).expect("u7 should convert to i8 seamlessly");
        let b = i8::try_from(u8::from(rhs.0)).expect("u7 should convert to i8 seamlessly");
        Interval::from(a - b)
    }
}

impl Sub<Interval> for AbsPitch {
    type Output = Self;

    fn sub(self, rhs: Interval) -> Self::Output {
        self + (-rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_a440_freq() {
        let pitch = Pitch::A(Octave::OneLined);
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
        let pitch = Pitch::C(Octave::OneLined);
        assert_eq!(pitch.abs().0, u7::new(60));
        assert_eq!(Pitch::from(AbsPitch(u7::new(60))), pitch);
        assert_is_close_freq(pitch.get_frequency(), 261.626);
    }

    #[test]
    fn get_smallest_herz_freq() {
        let pitch = Pitch::C(Octave::OctoContra);
        assert_eq!(pitch.abs().0, u7::new(0));
        assert_eq!(Pitch::from(AbsPitch(u7::new(0))), pitch);
        assert_is_close_freq(pitch.get_frequency(), 8.176);
    }
}
