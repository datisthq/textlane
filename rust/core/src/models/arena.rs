use std::fmt;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

use bumpalo::Bump;
use bumpalo::collections::Vec as BumpVec;

/// A vector allocated in a document's arena that never drops its items.
///
/// Items must own nothing outside the arena: the arena frees everything at once, and
/// skipping drops keeps freeing a deeply nested tree from recursing once per level.
pub struct ArenaVec<'a, T>(ManuallyDrop<BumpVec<'a, T>>);

impl<'a, T> ArenaVec<'a, T> {
    pub fn new_in(alloc: &'a Bump) -> Self {
        Self(ManuallyDrop::new(BumpVec::new_in(alloc)))
    }

    pub fn with_capacity_in(capacity: usize, alloc: &'a Bump) -> Self {
        Self(ManuallyDrop::new(BumpVec::with_capacity_in(
            capacity, alloc,
        )))
    }
}

impl<'a, T> Deref for ArenaVec<'a, T> {
    type Target = BumpVec<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ArenaVec<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T> IntoIterator for ArenaVec<'a, T> {
    type Item = T;
    type IntoIter = bumpalo::collections::vec::IntoIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        ManuallyDrop::into_inner(self.0).into_iter()
    }
}

impl<'v, T> IntoIterator for &'v ArenaVec<'_, T> {
    type Item = &'v T;
    type IntoIter = std::slice::Iter<'v, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'v, T> IntoIterator for &'v mut ArenaVec<'_, T> {
    type Item = &'v mut T;
    type IntoIter = std::slice::IterMut<'v, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<T: fmt::Debug> fmt::Debug for ArenaVec<'_, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl<T: PartialEq> PartialEq for ArenaVec<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.0[..] == other.0[..]
    }
}
