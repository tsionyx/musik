//! Helper macros for creating notes using simplified notations:
//! - including duration as its name;
//! - duration as rhythm as in [GMN](https://wiki.ccarh.org/wiki/Guido_Music_Notation);
//! - rests as in [GMN];

#[macro_export]
/// Create a note as a combination of
/// a [Pitch][crate::Pitch] and a [Duration][crate::Duration].
macro_rules! n {
    // https://wiki.ccarh.org/wiki/Guido_Music_Notation#Rests
    (_/ $rhythm:expr) => {{
        const _: () = assert!(
            ($rhythm == 1) ||
            ($rhythm == 2) ||
            ($rhythm == 4) ||
            ($rhythm == 8) ||
            ($rhythm == 16) ||
            ($rhythm == 32) ||
            ($rhythm == 64)
        );
        match $rhythm {
            1 => $crate::Dur::WHOLE,
            2 => $crate::Dur::HALF,
            4 => $crate::Dur::QUARTER,
            8 => $crate::Dur::EIGHTH,
            16 => $crate::Dur::SIXTEENTH,
            32 => $crate::Dur::THIRTY_SECOND,
            64 => $crate::Dur::SIXTY_FOURTH,
            _ => unreachable!("Invalid rhythm number: should be power of 2 up to 64"),
        }
    }};

    // TODO: dotted durations

    ($pitch:tt ## $octave:tt / $rhythm:expr) => {{
        let dur = $crate::n!(_/ $rhythm);
        let pc = $crate::p!($pitch ## $octave);
        (dur, pc)
    }};

    ($pitch:tt $accidental:tt $octave:tt / $rhythm:expr) => {{
        let dur = $crate::n!(_/ $rhythm);
        let pc = $crate::p!($pitch $accidental $octave);
        (dur, pc)
    }};

    ($pitch:tt $octave:tt / $rhythm:expr) => {{
        let dur = $crate::n!(_/ $rhythm);
        let pc = $crate::p!($pitch $octave);
        (dur, pc)
    }};

    ($pitch:tt ## $octave:tt ; $dur:ident) => {{
        let pc = $crate::p!($pitch ## $octave);
        ($crate::Dur::$dur, pc)
    }};

    ($pitch:tt $accidental:tt $octave:tt ; $dur:ident) => {{
        let pc = $crate::p!($pitch $accidental $octave);
        ($crate::Dur::$dur, pc)
    }};

    ($pitch:tt $octave:tt ; $dur:ident) => {{
        let pc = $crate::p!($pitch $octave);
        ($crate::Dur::$dur, pc)
    }};
}

#[cfg(test)]
mod tests {
    use crate::{Dur, Octave, Pitch, PitchClass};

    #[test]
    fn simple() {
        let n = n!(A 4 / 4);

        assert_eq!(n.0, Dur::QUARTER);
        assert_eq!(n.1, Pitch::new(PitchClass::A, Octave::OneLined));

        let n2 = n!(A 4; QUARTER);
        assert_eq!(n, n2);
    }

    #[test]
    fn sharp() {
        let n = n!(C # 4 / 4);
        assert_eq!(n.0, Dur::QUARTER);
        assert_eq!(n.1, Pitch::new(PitchClass::Cs, Octave::OneLined));

        let n2 = n!(C # 4; QUARTER);
        assert_eq!(n, n2);
    }

    #[test]
    fn double_sharp() {
        let n = n!(B ## 3 / 4);
        assert_eq!(n.0, Dur::QUARTER);
        assert_eq!(n.1, Pitch::new(PitchClass::Bss, Octave::Small));

        let n2 = n!(B ## 3; QUARTER);
        assert_eq!(n, n2);
    }

    #[test]
    fn flat() {
        let n = n!(D b 3 / 8);
        assert_eq!(n.0, Dur::EIGHTH);
        assert_eq!(n.1, Pitch::new(PitchClass::Df, Octave::Small));

        let n2 = n!(D b 3; EIGHTH);
        assert_eq!(n, n2);
    }

    #[test]
    fn double_flat() {
        let n = n!(G bb 5 / 4);
        assert_eq!(n.0, Dur::QUARTER);
        assert_eq!(n.1, Pitch::new(PitchClass::Gff, Octave::TwoLined));

        let n2 = n!(G bb 5; QUARTER);
        assert_eq!(n, n2);
    }

    #[test]
    fn all_durations() {
        let n = n!(_/ 1);
        assert_eq!(n, Dur::WHOLE);

        let n = n!(_/ 2);
        assert_eq!(n, Dur::HALF);

        let n = n!(_/ 4);
        assert_eq!(n, Dur::QUARTER);

        let n = n!(_/ 8);
        assert_eq!(n, Dur::EIGHTH);

        let n = n!(_/ 16);
        assert_eq!(n, Dur::SIXTEENTH);

        let n = n!(_/ 32);
        assert_eq!(n, Dur::THIRTY_SECOND);

        let n = n!(_/ 64);
        assert_eq!(n, Dur::SIXTY_FOURTH);
    }

    #[test]
    fn all_durations_notes() {
        let n = n!(A 4 / 1);
        assert_eq!(n.0, Dur::WHOLE);
        assert_eq!(n.1, Pitch::new(PitchClass::A, Octave::OneLined));

        let n = n!(Bf 4 / 2);
        assert_eq!(n.0, Dur::HALF);
        assert_eq!(n.1, Pitch::new(PitchClass::Bf, Octave::OneLined));

        let n = n!(Cb 4 / 4);
        assert_eq!(n.0, Dur::QUARTER);
        assert_eq!(n.1, Pitch::new(PitchClass::Cf, Octave::OneLined));

        let n = n!(D b 4 / 8);
        assert_eq!(n.0, Dur::EIGHTH);
        assert_eq!(n.1, Pitch::new(PitchClass::Df, Octave::OneLined));

        let n = n!(Fbb 4 / 16);
        assert_eq!(n.0, Dur::SIXTEENTH);
        assert_eq!(n.1, Pitch::new(PitchClass::Fff, Octave::OneLined));

        let n = n!(F bb 4 / 16);
        assert_eq!(n.0, Dur::SIXTEENTH);
        assert_eq!(n.1, Pitch::new(PitchClass::Fff, Octave::OneLined));

        let n = n!(Gs 4 / 32);
        assert_eq!(n.0, Dur::THIRTY_SECOND);
        assert_eq!(n.1, Pitch::new(PitchClass::Gs, Octave::OneLined));

        let n = n!(G # 4 / 32);
        assert_eq!(n.0, Dur::THIRTY_SECOND);
        assert_eq!(n.1, Pitch::new(PitchClass::Gs, Octave::OneLined));

        let n = n!(Ass 4 / 64);
        assert_eq!(n.0, Dur::SIXTY_FOURTH);
        assert_eq!(n.1, Pitch::new(PitchClass::Ass, Octave::OneLined));

        let n = n!(A ## 4 / 64);
        assert_eq!(n.0, Dur::SIXTY_FOURTH);
        assert_eq!(n.1, Pitch::new(PitchClass::Ass, Octave::OneLined));
    }
}
