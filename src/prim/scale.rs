use super::{
    interval::{Interval, Octave},
    pitch::{AbsPitch, Pitch, PitchClass},
};

#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
/// Diatonic tonality of the piece with the tonic specified.
///
/// See more: <https://en.wikipedia.org/wiki/Key_signature>
///           <https://en.wikipedia.org/wiki/Tonality>
pub enum KeySig {
    /// Major scale with a starting tonic.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Major_scale>
    Major(PitchClass),

    /// Minor scale with a starting tonic.
    ///
    /// See more: <https://en.wikipedia.org/wiki/Minor_scale>
    Minor(PitchClass),
}

impl Default for KeySig {
    fn default() -> Self {
        // the white piano keys
        Self::Major(PitchClass::C)
    }
}

impl KeySig {
    /// Iterate over a sequence of [`PitchClass`]-es of the scale.
    pub fn get_scale(self) -> impl Iterator<Item = PitchClass> {
        let oc4 = Octave::OneLined;
        let with_octave: Box<dyn Iterator<Item = Pitch>> = match self {
            Self::Major(pc) => Box::new(Pitch::new(pc, oc4).major_scale()),
            Self::Minor(pc) => Box::new(Pitch::new(pc, oc4).natural_minor_scale()),
        };
        with_octave.map(Pitch::class)
    }

    /// [Tonic](https://en.wikipedia.org/wiki/Tonic_(music)) [`PitchClass`] for this scale.
    pub const fn pitch_class(self) -> PitchClass {
        match self {
            Self::Major(pc) | Self::Minor(pc) => pc,
        }
    }

    /// Iterate over a sequence of [`Interval`]-s of the scale.
    pub fn get_intervals_scale(self) -> impl Iterator<Item = Interval> {
        let scale = match self {
            Self::Major(_) => Interval::major_scale(),
            Self::Minor(_) => Interval::natural_minor_scale(),
        };
        let tonic = self.pitch_class().into();
        scale.into_iter().scan(tonic, |state, p| {
            *state += p;
            Some(*state)
        })
    }
}

impl Interval {
    /// Sequence of [`Interval`]-s to create
    /// a [major scale](https://en.wikipedia.org/wiki/Major_scale)
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

    /// Sequence of [`Interval`]s to create
    /// a [minor scale](https://en.wikipedia.org/wiki/Minor_scale#Natural_minor_scale)
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

impl Pitch {
    /// Create a sequence of [`Pitch`]-es
    /// corresponding to cumulative sums of the
    /// given sequence of intervals.
    pub fn get_scale<I, Int>(self, intervals: I) -> impl Iterator<Item = Self> + 'static
    where
        I: Iterator<Item = Int> + 'static,
        Int: Copy + Into<Interval>,
    {
        intervals
            .scan(Interval::zero(), |tonic_distance, interval| {
                *tonic_distance += interval.into();
                Some(*tonic_distance)
            })
            .map(move |distance| self.trans(distance))
    }

    /// Sequence of [`Pitch`]-es that forms
    /// a [major scale](https://en.wikipedia.org/wiki/Major_scale)
    /// starting with the given [`Pitch`].
    pub fn major_scale(self) -> impl Iterator<Item = Self> {
        self.get_scale(Interval::major_scale().into_iter())
    }

    /// Sequence of [`Pitch`]-es that forms
    /// a [minor scale](https://en.wikipedia.org/wiki/Minor_scale#Natural_minor_scale)
    /// starting with the given [`Pitch`].
    pub fn natural_minor_scale(self) -> impl Iterator<Item = Self> {
        self.get_scale(Interval::natural_minor_scale().into_iter())
    }
}

const DIATONIC_SIZE: i8 = 7;

impl AbsPitch {
    /// Transpose current [`AbsPitch`]
    /// in the given [diatonic scale][KeySig].
    pub fn diatonic_trans(self, key: KeySig, degrees: i8) -> Self {
        if degrees == 0 {
            return self;
        }

        let oct_size = Octave::semitones_number();
        let oct_size_i = i8::try_from(u8::from(oct_size)).expect("12 is low enough");

        let scale: Vec<_> = key
            .get_intervals_scale()
            .take(7) // ignore the last one, it is an Octave higher than tonic
            .collect();

        let closest_index = scale
            .iter()
            .enumerate()
            .min_by_key(|(_, x)| {
                let x = u8::from((self - **x).0);
                x.rem_euclid(oct_size.into())
            })
            .map(|(i, _)| i)
            .expect("Scale is non-empty");

        let positive_shift = degrees.rem_euclid(DIATONIC_SIZE);
        let whole_octaves = (degrees - positive_shift) / DIATONIC_SIZE;

        let positive_shift = usize::try_from(positive_shift).expect("Shift is negative");
        let interval = scale
            .into_iter()
            .cycle()
            .nth(closest_index + positive_shift)
            .expect("Cycled non-empty scale has infinite items")
            .0;
        let shift = (interval + oct_size_i)
            .checked_sub(
                i8::try_from(u8::from(self.0) % u8::from(oct_size))
                    .expect("Modulo 12 is low enough for i8"),
            )
            .expect("Positive interval + 12 > X mod 12")
            % oct_size_i;

        self + Interval(shift) + Interval(whole_octaves * oct_size_i)
    }
}

#[cfg(test)]
mod tests {
    use super::{super::pitch::Pitch, *};

