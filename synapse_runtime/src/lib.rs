//! Core runtime: minimal C ABI functions for Synapse compiled programs.

mod alloc;
mod io;

pub use alloc::*;
pub use io::*;