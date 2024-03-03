#[cfg(test)]
use ux::u4;

#[cfg(test)]
use musik::Octave;
use musik::{music::Primitive, Dur, Interval, Music, Pitch};

#[cfg(test)]
use super::simple;

/// Exercise 3.1.1
/// Transposes each pitch by the amount specified.
fn f1(pitches: &[Pitch], delta: Interval) -> Vec<Pitch> {
    pitches.iter().map(|p| p.trans(delta)).collect()
}

#[test]
fn test_trans_map() {
    let oc3 = Octave::try_from(u4::new(3)).unwrap();
    let pitches = vec![Pitch::C(oc3), Pitch::Fs(oc3), Pitch::A(oc3)];
    let shifted = f1(&pitches, Interval::from(4));
    assert_eq!(
        &shifted,
        &[
            Pitch::E(oc3),
            Pitch::As(oc3),
            Pitch::Cs(Octave::try_from(u4::new(4)).unwrap())
        ]
    );
}

/// Exercise 3.1.2
/// Turns a list of durations into a list of rests, each having the corresponding duration.
fn f2(durations: &[Dur]) -> Vec<Music> {
    durations.iter().map(|&d| Music::rest(d)).collect()
}

#[test]
fn test_durations_map() {
    let durations = vec![Dur::QUARTER, Dur::WHOLE, Dur::DOTTED_HALF];
    let rests = f2(&durations);
    assert_eq!(
        &rests,
        &[
            Music::Prim(Primitive::Rest(Dur::QUARTER)),
            Music::Prim(Primitive::Rest(Dur::WHOLE)),
            Music::Prim(Primitive::Rest(Dur::DOTTED_HALF)),
        ]
    );
}

/// Exercise 3.1.3
/// Given a list of `Music` values (that are assumed to be single notes),
/// for each such note, halves its duration and places
/// a rest of that same duration after it.
fn staccato(musics: Vec<Music>) -> Vec<Music> {
    musics
        .into_iter()
        .map(|music| {
            if let Music::Prim(Primitive::Note(duration, pitch)) = music {
                let halved = duration.halve();
                Music::note(halved, pitch) + Music::rest(halved)
            } else {
                music
            }
        })
        .collect()
}

#[test]
fn test_staccato() {
    let oc4 = Octave::try_from(u4::new(4)).unwrap();
    // [c 4 qn, d 4 en, e 4 hn ]
    let music = vec![
        Music::C(oc4, Dur::QUARTER),
        Music::D(oc4, Dur::EIGHTH),
        Music::E(oc4, Dur::HALF),
    ];
    let staccated = staccato(music);
    assert_eq!(
        &staccated,
        &[
            Music::Sequential(
                Box::new(Music::C(oc4, Dur::EIGHTH)),
                Box::new(Music::rest(Dur::EIGHTH)),
            ),
            Music::Sequential(
                Box::new(Music::D(oc4, Dur::SIXTEENTH)),
                Box::new(Music::rest(Dur::SIXTEENTH)),
            ),
            Music::Sequential(
                Box::new(Music::E(oc4, Dur::QUARTER)),
                Box::new(Music::rest(Dur::QUARTER)),
            ),
        ]
    );
}

fn flip<A, B, C, F>(f: F) -> impl Fn(B, A) -> C
where
    F: Fn(A, B) -> C,
{
    move |a, b| f(b, a)
}

/// Exercise 3.2
/// Show that flip (flip f ) is the same as f.
#[test]
fn test_flip() {
    let f = |a: u64, b: i64| (a as i64) + 2 * b;
    assert_eq!(f(5, -100), -195);
    assert_eq!(flip(f)(-100, 5), -195);
    assert_eq!(flip(flip(f))(5, -100), -195);
}

