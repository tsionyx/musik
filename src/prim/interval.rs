use std::{
    convert::TryFrom,
    ops::{Add, AddAssign, Neg},
};

use enum_iterator::Sequence;
use enum_map::Enum;
use ux2::u4;

use super::pitch::PitchClass;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Enum, Sequence)]
// #[repr(i8)]
/// <https://en.wikipedia.org/wiki/Scientific_pitch_notation>
pub enum Octave {
    // about one octave below the human hearing threshold: its overtones, however, are audible
    OctoContra = -1,
    // A0 is the lowest pitch on a full piano
    SubContra = 0,
    Contra = 1,
    Great = 2,
    Small = 3,
    OneLined = 4,
    TwoLined = 5,
    ThreeLined = 6,
    FourLined = 7,
    // C8 is the highest pitch on a full piano
    FiveLined = 8,
    // G9 is the highest MIDI note
    SixLined = 9,
    // TODO: The 10-th Octave cannot be represented as MIDI
    // SevenLined = 10, // Ef10 is the human hearing threshold
}

impl Octave {
    // TODO: better error type
    pub(crate) fn from_i8(val: i8) -> Result<Self, String> {
        if let Ok(val) = u8::try_from(val) {
            let val = u4::try_from(val).map_err(|_| "Too high for Octave")?;
            Self::try_from(val)
        } else if val == -1 {
            Ok(Self::OctoContra)
        } else {
            Err("Too low Octave".into())
        }
    }
}

impl TryFrom<u4> for Octave {
    type Error = String;

    fn try_from(value: u4) -> Result<Self, Self::Error> {
        match u8::from(value) {
            oc if oc <= 10 => {
                let val: usize = oc.into();
                Ok(Self::from_usize(val + 1))
            }
            _ => Err("Bad value for octave".to_string()),
        }
    }
}

impl Octave {
    pub const MINIMAL_PITCHES: [PitchClass; 12] = [
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

    pub fn semitones_number() -> u8 {
        u8::try_from(Self::MINIMAL_PITCHES.len()).unwrap()
    }
}

#[derive(Debug, Clone, Copy, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct Interval(pub(crate) i8);

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

    pub const fn get_inner(self) -> i8 {
        self.0
    }
}

impl From<i8> for Interval {
    fn from(val: i8) -> Self {
        Self(val)
    }
}

impl Neg for Interval {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
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
