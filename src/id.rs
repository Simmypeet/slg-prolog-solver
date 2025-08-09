use std::{hash::Hash, marker::PhantomData};

pub struct ID<T> {
    id: usize,
    _marker: PhantomData<Box<T>>,
}

impl<T> std::fmt::Debug for ID<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ID").field("id", &self.id).finish()
    }
}

impl<T> Clone for ID<T> {
    fn clone(&self) -> Self { *self }
}

impl<T> Copy for ID<T> {}

impl<T> PartialEq for ID<T> {
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}

impl<T> Eq for ID<T> {}

impl<T> Ord for ID<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.id.cmp(&other.id) }
}

impl<T> PartialOrd for ID<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Hash for ID<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.id.hash(state); }
}

impl<T> Default for ID<T> {
    fn default() -> Self { ID { id: 0, _marker: PhantomData } }
}

impl<T> ID<T> {
    pub fn bump_id(&mut self) -> Self {
        let id = self.id;
        self.id += 1;
        ID { id, _marker: PhantomData }
    }
}
