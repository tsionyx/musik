#[allow(dead_code)]
/// Exercise 8.1
///
/// Gradually scale the volume of each event in the
/// performance by a factor of 1 through 1 + x,
/// using linear interpolation.
const fn scale_volume() {
    // this is already done in the definition of Fancy Player
    // see the inflate in the `fn fancy_interpret_phrase`.
}

// Exercises 8.2, 8.3, 8.5 are defined for the "Fancy" player

/// Exercise 8.4
///
/// Define a player `jazzMan` that plays a melody using a jazz “swing” feel.
/// Since there are different kinds and degrees of swing, we can be more specific
/// as follows: whenever there is a sequence of two eighth notes,
/// they should be interpreted instead as a quarter note followed by an eighth note,
/// but with tempo 3/2. So in essence, the first note is lengthened,
/// and the second note is shortened, so that the first note is twice as long as the second,
/// but they still take up the same amount of overall time.
///
/// (Hint: There are several ways to solve this problem.
/// One surprisingly effective and straightforward solution
/// is to implement `jazzMan` as a `NoteFun`, not a `PhraseFun`.
/// In jazz, if an eighth note falls on a quarter-note beat it is
/// said to fall on the “downbeat”, and the eighth notes that are in between are
/// said to fall on the “upbeat”.
///
/// For example, in the phrase `c4 en :+: d4 en :+: e4 en :+: f4 en`,
/// the `C` and `E` fall on the downbeat, and the `D` and `F` fall
/// on the upbeat. So to get a “swing feel,” the notes on the down beat need to
/// be lengthened, and ones on the upbeat need to be delayed and shortened.
/// Whether an event falls on a downbeat or upbeat can be determined from
/// the `start_time` and `duration` of the context.)
#[cfg(test)]
mod jazz_man {
    use num_rational::Ratio;

    use musik::{
        music::AttrNote,
        perf::{Context, DefaultPlayer, Event, EventAnnotator},
        Dur, Performance, PhraseAttribute, Player,
    };

    #[derive(Debug, Clone, Default)]
    struct SwingPlayer {
        default_player: DefaultPlayer,
    }

    impl Player<AttrNote> for SwingPlayer {
        fn name(&self) -> &'static str {
            "Jazz"
        }

        fn play_note(
            &self,
            (dur, (note_pitch, attrs)): (Dur, &AttrNote),
            ctx: Context<'_, AttrNote>,
        ) -> Performance {
            let start_time = ctx.start_time();
            let instrument = ctx.instrument().clone();
            let whole_note = ctx.whole_note();
            let transpose_interval = ctx.transpose_interval();
            let volume = ctx.volume();

            let number_of_beats_since_start = start_time / whole_note;
            // denom belongs to {1, 2, 4}
            let is_downbeat = 4 % (*number_of_beats_since_start.denom()) == 0;
            let is_upbeat = number_of_beats_since_start.denom() == &8;

            let (start_time, dur) = {
                // only for eight notes
                if dur == Dur::EIGHTH {
                    if is_downbeat {
                        (start_time, dur * Ratio::new(4, 3))
                    } else if is_upbeat {
                        let lengthened_on = Ratio::new(1, 24) * ctx.whole_note();
                        (start_time + lengthened_on, dur * Ratio::new(2, 3))
                    } else {
                        (start_time, dur)
                    }
                } else {
                    (start_time, dur)
                }
            };

            let init = Event {
                start_time,
                instrument,
                pitch: note_pitch.abs() + transpose_interval,
                duration: dur.into_ratio() * whole_note,
                volume,
                params: vec![],
            };

            let event = attrs.iter().fold(init, |acc, attr| {
                self.default_player.modify_event_with_attr(acc, attr, &ctx)
            });
            Performance::with_events(std::iter::once(event))
        }

        fn interpret_phrase(&self, perf: Performance, attr: &PhraseAttribute) -> Performance {
            <DefaultPlayer as Player<AttrNote>>::interpret_phrase(&self.default_player, perf, attr)
        }
    }

    mod tests {
        use ux2::u7;

        use musik::{
            midi::Instrument, AbsPitch, Music, Octave, Performable as _, Pitch, PitchClass, Volume,
        };

        use super::*;

        #[test]
        fn simple_swing() {
            let oc4 = Octave::OneLined;
            let m: Music<AttrNote> = Music::line(
                [PitchClass::C, PitchClass::D, PitchClass::E, PitchClass::F]
                    .into_iter()
                    .map(|pc| Music::note(Dur::EIGHTH, Pitch::new(pc, oc4)))
                    .collect(),
            )
            .into();

            let perf = m.with_player(SwingPlayer::default()).perform();

            assert_eq!(
                perf.iter().collect::<Vec<Event>>(),
                [
                    Event {
                        start_time: Ratio::from_integer(0),
                        instrument: Instrument::AcousticGrandPiano.into(),
                        pitch: AbsPitch::from(u7::new(60)),
                        duration: Ratio::new(1, 3),
                        volume: Volume::loudest(),
                        params: vec![]
                    },
                    Event {
                        start_time: Ratio::new(1, 3),
                        instrument: Instrument::AcousticGrandPiano.into(),
                        pitch: AbsPitch::from(u7::new(62)),
                        duration: Ratio::new(1, 6),
                        volume: Volume::loudest(),
                        params: vec![]
                    },
                    Event {
                        start_time: Ratio::new(1, 2),
                        instrument: Instrument::AcousticGrandPiano.into(),
                        pitch: AbsPitch::from(u7::new(64)),
                        duration: Ratio::new(1, 3),
                        volume: Volume::loudest(),
                        params: vec![]
                    },
                    Event {
                        start_time: Ratio::new(5, 6),
                        instrument: Instrument::AcousticGrandPiano.into(),
                        pitch: AbsPitch::from(u7::new(65)),
                        duration: Ratio::new(1, 6),
                        volume: Volume::loudest(),
                        params: vec![]
                    }
                ]
            );
        }
    }
}
