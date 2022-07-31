use std::{
    any::TypeId,
    fmt::Debug,
    marker::PhantomData,
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use crate::{
    registry::Registry,
    storage::{Archetype, Storage},
};

//pub trait Fetch {
//type FetchItem<'a>;
//fn fetch<'a>(storage: &'a Storage) -> FetchResult<Self::FetchItem<'a>>;
//}

//impl<A: ComponentLockFetch + 'static, B: ComponentLockFetch + 'static> Fetch for (A, B) {
//type FetchItem<'a> = (A::FetchItem<'a>, B::FetchItem<'a>);
//fn fetch<'a>(storage: &'a Storage) -> FetchResult<Self::FetchItem<'a>> {
//// Get archetypes in some way
////let component_type_id = TypeId::of::<T>();
////let component_kind = registry.type_id_to_kind(component_type_id);

//// Mock get archetypes
//let archetypes = [storage.archetype_by_bundle_kind.values().last().unwrap()];

//let mut lock_bundles = Vec::new();
//for archetype in archetypes {
//let lock_bundle = (A::fetch(&archetype), B::fetch(&archetype));
//lock_bundles.push(lock_bundle);
//}
//FetchResult::new(lock_bundles)
//}
//}

//pub struct FetchResult<'a, T: Fetch> {
//current_bundle: usize,
//current_index: usize,
//lock_bundles: Vec<<T as Fetch>::FetchItem<'a>>,
//storage: &'a Storage,
//}
//impl<'a, T> FetchResult<'a, T> {
//fn new(lock_bundles: Vec<T>) -> Self {
//Self {
//current_bundle: 0,
//current_index: 0,
//lock_bundles,
//}
//}
//}

//impl<'a, A, B> FetchResult<'a, (A, B)>
//where
//A: ComponentLock<'a>,
//B: ComponentLock<'a>,
//{
//fn next(&'a mut self) -> Option<&'a (A, B)> {
//let bundle = self.lock_bundles.get(self.current_bundle).or_else(|| {
//self.current_bundle = self.current_bundle + 1;
//self.current_index = 0;
//self.lock_bundles.get(self.current_bundle)
//});
//if bundle.is_none() {
//return None;
//}
//let bundle = bundle.unwrap();
//self.current_index = self.current_index + 1;
//Some(bundle)
//}
//}

//impl<'a, T: QueryTupleRef<'a>> Iterator for TupleFetchResult<'a, T> {
//type Item = <T as QueryTupleRef<'a>>::TupleFetchItemRef;
//fn next(&mut self) -> Option<Self::Item> {
//let bundle = self.lock_bundles.get(self.current_bundle).or_else(|| {
//self.current_bundle = self.current_bundle + 1;
//self.current_index = 0;
//self.lock_bundles.get(self.current_bundle)
//});
//if bundle.is_none() {
//return None;
//}
//let bundle = bundle.unwrap();
//self.current_index = self.current_index + 1;
//Some(bundle)
//}
//}

#[derive(Debug)]
pub struct TupleFetchResult<'a, T: LockTuple<'a>> {
    current_bundle: usize,
    current_index: usize,
    lock_bundles: Vec<T>,
    _phantom_data: PhantomData<&'a T>,
}
impl<'a, T: LockTuple<'a>> TupleFetchResult<'a, T> {
    fn new(lock_bundles: Vec<T>) -> Self {
        Self {
            current_bundle: 0,
            current_index: 0,
            lock_bundles,
            _phantom_data: PhantomData::default(),
        }
    }
    pub fn next(&'a mut self) -> Option<&T> {
        let bundle = self.lock_bundles.get(self.current_bundle).or_else(|| {
            self.current_bundle = self.current_bundle + 1;
            self.current_index = 0;
            self.lock_bundles.get(self.current_bundle)
        });
        if bundle.is_none() {
            return None;
        }
        let bundle = bundle.unwrap();
        self.current_index = self.current_index + 1;
        let iter = bundle.into_iter();
        //for i in iter {
        //println!("{:?}", i);
        //}
        Some(bundle)
    }
}

