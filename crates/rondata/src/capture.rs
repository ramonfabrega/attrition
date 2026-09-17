//! The capture's text, read the way a crate under `forbid(unsafe_code)`
//! can read it.
//!
//! **This module was a memory map, and is not one any more (item 280).**
//! Item 260 found that nothing a parse holds is ever given back: a probe
//! that parsed three captures in one process and read its own resident set
//! after each `drop` found it had not fallen by a byte — 5,332 MiB held
//! with nothing alive — because macOS's allocator keeps a freed block of
//! that size rather than unmapping it. So the two large things a parse
//! held, the capture's text and the arena's chunks, were mapped instead of
//! allocated, and `munmap` gave back what `free` did not.
//!
//! What that cost was the workspace's one memory tenet. A mapping cannot
//! be made in safe Rust, by anyone: `memmap2::Mmap::map` is an `unsafe fn`
//! and keeps that signature on purpose, because a `MAP_PRIVATE` mapping of
//! a file is undefined behaviour the moment another process rewrites or
//! truncates it under the `&[u8]` Rust believes is frozen. That hazard is
//! live in this repo rather than theoretical: `tools/gamelog/runqueue.sh`
//! renames `gamelog.txt` over an archive name, `tools/gamelog/captures.txt`
//! warns in its own header that re-using a run number silently overwrites
//! an existing archive, and a capture runs concurrently with the test
//! suite by design. The module made it worse than memmap2 would have: the
//! FFI was hand-rolled, `read` was a **safe** fn handing back a
//! `Deref<Target = str>` over the mapping, and the bytes went through
//! `from_utf8_unchecked` — so no caller could see the obligation, and a
//! rewrite mid-read was UB rather than a SIGBUS.
//!
//! So the trade is taken the other way (`docs/DECISIONS.md` 37): the tree
//! is back under one `forbid(unsafe_code)`, no crate in the tree spends an
//! allowance against it, and the memory is paid back to the meter. What survives of item 260 is the half that was always safe and
//! was always the larger win — the parse is lazy, the frames are indexed
//! and one is read when a caller asks (`gamelog::Log::parse`,
//! `Block::ensure`), which took the arena of a 1.3 GB capture from 2,221
//! MiB to 17.
//!
//! `indexed::IndexedCapture` now provides that successor for ordinary FRAME
//! records: safe file reads, a bounded offset cache, and one owned Frame yielded
//! at a time. The borrowed `Log` API stays unchanged by parsing each frame in a
//! short-lived GAME wrapper. Shutdown tails and frame/sibling slices are also
//! available; setup and other whole-log consumers still use `read`. See
//! `docs/lab/2026-09-09-shutdown-streaming.md` and the earlier streaming
//! report for equivalence
//! checks, measured memory, and the supported input boundary.

use std::path::Path;

/// Reads a capture into memory.
///
/// An unreadable path answers with the empty string — the shape every
/// caller here already expects, having checked the file is there, and the
/// same one `std::fs::read_to_string(..).unwrap_or_default()` has.
pub fn read(path: impl AsRef<Path>) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

pub mod indexed;
