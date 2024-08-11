# Musik

Musical theory and audio signals concepts expressed in Rust.
It is mostly expired by [HSoM](https://www.euterpea.com/haskell-school-of-music/) book.

## Install dependencies

```bash
sudo apt install libasound2-dev   # also `qjackctl` and `qsynth` for handling audio streams
```

## Examples

You could listen to your code examples (or [included example](examples/hsom-exercises))
by compiling with the `play-midi` feature (included by default).

1. Run one of the external MIDI servers, e.g.:

    - ```shell
      timidity -iA -Os
      ```

    - ```shell
      fluidsynth --audio-driver=alsa -o audio.alsa.device=pulse /usr/share/sounds/sf2/FluidR3_GM.sf2
      ```

2. Run your code, e.g. `cargo run --example hsom-exercises`.

## Useful Links

- On notes and everything else:
    - https://en.wikipedia.org/wiki/Music_theory
    - https://musictheory.pugetsound.edu/mt21c/
    - https://en.wikipedia.org/wiki/Music_and_mathematics
    - https://en.wikipedia.org/wiki/Algorithmic_composition
    - https://en.wikipedia.org/wiki/Computational_musicology
    - https://wiki.ccarh.org/wiki/Guido_Music_Notation

- On MIDI:
    - http://tedfelix.com/linux/linux-midi.html
    - https://sndrtj.eu/2019/10/19/Using-a-USB-midi-keyboard-on-Ubuntu/
    - https://askubuntu.com/q/572120
    - https://linuxaudio.github.io/libremusicproduction/html/workflow.html
    - https://www.audiolabs-erlangen.de/resources/MIR/FMP/C1/C1S2_MIDI.html (also see the FMP book)

- On signals:
    - https://www.youtube.com/watch?v=odeWLp96fdo
    - https://crates.io/crates/hound
    - https://crates.io/crates/rodio
    - https://crates.io/crates/fundsp (see the `combinator` module for signal processing arrows-style)

### Similar crates

- https://crates.io/crates/rust-music-theory
- https://crates.io/crates/rust-music
- https://crates.io/crates/tune-cli
- https://crates.io/crates/tonal
- search crates.io: https://crates.io/search?q=WORD where WORD: {music, sound, melody, chord, harmony, note, pitch,
  octave}.
