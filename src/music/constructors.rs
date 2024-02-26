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

    def_rest!(BN);
    def_rest!(WN);
    def_rest!(HN);
    def_rest!(QN);
    def_rest!(EN);
    def_rest!(SN);
    def_rest!(TN);
    def_rest!(SFN);
    def_rest!(DWN);
    def_rest!(DHN);
    def_rest!(DQN);
    def_rest!(DEN);
    def_rest!(DSN);
    def_rest!(DTN);
    def_rest!(DDHN);
    def_rest!(DDQN);
    def_rest!(DDEN);
}

pub const A440: Music = Music::A(Octave(4), Dur::WN);

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