fn partially_applied<T, U, V, F>(f: F, xs: Vec<T>) -> impl Iterator<Item = Box<dyn FnOnce(U) -> V>>
where
    F: FnOnce(T, U) -> V + Copy + 'static,
    T: 'static,
{
    xs.into_iter()
        .map(move |x| Box::new(move |y: U| f(x, y)) as Box<dyn FnOnce(U) -> V>)
}

/// Exercise 3.3
/// What is the type of ys in:
/// xs = [1, 2, 3] :: [Integer ]
/// ys = map (+) xs
fn partially_applied_sum(xs: Vec<u32>) -> impl Iterator<Item = Box<dyn FnOnce(u32) -> u32>> {
    partially_applied(|x, y| x + y, xs)
}

#[test]
fn partially_applied_test() {
    let funcs = partially_applied_sum(vec![1, 2, 3]);
    for (i, f) in funcs.enumerate() {
        assert_eq!(f(100), 101 + i as u32);
    }
}

fn apply_each<F, T, U>(fs: Vec<Box<F>>, x: T) -> impl Iterator<Item = U>
where
    F: Fn(T) -> U + ?Sized,
    T: Clone + 'static,
{
    fs.into_iter().map(move |f| f(x.clone()))
}

/// Exercise 3.4
/// Given a list of functions, applies each to some given value.
#[test]
fn apply_each_test() {
    let fs = vec![
        Box::new(|x| simple(2_u32, 2_u32, x)) as Box<dyn Fn(u32) -> u32>,
        Box::new(|x| x + 3_u32) as Box<dyn Fn(u32) -> u32>,
    ];
    assert_eq!(apply_each(fs, 5).collect::<Vec<_>>(), vec![14, 8]);
}

fn apply_all<F, T>(fs: Vec<Box<F>>, v: T) -> T
where
    F: Fn(T) -> T + ?Sized,
    T: Clone + 'static,
{
    fs.into_iter().rfold(v, move |x, f| f(x))
}

/// Exercise 3.5
/// Given a list of functions [f1, f2, ..., fn] and a value v,
/// returns the result f1(f2(...(fn v)...)).
#[test]
fn apply_all_test() {
    let fs = vec![
        Box::new(|x| simple(2_u32, 2_u32, x)) as Box<dyn Fn(u32) -> u32>,
        Box::new(|x| x + 3_u32) as Box<dyn Fn(u32) -> u32>,
    ];
    assert_eq!(apply_all(fs, 5), 20);

    let fs = vec![
        Box::new(|x| x + 3_u32) as Box<dyn Fn(u32) -> u32>,
        Box::new(|x| simple(2_u32, 2_u32, x)) as Box<dyn Fn(u32) -> u32>,
    ];
    assert_eq!(apply_all(fs, 5), 17);
}

/// Sum two lists together.
/// The complexity of this is the size of the first list.
fn sum_vec<T>(v1: Vec<T>, v2: Vec<T>) -> Vec<T> {
    v1.into_iter().rfold(v2, |mut acc, x| {
        acc.insert(0, x);
        acc
    })
}

#[test]
fn test_sum() {
    assert_eq!(
        sum_vec(vec![5, 17, 32], vec![2, 8, 9]),
        vec![5, 17, 32, 2, 8, 9]
    );
}

mod append {
    //! Exercise 3.6
    //! Recall the discussion about the efficiency of (++) and concat in Chapter 3.
    //! Which of the following functions is more efficient, and why?
    //! appendr, appendl :: [[a]] -> [a]
    //! appendr = foldr (flip (++)) []
    //! appendl = foldl (flip (++)) []

    use super::{flip, sum_vec};

