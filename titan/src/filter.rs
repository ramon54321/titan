use itertools::izip;

use crate::{
    registry::Registry,
    storage::{Archetype, Storage},
};
use std::{
    any::TypeId,
    fmt::Debug,
    marker::PhantomData,
    slice::Iter,
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

#[test]
fn fetch_single() {
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
    //let mut query_result =
    //<&Age>::fetch(&storage.archetype_by_bundle_kind.values().last().unwrap());
    for res in query_result.iter() {
        println!("{:?}", res);
    }
    //println!("{:?}", query_result);
}

trait Query<'fetch> {
    type ResultType;
    fn query(storage: &Storage) -> Self::ResultType;
}

impl<'fetch, A, B> Query<'fetch> for (A, B)
where
    A: 'static + Debug + Parameter,
    B: 'static + Debug + Parameter,
{
    type ResultType = Result<'fetch, A, B>;
    fn query(storage: &Storage) -> Self::ResultType {
        let archetype = storage.archetype_by_bundle_kind.values().last().unwrap();
        let component_vec_lock_a = <A::ParameterFetch>::fetch(archetype);
        let component_vec_lock_b = <B::ParameterFetch>::fetch(archetype);
        //todo!()
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
    fn fetch(archetype: &Archetype) -> Self::ResultType;
}

impl<'fetch, T: 'fetch> ParameterFetch<'fetch> for ParameterFetchRead<T> {
    type ResultType = RwLockReadGuard<'fetch, Vec<T>>;
    fn fetch(archetype: &Archetype) -> Self::ResultType {
        todo!()
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
    type IterType;
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
        todo!()
    }
}
impl<'borrow, 'fetch, T: 'borrow> ResultIter<'borrow> for RwLockReadGuard<'fetch, Vec<T>> {
    type IterType = std::slice::Iter<'borrow, T>;
    fn iter(&'borrow mut self) -> Self::IterType {
        todo!()
    }
}

//#[derive(Debug)]
//struct Result<'fetch, A, B>
//where
//A: Fetch<'fetch, A>,
//B: Fetch<'fetch, B>,
//{
//a: RwLockReadGuard<'storage, Vec<A>>,
//b: RwLockReadGuard<'storage, Vec<B>>,
//a: A::FetchResult,
//b: B::FetchResult,
//}
//impl<'fetch, A, B> Result<'fetch, A, B>
//where
//A: Fetch<'fetch, A>,
//B: Fetch<'fetch, B>,
//{
//fn iter<'a>(
//&'a mut self,
//) -> std::iter::Zip<
//std::slice::Iter<'a, <A::FetchResult as FetchResultRead>::IterItem>,
//std::slice::Iter<'a, <B::FetchResult as FetchResultRead>::IterItem>,
//> {
//izip!(self.a.iter(), self.b.iter())
//}
//}

//trait Fetch<'fetch, T> {
//type FetchResult: FetchResult<'fetch, T>;
//fn fetch(archetype: &'fetch Archetype) -> Self::FetchResult;
//}
//impl<'fetch, T> Fetch<'fetch, T> for &'fetch T {
//type FetchResult = FetchResultRead<'fetch, T>;
//fn fetch(archetype: &'fetch Archetype) -> Self::FetchResult {
//let vec_lock = archetype.get_component_vec_lock::<T>();
//FetchResultRead { inner: vec_lock }
//}
//}
//trait FetchResult<'fetch, T> {}

//struct FetchResultRead<'fetch, T> {
//inner: RwLockReadGuard<'fetch, Vec<T>>,
//}
//impl<'fetch, T> FetchResult<'fetch, T> for FetchResultRead<'fetch, T> {}
//impl<'borrow, 'fetch, T> Borrow<'borrow> for FetchResultRead<'fetch, T>
//where
//T: 'borrow,
//'fetch: 'borrow,
//{
//type InnerType = &'borrow RwLockReadGuard<'fetch, Vec<T>>;
//fn my_borrow(&'borrow mut self) -> Self::InnerType {
//&self.inner
//}
//}
//trait Borrow<'borrow> {
//type InnerType;
//fn my_borrow(&'borrow mut self) -> Self::InnerType;
//}

//impl<'a, 'fetch, T> Fetch<'fetch> for &'fetch T
//where
//T: 'static + Debug,
//{
//type FetchResult = RwLockReadGuard<'a, Vec<T>>;
//fn fetch(archetype: &'fetch Archetype) -> Self::FetchResult {
//&*archetype.get_component_vec_lock::<T>()
//}
//}

////struct FetchResult<'fetch, T> {
////data: RwLockReadGuard<>
////}

//trait FetchResult {
//type IterItem;
//fn iter(&self) -> std::slice::Iter<'_, Self::IterItem>;
//}
//impl<'a, T> FetchResult for RwLockReadGuard<'a, Vec<T>> {
//type IterItem = T;
//fn iter(&self) -> std::slice::Iter<'_, Self::IterItem> {
//todo!()
//}
//}

