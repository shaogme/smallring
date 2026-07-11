//! Shim module to abstract over std and loom primitives.
//!
//! This module provides a unified interface for synchronization primitives that transparently
//! switches between `std` implementation (for production) and `loom` implementation (for testing).

#[cfg(not(any(feature = "loom", feature = "portable-atomic")))]
pub use core::sync::atomic;

#[cfg(all(not(feature = "loom"), feature = "portable-atomic"))]
pub use portable_atomic as atomic;

#[cfg(feature = "loom")]
pub use loom::sync::atomic;

#[cfg(not(any(feature = "loom", feature = "portable-atomic")))]
pub mod sync {
    pub use alloc::sync::Arc;
}

#[cfg(all(not(feature = "loom"), feature = "portable-atomic"))]
pub mod sync {
    pub use portable_atomic_util::Arc;
}

#[cfg(feature = "loom")]
pub mod sync {
    pub use loom::sync::Arc;
}
