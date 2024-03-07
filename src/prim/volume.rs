use ux2::u7;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Volume(pub(crate) u7);

impl Volume {
    pub const fn softest() -> Self {
        Self(u7::MIN)
    }

    pub const fn loudest() -> Self {
        Self(u7::MAX)
    }

    pub const fn get_inner(self) -> u7 {
        self.0
    }
}

impl From<u8> for Volume {
    fn from(value: u8) -> Self {
        // if the conversion fails, the number is simply clipped
        Self(u7::try_from(value).unwrap_or(u7::MAX))
    }
}
