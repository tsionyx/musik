//! Musical theory and audio signals concepts expressed in Rust

// TODO: move to Cargo.toml's [lints] when on >=1.74
//  https://doc.rust-lang.org/nightly/cargo/reference/manifest.html#the-lints-section

// do not warn on older Rust versions
#![allow(unknown_lints)]
//
// The following list was generated with the command
//   $ rustc -W help | grep ' allow ' | awk '{print $1}' | tr - _ | sort | xargs -I{} echo '#![warn({})]'
#![warn(absolute_paths_not_starting_with_crate)]
// deprecated in 1.81
// #![allow(box_pointers)]
#![warn(deprecated_in_future)]
#![warn(deprecated_safe)]
#![warn(elided_lifetimes_in_paths)]
#![warn(explicit_outlives_requirements)]
#![warn(ffi_unwind_calls)]
#![warn(fuzzy_provenance_casts)]
#![warn(impl_trait_overcaptures)]
#![warn(keyword_idents_2018)]
#![warn(keyword_idents_2024)]
#![warn(let_underscore_drop)]
#![warn(lossy_provenance_casts)]
#![warn(macro_use_extern_crate)]
#![warn(meta_variable_misuse)]
#![warn(missing_abi)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(missing_unsafe_on_extern)]
#![warn(multiple_supertrait_upcastable)]
#![warn(must_not_suspend)]
#![warn(non_ascii_idents)]
#![warn(non_exhaustive_omitted_patterns)]
#![warn(non_local_definitions)]
#![warn(redundant_lifetimes)]
#![warn(rust_2021_incompatible_closure_captures)]
#![warn(rust_2021_incompatible_or_patterns)]
#![warn(rust_2021_prefixes_incompatible_syntax)]
#![warn(rust_2021_prelude_collisions)]
#![warn(rust_2024_incompatible_pat)]
#![warn(single_use_lifetimes)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unit_bindings)]
#![warn(unnameable_types)]
// conflicts with the `clippy::redundant_pub_crate`
#![allow(unreachable_pub)]
// !!! NO UNSAFE
#![forbid(unsafe_code)]
#![warn(unsafe_op_in_unsafe_fn)]
#![warn(unstable_features)]
// TODO: enable after fixing false-positive detecting
//  no usage of [dev-dependencies] that are used in examples
// #![warn(unused_crate_dependencies)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_lifetimes)]
#![warn(unused_macro_rules)]
#![warn(unused_qualifications)]
#![warn(unused_results)]
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

mod instruments;
pub mod music;
mod output;
mod prim;

pub use self::{
    instruments::InstrumentName,
    music::{
        perf::{self, metro, Performable, Performance, Player},
        phrase::{self as attributes, PhraseAttribute},
        Music, NoteAttribute, Temporal,
    },
    output::midi,
    prim::{
        duration::Dur,
        helpers::{self, pitch_class::accidentals},
        interval::{ErrorOctaveTryFromNum, Interval, Octave},
        pitch::{AbsPitch, ErrorPitchClipping, Pitch, PitchClass},
        scale::KeySig,
        volume::Volume,
    },
};
