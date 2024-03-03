use num_rational::Ratio;

use musik::{midi::Instrument, Dur, Music, Octave, Pitch};

type M = Music;
type P = Pitch;

/// Excerpt from Chick Corea’s Children’s Songs No. 6
fn child_song_6() -> Music {
    let oc3 = Octave::SMALL;
    let oc4 = Octave::ONE_LINED;

    let b1 = M::with_dur(
        vec![P::B(oc3), P::Fs(oc4), P::G(oc4), P::Fs(oc4)],
        Dur::QUARTER,
    );
    let b2 = M::with_dur(
        vec![P::B(oc3), P::Es(oc4), P::Fs(oc4), P::Es(oc4)],
        Dur::QUARTER,
    );
    let b3 = M::with_dur(
        vec![P::As(oc3), P::Fs(oc4), P::G(oc4), P::Fs(oc4)],
        Dur::QUARTER,
    );
    let bass_line = b1.times(3) + b2.times(2) + b3.times(4) + b1.times(5);

    let oc5 = Octave::TWO_LINED;
    let v1a = M::with_dur(
        vec![
            P::A(oc5),
            P::E(oc5),
            P::D(oc5),
            P::Fs(oc5),
            P::Cs(oc5),
            P::B(oc5),
            P::E(oc5),
            P::B(oc4),
        ],
        Dur::EIGHTH,
    );
    let v1b = M::with_dur(vec![P::Cs(oc5), P::B(oc4)], Dur::EIGHTH);

    let default_grace_note_fraction = Ratio::new(1, 8);
    let v1 = v1a
        + M::D(oc5, Dur::QUARTER)
            .grace_note((-1).into(), default_grace_note_fraction)
            .unwrap()
        + v1b; // bars 1-2

    let oc6 = Octave::THREE_LINED;
    let v2 = vec![
        // bars 7-11
        M::line(vec![
            M::Cs(oc5, Dur::DOTTED_HALF),
            M::Cs(oc5, Dur::DOTTED_HALF),
            M::D(oc5, Dur::DOTTED_HALF),
            M::F(oc5, Dur::HALF),
            M::Gs(oc5, Dur::QUARTER),
            M::Fs(oc5, Dur::HALF),
            M::Fs(oc5, Dur::EIGHTH),
            M::G(oc5, Dur::EIGHTH),
        ]),
        // bars 12-13
        M::with_dur(
            vec![P::Fs(oc5), P::E(oc5), P::Cs(oc5), P::As(oc4)],
            Dur::EIGHTH,
        ) + M::A(oc4, Dur::DOTTED_QUARTER)
            + M::with_dur(
                vec![P::As(oc4), P::Cs(oc5), P::Fs(oc5), P::E(oc5), P::Fs(oc5)],
                Dur::EIGHTH,
            ),
        // bars 14-16
        M::line(vec![
            M::G(oc5, Dur::EIGHTH),
            M::As(oc5, Dur::EIGHTH),
            M::Cs(oc6, Dur::HALF),
            M::Cs(oc6, Dur::EIGHTH),
            M::D(oc6, Dur::EIGHTH),
            M::Cs(oc6, Dur::EIGHTH),
        ]) + M::E(oc5, Dur::EIGHTH)
            + M::rest(Dur::EIGHTH)
            + M::line(vec![
                M::As(oc5, Dur::EIGHTH),
                M::A(oc5, Dur::EIGHTH),
                M::G(oc5, Dur::EIGHTH),
                M::D(oc5, Dur::QUARTER),
                M::C(oc5, Dur::EIGHTH),
                M::Cs(oc5, Dur::EIGHTH),
            ]),
        // bars 17-18.5
        M::with_dur(
            vec![
                P::Fs(oc5),
                P::Cs(oc5),
                P::E(oc5),
                P::Cs(oc5),
                P::A(oc4),
                P::As(oc4),
                P::D(oc5),
                P::E(oc5),
                P::Fs(oc5),
            ],
            Dur::EIGHTH,
        ),
        // bars 18.5-20
        M::line(vec![
            M::E(oc5, Dur::QUARTER)
                .grace_note(2.into(), default_grace_note_fraction)
                .unwrap(),
            M::D(oc5, Dur::EIGHTH),
            M::D(oc5, Dur::QUARTER)
                .grace_note(2.into(), default_grace_note_fraction)
                .unwrap(),
            M::Cs(oc5, Dur::EIGHTH),
            M::Cs(oc5, Dur::QUARTER)
                .grace_note(1.into(), default_grace_note_fraction)
                .unwrap(),
            M::B(oc4, Dur::EIGHTH),
            M::B(oc4, Dur::HALF),
            M::Cs(oc5, Dur::EIGHTH),
            M::B(oc4, Dur::EIGHTH),
        ]),
        // bars 21-23
        M::line(vec![
            M::Fs(oc5, Dur::EIGHTH),
            M::A(oc5, Dur::EIGHTH),
            M::B(oc5, Dur::HALF),
            M::B(oc5, Dur::QUARTER),
            M::A(oc5, Dur::EIGHTH),
            M::Fs(oc5, Dur::EIGHTH),
            M::E(oc5, Dur::QUARTER),
            M::D(oc5, Dur::EIGHTH),
            M::Fs(oc5, Dur::EIGHTH),
            M::E(oc5, Dur::HALF),
            M::D(oc5, Dur::HALF),
            M::Fs(oc5, Dur::QUARTER),
        ]),
        // bars 24-28
        M::with_dur(vec![P::Cs(oc5), P::D(oc5), P::Cs(oc5)], Dur::EIGHTH)
            .with_tempo(Ratio::new(3, 2))
            + M::B(oc4, Dur::DOTTED_HALF).times(3)
            + M::B(oc4, Dur::HALF),
    ];
    let main_voice = v1.times(3) + M::line(v2);
    let t = (Dur::DOTTED_HALF.into_ratio() / Dur::QUARTER.into_ratio()) * Ratio::new(69, 120);
    (bass_line | main_voice)
        .with_tempo(t)
        .with_instrument(Instrument::RhodesPiano)
}

