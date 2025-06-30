//! `deferred-cell`: A single-assignment, weak reference wrapper for write-once cyclic node graphs with late initialization.
//!
//! This crate provides a lightweight alternative to runtime mutation or interior mutability
//! when building write-once reference graphs such as cyclic trees or bidirectional structures.
//!
//! Instead of littering your code with `RefCell<Option<Weak<T>>>`, you can use `Deferred<T>` to express
//! write-once assignment of weak links clearly and safely.
//!
//! ## Example
//!
//! ```rust
//! use deferred_cell::{Deferred, DeferredMut, DeferredError};
//! use std::rc::Rc;
//!
//! struct Node {
//!     value: String,
//!     neighbor: Deferred<Node>,
//! }
//!
//! fn main() -> Result<(), DeferredError> {
//!     let node = Rc::new(Node {
//!         value: "A".into(),
//!         neighbor: Deferred::default(),
//!     });
//!     let neighbor = Rc::new(Node {
//!         value: "B".into(),
//!         neighbor: Deferred::default(),
//!     });
//!
//!     DeferredMut::from(&node.neighbor).try_set(&neighbor)?;
//!     let linked = node.neighbor.try_get()?;
//!     assert_eq!(linked.value, "B");
//!     assert_eq!(node.neighbor.get().value, "B");
//!
//!     Ok(())
//! }
//! ```
#![deny(clippy::unwrap_used, clippy::expect_used)]
#![warn(clippy::all, clippy::nursery)]

use std::{
    cell::OnceCell,
    rc::{Rc, Weak},
};

use thiserror::Error;

/// Errors thrown by deferred-cell
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DeferredError {
    #[error("Cannot initialize Deferred twice!")]
    DuplicateInitialization(),
    #[error("Cannot use uninitialized value!")]
    NotInitializedError(),
}

/// A write-once, weak reference wrapper, for late initialization.
#[derive(Debug, Clone)]
pub struct Deferred<T>(OnceCell<Weak<T>>);

impl<T> Default for Deferred<T> {
    fn default() -> Self {
        Self(OnceCell::new())
    }
}

impl<T> Deferred<T> {
    pub fn try_get(&self) -> Result<Rc<T>, DeferredError> {
        self.0
            .get()
            .ok_or(DeferredError::NotInitializedError())?
            .upgrade()
            .ok_or(DeferredError::NotInitializedError())
    }
    #[must_use]
    pub fn get(&self) -> Rc<T> {
        #[allow(clippy::expect_used)]
        self.try_get().expect("Deferred value not yet set!")
    }
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.0.get().is_some()
    }
}

/// Exposes write-once interface on Deferred<T>
#[derive(Debug, Clone)]
pub struct DeferredMut<'a, T>(&'a Deferred<T>);

impl<'a, T> DeferredMut<'a, T> {
    pub const fn from(cell: &'a Deferred<T>) -> Self {
        Self(cell)
    }
    pub fn try_set(&self, value: &Rc<T>) -> Result<(), DeferredError> {
        self.0
            .0
            .set(Rc::downgrade(value))
            .map_err(|_| DeferredError::DuplicateInitialization())
    }
    #[inline]
    pub fn can_set(&self) -> bool {
        self.0.0.get().is_none()
    }
}

/// Iterator extension trait to improve the ergonomics of `Deferred<T>` collections
pub trait DeferredIteratorExt<T>: Iterator<Item = Deferred<T>> + Sized {
    /// Returns an iterator of `Rc<T>` from an iterator of `Deferred<T>`.
    ///
    /// # Panics
    /// Panics if any `Deferred<T>` is not initialized.
    fn get_deferred(self) -> impl Iterator<Item = Rc<T>> {
        self.map(|d| d.get())
    }

    /// Returns an iterator of `Result<Rc<T>, DeferredError>` from an iterator of `Deferred<T>`.
    fn try_get_deferred(self) -> impl Iterator<Item = Result<Rc<T>, DeferredError>> {
        self.map(|d| d.try_get())
    }
}

impl<T, I> DeferredIteratorExt<T> for I where I: Iterator<Item = Deferred<T>> {}

