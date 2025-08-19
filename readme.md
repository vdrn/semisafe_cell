# semisafe_cell

A interior-mutability types that enforce runtime borrow checking in debug builds or when `full_safety` is enabled, **otherwise the caller is reponsible for upholding Rust's aliasing rules**.

Variants: 
- `SemiSafeCell<T>`: single-threaded (uses `RefCell` for runtime borrow checking).
- `SyncSemiSafeCell<T>`: thread-safe (uses `atomic_refcell::AtomicRefCell` for runtime borrow checking).

Features:
- `full_safety`: enables runtime borrow checks in release too.
- `coerce_unsized`: nightly-only; allows `CoerceUnsized` coercions.
