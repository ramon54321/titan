use crate::storage::{Archetype, Storage};
use itertools::izip;
use std::{
    fmt::Debug,
    marker::PhantomData,
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

///
/// Main Query entry point trait.
/// Query is implemented for many sizes of tuples containing generic `Parameter`s.
///
/// Query exposes the `query` method which is responsible for fetching each `Parameter`s
/// `ParameterFetch`s ResultType, which in turn implements `ResultIter`.
///
/// The `query` method returns a `Result{#}` struct containing the `ResultType` of each `Parameter`.
/// The `Result{#}` struct also implements `ResultIter`, exposing the `iter` method to the caller
/// of the `query` method.
///
pub trait Query<'fetch> {
    type ResultType;
    fn query(storage: &'fetch Storage) -> Self::ResultType;
}

///
/// Main `Parameter` trait, defining only the associated type `ParameterFetch` which contains some
/// struct implementing `ParameterFetch`.
///
pub trait Parameter {
    type ParameterFetch: for<'borrow> ParameterFetch<'borrow>;
}

///
/// Implementations for `Parameter` for Read.
///
impl<T> Parameter for &T
where
    T: 'static,
{
    type ParameterFetch = ParameterFetchRead<T>;
}
///
/// Implementations for `Parameter` for Write.
///
impl<T> Parameter for &mut T
where
    T: 'static,
{
    type ParameterFetch = ParameterFetchWrite<T>;
}

///
/// Defines the `fetch` method which is called for each `Parameter` from the main `query` method.
///
pub trait ParameterFetch<'fetch> {
    type ResultType;
    fn fetch(archetype: &'fetch Archetype) -> Self::ResultType;
}

///
/// ParameterFetch marker struct for Read.
///
pub struct ParameterFetchRead<T> {
    phantom: PhantomData<T>,
}
///
/// ParameterFetch marker struct for Write.
///
pub struct ParameterFetchWrite<T> {
    phantom: PhantomData<T>,
}

///
/// `ParameterFetch` implementation for Read.
///
impl<'fetch, T> ParameterFetch<'fetch> for ParameterFetchRead<T>
where
    T: 'static,
{
    type ResultType = RwLockReadGuard<'fetch, Vec<T>>;
    fn fetch(archetype: &'fetch Archetype) -> Self::ResultType {
        archetype.get_component_vec_lock::<T>()
    }
}
///
/// `ParameterFetch` implementation for Write.
///
impl<'fetch, T> ParameterFetch<'fetch> for ParameterFetchWrite<T>
where
    T: 'static,
{
    type ResultType = RwLockWriteGuard<'fetch, Vec<T>>;
    fn fetch(archetype: &'fetch Archetype) -> Self::ResultType {
        archetype.get_component_vec_lock_mut::<T>()
    }
}

pub trait QueryTuple<'fetch>: Query<'fetch> {}

impl<'fetch, A, B> QueryTuple<'fetch> for (A, B)
where
    A: 'static + Debug + Parameter,
    B: 'static + Debug + Parameter,
{
}

///
/// Implementations of `Query` for `Parameter` tuples.
///
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

///
/// Result structs for `Paremeter` tuples.
///
pub struct Result<'fetch, A, B>
where
    A: Parameter,
    B: Parameter,
{
    a: <A::ParameterFetch as ParameterFetch<'fetch>>::ResultType,
    b: <B::ParameterFetch as ParameterFetch<'fetch>>::ResultType,
}

///
/// Defines the `iter` method which the called of the main `query` method will call. This trait is
/// implemented on the main `Result{#}` struct itself and the Read and Write locks, which are the
/// possible `ResultType`s from the various `ParameterFetch` implementations, allowing the main
/// `Result{#}` implementation to zip the `ParameterFetch` results together.
///
pub trait ResultIter<'borrow> {
    type IterType: Iterator;
    fn iter(&'borrow mut self) -> Self::IterType;
}

///
/// ResultIter implementations for all `Result{#}` structs.
///
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

///
/// ResultIter implementation for Read
///
impl<'borrow, 'fetch, T: 'borrow> ResultIter<'borrow> for RwLockReadGuard<'fetch, Vec<T>> {
    type IterType = std::slice::Iter<'borrow, T>;
    fn iter(&'borrow mut self) -> Self::IterType {
        <[T]>::iter(self)
    }
}

///
/// ResultIter implementation for Write
///
impl<'borrow, 'fetch, T: 'borrow> ResultIter<'borrow> for RwLockWriteGuard<'fetch, Vec<T>> {
    type IterType = std::slice::IterMut<'borrow, T>;
    fn iter(&'borrow mut self) -> Self::IterType {
        <[T]>::iter_mut(self)
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

    for (age, name) in <(&mut Age, &Name)>::query(&storage).iter() {
        println!("{:?}", (age.0.clone(), name));
        age.0 = age.0 + 1;
    }
    for (age, name) in <(&mut Age, &Name)>::query(&storage).iter() {
        println!("{:?}", (age.0.clone(), name));
    }
    for (age, name) in storage.query::<(&mut Age, &Name)>().iter() {
        println!("{:?}", (age.0.clone(), name));
    }
}
