#![allow(dead_code)]

mod ch1;
mod ch2;
mod ch3;
mod ch4;
mod ch5;
mod ch6;
mod ch7;
mod ch8;

fn main() {
    use musik::Performable as _;
    let m = ch6::drum_pattern();
    m.perform().play().unwrap();
}
