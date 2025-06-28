//! Code from <https://github.com/DylanRJohnston/abiogenesis>.
//! Used with permission (I asked for a license on the repo, so hopefully this won't be so handwavey in the future).

use bevy::prelude::*;

use bevy::ecs::system::IntoObserverSystem;

use crate::bundle_fn::BundleFn;

pub fn observe<E, B, M, O>(observer: O) -> BundleFn<impl FnOnce(&mut EntityWorldMut)>
where
    E: Event,
    B: Bundle,
    M: Send + Sync + 'static,
    O: IntoObserverSystem<E, B, M> + Send + Sync + 'static,
{
    BundleFn(move |entity| {
        entity.observe(observer);
    })
}
