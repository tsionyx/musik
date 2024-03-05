use musik::{AbsPitch, Dur, Music};

use crate::compose;

// twice:: (a->a) -> (a->a)
fn twice<T>(f: impl Fn(T) -> T) -> impl Fn(T) -> T {
    move |x| f(f(x))
}

/// Exercise 5.1
/// Define a function twice that, given a function `f`,
/// returns a function that applies `f` twice to its argument.
#[test]
fn twice_test() {
    let f = |x| x + 1;
    assert_eq!(twice(f)(2), 4);

    let f = |x| x * x + 5;
    assert_eq!(twice(f)(1), 6 * 6 + 5);
}

fn twice_using_compose<T>(f: impl Fn(T) -> T + Clone) -> impl Fn(T) -> T {
    compose(f.clone(), f)
}

#[test]
fn twice_using_compose_test() {
    let f = |x| x + 1;
    assert_eq!(twice(f)(2), 4);

    let f = |x| x * x + 5;
    assert_eq!(twice(f)(1), 6 * 6 + 5);
}

fn twice_boxed<T>(f: impl Fn(T) -> T + 'static) -> Box<dyn Fn(T) -> T> {
    Box::new(move |x| f(f(x)))
}

#[test]
fn double_twice() {
    let f = |x| x * 2;
    assert_eq!(twice(f)(1), 4);

    // compose(twice, twice) -> applies 2 * 2 = 4 times
    let f = |x| x * 3;
    assert_eq!(compose(twice, twice)(f)(1), 81);

    // but twice(twice) applies 2^2 = 4 times (power, not multiplication!)
    assert_eq!(twice_boxed(twice_boxed)(Box::new(f))(1), 81);
}

// thrice:: (a->a) -> (a->a)
fn thrice<T>(f: impl Fn(T) -> T + 'static) -> Box<dyn Fn(T) -> T> {
    power(f, 3)
}

#[test]
fn twice_and_thrice() {
    let f = |x| x + 1;
    assert_eq!(compose(thrice, twice)(f)(0), 6);

    // Haskell: thrice twice (+1) 0
    assert_eq!(thrice(twice_boxed)(Box::new(f))(0), 8); // applied 2^3 times

    // Haskell: twice thrice (+1) 0
    assert_eq!(twice_boxed(thrice)(Box::new(f))(0), 9); // applied 3^2 times

    // Haskell: twice (thrice twice) (+1) 0
    assert_eq!(twice_boxed(thrice(twice_boxed))(Box::new(f))(0), 64); // applied (2^3)^2 times

    // Haskell: (twice thrice) twice (+1) 0
    assert_eq!(
        twice_boxed(thrice)(Box::new(twice_boxed))(Box::new(f))(0),
        512
    ); // applied 2^(3^2) times
}

#[test]
fn three_floors_power() {
    let f = Box::new(|x| x + 1);
    // Haskell: (thrice thrice) twice (+1) 0
    assert_eq!(thrice(thrice)(Box::new(twice_boxed))(f)(0_u32), 134_217_728); // applied 2^(3^3) == 2^27 times
}

/// Exercise 5.2
/// Generalize `twice` defined in the previous exercise by defining
/// a function `power` that takes a function `f` and an integer `n`,
/// and returns a function that applies the function `f` to its argument n times.
/// power:: (a->a) -> Int -> (a->a)
fn power<T>(f: impl Fn(T) -> T + 'static, n: usize) -> Box<dyn Fn(T) -> T> {
    Box::new(move |x| (0..n).fold(x, |acc, _| f(acc)))
}

#[test]
fn power_test() {
    let f = |x| x + 2;
    assert_eq!(power(f, 5)(1), 11);
}

mod fix_impl {
    //! <https://stackoverflow.com/a/42182841>
    pub type Lazy<'a, T> = Box<dyn FnOnce() -> T + 'a>;

    // fix: (Lazy<T> -> T) -> T
    fn fix<'a, T, F>(f: &'a F) -> T
    where
        F: Fn(Lazy<'a, T>) -> T + 'a,
    {
        f(Box::new(move || fix(f)))
    }

    pub type BoxFn<'a, T, U> = Box<dyn FnOnce(T) -> U + 'a>;

    /// Find out more at
    /// <https://en.wikibooks.org/wiki/Haskell/Fix_and_recursion>
    pub trait FixedPoint {
        type In;
        type Out;

        #[allow(clippy::type_complexity)]
        fn step<'a>(
            rec: Lazy<'a, BoxFn<'a, Self::In, Self::Out>>,
        ) -> BoxFn<'a, Self::In, Self::Out>;

        fn get_impl() -> Box<dyn FnOnce(Self::In) -> Self::Out>
        where
            Self: 'static,
        {
            fix(&Self::step)
        }

        fn eval(x: Self::In) -> Self::Out
        where
            Self: 'static,
        {
            Self::get_impl()(x)
        }
    }

    fn factorial() -> Box<dyn FnOnce(u64) -> u64> {
        // f: Lazy<u64 -> u64> -> u64 -> u64
        fix(&|fac: Lazy<'_, Box<dyn FnOnce(u64) -> u64>>| {
            Box::new(move |n| if n == 0 { 1 } else { n * fac()(n - 1) })
        })
    }

    #[test]
    fn test_factorial() {
        assert_eq!(factorial()(6), 720);
    }
}

