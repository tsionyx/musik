use std::{
    convert::TryFrom,
    ops::{Add, AddAssign, Neg},
};

use super::pitch::PitchClass;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// 0..8 on piano
pub struct Octave(pub i8);

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

    pub fn semitones_number() -> Interval {
        let len = i8::try_from(Self::MINIMAL_PITCHES.len()).unwrap();
        Interval(len)
    }
}

#[derive(Debug, Clone, Copy, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct Interval(pub i8);

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
