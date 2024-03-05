use std::{
    ops::{Add, Sub},
    str::FromStr,
};

use enum_iterator::Sequence;
use enum_map::Enum;

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
        let octaves_from_a4 = f64::from(interval_to_a4.get_inner())
            / f64::from(Octave::semitones_number().get_inner());
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
pub struct AbsPitch(pub i8);

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

        // TODO: make roundtrip tests for every value in 0..127
        (
            Octave::from_i8(octave - 1).expect("Abs pitch conversion is always valid"),
            u8::try_from(n).expect("Negative interval found"),
        )
    }
}

impl From<Octave> for AbsPitch {
    fn from(octave: Octave) -> Self {
        let octave_size = Octave::semitones_number().0;
        let octave = i8::try_from(octave as isize + 1).expect("Invalid octave");
        Self(octave * octave_size)
    }
}

impl Add for AbsPitch {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
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
        Interval::from(self.0 - rhs.0)
    }
}

impl From<Interval> for AbsPitch {
    fn from(int: Interval) -> Self {
        Self(int.0)
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
        assert_eq!(pitch.abs().0, 60);
        assert_eq!(Pitch::from(AbsPitch(60)), pitch);
        assert_is_close_freq(pitch.get_frequency(), 261.626);
    }

    #[test]
    fn get_smallest_herz_freq() {
        let pitch = Pitch::C(Octave::OctoContra);
        assert_eq!(pitch.abs().0, 0);
        assert_eq!(Pitch::from(AbsPitch(0)), pitch);
        assert_is_close_freq(pitch.get_frequency(), 8.176);
    }
}
