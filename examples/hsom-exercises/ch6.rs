use musik::{Dur, Music, Octave};

type M = Music;

/// Exercise 6.1
/// Show that `retro ◦ retro`, `invert ◦ invert`,
/// and `retroInvert ◦ invertRetro` are the identity on values created by `line`.
/// (You may use the lemma that `reverse (reverse l) = l`.)
#[cfg(test)]
mod retro_invert {
    use super::*;

    #[test]
    fn retro_is_involution() {
        let m = {
            let oc4 = Octave::ONE_LINED;
            let oc5 = Octave::TWO_LINED;
            Music::line(vec![
                M::C(oc5, Dur::EN),
                M::E(oc5, Dur::SN),
                M::G(oc5, Dur::EN),
                M::B(oc5, Dur::SN),
                M::A(oc5, Dur::EN),
                M::F(oc5, Dur::SN),
                M::D(oc5, Dur::EN),
                M::B(oc4, Dur::SN),
                M::C(oc5, Dur::EN),
            ])
        };

        assert_eq!(m.clone().retrograde().retrograde(), m);
    }

    #[test]
    fn invert_is_involution() {
        let m = {
            let oc5 = Octave::TWO_LINED;
            Music::line(vec![
                M::Fs(oc5, Dur::EN),
                M::A(oc5, Dur::EN),
                M::B(oc5, Dur::HN),
                M::B(oc5, Dur::QN),
                M::A(oc5, Dur::EN),
                M::Fs(oc5, Dur::EN),
                M::E(oc5, Dur::QN),
                M::D(oc5, Dur::EN),
                M::Fs(oc5, Dur::EN),
                M::E(oc5, Dur::HN),
                M::D(oc5, Dur::HN),
                M::Fs(oc5, Dur::QN),
            ])
        };

        assert_eq!(m.clone().invert().invert(), m);
    }

    #[test]
    fn invert_retro_is_inverse_to_retro_invert() {
        let m = {
            let oc5 = Octave::TWO_LINED;
            let oc6 = Octave::THREE_LINED;
            Music::line(vec![
                M::G(oc5, Dur::EN),
                M::As(oc5, Dur::EN),
                M::Cs(oc6, Dur::HN),
                M::Cs(oc6, Dur::EN),
                M::D(oc6, Dur::EN),
                M::Cs(oc6, Dur::EN),
            ])
        };

        assert_eq!(m.clone().invert_retro().retro_invert(), m);
    }
}
