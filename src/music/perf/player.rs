//! Defines players structure responsible for interpretation details.
use std::{cmp::Ordering, fmt, ops::Deref};

use dyn_clone::{clone_trait_object, DynClone};
use intertrait::{cast::CastBox as _, CastFrom};
use log::{info, warn};

use crate::{
    music::{combinators::MapToOther, phrase::PhraseAttribute, Music},
    prim::duration::Dur,
    utils::Measure,
};

use super::{Context, Duration, Performance};

/// Defines the ways to interpret different parts of [`Music`]
/// to produce the [`Performance`].
///
/// # Warning
/// When implementing many versions of this trait
/// for a particular struct, do not forget to provide
/// ways to cast between various `dyn Player<T>` -> `dyn Player<U>`
/// using [`intertrait::cast_to`] or [`intertrait::castable_to`] for the target `impl Player<U>`
/// (see [interpretations](../interpretations.rs) for examples).
pub trait Player<P>: DynClone + CastFrom {
    /// Distinguish player struct from other implementations.
    fn name(&self) -> &'static str;

    /// Play individual notes.
    fn play_note(&self, note: (Dur, &P), ctx: Context<P>) -> Performance;

    /// Playing [`Music`] phrases
    /// taking into account [`PhraseAttribute`]-s and [`Context`].
    fn interpret_phrases(
        &self,
        music: &Music<P>,
        attrs: &[PhraseAttribute],
        ctx: Context<P>,
    ) -> (Performance, Measure<Duration>) {
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

impl<P: 'static> fmt::Debug for DynPlayer<P> {
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

impl<P: 'static> PartialEq for DynPlayer<P> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.name() == other.inner.name()
    }
}

impl<P: 'static> Eq for DynPlayer<P> {}

impl<P: 'static> PartialOrd for DynPlayer<P> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<P: 'static> Ord for DynPlayer<P> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.name().cmp(other.inner.name())
    }
}

impl<T: 'static, U: 'static> MapToOther<DynPlayer<U>> for DynPlayer<T> {
    fn into_other(self) -> Option<DynPlayer<U>> {
        self.inner
            .cast::<dyn Player<U>>()
            .map(|inner| {
                info!(
                    "Successfully casted `dyn Player<{}>` to `dyn Player<{}>`",
                    std::any::type_name::<T>(),
                    std::any::type_name::<U>(),
                );
                DynPlayer { inner }
            })
            .map_err(|_| {
                warn!(
                    "Cannot cast `dyn Player<{}>` to `dyn Player<{}>`",
                    std::any::type_name::<T>(),
                    std::any::type_name::<U>(),
                );
            })
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        music::{Control, MusicAttr},
        n, p,
        perf::DefaultPlayer,
        Octave, Pitch, PitchClass, Temporal as _, Volume,
    };

    #[test]
    fn convert_player_to_volume() {
        let tonic = Pitch::new(PitchClass::C, Octave::OneLined);
        let scale: Vec<_> = tonic.major_scale().collect();
        let m = Music::with_dur(scale, Dur::QUARTER).with_player(DefaultPlayer::default());
        assert!(
            matches!(m, Music::Modify(Control::Player(ref pl), _) if pl.name() == "Default (Pitch)")
        );

        let m: Music<(Pitch, Volume)> = m.into();
        assert!(
            matches!(m, Music::Modify(Control::Player(ref pl), _) if pl.name() == "Default (Pitch + Volume)")
        );
    }

    #[test]
    fn convert_player_to_volume_preserves_structure() {
        let tonic = p!(C 4);
        let scale: Vec<_> = tonic.major_scale().collect();
        let m = Music::with_dur(scale, n!(_ / 4)).with_default_player::<DefaultPlayer>();
        let m: Music<(Pitch, Volume)> = m.take(n!(_ / 2)).remove_zeros().into();

        assert_eq!(
            m,
            (Music::from(n!(C 4 / 4)) + n!(D 4 / 4).into())
                .with_default_player::<DefaultPlayer>()
                .into()
        );
        assert!(
            matches!(m, Music::Modify(Control::Player(ref pl), _) if pl.name() == "Default (Pitch + Volume)")
        );
    }

    #[test]
    fn convert_player_to_attributes() {
        let tonic = Pitch::new(PitchClass::C, Octave::OneLined);
        let scale: Vec<_> = tonic.major_scale().collect();
        let m = Music::with_dur(scale, Dur::QUARTER).with_player(DefaultPlayer::default());
        assert!(
            matches!(m, Music::Modify(Control::Player(ref pl), _) if pl.name() == "Default (Pitch)")
        );

        let m = MusicAttr::from(m);
        assert!(
            matches!(m, Music::Modify(Control::Player(ref pl), _) if pl.name() == "Default (Pitch with attributes)")
        );
    }

    #[test]
    fn convert_player_to_volume_then_to_attributes() {
        let tonic = Pitch::new(PitchClass::C, Octave::OneLined);
        let scale: Vec<_> = tonic.major_scale().collect();
        let m: Music<(Pitch, Volume)> = Music::with_dur(scale, Dur::QUARTER)
            .with_player(DefaultPlayer::default())
            .into();
        assert!(
            matches!(m, Music::Modify(Control::Player(ref pl), _) if pl.name() == "Default (Pitch + Volume)")
        );

        let m = MusicAttr::from(m);
        assert!(
            matches!(m, Music::Modify(Control::Player(ref pl), _) if pl.name() == "Default (Pitch with attributes)")
        );
    }
}