    fn append_right<T>(xss: Vec<Vec<T>>) -> Vec<T>
    where
        T: 'static,
    {
        //! The following is a Haskell model of calculation,
        //! where the operation of `foldr` applies as `op(xn, init)`.
        //! It is completely not valid for Rust, since in `rfold` as in `fold`
        //! the operation always applies as `op(init, xn)`.
        //!
        //! ==>
        //! flip(sum_vec) (
        //!     x1,
        //!     flip(sum_vec) (
        //!         x2,
        //!         ..
        //!         flip(sum_vec) (
        //!             xn,
        //!             []
        //!         )
        //!     )
        //! )
        //!
        //! ==>
        //! sum_vec (
        //!     sum_vec (
        //!         sum_vec (
        //!             xn,
        //!             []
        //!         )
        //!         ..
        //!         x2,
        //!     )
        //!     x1,
        //! )
        //!
        //! ==>
        //! (((xn ++ []) ++ .. x2) ++ x1)
        //!
        //! The total complexity is `sum(sum(len(prev) for prev in xss[i:]) for i, xs in enumerate(xss))`.
        //! Equivalent to quadratic complexity.
        //!
        //! ---
        //! For Rust, both models has the same linear complexity.
        xss.into_iter().rfold(vec![], flip(sum_vec))
    }

    fn append_left<T>(xss: Vec<Vec<T>>) -> Vec<T>
    where
        T: 'static,
    {
        //! ==>
        //! flip(sum_vec) (
        //!     flip(sum_vec) (
        //!         flip(sum_vec) (
        //!             []
        //!             x1,
        //!         )
        //!         x2,
        //!     )
        //!     ..
        //!     xn,
        //! )
        //!
        //! ==>
        //! sum_vec (
        //!     xn,
        //!     ..
        //!     sum_vec (
        //!         x2,
        //!         sum_vec (
        //!             x1,
        //!             []
        //!         )
        //!     )
        //! )
        //!
        //! ==>
        //!
        //! x1 ++ (x2 ++ .. (xn + []))
        //!
        //! The total complexity is sum(len(xs) for xs in xss)
        xss.into_iter().fold(vec![], flip(sum_vec))
    }

    #[test]
    fn test_append_right() {
        let in_v = vec![vec![1], vec![3, 4], vec![], vec![5, 6]];
        assert_eq!(append_right(in_v), vec![1, 3, 4, 5, 6]);
    }

    #[test]
    fn test_append_left_is_reversing_the_items() {
        let in_v = vec![vec![1], vec![3, 4], vec![], vec![5, 6]];
        assert_eq!(append_left(in_v), vec![5, 6, 3, 4, 1]);
    }
}

/// Exercise 3.7 Rewrite the definition of length non-recursively
fn len<T>(xs: &[T]) -> usize {
    xs.iter().fold(0, |acc, _| acc + 1)
}

#[test]
fn test_length() {
    let x = &[5, 8, 15, 22];
    assert_eq!(len(x), 4);
}

mod map_examples {
    //! Exercise 3.8

    /// Doubles each number in a list
    fn double_each<T, U>(xs: Vec<T>) -> Vec<U>
    where
        T: std::ops::Mul<u8, Output = U>,
    {
        xs.into_iter().map(|x| x * 2).collect()
    }

    #[test]
    fn test_double_each() {
        assert_eq!(double_each(vec![1, 2, 3]), vec![2, 4, 6]);
    }

    /// Pairs each element in a list with that number and one plus that number.
    fn pair_and_one<T, U>(xs: Vec<T>) -> Vec<(T, U)>
    where
        T: Copy + std::ops::Add<u8, Output = U>,
    {
        xs.into_iter().map(|x| (x, x + 1)).collect()
    }

    #[test]
    fn test_pair_and_one() {
        assert_eq!(pair_and_one(vec![1, 2, 3]), vec![(1, 2), (2, 3), (3, 4)]);
    }

    /// Adds together each pair of numbers in a list.
    fn add_each_pair<T, U>(xs: Vec<(T, T)>) -> Vec<U>
    where
        T: Copy + std::ops::Add<Output = U>,
    {
        xs.into_iter().map(|(x, y)| x + y).collect()
    }

