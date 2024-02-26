use std::{
    cmp::Ordering,
    ops::{Add, Div, Mul, Sub},
};

use num_rational::Ratio;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// <https://en.wikipedia.org/wiki/Duration_(music)>
/// <https://en.wikipedia.org/wiki/Note_value>
pub struct Dur(u8, u8);

impl PartialOrd for Dur {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Dur {
    fn cmp(&self, other: &Self) -> Ordering {
        self.into_ratio::<u8>().cmp(&other.into_ratio())
    }
}

impl Dur {
    const fn from_integer(i: u8) -> Self {
        Self(i, 1)
    }

    const fn new(num: u8, denom: u8) -> Self {
        Self(num, denom)
    }

    pub fn into_ratio<T>(self) -> Ratio<T>
    where
        T: From<u8> + Clone + num_integer::Integer,
    {
        Ratio::new(T::from(self.0), T::from(self.1))
    }

    pub const ZERO: Self = Self::from_integer(0);
    pub const BN: Self = Self::from_integer(2); // brevis note
    pub const WN: Self = Self::from_integer(1); // whole note
    pub const HN: Self = Self::new(1, 2); // half note
    pub const QN: Self = Self::new(1, 4); // quarter note
    pub const EN: Self = Self::new(1, 8); // eighth note
    pub const SN: Self = Self::new(1, 16); // sixteenth note
    pub const TN: Self = Self::new(1, 32); // thirty-second note
    pub const SFN: Self = Self::new(1, 64); // sixty-fourth note

    pub const DWN: Self = Self::new(3, 2); // dotted whole note
    pub const DHN: Self = Self::new(3, 4); // dotted half note
    pub const DQN: Self = Self::new(3, 8); // dotted quarter note
    pub const DEN: Self = Self::new(3, 16); // dotted eighth note
    pub const DSN: Self = Self::new(3, 32); // dotted sixteenth note
    pub const DTN: Self = Self::new(3, 64); // dotted thirty-second note

    pub const DDHN: Self = Self::new(7, 8); // double-dotted half note
    pub const DDQN: Self = Self::new(7, 16); // double-dotted quarter note
    pub const DDEN: Self = Self::new(7, 32); // double-dotted eighth note

    pub const fn double(self) -> Self {
        if self.1 & 1 == 0 {
            //&Self::WN => Self::BN,
            //&Self::HN => Self::WN,
            //&Self::QN => Self::HN,
            //&Self::EN => Self::QN,
            //&Self::SN => Self::EN,
            //&Self::TN => Self::SN,
            //&Self::SFN => Self::TN,
            //
            //&Self::DHN => Self::DWN,
            //&Self::DQN => Self::DHN,
            //&Self::DEN => Self::DQN,
            //&Self::DSN => Self::DEN,
            //&Self::DTN => Self::DSN,
            //
            //&Self::DDQN => Self::DDHN,
            //&Self::DDEN => Self::DDQN,

            Self::new(self.0, self.1 >> 1)
        } else {
            Self::new(self.0 << 1, self.1)
        }
    }

    pub const fn halve(self) -> Self {
        if self.0 & 1 == 0 {
            Self::new(self.0 >> 1, self.1)
        } else {
            Self::new(self.0, self.1 << 1)
        }
    }

    pub const fn dotted(self) -> Self {
        let self_ = self.halve();
        Self::new(self_.0 * 3, self_.1)
    }

    pub fn saturating_sub(self, rhs: Self) -> Self {
        if self > rhs {
            self - rhs
        } else {
            Self::ZERO
        }
    }
}

impl From<u8> for Dur {
    fn from(value: u8) -> Self {
        Self::new(value, 1)
    }
}

impl From<Ratio<u8>> for Dur {
    fn from(value: Ratio<u8>) -> Self {
        Self::new(*value.numer(), *value.denom())
    }
}

impl Add for Dur {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        (self.into_ratio() + rhs.into_ratio()).into()
    }
}

impl Sub for Dur {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        (self.into_ratio() - rhs.into_ratio()).into()
    }
}

impl Mul<u8> for Dur {
    type Output = Self;

    fn mul(self, rhs: u8) -> Self::Output {
        (self.into_ratio() * rhs).into()
    }
}

impl Mul<Ratio<u8>> for Dur {
    type Output = Self;

    fn mul(self, rhs: Ratio<u8>) -> Self::Output {
        (self.into_ratio() * rhs).into()
    }
}

impl Div<u8> for Dur {
    type Output = Self;

    fn div(self, rhs: u8) -> Self::Output {
        (self.into_ratio() / rhs).into()
    }
}

impl Div<Ratio<u8>> for Dur {
    type Output = Self;

    fn div(self, rhs: Ratio<u8>) -> Self::Output {
        (self.into_ratio() / rhs).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn double_duration() {
        assert_eq!(Dur::WN.double(), Dur::BN);
        assert_eq!(Dur::HN.double(), Dur::WN);
        assert_eq!(Dur::QN.double(), Dur::HN);
        assert_eq!(Dur::EN.double(), Dur::QN);
        assert_eq!(Dur::SN.double(), Dur::EN);
        assert_eq!(Dur::TN.double(), Dur::SN);
        assert_eq!(Dur::SFN.double(), Dur::TN);

        assert_eq!(Dur::DHN.double(), Dur::DWN);
        assert_eq!(Dur::DQN.double(), Dur::DHN);
        assert_eq!(Dur::DEN.double(), Dur::DQN);
        assert_eq!(Dur::DSN.double(), Dur::DEN);
        assert_eq!(Dur::DTN.double(), Dur::DSN);

        assert_eq!(Dur::DDQN.double(), Dur::DDHN);
        assert_eq!(Dur::DDEN.double(), Dur::DDQN);
    }
}
