use num_rational::Ratio;

use musik::{midi::Instrument, Dur, Music, Octave, Pitch};

type M = Music;
type P = Pitch;

/// Excerpt from Chick Corea’s Children’s Songs No. 6
#[allow(clippy::too_many_lines, clippy::similar_names)]
pub fn child_song_6() -> Music {
    let oc3 = Octave::Small;
    let oc4 = Octave::OneLined;

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

    let oc5 = Octave::TwoLined;
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

    let oc6 = Octave::ThreeLined;
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

#[allow(clippy::cognitive_complexity, clippy::similar_names)]
/// Exercise 4.1
///
/// First 15 bars of the Beethohen's Moonlight Sonata.
///
/// See the full score at <https://musescore.com/bntt-piano/moonlight-sonata>
pub fn moonlight() -> Music {
    // C#, D#, F#, G#
    use musik::{
        attributes::{Dynamic, StdLoudness},
        dur, m,
        music::Control,
        n, p, Interval, Music as M, PhraseAttribute,
    };

    fn twelfth<const N: usize>(start: Pitch, intervals: [i8; N]) -> Music {
        let pitches = start
            .get_scale(std::iter::once(0).chain(intervals))
            .collect();
        M::with_dur(pitches, dur!(1 / 12))
    }

    fn with_octave_lower(note @ (d, p): (Dur, Pitch)) -> Music {
        let lower = p << Interval::octave();
        M::from(note) | M::from((d, lower))
    }

    let m1 = twelfth(p!(G # 3), [5, 3]);
    let b1 = with_octave_lower(n!(C # 3 / 1));

    let b2 = with_octave_lower(n!(B 2 / 1));

    let m3 = twelfth(p!(A 3), [4, 3]) * 2 + twelfth(p!(A 3), [5, 4]) * 2;
    let b3 = with_octave_lower(n!(A 2 / 2)) + with_octave_lower(n!(F # 2 / 2));

    let m4 = twelfth(p!(G # 3), [4, 6])
        + twelfth(p!(G # 3), [5, 3])
        + twelfth(p!(G # 3), [5, 2])
        + twelfth(p!(F # 3), [6, 3]);
    let b4 = with_octave_lower(n!(G # 2 / 2)) * 2;

    let m5 = twelfth(p!(E 3), [4, 5])
        + twelfth(p!(G # 3), [5, 3]) * 2
        + (twelfth(p!(F # 3), [7, 4]) | m!( {G # 4 / .8}, {G # 4 / 16}));
    let b5 = with_octave_lower(n!(C # 3 / 1)) | m!(G # 2 / 1);

    let full_pp = vec![
        (m1.clone() * 4) | b1,
        (m1 * 4) | b2,
        m3 | b3,
        m4 | b4,
        m5 | b5,
    ];

    let m6 = ((twelfth(p!(G # 3), [7, 3]) * 3) | m!(G # 4 / .2))
        + (twelfth(p!(G # 3), [7, 3]) | m!({G # 4 / .8}, {G # 4 / 16}));
    let b6 = with_octave_lower(n!(B # 2 / 1)) | m!(G # 2 / 1);

    let m7 = ((twelfth(p!(G # 3), [5, 3]) * 2) | m!(G # 4 / 2))
        + ((twelfth(p!(A 3), [4, 5]) * 2) | m!(A 4 / 2));
    let b7 = with_octave_lower(n!(C # 3 / 2)) + with_octave_lower(n!(F # 2 / 2));

    let m8 = ((twelfth(p!(G # 3), [3, 5]) * 2) | m!(G # 4 / 2))
        + ((twelfth(p!(A 3), [2, 4]) * 2) | m!({F # 4 / 4}, {B 4 / 4}));
    let b8 = with_octave_lower(n!(B 2 / 2)) * 2;

    let m9 = (twelfth(p!(G # 3), [3, 5]) * 2) | m!(E 4 / 4);
    let b9 = with_octave_lower(n!(E 3 / 2));

    let m10 =
        (twelfth(p!(G 3), [4, 5]) * 3) + (twelfth(p!(G 3), [4, 5]) | m!({G 4 / .8}, {G 4 / 16}));
    let b10 = with_octave_lower(n!(E 3 / 1));

    let m11 = ((twelfth(p!(G 3), [4, 6]) * 3) | m!(G 4 / .2))
        + (twelfth(p!(G 3), [4, 6]) | m!({G 4 / .8}, {G 4 / 16}));
    let b11 = with_octave_lower(n!(D 3 / 1));

    let m12 = (((twelfth(p!(G 3), [5, 4]) * 2) + twelfth(p!(G 3), [6, 3])) | m!(G 4 / .2))
        + (twelfth(p!(F # 3), [7, 3]) | m!(F # 4 / 4));
    let b12 = with_octave_lower(n!(C 3 / 4))
        + with_octave_lower(n!(B 2 / 4))
        + with_octave_lower(n!(A # 2 / 2));

    let m13 = ((twelfth(p!(F # 3), [5, 3]) * 2) | m!(F # 4 / 2))
        + (twelfth(p!(G 3), [4, 2]) | m!(G 4 / 4))
        + (twelfth(p!(E 3), [7, 2]) | m!(E 4 / 4));
    let b13 = with_octave_lower(n!(B 2 / 2)) + m!({E 2 / 4}, {G 2 / 4});

    let m14 = ((twelfth(p!(F # 3), [5, 3]) * 2) | m!(F # 4 / 2))
        + ((twelfth(p!(F # 3), [4, 3]) * 2) | m!(F # 4 / 2));
    let b14 = m!(F 2 / 2) + with_octave_lower(n!(F 2 / 2));

    let m15 = (twelfth(p!(B 3), [3, 4]) * 2)
        + twelfth(p!(B 3), [4, 3])
        + (twelfth(p!(B 3), [4, 3]) | m!(B 4 / 4));
    let b15 = with_octave_lower(n!(B 2 / 1));

    let full_p = vec![
        m6 | b6,
        m7 | b7,
        m8 | b8,
        m9 | b9,
        m10 | b10,
        m11 | b11,
        m12 | b12,
        m13 | b13,
        m14 | b14,
        m15 | b15,
    ];

    let pp = PhraseAttribute::Dyn(Dynamic::StdLoudness(StdLoudness::Pianissimo));
    let p = PhraseAttribute::Dyn(Dynamic::StdLoudness(StdLoudness::Piano));
    let full = (M::line(full_pp) & Control::Phrase(vec![pp]))
        + (M::line(full_p) & Control::Phrase(vec![p]));

    // let full = musik::Temporal::skip(full, Dur::new(17, 2));
    full & Control::Tempo(Ratio::new(56, 120))
}

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

pub fn prefixed_mel_1() -> Music {
    let oc4 = Octave::OneLined;
    let oc5 = Octave::TwoLined;
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

pub fn prefixed_mel_2() -> Music {
    let oc4 = Octave::OneLined;
    let oc5 = Octave::TwoLined;
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