    #[test]
    fn major() {
        let oc3 = Octave::Small;
        let middle_c = Pitch::C(oc3);
        let major: Vec<_> = middle_c.major_scale().collect();

        assert_eq!(
            major,
            vec![
                Pitch::C(oc3),
                Pitch::D(oc3),
                Pitch::E(oc3),
                Pitch::F(oc3),
                Pitch::G(oc3),
                Pitch::A(oc3),
                Pitch::B(oc3),
                Pitch::C(Octave::OneLined),
            ]
        );
    }

    #[test]
    fn minor() {
        let oc4 = Octave::OneLined;
        let oc5 = Octave::TwoLined;

        let concert_a = Pitch::A(oc4);
        let minor: Vec<_> = concert_a.natural_minor_scale().collect();

        assert_eq!(
            minor,
            vec![
                Pitch::A(oc4),
                Pitch::B(oc4),
                Pitch::C(oc5),
                Pitch::D(oc5),
                Pitch::E(oc5),
                Pitch::F(oc5),
                Pitch::G(oc5),
                Pitch::A(oc5),
            ]
        );
    }

    #[test]
    fn key_sig_c_major_scale() {
        let scale: Vec<_> = KeySig::Major(PitchClass::C).get_scale().collect();
        assert_eq!(
            scale,
            [
                PitchClass::C,
                PitchClass::D,
                PitchClass::E,
                PitchClass::F,
                PitchClass::G,
                PitchClass::A,
                PitchClass::B,
                PitchClass::C,
            ]
        );

        let i_scale: Vec<_> = KeySig::Major(PitchClass::C).get_intervals_scale().collect();
        assert_eq!(
            i_scale,
            [
                Interval::from(0),
                Interval::from(2),
                Interval::from(4),
                Interval::from(5),
                Interval::from(7),
                Interval::from(9),
                Interval::from(11),
                Interval::from(12),
            ]
        );
    }

    #[test]
    fn key_sig_g_major_scale() {
        let scale: Vec<_> = KeySig::Major(PitchClass::G).get_scale().collect();
        assert_eq!(
            scale,
            [
                PitchClass::G,
                PitchClass::A,
                PitchClass::B,
                PitchClass::C,
                PitchClass::D,
                PitchClass::E,
                PitchClass::Fs,
                PitchClass::G,
            ]
        );

        let i_scale: Vec<_> = KeySig::Major(PitchClass::G).get_intervals_scale().collect();
        assert_eq!(
            i_scale,
            [
                Interval::from(7),
                Interval::from(9),
                Interval::from(11),
                Interval::from(12),
                Interval::from(14),
                Interval::from(16),
                Interval::from(18),
                Interval::from(19),
            ]
        );
    }

    #[test]
    fn diatonic_trans_c_major() {
        let oc4 = Octave::OneLined;
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
        let oc4 = Octave::OneLined;
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
        let oc4 = Octave::OneLined;
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
        let oc4 = Octave::OneLined;
        let oc5 = Octave::TwoLined;
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
                Pitch::new(PitchClass::D, oc5),
            ]
        );
    }

    #[test]
    fn diatonic_trans_more_than_an_octave() {
        let oc4 = Octave::OneLined;
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

        let oc5 = Octave::TwoLined;
        let oc6 = Octave::ThreeLined;
        assert_eq!(
            transposed,
            [
                Pitch::new(PitchClass::F, oc5),
                Pitch::new(PitchClass::G, oc5),
                Pitch::new(PitchClass::D, oc6),
            ]
        );
    }

    #[test]
    fn diatonic_trans_back_more_than_two_octaves() {
        let oc4 = Octave::OneLined;
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

        let oc1 = Octave::Contra;
        let oc2 = Octave::Great;
        assert_eq!(
            transposed,
            [
                Pitch::new(PitchClass::E, oc1),
                Pitch::new(PitchClass::F, oc1),
                Pitch::new(PitchClass::C, oc2),
            ]
        );
    }
}
