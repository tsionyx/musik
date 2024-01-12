#![allow(dead_code)]

mod ch1;
mod ch2;
mod ch3;
mod ch4;
mod ch5;

fn simple<A, B, C, D, E>(x: A, y: B, z: C) -> E
where
    A: std::ops::Mul<D, Output = E>,
    B: std::ops::Add<C, Output = D>,
{
    x * (y + z)
}

fn main() {
    println!(
        "This crate contains a collection of tests \
    representing exercises from the 'Haskell School of Music' bool"
    )
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
