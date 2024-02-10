use std::{
    convert::TryFrom,
    ops::{Add, AddAssign, Neg, Sub},
};

use super::{pitch::PitchClass, KeySig};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// 0..8 on piano
pub struct Octave(pub(crate) i8);

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
    pub(super) const MINIMAL_PITCHES: [PitchClass; 12] = [
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

    pub(super) fn semitones_number() -> Interval {
        let len = i8::try_from(Self::MINIMAL_PITCHES.len()).unwrap();
        Interval(len)
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

    pub const fn get_inner(self) -> i8 {
        self.0
    }

    pub const fn major_scale() -> [Self; 8] {
        [
            Self::zero(),
            Self::tone(),
            Self::tone(),
            Self::semi_tone(),
            Self::tone(),
            Self::tone(),
            Self::tone(),
            Self::semi_tone(),
        ]
    }

    pub const fn natural_minor_scale() -> [Self; 8] {
        [
            Self::zero(),
            Self::tone(),
            Self::semi_tone(),
            Self::tone(),
            Self::tone(),
            Self::semi_tone(),
            Self::tone(),
            Self::tone(),
        ]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
        Interval(self.0 - rhs.0)
    }
}

impl From<Interval> for AbsPitch {
    fn from(int: Interval) -> Self {
        Self(int.0)
    }
}

impl AbsPitch {
    pub fn diatonic_trans(self, key: KeySig, degrees: i8) -> Self {
        if degrees == 0 {
            return self;
        }

        const DIATONIC_SIZE: i8 = 7;
        let oct_size = Octave::semitones_number().0;

        let scale: Vec<_> = key
            .get_intervals_scale()
            .map(Self::from)
            .take(7) // ignore the last one, it is an Octave higher than tonic
            .collect();

        let closest_index = scale
            .iter()
            .enumerate()
            .min_by_key(|(_, x)| (self - **x).0.rem_euclid(oct_size))
            .map(|(i, _)| i)
            .expect("Scale is non-empty");

        let positive_shift = degrees.rem_euclid(DIATONIC_SIZE);
        let whole_octaves = (degrees - positive_shift) / DIATONIC_SIZE;

        let interval = scale
            .into_iter()
            .cycle()
            .nth(closest_index + positive_shift as usize)
            .expect("Cycled non-empty scale has infinite items")
            .0;
        let shift = (interval + oct_size)
            .checked_sub(self.0 % oct_size)
            .unwrap()
            % oct_size;

        self + Self(shift + (whole_octaves * oct_size))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{pitch::Pitch, KeySig},
        *,
    };

    #[test]
    fn diatonic_trans_c_major() {
        let oc4 = Octave::ONE_LINED;
        let key = KeySig::Major(PitchClass::C);

        let pitches = [
            Pitch::new(PitchClass::C, oc4),
            Pitch::new(PitchClass::D, oc4),
            Pitch::new(PitchClass::E, oc4),
        ];

        let transposed: Vec<_> = pitches
            .into_iter()
            .map(|p| Pitch::from(p.abs().diatonic_trans(key, 2)))
            .collect();

        assert_eq!(
            transposed,
            [
                Pitch::new(PitchClass::E, oc4),
                Pitch::new(PitchClass::F, oc4),
                Pitch::new(PitchClass::G, oc4),
            ]
        );
    }

    #[test]
    fn diatonic_trans_g_major() {
        let oc4 = Octave::ONE_LINED;
        let key = KeySig::Major(PitchClass::G);

        let pitches = [
            Pitch::new(PitchClass::C, oc4),
            Pitch::new(PitchClass::D, oc4),
            Pitch::new(PitchClass::E, oc4),
        ];

        let transposed: Vec<_> = pitches
            .into_iter()
            .map(|p| Pitch::from(p.abs().diatonic_trans(key, 2)))
            .collect();

        assert_eq!(
            transposed,
            [
                Pitch::new(PitchClass::E, oc4),
                Pitch::new(PitchClass::Fs, oc4),
                Pitch::new(PitchClass::G, oc4),
            ]
        );
    }

    #[test]
    fn diatonic_trans_not_matching() {
        let oc4 = Octave::ONE_LINED;
        let key = KeySig::Major(PitchClass::C);

        let pitches = [
            Pitch::new(PitchClass::C, oc4),
            Pitch::new(PitchClass::Ds, oc4), // this Pitch is not from the C-Major scale
            Pitch::new(PitchClass::E, oc4),
        ];

        let transposed: Vec<_> = pitches
            .into_iter()
            .map(|p| Pitch::from(p.abs().diatonic_trans(key, 2)))
            .collect();

        assert_eq!(
            transposed,
            [
                Pitch::new(PitchClass::E, oc4),
                Pitch::new(PitchClass::F, oc4),
                Pitch::new(PitchClass::G, oc4),
            ]
        );
    }

    #[test]
    fn diatonic_trans_wrapping_around_octave() {
        let oc4 = Octave::ONE_LINED;
        let key = KeySig::Major(PitchClass::C);

        let pitches = [
            Pitch::new(PitchClass::C, oc4),
            Pitch::new(PitchClass::D, oc4),
            Pitch::new(PitchClass::A, oc4),
        ];

        let transposed: Vec<_> = pitches
            .into_iter()
            .map(|p| Pitch::from(p.abs().diatonic_trans(key, 3)))
            .collect();

        assert_eq!(
            transposed,
            [
                Pitch::new(PitchClass::F, oc4),
                Pitch::new(PitchClass::G, oc4),
                Pitch::new(PitchClass::D, Octave::from(5)),
            ]
        );
    }

    #[test]
    fn diatonic_trans_more_than_an_octave() {
        let oc4 = Octave::ONE_LINED;
        let key = KeySig::Major(PitchClass::C);

        let pitches = [
            Pitch::new(PitchClass::C, oc4),
            Pitch::new(PitchClass::D, oc4),
            Pitch::new(PitchClass::A, oc4),
        ];

        let transposed: Vec<_> = pitches
            .into_iter()
            // single octave is exactly 7 diatonic notes long
            // so we should transpose one octave and 3 notes more
            .map(|p| Pitch::from(p.abs().diatonic_trans(key, 10)))
            .collect();

        let oc5 = Octave::from(5);
        assert_eq!(
            transposed,
            [
                Pitch::new(PitchClass::F, oc5),
                Pitch::new(PitchClass::G, oc5),
                Pitch::new(PitchClass::D, Octave::from(6)),
            ]
        );
    }

    #[test]
    fn diatonic_trans_back_more_than_two_octaves() {
        let oc4 = Octave::ONE_LINED;
        let key = KeySig::Major(PitchClass::C);

        let pitches = [
            Pitch::new(PitchClass::C, oc4),
            Pitch::new(PitchClass::Ds, oc4),
            Pitch::new(PitchClass::A, oc4),
        ];

        let transposed: Vec<_> = pitches
            .into_iter()
            // shift 3 octaves back and two notes forward (7 * -3 + 2 = -19)
            .map(|p| Pitch::from(p.abs().diatonic_trans(key, -19)))
            .collect();

        let oc1 = Octave::from(1);
        assert_eq!(
            transposed,
            [
                Pitch::new(PitchClass::E, oc1),
                Pitch::new(PitchClass::F, oc1),
                Pitch::new(PitchClass::C, Octave::from(2)),
            ]
        );
    }
}
