use std::collections::VecDeque;

use num_rational::Ratio;

use musik::{
    midi::{Instrument, PercussionSound},
    utils::CloneableIterator,
    AbsPitch, Dur, InstrumentName, Interval, Music,
};

#[derive(Clone)]
struct RoseTree<T> {
    root: T,
    children: Box<dyn CloneableIterator<Item = Self>>,
}

trait MutateFn<T>: Fn(&T, &T) -> T + Clone + 'static {}

impl<T, F> MutateFn<T> for F where F: Fn(&T, &T) -> T + Clone + 'static {}

impl<T> RoseTree<T>
where
    T: Clone + 'static,
{
    fn self_sim<F>(init: T, seed: Vec<T>, mutate: F) -> Self
    where
        F: MutateFn<T>,
    {
        Self {
            root: init,
            children: Box::new(
                seed.clone()
                    .into_iter()
                    .map(move |s| Self::maker_rec(s, seed.clone(), mutate.clone())),
            ),
        }
    }

    fn maker_rec<F>(init: T, seed: Vec<T>, mutate: F) -> Self
    where
        F: MutateFn<T>,
    {
        let root = init.clone();
        let children = seed
            .clone()
            .into_iter()
            .map(move |s| Self::maker_rec(mutate(&init, &s), seed.clone(), mutate.clone()));
        Self {
            root,
            children: Box::new(children),
        }
    }

    #[allow(dead_code)]
    fn fringe(self, level: usize) -> Box<dyn CloneableIterator<Item = T>> {
        let Self { root, children } = self;
        if level == 0 {
            return Box::new(std::iter::once(root));
        }

        Box::new(children.flat_map(move |c| c.fringe(level - 1)))
    }

    fn fringe_children(
        self,
        level: usize,
    ) -> PartialFringe<T, impl Iterator<Item = (Self, usize)>> {
        let Self { root, children } = self;
        if level == 0 {
            PartialFringe::Root(root)
        } else {
            PartialFringe::Children(children.map(move |child| (child, level - 1)))
        }
    }

    /// Exercise 9.2
    fn fringe_efficient(self, level: usize) -> impl Iterator<Item = T> + Clone + 'static {
        let mut deque = VecDeque::from([(self, level)]);
        std::iter::from_fn(move || loop {
            let (tree, lvl) = deque.pop_front()?;
            match tree.fringe_children(lvl) {
                PartialFringe::Root(item) => return Some(item),
                PartialFringe::Children(children) => {
                    deque.extend(children);
                }
            }
        })
    }

    fn fringe_vertical(self, level: usize) -> Box<dyn CloneableIterator<Item = Vec<T>>> {
        let Self { root, children } = self;
        if level == 0 {
            return Box::new(std::iter::once(vec![root]));
        }

        Box::new(children.map(move |c| c.fringe(level - 1).collect()))
    }
}

enum PartialFringe<R, C> {
    Root(R),
    Children(C),
}

type SNote = (Dur, AbsPitch);

type Cluster = RoseTree<SNote>;

fn mk_note((dur, ap): SNote) -> Music {
    Music::note(dur, ap.into())
}

fn sim_to_music(notes: impl Iterator<Item = SNote> + Clone + 'static) -> Music {
    Music::lazy_line(notes.map(mk_note))
}

fn mk_cluster(pat: Vec<SNote>) -> Cluster {
    let init = (Dur::ZERO, AbsPitch::from(ux2::u7::new(0)));
    let add_mult = |(d0, p0): &SNote, (d1, p1): &SNote| {
        let d = Dur::from((*d0).into_ratio() * (*d1).into_ratio());
        let p = p0.get_u8() + p1.get_u8();
        let p = p.min(127);
        let p = AbsPitch::from(ux2::u7::new(p));
        (d, p)
    };
    Cluster::self_sim(init, pat, add_mult)
}

fn self_similar(
    pat: Vec<SNote>,
    level: usize,
    trans_delta: Interval,
    tempo: impl Into<Ratio<u32>>,
) -> Music {
    let cls = mk_cluster(pat);
    let notes = cls.fringe_efficient(level);
    sim_to_music(notes).with_tempo(tempo).trans(trans_delta)
}

fn get_seed<T>(input: Vec<(T, u8)>) -> Vec<SNote>
where
    Dur: From<T>,
{
    input
        .into_iter()
        .map(|(d, p)| (Dur::from(d), AbsPitch::from(ux2::u7::new(p))))
        .collect()
}

pub fn tm0() -> Music {
    let m0 = get_seed(vec![
        (Ratio::from_integer(1), 2),
        ((1, 2).into(), 0),
        (1.into(), 5),
        ((3, 4).into(), 7),
    ]);
    self_similar(m0, 4, Interval::from(50), 5).with_instrument(Instrument::Vibraphone)
}

