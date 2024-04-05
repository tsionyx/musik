//! Defines players structure responsible for interpretation details.
use std::{cmp::Ordering, fmt, ops::Deref};

use dyn_clone::{clone_trait_object, DynClone};

use crate::{
    music::{combinators::MapToOther, phrase::PhraseAttribute, Music},
    prim::duration::Dur,
};

use super::{Context, Duration, Performance};

/// Defines the ways to interpret different parts of [`Music`]
/// to produce the [`Performance`].
pub trait Player<P>: DynClone {
    /// Distinguish player struct from other implementations.
    fn name(&self) -> &'static str;

    /// Play individual notes.
    fn play_note(&self, note: (Dur, &P), ctx: Context<'_, P>) -> Performance;

    /// Playing [`Music`] phrases
    /// taking into account [`PhraseAttribute`]-s and [`Context`].
    fn interpret_phrases(
        &self,
        music: &Music<P>,
        attrs: &[PhraseAttribute],
        ctx: Context<'_, P>,
    ) -> (Performance, Duration) {
        let (perf, dur) = music.perf(ctx);
        let perf = attrs
            .iter()
            .fold(perf, |perf, attr| self.interpret_phrase(perf, attr));
        (perf, dur)
    }

    /// Transform the performance according to [`PhraseAttribute`].
    fn interpret_phrase(&self, perf: Performance, attr: &PhraseAttribute) -> Performance;

    /// Notate a musical score in its own unique way (**not implemented**).
    fn notate_player(&self) {
        todo!("producing a properly notated score is not defined yet")
    }
}

clone_trait_object!(<P> Player<P>);

/// Wrapper for a `dyn Player`.
pub struct DynPlayer<P> {
    inner: Box<dyn Player<P>>,
}

impl<P> DynPlayer<P> {
    /// Create a dynamic [`Player`] but erasing the concrete type.
    pub fn from_player<Pl>(player: Pl) -> Self
    where
        Pl: Player<P> + 'static,
    {
        let inner = Box::new(player);
        Self { inner }
    }
}

impl<P> Deref for DynPlayer<P> {
    type Target = Box<dyn Player<P>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<P> fmt::Debug for DynPlayer<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DynPlayer")
            .field("name", &self.inner.name())
            .field("note", &std::any::type_name::<P>())
            .finish_non_exhaustive()
    }
}

impl<P> Clone for DynPlayer<P> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<P> PartialEq for DynPlayer<P> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.name() == other.inner.name()
    }
}

impl<P> Eq for DynPlayer<P> {}

impl<P> PartialOrd for DynPlayer<P> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<P> Ord for DynPlayer<P> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.name().cmp(other.inner.name())
    }
}

impl<T, U> MapToOther<DynPlayer<U>> for DynPlayer<T> {
    /// TODO: implement some logic here,
    ///  otherwise the annotation [`Control::Player`][super::Control::Player]
    ///  does nothing when converted (e.g. during [`Music::map`]).
    fn into_other(self) -> Option<DynPlayer<U>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{perf::DefaultPlayer, Octave, Performable as _, Pitch, PitchClass};

    #[test]
    fn convert_player() {
        let tonic = Pitch::new(PitchClass::C, Octave::OneLined);
        let scale: Vec<_> = tonic.major_scale().collect();
        let perf = Music::with_dur(scale, Dur::QUARTER)
            .with_player(DefaultPlayer::default())
            .perform();
        assert!(!perf.into_events().is_empty());
    }
}
