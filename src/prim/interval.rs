use std::ops::{Add, AddAssign, Neg};

use enum_iterator::Sequence;
use enum_map::Enum;
use ux2::u4;

use super::pitch::PitchClass;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Enum, Sequence)]
#[repr(i8)]
/// [`Octave`] registers start from the [`C`][PitchClass::C]
/// and end with the [`B`][PitchClass::B].
///
/// A single octave registry occupies one [whole octave][Interval::octave] (in terms of intervals).
///
/// See more:
/// - <https://en.wikipedia.org/wiki/Scientific_pitch_notation>
/// - <https://en.wikipedia.org/wiki/Musical_note#Scientific_versus_Helmholtz_pitch_notation>
pub enum Octave {
    /// About one octave below the human
    /// [hearing threshold](https://en.wikipedia.org/wiki/Hearing_range):
    /// its overtones, however, are audible.
    ///
    /// Frequency range: 8.176 Hz - 15.434 Hz
    ///
    /// The lowest pitch (C-1) is also called 'Quadruple low C' or 'Quadruple pedal C'.
    OctoContra = -1,

    /// E0 is the commonly accepted lower limit of human
    /// [hearing threshold](https://en.wikipedia.org/wiki/Hearing_range).
    ///
    /// A0 is the lowest pitch on a full piano.
    ///
    /// Frequency range: 16.352 Hz - 30.868 Hz
    ///
    /// The lowest pitch (C0) is also called 'Triple low C' or 'Triple pedal C'.
    SubContra = 0,

    /// Frequency range: 32.703 Hz - 61.735 Hz
    ///
    /// The lowest pitch (C1) is also called 'Double low C' or 'Double pedal C'.
    Contra = 1,

    /// Frequency range: 65.406 Hz - 123.47 Hz
    ///
    /// The lowest pitch (C2) is also called 'Low C', 'Pedal C' or 'Cello C'.
    Great = 2,

    /// Frequency range: 130.813 Hz - 246.941 Hz
    ///
    /// The lowest pitch (C3) is also called 'Bass C' or 'Viola C'.
    Small = 3,

    /// [A4](https://en.wikipedia.org/wiki/A440_(pitch_standard))
    /// is the reference note for tuning standard
    /// (also known as [A440][crate::music::constructors::A440]).
    ///
    /// Frequency range: 261.626 Hz - 493.883 Hz
    ///
    /// The lowest pitch (C4) is also called
    /// [Middle C](https://en.wikipedia.org/wiki/C_(musical_note)#Middle_C).
    OneLined = 4,

    /// Frequency range: 523.251 Hz - 987.767 Hz
    ///
    /// The lowest pitch (C5) is also called 'Treble C'.
    TwoLined = 5,

    /// Frequency range: 1046.502 Hz - 1975.533 Hz
    ///
    /// The lowest pitch (C6) is also called 'High C' or 'Top C'.
    ThreeLined = 6,

    /// Frequency range: 2093.005 Hz - 3951.066 Hz
    ///
    /// The lowest pitch (C7) is also called 'Double high C' or 'Double top C'.
    FourLined = 7,

    /// C8 is the highest pitch on a full piano.
    ///
    /// Frequency range: 4186.009 Hz - 7902.133 Hz
    ///
    /// The lowest pitch (C8) is also called 'Triple high C' or 'Triple top C'.
    FiveLined = 8,

    /// G9 is the highest MIDI note.
    ///
    /// All the subsequent pitches:
    /// - _G#9_ (_Ab9_);
    /// - _A9_ (_G##9_, _Bbb9_);
    /// - _A#9_ (_Bb9_, _Cbb9_);
    /// - _B9_ (_A##9_)
    ///
    /// representable with this [octave][Octave::SixLined]
    /// will be eventually clipped to G9 when playing through MIDI.
    ///
    /// Frequency range: 8372.018 Hz - 15804.27 Hz
    ///
    /// The lowest pitch (C9) is also called 'Quadruple high C' or 'Quadruple top C'.
    SixLined = 9,
    // FIXME: The 10-th Octave cannot be represented as MIDI
    // /// Ef10 is the commonly accepted upper limit of human
    // /// [hearing threshold](https://en.wikipedia.org/wiki/Hearing_range).
    // ///
    // /// Frequency range: 16744.04 Hz - 31608.53 Hz
    // ///
    // /// The lowest pitch (C10) is also called 'Quintuple high C' or 'Quintuple top C'.
    // SevenLined = 10,
}

