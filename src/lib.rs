//! A priority queue based on a [binomial heap].
//!
//! See [`BinomialHeap`](struct.BinomialHeap.html) for details.
//!
//! [binomial heap]: https://en.wikipedia.org/wiki/Binomial_heap

#![deny(missing_docs)]

use std::fmt::{self, Debug};
use std::marker::PhantomData;
use std::mem;

mod node;

pub use node::{IntoIter, Iter};

/// A priority queue based on a binomial heap.
///
/// Like [`BinaryHeap`], `BionmialHeap` is an implementation of a priority queue. Unlike
/// `BinaryHeap`, `BionmialHeap` provides an efficient `append` method, at the cost of greater
/// memory usage, slower iteration, and poor cache locality.
///
/// # Time Complexity
///
/// | Operation                      | Time Complexity        |
/// |--------------------------------|------------------------|
/// | [`append`](#method.append)     | `O(log n)` (amortized) |
/// | [`peek`](#method.peek)         | `O(log n)`             |
/// | [`pop`](#method.pop)           | `O(log n)`             |
/// | [`push`](#method.push)         | `O(1)` (amortized)     |
/// | [`push_pop`](#method.push_pop) | `O(log n)`             |
/// | [`replace`](#method.replace)   | `O(log n)`             |
///
/// [`BinaryHeap`]: https://doc.rust-lang.org/std/collections/struct.BinaryHeap.html
#[derive(Clone)]
pub struct BinomialHeap<T: Ord> {
    root: Option<Box<node::Node<T>>>,
    len: usize,
}

impl<T: Ord> BinomialHeap<T> {
    /// Returns a new heap.
    pub fn new() -> Self {
        BinomialHeap { root: None, len: 0 }
    }

    /// Checks if the heap is empty.
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Returns the number of items in the heap.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns an iterator that yields references to the items in the heap in arbitrary order.
    pub fn iter(&self) -> Iter<T> {
        node::iter(&self.root, self.len)
    }

    /// Returns a reference to the greatest item in the heap.
    ///
    /// Returns `None` if the heap is empty.
    pub fn peek(&self) -> Option<&T> {
        node::peek(&self.root)
    }

    /// Pushes the given item onto the heap.
    pub fn push(&mut self, item: T) {
        node::push(&mut self.root, item);
        self.len += 1;
    }

    /// Moves the given heap's items into the heap, leaving the given heap empty.
    ///
    /// This is equivalent to, but likely to be faster than, the following:
    ///
    /// ```
    /// # let mut heap = binomial_heap::BinomialHeap::<i32>::new();
    /// # let mut heap2 = binomial_heap::BinomialHeap::new();
    /// heap.extend(heap2.drain());
    /// ```
    pub fn append(&mut self, other: &mut Self) {
        match self.root {
            None => mem::swap(self, other),
            Some(ref mut root) => {
                node::append(root, other.root.take());
                self.len += mem::replace(&mut other.len, 0);
            }
        }
    }

    /// Pushes the given item onto the heap, then removes the greatest item in the heap and returns
    /// it.
    ///
    /// This method is equivalent to, but likely faster than, the following:
    ///
    /// ```
    /// # let mut heap = binomial_heap::BinomialHeap::new();
    /// # let item = 0;
    /// heap.push(item);
    /// let max = heap.pop().unwrap();
    /// ```
    pub fn push_pop(&mut self, item: T) -> T {
        self.push(item);
        self.pop().expect("heap was empty")
    }

    /// Removes the greatest item in the heap, then pushes the given item onto the heap.
    ///
    /// Returns the item that was removed, or `None` if the heap was empty.
    ///
    /// This method is equivalent to, but likely faster than, the following:
    ///
    /// ```
    /// # let mut heap = binomial_heap::BinomialHeap::new();
    /// # let item = 0;
    /// let max = heap.pop();
    /// heap.push(item);
    /// ```
    pub fn replace(&mut self, item: T) -> Option<T> {
        let max = self.pop();
        self.push(item);
        max
    }

    /// Removes the greatest item in the heap and returns it.
    ///
    /// Returns `None` if the heap was empty.
    ///
    /// If a call to this method is immediately preceded by a call to [`push`], consider using
    /// [`push_pop`] instead. If a call to this method is immediately followed by a call to
    /// [`push`], consider using [`replace`] instead.
    ///
    /// [`push`]: #method.push
    /// [`push_pop`]: #method.push_pop
    /// [`replace`]: #method.replace
    pub fn pop(&mut self) -> Option<T> {
        node::pop(&mut self.root, &mut self.len)
    }

    /// Removes all items from the heap.
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// Removes all items from the heap and returns an iterator that yields them in arbitrary
    /// order.
    ///
    /// All items are removed even if the iterator is not exhausted. However, the behavior of
    /// this method is unspecified if the iterator is leaked (e.g. via [`mem::forget`]).
    ///
    /// [`mem::forget`]: https://doc.rust-lang.org/std/mem/fn.forget.html
    pub fn drain(&mut self) -> Drain<T> {
        Drain { iter: mem::replace(self, Self::new()).into_iter(), marker: PhantomData }
    }
}

impl<T: Ord + Debug> Debug for BinomialHeap<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<T: Ord> Default for BinomialHeap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord> Extend<T> for BinomialHeap<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, items: I) {
        for item in items { self.push(item); }
    }
}

impl<T: Ord> std::iter::FromIterator<T> for BinomialHeap<T> {
    fn from_iter<I: IntoIterator<Item = T>>(items: I) -> Self {
        let mut heap = Self::new();
        heap.extend(items);
        heap
    }
}

impl<T: Ord> IntoIterator for BinomialHeap<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        node::into_iter(self.root, self.len)
    }
}

impl<'a, T: Ord> IntoIterator for &'a BinomialHeap<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

/// An iterator that drains a `BinomialHeap`, yielding its items in arbitrary order.
///
/// Acquire through [`BinomialHeap::drain`](struct.BinomialHeap.html#method.drain).
pub struct Drain<'a, T: 'a> {
    iter: IntoIter<T>,
    marker: PhantomData<&'a mut IntoIter<T>>,
}

impl<'a, T: Ord> Iterator for Drain<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T: Ord> ExactSizeIterator for Drain<'a, T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

#[allow(dead_code)]
fn assert_covariance() {
    fn heap<'a, T: Ord>(heap: BinomialHeap<&'static T>) -> BinomialHeap<&'a T> {
        heap
    }

    fn into_iter<'a, T: Ord>(iter: IntoIter<&'static T>) -> IntoIter<&'a T> {
        iter
    }

    fn iter<'i, 'a, T: Ord>(iter: Iter<'i, &'static T>) -> Iter<'i, &'a T> {
        iter
    }
}
