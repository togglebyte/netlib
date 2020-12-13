use std::marker::PhantomData;

use crate::{Reaction, Reactor};

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