    #[test]
    fn test_add_each_pair() {
        assert_eq!(add_each_pair(vec![(1, 2), (3, 4), (5, 6)]), vec![3, 7, 11]);
    }

    /// Adds “pointwise” the elements of a list of pairs
    fn add_pairs_pointwise<T>(xs: Vec<(T, T)>) -> (T, T)
    where
        T: Default + std::ops::Add<Output = T>,
    {
        xs.into_iter()
            .fold((T::default(), T::default()), |(acc_x, acc_y), (x, y)| {
                (acc_x + x, acc_y + y)
            })
    }

    #[test]
    fn test_add_pairs_pointwise() {
        assert_eq!(add_pairs_pointwise(vec![(1, 2), (3, 4), (5, 6)]), (9, 12));
    }
}

/// Exercise 3.9
/// Combines a list of durations with a list of notes
/// lacking a duration, to create a list of complete notes.
fn fuse<P>(
    dur: &[Dur],
    notes_ctr: Vec<Box<dyn Fn(Dur) -> Music<P>>>,
) -> Result<Vec<Music<P>>, String> {
    if dur.len() != notes_ctr.len() {
        return Err("Lengths does not match".into());
    }

    Ok(dur.iter().zip(notes_ctr).map(|(d, f)| f(*d)).collect())
}

#[test]
fn test_fuse() {
    let oc = Octave::try_from(u4::new(4)).unwrap();

    let constructors = vec![
        Box::new(move |d| Music::C(oc, d)) as Box<dyn Fn(Dur) -> Music<Pitch>>,
        Box::new(move |d| Music::D(oc, d)) as Box<dyn Fn(Dur) -> Music<Pitch>>,
        Box::new(move |d| Music::E(oc, d)) as Box<dyn Fn(Dur) -> Music<Pitch>>,
    ];

    assert_eq!(
        fuse(&[Dur::QUARTER, Dur::HALF, Dur::SIXTEENTH], constructors).unwrap(),
        vec![
            Music::C(oc, Dur::QUARTER),
            Music::D(oc, Dur::HALF),
            Music::E(oc, Dur::SIXTEENTH),
        ]
    );
}

mod max_min_pitches {
    //! Exercise 3.10

    use musik::AbsPitch;

    fn max_pitch(pitches: &[AbsPitch]) -> AbsPitch {
        if pitches.is_empty() {
            panic!("no pitches");
        }

        pitches[1..].iter().fold(pitches[0], |acc, p| {
            if acc.get_inner() > p.get_inner() {
                acc
            } else {
                *p
            }
        })
    }

    fn max_pitch_rec(pitches: &[AbsPitch]) -> AbsPitch {
        if pitches.is_empty() {
            panic!("no pitches");
        }

        fn max_pitch_inner(h: AbsPitch, t: &[AbsPitch]) -> AbsPitch {
            if t.is_empty() {
                return h;
            }

            let tail_max = max_pitch_inner(t[0], &t[1..]);

            if tail_max.get_inner() > h.get_inner() {
                tail_max
            } else {
                h
            }
        }

        max_pitch_inner(pitches[0], &pitches[1..])
    }

    #[test]
    #[should_panic]
    fn test_max_pitch_empty() {
        let _ = max_pitch(&[]);
    }

    #[test]
    #[should_panic]
    fn test_max_pitch_rec_empty() {
        let _ = max_pitch_rec(&[]);
    }

    #[test]
    fn test_max_pitch() {
        let pitches = vec![
            AbsPitch::from(5),
            AbsPitch::from(2),
            AbsPitch::from(-1),
            AbsPitch::from(6),
        ];
        assert_eq!(max_pitch(&pitches), AbsPitch::from(6));
        assert_eq!(max_pitch_rec(&pitches), AbsPitch::from(6));
    }

    fn min_pitch(pitches: &[AbsPitch]) -> AbsPitch {
        if pitches.is_empty() {
            panic!("no pitches");
        }

        pitches[1..].iter().fold(pitches[0], |acc, p| {
            if acc.get_inner() < p.get_inner() {
                acc
            } else {
                *p
            }
        })
    }