use fix_impl::{BoxFn, FixedPoint, Lazy};

struct Factorial;

impl FixedPoint for Factorial {
    type In = u64;
    type Out = u64;

    fn step<'a>(rec: Lazy<'a, BoxFn<'a, Self::In, Self::Out>>) -> BoxFn<'a, Self::In, Self::Out> {
        Box::new(move |n| if n == 0 { 1 } else { n * rec()(n - 1) })
    }
}

#[test]
fn test_factorial() {
    assert_eq!(Factorial::eval(6), 720);
}

struct Remainder;

/// Exercise 5.3 Suppose we define a recursive function:
/// remainder :: Integer -> Integer -> Integer
/// remainder a b = if a < b then a
///                 else remainder (a − b) b
/// Rewrite this function using fix so that it is not recursive.
impl FixedPoint for Remainder {
    type In = (u64, u64);
    type Out = u64;

    fn step<'a>(rec: Lazy<'a, BoxFn<'a, Self::In, Self::Out>>) -> BoxFn<'a, Self::In, Self::Out> {
        Box::new(move |(a, b)| if a < b { a } else { rec()((a - b, b)) })
    }
}

#[test]
fn test_remainder() {
    assert_eq!(Remainder::eval((5428, 100)), 28);
    assert_eq!(Remainder::eval((100, 328)), 100);
}

/// Exercise 5.4
/// Using list comprehensions, define a function `apPairs`
/// such that `apPairs aps1 aps2` is a list of all combinations of the absolute
/// pitches in `aps1` and `aps2` . Furthermore, for each pair `(ap1, ap2)` in the result,
/// the absolute value of `ap1−ap2` must be greater than two and less than eight.
fn ap_pairs(aps1: &[AbsPitch], aps2: &[AbsPitch]) -> Vec<(AbsPitch, AbsPitch)> {
    aps1.iter()
        .flat_map(|&ap1| aps2.iter().map(move |&ap2| (ap1, ap2)))
        .filter(|(ap1, ap2)| {
            let diff = (*ap1 - *ap2).get_inner().abs();
            (3..8).contains(&diff)
        })
        .collect()
}

// TODO: play me
/// Finally, write a function to turn the result of `apPairs` into a `Music Pitch`
/// value by playing each pair of pitches in parallel, and stringing them all
/// together sequentially. Try varying the rhythm by, for example, using an
/// eighth note when the first absolute pitch is odd, and a sixteenth note when
/// it is even, or some other criterion.
fn generate_music_with_pairs(aps1: &[AbsPitch], aps2: &[AbsPitch]) -> Music {
    let pairs = ap_pairs(aps1, aps2);

    Music::line(
        pairs
            .into_iter()
            .map(|(ap1, ap2)| {
                let dur = if u8::from(ap1.get_inner()) % 2 == 0 {
                    Dur::SIXTEENTH
                } else {
                    Dur::EIGHTH
                };

                Music::note(dur, ap1.into()) | Music::note(dur, ap2.into())
            })
            .collect(),
    )
}

/// Exercise 5.7
/// Rewrite this example:
/// map (λx → (x + 1)/2) xs
/// using a composition of sections.
///
/// Exercise 5.8
/// Then rewrite the earlier example:
/// as a “map of a map” (i.e. using two maps).
#[test]
fn lambda_as_two_sections() {
    let v = vec![4, 10, 17, 63, 11];
    let init = |x| (x + 1) / 2;
    assert_eq!(
        v.iter().copied().map(init).collect::<Vec<_>>(),
        [2, 5, 9, 32, 6]
    );

    let comp = compose(|x| x + 1, |x| x / 2);
    assert_eq!(
        v.iter().copied().map(comp).collect::<Vec<_>>(),
        [2, 5, 9, 32, 6]
    );

    // map (+1) $ map (/2)
    assert_eq!(
        v.into_iter()
            .map(|x| x + 1)
            .map(|x| x / 2)
            .collect::<Vec<_>>(),
        [2, 5, 9, 32, 6]
    );
}

/// Exercise 5.10
/// Using higher-order functions introduced in this chapter,
/// fill in the two missing functions, `f1` and `f2` ,
/// in the evaluation below so that it is valid:
/// f1 (f2 (*) [1, 2, 3, 4]) 5 -> [5, 10, 15, 20]
#[test]
fn use_higher_order_fns() {
    let v = vec![1_u32, 2, 3, 4];

    let f1 = |fs: Box<dyn Iterator<Item = Box<dyn Fn(u32) -> u32>>>, x| fs.map(move |f| f(x));
    let f2 = Iterator::map;
    assert_eq!(
        f1(
            Box::new(f2(v.into_iter(), |x| {
                Box::new(move |y| x * y) as Box<dyn Fn(u32) -> u32>
            })),
            5
        )
        .collect::<Vec<_>>(),
        [5, 10, 15, 20]
    )
}
