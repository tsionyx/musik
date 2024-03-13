- improve ergonomics
  - mandatory doc comments
  - remove some functions (like roll or trill) in favor of PhraseAttributes
  - provide macro for easy creation of notes and durs:
    - n!(C4, HALF), n!(Df5, QUARTER), [Db5 / 4, C#4 / 8](https://wiki.ccarh.org/wiki/Guido_Music_Notation)
  - pub everything that can be used

- made example with listing every playable example and a way to save or play it

- made iterator-based version of Sequential and everything else to allow
  playing infinite music