    fn min_pitch_rec(pitches: &[AbsPitch]) -> AbsPitch {
        if pitches.is_empty() {
            panic!("no pitches");
        }

        fn min_pitch_inner(h: AbsPitch, t: &[AbsPitch]) -> AbsPitch {
            if t.is_empty() {
                return h;
            }

            let tail_max = min_pitch_inner(t[0], &t[1..]);

            if tail_max.get_inner() < h.get_inner() {
                tail_max
            } else {
                h
            }
        }

        min_pitch_inner(pitches[0], &pitches[1..])
    }

    #[test]
    #[should_panic]
    fn test_min_pitch_empty() {
        let _ = min_pitch(&[]);
    }

    #[test]
    #[should_panic]
    fn test_min_pitch_rec_empty() {
        let _ = min_pitch_rec(&[]);
    }

    #[test]
    fn test_min_pitch() {
        let pitches = vec![
            AbsPitch::from(5),
            AbsPitch::from(-1),
            AbsPitch::from(2),
            AbsPitch::from(-2),
            AbsPitch::from(6),
        ];
        assert_eq!(min_pitch(&pitches), AbsPitch::from(-2));
        assert_eq!(min_pitch_rec(&pitches), AbsPitch::from(-2));
    }
}

mod chromatic {
    //! Exercise 3.11
    //! Define a function chrom :: Pitch → Pitch → Music Pitch
    //! such that chrom p1 p2 is a chromatic scale of quarter-notes whose first pitch
    //! is p1 and last pitch is p2. If p1 > p2 , the scale should be descending, otherwise
    //! it should be ascending. If p1 == p2, then the scale should contain just one
    //! note. (A chromatic scale is one whose successive pitches are separated by
    //! one absolute pitch (i.e. one semitone)).

    use std::cmp::Ordering;

    use super::*;

    fn chrom(p1: Pitch, p2: Pitch) -> Music {
        let interval = p2.abs() - p1.abs();
        let current_note = Music::note(Dur::QUARTER, p1);

        current_note
            + match interval.cmp(&Interval::zero()) {
                Ordering::Less => chrom(p1.prev(), p2),
                Ordering::Equal => Music::rest(Dur::ZERO),
                Ordering::Greater => chrom(p1.next(), p2),
            }
    }

    #[test]
    fn test_chrom_rec_ascending() {
        let oc = Octave::try_from(u4::new(4)).unwrap();
        let d = Dur::QUARTER;
        let res = chrom(Pitch::C(oc), Pitch::F(oc));

        assert_eq!(
            res,
            Music::C(oc, d)
                + (Music::Cs(oc, d)
                    + (Music::D(oc, d)
                        + (Music::Ds(oc, d)
                            + (Music::E(oc, d) + (Music::F(oc, d) + Music::rest(Dur::ZERO))))))
        );
    }

    #[test]
    fn test_chrom_rec_descending() {
        let o4 = Octave::try_from(u4::new(4)).unwrap();
        let o3 = Octave::try_from(u4::new(3)).unwrap();
        let d = Dur::QUARTER;
        let res = chrom(Pitch::C(o4), Pitch::A(o3));

        assert_eq!(
            res,
            Music::C(o4, d)
                + (Music::B(o3, d)
                    + (Music::As(o3, d) + (Music::A(o3, d) + Music::rest(Dur::ZERO))))
        );
    }
}

