use crate::storage::{Archetype, Storage};
use crate::ComponentMeta;
use itertools::izip;
use paste::paste;
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
    T: 'static + ComponentMeta,
{
    type ParameterFetch = ParameterFetchRead<T>;
}
///
/// Implementations for `Parameter` for Write.
///
impl<T> Parameter for &mut T
where
    T: 'static + ComponentMeta,
{
    type ParameterFetch = ParameterFetchWrite<T>;
}

///
/// Defines the `fetch` method which is called for each `Parameter` from the main `query` method.
///
pub trait ParameterFetch<'fetch> {
    type ResultType;
    fn fetch<'a>(archetypes: &'a [&'fetch Archetype]) -> Self::ResultType;
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
    T: 'static + ComponentMeta,
{
    type ResultType = Vec<RwLockReadGuard<'fetch, Vec<T>>>;
    fn fetch<'a>(archetypes: &'a [&'fetch Archetype]) -> Self::ResultType {
        let mut locks = Vec::new();
        for archetype in archetypes {
            locks.push(archetype.get_component_vec_lock::<T>());
        }
        locks
    }
}
///
/// `ParameterFetch` implementation for Write.
///
impl<'fetch, T> ParameterFetch<'fetch> for ParameterFetchWrite<T>
where
    T: 'static + ComponentMeta,
{
    type ResultType = Vec<RwLockWriteGuard<'fetch, Vec<T>>>;
    fn fetch<'a>(archetypes: &'a [&'fetch Archetype]) -> Self::ResultType {
        let mut locks = Vec::new();
        for archetype in archetypes {
            locks.push(archetype.get_component_vec_lock_mut::<T>());
        }
        locks
    }
}

///
/// Defines the `iter` method which the called of the main `query` method will call. This trait is
/// implemented on the main `Result{#}` struct itself and the Read and Write locks, which are the
/// possible `ResultType`s from the various `ParameterFetch` implementations, allowing the main
/// `Result{#}` implementation to zip the `ParameterFetch` results together.
///
pub trait ResultIter<'borrow> {
    type IterType: Iterator;
    fn result_iter(&'borrow mut self) -> Self::IterType;
}

///
/// ResultIter implementation for Read
///
impl<'borrow, 'fetch: 'borrow, T: 'fetch> ResultIter<'borrow>
    for Vec<RwLockReadGuard<'fetch, Vec<T>>>
{
    type IterType = impl Iterator<Item = &'borrow T>;
    fn result_iter(&'borrow mut self) -> Self::IterType {
        <[_]>::iter(self).map(|guard| guard.iter()).flatten()
    }
}

///
/// ResultIter implementation for Write
///
impl<'borrow, 'fetch: 'borrow, T: 'fetch> ResultIter<'borrow>
    for Vec<RwLockWriteGuard<'fetch, Vec<T>>>
{
    type IterType = impl Iterator<Item = &'borrow mut T>;
    fn result_iter(&'borrow mut self) -> Self::IterType {
        <[_]>::iter_mut(self)
            .map(|guard| guard.iter_mut())
            .flatten()
    }
}

///
///
/// Macros for generating tuple size specific structs and implementations.
///
///

///
/// Implementations of `Query` for `Parameter` tuples.
///
macro_rules! query_impl {
    ($count:tt, $($name:ident),*) => {
        paste!{
            impl<'fetch, $($name),*> Query<'fetch> for ($($name),*,)
            where
                $($name: 'static + Debug + Parameter + ComponentMeta),*,
            {
                type ResultType = [<Result $count>]<'fetch, $($name),*>;
                fn query(storage: &'fetch Storage) -> Self::ResultType {
                    let archetypes = <($($name),*,)>::find_matching_archetypes(storage);
                    $(let [<component_vec_locks_ $name:lower>] = <$name::ParameterFetch>::fetch(&archetypes[..]));*;
                    [<Result $count>] {
                        $([<$name:lower>]: [<component_vec_locks_ $name:lower>]),*,
                    }
                }
            }
        }
    };
}
query_impl!(1, A);
query_impl!(2, A, B);
query_impl!(3, A, B, C);
query_impl!(4, A, B, C, D);
query_impl!(5, A, B, C, D, E);
query_impl!(6, A, B, C, D, E, F);
query_impl!(7, A, B, C, D, E, F, G);
query_impl!(8, A, B, C, D, E, F, G, H);

///
/// Archetype matching trait and implementations
///
trait MatchArchetype<'a> {
    fn find_matching_archetypes(storage: &Storage) -> Vec<&Archetype>;
}
macro_rules! match_archetype_impl {
    ($($name:ident),*) => {
        paste!{
            impl<'a, $($name),*> MatchArchetype<'a> for ($($name),*,)
            where
                $($name: 'static + Debug + Parameter + ComponentMeta),*,
            {
                fn find_matching_archetypes(storage: &Storage) -> Vec<&Archetype> {
                    storage
                        .archetype_by_bundle_kind
                        .values()
                        .filter(|archetype| $(archetype.has_component::<$name>())&&*).collect()
                }
            }
        }
    };
}
match_archetype_impl!(A);
match_archetype_impl!(A, B);
match_archetype_impl!(A, B, C);
match_archetype_impl!(A, B, C, D);
match_archetype_impl!(A, B, C, D, E);
match_archetype_impl!(A, B, C, D, E, F);
match_archetype_impl!(A, B, C, D, E, F, G);
match_archetype_impl!(A, B, C, D, E, F, G, H);

///
/// Result structs for `Paremeter` tuples.
///
macro_rules! result_struct {
    ($count:tt, $($name:ident),*) => {
        paste!{
            pub struct [<Result $count>]<'fetch, $($name),*>
            where
                $($name: Parameter),*
            {
                $([<$name:lower>]: <$name::ParameterFetch as ParameterFetch<'fetch>>::ResultType),*
            }
        }
    };
}
result_struct!(1, A);
result_struct!(2, A, B);
result_struct!(3, A, B, C);
result_struct!(4, A, B, C, D);
result_struct!(5, A, B, C, D, E);
result_struct!(6, A, B, C, D, E, F);
result_struct!(7, A, B, C, D, E, F, G);
result_struct!(8, A, B, C, D, E, F, G, H);

///
/// ResultIter implementations for all `Result{#}` structs.
///
macro_rules! iter_return_parameter {
    ($name:ident) => {
        <<<$name::ParameterFetch as ParameterFetch<'fetch>>::ResultType as ResultIter<'borrow>>::IterType as Iterator>::Item
    };
}
macro_rules! result_iter_impl {
    ($count:tt, $($name:ident),*) => {
        paste!{
            impl<'borrow, 'fetch, $($name),*> ResultIter<'borrow> for [<Result $count>]<'fetch, $($name),*>
            where
                $($name: Parameter + ComponentMeta),*,
                $(<$name::ParameterFetch as ParameterFetch<'fetch>>::ResultType: ResultIter<'borrow>),*,
            {
                #[allow(unused_parens)]
                type IterType = impl Iterator<Item = ($(iter_return_parameter!($name)),*)>;
                fn result_iter(&'borrow mut self) -> Self::IterType {
                    izip!($(self.[<$name:lower>].result_iter()),*)
                }
            }
        }
    };
}
result_iter_impl!(1, A);
result_iter_impl!(2, A, B);
result_iter_impl!(3, A, B, C);
result_iter_impl!(4, A, B, C, D);
result_iter_impl!(5, A, B, C, D, E);
result_iter_impl!(6, A, B, C, D, E, F);
result_iter_impl!(7, A, B, C, D, E, F, G);
result_iter_impl!(8, A, B, C, D, E, F, G, H);
