//! Helper macros for creating octaves from number.

#[macro_export]
/// Create an [`Octave`][crate::Octave]
/// using numbers
///
/// ```
/// # use musik::{o, Octave};
/// assert_eq!(o!(-1), Octave::OctoContra);
/// assert_eq!(o!(4), Octave::OneLined);
/// assert_eq!(o!(9), Octave::SixLined);
macro_rules! o {
    ($octave:expr) => {{
        const _: () = assert!($octave >= -1, "Octave number should be at least -1");
        const _: () = assert!($octave < 10, "Octave number should be less than 10");
        match $octave {
            -1 => $crate::Octave::OctoContra,
            0 => $crate::Octave::SubContra,
            1 => $crate::Octave::Contra,
            2 => $crate::Octave::Great,
            3 => $crate::Octave::Small,
            4 => $crate::Octave::OneLined,
            5 => $crate::Octave::TwoLined,
            6 => $crate::Octave::ThreeLined,
            7 => $crate::Octave::FourLined,
            8 => $crate::Octave::FiveLined,
            9 => $crate::Octave::SixLined,
            _ => unreachable!("Invalid octave number should be handled by assertions"),
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::super::super::interval::Octave;

    #[test]
    fn all_octaves() {
        assert_eq!(o!(-1), Octave::OctoContra);
        assert_eq!(o!(0), Octave::SubContra);
        assert_eq!(o!(1), Octave::Contra);
        assert_eq!(o!(2), Octave::Great);
        assert_eq!(o!(3), Octave::Small);
        assert_eq!(o!(4), Octave::OneLined);
        assert_eq!(o!(5), Octave::TwoLined);
        assert_eq!(o!(6), Octave::ThreeLined);
        assert_eq!(o!(7), Octave::FourLined);
        assert_eq!(o!(8), Octave::FiveLined);
        assert_eq!(o!(9), Octave::SixLined);
    }
}
