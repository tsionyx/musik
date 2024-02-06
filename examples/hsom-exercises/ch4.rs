use num_rational::Ratio;

use musik::{instruments::StandardMidiInstrument, rests, Dur, Music, Octave, Pitch};

type M = Music;
type P = Pitch;

/// Excerpt from Chick Corea’s Children’s Songs No. 6
fn child_song_6() -> Music {
    let oc3 = Octave::SMALL;
    let oc4 = Octave::ONE_LINED;

    let b1 = M::with_dur(vec![P::B(oc3), P::Fs(oc4), P::G(oc4), P::Fs(oc4)], Dur::QN);
    let b2 = M::with_dur(vec![P::B(oc3), P::Es(oc4), P::Fs(oc4), P::Es(oc4)], Dur::QN);
    let b3 = M::with_dur(vec![P::As(oc3), P::Fs(oc4), P::G(oc4), P::Fs(oc4)], Dur::QN);
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
        Dur::EN,
    );
    let v1b = M::with_dur(vec![P::Cs(oc5), P::B(oc4)], Dur::EN);

    let default_grace_note_fraction = Ratio::new(1, 8);
    let v1 = v1a
        + M::D(oc5, Dur::QN)
            .grace_note((-1).into(), default_grace_note_fraction)
            .unwrap()
        + v1b; // bars 1-2

    let oc6 = Octave::THREE_LINED;
    let v2 = vec![
        // bars 7-11
        M::line(vec![
            M::Cs(oc5, Dur::DHN),
            M::Cs(oc5, Dur::DHN),
            M::D(oc5, Dur::DHN),
            M::F(oc5, Dur::HN),
            M::Gs(oc5, Dur::QN),
            M::Fs(oc5, Dur::HN),
            M::Fs(oc5, Dur::EN),
            M::G(oc5, Dur::EN),
        ]),
        // bars 12-13
        M::with_dur(vec![P::Fs(oc5), P::E(oc5), P::Cs(oc5), P::As(oc4)], Dur::EN)
            + M::A(oc4, Dur::DQN)
            + M::with_dur(
                vec![P::As(oc4), P::Cs(oc5), P::Fs(oc5), P::E(oc5), P::Fs(oc5)],
                Dur::EN,
            ),
        // bars 14-16
        M::line(vec![
            M::G(oc5, Dur::EN),
            M::As(oc5, Dur::EN),
            M::Cs(oc6, Dur::HN),
            M::Cs(oc6, Dur::EN),
            M::D(oc6, Dur::EN),
            M::Cs(oc6, Dur::EN),
        ]) + M::E(oc5, Dur::EN)
            + rests::EN
            + M::line(vec![
                M::As(oc5, Dur::EN),
                M::A(oc5, Dur::EN),
                M::G(oc5, Dur::EN),
                M::D(oc5, Dur::QN),
                M::C(oc5, Dur::EN),
                M::Cs(oc5, Dur::EN),
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
            Dur::EN,
        ),
        // bars 18.5-20
        M::line(vec![
            M::E(oc5, Dur::QN)
                .grace_note(2.into(), default_grace_note_fraction)
                .unwrap(),
            M::D(oc5, Dur::EN),
            M::D(oc5, Dur::QN)
                .grace_note(2.into(), default_grace_note_fraction)
                .unwrap(),
            M::Cs(oc5, Dur::EN),
            M::Cs(oc5, Dur::QN)
                .grace_note(1.into(), default_grace_note_fraction)
                .unwrap(),
            M::B(oc4, Dur::EN),
            M::B(oc4, Dur::HN),
            M::Cs(oc5, Dur::EN),
            M::B(oc4, Dur::EN),
        ]),
        // bars 21-23
        M::line(vec![
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
        ]),
        // bars 24-28
        M::with_dur(vec![P::Cs(oc5), P::D(oc5), P::Cs(oc5)], Dur::EN).with_tempo(Ratio::new(3, 2))
            + M::B(oc4, Dur::DHN).times(3)
            + M::B(oc4, Dur::HN),
    ];
    let main_voice = v1.times(3) + M::line(v2);
    let t = (Dur::DHN.into_ratio() / Dur::QN.into_ratio()) * Ratio::new(69, 120);
    (bass_line | main_voice)
        .with_tempo(t)
        .with_instrument(StandardMidiInstrument::RhodesPiano)
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
    let m = m1.with_instrument(StandardMidiInstrument::Flute)
        | m2.with_instrument(StandardMidiInstrument::VoiceOohs);
    m.clone() + m.clone().with_transpose(5.into()) + m
}

fn prefixed_mel_1() -> Music {
    let oc4 = Octave::ONE_LINED;
    let oc5 = Octave::TWO_LINED;
    prefix(vec![
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
}

fn prefixed_mel_2() -> Music {
    let oc4 = Octave::ONE_LINED;
    let oc5 = Octave::TWO_LINED;
    prefix(vec![
        M::C(oc5, Dur::SN),
        M::E(oc5, Dur::SN),
        M::G(oc5, Dur::SN),
        M::B(oc5, Dur::SN),
        M::A(oc5, Dur::SN),
        M::F(oc5, Dur::SN),
        M::D(oc5, Dur::SN),
        M::B(oc4, Dur::SN),
        M::C(oc5, Dur::SN),
    ])
}

// TODO: Exercises 4.2, 4.3
