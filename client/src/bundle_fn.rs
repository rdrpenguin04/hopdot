//! Code from https://github.com/DylanRJohnston/abiogenesis.
//! Used with permission (I asked for a license on the repo, so hopefully this won't be so handwavey in the future).

use bevy::prelude::*;

use bevy::ecs::bundle::{BundleEffect, DynamicBundle};
use bevy::ecs::component::{ComponentId, Components, ComponentsRegistrator, RequiredComponents, StorageType};

pub trait Thunk: FnOnce(&mut EntityWorldMut) + Send + Sync + 'static {}
impl<F: FnOnce(&mut EntityWorldMut) + Send + Sync + 'static> Thunk for F {}

pub struct BundleFn<F: Thunk>(pub F);

unsafe impl<F: Thunk> Bundle for BundleFn<F> {
    fn component_ids(_: &mut ComponentsRegistrator, _: &mut impl FnMut(ComponentId)) {}

    fn get_component_ids(_: &Components, _: &mut impl FnMut(Option<ComponentId>)) {}

    fn register_required_components(_: &mut ComponentsRegistrator, _: &mut RequiredComponents) {}
}

impl<F: Thunk> DynamicBundle for BundleFn<F> {
    type Effect = Self;

    fn get_components(self, _func: &mut impl FnMut(StorageType, bevy::ptr::OwningPtr<'_>)) -> Self {
        self
    }
}

impl<F: Thunk> BundleEffect for BundleFn<F> {
    fn apply(self, entity: &mut EntityWorldMut) {
        (self.0)(entity);
    }
}
