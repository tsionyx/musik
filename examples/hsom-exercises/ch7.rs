use std::cmp::Ordering;

use enum_iterator::Sequence;
use enum_map::Enum;

use musik::{Dur, Music};

/// Exercise 7.1
/// Prove that the instance of `Music` in the class `Eq`
/// satisfies the laws of its class.
/// Also prove that the modified instance of `Music`
/// in the class `Ord` satisfies the laws of its class.
#[cfg(test)]
mod eq_ord_music {
    use super::*;

    use musik::Octave;

    const OC3: Octave = Octave::Small;
    const OC4: Octave = Octave::OneLined;

    #[test]
    fn primitive_notes_same_pitch() {
        let m1 = Music::C(OC4, Dur::QUARTER);
        let m2 = Music::C(OC4, Dur::QUARTER);

        assert_eq!(m1, m2);

        let m_smaller_dur = Music::C(OC4, Dur::EIGHTH);
        assert_ne!(m1, m_smaller_dur);
        assert!(m1 > m_smaller_dur);
        assert!(m_smaller_dur < m1);

        let m_lower_octave = Music::C(OC3, Dur::QUARTER);
        assert_ne!(m1, m_lower_octave);
        assert!(m1 > m_lower_octave);
        assert!(m_lower_octave < m1);
    }

    #[test]
    fn primitive_notes_diff_by_dur_then_by_pitch() {
        let m1 = Music::C(OC4, Dur::QUARTER);
        let m2 = Music::D(OC4, Dur::QUARTER);

        assert_ne!(m1, m2);
        assert!(m1 < m2);
        assert!(m2 > m1);

        let m_smaller_dur = Music::D(OC4, Dur::EIGHTH);
        assert_ne!(m1, m_smaller_dur);
        assert!(m1 > m_smaller_dur);
        assert!(m_smaller_dur < m1);
    }

    #[test]
    fn primitive_notes_smaller_than_rest() {
        let m1 = Music::C(OC4, Dur::QUARTER);
        let m2 = Music::rest(Dur::SIXTEENTH);

        assert_ne!(m1, m2);
        assert!(m1 < m2);
        assert!(m2 > m1);

        let m2 = Music::rest(Dur::WHOLE);
        assert_ne!(m1, m2);
        assert!(m1 < m2);
        assert!(m2 > m1);
    }

    #[test]
    fn rests_are_sorted_by_dur() {
        let m1 = Music::C(OC4, Dur::QUARTER);
        let m2 = Music::rest(Dur::SIXTEENTH);

        assert_ne!(m1, m2);
        assert!(m1 < m2);
        assert!(m2 > m1);

        let m2 = Music::rest(Dur::WHOLE);
        assert_ne!(m1, m2);
        assert!(m1 < m2);
        assert!(m2 > m1);
    }

    #[test]
    fn primitives_are_lower_than_complex() {
        let m1 = Music::C(OC4, Dur::QUARTER);
        let m2 = Music::rest(Dur::SIXTEENTH);

        let m3 = m1.clone() + m2.clone();
        assert_ne!(m1, m3);
        assert!(m1 < m3);
        assert!(m3 > m1);

        assert_ne!(m2, m3);
        assert!(m2 < m3);
        assert!(m3 > m2);

        let m3 = m1.clone() | m2.clone();
        assert_ne!(m1, m3);
        assert!(m1 < m3);
        assert!(m3 > m1);

        assert_ne!(m2, m3);
        assert!(m2 < m3);
        assert!(m3 > m2);

        let m3 = m1.clone().with_tempo(2);
        assert_ne!(m1, m3);
        assert!(m1 < m3);
        assert!(m3 > m1);
    }

    #[test]
    fn sequential_are_ordered_lexicographically() {
        let m1 = Music::C(OC4, Dur::QUARTER);
        let m2 = Music::rest(Dur::SIXTEENTH);
        let m3 = Music::rest(Dur::WHOLE);

        let m4 = m1.clone() + m2;
        let m5 = m1 + m3;
        assert_ne!(m4, m5);
        assert!(m4 < m5);
        assert!(m5 > m4);
    }

    #[test]
    fn parallel_are_ordered_lexicographically() {
        let m1 = Music::C(OC4, Dur::QUARTER);
        let m2 = Music::rest(Dur::SIXTEENTH);
        let m3 = Music::rest(Dur::WHOLE);

        let m4 = m1.clone() | m2;
        let m5 = m1 | m3;
        assert_ne!(m4, m5);
        assert!(m4 < m5);
        assert!(m5 > m4);
    }

    #[test]
    fn sequential_always_smaller_parallel() {
        let m1 = Music::C(OC4, Dur::QUARTER);
        let m2 = Music::rest(Dur::SIXTEENTH);

        let m3 = m1.clone() + m2.clone();
        let m4 = m1 | m2;
        assert_ne!(m3, m4);
        assert!(m3 < m4);
        assert!(m4 > m3);
    }
}

#[derive(Debug)]
enum Color {
    Red,
    Green,
    Blue,
}

/// Exercise 7.2
/// Write out appropriate instance declarations for the `Color`
/// type in the classes `Eq`, `Ord`, and `Enum`.
impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Red, Self::Red) => true,
            (Self::Red, _) => false,
            (Self::Green, Self::Green) => true,
            (Self::Green, _) => false,
            (Self::Blue, Self::Blue) => true,
            (Self::Blue, _) => false,
        }
    }
}