// Allowed in tests
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, Clone)]
    struct Node {
        value: String,
        neighbors: Vec<Deferred<Node>>,
    }
    impl Node {
        fn new(value: &str, n_neighbors: usize) -> Rc<Self> {
            Rc::new(Self {
                value: value.into(),
                neighbors: (0..n_neighbors)
                    .map(|_| Deferred::default())
                    .collect::<Vec<_>>(),
            })
        }
    }

    fn make_cyclic_graph() -> Vec<Rc<Node>> {
        /*
                   North
                /    |     \
            East - Center - West
                \    |     /
                   South
        */
        let center = Node::new("Center", 4);
        let north = Node::new("North", 3);
        let east = Node::new("East", 3);
        let south = Node::new("South", 3);
        let west = Node::new("West", 3);

        DeferredMut::from(&center.neighbors[0])
            .try_set(&north)
            .unwrap();
        DeferredMut::from(&center.neighbors[1])
            .try_set(&west)
            .unwrap();
        DeferredMut::from(&center.neighbors[2])
            .try_set(&south)
            .unwrap();
        DeferredMut::from(&center.neighbors[3])
            .try_set(&east)
            .unwrap();

        DeferredMut::from(&north.neighbors[0])
            .try_set(&west)
            .unwrap();
        DeferredMut::from(&north.neighbors[1])
            .try_set(&center)
            .unwrap();
        DeferredMut::from(&north.neighbors[2])
            .try_set(&east)
            .unwrap();

        DeferredMut::from(&west.neighbors[0])
            .try_set(&north)
            .unwrap();
        DeferredMut::from(&west.neighbors[1])
            .try_set(&south)
            .unwrap();
        DeferredMut::from(&west.neighbors[2])
            .try_set(&center)
            .unwrap();

        DeferredMut::from(&south.neighbors[0])
            .try_set(&center)
            .unwrap();
        DeferredMut::from(&south.neighbors[1])
            .try_set(&west)
            .unwrap();
        DeferredMut::from(&south.neighbors[2])
            .try_set(&east)
            .unwrap();

        DeferredMut::from(&east.neighbors[0])
            .try_set(&north)
            .unwrap();
        DeferredMut::from(&east.neighbors[1])
            .try_set(&center)
            .unwrap();
        DeferredMut::from(&east.neighbors[2])
            .try_set(&south)
            .unwrap();

        vec![center, north, east, south, west]
    }

    #[test]
    fn cyclic_graph() {
        let graph = make_cyclic_graph();
        let center = graph.first().unwrap();

        assert_eq!(center.value, "Center");

        let north = center.neighbors[0].get();
        let west = north.neighbors[0].get();
        let south = west.neighbors[1].get();
        let east = south.neighbors[2].get();
        let center_again = east.neighbors[1].get();

        assert_eq!(north.value, "North");
        assert_eq!(west.value, "West");
        assert_eq!(south.value, "South");
        assert_eq!(east.value, "East");
        assert_eq!(center_again.value, "Center");
    }

    #[test]
    fn duplicate_initialization_fails() {
        let graph = make_cyclic_graph();
        let center = graph.first().unwrap();

        let neighbor_slot = &center.neighbors[0];
        let mutator = DeferredMut::from(neighbor_slot);
        let duplicate_set = mutator.try_set(center);

        assert!(
            matches!(duplicate_set, Err(DeferredError::DuplicateInitialization())),
            "Expected DuplicateInitialization error"
        );
    }

    #[test]
    fn uninitialized_access_fails() {
        let uninitialized: Deferred<Node> = Deferred::default();
        let result = uninitialized.try_get();

        assert!(
            matches!(result, Err(DeferredError::NotInitializedError())),
            "Expected NotInitializedError"
        );
    }

    #[test]
    fn iterator_extension_works() {
        let graph = make_cyclic_graph();
        let center = graph.first().unwrap();

        let values: Vec<_> = center
            .neighbors
            .clone()
            .into_iter()
            .get_deferred()
            .map(|rc| rc.value.clone())
            .collect();

        assert_eq!(values, vec!["North", "West", "South", "East"]);
    }

    #[test]
    fn deferred_state_checking() {
        let graph = make_cyclic_graph();
        let center = graph.first().unwrap();
        let neighbor = &center.neighbors[0];

        assert!(neighbor.is_ready());
        let m = DeferredMut::from(neighbor);
        assert!(!m.can_set());
    }
}
