use crate::prim::{duration::Dur, interval::Octave, pitch::Pitch};

use super::{Music, Primitive};

macro_rules! def_note_constructor {
    ($pitch: ident) => {
        #[allow(non_snake_case)]
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

pub mod rests {
    use crate::prim::duration::Dur;

    use super::super::Music;

    macro_rules! def_rest {
        ($rest_name: ident) => {
            pub const $rest_name: Music = Music::rest(Dur::$rest_name);
        };
    }

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

pub const A440: Music = Music::A(Octave::OneLined, Dur::WHOLE);

impl<P> Music<P> {
    pub const fn note(duration: Dur, pitch: P) -> Self {
        Self::Prim(Primitive::Note(duration, pitch))
    }

    pub const fn rest(duration: Dur) -> Self {
        Self::Prim(Primitive::Rest(duration))
    }

    pub fn with_dur(pitches: Vec<P>, dur: Dur) -> Self {
        Self::line(
            pitches
                .into_iter()
                .map(|pitch| Self::note(dur, pitch))
                .collect(),
        )
    }
}
