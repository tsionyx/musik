#[cfg(test)]
use ux2::{u4, u7};

#[cfg(test)]
use musik::{AbsPitch, Interval};
use musik::{Dur, Music, Octave, Pitch};

pub fn t251() -> Music {
    let oc4 = Octave::OneLined;
    let oc5 = Octave::TwoLined;
    let d_minor = Music::D(oc4, Dur::WHOLE) | Music::F(oc4, Dur::WHOLE) | Music::A(oc4, Dur::WHOLE);
    let g_major = Music::G(oc4, Dur::WHOLE) | Music::B(oc4, Dur::WHOLE) | Music::D(oc5, Dur::WHOLE);
    let c_major =
        Music::C(oc4, Dur::BREVIS) | Music::E(oc4, Dur::BREVIS) | Music::G(oc4, Dur::BREVIS);

    d_minor + g_major + c_major
}

/// Exercise 2.1
/// Generate a ii-V-I chord progression
/// in the key whose major scale begins on the `pitch`
/// (i.e. the first degree of the major scale on which the progression is being constructed)
/// where the duration of the first two chords is each `duration`,
/// and the duration of the last chord is `2 ∗ duration`.
pub fn two_five_one(pitch: Pitch, duration: Dur) -> Music {
    let double_duration = duration.double();
    let whole_major_scale: Vec<_> = pitch.major_scale().collect();
    let second = whole_major_scale[1];
    let fifth = whole_major_scale[4];

    let second_minor: Vec<_> = second.natural_minor_scale().collect();
    let fifth_major: Vec<_> = fifth.major_scale().collect();
    let first_major: Vec<_> = pitch.major_scale().collect();

    let chord_indexes = [2_usize, 4];

    let second_minor_chord = chord_indexes
        .iter()
        .map(|&index| Music::note(duration, second_minor[index]))
        .fold(Music::note(duration, second), |acc, x| acc | x);

    let fifth_major_chord = chord_indexes
        .iter()
        .map(|&index| Music::note(duration, fifth_major[index]))
        .fold(Music::note(duration, fifth), |acc, x| acc | x);

    let first_major_chord = chord_indexes
        .iter()
        .map(|&index| Music::note(double_duration, first_major[index]))
        .fold(Music::note(double_duration, pitch), |acc, x| acc | x);

    second_minor_chord + fifth_major_chord + first_major_chord
}

#[test]
fn test_t251() {
    let oc = Octave::try_from(u4::new(4)).unwrap();
    assert_eq!(t251(), two_five_one(Pitch::C(oc), Dur::WHOLE));
}

pub mod blues {
    //! Exercise 2.2
    //! Pentatonic blues scale consists of five notes
    //! and, in the key of C, approximately corresponds
    //! to the notes C, Ef, F, G, and Bf.
    use super::{Dur, Music, Octave, Pitch};
    use musik::PitchClass;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BluesPitchClass {
        Root,
        MinorThird,
        Fourth,
        Fifth,
        MinorSeven,
    }

    impl BluesPitchClass {
        pub const fn to_western(self) -> PitchClass {
            match self {
                Self::Root => PitchClass::C,
                Self::MinorThird => PitchClass::Ef,
                Self::Fourth => PitchClass::F,
                Self::Fifth => PitchClass::G,
                Self::MinorSeven => PitchClass::Bf,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct BluesPitch {
        class: BluesPitchClass,
        octave: Octave,
    }

    impl BluesPitch {
        pub const fn to_western(self) -> Pitch {
            Pitch::new(self.class.to_western(), self.octave)
        }
    }

    macro_rules! def_pitch_constructor {
        ($pitch: ident) => {
            #[allow(non_snake_case)]
            pub const fn $pitch(octave: Octave) -> Self {
                Self::new(BluesPitchClass::$pitch, octave)
            }
        };
    }

    impl BluesPitch {
        const fn new(class: BluesPitchClass, octave: Octave) -> Self {
            Self { class, octave }
        }

        def_pitch_constructor!(Root);
        def_pitch_constructor!(MinorThird);
        def_pitch_constructor!(Fourth);
        def_pitch_constructor!(Fifth);
        def_pitch_constructor!(MinorSeven);
    }

    pub const fn ro(octave: Octave, duration: Dur) -> Music<BluesPitch> {
        Music::note(duration, BluesPitch::Root(octave))
    }

    pub const fn mt(octave: Octave, duration: Dur) -> Music<BluesPitch> {
        Music::note(duration, BluesPitch::MinorThird(octave))
    }

    pub const fn fo(octave: Octave, duration: Dur) -> Music<BluesPitch> {
        Music::note(duration, BluesPitch::Fourth(octave))
    }

    pub const fn fi(octave: Octave, duration: Dur) -> Music<BluesPitch> {
        Music::note(duration, BluesPitch::Fifth(octave))
    }

    pub const fn ms(octave: Octave, duration: Dur) -> Music<BluesPitch> {
        Music::note(duration, BluesPitch::MinorSeven(octave))
    }

    fn blues_into_western(blues_music: Music<BluesPitch>) -> Music {
        blues_music.map(BluesPitch::to_western)
    }

    pub fn melody() -> Music {
        let oc = Octave::OneLined;
        let blues_melody = (ro(oc, Dur::QUARTER) | ms(oc, Dur::QUARTER))
            + (mt(oc, Dur::HALF) | fi(oc, Dur::HALF) | fo(oc, Dur::HALF));
        blues_into_western(blues_melody)
    }
}

/// Exercise 2.3 Show that abspitch (pitch ap) = ap, and, up to enharmonic
/// equivalences, pitch (abspitch p) = p.
#[test]
fn from_abs_roundtrip() {
    for p in 0..=127 {
        let abs_pitch = AbsPitch::from(u7::new(p));
        assert_eq!(Pitch::from(abs_pitch).abs(), abs_pitch);
    }
}

#[test]
fn enharmonic_roundtrip_with_abs_conversion() {
    for pitch_constructor in &[
        Pitch::C,
        Pitch::Cs,
        Pitch::D,
        Pitch::Ds,
        Pitch::E,
        Pitch::F,
        Pitch::Fs,
        Pitch::G,
        Pitch::Gs,
        Pitch::A,
        Pitch::As,
        Pitch::B,
    ] {
        let oc = Octave::try_from(u4::new(3)).unwrap();
        let pitch = pitch_constructor(oc);
        assert_eq!(Pitch::from(pitch.abs()), pitch);
    }
}

#[test]
/// Exercise 2.4 Show that trans i (trans j p) = trans (i + j ) p.
fn trans_is_sum() {
    for pitch_constructor in &[
        Pitch::C,
        Pitch::Cs,
        Pitch::D,
        Pitch::Ds,
        Pitch::E,
        Pitch::F,
        Pitch::Fs,
        Pitch::G,
        Pitch::Gs,
        Pitch::A,
        Pitch::As,
        Pitch::B,
    ] {
        let oc = Octave::try_from(u4::new(3)).unwrap();
        let pitch = pitch_constructor(oc);
        for i in 0..12 {
            let int1 = Interval::from(i);
            for j in 0..12 {
                let int2 = Interval::from(j);
                assert_eq!(pitch.trans(int2).trans(int1), pitch.trans(int1 + int2));
            }
        }
    }
}
