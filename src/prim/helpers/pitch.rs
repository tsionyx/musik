//! Helper macro for creating pitches using standard notation.

#[macro_export]
/// Create a [`Pitch`][crate::Pitch]
/// using standard notations:
///
/// ```
/// # use musik::{p, Pitch, PitchClass, Octave};
/// assert_eq!(p!(C 4), Pitch::new(PitchClass::C, Octave::OneLined));
///
/// assert_eq!(p!(Cs 5), Pitch::new(PitchClass::Cs, Octave::TwoLined));
/// assert_eq!(p!(C # 5), Pitch::new(PitchClass::Cs, Octave::TwoLined));
///
/// assert_eq!(p!(Fss 2), Pitch::new(PitchClass::Fss, Octave::Great));
/// assert_eq!(p!(F ## 2), Pitch::new(PitchClass::Fss, Octave::Great));
///
/// assert_eq!(p!(Bf 3), Pitch::new(PitchClass::Bf, Octave::Small));
/// assert_eq!(p!(Bb 3), Pitch::new(PitchClass::Bf, Octave::Small));
/// assert_eq!(p!(B b 3), Pitch::new(PitchClass::Bf, Octave::Small));
///
/// assert_eq!(p!(Gff 6), Pitch::new(PitchClass::Gff, Octave::ThreeLined));
/// assert_eq!(p!(Gbb 6), Pitch::new(PitchClass::Gff, Octave::ThreeLined));
/// assert_eq!(p!(G bb 6), Pitch::new(PitchClass::Gff, Octave::ThreeLined));
macro_rules! p {
    // special case for double sharp (they are interpreted separately)
    ($name:ident ## $octave:expr) => {{
        let pc = $crate::pc!($name ##);
        $crate::p!(pc $octave)
    }};

    ($name:ident $accidental:tt $octave:expr) => {{
        let pc = $crate::pc!($name $accidental);
        $crate::p!(pc $octave)
    }};

    ($name:ident $octave:expr) => {{
        #[allow(unused_imports)]
        use $crate::{helpers::pitch_class::aliases::*, PitchClass::*};

        $crate::Pitch::new($name, $crate::o!($octave))
    }};
}

#[cfg(test)]
mod tests {
    use crate::pc;

    use super::super::{
        super::{
            interval::Octave,
            pitch::{Pitch, PitchClass},
        },
        pitch_class::aliases::*,
    };

    mod two_parts {
        use super::*;

        #[test]
        fn plain() {
            assert_eq!(p!(C 4), Pitch::new(PitchClass::C, Octave::OneLined));
            assert_eq!(p!(D 4), Pitch::new(PitchClass::D, Octave::OneLined));
            assert_eq!(p!(E 4), Pitch::new(PitchClass::E, Octave::OneLined));
            assert_eq!(p!(F 4), Pitch::new(PitchClass::F, Octave::OneLined));
            assert_eq!(p!(G 4), Pitch::new(PitchClass::G, Octave::OneLined));
            assert_eq!(p!(A 4), Pitch::new(PitchClass::A, Octave::OneLined));
            assert_eq!(p!(B 4), Pitch::new(PitchClass::B, Octave::OneLined));
        }

        #[test]
        fn standard_sharps() {
            assert_eq!(p!(Cs 4), Pitch::new(PitchClass::Cs, Octave::OneLined));
            assert_eq!(p!(Ds 4), Pitch::new(PitchClass::Ds, Octave::OneLined));
            assert_eq!(p!(Es 4), Pitch::new(PitchClass::Es, Octave::OneLined));
            assert_eq!(p!(Fs 4), Pitch::new(PitchClass::Fs, Octave::OneLined));
            assert_eq!(p!(Gs 4), Pitch::new(PitchClass::Gs, Octave::OneLined));
            assert_eq!(p!(As 4), Pitch::new(PitchClass::As, Octave::OneLined));
            assert_eq!(p!(Bs 4), Pitch::new(PitchClass::Bs, Octave::OneLined));
        }

        #[test]
        fn standard_double_sharps() {
            assert_eq!(p!(Css 4), Pitch::new(PitchClass::Css, Octave::OneLined));
            assert_eq!(p!(Dss 4), Pitch::new(PitchClass::Dss, Octave::OneLined));
            assert_eq!(p!(Ess 4), Pitch::new(PitchClass::Ess, Octave::OneLined));
            assert_eq!(p!(Fss 4), Pitch::new(PitchClass::Fss, Octave::OneLined));
            assert_eq!(p!(Gss 4), Pitch::new(PitchClass::Gss, Octave::OneLined));
            assert_eq!(p!(Ass 4), Pitch::new(PitchClass::Ass, Octave::OneLined));
            assert_eq!(p!(Bss 4), Pitch::new(PitchClass::Bss, Octave::OneLined));
        }

