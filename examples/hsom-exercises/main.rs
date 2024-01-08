#![allow(dead_code)]

mod ch1;
mod ch2;
mod ch3;

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