/////
///// Main query trait defining `query` method which can be called on any tuple of `QueryParameter`s.
/////
//pub trait Query<'a> {
//type QueryResult: QueryResult;
//fn query(storage: &'a Storage) -> Self::QueryResult;
//}

//impl<'a, A, B> Query<'a> for (A, B)
//where
//A: QueryParameter,
//B: QueryParameter,
//{
////type QueryResult = QueryTupleResult<
////<<A as QueryParameter>::QueryParameterFetch as QueryParameterFetch<'b>>::QueryParameterFetchResult,
////<<B as QueryParameter>::QueryParameterFetch as QueryParameterFetch<'b>>::QueryParameterFetchResult,
////>;
//type QueryResult = QueryTupleResult;
//fn query(storage: &'a Storage) -> Self::QueryResult {
//let fetch_result = <(A, B)>::fetch(storage);
//let a = fetch_result.0._iter();
//todo!()
//}
//}

//pub trait Fetch<'a> {
//type FetchType;
//fn fetch(storage: &'a Storage) -> Self::FetchType;
//}
//impl<'a, A, B> Fetch<'a> for (A, B)
//where
//A: QueryParameter,
//B: QueryParameter,
//{
//type FetchType = (
//<<A as QueryParameter>::QueryParameterFetch as QueryParameterFetch<'a>>::QueryParameterFetchResult,
//<<B as QueryParameter>::QueryParameterFetch as QueryParameterFetch<'a>>::QueryParameterFetchResult,
//);
//fn fetch(storage: &'a Storage) -> Self::FetchType {
//let archetype = storage.archetype_by_bundle_kind.values().last().unwrap();
//let a = <A::QueryParameterFetch as QueryParameterFetch<'a>>::fetch(archetype);
//let b = <B::QueryParameterFetch as QueryParameterFetch<'a>>::fetch(archetype);
//(a, b)
//}
//}

/////
///// QueryResult is the trait which is implemented on the return of the `query` function.
///// This trait indicates functionality which is shared by all types returned by the `query`
///// function.
/////
//pub trait QueryResult {
////type IterItem;
////fn iter(&self) -> QueryResultIter<Self::IterItem>;
//}
//pub struct QueryTupleResult {}
//impl QueryResult for QueryTupleResult {}

////#[derive(Debug)]
////pub struct QueryTupleResult<'a, A, B>
////where
////A: QueryParameterFetchResult<'a>,
////B: QueryParameterFetchResult<'a>,
////{
////a: A,
////b: B,
////phantom: PhantomData<&'a A>,
////}
////impl<'a, A, B> QueryResult<'a> for QueryTupleResult<'a, A, B>
////where
////A: QueryParameterFetchResult<'a>,
////B: QueryParameterFetchResult<'a>,
////{
////}

/////
///// Individual parameter in query tuple. Eg. the `B` in (A, B, C).
///// The individual query parameter is responsible for fetching its respective lock from storage.
/////
//pub trait QueryParameter {
//type QueryParameterFetch: for<'a> QueryParameterFetch<'a>;
//}
//impl<T> QueryParameter for &T
//where
//T: 'static + Debug,
//{
//type QueryParameterFetch = QueryParameterFetchRead<T>;
//}
//impl<T> QueryParameter for &mut T
//where
//T: 'static + Debug,
//{
//type QueryParameterFetch = QueryParameterFetchWrite<T>;
//}

//pub struct QueryParameterFetchRead<T> {
//phantom: PhantomData<T>,
//}
//pub struct QueryParameterFetchWrite<T> {
//phantom: PhantomData<T>,
//}

/////
///// In order to fetch for any lifetime which is not coupled to that of the storage, an inner struct
///// is used.
/////
//pub trait QueryParameterFetch<'a> {
//type QueryParameterFetchResult: QueryParameterFetchResult<'a>;
//fn fetch(archetype: &'a Archetype) -> Self::QueryParameterFetchResult;
//}
//impl<'a, T> QueryParameterFetch<'a> for QueryParameterFetchRead<T>
//where
//T: 'static + Debug,
//{
//type QueryParameterFetchResult = RwLockReadGuard<'a, Vec<T>>;
//fn fetch(archetype: &'a Archetype) -> Self::QueryParameterFetchResult {
//archetype.get_component_vec_lock::<T>()
//}
//}
//impl<'a, T> QueryParameterFetch<'a> for QueryParameterFetchWrite<T>
//where
//T: 'static + Debug,
//{
//type QueryParameterFetchResult = RwLockWriteGuard<'a, Vec<T>>;
//fn fetch(archetype: &'a Archetype) -> Self::QueryParameterFetchResult {
//archetype.get_component_vec_lock_mut::<T>()
//}
//}

