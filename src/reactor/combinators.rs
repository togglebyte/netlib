use super::{Reactor, Reaction};

// -----------------------------------------------------------------------------
//     - Chain -
// -----------------------------------------------------------------------------
pub struct Chain<A, B>
where
    A: Reactor,
    B: Reactor<Input = A::Output>,
{
    first: A,
    second: B,
}

impl<A, B> Chain<A, B>
where
    A: Reactor,
    B: Reactor<Input = A::Output>,
{
    pub fn new(first: A, second: B) -> Self {
        Self {
            first,
            second
        }
    }
}

impl<A, B> Reactor for Chain<A, B>
where
    A: Reactor,
    B: Reactor<Input = A::Output>,
{
    type Input = A::Input;
    type Output = B::Output;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match self.first.react(reaction) {
            Reaction::Value(val) => self.second.react(Reaction::Value(val)),
            Reaction::Continue => Reaction::Continue,
            Reaction::Event(e) => self.second.react(Reaction::Event(e)),
        }
    }
}

