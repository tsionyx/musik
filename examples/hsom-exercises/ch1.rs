#[cfg(test)]
pub(super) fn simple<A, B, C, D, E>(x: A, y: B, z: C) -> E
where
    A: std::ops::Mul<D, Output = E>,
    B: std::ops::Add<C, Output = D>,
{
    x * (y + z)
}

/// Exercise 1.1 Write out all of the steps in the calculation of the value of
/// simple (simple 2 3 4) 5 6
#[test]
fn evaluate_simple() {
    let res = simple(simple(2_u32, 3_u32, 4_u32), 5_u32, 6_u32);
    assert_eq!(res, (2 * (3 + 4)) * (5 + 6));
    assert_eq!(res, 154);
}

/// Exercise 1.2 Prove by calculation that
/// simple (a − b) a b ==> a^2 − b^2
#[test]
fn multiply_diff_and_sum() {
    let cases = vec![(15_u32, 0_u32), (8934, 12), (32333, 1), (189, 189)];

    for (a, b) in cases {
        assert!(a >= b);
        assert_eq!(simple(a - b, a, b), a * a - b * b);
    }
}

///Exercise 1.3 Identify the well-typed expressions in the following,
/// and, for each, give its proper type:
#[test]
fn type_me() {
    use musik::PitchClass;

    let _unused: Vec<PitchClass> = vec![PitchClass::A, PitchClass::B, PitchClass::C];
    // let _type_mismatch_in_homogenous_Vec = vec![PitchClass::D, 42];
    let _: (i8, PitchClass) = (-42, PitchClass::Ef);
    let _unused: Vec<(char, u8)> = vec![('a', 3), ('b', 5)];
    // let _arithmetic_ops_not_applicable_to_chars: char = simple('a', 'b', 'c');
    #[allow(clippy::type_complexity)]
    let _unused: (u8, Box<dyn Fn(u8, u8, u8) -> u8>) = (simple(1_u8, 2_u8, 3_u8), Box::new(simple));
    let _unused: Vec<&str> = vec!["I", "love", "Euterpea"];
}

pub mod harmonic {
    //! Exercise 1.4 Modify the definitions of hNote and hList so that they each
    //! take an extra argument that specifies the interval of harmonization (rather
    //! than being fixed at -3). Rewrite the definition of mel to take these changes
    //! into account.
    use musik::{Dur, Interval, Music, Pitch};

    fn harmonic_note(d: Dur, p: Pitch, i: Interval) -> Music {
        Music::note(d, p) | Music::note(d, p.trans(i))
    }

    fn harmonic_list(d: Dur, pitches: &[Pitch], i: Interval) -> Music {
        pitches.iter().fold(Music::rest(Dur::ZERO), |prev, p| {
            prev + harmonic_note(d, *p, i)
        })
    }

    pub fn mel(pitches: [Pitch; 3]) -> Music {
        harmonic_list(Dur::QUARTER, &pitches, Interval::from(-3))
    }
}