impl Eq for Color {}

impl PartialOrd for Color {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Color {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Red, Self::Red) => Ordering::Equal,
            (Self::Red, _) => Ordering::Less,
            (Self::Green, Self::Red) => Ordering::Greater,
            (Self::Green, Self::Green) => Ordering::Equal,
            (Self::Green, Self::Blue) => Ordering::Less,
            (Self::Blue, Self::Blue) => Ordering::Equal,
            (Self::Blue, _) => Ordering::Greater,
        }
    }
}

impl Enum for Color {
    const LENGTH: usize = 3;

    fn from_usize(value: usize) -> Self {
        match value {
            0 => Self::Red,
            1 => Self::Green,
            2 => Self::Blue,
            _ => panic!("Not a valid value"),
        }
    }

    fn into_usize(self) -> usize {
        match self {
            Self::Red => 0,
            Self::Green => 1,
            Self::Blue => 2,
        }
    }
}

impl Sequence for Color {
    const CARDINALITY: usize = 3;

    fn next(&self) -> Option<Self> {
        match self {
            Self::Red => Some(Self::Green),
            Self::Green => Some(Self::Blue),
            Self::Blue => None,
        }
    }

    fn previous(&self) -> Option<Self> {
        match self {
            Self::Red => None,
            Self::Green => Some(Self::Red),
            Self::Blue => Some(Self::Green),
        }
    }

    fn first() -> Option<Self> {
        Some(Self::Red)
    }

    fn last() -> Option<Self> {
        Some(Self::Blue)
    }
}

/// Exercise 7.3
/// Define a type class called `Temporal` whose members are types
/// that can be interpreted as having a temporal duration.
trait Temporal {
    fn duration_t(&self) -> Dur;
    fn take_t(self, dur: Dur) -> Self;
    fn drop_t(self, dur: Dur) -> Self;
}

impl<P> Temporal for Music<P> {
    fn duration_t(&self) -> Dur {
        self.duration()
    }

    fn take_t(self, dur: Dur) -> Self {
        self.take(dur)
    }

    fn drop_t(self, dur: Dur) -> Self {
        self.drop(dur)
    }
}

/// Exercise 7.4
/// Functions are not members of the `Eq` class, because,
/// in general, determining whether two functions are equal is undecideable.
/// But functions whose domains are finite, and can be completely enumerated,
/// can be tested for equality.
/// We just need to test that each function, when applied
/// to each element in the domain, returns the same result.
#[cfg(test)]
mod tests {
    use std::{any::type_name, fmt};

    use musik::{Interval, Octave, Pitch, PitchClass};

    use crate::compose;

    use super::*;

    struct EqFn<T, U> {
        inner: Box<dyn Fn(T) -> U>,
    }

    impl<T, U> fmt::Debug for EqFn<T, U> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "Wrapper for Equallible function from {} to {}",
                type_name::<T>(),
                type_name::<U>()
            )
        }
    }

    impl<T, U> From<Box<dyn Fn(T) -> U>> for EqFn<T, U> {
        fn from(value: Box<dyn Fn(T) -> U>) -> Self {
            Self { inner: value }
        }
    }

    impl<T, U> PartialEq for EqFn<T, U>
    where
        T: Sequence,
        U: Eq,
    {
        fn eq(&self, other: &Self) -> bool {
            enum_iterator::all::<T>()
                .zip(enum_iterator::all::<T>())
                .all(|(i, j)| (self.inner)(i) == (other.inner)(j))
        }
    }

    impl<T, U> Eq for EqFn<T, U>
    where
        T: Sequence + Clone,
        U: Eq,
    {
    }

    #[test]
    fn colors_fancy_name() {
        fn foo(color: Color) -> String {
            if color == Color::Blue {
                return "Like a sky".into();
            }

            if color == Color::Red {
                return "Blood-like".into();
            }

            if color == Color::Green {
                return "It's definitely a grass".into();
            }

            unreachable!()
        }

        let bar = |c: Color| -> String {
            match c {
                Color::Red => "Blood-like",
                Color::Green => "It's definitely a grass",
                Color::Blue => "Like a sky",
            }
            .into()
        };

        assert_eq!(
            EqFn {
                inner: Box::new(foo),
            },
            EqFn {
                inner: Box::new(bar)
            }
        );
    }

    #[test]
    fn pitch_class_to_abs() {
        fn foo(pc: Option<PitchClass>) -> i8 {
            pc.map(|pc| Pitch::new(pc, Octave::OneLined).abs().get_inner())
                .unwrap_or(i8::MIN)
        }

        let bar = |c: Option<PitchClass>| -> i8 {
            if c.is_none() {
                return -127;
            }

            let c = c.unwrap();
            Interval::from(c).get_inner() + 49
        };

        assert_ne!(
            EqFn {
                inner: Box::new(bar),
            },
            (Box::new(foo) as Box<dyn Fn(Option<PitchClass>) -> i8>).into(),
        );

        // fix the bar function bu subtracting 1 to make it equal again
        assert_eq!(
            EqFn {
                inner: Box::new(compose(bar, |x| x - 1)),
            },
            (Box::new(foo) as Box<dyn Fn(Option<PitchClass>) -> i8>).into(),
        );
    }
}