/// Exercise 3.12 Abstractly, a scale can be described by the intervals between
/// successive notes. For example, the 7-note major scale can be defined as the
/// sequence of 6 intervals [2, 2, 1, 2, 2, 2], and the 12-note chromatic scale by the
/// 11 intervals [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1].
/// Define a function mkScale :: Pitch → [Int] → Music Pitch
/// such that `mkScale p ints` is the scale beginning at pitch
/// `p` and having the intervallic structure `ints`.
fn mk_scale(p: Pitch, dur: Dur, ints: &[Interval]) -> Music {
    ints.iter()
        .fold(
            (Music::note(dur, p), Interval::zero()),
            |(music, acc_int), int| {
                let acc_int = acc_int + *int;
                (music + Music::note(dur, p.trans(acc_int)), acc_int)
            },
        )
        .0
}

#[test]
fn test_mk_scale_at_7_major() {
    let oc = Octave::try_from(u4::new(4)).unwrap();
    let d = Dur::QUARTER;
    let tone = Interval::tone();
    let semi_tone = Interval::semi_tone();
    let res = mk_scale(Pitch::C(oc), d, &[tone, tone, semi_tone, tone, tone, tone]);

    assert_eq!(
        res,
        Music::C(oc, d)
            + Music::D(oc, d)
            + Music::E(oc, d)
            + Music::F(oc, d)
            + Music::G(oc, d)
            + Music::A(oc, d)
            + Music::B(oc, d)
    );
}

mod major_scale {
    //! Exercise 3.13
    //! Define an enumerated data type that captures each of the
    //! standard major scale modes: Ionian, Dorian, Phrygian, Lydian, Mixolydian,
    //! Aeolian, and Locrian. Then define a function genScale that, given one
    //! of these contructors, generates a scale in the intervalic form described in
    //! Exercise 3.12.

    use super::*;

    #[derive(Debug, Copy, Clone)]
    enum ScaleMode {
        Ionian,
        Dorian,
        Phrygian,
        Lydian,
        Mixolydian,
        Aeolian,
        Locrian,
    }

    impl ScaleMode {
        const fn get_intervals(self) -> [Interval; 7] {
            let tone = Interval::tone();
            let semi_tone = Interval::semi_tone();
            match self {
                Self::Ionian => [tone, tone, semi_tone, tone, tone, tone, semi_tone],
                Self::Dorian => [tone, semi_tone, tone, tone, tone, semi_tone, tone],
                Self::Phrygian => [semi_tone, tone, tone, tone, semi_tone, tone, tone],
                Self::Lydian => [tone, tone, tone, semi_tone, tone, tone, semi_tone],
                Self::Mixolydian => [tone, tone, semi_tone, tone, tone, semi_tone, tone],
                Self::Aeolian => [tone, semi_tone, tone, tone, semi_tone, tone, tone],
                Self::Locrian => [semi_tone, tone, tone, semi_tone, tone, tone, tone],
            }
        }

        fn gen_scale(self, p: Pitch, d: Dur) -> Music {
            mk_scale(p, d, &self.get_intervals())
        }
    }

    #[test]
    fn ionian_c() {
        let oc = Octave::try_from(u4::new(4)).unwrap();
        let oc5 = Octave::try_from(u4::new(5)).unwrap();
        let d = Dur::QUARTER;
        let res = ScaleMode::Ionian.gen_scale(Pitch::C(oc), d);

        assert_eq!(
            res,
            Music::C(oc, d)
                + Music::D(oc, d)
                + Music::E(oc, d)
                + Music::F(oc, d)
                + Music::G(oc, d)
                + Music::A(oc, d)
                + Music::B(oc, d)
                + Music::C(oc5, d)
        );
    }

    #[test]
    fn dorian_d() {
        let oc = Octave::try_from(u4::new(4)).unwrap();
        let oc5 = Octave::try_from(u4::new(5)).unwrap();
        let d = Dur::QUARTER;
        let res = ScaleMode::Dorian.gen_scale(Pitch::D(oc), d);

        assert_eq!(
            res,
            Music::D(oc, d)
                + Music::E(oc, d)
                + Music::F(oc, d)
                + Music::G(oc, d)
                + Music::A(oc, d)
                + Music::B(oc, d)
                + Music::C(oc5, d)
                + Music::D(oc5, d)
        );
    }

