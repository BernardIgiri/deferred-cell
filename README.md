# `deferred-cell`

A single-assignment, weak reference wrapper for static cyclic node graphs with late initialization.

This crate provides a lightweight alternative to `RefCell<Option<Weak<T>>>` when building reference graphs such as cyclic trees, linked lists, and bidirectional or circular structures. It enables you to cleanly and safely establish weak links between nodes **after** they are constructed.

---

## ✨ Features

- ✅ Write-once semantics using `OnceCell<Weak<T>>`
- ✅ Safe and ergonomic API for deferred initialization
- ✅ Strong test coverage
- ✅ Optional helper type for setting values: `DeferredMut`
- ✅ Iterator extension trait for working with collections

---

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
deferred-cell = "0.1"
```

---

## 🧠 Motivation

In Rust, it’s often tricky to build cyclic data structures due to ownership rules. A common workaround is:

```rust
RefCell<Option<Weak<T>>>
```

However, this allows re-assignment and mutation, which is overkill in cases where the weak reference should be set only once. `deferred-cell` simplifies this pattern with explicit, single-assignment behavior and no runtime borrow checking.

---

## 🚀 Example

```rust
use deferred_cell::{Deferred, DeferredMut, DeferredError};
use std::rc::Rc;

struct Node {
    value: String,
    neighbor: Deferred<Node>,
}

fn main() -> Result<(), DeferredError> {
    let node = Rc::new(Node {
        value: "A".into(),
        neighbor: Deferred::default(),
    });

    let neighbor = Rc::new(Node {
        value: "B".into(),
        neighbor: Deferred::default(),
    });

    // Assign weak reference after both nodes are constructed
    DeferredMut::from(&node.neighbor).try_set(&neighbor)?;

    // Access the neighbor
    let linked = node.neighbor.try_get()?;
    assert_eq!(linked.value, "B");

    Ok(())
}
```

---

## 🔧 API Highlights

```rust
let d: Deferred<T> = Deferred::default();
let rc: Rc<T> = ...;

// One-time set
DeferredMut::from(&d).try_set(&rc)?;

// Later access
let strong: Rc<T> = d.try_get()?;

// Optional checks
if d.is_ready() { ... }
```

Also includes a `DeferredIteratorExt` trait to streamline iteration:

```rust
let values: Vec<_> = list
    .into_iter()
    .get_deferred()
    .map(|rc| rc.value.clone())
    .collect();
```

---

## ⚠️ Errors

Two error types are defined:

- `DeferredError::DuplicateInitialization` – if `try_set()` is called more than once
- `DeferredError::NotInitializedError` – if `get()` or `try_get()` is called before a value is set

---
