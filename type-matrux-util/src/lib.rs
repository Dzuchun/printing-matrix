//! These are random utility types that feel separate enough to live in a dedicated crate

#![no_std]
#[cfg(any(feature = "sow", feature = "asow"))]
extern crate alloc;

/// This holds "Sow" (share-on-write) types
#[cfg(any(feature = "sow", feature = "asow"))]
pub mod sow;
