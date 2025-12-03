//! Helper macro for creating primitive [`Music`]
//! values using simplified notations as introduced
//! with the [note macro][crate::n].

#[macro_export]
/// Create a primitive [`Music`] value (notes and rests)
/// or the succession of those values.
macro_rules! m {
    (_/ $($rhythm:tt)+) => {
        $crate::Music::rest($crate::n!(_ / $($rhythm)+))
    };


    ($( {$($n_args:tt)+} ),+ $(,)?) => {
        $crate::Music::line(
            vec![
                $(
                  $crate::m!($($n_args)+)
                ),+
            ]
        )
    };

    ($($n_args:tt)+) => {
        $crate::Music::Prim($crate::n!($($n_args)+).into())
    };
}
