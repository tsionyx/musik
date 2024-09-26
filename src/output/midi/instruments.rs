//! The list of MIDI instruments
//!
//! <https://www.midi.org/specifications-old/item/gm-level-1-sound-set>
//! <https://soundprogramming.net/file-formats/general-midi-instrument-list/>

use enum_iterator::Sequence;
use enum_map::Enum;
use ux2::u7;

use crate::{
    music::Music,
    prim::{duration::Dur, pitch::AbsPitch},
};

// https://github.com/rust-lang/rfcs/issues/284#issuecomment-1592343574
#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Hash, Enum, Sequence)]
#[allow(missing_docs)]
pub enum Instrument {
    AcousticGrandPiano,
    BrightAcousticPiano,
    ElectricGrandPiano,
    HonkyTonkPiano,
    RhodesPiano,
    ChorusedPiano,
    Harpsichord,
    Clavinet,
    Celesta,
    Glockenspiel,
    MusicBox,
    Vibraphone,
    Marimba,
    Xylophone,
    TubularBells,
    Dulcimer,
    HammondOrgan,
    PercussiveOrgan,
    RockOrgan,
    ChurchOrgan,
    ReedOrgan,
    Accordion,
    Harmonica,
    TangoAccordion,
    AcousticGuitarNylon,
    AcousticGuitarSteel,
    ElectricGuitarJazz,
    ElectricGuitarClean,
    ElectricGuitarMuted,
    OverdrivenGuitar,
    DistortionGuitar,
    GuitarHarmonics,
    AcousticBass,
    ElectricBassFingered,
    ElectricBassPicked,
    FretlessBass,
    SlapBass1,
    SlapBass2,
    SynthBass1,
    SynthBass2,
    Violin,
    Viola,
    Cello,
    Contrabass,
    TremoloStrings,
    PizzicatoStrings,
    OrchestralHarp,
    Timpani,
    StringEnsemble1,
    StringEnsemble2,
    SynthStrings1,
    SynthStrings2,
    ChoirAahs,
    VoiceOohs,
    SynthVoice,
    OrchestraHit,
    Trumpet,
    Trombone,
    Tuba,
    MutedTrumpet,
    FrenchHorn,
    BrassSection,
    SynthBrass1,
    SynthBrass2,
    SopranoSax,
    AltoSax,
    TenorSax,
    BaritoneSax,
    Oboe,
    Bassoon,
    EnglishHorn,
    Clarinet,
    Piccolo,
    Flute,
    Recorder,
    PanFlute,
    BlownBottle,
    Shakuhachi,
    Whistle,
    Ocarina,
    Lead1Square,
    Lead2Sawtooth,
    Lead3Calliope,
    Lead4Chiff,
    Lead5Charang,
    Lead6Voice,
    Lead7Fifths,
    Lead8BassLead,
    Pad1NewAge,
    Pad2Warm,
    Pad3Polysynth,
    Pad4Choir,
    Pad5Bowed,
    Pad6Metallic,
    Pad7Halo,
    Pad8Sweep,
    FX1Train,
    FX2Soundtrack,
    FX3Crystal,
    FX4Atmosphere,
    FX5Brightness,
    FX6Goblins,
    FX7Echoes,
    FX8SciFi,
    Sitar,
    Banjo,
    Shamisen,
    Koto,
    Kalimba,
    Bagpipe,
    Fiddle,
    Shanai,
    TinkleBell,
    Agogo,
    SteelDrums,
    Woodblock,
    TaikoDrum,
    MelodicDrum,
    SynthDrum,
    ReverseCymbal,
    GuitarFretNoise,
    BreathNoise,
    Seashore,
    BirdTweet,
    TelephoneRing,
    Helicopter,
    Applause,
    Gunshot,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Enum, Sequence)]
#[allow(missing_docs)]
pub enum PercussionSound {
    // the following comments specify the location of the instrument
    // on the musical score sheet (as if it were a treble clef)
    //
    // <https://en.wikipedia.org/wiki/Percussion_notation#Key_or_legend_for_drum_kit>
    // <https://drumbeatsonline.com/blog/drum-notation-sheet-music-how-to-read-it>
    AcousticBassDrum, // F4
    BassDrum1,        // E4
    SideStick,        // C5 with diagonal slash through note head
    AcousticSnare,    // C5
    HandClap,         // -
    ElectricSnare,    // C5
    LowFloorTom,      // G4
    ClosedHiHat,      // G5 with X head
    HighFloorTom,     // A4
    PedalHiHat,       // D4 with X head
    LowTom,           // B4
    OpenHiHat,        // G5 with diamond head or small circle above
    LowMidTom,        // D5
    HiMidTom,         // E5
    CrashCymbal1,     // A5 with X head
    HighTom,          // F5
    RideCymbal1,      // F5 with X head
    ChineseCymbal,    // C6 with X head (+maybe encircled)
    RideBell,         // F5 with diamond head?
    Tambourine,       // B4 with triangle head
    SplashCymbal,     // B5 with X head (or C6 with X head)
    Cowbell,          // E5 with triangle head
    CrashCymbal2,     // B5 with X head
    Vibraslap,        // -
    RideCymbal2,      // D5 with X head
    HiBongo,          // Bongos and congas
    LowBongo,         //   are normally notated
    MuteHiConga,      //   on a two lined staff,
    OpenHiConga,      //   with one line representing
    LowConga,         //   each of the drums.
    HighTimbale,      // -
    LowTimbale,       // -
    HighAgogo,        // -
    LowAgogo,         // -
    Cabasa,           // -
    Maracas,          // -
    ShortWhistle,     // -
    LongWhistle,      // -
    ShortGuiro,       // -
    LongGuiro,        // -
    Claves,           // -
    HiWoodBlock,      // D5 with triangle head
    LowWoodBlock,     // C5 with triangle head
    MuteCuica,        // -
    OpenCuica,        // -
    MuteTriangle,     // A5 with triangle head and stopped symbol ('+' above)
    OpenTriangle,     // A5 with triangle head
}

impl PercussionSound {
    /// Produce a MIDI note for the [`PercussionSound`].
    ///
    /// The corresponding MIDI notes are spanned
    /// the same interval as MIDI 35..=81, i.e. B1..=A5 pitches.
    pub fn note(self, dur: Dur) -> Music {
        let midi_key = u7::try_from(self.into_usize())
            .expect("<=46 fits into u7")
            .checked_add(u7::new(35))
            .expect("<=81 fits into u7");
        Music::note(dur, AbsPitch::from(midi_key).into())
    }
}
