#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Volume(pub u8);

impl Volume {
    pub const fn softest() -> Self {
        Self(0)
    }

    pub const fn loudest() -> Self {
        Self(127)
    }
}
