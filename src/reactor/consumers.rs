use std::marker::PhantomData;

use crate::{Reaction, Reactor};

// -----------------------------------------------------------------------------
//     - Map -
// -----------------------------------------------------------------------------
pub struct Map<T, F, U>
where
    T: Reactor,
    F: FnMut(T::Output) -> U,
{
    reactor: T,
    f: F,
    _p1: PhantomData<U>,
}

impl<T, F, U> Map<T, F, U>
where
    T: Reactor,
    F: FnMut(T::Output) -> U,
{
    pub fn new(reactor: T, f: F) -> Self {
        Self {
            reactor,
            f,
            _p1: PhantomData,
        }
    }
}

impl<T, F, U> Reactor for Map<T, F, U>
where
    T: Reactor,
    F: FnMut(T::Output) -> U,
{
    type Input = T::Input;
    type Output = U;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match self.reactor.react(reaction) {
            Reaction::Value(val) => Reaction::Value((self.f)(val)),
            Reaction::Continue => Reaction::Continue,
            Reaction::Event(e) => Reaction::Event(e),
        }
    }
}

// -----------------------------------------------------------------------------
//     - Filter -
// -----------------------------------------------------------------------------
pub struct FilterMap<T, F, U>
    where 
        T: Reactor,
        F: FnMut(T::Output) -> Option<U>,
{
    reactor: T,
    f: F,
    _p1: PhantomData<U>
}

impl<T, F, U> FilterMap<T, F, U>
where
    T: Reactor,
    F: FnMut(T::Output) -> Option<U>,
{
    pub fn new(reactor: T, f: F) -> Self {
        Self {
            reactor,
            f,
            _p1: PhantomData,
        }
    }
}


impl<T, F, U> Reactor for FilterMap<T, F, U>
    where 
        T: Reactor,
        F: FnMut(T::Output) -> Option<U>,
{
    type Input = T::Input;
    type Output = U;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match self.reactor.react(reaction) {
            Reaction::Event(ev) => Reaction::Event(ev),
            Reaction::Value(val) => match (self.f)(val) {
                Some(v) => Reaction::Value(v),
                None => Reaction::Continue,
            }
            Reaction::Continue => Reaction::Continue,
        }
    }
}
