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

    pub fn abs_checked(self) -> Result<AbsPitch, ErrorPitchClipping> {
        AbsPitch::from(self.octave).checked_add(Interval::from(self.class))
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
    // TODO: operator >> and <<
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ErrorPitchClipping {
    TooLow,
    TooHigh,
}

impl ErrorPitchClipping {
    /// Saturating the values into the defined range.
    fn clip_to(self) -> u7 {
        match self {
            Self::TooLow => u7::MIN,
            Self::TooHigh => u7::MAX,
        }
    }
}

impl AbsPitch {
    pub fn checked_add(self, rhs: Interval) -> Result<Self, ErrorPitchClipping> {
        let a = i8::try_from(u8::from(self.0)).expect("u7 should convert to i8 seamlessly");
        let b = rhs.0;

        let sum = a.checked_add(b).ok_or(ErrorPitchClipping::TooHigh)?;
        let val = u8::try_from(sum).map_err(|_| ErrorPitchClipping::TooLow)?;
        let val = u7::try_from(val).expect("i8 > 0 should convert to u7 seamlessly");
        Ok(Self(val))
    }

    pub fn checked_sub(self, rhs: Interval) -> Result<Self, ErrorPitchClipping> {
        if rhs.0 == i8::MIN {
            // prevent negate with overflow
            return Err(ErrorPitchClipping::TooHigh);
        }
        self.checked_add(-rhs)
    }
}

impl Add<Interval> for AbsPitch {
    type Output = Self;

    fn add(self, rhs: Interval) -> Self {
        self.checked_add(rhs)
            .unwrap_or_else(|err| Self(err.clip_to()))
    }
}

impl Sub<Interval> for AbsPitch {
    type Output = Self;

    fn sub(self, rhs: Interval) -> Self::Output {
        self.checked_sub(rhs)
            .unwrap_or_else(|err| Self(err.clip_to()))
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

    #[test]
    fn all_pitches_abs() {
        fn f(abs: u8, pitch: (Octave, PitchClass)) {
            let abs = AbsPitch::from(u7::try_from(abs).unwrap());
            let p = Pitch {
                class: pitch.1,
                octave: pitch.0,
            };
            assert_eq!(abs, p.abs());
            assert_eq!(p, Pitch::from(abs));
        }

        use {Octave::*, PitchClass::*};

        f(0, (OctoContra, C));
        f(2, (OctoContra, D));
        f(4, (OctoContra, E));
        f(5, (OctoContra, F));
        f(7, (OctoContra, G));
        f(9, (OctoContra, A));
        f(11, (OctoContra, B));

        f(12, (SubContra, C));
        f(13, (SubContra, Cs));
        f(14, (SubContra, D));
        f(15, (SubContra, Ds));
        f(16, (SubContra, E));
        f(17, (SubContra, F));
        f(18, (SubContra, Fs));
        f(19, (SubContra, G));
        f(20, (SubContra, Gs));
        f(21, (SubContra, A));
        f(22, (SubContra, As));
        f(23, (SubContra, B));

        f(60, (OneLined, C)); // Middle C
        f(69, (OneLined, A)); // A440

        f(120, (SixLined, C));
        f(121, (SixLined, Cs));
        f(122, (SixLined, D));
        f(123, (SixLined, Ds));
        f(124, (SixLined, E));
        f(125, (SixLined, F));
        f(126, (SixLined, Fs));
        f(127, (SixLined, G));
    }

    #[test]
    fn clipping_at_too_low_pitches() {
        use PitchClass::*;

        for pc in [C, Cf, Cff] {
            let p = Pitch::new(pc, Octave::OctoContra);
            let abs = p.abs();
            assert_eq!(u8::from(abs.0), 0);

            if pc == C {
                assert_eq!(Pitch::from(abs), p);
                assert_eq!(p.abs_checked().unwrap(), abs);
            } else {
                assert_ne!(Pitch::from(abs), p);
                assert_eq!(p.abs_checked().unwrap_err(), ErrorPitchClipping::TooLow);
            }
        }
    }

    #[test]
    fn clipping_at_too_high_pitches() {
        use PitchClass::*;

        for pc in [
            Fss, G, Aff, /*the first three are valid*/
            Gs, Af, Gss, A, Bff, As, Bf, Ass, B,
        ] {
            let p = Pitch::new(pc, Octave::SixLined);
            let abs = p.abs();
            assert_eq!(u8::from(abs.0), 127);

            if [Fss, G, Aff].contains(&pc) {
                assert_eq!(p.abs_checked().unwrap(), abs);
                if pc == G {
                    assert_eq!(Pitch::from(abs), p);
                }
            } else {
                assert_ne!(Pitch::from(abs), p);
                assert_eq!(p.abs_checked().unwrap_err(), ErrorPitchClipping::TooHigh);
            }
        }
    }

    #[test]
    fn all_intervals_for_all_pitches() {
        for p in 0..=127 {
            let abs = AbsPitch(u7::new(p));

            for i in -128..=0 {
                let int = Interval(i);
                if i16::from(p) < -i16::from(i) {
                    assert_eq!(
                        abs.checked_add(int).unwrap_err(),
                        ErrorPitchClipping::TooLow
                    );
                    assert_eq!(u8::from((abs + int).0), 0);
                } else {
                    assert_eq!(
                        u8::from(abs.checked_add(int).unwrap().0),
                        (i16::from(p) + i16::from(i)) as u8,
                    );
                }

                if i16::from(p) - i16::from(i) > 127 {
                    assert_eq!(
                        abs.checked_sub(int).unwrap_err(),
                        ErrorPitchClipping::TooHigh
                    );
                    assert_eq!(u8::from((abs - int).0), 127);
                } else {
                    assert_eq!(
                        u8::from(abs.checked_sub(int).unwrap().0),
                        (i16::from(p) - i16::from(i)) as u8,
                    );
                }
            }

            for i in 0..=127 {
                let int = Interval(i);
                if i16::from(p) < i16::from(i) {
                    assert_eq!(
                        abs.checked_sub(int).unwrap_err(),
                        ErrorPitchClipping::TooLow
                    );
                    assert_eq!(u8::from((abs - int).0), 0);
                } else {
                    assert_eq!(
                        u8::from(abs.checked_sub(int).unwrap().0),
                        (i16::from(p) - i16::from(i)) as u8,
                    );
                }

                if i16::from(p) + i16::from(i) > 127 {
                    assert_eq!(
                        abs.checked_add(int).unwrap_err(),
                        ErrorPitchClipping::TooHigh
                    );
                    assert_eq!(u8::from((abs + int).0), 127);
                } else {
                    assert_eq!(
                        u8::from(abs.checked_add(int).unwrap().0),
                        (i16::from(p) + i16::from(i)) as u8,
                    );
                }
            }
        }
    }

    #[test]
    fn next_for_all_but_last_pitches() {
        for i in 0..127 {
            let abs = AbsPitch(u7::new(i));
            let p = Pitch::from(abs);
            assert_eq!(u8::from(p.next().abs().0), i + 1);
        }
    }

    #[test]
    fn clipped_next_for_last_pitch() {
        let p = Pitch::from(AbsPitch(u7::new(127)));
        assert_eq!(u8::from(p.next().abs().0), 127);
    }

    #[test]
    fn prev_for_all_but_first_pitches() {
        for i in 1..=127 {
            let abs = AbsPitch(u7::new(i));
            let p = Pitch::from(abs);
            assert_eq!(u8::from(p.prev().abs().0), i - 1);
        }
    }

    #[test]
    fn clipped_prev_for_first_pitch() {
        let p = Pitch::from(AbsPitch(u7::new(0)));
        assert_eq!(u8::from(p.prev().abs().0), 0);
    }

    #[test]
    fn from_octave() {
        for (i, oc) in enum_iterator::all::<Octave>().enumerate() {
            assert_eq!(oc, Octave::from_i8(i as i8 - 1).unwrap());
            let p = AbsPitch::from(oc);
            assert_eq!(u8::from(p.get_inner()), 12 * (i as u8));
        }
    }

    #[test]
    fn to_octave_and_offset() {
        for p in 0..=127 {
            let p1 = AbsPitch(u7::new(p));
            let (oc, offset) = p1.into();
            let o = p / 12;
            assert_eq!(oc, Octave::from_i8(o as i8 - 1).unwrap());
            assert_eq!(u8::from(offset), p % 12);

            let p2 = Pitch::from(p1);
            assert_eq!(p2.class().distance_from_c() as u8, u8::from(offset));
            assert_eq!(p2.octave, oc);
            assert_eq!(p2.abs(), p1);
        }
    }

    #[test]
    fn all_pitch_differences() {
        for p in 0..=127 {
            let p1 = AbsPitch(u7::new(p));

            for p in 0..=127 {
                let p2 = AbsPitch(u7::new(p));
                let diff = p1 - p2;
                dbg!(p1, p2, diff);

                assert_eq!(p2 + diff, p1);
                assert_eq!(p1 - diff, p2);
                assert_eq!(p2 - p1, -diff);
            }
        }
    }
}