        #[test]
        fn standard_flats() {
            assert_eq!(p!(Cf 4), Pitch::new(PitchClass::Cf, Octave::OneLined));
            assert_eq!(p!(Df 4), Pitch::new(PitchClass::Df, Octave::OneLined));
            assert_eq!(p!(Ef 4), Pitch::new(PitchClass::Ef, Octave::OneLined));
            assert_eq!(p!(Ff 4), Pitch::new(PitchClass::Ff, Octave::OneLined));
            assert_eq!(p!(Gf 4), Pitch::new(PitchClass::Gf, Octave::OneLined));
            assert_eq!(p!(Af 4), Pitch::new(PitchClass::Af, Octave::OneLined));
            assert_eq!(p!(Bf 4), Pitch::new(PitchClass::Bf, Octave::OneLined));
        }

        #[test]
        fn custom_flats() {
            assert_eq!(p!(Cb 4), Pitch::new(PitchClass::Cf, Octave::OneLined));
            assert_eq!(p!(Db 4), Pitch::new(PitchClass::Df, Octave::OneLined));
            assert_eq!(p!(Eb 4), Pitch::new(PitchClass::Ef, Octave::OneLined));
            assert_eq!(p!(Fb 4), Pitch::new(PitchClass::Ff, Octave::OneLined));
            assert_eq!(p!(Gb 4), Pitch::new(PitchClass::Gf, Octave::OneLined));
            assert_eq!(p!(Ab 4), Pitch::new(PitchClass::Af, Octave::OneLined));
            assert_eq!(p!(Bb 4), Pitch::new(PitchClass::Bf, Octave::OneLined));
        }

        #[test]
        fn standard_double_flats() {
            assert_eq!(p!(Cff 4), Pitch::new(PitchClass::Cff, Octave::OneLined));
            assert_eq!(p!(Dff 4), Pitch::new(PitchClass::Dff, Octave::OneLined));
            assert_eq!(p!(Eff 4), Pitch::new(PitchClass::Eff, Octave::OneLined));
            assert_eq!(p!(Fff 4), Pitch::new(PitchClass::Fff, Octave::OneLined));
            assert_eq!(p!(Gff 4), Pitch::new(PitchClass::Gff, Octave::OneLined));
            assert_eq!(p!(Aff 4), Pitch::new(PitchClass::Aff, Octave::OneLined));
            assert_eq!(p!(Bff 4), Pitch::new(PitchClass::Bff, Octave::OneLined));
        }

        #[test]
        fn custom_double_flats() {
            assert_eq!(p!(Cbb 4), Pitch::new(PitchClass::Cff, Octave::OneLined));
            assert_eq!(p!(Dbb 4), Pitch::new(PitchClass::Dff, Octave::OneLined));
            assert_eq!(p!(Ebb 4), Pitch::new(PitchClass::Eff, Octave::OneLined));
            assert_eq!(p!(Fbb 4), Pitch::new(PitchClass::Fff, Octave::OneLined));
            assert_eq!(p!(Gbb 4), Pitch::new(PitchClass::Gff, Octave::OneLined));
            assert_eq!(p!(Abb 4), Pitch::new(PitchClass::Aff, Octave::OneLined));
            assert_eq!(p!(Bbb 4), Pitch::new(PitchClass::Bff, Octave::OneLined));
        }

