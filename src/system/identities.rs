use std::mem::swap;
use log::error;

#[derive(Debug, Clone, Copy)] 
enum Identity {
    Vacant { next: usize },
    Occupied,
}

// -----------------------------------------------------------------------------
//     - Reactor identities -
// -----------------------------------------------------------------------------
pub(super) struct Identities {
    inner: Vec<Identity>,
    next: usize,
}

impl Identities {
    pub(super) fn empty() -> Self {
        Self { 
            inner: Vec::new(),
            next: 0,
        }
    }

    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
            next: 0,
        }
    }

    pub(super) fn free(&mut self, index: u64) {
        let mut vacant = Identity::Vacant { next: self.next };
        let mut occ = &mut self.inner[index as usize];

        match occ {
            Identity::Occupied => {
                swap(&mut vacant, &mut occ);
                self.next = index as usize;
            }
            Identity::Vacant { .. } => error!("tried to free a vacant entry"),
        }


    }

    pub(super) fn reserve(&mut self) -> u64 {
        let id = self.next as u64;

        if self.next == self.inner.len() {
            self.inner.push(Identity::Occupied);
            self.next = self.inner.len();
        } else {
            match self.inner.remove(self.next) {
                Identity::Vacant { next } => {
                    self.inner.insert(self.next, Identity::Occupied);
                    self.next = next;
                }
                Identity::Occupied => panic!("tried to reserve occupied entry"),
            }
        }

        id
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_one_to_empty() {
        let mut idents = Identities::empty();
        assert_eq!(idents.reserve(), 0);
    }

    #[test]
    fn free_one() {
        let mut idents = Identities::empty();
        assert_eq!(idents.reserve(), 0); // 0
        assert_eq!(idents.reserve(), 1); // 1
        assert_eq!(idents.reserve(), 2); // 2
        // Free one in the middle
        idents.free(1);

        // The next one should thus be 1
        assert_eq!(idents.next, 1);
        assert_eq!(idents.reserve(), 1);

        // And since all the entries are now occupied until the
        // last one, the next entry should be the length of the
        // `Identities` instance:
        assert_eq!(idents.next, 3);
        assert_eq!(idents.inner.len(), 3);
        assert_eq!(idents.reserve(), 3);
    }

    #[test]
    fn free_non_existing() {
        let mut idents = Identities::empty();
        idents.reserve();
        idents.free(0);
        idents.free(0); // freed twice
    }
}