    #[test]
    fn phrygian_e() {
        let oc = Octave::try_from(u4::new(4)).unwrap();
        let oc5 = Octave::try_from(u4::new(5)).unwrap();
        let d = Dur::QUARTER;
        let res = ScaleMode::Phrygian.gen_scale(Pitch::E(oc), d);

        assert_eq!(
            res,
            Music::E(oc, d)
                + Music::F(oc, d)
                + Music::G(oc, d)
                + Music::A(oc, d)
                + Music::B(oc, d)
                + Music::C(oc5, d)
                + Music::D(oc5, d)
                + Music::E(oc5, d)
        );
    }

    #[test]
    fn lydian_f() {
        let oc = Octave::try_from(u4::new(4)).unwrap();
        let oc5 = Octave::try_from(u4::new(5)).unwrap();
        let d = Dur::QUARTER;
        let res = ScaleMode::Lydian.gen_scale(Pitch::F(oc), d);

        assert_eq!(
            res,
            Music::F(oc, d)
                + Music::G(oc, d)
                + Music::A(oc, d)
                + Music::B(oc, d)
                + Music::C(oc5, d)
                + Music::D(oc5, d)
                + Music::E(oc5, d)
                + Music::F(oc5, d)
        );
    }

    #[test]
    fn mixolydian_g() {
        let oc3 = Octave::try_from(u4::new(3)).unwrap();
        let oc = Octave::try_from(u4::new(4)).unwrap();
        let d = Dur::QUARTER;
        let res = ScaleMode::Mixolydian.gen_scale(Pitch::G(oc3), d);

        assert_eq!(
            res,
            Music::G(oc3, d)
                + Music::A(oc3, d)
                + Music::B(oc3, d)
                + Music::C(oc, d)
                + Music::D(oc, d)
                + Music::E(oc, d)
                + Music::F(oc, d)
                + Music::G(oc, d)
        );
    }

    #[test]
    fn aeolian_a() {
        let oc3 = Octave::try_from(u4::new(3)).unwrap();
        let oc = Octave::try_from(u4::new(4)).unwrap();
        let d = Dur::QUARTER;
        let res = ScaleMode::Aeolian.gen_scale(Pitch::A(oc3), d);

        assert_eq!(
            res,
            Music::A(oc3, d)
                + Music::B(oc3, d)
                + Music::C(oc, d)
                + Music::D(oc, d)
                + Music::E(oc, d)
                + Music::F(oc, d)
                + Music::G(oc, d)
                + Music::A(oc, d)
        );
    }

    #[test]
    fn locrian_b() {
        let oc3 = Octave::try_from(u4::new(3)).unwrap();
        let oc = Octave::try_from(u4::new(4)).unwrap();
        let d = Dur::QUARTER;
        let res = ScaleMode::Locrian.gen_scale(Pitch::B(oc3), d);

        assert_eq!(
            res,
            Music::B(oc3, d)
                + Music::C(oc, d)
                + Music::D(oc, d)
                + Music::E(oc, d)
                + Music::F(oc, d)
                + Music::G(oc, d)
                + Music::A(oc, d)
                + Music::B(oc, d)
        );
    }
}

mod brother_john {
    //! Exercise 3.14
    //! Write the melody of “Frère Jacques” (or, “Are You Sleeping”) in Euterpea.
    //! Try to make it as succinct as possible. Then, using functions already defined,
    //! generate a traditional four-part round, i.e. four identical voices,
    //! each delayed successively by two measures.
    //! Use a different instrument to realize each voice.
    use std::iter;

    use musik::{midi::Instrument, Dur, Music, Octave, Pitch};

