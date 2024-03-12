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
/// Interval between musical pitch and another with half or double of its frequency.
///
/// [`Octave`] registers start from the [`C`][PitchClass::C]
/// and end with the [`B`][PitchClass::B].
///
/// https://en.wikipedia.org/wiki/Octave
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
    // FIXME: The 10-th Octave cannot be represented as MIDI
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
/// Distance between pitches in terms of semitones.
///
/// Could be positive or negative.
///
/// <https://en.wikipedia.org/wiki/Interval_(music)>
pub struct Interval(pub(crate) i8);

impl Interval {
    /// No distance. Also known as [unison](https://en.wikipedia.org/wiki/Unison).
    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn semi_tone() -> Self {
        Self(1)
    }

    pub const fn tone() -> Self {
        Self(2)
    }

    pub fn octave() -> Self {
        let twelve_val = ux2::i5::from(Octave::semitones_number());
        Self(i8::from(twelve_val))
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
    fn enharmonic_equivalence() {
        // 0
        assert!(PitchClass::C.is_enharmonic_equivalent(PitchClass::Dff));

        // 1
        assert!(PitchClass::Cs.is_enharmonic_equivalent(PitchClass::Df));

        // 2
        assert!(PitchClass::Css.is_enharmonic_equivalent(PitchClass::D));
        assert!(PitchClass::D.is_enharmonic_equivalent(PitchClass::Eff));

        // 3
        assert!(PitchClass::Ds.is_enharmonic_equivalent(PitchClass::Ef));
        assert!(PitchClass::Ef.is_enharmonic_equivalent(PitchClass::Fff));

        // 4
        assert!(PitchClass::Dss.is_enharmonic_equivalent(PitchClass::E));
        assert!(PitchClass::E.is_enharmonic_equivalent(PitchClass::Ff));

        // 5
        assert!(PitchClass::Es.is_enharmonic_equivalent(PitchClass::F));
        assert!(PitchClass::F.is_enharmonic_equivalent(PitchClass::Gff));

        // 6
        assert!(PitchClass::Ess.is_enharmonic_equivalent(PitchClass::Fs));
        assert!(PitchClass::Fs.is_enharmonic_equivalent(PitchClass::Gf));

        // 7
        assert!(PitchClass::Fss.is_enharmonic_equivalent(PitchClass::G));
        assert!(PitchClass::G.is_enharmonic_equivalent(PitchClass::Aff));

        // 8
        assert!(PitchClass::Gs.is_enharmonic_equivalent(PitchClass::Af));

        // 9
        assert!(PitchClass::Gss.is_enharmonic_equivalent(PitchClass::A));
        assert!(PitchClass::A.is_enharmonic_equivalent(PitchClass::Bff));

        // 10
        assert!(PitchClass::As.is_enharmonic_equivalent(PitchClass::Bf));

        // 11
        assert!(PitchClass::Ass.is_enharmonic_equivalent(PitchClass::B));
    }
}
