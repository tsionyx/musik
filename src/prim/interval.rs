use std::{
    convert::TryFrom,
    ops::{Add, AddAssign, Neg},
};

use enum_iterator::Sequence;
use enum_map::Enum;
use ux2::u4;

use super::pitch::PitchClass;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Enum, Sequence)]
#[repr(i8)]
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
    pub(crate) fn from_i8(val: i8) -> Result<Self, ErrorOctaveFromNum> {
        if let Ok(val) = u8::try_from(val) {
            let val = u4::try_from(val).map_err(|_| ErrorOctaveFromNum::TooHigh)?;
            Self::try_from(val)
        } else if val == -1 {
            Ok(Self::OctoContra)
        } else {
            Err(ErrorOctaveFromNum::TooLow)
        }
    }
}

impl TryFrom<u4> for Octave {
    type Error = ErrorOctaveFromNum;

    fn try_from(value: u4) -> Result<Self, Self::Error> {
        match u8::from(value) {
            oc if oc < 10 => {
                let val: usize = oc.into();
                Ok(Self::from_usize(val + 1))
            }
            _ => Err(ErrorOctaveFromNum::TooHigh),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ErrorOctaveFromNum {
    TooLow,
    TooHigh,
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

    pub fn semitones_number() -> u4 {
        u4::try_from(Self::MINIMAL_PITCHES.len()).expect("12 is low enough")
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

impl PitchClass {
    pub(crate) fn distance_from_c(self) -> i8 {
        match self {
            Self::Cff => -2,
            Self::Cf => -1,
            Self::C => 0,
            Self::Cs => 1,
            Self::Css => 2,
            Self::Dff => 0,
            Self::Df => 1,
            Self::D => 2,
            Self::Ds => 3,
            Self::Dss => 4,
            Self::Eff => 2,
            Self::Ef => 3,
            Self::E => 4,
            Self::Es => 5,
            Self::Ess => 6,
            Self::Fff => 3,
            Self::Ff => 4,
            Self::F => 5,
            Self::Fs => 6,
            Self::Fss => 7,
            Self::Gff => 5,
            Self::Gf => 6,
            Self::G => 7,
            Self::Gs => 8,
            Self::Gss => 9,
            Self::Aff => 7,
            Self::Af => 8,
            Self::A => 9,
            Self::As => 10,
            Self::Ass => 11,
            Self::Bff => 9,
            Self::Bf => 10,
            Self::B => 11,
            Self::Bs => 12,
            Self::Bss => 13,
        }
    }
}

impl From<PitchClass> for Interval {
    fn from(pc: PitchClass) -> Self {
        Self(pc.distance_from_c())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn octave_conversion_from_i8() {
        for i in -128..-1 {
            let err = Octave::from_i8(i).unwrap_err();
            assert_eq!(err, ErrorOctaveFromNum::TooLow);
        }

        let oc = Octave::from_i8(-1).unwrap();
        assert_eq!(oc, Octave::OctoContra);

        let oc = Octave::from_i8(0).unwrap();
        assert_eq!(oc, Octave::SubContra);

        let oc = Octave::from_i8(1).unwrap();
        assert_eq!(oc, Octave::Contra);

        let oc = Octave::from_i8(2).unwrap();
        assert_eq!(oc, Octave::Great);

        let oc = Octave::from_i8(3).unwrap();
        assert_eq!(oc, Octave::Small);

        let oc = Octave::from_i8(4).unwrap();
        assert_eq!(oc, Octave::OneLined);

        let oc = Octave::from_i8(5).unwrap();
        assert_eq!(oc, Octave::TwoLined);

        let oc = Octave::from_i8(6).unwrap();
        assert_eq!(oc, Octave::ThreeLined);

        let oc = Octave::from_i8(7).unwrap();
        assert_eq!(oc, Octave::FourLined);

        let oc = Octave::from_i8(8).unwrap();
        assert_eq!(oc, Octave::FiveLined);

        let oc = Octave::from_i8(9).unwrap();
        assert_eq!(oc, Octave::SixLined);

        // let oc = Octave::from_i8(10).unwrap();
        // assert_eq!(oc, Octave::SevenLined);

        for i in 10..=127 {
            let err = Octave::from_i8(i).unwrap_err();
            assert_eq!(err, ErrorOctaveFromNum::TooHigh);
        }
    }

    #[test]
    fn octave_conversion_from_u4() {
        let oc = Octave::try_from(u4::new(0)).unwrap();
        assert_eq!(oc, Octave::SubContra);

        let oc = Octave::try_from(u4::new(1)).unwrap();
        assert_eq!(oc, Octave::Contra);

        let oc = Octave::try_from(u4::new(2)).unwrap();
        assert_eq!(oc, Octave::Great);

        let oc = Octave::try_from(u4::new(3)).unwrap();
        assert_eq!(oc, Octave::Small);

        let oc = Octave::try_from(u4::new(4)).unwrap();
        assert_eq!(oc, Octave::OneLined);

        let oc = Octave::try_from(u4::new(5)).unwrap();
        assert_eq!(oc, Octave::TwoLined);

        let oc = Octave::try_from(u4::new(6)).unwrap();
        assert_eq!(oc, Octave::ThreeLined);

        let oc = Octave::try_from(u4::new(7)).unwrap();
        assert_eq!(oc, Octave::FourLined);

        let oc = Octave::try_from(u4::new(8)).unwrap();
        assert_eq!(oc, Octave::FiveLined);

        let oc = Octave::try_from(u4::new(9)).unwrap();
        assert_eq!(oc, Octave::SixLined);

        // let oc = Octave::try_from(u4::new(10)).unwrap();
        // assert_eq!(oc, Octave::SevenLined);

        for i in 10..16 {
            let err = Octave::try_from(u4::new(i)).unwrap_err();
            assert_eq!(err, ErrorOctaveFromNum::TooHigh);
        }
    }

    #[test]
    /// <https://en.wikipedia.org/wiki/Enharmonic_equivalence>
    fn enharmonic_equivalence() {
        // 0
        assert_eq!(
            PitchClass::C.distance_from_c(),
            PitchClass::Dff.distance_from_c()
        );

        // 1
        assert_eq!(
            PitchClass::Cs.distance_from_c(),
            PitchClass::Df.distance_from_c()
        );

        // 2
        assert_eq!(
            PitchClass::Css.distance_from_c(),
            PitchClass::D.distance_from_c()
        );
        assert_eq!(
            PitchClass::D.distance_from_c(),
            PitchClass::Eff.distance_from_c()
        );

        // 3
        assert_eq!(
            PitchClass::Ds.distance_from_c(),
            PitchClass::Ef.distance_from_c()
        );
        assert_eq!(
            PitchClass::Ef.distance_from_c(),
            PitchClass::Fff.distance_from_c()
        );

        // 4
        assert_eq!(
            PitchClass::Dss.distance_from_c(),
            PitchClass::E.distance_from_c()
        );
        assert_eq!(
            PitchClass::E.distance_from_c(),
            PitchClass::Ff.distance_from_c()
        );

        // 5
        assert_eq!(
            PitchClass::Es.distance_from_c(),
            PitchClass::F.distance_from_c()
        );
        assert_eq!(
            PitchClass::F.distance_from_c(),
            PitchClass::Gff.distance_from_c()
        );

        // 6
        assert_eq!(
            PitchClass::Ess.distance_from_c(),
            PitchClass::Fs.distance_from_c()
        );
        assert_eq!(
            PitchClass::Fs.distance_from_c(),
            PitchClass::Gf.distance_from_c()
        );

        // 7
        assert_eq!(
            PitchClass::Fss.distance_from_c(),
            PitchClass::G.distance_from_c()
        );
        assert_eq!(
            PitchClass::G.distance_from_c(),
            PitchClass::Aff.distance_from_c()
        );

        // 8
        assert_eq!(
            PitchClass::Gs.distance_from_c(),
            PitchClass::Af.distance_from_c()
        );

        // 9
        assert_eq!(
            PitchClass::Gss.distance_from_c(),
            PitchClass::A.distance_from_c()
        );
        assert_eq!(
            PitchClass::A.distance_from_c(),
            PitchClass::Bff.distance_from_c()
        );

        // 10
        assert_eq!(
            PitchClass::As.distance_from_c(),
            PitchClass::Bf.distance_from_c()
        );

        // 11
        assert_eq!(
            PitchClass::Ass.distance_from_c(),
            PitchClass::B.distance_from_c()
        );
    }
}
