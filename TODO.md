- improve ergonomics
  - remove some functions (like roll or trill) in favor of PhraseAttributes
  - pub everything that can be used

- made example with listing every playable example and a way to save or play it

- made iterator-based version of Sequential and everything else to allow
  playing infinite music

- run MIDI server:
  - `timidity -iA -Os`
  - `fluidsynth --audio-driver=alsa -o audio.alsa.device=pulse /usr/share/sounds/sf2/FluidR3_GM.sf2`
