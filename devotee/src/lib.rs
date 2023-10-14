#![deny(missing_docs)]

//! `devotee` is small visualization engine based on [winit](https://crates.io/crates/winit) and [rodio](https://crates.io/crates/rodio).
//! It aims to provide minimalist visualization capabilities.
//!
//! To use `devotee` specify `Config`, create `Setup`, initialize `App` and `run` it.

#[cfg(all(feature = "back-pixels", feature = "back-softbuffer"))]
compile_error!("Can't have two backends at the same time enabled");

#[cfg(not(any(feature = "back-pixels", feature = "back-softbuffer")))]
compile_error!("Need one backend");

pub use devotee_backend::winit;

/// Application is the visualization core.
/// It provides basic event loop handling.
pub mod app;
/// Set of random useful stuff.
pub mod util;
/// Set of visualization primitives.
pub mod visual;
