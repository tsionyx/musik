- improve ergonomics
  - mandatory doc comments
  - rename enums and functions
  - impl Into
  - remove some functions (like roll or trill) in favor of PhraseAttributes
  - provide macro for easy creation of notes and durs:
    - dur!(1, 2), dur!(1/2), Dur::HALF, n!(C4, HALF), n!(Df5, QUARTER)
  - pub everything that can be used
  - prelude with Music, Dur, Pitch, Instruments, Performable
  - use logs!

- made example with listing every playable example and a way to save or play it

- improve docs

- use the reduced integers from uX

- made iterator-based version of Sequential and everything else to allow
  playing infinite music

- introduce 'midi' (save to .mid) feature;
