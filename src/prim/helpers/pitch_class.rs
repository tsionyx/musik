//! Helper functions for creating pitch classes using standard notation.

#[macro_export]
/// Create a [`PitchClass`][crate::PitchClass]
/// using standard notation:
///
/// ```
/// # use musik::{pc, PitchClass};
/// assert_eq!(pc!(C #), PitchClass::Cs);
/// assert_eq!(pc!(F ##), PitchClass::Fss);
/// assert_eq!(pc!(B b), PitchClass::Bf);
/// assert_eq!(pc!(G bb), PitchClass::Gff);
macro_rules! pc {
    ($name:ident) => {{
        const _: () = assert!(
            stringify!($name).len() == 1,
            "Pitch name must be a single character. Better use PitchClass::$name directly",
        );

        $crate::PitchClass::$name
    }};

    ($name:ident #) => {
        $crate::accidentals::Sharp::$name()
    };

    ($name:ident ##) => {
        $crate::accidentals::DoubleSharp::$name()
    };

    ($name: ident b) => {
        $crate::accidentals::Flat::$name()
    };

    ($name: ident bb) => {
        $crate::accidentals::DoubleFlat::$name()
    };
}

/// Introduce some constant aliases that allows to define
/// [`PitchClass`][crate::PitchClass]
/// in a more standard way, e.g.:
/// - Cb for [`crate::PitchClass::Cf`];
/// - Bbb for [`crate::PitchClass::Bff`];
///
///  TODO: create sharp aliases the similar way.
///    This is challenging since the '#' symbol
///    cannot be used in identifiers.
pub mod aliases {
    macro_rules! def_pitch_class_alias {
        ($name: ident -> $pitch_class: ident) => {
             const _: () = assert!(
                stringify!($name).len() > 1,
                "Pitch alias should not be a single character.",
            );

            #[allow(non_upper_case_globals)]
            #[doc="An alias for defining a [`PitchClass::"]
            #[doc = stringify!($pitch_class)]
            #[doc="`]"]
            pub const $name: $crate::PitchClass = $crate::PitchClass::$pitch_class;
        };

        ( $( $name: ident -> $pitch_class: ident ),+ $(,)? ) => {
            $(
                def_pitch_class_alias!($name -> $pitch_class);
            )+
        }
    }

    def_pitch_class_alias![
        Cb -> Cf,
        Db -> Df,
        Eb -> Ef,
        Fb -> Ff,
        Gb -> Gf,
        Ab -> Af,
        Bb -> Bf,

        Cbb -> Cff,
        Dbb -> Dff,
        Ebb -> Eff,
        Fbb -> Fff,
        Gbb -> Gff,
        Abb -> Aff,
        Bbb -> Bff,
    ];
}

/// Define transformation to the provided [`PitchClass`][crate::PitchClass].
pub mod accidentals {
    use super::super::super::pitch::PitchClass;

    #[derive(Debug, Copy, Clone)]
    /// Raise the note by a semitone.
    pub struct Sharp;

    #[derive(Debug, Copy, Clone)]
    /// Raise the note by a whole tone.
    pub struct DoubleSharp;

    #[derive(Debug, Copy, Clone)]
    /// Lower the note by a semitone.
    pub struct Flat;

    #[derive(Debug, Copy, Clone)]
    /// Lower the note by a whole tone.
    pub struct DoubleFlat;

    macro_rules! impl_pitch_classes {
        ($accidental:ident: $($from:ident -> $to:ident),+ $(,)?) => {
            #[allow(non_snake_case)]
            impl $accidental {
            $(
                #[doc="Apply [`"]
                #[doc = stringify!($accidental)]
                #[doc="`] transformation to a [`PitchClass::"]
                #[doc = stringify!($from)]
                #[doc="`] to create a [`PitchClass::"]
                #[doc = stringify!($to)]
                #[doc="`] from it."]
                pub const fn $from() -> PitchClass {
                    PitchClass::$to
                }
            )+
            }
        };
    }

    impl_pitch_classes!(
        Sharp:
        C -> Cs,
        D -> Ds,
        E -> Es,
        F -> Fs,
        G -> Gs,
        A -> As,
        B -> Bs,
    );