/////
///// Behaviour applicable to all results from a parameter fetch.
/////
//pub trait QueryParameterFetchResult<'a> {
//type Iterator: Debug;
//fn _iter(&'a self) -> Self::Iterator;
//}
//impl<'a, 'b, T: Debug> QueryParameterFetchResult<'a> for RwLockReadGuard<'b, Vec<T>>
//where
//'a: 'b,
//{
//type Iterator = std::slice::Iter<'b, T>;
//fn _iter(&'a self) -> Self::Iterator {
//self.iter()
//}
//}
//impl<'a, 'b, T: Debug> QueryParameterFetchResult<'a> for RwLockWriteGuard<'b, Vec<T>>
//where
//'a: 'b,
//{
//type Iterator = std::slice::Iter<'b, T>;
//fn _iter(&'a self) -> Self::Iterator {
//self.iter()
//}
//}

//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//

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

//#[derive(Debug)]
//pub struct TupleFetchResult<T: LockTuple> {
//current_bundle: usize,
//current_index: usize,
//lock_bundles: Vec<T>,
//}
//impl<T: LockTuple> TupleFetchResult<T> {
//fn new(lock_bundles: Vec<T>) -> Self {
//Self {
//current_bundle: 0,
//current_index: 0,
//lock_bundles,
//}
//}
////pub fn next(&mut self) -> Option<&T> {
////let bundle = self.lock_bundles.get(self.current_bundle).or_else(|| {
////self.current_bundle = self.current_bundle + 1;
////self.current_index = 0;
////self.lock_bundles.get(self.current_bundle)
////});
////if bundle.is_none() {
////return None;
////}
////let bundle = bundle.unwrap();
////self.current_index = self.current_index + 1;
////let iter = bundle.test();
//////for i in iter {
//////println!("{:?}", i);
//////}
////Some(bundle)
////}
//}

//pub trait LockTuple {
//fn test(&self);
//}
//impl<'a, A, B> LockTuple for (A, B)
//where
//A: ComponentLock<'a> + Debug,
//B: ComponentLock<'a> + Debug,
//{
//fn test(&self) {
//println!("Helllllllllooooooooo");
//let itera = self.0.iter();
//for item in itera {
//println!("{:?}", item);
//}
//}
//}

//pub trait QueryTuple<'a> {
//type QueryTupleElement;
//type QueryTupleLocks: LockTuple;
//fn fetch(storage: &'a Storage) -> TupleFetchResult<Self::QueryTupleLocks>;
//}
//impl<'a, A: ComponentLockFetch + 'a + Debug, B: ComponentLockFetch + 'a + Debug> QueryTuple<'a>
//for (A, B)
//{
//type QueryTupleElement = (A, B);
//type QueryTupleLocks = (
//<A as ComponentLockFetch>::ComponentLock<'a>,
//<B as ComponentLockFetch>::ComponentLock<'a>,
//);
//fn fetch(storage: &'a Storage) -> TupleFetchResult<Self::QueryTupleLocks> {
//let archetypes = [storage.archetype_by_bundle_kind.values().last().unwrap()].to_vec();
//let mut result = Vec::new();
//for archetype in archetypes {
//let lock_bundle = (A::fetch(&archetype), B::fetch(&archetype));
//result.push(lock_bundle);
//}
//TupleFetchResult::new(result)
//}
//}

//pub trait ComponentLockFetch {
//type ComponentLock<'a>: ComponentLock<'a> + Debug;
//fn fetch<'b>(archetype: &'b Archetype) -> Self::ComponentLock<'b>;
//}

//impl<T: 'static + Debug> ComponentLockFetch for &T {
//type ComponentLock<'a> = RwLockReadGuard<'a, Vec<T>>;
//fn fetch<'b>(archetype: &'b Archetype) -> Self::ComponentLock<'b> {
//archetype.get_component_vec_lock::<T>()
//}
//}
//impl<T: 'static + Debug> ComponentLockFetch for &mut T {
//type ComponentLock<'a> = RwLockWriteGuard<'a, Vec<T>>;
//fn fetch<'b>(archetype: &'b Archetype) -> Self::ComponentLock<'b> {
//archetype.get_component_vec_lock_mut::<T>()
//}
//}

//pub trait ComponentLock<'a> {
//type IterItem: Debug;
//type IterType: Iterator<Item = Self::IterItem>;
//fn iter(&'a self) -> Self::IterType;
//}

//impl<'a, T: Debug> ComponentLock<'a> for RwLockReadGuard<'a, Vec<T>> {
//type IterItem = &'a T;
//type IterType = std::slice::Iter<'a, T>;
//fn iter(&'a self) -> Self::IterType {
//<[T]>::iter(self)
//}
//}
//impl<'a, T: Debug> ComponentLock<'a> for RwLockWriteGuard<'a, Vec<T>> {
//type IterItem = &'a T;
//type IterType = std::slice::Iter<'a, T>;
//fn iter(&'a self) -> Self::IterType {
//<[T]>::iter(self)
//}
//}
