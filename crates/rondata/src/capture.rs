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

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::SystemTime;

/// The texts some test in this process holds right now, by path, so that
/// two tests walking one capture at the same time read it once (parked
/// 1138, the suite's second half). A `Weak` keeps nothing alive: the
/// suite's peak memory is what the callers hold, as before, and a text
/// nobody holds is read again. The stamp is the file's length and
/// modification time, because a capture is renamed over an archive name
/// while the suite runs (the module's header).
#[allow(clippy::type_complexity)]
static SHARED: Mutex<Option<HashMap<PathBuf, (u64, Option<SystemTime>, Weak<String>)>>> =
    Mutex::new(None);

/// Reads and shares, counted: `RON_READ_STATS=<file>` appends `reads
/// shared` after each read, so a suite's sharing can be measured after it
/// ran (the twenty-sixth pass's measure of parked 1138). Off, it costs two
/// atomic adds.
static READS: AtomicUsize = AtomicUsize::new(0);
static SHARES: AtomicUsize = AtomicUsize::new(0);

fn counted(shared: bool) {
    let reads = READS.fetch_add(1, Ordering::Relaxed) + 1;
    let shares = SHARES.fetch_add(usize::from(shared), Ordering::Relaxed) + usize::from(shared);
    let Ok(stats) = std::env::var("RON_READ_STATS") else {
        return;
    };
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(stats)
    {
        use std::io::Write;
        let _ = writeln!(f, "{reads} {shares}");
    }
}

/// A capture's text, shared: the `String` the file was read into, behind an
/// `Arc`, and never copied — `Arc<str>` would copy it once more on the way
/// in, and the gate measured that copy as 2.6 GiB on the suite's peak.
#[derive(Clone, Debug)]
pub struct Text(Arc<String>);

impl std::ops::Deref for Text {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl Text {
    /// Whether two handles hold one allocation.
    pub fn same(a: &Text, b: &Text) -> bool {
        Arc::ptr_eq(&a.0, &b.0)
    }
}

/// Reads a capture into memory, or shares the copy another caller holds.
///
/// An unreadable path answers with the empty string — the shape every
/// caller here already expects, having checked the file is there, and the
/// same one `std::fs::read_to_string(..).unwrap_or_default()` has.
pub fn read(path: impl AsRef<Path>) -> Text {
    let path = path.as_ref();
    let stamp = std::fs::metadata(path)
        .ok()
        .map(|m| (m.len(), m.modified().ok()));
    if let Some((len, modified)) = stamp {
        let mut shared = SHARED.lock().unwrap_or_else(|e| e.into_inner());
        let map = shared.get_or_insert_with(HashMap::new);
        let held = map
            .get(path)
            .filter(|(l, m, _)| *l == len && *m == modified)
            .and_then(|(_, _, weak)| weak.upgrade());
        if let Some(text) = held {
            counted(true);
            return Text(text);
        }
        drop(shared);
        counted(false);
        let text = Arc::new(std::fs::read_to_string(path).unwrap_or_default());
        let mut shared = SHARED.lock().unwrap_or_else(|e| e.into_inner());
        let map = shared.get_or_insert_with(HashMap::new);
        map.retain(|_, (_, _, w)| w.strong_count() > 0);
        map.insert(path.to_path_buf(), (len, modified, Arc::downgrade(&text)));
        return Text(text);
    }
    Text(Arc::new(String::new()))
}

/// How many of the texts this process holds are shared right now: a
/// measure for the suite, not a rule.
pub fn shared_now() -> usize {
    let shared = SHARED.lock().unwrap_or_else(|e| e.into_inner());
    shared.as_ref().map_or(0, |m| {
        m.values().filter(|(_, _, w)| w.strong_count() > 0).count()
    })
}

pub mod indexed;

#[cfg(test)]
mod tests {
    use super::{Text, read};

    /// Two holders of one capture share one text; a capture rewritten
    /// under the suite (a different length) is read afresh; a text nobody
    /// holds is read again rather than kept (parked 1138).
    #[test]
    fn a_held_text_is_shared_and_a_rewritten_one_is_not() {
        let dir = std::env::temp_dir().join(format!("capture-share-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("gamelog.txt");
        std::fs::write(&path, "BEGIN FRAME 1\n").unwrap();
        let first = read(&path);
        let second = read(&path);
        assert!(Text::same(&first, &second), "a held text is shared");
        std::fs::write(&path, "BEGIN FRAME 1\nBEGIN FRAME 2\n").unwrap();
        let third = read(&path);
        assert!(
            !Text::same(&first, &third),
            "a rewritten capture is read afresh"
        );
        assert_eq!(&*third, "BEGIN FRAME 1\nBEGIN FRAME 2\n");
        drop((first, second, third));
        let later = read(&path);
        assert_eq!(super::shared_now(), 1, "only what is held is counted");
        drop(later);
        assert_eq!(super::shared_now(), 0);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// Experimental opt-in archive writer; indexed reads accept explicit `.rcap` paths.
pub mod archive;
mod source;