pub fn ttm0() -> Music {
    tm0().reverse().trans(Interval::octave()) | tm0()
}

pub fn tm1() -> Music {
    let m1 = get_seed(vec![
        (Ratio::from_integer(1), 0),
        ((1, 2).into(), 0),
        ((1, 2).into(), 0),
    ]);
    self_similar(m1, 4, PercussionSound::HighFloorTom.as_interval(), 2)
        .with_instrument(InstrumentName::Percussion)
}

pub fn tm2() -> Music {
    let m2 = get_seed(vec![(Dur::DOTTED_QUARTER, 0), (Dur::QUARTER, 4)]);
    self_similar(m2, 6, Interval::from(50), (1, 50))
}

pub fn tm3() -> Music {
    let m3 = get_seed(vec![
        (Dur::HALF, 3),
        (Dur::QUARTER, 4),
        (Dur::QUARTER, 0),
        (Dur::HALF, 6),
    ]);
    self_similar(m3, 4, Interval::from(50), (1, 4))
}

pub fn ttm3() -> Music {
    let l1 = tm3().with_instrument(Instrument::Flute);
    let l2 = tm3()
        .reverse()
        .trans(Interval::from(-9))
        .with_instrument(Instrument::AcousticBass);
    l1 | l2
}

pub fn tm4() -> Music {
    let m4 = get_seed(vec![
        (Dur::HALF, 3),
        (Dur::HALF, 8),
        (Dur::HALF, 22),
        (Dur::QUARTER, 4),
        (Dur::QUARTER, 7),
        (Dur::QUARTER, 21),
        (Dur::QUARTER, 0),
        (Dur::QUARTER, 5),
        (Dur::QUARTER, 15),
        (Dur::WHOLE, 6),
        (Dur::WHOLE, 9),
        (Dur::WHOLE, 19),
    ]);
    self_similar(m4, 3, Interval::from(50), 8)
}

/// Exercise 9.1
pub fn experimental() -> Music {
    let m2 = get_seed(vec![
        (Dur::HALF, 3),
        (Dur::QUARTER, 4),
        (Dur::DOTTED_QUARTER, 3),
        (Dur::QUARTER, 7),
    ]);
    let l1 = self_similar(m2, 4, Interval::from(50), (1, 20));
    let l2 = l1
        .clone()
        .invert()
        .trans(-Interval::octave())
        .with_instrument(InstrumentName::Percussion);
    let l3 = l1
        .clone()
        .reverse()
        .with_instrument(Instrument::ElectricGuitarMuted);

    l1 | l2 | l3
}

fn sim_to_chord_music(notes: impl Iterator<Item = Vec<SNote>> + Clone + 'static) -> Music {
    let melodies = notes
        .map(|line| Music::lazy_line(line.into_iter().map(mk_note)))
        .collect();
    Music::chord(melodies)
}

fn self_similar_parallel(
    pat: Vec<SNote>,
    level: usize,
    trans_delta: Interval,
    tempo: impl Into<Ratio<u32>>,
) -> Music {
    let cls = mk_cluster(pat);

    let notes = cls.fringe_vertical(level);
    sim_to_chord_music(notes)
        .with_tempo(tempo)
        .trans(trans_delta)
}

fn m5() -> Vec<SNote> {
    get_seed(vec![
        (Dur::EIGHTH, 4),
        (Dur::SIXTEENTH, 7),
        (Dur::EIGHTH, 0),
    ])
}

pub fn ss5() -> Music {
    self_similar(m5(), 4, Interval::from(45), (1, 500))
}

pub fn ss6() -> Music {
    self_similar_parallel(m5(), 4, Interval::from(45), (1, 1000))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fringe_simple() {
        let tree = RoseTree::self_sim(0, vec![1, 2], |x, y| x + y);
        let level_2: Vec<_> = tree.fringe(2).collect();
        assert_eq!(level_2, vec![2, 3, 3, 4]);
    }

    #[test]
    fn fringe_efficient() {
        let tree = RoseTree::self_sim(1, vec![1, 2], |x, y| x * y);
        let level_3: Vec<_> = tree.fringe_efficient(3).collect();
        assert_eq!(level_3, vec![1, 2, 2, 4, 2, 4, 4, 8]);
    }

    #[test]
    fn fringe_compare() {
        let tree = RoseTree::self_sim(1, vec![1, 2], |x, y| x * y);
        let tree_efficient = tree.clone();

        let t = std::time::Instant::now();
        let _levels: Vec<_> = tree_efficient.fringe_efficient(12).collect();
        println!("Efficient time: {:?}", t.elapsed());

        let t = std::time::Instant::now();
        let _levels: Vec<_> = tree.fringe(12).collect();
        println!("Simple fringe time: {:?}", t.elapsed());
    }
}
