use std::{
    any::TypeId,
    marker::PhantomData,
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use crate::{
    registry::Registry,
    storage::{Archetype, Storage},
};

pub trait Fetch {
    type FetchItem<'a>;
    fn fetch<'a>(storage: &'a Storage) -> Vec<Self::FetchItem<'a>>;
}

impl<A: TypeFetch + 'static, B: TypeFetch + 'static> Fetch for (A, B) {
    type FetchItem<'a> = (A::FetchItem<'a>, B::FetchItem<'a>);
    fn fetch<'a>(storage: &'a Storage) -> Vec<Self::FetchItem<'a>> {
        // Get archetypes in some way
        //let component_type_id = TypeId::of::<T>();
        //let component_kind = registry.type_id_to_kind(component_type_id);

        // Mock get archetypes
        let archetypes = [storage.archetype_by_bundle_kind.values().last().unwrap()];

        let mut lock_bundles = Vec::new();
        for archetype in archetypes {
            let lock_bundle = (A::fetch(&archetype), B::fetch(&archetype));
            lock_bundles.push(lock_bundle);
        }
        //let lock_bundle = (A::fetch(&archetype), B::fetch(&archetype));
        //lock_bundles.push(lock_bundle);

        lock_bundles
    }
}

struct FetchIter<'a, T> {
    lock_bundle: T,
    archetype: &'a mut Archetype,
}
//impl FetchIter<'a, T> {
//fn new(archetype: &'a mut Archetype) -> Self {
//FetchIter {
//lock_bundle: (A::fetch(&archetype), B::fetch(&archetype)),
//archetype: archetype,
//};
//}
//}

pub trait TypeFetch {
    type FetchItem<'a>: SomeLock<'a>;
    fn fetch<'b>(archetype: &'b Archetype) -> Self::FetchItem<'b>;
}

impl<T: 'static> TypeFetch for &T {
    type FetchItem<'a> = RwLockReadGuard<'a, Vec<T>>;
    fn fetch<'b>(archetype: &'b Archetype) -> Self::FetchItem<'b> {
        archetype.get_component_vec_lock::<T>()
    }
}
impl<T: 'static> TypeFetch for &mut T {
    type FetchItem<'a> = RwLockWriteGuard<'a, Vec<T>>;
    fn fetch<'b>(archetype: &'b Archetype) -> Self::FetchItem<'b> {
        archetype.get_component_vec_lock_mut::<T>()
    }
}

pub trait SomeLock<'a> {
    type IterType;
    fn iter(&'a self) -> Self::IterType;
}

impl<'a, T> SomeLock<'a> for RwLockReadGuard<'a, Vec<T>> {
    type IterType = std::slice::Iter<'a, T>;
    fn iter(&'a self) -> Self::IterType {
        <[T]>::iter(self)
    }
}

impl<'a, T> SomeLock<'a> for RwLockWriteGuard<'a, Vec<T>> {
    type IterType = std::slice::Iter<'a, T>;
    fn iter(&'a self) -> Self::IterType {
        <[T]>::iter(self)
    }
}
