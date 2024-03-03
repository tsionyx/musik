//! Grace, trill and roll here

use std::iter;

use num_rational::Ratio;
use num_traits::{CheckedSub as _, Zero as _};

use crate::prim::{duration::Dur, interval::Interval, pitch::AbsPitch};

use super::{control::Control, phrase::TrillOptions, Music, Primitive};

impl Music {
    pub fn grace_note(&self, offset: AbsPitch, grace_fraction: Ratio<u8>) -> Result<Self, String> {
        if let Self::Prim(Primitive::Note(d, p)) = self {
            Ok(Self::note(*d * grace_fraction, p.trans(offset.into()))
                + Self::note(*d * (Ratio::from_integer(1) - grace_fraction), *p))
        } else {
            Err("Can only add a grace note to a note".into())
        }
    }

    pub fn trill(
        &self,
        interval: Interval,
        opts: impl Into<TrillOptions<Dur>>,
    ) -> Result<Self, String> {
        match self {
            Self::Prim(Primitive::Note(d, p)) => {
                let dur_seq: Box<dyn Iterator<Item = Dur>> = match opts.into() {
                    TrillOptions::Duration(single) => {
                        let n: u8 = (d.into_ratio() / single.into_ratio()).to_integer();
                        let last_dur: Ratio<u8> = d
                            .into_ratio()
                            .checked_sub(&(Ratio::from(n) * single.into_ratio()))
                            .expect("Parts total duration should not be bigger than the whole");

                        Box::new(
                            iter::repeat(single)
                                .take(usize::from(n))
                                .chain((!last_dur.is_zero()).then_some(Dur::from(last_dur))),
                        )
                    }
                    TrillOptions::Count(n) => {
                        let single = *d / n;
                        Box::new(iter::repeat(single).take(usize::from(n)))
                    }
                };
                Ok(Self::line(
                    dur_seq
                        .enumerate()
                        .map(|(i, dur)| {
                            // odd are trills
                            let trill_pitch = i % 2 == 1;
                            let pitch = if trill_pitch { p.trans(interval) } else { *p };
                            Self::note(dur, pitch)
                        })
                        .collect(),
                ))
            }
            Self::Prim(Primitive::Rest(_)) => Err("Cannot construct trill from the Rest".into()),
            Self::Sequential(_, _) | Self::Parallel(_, _) => {
                Err("Cannot construct trill from the complex".into())
            }
            Self::Modify(Control::Tempo(r), m) => {
                let single = match opts.into() {
                    TrillOptions::Duration(single) => single,
                    TrillOptions::Count(n) => m.duration() / n,
                };
                m.trill(interval, single * *r).map(|m| m.with_tempo(*r))
            }
            Self::Modify(c, m) => m.trill(interval, opts).map(|m| m.with(c.clone())),
        }
    }

    pub fn roll(&self, opts: impl Into<TrillOptions<Dur>>) -> Result<Self, String> {
        self.trill(Interval::zero(), opts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prim::interval::Octave;
    use crate::Pitch;

    #[test]
    fn trill() {
        let oc4 = Octave::ONE_LINED;
        let m = Music::C(oc4, Dur::WHOLE);

        assert_eq!(
            m.trill(Interval::tone(), Dur::EIGHTH.dotted()).unwrap(),
            Music::with_dur(
                vec![
                    Pitch::C(oc4),
                    Pitch::D(oc4),
                    Pitch::C(oc4),
                    Pitch::D(oc4),
                    Pitch::C(oc4)
                ],
                Dur::DOTTED_EIGHTH
            ) + Music::D(oc4, Dur::SIXTEENTH)
        );
    }

    #[test]
    fn trill_count() {
        let oc4 = Octave::ONE_LINED;
        let m = Music::C(oc4, Dur::WHOLE);

        assert_eq!(
            m.trill(Interval::semi_tone(), TrillOptions::Count(4))
                .unwrap(),
            Music::line(vec![
                Music::C(oc4, Dur::QUARTER),
                Music::Cs(oc4, Dur::QUARTER),
                Music::C(oc4, Dur::QUARTER),
                Music::Cs(oc4, Dur::QUARTER),
            ])
        );
    }
}
