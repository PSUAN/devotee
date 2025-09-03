#![deny(missing_docs)]

//! Devotee is a simple visualization engine.

pub extern crate devotee_backend as backend;

/// Default application implementation for the devotee project.
pub mod app;

/// Input implementations.
pub mod input;

/// Various utility.
pub mod util;

/// Visual-related module.
pub mod visual;