        #[test]
        fn using_variables() {
            let pc = pc!(A);
            assert_eq!(p!(pc 4), Pitch::new(PitchClass::A, Octave::OneLined));

            let pc = Bb;
            assert_eq!(p!(pc 4), Pitch::new(PitchClass::Bf, Octave::OneLined));

            let pc = pc!(C #);
            assert_eq!(p!(pc 4), Pitch::new(PitchClass::Cs, Octave::OneLined));

            let pc = pc!(D ##);
            assert_eq!(p!(pc 4), Pitch::new(PitchClass::Dss, Octave::OneLined));

            let pc = pc!(G b);
            assert_eq!(p!(pc 4), Pitch::new(PitchClass::Gf, Octave::OneLined));

            let pc = pc!(A bb);
            assert_eq!(p!(pc 4), Pitch::new(PitchClass::Aff, Octave::OneLined));
        }

        #[test]
        fn all_octaves() {
            #![allow(unused_parens)]
            assert_eq!(p!(C(-1)), Pitch::new(PitchClass::C, Octave::OctoContra));
            assert_eq!(p!(Cs 0), Pitch::new(PitchClass::Cs, Octave::SubContra));
            assert_eq!(p!(D 1), Pitch::new(PitchClass::D, Octave::Contra));
            assert_eq!(p!(Ds 2), Pitch::new(PitchClass::Ds, Octave::Great));
            assert_eq!(p!(E 3), Pitch::new(PitchClass::E, Octave::Small));
            assert_eq!(p!(F 4), Pitch::new(PitchClass::F, Octave::OneLined));
            assert_eq!(p!(Fs 5), Pitch::new(PitchClass::Fs, Octave::TwoLined));
            assert_eq!(p!(G 6), Pitch::new(PitchClass::G, Octave::ThreeLined));
            assert_eq!(p!(Gs 7), Pitch::new(PitchClass::Gs, Octave::FourLined));
            assert_eq!(p!(A 8), Pitch::new(PitchClass::A, Octave::FiveLined));
            assert_eq!(p!(As 9), Pitch::new(PitchClass::As, Octave::SixLined));
            // assert_eq!(p!(B 10), Pitch::new(PitchClass::As, Octave::SevenLined));
        }
    }

    mod three_parts {
        use super::*;

        #[test]
        fn custom_sharps() {
            assert_eq!(p!(C # 4), Pitch::new(PitchClass::Cs, Octave::OneLined));
            assert_eq!(p!(D # 4), Pitch::new(PitchClass::Ds, Octave::OneLined));
            assert_eq!(p!(E # 4), Pitch::new(PitchClass::Es, Octave::OneLined));
            assert_eq!(p!(F # 4), Pitch::new(PitchClass::Fs, Octave::OneLined));
            assert_eq!(p!(G # 4), Pitch::new(PitchClass::Gs, Octave::OneLined));
            assert_eq!(p!(A # 4), Pitch::new(PitchClass::As, Octave::OneLined));
            assert_eq!(p!(B # 4), Pitch::new(PitchClass::Bs, Octave::OneLined));
        }

        #[test]
        fn custom_double_sharps() {
            assert_eq!(p!(C ## 4), Pitch::new(PitchClass::Css, Octave::OneLined));
            assert_eq!(p!(D ## 4), Pitch::new(PitchClass::Dss, Octave::OneLined));
            assert_eq!(p!(E ## 4), Pitch::new(PitchClass::Ess, Octave::OneLined));
            assert_eq!(p!(F ## 4), Pitch::new(PitchClass::Fss, Octave::OneLined));
            assert_eq!(p!(G ## 4), Pitch::new(PitchClass::Gss, Octave::OneLined));
            assert_eq!(p!(A ## 4), Pitch::new(PitchClass::Ass, Octave::OneLined));
            assert_eq!(p!(B ## 4), Pitch::new(PitchClass::Bss, Octave::OneLined));
        }

        #[test]
        fn custom_flats() {
            assert_eq!(p!(C b 4), Pitch::new(PitchClass::Cf, Octave::OneLined));
            assert_eq!(p!(D b 4), Pitch::new(PitchClass::Df, Octave::OneLined));
            assert_eq!(p!(E b 4), Pitch::new(PitchClass::Ef, Octave::OneLined));
            assert_eq!(p!(F b 4), Pitch::new(PitchClass::Ff, Octave::OneLined));
            assert_eq!(p!(G b 4), Pitch::new(PitchClass::Gf, Octave::OneLined));
            assert_eq!(p!(A b 4), Pitch::new(PitchClass::Af, Octave::OneLined));
            assert_eq!(p!(B b 4), Pitch::new(PitchClass::Bf, Octave::OneLined));
        }

        #[test]
        fn custom_double_flats() {
            assert_eq!(p!(C bb 4), Pitch::new(PitchClass::Cff, Octave::OneLined));
            assert_eq!(p!(D bb 4), Pitch::new(PitchClass::Dff, Octave::OneLined));
            assert_eq!(p!(E bb 4), Pitch::new(PitchClass::Eff, Octave::OneLined));
            assert_eq!(p!(F bb 4), Pitch::new(PitchClass::Fff, Octave::OneLined));
            assert_eq!(p!(G bb 4), Pitch::new(PitchClass::Gff, Octave::OneLined));
            assert_eq!(p!(A bb 4), Pitch::new(PitchClass::Aff, Octave::OneLined));
            assert_eq!(p!(B bb 4), Pitch::new(PitchClass::Bff, Octave::OneLined));
        }

        #[test]
        fn all_octaves() {
            assert_eq!(p!(C # -1), Pitch::new(PitchClass::Cs, Octave::OctoContra));
            assert_eq!(p!(D ## 0), Pitch::new(PitchClass::Dss, Octave::SubContra));
            assert_eq!(p!(G b 6), Pitch::new(PitchClass::Gf, Octave::ThreeLined));
            assert_eq!(p!(B bb 7), Pitch::new(PitchClass::Bff, Octave::FourLined));
        }
    }
}
