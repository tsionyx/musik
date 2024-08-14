#![allow(clippy::wildcard_imports, clippy::enum_glob_use)]

//! Playable exercises from the 'Haskell School of Music' book
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use ux2::u7;

use musik::{p, AbsPitch, Dur, Interval, Music, Performable as _, Pitch, Volume};

mod ch1;
mod ch2;
mod ch3;
mod ch4;
mod ch5;
mod ch6;
mod ch7;
mod ch8;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cli = Cli::parse();
    let m: Music<(Pitch, Volume)> = match cli.chapter {
        Chapter::Ch1(a) => match a.sample {
            Chapter1::Mel => ch1::harmonic::mel([p!(C 4), p!(E 4), p!(G 4)]).into(),
        },
        Chapter::Ch2(a) => match a.sample {
            Chapter2::T251 => ch2::t251().into(),
            Chapter2::TwoFiveOne => ch2::two_five_one(p!(Bb 3), Dur::HALF).into(),
            Chapter2::Blues => ch2::blues::melody().into(),
        },
        Chapter::Ch3(a) => match a.sample {
            Chapter3::Staccato => Music::line(ch3::staccato(
                p!(G 4)
                    .natural_minor_scale()
                    .map(|p| (Dur::QUARTER, (p, Volume::loudest())).into())
                    .collect(),
            )),
            Chapter3::Chromatic => ch3::chromatic::chrom(p!(C 5), p!(F 5)).into(),
            Chapter3::BrotherJohn => ch3::brother_john::frere_jacques_four_part_round().into(),
        },
        Chapter::Ch4(a) => match a.sample {
            Chapter4::ChildSong => ch4::child_song_6().into(),
            Chapter4::Prefixed1 => ch4::prefixed_mel_1().into(),
            Chapter4::Prefixed2 => ch4::prefixed_mel_2().into(),
        },
        Chapter::Ch5(a) => match a.sample {
            Chapter5::WithPairs => {
                let f1: Vec<AbsPitch> = [61, 80, 77, 81, 77]
                    .into_iter()
                    .map(|x| u7::new(x).into())
                    .collect();
                let f2: Vec<AbsPitch> = [64, 70, 72, 75, 81, 66]
                    .into_iter()
                    .map(|x| u7::new(x).into())
                    .collect();
                ch5::generate_music_with_pairs(&f1, &f2).into()
            }
        },
        Chapter::Ch6(a) => match a.sample {
            Chapter6::StarsAndStripes => ch6::stars_and_stripes().into(),
            Chapter6::FunkGroove => ch6::funk_groove().into(),
            Chapter6::Percussion => ch6::sequence_all_percussions().into(),
            Chapter6::Drum => ch6::drum_pattern().into(),
            Chapter6::Volumed => ch6::test_volume(Volume::loudest()),
            Chapter6::InsideOut => ch6::inside_out::example().into(),
            Chapter6::Recursion1 => ch6::crazy_recursion::example1().into(),
            Chapter6::Recursion2 => ch6::crazy_recursion::example2().into(),
            Chapter6::ShepardAsc => {
                use musik::midi::Instrument::*;
                ch6::shepard_scale::music(
                    Interval::semi_tone(),
                    &[
                        (AcousticGrandPiano, 18774),
                        (ElectricGuitarClean, 33300),
                        (Flute, 19231),
                        (Cello, 99),
                    ],
                )
            }
            Chapter6::ShepardDesc => {
                use musik::midi::Instrument::*;
                ch6::shepard_scale::music(
                    -Interval::semi_tone(),
                    &[
                        (AcousticGrandPiano, 2323),
                        (ElectricGuitarClean, 9940),
                        (Flute, 7899),
                        (Cello, 15000),
                    ],
                )
            }
        },
    };

    let perf = m.perform();
    if cli.mode.play {
        // TODO: make the whole flow lazy (test on `ch6 shepard-desc`):
        //  - introduce lazy `midly::Smf`, then check that it starts real sound almost instantly without any `take`;
        //  - not it behaves like (after fixing every overflow):
        //    - good on `perf.iter().take(1604)` and less;
        //    - fails on `Event::as_midi` on `perf.iter().take(1605)` (while calculating `self.start_time * ticks_per_second` (FIXED));
        //    - suspicious pauses (probably by overflows produced): listen for ~60s to detect.
        //    - to restore the previous (no pause behaviour):
        //      - change `LineConfig::scale` to return `Music::Line`;
        //      - `let perf = Performance::with_events(perf.iter().take(10_000));`
        perf.clone().play()?;
    }

    if let Some(path) = cli.mode.save_into {
        perf.save_to_file(path)?;
    }

    Ok(())
}

#[derive(Debug, Clone, Parser)]
/// Run examples from Haskell School of Music.
struct Cli {
    #[command(subcommand)]
    chapter: Chapter,

    #[command(flatten)]
    mode: Mode,
}

#[derive(Debug, Clone, Args)]
#[group(required = true, multiple = true)]
struct Mode {
    /// Play the example using default MIDI server
    /// (should run in the separate process, see README).
    #[arg(short, long)]
    play: bool,

    /// Save the example into .midi file
    #[arg(short('o'), long, value_name = "MIDI FILE")]
    save_into: Option<PathBuf>,
}

#[derive(Debug, Clone, Subcommand)]
enum Chapter {
    Ch1(ChArgs<Chapter1>),
    Ch2(ChArgs<Chapter2>),
    Ch3(ChArgs<Chapter3>),
    Ch4(ChArgs<Chapter4>),
    Ch5(ChArgs<Chapter5>),
    Ch6(ChArgs<Chapter6>),
}

#[derive(Debug, Copy, Clone, Args)]
struct ChArgs<T>
where
    T: Subcommand,
{
    #[command(subcommand)]
    sample: T,
}

#[derive(Debug, Copy, Clone, Subcommand)]
enum Chapter1 {
    Mel,
}

#[derive(Debug, Copy, Clone, Subcommand)]
enum Chapter2 {
    T251,
    TwoFiveOne,
    Blues,
}

#[derive(Debug, Copy, Clone, Subcommand)]
enum Chapter3 {
    Staccato,
    Chromatic,
    BrotherJohn,
}

#[derive(Debug, Copy, Clone, Subcommand)]
enum Chapter4 {
    ChildSong,
    Prefixed1,
    Prefixed2,
}

#[derive(Debug, Copy, Clone, Subcommand)]
enum Chapter5 {
    WithPairs,
}

#[derive(Debug, Copy, Clone, Subcommand)]
enum Chapter6 {
    StarsAndStripes,
    FunkGroove,
    Percussion,
    Drum,
    Volumed,
    InsideOut,
    Recursion1,
    Recursion2,
    ShepardAsc,
    ShepardDesc,
}
