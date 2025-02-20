use num_rational::Ratio;

use musik::{midi::Instrument, AbsPitch, Dur, Interval, Music};

#[derive(Debug, Clone)]
struct RoseTree<T> {
    root: T,
    children: Vec<Self>,
}

impl<T> RoseTree<T>
where
    T: Clone,
{
    fn self_sim<F>(init: T, seed: &[T], mutate: F) -> Self
    where
        F: Fn(&T, &T) -> T,
    {
        // workaround to emulate recursive closures
        // https://stackoverflow.com/a/16953239/10491406
        struct MkCluster<'s, T> {
            f: &'s dyn Fn(&Self, &T) -> RoseTree<T>,
        }
        let maker = MkCluster {
            f: &|maker, item: &T| {
                let children = seed
                    .iter()
                    .map(|i| (maker.f)(maker, &mutate(item, i)))
                    .collect();
                Self {
                    root: item.clone(),
                    children,
                }
            },
        };

        Self {
            root: init,
            children: seed.iter().map(|i| (maker.f)(&maker, i)).collect(),
        }
    }

    fn fringe(&self, level: usize) -> Vec<T> {
        if level == 0 {
            return vec![self.root.clone()];
        }

        self.children
            .iter()
            .flat_map(|c| c.fringe(level - 1))
            .collect()
    }
}

type SNote = (Dur, AbsPitch);

type Cluster = RoseTree<SNote>;

fn make_note((d, ap): SNote) -> Music {
    Music::note(d, ap.into())
}

fn sim_to_music(notes: Vec<SNote>) -> Music {
    Music::lazy_line(notes.into_iter().map(make_note))
}

fn ss(pat: &[SNote], level: usize, trans_delta: Interval, tempo: Ratio<u32>) -> Music {
    let init = (Dur::ZERO, AbsPitch::from(ux2::u7::new(0)));
    let add_mult = |(d0, p0): &SNote, (d1, p1): &SNote| {
        let d = Dur::from(d0.into_ratio() * d1.into_ratio());
        let p = p0.get_u8() + p1.get_u8();
        let p = p.min(127);
        let p = AbsPitch::from(ux2::u7::new(p));
        (d, p)
    };
    let cls = Cluster::self_sim(init, pat, add_mult);

    sim_to_music(cls.fringe(level))
        .with_tempo(tempo)
        .trans(trans_delta)
}

pub fn tm0() -> Music {
    let m0 = [(1, 2), (1, 0), (1, 5), (1, 7)]
        .map(|(d, p)| (Dur::from(d), AbsPitch::from(ux2::u7::new(p))));
    ss(&m0, 4, Interval::from(50), Ratio::from(20)).with_instrument(Instrument::Vibraphone)
}
