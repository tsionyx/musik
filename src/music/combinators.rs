use super::{control::Control, Music, Primitive};

impl<P> Primitive<P> {
    pub fn map<U, F>(self, mut f: F) -> Primitive<U>
    where
        F: FnMut(P) -> U,
    {
        match self {
            Self::Note(d, p) => Primitive::Note(d, f(p)),
            Self::Rest(d) => Primitive::Rest(d),
        }
    }
}

impl<P> Music<P> {
    pub fn map<U, F>(self, f: F) -> Music<U>
    where
        F: FnMut(P) -> U + Clone,
    {
        match self {
            Self::Prim(p) => Music::Prim(p.map(f)),
            Self::Sequential(m1, m2) => m1.map(f.clone()) + m2.map(f),
            Self::Parallel(m1, m2) => m1.map(f.clone()) | m2.map(f),
            Self::Modify(c, m) => m.map(f).with(c),
        }
    }

    pub fn fold<U, Prim, Seq, Par, Mod>(
        self,
        mut prim: Prim,
        mut seq: Seq,
        mut par: Par,
        modify: Mod,
    ) -> U
    where
        Prim: FnMut(Primitive<P>) -> U + Clone,
        Seq: FnMut(U, U) -> U + Clone,
        Par: FnMut(U, U) -> U + Clone,
        Mod: FnMut(Control, U) -> U + Clone,
    {
        match self {
            Self::Prim(p) => prim(p),
            Self::Sequential(m1, m2) => {
                let u1 = m1.fold(prim.clone(), seq.clone(), par.clone(), modify.clone());
                let u2 = m2.fold(prim, seq.clone(), par, modify);
                seq(u1, u2)
            }
            Self::Parallel(m1, m2) => {
                let u1 = m1.fold(prim.clone(), seq.clone(), par.clone(), modify.clone());
                let u2 = m2.fold(prim, seq, par.clone(), modify);
                par(u1, u2)
            }
            Self::Modify(c, m) => modify.clone()(c, m.fold(prim, seq, par, modify)),
        }
    }
}
