use std::str::FromStr;

use enum_iterator::Sequence;
use enum_map::Enum;

use super::interval::{AbsPitch, Interval, Octave};

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
        let a4 = Self::A(Octave::ONE_LINED);
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
