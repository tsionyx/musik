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
    AcousticBassDrum,
    BassDrum1,
    SideStick,
    AcousticSnare,
    HandClap,
    ElectricSnare,
    LowFloorTom,
    ClosedHiHat,
    HighFloorTom,
    PedalHiHat,
    LowTom,
    OpenHiHat,
    LowMidTom,
    HiMidTom,
    CrashCymbal1,
    HighTom,
    RideCymbal1,
    ChineseCymbal,
    RideBell,
    Tambourine,
    SplashCymbal,
    Cowbell,
    CrashCymbal2,
    Vibraslap,
    RideCymbal2,
    HiBongo,
    LowBongo,
    MuteHiConga,
    OpenHiConga,
    LowConga,
    HighTimbale,
    LowTimbale,
    HighAgogo,
    LowAgogo,
    Cabasa,
    Maracas,
    ShortWhistle,
    LongWhistle,
    ShortGuiro,
    LongGuiro,
    Claves,
    HiWoodBlock,
    LowWoodBlock,
    MuteCuica,
    OpenCuica,
    MuteTriangle,
    OpenTriangle,
}

impl PercussionSound {
    /// Produce a MIDI note for the [`PercussionSound`].
    pub fn note(self, dur: Dur) -> Music {
        let midi_key = u7::try_from(self.into_usize())
            .expect("<=46 fits into u7")
            .checked_add(u7::new(35))
            .expect("<=81 fits into u7");
        Music::note(dur, AbsPitch::from(midi_key).into())
    }
}