    fn frere_jacques_one_voice() -> Music {
        let oc4 = Octave::OneLined;
        let oc5 = Octave::TwoLined;
        let frere_jacques = vec![
            (Pitch::F(oc4), Dur::QUARTER),
            (Pitch::G(oc4), Dur::QUARTER),
            (Pitch::A(oc4), Dur::QUARTER),
            (Pitch::F(oc4), Dur::QUARTER),
        ];
        let dormez_vous = vec![
            (Pitch::A(oc4), Dur::QUARTER),
            (Pitch::Bf(oc4), Dur::QUARTER),
            (Pitch::C(oc5), Dur::HALF),
        ];

        let sonnez_les_matines = vec![
            (Pitch::C(oc5), Dur::EIGHTH),
            (Pitch::D(oc5), Dur::EIGHTH),
            (Pitch::C(oc5), Dur::EIGHTH),
            (Pitch::Bf(oc4), Dur::EIGHTH),
            (Pitch::A(oc4), Dur::QUARTER),
            (Pitch::F(oc4), Dur::QUARTER),
        ];

        let din_dan_don = vec![
            (Pitch::F(oc4), Dur::QUARTER),
            (Pitch::C(oc4), Dur::QUARTER),
            (Pitch::F(oc4), Dur::HALF),
        ];

        let measures = vec![frere_jacques, dormez_vous, sonnez_les_matines, din_dan_don];

        sequential(
            measures
                .into_iter()
                .flat_map(twice)
                .map(|(p, dur)| Music::note(dur, p)),
        )
    }

    fn twice<T>(xs: impl IntoIterator<Item = T> + Clone) -> impl Iterator<Item = T> {
        xs.clone().into_iter().chain(xs)
    }

    fn sequential(musics: impl Iterator<Item = Music>) -> Music {
        musics.fold(Music::rest(Dur::ZERO), |melody, m| melody + m)
    }

    fn parallel(musics: impl Iterator<Item = Music>) -> Music {
        musics.fold(Music::rest(Dur::ZERO), |melody, m| melody | m)
    }

    fn make_round(one_voice: Music, instruments: Vec<Instrument>, delay: Dur) -> Music {
        let voices = instruments.into_iter().enumerate().map(|(i, instrument)| {
            let init_rest = iter::repeat(Music::rest(delay)).take(i);
            let voice = init_rest.chain(iter::once(one_voice.clone().with_instrument(instrument)));
            sequential(voice)
        });
        parallel(voices)
    }

    /// `https://en.wikipedia.org/wiki/Fr%C3%A8re_Jacques`
    fn frere_jacques_four_part_round() -> Music {
        make_round(
            frere_jacques_one_voice(),
            vec![
                Instrument::AcousticGrandPiano,
                Instrument::Contrabass,
                Instrument::ElectricGuitarClean,
                Instrument::Accordion,
            ],
            Dur::BREVIS,
        )
    }
}

mod freddie_the_frog {
    //! Exercise 3.15
    //! Freddie the Frog wants to communicate privately with his girlfriend Francine
    //! by encrypting messages sent to her. Frog brains are not that large,
    //! so they agree on this simple strategy: each character in the text
    //! shall be converted to the character “one greater” than it, based
    //! on the representation described below (with wrap-around from 255 to 0).
    //! Define functions encrypt and decrypt that will allow Freddie and Francine
    //! to communicate using this strategy.

    fn encrypt(msg: &str) -> String {
        msg.chars()
            .map(|ch| {
                let code = ch as u8;
                let enc = code.wrapping_add(1);
                char::from(enc)
            })
            .collect()
    }

    fn decrypt(msg: &str) -> String {
        msg.chars()
            .map(|ch| {
                let code = ch as u8;
                let enc = code.wrapping_sub(1);
                char::from(enc)
            })
            .collect()
    }

    #[test]
    fn test_encrypt() {
        let msg = "Test TEXT";
        assert_eq!(encrypt(msg), "Uftu!UFYU");
    }

    #[test]
    fn roundtrip() {
        let msg = "send you a letter";
        assert_eq!(decrypt(&encrypt(msg)), msg);
    }
}
