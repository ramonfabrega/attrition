//! Independent oracle sweeps; original sweep:: test names are preserved.

// Preserve crate:: paths while exposing only the library's public API.
pub use sim::*;

#[path = "../src/sweep/standalone_a_f.rs"]
mod sweep;
#[path = "../src/test_support/install.rs"]
mod testenv;
