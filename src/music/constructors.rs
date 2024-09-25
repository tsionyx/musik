use crate::prim::{duration::Dur, interval::Octave, pitch::Pitch};

use super::{Music, Primitive};

macro_rules! def_note_constructor {
    ($pitch: ident) => {
        #[allow(non_snake_case)]
        #[doc="Helper function for defining a [`Pitch`] value of [`PitchClass::"]
        #[doc = stringify!($pitch)]
        #[doc="`] with given [`Octave`] and [`Dur`]"]
        // #[doc = concat!("Helper function for defining a primitive [`Music`] value of ",
        //     stringify!($pitch),
        //     " with given [`Octave`] and [`Dur`]"
        // )]
        pub const fn $pitch(octave: Octave, duration: Dur) -> Self {
            Self::note(duration, Pitch::$pitch(octave))
        }
    };

    ( $( $pitch: ident ),+ $(,)? ) => {
        $(
            def_note_constructor!($pitch);
        )+
    }
}

impl Music {
    def_note_constructor![Aff, Af, A, As, Ass];
    def_note_constructor![Bff, Bf, B, Bs, Bss];
    def_note_constructor![Cff, Cf, C, Cs, Css];
    def_note_constructor![Dff, Df, D, Ds, Dss];
    def_note_constructor![Eff, Ef, E, Es, Ess];
    def_note_constructor![Fff, Ff, F, Fs, Fss];
    def_note_constructor![Gff, Gf, G, Gs, Gss];
}

/// Defines [rest][super::Primitive::Rest] constants of [`Music`]
/// for all the common [durations][Dur].
pub mod rests {
    use super::{Dur, Music};

    macro_rules! def_rest {
        ($rest_name: ident) => {
            #[doc = "[`Music`] silent value of the length of"]
            #[doc = stringify!($rest_name)]
            #[doc = "note(s)"]
            pub const $rest_name: Music = Music::rest(Dur::$rest_name);
        };
    }

    def_rest!(LONGA);
    def_rest!(BREVIS);
    def_rest!(WHOLE);
    def_rest!(HALF);
    def_rest!(QUARTER);
    def_rest!(EIGHTH);
    def_rest!(SIXTEENTH);
    def_rest!(THIRTY_SECOND);
    def_rest!(SIXTY_FOURTH);
    def_rest!(DOTTED_WHOLE);
    def_rest!(DOTTED_HALF);
    def_rest!(DOTTED_QUARTER);
    def_rest!(DOTTED_EIGHTH);
    def_rest!(DOTTED_SIXTEENTH);
    def_rest!(DOTTED_THIRTY_SECOND);
    def_rest!(DOUBLE_DOTTED_HALF);
    def_rest!(DOUBLE_DOTTED_QUARTER);
    def_rest!(DOUBLE_DOTTED_EIGHTH);
}

/// Reference pitch for tuning, corresponding to 440 Hz
///
/// See more: <https://en.wikipedia.org/wiki/A440_(pitch_standard)>
pub const A440: Music = Music::A(Octave::OneLined, Dur::WHOLE);

impl<P> Music<P> {
    /// Construct a primitive single-noted [`Music`] of a given duration.
    pub const fn note(duration: Dur, key: P) -> Self {
        Self::Prim(Primitive::Note(duration, key))
    }

    /// Construct a silent piece of [`Music`] of a given duration.
    pub const fn rest(duration: Dur) -> Self {
        Self::Prim(Primitive::Rest(duration))
    }

    /// Construct a sequence of [`Music`]al notes of the same duration.
    pub fn with_dur(keys: Vec<P>, dur: Dur) -> Self {
        Self::line(
            keys.into_iter()
                .map(|pitch| Self::note(dur, pitch))
                .collect(),
        )
    }

    /// Construct a sequence of [`Music`]al notes of the same duration.
    pub fn with_dur_lazy<I>(keys: I, dur: Dur) -> Self
    where
        P: Clone,
        I: Iterator<Item = P> + Clone + 'static,
    {
        Self::lazy_line(keys.map(move |pitch| Self::note(dur, pitch)))
    }
}
