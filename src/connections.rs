use std::ops::{IndexMut, Index};

#[derive(Debug)] 
pub struct Connections<T> {
    inner: Vec<Option<T>>,
    free_slots: Vec<usize>,
}

impl<T> Connections<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        // let inner = (0..capacity).map(|_| None).collect::<Vec<_>>();

        Self {
            inner: Vec::with_capacity(capacity),
            free_slots: Vec::with_capacity(capacity),
        }
    }

    pub fn insert(&mut self, val: T) -> usize {
        match self.free_slots.pop() {
            Some(index) => {
                self.inner.insert(index, Some(val));
                index
            }
            None => {
                let index = self.inner.len();
                self.inner.push(Some(val));
                index
            }
        }
    }

    pub fn remove(&mut self, index: usize) {
        self.inner[index].take();
        self.free_slots.push(index);
    }
}

impl<T> Index<usize> for Connections<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        match &self.inner[index] {
            Some(ref val) => val,
            None => panic!(),
        }
    }
}

impl<T> IndexMut<usize> for Connections<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match &mut self.inner[index] {
            Some(ref mut val) => val,
            None => panic!(),
        }
    }
}
