//! Musical theory and audio signals concepts expressed in Rust

// do not warn on older Rust versions
#![allow(unknown_lints)]
//
// The following list was generated with the command
//   $ rustc -W help | grep ' allow ' | awk '{print $1}' | tr - _ | sort | xargs -I{} echo '#![warn({})]'
#![warn(absolute_paths_not_starting_with_crate)]
#![allow(box_pointers)]
#![warn(deprecated_in_future)]
#![warn(elided_lifetimes_in_paths)]
#![warn(explicit_outlives_requirements)]
#![warn(ffi_unwind_calls)]
#![warn(keyword_idents)]
#![warn(let_underscore_drop)]
#![warn(macro_use_extern_crate)]
#![warn(meta_variable_misuse)]
#![warn(missing_abi)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(non_ascii_idents)]
#![warn(pointer_structural_match)]
#![warn(rust_2021_incompatible_closure_captures)]
#![warn(rust_2021_incompatible_or_patterns)]
#![warn(rust_2021_prefixes_incompatible_syntax)]
#![warn(rust_2021_prelude_collisions)]
#![warn(single_use_lifetimes)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
// conflicts with the `clippy::redundant_pub_crate`
#![allow(unreachable_pub)]
// !!! NO UNSAFE
#![forbid(unsafe_code)]
#![warn(unsafe_op_in_unsafe_fn)]
#![warn(unstable_features)]
#![warn(unused_crate_dependencies)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_lifetimes)]
#![warn(unused_macro_rules)]
#![warn(unused_qualifications)]
#![warn(unused_results)]
#![warn(unused_tuple_struct_fields)]
#![warn(variant_size_differences)]
//
// additional recommendations
#![deny(clippy::mem_forget)]
// suppress some pedantic warnings
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
// `use super::*` in tests
#![cfg_attr(test, allow(clippy::wildcard_imports))]
// TODO: remove when not 1.67
#![allow(clippy::uninlined_format_args)]
// using `expect` is almost always better, but `unwrap` still allowed in tests
#![cfg_attr(not(test), warn(clippy::unwrap_used))]
// #![warn(clippy::expect_used)]

pub mod instruments;
pub mod music;
mod output;
mod prim;

pub use self::{
    music::{
        perf::{self, metro, Performable, Performance, Player},
        phrase::{self as attributes, PhraseAttribute},
        Music, Temporal,
    },
    output::midi,
    prim::{
        duration::Dur,
        interval::{ErrorOctaveTryFromNum, Interval, Octave},
        pitch::{AbsPitch, ErrorPitchClipping, Pitch, PitchClass},
        volume::Volume,
    },
};
