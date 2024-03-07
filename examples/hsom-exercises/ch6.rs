use std::collections::HashSet;

use num_rational::Ratio;

use musik::{
    instruments::InstrumentName,
    midi::{Instrument, PercussionSound},
    music::{rests, Primitive},
    Dur, Interval, Music, Octave, Pitch, TrillOptions, Volume,
};

type M = Music;

/// Exercise 6.1
/// Show that `retro ◦ retro`, `invert ◦ invert`,
/// and `retroInvert ◦ invertRetro` are the identity on values created by `line`.
/// (You may use the lemma that `reverse (reverse l) = l`.)
#[cfg(test)]
mod retro_invert {
    use musik::{Dur, Octave};

    use super::*;

    #[test]
    fn retro_is_involution() {
        let m = {
            let oc4 = Octave::OneLined;
            let oc5 = Octave::TwoLined;
            Music::line(vec![
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
        };

        assert_eq!(m.clone().retrograde().retrograde(), m);
    }

    #[test]
    fn invert_is_involution() {
        let m = {
            let oc5 = Octave::TwoLined;
            Music::line(vec![
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
            ])
        };

        assert_eq!(m.clone().invert().invert(), m);
    }

    #[test]
    fn invert_retro_is_inverse_to_retro_invert() {
        let m = {
            let oc5 = Octave::TwoLined;
            let oc6 = Octave::ThreeLined;
            Music::line(vec![
                M::G(oc5, Dur::EIGHTH),
                M::As(oc5, Dur::EIGHTH),
                M::Cs(oc6, Dur::HALF),
                M::Cs(oc6, Dur::EIGHTH),
                M::D(oc6, Dur::EIGHTH),
                M::Cs(oc6, Dur::EIGHTH),
            ])
        };

        assert_eq!(m.clone().invert_retro().retro_invert(), m);
    }
}

/// Exercise 6.2
/// Define a function `properRow :: Music Pitch -> Bool`
/// that determines whether or not its argument is a “proper” twelve-tone row,
/// meaning that: (a) it must have exactly twelve notes,
/// and (b) each unique pitch class is used exactly once (regardless of the octave).
/// Enharmonically equivalent pitch classes are not considered unique.
/// You may assume that the `Music Pitch` value is generated by the function `line`
/// but note that rests are allowed.
fn is_proper_row(m: Music) -> bool {
    let tones: Vec<_> = Vec::from(m)
        .into_iter()
        .map(|m| match m {
            Music::Prim(Primitive::Note(_, p)) => {
                let interval_with_c = (12 + Interval::from(p.class()).get_inner()) % 12;
                Some(Some(interval_with_c))
            }
            Music::Prim(Primitive::Rest(_)) => Some(None),
            _ => None,
        })
        .collect();

    if tones.contains(&None) {
        // an item is not a `Primitive`
        return false;
    }

    // ignore the rests
    let tones: Vec<_> = tones.into_iter().flatten().collect();
    if tones.len() != 12 {
        // proper row has size 12
        return false;
    }

    let uniq: HashSet<_> = tones.iter().collect();
    // proper row has all uniques PitchClass
    uniq.len() == tones.len()
}

/// Exercise 6.3
/// Define a function `palin :: Music Pitch -> Bool`
/// that determines whether or not a given line
/// (as generated by the `line` function) is a palindrome or not.
/// You should ignore rests, and disregard note durations —
/// the main question is whether or not the melody is a palindrome.
fn is_palindrome(m: Music) -> bool {
    let abs_pitches: Vec<_> = Vec::from(m)
        .into_iter()
        .map(|m| match m {
            Music::Prim(Primitive::Note(_, p)) => Some(Some(p.abs())),
            Music::Prim(Primitive::Rest(_)) => Some(None),
            _ => None,
        })
        .collect();

    if abs_pitches.contains(&None) {
        // an item is not a `Primitive`
        return false;
    }

    // ignore the rests
    let abs_pitches: Vec<_> = abs_pitches.into_iter().flatten().collect();
    abs_pitches.iter().copied().rev().collect::<Vec<_>>() == abs_pitches
}

/// Exercise 6.4
/// Define a function `retroPitches :: Music Pitch -> Music Pitch`
/// that reverses the pitches in a line, but maintains
/// the durations in the same order from beginning to end.
fn retro_pitches(m: Music) -> Option<Music> {
    let abs_pitches = Vec::from(m)
        .into_iter()
        .map(|m| {
            if let Music::Prim(p) = m {
                Some(p)
            } else {
                None
            }
        })
        .collect::<Option<Vec<_>>>()?;

    let (durations, pitches): (Vec<_>, Vec<_>) = abs_pitches
        .into_iter()
        .map(|p| match p {
            Primitive::Note(d, p) => (d, Some(p)),
            Primitive::Rest(d) => (d, None),
        })
        .unzip();

    #[allow(clippy::option_if_let_else)]
    let musics = pitches.into_iter().rev().zip(durations).map(|(p, d)| {
        if let Some(p) = p {
            Music::note(d, p)
        } else {
            Music::rest(d)
        }
    });

    Some(Music::line(musics.collect()))
}

#[cfg(test)]
mod tests {
    use musik::{Dur, Octave};

    use super::*;

    #[test]
    fn test_retro_pitches() {
        let oc4 = Octave::OneLined;
        let m = Music::line(vec![
            M::C(oc4, Dur::EIGHTH),
            M::rest(Dur::SIXTEENTH),
            M::D(oc4, Dur::QUARTER),
        ]);

        assert_eq!(
            retro_pitches(m).unwrap(),
            Music::line(vec![
                M::D(oc4, Dur::EIGHTH),
                M::rest(Dur::SIXTEENTH),
                M::C(oc4, Dur::QUARTER),
            ])
        );
    }

    #[test]
    fn strip_zeros() {
        let oc4 = Octave::OneLined;
        let m = M::C(oc4, Dur::EIGHTH) + M::D(oc4, Dur::EIGHTH).times(16);
        assert_eq!(
            m.drop(Dur::HALF).take(Dur::HALF).remove_zeros(),
            M::D(oc4, Dur::EIGHTH).times(4).remove_zeros()
        );
    }
}

// TODO: play me
fn stars_and_stripes() -> Music {
    type M = Music;

    let oc5 = Octave::TwoLined;
    let oc6 = Octave::ThreeLined;
    let oc7 = Octave::FourLined;

    let melody = Music::line(vec![
        // bar 1
        M::Bf(oc6, Dur::EIGHTH)
            .trill(Interval::tone(), TrillOptions::Count(5))
            .unwrap(),
        M::Ef(oc7, Dur::EIGHTH),
        M::Ef(oc6, Dur::EIGHTH),
        M::Ef(oc7, Dur::EIGHTH),
        // bar 2
        M::Bf(oc6, Dur::SIXTEENTH),
        M::C(oc7, Dur::SIXTEENTH),
        M::Bf(oc6, Dur::SIXTEENTH),
        M::G(oc6, Dur::SIXTEENTH),
        M::Ef(oc6, Dur::EIGHTH),
        M::Bf(oc5, Dur::EIGHTH),
        // bar 3
        M::Ef(oc6, Dur::SIXTEENTH),
        M::F(oc6, Dur::SIXTEENTH),
        M::G(oc6, Dur::SIXTEENTH),
        M::Af(oc6, Dur::SIXTEENTH),
        M::Bf(oc6, Dur::EIGHTH),
        M::Ef(oc7, Dur::EIGHTH),
        // bar 4
        M::Bf(oc6, Dur::QUARTER)
            .trill(Interval::tone(), Dur::THIRTY_SECOND)
            .unwrap(),
        M::Bf(oc6, Dur::SIXTEENTH),
        M::rest(Dur::DOTTED_EIGHTH),
    ]);

    melody.with_instrument(Instrument::Flute)
}

/// Exercise 6.6
///
/// Related to trills and grace notes in Western classical music
/// are the notions of `mordent`, `turn`, and `appoggiatura`.
///
/// <https://en.wikipedia.org/wiki/Ornament_(music)>
mod ornamentations {
    use super::*;

    fn mordent(music: Music, upper: bool) -> Result<Music, String> {
        if let Music::Prim(Primitive::Note(d, p)) = music {
            let other = if upper {
                Interval::tone()
            } else {
                -Interval::tone()
            };
            Ok(Music::line(vec![
                Music::note(d / 8, p),
                Music::note(d / 8, p.trans(other)),
                Music::note(d / 4, p),
                Music::note(d / 2, p),
            ]))
        } else {
            Err("Can only construct a mordent from a note".into())
        }
    }

    fn turn(music: Music, upper: bool) -> Result<Music, String> {
        if let Music::Prim(Primitive::Note(d, p)) = music {
            let other = if upper {
                Interval::tone()
            } else {
                -Interval::tone()
            };
            Ok(Music::line(vec![
                Music::note(d / 4, p.trans(other)),
                Music::note(d / 4, p),
                Music::note(d / 4, p.trans(-other)),
                Music::note(d / 4, p),
            ]))
        } else {
            Err("Can only construct a turn from a note".into())
        }
    }
}

// TODO: play me
fn funk_groove() -> Music {
    let p1 = PercussionSound::LowTom.note(Dur::QUARTER);
    let p2 = PercussionSound::AcousticSnare.note(Dur::EIGHTH);
    let m1 = Music::line(vec![
        p1.clone(),
        rests::QUARTER,
        p2.clone(),
        rests::QUARTER,
        p2.clone(),
        p1.clone(),
        p1,
        rests::QUARTER,
        p2,
        rests::EIGHTH,
    ]);
    let m2 = PercussionSound::ClosedHiHat
        .note(Dur::BREVIS)
        .roll(Dur::EIGHTH)
        .unwrap();

    (m1 | m2)
        .times(4)
        .take(Dur::from(8))
        .with_instrument(InstrumentName::Percussion)
        .with_tempo(3)
}

/// Exercise 6.7
/// Write a program that generates all of the General MIDI
/// percussion sounds, playing through each of them one at a time.
fn sequence_all_percussions() -> Music {
    let dur = Dur::QUARTER;
    Music::line(
        enum_iterator::all::<PercussionSound>()
            .map(|s| s.note(dur))
            .collect(),
    )
}

// TODO: Exercise 6.8

/// Exercise 6.8
///
/// TODO: test more at https://en.wikipedia.org/wiki/Drum_beat
///   https://www.songsterr.com/a/wsa/nirvana-in-bloom-drum-tab-s295
pub fn drum_pattern() -> Music {
    let m1 = PercussionSound::ClosedHiHat.note(Dur::QUARTER).times(4);
    let m2 = Music::rest(Dur::HALF) + PercussionSound::AcousticSnare.note(Dur::HALF);

    let m3 = (PercussionSound::ClosedHiHat.note(Dur::EIGHTH) + Music::rest(Dur::EIGHTH)).times(4);
    let m4 = Music::rest(Dur::HALF) + PercussionSound::AcousticSnare.note(Dur::QUARTER);

    ((m1 | m2) + (m3 | m4))
        .with_instrument(InstrumentName::Percussion)
        .with_tempo(Ratio::new(4, 3))
}

// TODO: play me
fn test_volume(vol: Volume) -> Music<(Pitch, Volume)> {
    type M = Music;

    let oc4 = Octave::OneLined;
    Music::line(vec![
        M::C(oc4, Dur::QUARTER),
        M::D(oc4, Dur::QUARTER),
        M::E(oc4, Dur::QUARTER),
        M::C(oc4, Dur::QUARTER),
    ])
    .with_volume(vol)
}

/// Exercise 6.9
/// Using mMap, define a function that
/// scales the volume of each note in `m` by the factor `s`.
fn scale_volume(m: Music<(Pitch, Volume)>, s: Ratio<u8>) -> Music<(Pitch, Volume)> {
    m.map(|(p, v)| {
        let new = (Ratio::from_integer(u8::from(v.get_inner())) * s).to_integer();
        (p, Volume::from(new))
    })
}

/// Exercise 6.10
/// Redefine `revM` from Section 6.6 using `mFold`.
pub fn rev<P>(m: Music<P>) -> Music<P> {
    m.fold(
        Music::Prim,
        |m1, m2| m2 + m1,
        |m1, m2| {
            let d1 = m1.duration();
            let d2 = m2.duration();
            if d1 > d2 {
                m1 | (Music::rest(d1 - d2) + m2)
            } else {
                (Music::rest(d2 - d1) + m1) | m2
            }
        },
        |c, m| m.with(c),
    )
}

/// Exercise 6.11
/// Define a function `inside_out` that inverts
/// the role of serial and parallel composition in a `Music` value.
/// Using `inside_out`, see if you can:
/// - find a non-trivial value `Music<Pitch>` such that
///   m is “musically equivalent” to (i.e. sounds the same as)
///   `inside_out(m)`;
/// - find a value `Music<Pitch>` such that
///   `m + inside_out(m) + m` sounds interesting.
///   (You are free to define what “sounds interesting” means.)
mod inside_out {
    use super::*;

    fn inside_out(m: Music) -> Music {
        m.fold(
            Music::Prim,
            |m1, m2| m1 | m2,
            |m1, m2| m1 + m2,
            |c, m| m.with(c),
        )
    }

    /// If we represent the `Music` value as a matrix
    /// where every row represents a single parallel voice
    /// and every cell represent a single Note, then the
    /// `inside_out` function is the transpose of this matrix.
    ///
    /// So, the `Music` resistant to the transposition
    /// is represented with the symmetric matrix.
    /// E.g.:
    ///           1st QN   2nd QN   3rd QN
    /// voice1     C4        -         D4
    /// voice2     -         -         D4
    /// voice3     D4        D4        E4
    fn example() -> Music {
        let oc4 = Octave::OneLined;
        Music::line(vec![
            Music::C(oc4, Dur::QUARTER),
            rests::QUARTER,
            Music::D(oc4, Dur::QUARTER),
        ]) | Music::line(vec![
            rests::QUARTER,
            rests::QUARTER,
            Music::D(oc4, Dur::QUARTER),
        ]) | Music::line(vec![
            Music::D(oc4, Dur::QUARTER),
            Music::D(oc4, Dur::QUARTER),
            Music::E(oc4, Dur::QUARTER),
        ])
    }
}

mod crazy_recursion {
    use super::*;

    fn rep<P, F, G>(m: Music<P>, f: F, g: G, n: usize) -> Music<P>
    where
        P: Clone,
        F: Fn(Music<P>) -> Music<P>,
        G: Fn(Music<P>) -> Music<P> + Clone,
    {
        if n == 0 {
            return Music::rest(Dur::ZERO);
        }

        m.clone() | g.clone()(rep(f(m), f, g, n - 1))
    }

    // TODO: play me
    fn example1() -> Music {
        let oc4 = Octave::OneLined;
        let run = rep(
            Music::C(oc4, Dur::THIRTY_SECOND),
            |m| m.with_transpose(5.into()),
            |m| m.with_delay(Dur::THIRTY_SECOND),
            8,
        );
        let cascade = rep(
            run,
            |m| m.with_transpose(4.into()),
            |m| m.with_delay(Dur::EIGHTH),
            8,
        );
        let cascades = rep(cascade, |m| m, |m| m.with_delay(Dur::SIXTEENTH), 2);
        cascades.clone() + cascades.reverse()
    }

    fn example2() -> Music {
        let oc4 = Octave::OneLined;
        let run = rep(
            Music::C(oc4, Dur::THIRTY_SECOND),
            |m| m.with_delay(Dur::THIRTY_SECOND),
            |m| m.with_transpose(5.into()),
            8,
        );
        let cascade = rep(
            run,
            |m| m.with_delay(Dur::EIGHTH),
            |m| m.with_transpose(4.into()),
            8,
        );
        let cascades = rep(cascade, |m| m.with_delay(Dur::SIXTEENTH), |m| m, 2);
        cascades.clone() + cascades.reverse()
    }
}

/// Exercise 6.12
/// 1. Define a function `to_intervals` that takes a list of N numbers,
/// and generates a list of N lists, such that the i-th list is
/// the sequence of differences between the adjacent items of the previous sequence.
/// 2. Define a function `get_heads` that takes a list of N lists
/// and returns a list of N numbers such that the i-th element
/// is the head of the i-th list.
/// 3. Compose the above two functions in a suitable way
/// to define a function `interval_closure` that takes an N-element list
/// and returns its interval closure.
/// 4. Define a function `interval_closures` that takes an N-element list
/// and returns an infinite sequence of interval closures.
/// 5. Now for the open-ended part of this exercise:  // TODO and play
/// Interpret the outputs of any of the functions above to create some “interesting” music.
mod intervals {
    fn adjacent_diff<T: Copy + std::ops::Sub<Output = T>>(numbers: &[T]) -> Vec<T> {
        numbers
            .iter()
            .skip(1)
            .zip(numbers.iter())
            .map(|(next, prev)| *next - *prev)
            .collect()
    }

    fn to_intervals<T: Copy + std::ops::Sub<Output = T>>(numbers: Vec<T>) -> Vec<Vec<T>> {
        std::iter::successors(Some(numbers), |xs| {
            (xs.len() > 1).then(|| adjacent_diff(xs))
        })
        .collect()
    }

    fn get_heads<T: Copy>(s: &[Vec<T>]) -> Vec<T> {
        s.iter().filter_map(|xs| xs.first()).copied().collect()
    }

    fn interval_closure<T: Copy + std::ops::Sub<Output = T>>(numbers: Vec<T>) -> Vec<T> {
        get_heads(&to_intervals(numbers))
            .into_iter()
            .rev()
            .collect()
    }

    fn interval_closures<T: Copy + std::ops::Sub<Output = T>>(
        numbers: Vec<T>,
    ) -> impl Iterator<Item = Vec<T>> {
        std::iter::successors(Some(numbers), |xs| Some(interval_closure(xs.clone())))
    }

    #[test]
    fn intervals_example() {
        assert_eq!(
            to_intervals(vec![1, 5, 3, 6, 5, 0, 1, 1]),
            [
                vec![1, 5, 3, 6, 5, 0, 1, 1],
                vec![4, -2, 3, -1, -5, 1, 0],
                vec![-6, 5, -4, -4, 6, -1],
                vec![11, -9, 0, 10, -7],
                vec![-20, 9, 10, -17],
                vec![29, 1, -27],
                vec![-28, -28],
                vec![0],
            ]
        );

        assert_eq!(
            interval_closure(vec![1, 5, 3, 6, 5, 0, 1, 1]),
            vec![0, -28, 29, -20, 11, -6, 4, 1]
        );
    }
}

/// Exercise 6.13
///
/// Write a program that sounds like an infinitely
/// descending (in pitch) sequence of musical lines.
/// Each descending line should fade into the audible range
/// as it begins its descent, and then fade out as it descends further.
/// So the beginning and end of each line will be difficult to hear.
/// And there will be many such lines, each starting at a different time,
/// some perhaps descending a little faster than others, or perhaps using
/// different instrument sounds, and so on.
/// The effect will be that as the music is listened to,
/// everything will seem to be falling, falling, falling with no end,
/// but no beginning either.
/// (This illusion is called the _Shepard Tone_, or _Shepard Scale_,
/// first introduced by Roger Shepard in 1964 [She64].)
///
/// Try to parameterize things in such a way that, for example,
/// with a simple change, you could generate an infinite _ascension_ as well.
mod shepard_scale {
    use std::iter;

    use musik::{midi::Instrument, AbsPitch, Dur, Interval, Music, Octave, Pitch, Volume};

    fn interval_line(start: Pitch, dur: Dur, delta: Interval) -> impl Iterator<Item = Music> {
        iter::successors(Some(start), move |prev| Some(prev.trans(delta)))
            .map(move |p| Music::note(dur, p))
    }

    #[derive(Debug, Copy, Clone)]
    struct LineConfig {
        start: Pitch,
        /// Duration of a single note
        dur: Dur,
        /// Total number of notes played
        size: u8,
        delta: Interval,
        /// Volume levels to increase on each note
        /// at the beginning of the line (1..127)
        fade_in_volume_step: u8,
        /// Volume levels to decrease on each not
        /// at the end of the line (1..127)
        fade_out_volume_step: u8,

        trailing_delay: Dur,
    }

    impl LineConfig {
        fn from_number(seed: u16, delta: Interval) -> Self {
            // C4..=B4
            let oc4 = Octave::OneLined;
            let pitch_range: Vec<Pitch> = (u8::from(Pitch::C(oc4).abs().get_inner())
                ..=u8::from(Pitch::B(oc4).abs().get_inner()))
                .map(|x| AbsPitch::from(ux2::u7::new(x)).into())
                .collect();

            // 1/16, 3/32, 1/8, 3/16, 1/4
            let dur_range = [
                Dur::SIXTEENTH,
                Dur::DOTTED_SIXTEENTH,
                Dur::EIGHTH,
                Dur::DOTTED_EIGHTH,
                Dur::QUARTER,
            ];

            let size_range: Vec<_> = (12..=24).collect();
            let fade_in_range: Vec<_> = (25..=40).collect();
            let fade_out_range: Vec<_> = (25..=40).collect();
            let delay_range = [Dur::EIGHTH, Dur::QUARTER];

            const fn choose_value<T>(xs: &[T], seed: u16) -> &T {
                let index = seed as usize % xs.len();
                &xs[index]
            }

            Self {
                start: *choose_value(&pitch_range, seed),
                dur: *choose_value(&dur_range, seed),
                size: *choose_value(&size_range, seed),
                delta,
                fade_in_volume_step: *choose_value(&fade_in_range, seed),
                fade_out_volume_step: *choose_value(&fade_out_range, seed),
                trailing_delay: *choose_value(&delay_range, seed),
            }
        }

        fn scale(&self) -> Music<(Pitch, Volume)> {
            let max_volume = u8::from(Volume::loudest().get_inner());
            let min_volume = u8::from(Volume::softest().get_inner());

            let fade_out_parts = (max_volume / self.fade_out_volume_step).min(self.size);

            let mut volume = min_volume;
            Music::line(
                interval_line(self.start, self.dur, self.delta)
                    .take(self.size as usize)
                    .zip(0..)
                    .map(|(step, i)| {
                        if i < self.size - fade_out_parts {
                            volume = (volume + self.fade_in_volume_step).min(max_volume);
                        } else {
                            volume = volume.saturating_sub(self.fade_out_volume_step);
                        }

                        Music::with_volume(step, Volume::from(volume))
                    })
                    .chain(Some(Music::rest(self.trailing_delay)))
                    .collect(),
            )
        }
    }

    const fn pseudo_random_gen(seed: u16) -> u16 {
        let next = (seed.wrapping_mul(seed)).wrapping_add(seed).wrapping_add(1);
        if next == seed {
            next + 1
        } else {
            next
        }
    }

    fn music(delta: Interval, lines: &[(Instrument, u16)]) -> Music<(Pitch, Volume)> {
        Music::chord(
            lines
                .iter()
                .map(|(instrument, seed)| {
                    Music::line(
                        iter::successors(Some(*seed), |x| Some(pseudo_random_gen(*x)))
                            // TODO: make it infinite by changing
                            //  Music::Sequential to wrap an Iterator<Item=Music>
                            //  Without that `.take(638)` leads to stack overflow
                            .take(100)
                            .map(|x| LineConfig::from_number(x, delta).scale())
                            .collect(),
                    )
                    .with_instrument(*instrument)
                })
                .collect(),
        )
    }

    #[test]
    fn test_save() {
        use musik::{midi::Instrument::*, Performable as _};

        let m = music(
            -Interval::semi_tone(),
            &[
                (AcousticGrandPiano, 2323),
                (ElectricGuitarClean, 9940),
                (Flute, 7899),
                (Cello, 15000),
            ],
        );
        m.perform_default().save_to_file("desc.mid").unwrap();

        let m = music(
            Interval::semi_tone(),
            &[
                (AcousticGrandPiano, 18774),
                (ElectricGuitarClean, 33300),
                (Flute, 19231),
                (Cello, 99),
            ],
        );
        m.perform_default().save_to_file("asc.mid").unwrap();
    }
}