// TODO: Exercise 4.1
//  https://musescore.com/bntt-piano/moonlight-sonata

fn prefixes<T: Clone>(xs: Vec<T>) -> Vec<Vec<T>> {
    xs.into_iter()
        .scan(vec![], |pref, x| {
            pref.push(x);
            Some(pref.clone())
        })
        .collect()
}

#[test]
fn test_prefix() {
    let v = (1..=5).collect();
    assert_eq!(
        prefixes(v),
        [
            vec![1],
            vec![1, 2],
            vec![1, 2, 3],
            vec![1, 2, 3, 4],
            vec![1, 2, 3, 4, 5],
        ]
    );
}

fn prefix<P: Clone>(mel: Vec<Music<P>>) -> Music<P> {
    let m1 = Music::line(prefixes(mel.clone()).into_iter().flatten().collect());
    let m2 = Music::line(
        prefixes(mel.into_iter().rev().collect())
            .into_iter()
            .flatten()
            .collect(),
    )
    .with_transpose(12.into());
    let m = m1.with_instrument(Instrument::Flute) | m2.with_instrument(Instrument::VoiceOohs);
    m.clone() + m.clone().with_transpose(5.into()) + m
}

fn prefixed_mel_1() -> Music {
    let oc4 = Octave::ONE_LINED;
    let oc5 = Octave::TWO_LINED;
    prefix(vec![
        M::C(oc5, Dur::EIGHTH),
        M::E(oc5, Dur::SIXTEENTH),
        M::G(oc5, Dur::EIGHTH),
        M::B(oc5, Dur::SIXTEENTH),
        M::A(oc5, Dur::EIGHTH),
        M::F(oc5, Dur::SIXTEENTH),
        M::D(oc5, Dur::EIGHTH),
        M::B(oc4, Dur::SIXTEENTH),
        M::C(oc5, Dur::EIGHTH),
    ])
}

fn prefixed_mel_2() -> Music {
    let oc4 = Octave::ONE_LINED;
    let oc5 = Octave::TWO_LINED;
    prefix(vec![
        M::C(oc5, Dur::SIXTEENTH),
        M::E(oc5, Dur::SIXTEENTH),
        M::G(oc5, Dur::SIXTEENTH),
        M::B(oc5, Dur::SIXTEENTH),
        M::A(oc5, Dur::SIXTEENTH),
        M::F(oc5, Dur::SIXTEENTH),
        M::D(oc5, Dur::SIXTEENTH),
        M::B(oc4, Dur::SIXTEENTH),
        M::C(oc5, Dur::SIXTEENTH),
    ])
}

// TODO: Exercises 4.2, 4.3
