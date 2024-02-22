#![allow(dead_code)]

mod ch1;
mod ch2;
mod ch3;
mod ch4;
mod ch5;
mod ch6;
mod ch7;
mod ch8;

fn simple<A, B, C, D, E>(x: A, y: B, z: C) -> E
where
    A: std::ops::Mul<D, Output = E>,
    B: std::ops::Add<C, Output = D>,
{
    x * (y + z)
}

fn main() {
    use musik::{instruments::StandardMidiInstrument::*, Interval, Performable as _};

    // let perf = ch6::funk_groove().perform_default();
    // perf.clone().save_to_file("funk.mid").unwrap();
    // perf.play().unwrap();
    ch6::crazy_recursion::example1()
        .perform_default()
        .play()
        .unwrap();

    ch6::shepard_scale::music(
        -Interval::semi_tone(),
        &[
            (AcousticGrandPiano, 2323),
            (DistortionGuitar, 9940),
            (Flute, 7899),
            (Cello, 15000),
        ],
    )
    .perform_default()
    .play()
    .unwrap();
}

fn compose<T, U, V, F, G>(f: F, g: G) -> impl Fn(T) -> V
where
    F: Fn(T) -> U,
    G: Fn(U) -> V,
{
    move |x| g(f(x))
}

#[test]
fn compose_test() {
    let duplicate = |v: Vec<_>| v.clone().into_iter().chain(v).collect();
    let size = |v: Vec<_>| v.len();

    let size_of_duplicated = compose(duplicate, size);
    assert_eq!(size_of_duplicated(vec![1, 2, 3, 4]), 8);
}
