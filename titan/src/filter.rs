use crate::storage::{Archetype, Storage};
use itertools::izip;
use std::{
    fmt::Debug,
    marker::PhantomData,
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

trait Query<'fetch> {
    type ResultType;
    fn query(storage: &'fetch Storage) -> Self::ResultType;
}

impl<'fetch, A, B> Query<'fetch> for (A, B)
where
    A: 'static + Debug + Parameter,
    B: 'static + Debug + Parameter,
{
    type ResultType = Result<'fetch, A, B>;
    fn query(storage: &'fetch Storage) -> Self::ResultType {
        let archetype = storage.archetype_by_bundle_kind.values().last().unwrap();
        let component_vec_lock_a = <A::ParameterFetch>::fetch(archetype);
        let component_vec_lock_b = <B::ParameterFetch>::fetch(archetype);
        Result {
            a: component_vec_lock_a,
            b: component_vec_lock_b,
        }
    }
}

struct ParameterFetchRead<T> {
    phantom: PhantomData<T>,
}

trait Parameter {
    type ParameterFetch: for<'borrow> ParameterFetch<'borrow>;
}
impl<T> Parameter for &T
where
    T: 'static,
{
    type ParameterFetch = ParameterFetchRead<T>;
}

trait ParameterFetch<'fetch> {
    type ResultType;
    fn fetch(archetype: &'fetch Archetype) -> Self::ResultType;
}

impl<'fetch, T> ParameterFetch<'fetch> for ParameterFetchRead<T>
where
    T: 'static,
{
    type ResultType = RwLockReadGuard<'fetch, Vec<T>>;
    fn fetch(archetype: &'fetch Archetype) -> Self::ResultType {
        archetype.get_component_vec_lock::<T>()
    }
}

struct Result<'fetch, A, B>
where
    A: Parameter,
    B: Parameter,
{
    a: <A::ParameterFetch as ParameterFetch<'fetch>>::ResultType,
    b: <B::ParameterFetch as ParameterFetch<'fetch>>::ResultType,
}

trait ResultIter<'borrow> {
    type IterType: Iterator;
    fn iter(&'borrow mut self) -> Self::IterType;
}
impl<'borrow, 'fetch, A, B> ResultIter<'borrow> for Result<'fetch, A, B>
where
    A: Parameter,
    B: Parameter,
    <A::ParameterFetch as ParameterFetch<'fetch>>::ResultType: ResultIter<'borrow>,
    <B::ParameterFetch as ParameterFetch<'fetch>>::ResultType: ResultIter<'borrow>,
{
    type IterType = std::iter::Zip<
        <<A::ParameterFetch as ParameterFetch<'fetch>>::ResultType as ResultIter<'borrow>>::IterType,
        <<B::ParameterFetch as ParameterFetch<'fetch>>::ResultType as ResultIter<'borrow>>::IterType,
    >;
    fn iter(&'borrow mut self) -> Self::IterType {
        izip!(self.a.iter(), self.b.iter())
    }
}
impl<'borrow, 'fetch, T: 'borrow> ResultIter<'borrow> for RwLockReadGuard<'fetch, Vec<T>> {
    type IterType = std::slice::Iter<'borrow, T>;
    fn iter(&'borrow mut self) -> Self::IterType {
        <[T]>::iter(self)
    }
}

#[test]
fn fetch_single() {
    use crate::registry::Registry;
    use serde::{Deserialize, Serialize};
    #[derive(Debug, Serialize, Deserialize)]
    struct Age(u8);
    #[derive(Debug, Serialize, Deserialize)]
    struct Name(String);
    #[derive(Debug, Serialize, Deserialize)]
    struct Person {
        height: u16,
        weight: u16,
    }

    let mut registry = Registry::new();
    registry.register_component::<Age>(&"Age");
    registry.register_component::<Name>(&"Name");
    registry.register_component::<Person>(&"Person");
    registry.register_archetype::<(Name, Age)>();

    let mut storage = Storage::new();
    storage.spawn(&registry, (Age(23), Name("Jeff".to_string())));
    storage.spawn(&registry, (Age(19), Name("Julia".to_string())));
    storage.spawn(&registry, (Name("Bob".to_string()), Age(29)));

    let mut query_result = <(&Age, &Name)>::query(&storage);
    for res in query_result.iter() {
        println!("{:?}", res);
    }
}