impl Octave {
    pub(crate) fn from_i8(val: i8) -> Result<Self, ErrorOctaveTryFromNum> {
        if let Ok(val) = u8::try_from(val) {
            let val = u4::try_from(val).map_err(|_| ErrorOctaveTryFromNum::TooHigh)?;
            Self::try_from(val)
        } else if val == -1 {
            Ok(Self::OctoContra)
        } else {
            Err(ErrorOctaveTryFromNum::TooLow)
        }
    }
}

impl TryFrom<u4> for Octave {
    type Error = ErrorOctaveTryFromNum;

    fn try_from(value: u4) -> Result<Self, Self::Error> {
        match u8::from(value) {
            oc if oc < 10 => {
                let val: usize = oc.into();
                Ok(Self::from_usize(val + 1))
            }
            _ => Err(ErrorOctaveTryFromNum::TooHigh),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
/// Signifies that the conversion of a number
/// to the [`Octave`] value
/// jumps out of its defined range [-1..=9].
pub enum ErrorOctaveTryFromNum {
    /// Underflow.
    ///
    /// The [`Octave`] cannot be created as the
    /// given number is less than the minimum bound.
    TooLow,

    /// Overflow.
    ///
    /// The [`Octave`] cannot be created as the
    /// given number is greater than the maximum bound.
    TooHigh,
}

impl Octave {
    /// The set of unique [`PitchClass`]es in a single [`Octave`]
    /// (up to enharmonic equivalence).
    ///
    /// If the [`PitchClass`] have an enharmonically equivalent
    /// diatonic pitch class (a white piano key),
    /// then this [`PitchClass`] is used here.
    ///
    /// Otherwise (for black piano keys), the minimal form with # (sharp) is used.
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

    /// Number of chromatic semitones in an [`Octave`].
    ///
    /// ```
    /// # use musik::Octave;
    /// assert_eq!(u8::from(Octave::semitones_number()), 12);
    /// ```
    pub fn semitones_number() -> u4 {
        u4::try_from(Self::MINIMAL_PITCHES.len()).expect("12 is low enough")
    }
}

#[derive(Debug, Clone, Copy, Default, Ord, PartialOrd, Eq, PartialEq)]
/// Distance between pitches in terms of semitones.
///
/// Could be positive or negative.
///
/// See more: <https://en.wikipedia.org/wiki/Interval_(music)>
pub struct Interval(pub(crate) i8);

impl Interval {
    /// No distance. Also known as [unison](https://en.wikipedia.org/wiki/Unison).
    pub const fn zero() -> Self {
        Self(0)
    }

    /// The smallest possible [`Interval`].
    /// Defined as the interval between two adjacent notes in a 12-tone scale.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Semitone>
    pub const fn semi_tone() -> Self {
        Self(1)
    }

    /// Interval composed of two [semitones][Self::semi_tone].
    /// Also called whole tone or major second (M2).
    ///
    /// See more: <https://en.wikipedia.org/wiki/Major_second>
    pub const fn tone() -> Self {
        Self(2)
    }

    /// Interval between musical pitch and another with half or double of its frequency.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Octave>
    pub fn octave() -> Self {
        let twelve_val = ux2::i5::from(Octave::semitones_number());
        Self(i8::from(twelve_val))
    }

    /// Get the internal numeric representation.
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
            assert_eq!(err, ErrorOctaveTryFromNum::TooLow);
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
            assert_eq!(err, ErrorOctaveTryFromNum::TooHigh);
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
            assert_eq!(err, ErrorOctaveTryFromNum::TooHigh);
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
