//! The renderer's headless core (`docs/RENDERER.md`).
//!
//! A thin client over observed simulation state: it reads, it never writes
//! back, and nothing here is a dependency of `sim`. Floats are allowed in
//! this crate — the no-float rule binds the simulation, not the client
//! (`docs/RENDERER.md` §1) — but no value computed here may reach a tick.
//!
//! The first piece is the camera (`camera`): the original's view and
//! projection, written from reading its functions and checked against them
//! under unicorn (`tests/camera_sweep.rs`, `docs/RENDERER.md` §3.4).
//!
//! The second is the model decoders (`bh3`, `bha`, over `chunk`): the
//! install's `.BH3` and `.BHa` files read whole and self-checking, from
//! bytes handed in — nothing here opens the install (`docs/FORMATS.md` §3).

pub mod bh3;
pub mod bha;
pub mod camera;
pub mod chunk;

pub use camera::{Camera, Mat4, Projected, Screen};