//struct LockTupleIterator<T> {
//lock_tuple: T,
//current_index: usize,
//}
//impl<T> Iterator for LockTupleIterator<T> {
//type Item = T;
//fn next(&mut self) -> Option<Self::Item> {
//let a = self.lock_tuple.0
//}
//}

pub trait LockTuple<'a> {
    //type IteratorType;
    fn into_iter(&'a self);
    //type Iter: Iterator;
    //fn iter(&'a mut self) -> Self::Iter;
}
impl<'a, A: ComponentLock<'a> + 'a + Debug, B: ComponentLock<'a> + 'a + Debug> LockTuple<'a>
    for (A, B)
{
    //type IteratorType = LockTupleIterator<(A, B)>;
    fn into_iter(&'a self) {
        //LockTupleIterator {
        //lock_tuple: self,
        //current_index: 0,
        //}
        println!("Helllllllllooooooooo");

        let itera = self.0.iter();
        for item in itera {
            println!("{:?}", item);
        }
    }
    //type Iter = std::slice::Iter<'a, (A, B)>;
    //fn iter(&'a mut self) -> Self::Iter {
    //let a = self.0.iter();
    //let b = self.1.iter();

    //let zipped_iter = Iterator::zip(a, b);

    //zipped_iter
    //}
}

pub trait QueryTuple<'a> {
    type QueryTupleElement;
    type QueryTupleLocks: LockTuple<'a>;
    fn fetch(storage: &'a Storage) -> TupleFetchResult<Self::QueryTupleLocks>;
}
impl<'a, A: ComponentLockFetch + 'a + Debug, B: ComponentLockFetch + 'a + Debug> QueryTuple<'a>
    for (A, B)
{
    type QueryTupleElement = (A, B);
    type QueryTupleLocks = (
        <A as ComponentLockFetch>::ComponentLock<'a>,
        <B as ComponentLockFetch>::ComponentLock<'a>,
    );
    fn fetch(storage: &'a Storage) -> TupleFetchResult<Self::QueryTupleLocks> {
        let archetypes = [storage.archetype_by_bundle_kind.values().last().unwrap()].to_vec();
        let mut result = Vec::new();
        for archetype in archetypes {
            let lock_bundle = (A::fetch(&archetype), B::fetch(&archetype));
            result.push(lock_bundle);
        }
        TupleFetchResult::new(result)
    }
}

pub trait ComponentLockFetch {
    type ComponentLock<'a>: ComponentLock<'a> + Debug;
    fn fetch<'b>(archetype: &'b Archetype) -> Self::ComponentLock<'b>;
}

impl<T: 'static + Debug> ComponentLockFetch for &T {
    type ComponentLock<'a> = RwLockReadGuard<'a, Vec<T>>;
    fn fetch<'b>(archetype: &'b Archetype) -> Self::ComponentLock<'b> {
        archetype.get_component_vec_lock::<T>()
    }
}
impl<T: 'static + Debug> ComponentLockFetch for &mut T {
    type ComponentLock<'a> = RwLockWriteGuard<'a, Vec<T>>;
    fn fetch<'b>(archetype: &'b Archetype) -> Self::ComponentLock<'b> {
        archetype.get_component_vec_lock_mut::<T>()
    }
}

pub trait ComponentLock<'a> {
    type IterItem: Debug;
    type IterType: Iterator<Item = Self::IterItem>;
    fn iter(&'a self) -> Self::IterType;
}

impl<'a, T: Debug> ComponentLock<'a> for RwLockReadGuard<'a, Vec<T>> {
    type IterItem = &'a T;
    type IterType = std::slice::Iter<'a, T>;
    fn iter(&'a self) -> Self::IterType {
        <[T]>::iter(self)
    }
}
impl<'a, T: Debug> ComponentLock<'a> for RwLockWriteGuard<'a, Vec<T>> {
    type IterItem = &'a T;
    type IterType = std::slice::Iter<'a, T>;
    fn iter(&'a self) -> Self::IterType {
        <[T]>::iter(self)
    }
}

//impl<'a, T: Debug> ComponentLock<'a> for RwLockWriteGuard<'a, Vec<T>> {
//type IterItem = &'a mut T;
//type IterType = std::slice::IterMut<'a, T>;
//fn iter(&'a mut self) -> Self::IterType {
//<[T]>::iter_mut(self)
//}
//}