    impl_pitch_classes!(
        DoubleSharp:
        C -> Css,
        D -> Dss,
        E -> Ess,
        F -> Fss,
        G -> Gss,
        A -> Ass,
        B -> Bss,
    );

    impl_pitch_classes!(
        Flat:
        C -> Cf,
        D -> Df,
        E -> Ef,
        F -> Ff,
        G -> Gf,
        A -> Af,
        B -> Bf,
    );

    impl_pitch_classes!(
        DoubleFlat:
        C -> Cff,
        D -> Dff,
        E -> Eff,
        F -> Fff,
        G -> Gff,
        A -> Aff,
        B -> Bff,
    );
}

#[cfg(test)]
mod tests {
    use super::super::super::pitch::PitchClass;

    use super::aliases::*;

    #[test]
    fn plain() {
        assert_eq!(pc!(C), PitchClass::C);
        assert_eq!(pc!(D), PitchClass::D);
        assert_eq!(pc!(E), PitchClass::E);
        assert_eq!(pc!(F), PitchClass::F);
        assert_eq!(pc!(G), PitchClass::G);
        assert_eq!(pc!(A), PitchClass::A);
        assert_eq!(pc!(B), PitchClass::B);
    }

    #[test]
    fn sharps() {
        assert_eq!(pc!(C #), PitchClass::Cs);
        assert_eq!(pc!(D #), PitchClass::Ds);
        assert_eq!(pc!(E #), PitchClass::Es);
        assert_eq!(pc!(F #), PitchClass::Fs);
        assert_eq!(pc!(G #), PitchClass::Gs);
        assert_eq!(pc!(A #), PitchClass::As);
        assert_eq!(pc!(B #), PitchClass::Bs);
    }

    #[test]
    fn double_sharps() {
        assert_eq!(pc!(C ##), PitchClass::Css);
        assert_eq!(pc!(D ##), PitchClass::Dss);
        assert_eq!(pc!(E ##), PitchClass::Ess);
        assert_eq!(pc!(F ##), PitchClass::Fss);
        assert_eq!(pc!(G ##), PitchClass::Gss);
        assert_eq!(pc!(A ##), PitchClass::Ass);
        assert_eq!(pc!(B ##), PitchClass::Bss);
    }

    #[test]
    fn flats() {
        assert_eq!(pc!(C b), PitchClass::Cf);
        assert_eq!(pc!(D b), PitchClass::Df);
        assert_eq!(pc!(E b), PitchClass::Ef);
        assert_eq!(pc!(F b), PitchClass::Ff);
        assert_eq!(pc!(G b), PitchClass::Gf);
        assert_eq!(pc!(A b), PitchClass::Af);
        assert_eq!(pc!(B b), PitchClass::Bf);
    }

    #[test]
    fn double_flats() {
        assert_eq!(pc!(C bb), PitchClass::Cff);
        assert_eq!(pc!(D bb), PitchClass::Dff);
        assert_eq!(pc!(E bb), PitchClass::Eff);
        assert_eq!(pc!(F bb), PitchClass::Fff);
        assert_eq!(pc!(G bb), PitchClass::Gff);
        assert_eq!(pc!(A bb), PitchClass::Aff);
        assert_eq!(pc!(B bb), PitchClass::Bff);
    }

    #[test]
    fn flats_constants() {
        assert_eq!(Cb, PitchClass::Cf);
        assert_eq!(Db, PitchClass::Df);
        assert_eq!(Eb, PitchClass::Ef);
        assert_eq!(Fb, PitchClass::Ff);
        assert_eq!(Gb, PitchClass::Gf);
        assert_eq!(Ab, PitchClass::Af);
        assert_eq!(Bb, PitchClass::Bf);
    }

    #[test]
    fn double_flats_constants() {
        assert_eq!(Cbb, PitchClass::Cff);
        assert_eq!(Dbb, PitchClass::Dff);
        assert_eq!(Ebb, PitchClass::Eff);
        assert_eq!(Fbb, PitchClass::Fff);
        assert_eq!(Gbb, PitchClass::Gff);
        assert_eq!(Abb, PitchClass::Aff);
        assert_eq!(Bbb, PitchClass::Bff);
    }
}
