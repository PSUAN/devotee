#![deny(missing_docs)]

//! `devotee` is small visualization engine based on [pixels](https://crates.io/crates/pixels) and [winit](https://crates.io/crates/winit).
//! It aims to provide minimalist visualization platform.

/// Application is the visualization core.
/// It provides basic EventLoop handling.
pub mod app;
/// Math is dedicated to basic concepts like Vectors.
pub mod math;
/// Node is the building block of the `devotee` project.
/// Currently it is used only in the app.
pub mod node;
/// Set of visualization primitives.
pub mod visual;
