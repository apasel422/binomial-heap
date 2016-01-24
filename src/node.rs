use std::mem;

#[derive(Clone)]
pub struct Node<T> {
    item: T,
    order: usize,
    next: Option<Box<Node<T>>>,
    child: Option<Box<Node<T>>>,
}

pub fn append<T: Ord>(root: &mut Box<Node<T>>, other: Option<Box<Node<T>>>) {
    if let Some(other) = other {
        merge(root, other);
        coalesce(root);
    }
}

pub fn push<T: Ord>(root: &mut Option<Box<Node<T>>>, item: T) {
    let node = Some(Box::new(Node { item: item, order: 0, next: None, child: None }));

    match *root {
        None => *root = node,
        Some(ref mut root) => append(root, node),
    }
}

pub fn peek<T: Ord>(root: &Option<Box<Node<T>>>) -> Option<&T> {
    root.as_ref().map(|mut max| {
        let mut a = &max.next;

        while let Some(ref b) = *a {
            if b.item > max.item { max = b; }
            a = &b.next;
        }

        &max.item
    })
}

pub fn pop<T: Ord>(root: &mut Option<Box<Node<T>>>, len: &mut usize) -> Option<T> {
    remove_max(root).map(|max| {
        let max = *max;
        let Node { item, child, order: _order, next: _next } = max;

        match *root {
            None => *root = child,
            Some(ref mut root) => append(root, child),
        }

        *len -= 1;
        item
    })
}

pub fn iter<T>(root: &Option<Box<Node<T>>>, len: usize) -> Iter<T> {
    debug_assert!(root.is_some() ^ (len == 0));
    Iter { nodes: root.as_ref().map(|root| &**root).into_iter().collect(), len: len }
}

pub fn into_iter<T>(root: Option<Box<Node<T>>>, len: usize) -> IntoIter<T> {
    debug_assert!(root.is_some() ^ (len == 0));
    IntoIter { nodes: root.into_iter().collect(), len: len }
}

/// An iterator that yields references to the items in a `BinomialHeap` in arbitrary order.
///
/// Acquire through [`BinomialHeap::iter`](struct.BinomialHeap.html#method.iter).
pub struct Iter<'a, T: 'a> {
    nodes: Vec<&'a Node<T>>,
    len: usize,
}

impl<'a, T> Clone for Iter<'a, T> {
    fn clone(&self) -> Self {
        Iter { nodes: self.nodes.clone(), len: self.len }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.nodes.pop().map(|node| {
            self.len -= 1;

            if let Some(ref next) = node.next { self.nodes.push(next); }
            if let Some(ref child) = node.child { self.nodes.push(child); }

            &node.item
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

/// An iterator that yields the items in a `BinomialHeap` in arbitrary order.
///
/// Acquire through [`IntoIterator::into_iter`](struct.BinomialHeap.html#method.into_iter).
pub struct IntoIter<T> {
    nodes: Vec<Box<Node<T>>>,
    len: usize,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.nodes.pop().map(|node| {
            self.len -= 1;

            let node = *node;
            let Node { item, next, child, order: _order } = node;

            if let Some(next) = next { self.nodes.push(next); }
            if let Some(child) = child { self.nodes.push(child); }

            item
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.len
    }
}

/// Merges the sibling list rooted at `b` into the sibling list rooted at `a` such that the
/// resulting list is monotonically increasing by order.
///
/// The lists rooted at `a` and `b` must be monotonically increasing by order.
///
/// This method should always be followed by `coalesce(a)`.
fn merge<T>(mut a: &mut Box<Node<T>>, mut b: Box<Node<T>>) {
    loop {
        let a_ = a;

        if a_.order > b.order {
            mem::swap(a_, &mut b);
        }

        match a_.next {
            None => return a_.next = Some(b),
            Some(ref mut next) => a = next,
        }
    }
}

/// Makes `b` a child of `a`.
fn link<T: Ord>(a: &mut Node<T>, mut b: Box<Node<T>>) {
    debug_assert!(a.order == b.order);
    debug_assert!(b.next.is_none());
    debug_assert!(a.item >= b.item);

    b.next = a.child.take();
    a.child = Some(b);
    a.order += 1;
}

/// Coalesces nodes in the given sibling list in order to restore the binomial heap property that
/// no two nodes in the list have the same order.
///
/// The list rooted at `a` must be monotonically increasing by order and its individual nodes must
/// be valid max-heaps.
///
/// This method should always be preceded by `merge`.
fn coalesce<T: Ord>(mut a: &mut Box<Node<T>>) {
    enum Case {
        A,
        B,
        C,
    }

    loop {
        let a_ = a;

        let case = match a_.next {
            None => return,
            Some(ref b) =>
                if a_.order != b.order || b.next.as_ref().map_or(false, |c| c.order == b.order) {
                    Case::A
                } else if a_.item >= b.item {
                    Case::B
                } else {
                    Case::C
                },
        };

        match case {
            Case::A => a = a_.next.as_mut().unwrap(),
            Case::B => {
                let mut b = a_.next.take().unwrap();
                a_.next = b.next.take();
                link(a_, b);

                match a_.next {
                    None => return,
                    Some(ref mut c) => a = c,
                }
            }
            Case::C => {
                let mut b = a_.next.take().unwrap();
                mem::swap(a_, &mut b);
                link(a_, b);
                a = a_;
            }
        }
    }
}

/// Removes and returns the node with the maximum item from the sibling list rooted at `a`.
fn remove_max<T: Ord>(mut a: &mut Option<Box<Node<T>>>) -> Option<Box<Node<T>>> {
    a.take().map(|mut max| {
        *a = max.next.take();

        loop {
            let a_ = a;

            match *a_ {
                None => return max,
                Some(ref mut b) => {
                    if b.item > max.item {
                        max.next = b.next.take();
                        mem::swap(&mut max, b);
                    }
                }
            }

            a = &mut a_.as_mut().unwrap().next;
        }
    })
}
