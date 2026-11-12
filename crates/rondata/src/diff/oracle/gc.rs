//! The decoded-oracle cache's collector. It sits apart from `oracle.rs`
//! because that file is a schema source: an edit there turns every
//! generation over, and what the collector removes is no part of any
//! entry's value.
//!
//! A generation that is not the running build's own is not necessarily
//! stale: a lane or builder whose branch edits a schema source has its own
//! generation and reads and writes it while it works. Each process marks a
//! generation the first time it reads or writes there ([`mark`]), and
//! [`gc`] keeps a foreign generation used within [`IN_USE`]. The pass of
//! 2026-11-12 measured the cost of not doing so: a green gate with four
//! workers out removed one or two live generations (about 1.5 GiB and 90k
//! files each) that their owners then rebuilt cold, and twice raced such a
//! writer into `Directory not empty`, which aborted the whole walk.
use super::{CACHE, civilian_generation, generation};
use std::path::Path;
use std::time::{Duration, SystemTime};

/// The file a process touches in a generation directory the first time it
/// reads or writes an entry there.
pub const MARK: &str = ".in-use";

/// How long a foreign generation is kept after its last use: the newer of
/// its [`MARK`]'s mtime and its directory's (an old-layout generation has no
/// mark; a write of a new entry moves the directory's). Pinned by the
/// steering pass of 2026-11-12: a lane lives under three hours, a live
/// generation costs 1.5 GiB, and the disk read 250 GB free — so twelve
/// hours never sweeps a working lane and over-keeps a few GiB at most.
pub const IN_USE: Duration = Duration::from_secs(12 * 60 * 60);

/// What a sweep did.
#[derive(Debug, Default)]
pub struct Swept {
    /// The caches found.
    pub caches: usize,
    /// The stale generations removed (the flat layout's files count as one).
    pub removed: usize,
    /// The foreign generations kept because they were used within [`IN_USE`].
    pub in_use: usize,
    /// Each removal or read that failed, as `path: error`; the sweep went on.
    pub failed: Vec<String>,
}

impl Swept {
    fn absorb(&mut self, other: Swept) {
        self.caches += other.caches;
        self.removed += other.removed;
        self.in_use += other.in_use;
        self.failed.extend(other.failed);
    }
}

/// Marks `generation_dir` as in use, once per process per directory: the
/// set makes every later read of the same generation a lock and a lookup.
/// A failure to mark is ignored — the directory's own mtime still moves on
/// every write, and the cache is a cache.
#[cfg(test)]
pub(super) fn mark(generation_dir: &Path) {
    use std::collections::HashSet;
    use std::sync::{Mutex, OnceLock};
    static MARKED: OnceLock<Mutex<HashSet<std::path::PathBuf>>> = OnceLock::new();
    let mut marked = MARKED
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if marked.insert(generation_dir.to_path_buf()) {
        let _ = touch(&generation_dir.join(MARK));
    }
}

#[cfg(test)]
fn touch(path: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?
        .set_modified(SystemTime::now())
}

/// When `dir` was last used: the newer of its [`MARK`]'s mtime and its own.
fn last_used(dir: &Path) -> Option<SystemTime> {
    let own = std::fs::symlink_metadata(dir).and_then(|m| m.modified());
    let mark = std::fs::metadata(dir.join(MARK)).and_then(|m| m.modified());
    own.ok().max(mark.ok())
}

fn in_use(dir: &Path, now: SystemTime) -> bool {
    last_used(dir).is_some_and(|t| now.duration_since(t).map_or(true, |age| age < IN_USE))
}

/// Removes every stale generation in `dump_dir`'s cache. `dump_dir` is the
/// directory holding a dump (its cache is `dump_dir/.ron-oracle-v1`) or
/// that cache itself; one with no cache has nothing to remove. A
/// generation is a directory; it is kept when its name is [`generation`] or
/// [`civilian_generation`], or when it was used within [`IN_USE`], and
/// removed otherwise. The flat entries and torn temporaries of the layout
/// before generations are together one more stale generation. A symlink,
/// and any file that is neither, are left where they are. A removal that
/// fails is recorded in [`Swept::failed`] and the sweep goes on. It is a
/// cache: whatever it removes, a read regenerates.
pub fn gc(dump_dir: &Path) -> std::io::Result<Swept> {
    gc_with(dump_dir, SystemTime::now(), &mut |p| {
        std::fs::remove_dir_all(p)
    })
}

fn gc_with(
    dump_dir: &Path,
    now: SystemTime,
    remove: &mut dyn FnMut(&Path) -> std::io::Result<()>,
) -> std::io::Result<Swept> {
    let cache = if dump_dir.file_name() == Some(std::ffi::OsStr::new(CACHE)) {
        dump_dir.to_path_buf()
    } else {
        dump_dir.join(CACHE)
    };
    let mut swept = Swept::default();
    let entries = match std::fs::read_dir(&cache) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(swept),
        Err(e) => return Err(e),
    };
    swept.caches = 1;
    let current = [generation(), civilian_generation()];
    let mut flat = false;
    let mut fail = |path: &Path, e: std::io::Error| {
        if e.kind() != std::io::ErrorKind::NotFound {
            swept.failed.push(format!("{}: {e}", path.display()));
        }
    };
    let (mut removed, mut kept) = (0, 0);
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                fail(&cache, e);
                continue;
            }
        };
        let path = entry.path();
        let kind = match entry.file_type() {
            Ok(kind) => kind,
            Err(e) => {
                fail(&path, e);
                continue;
            }
        };
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if kind.is_dir() {
            if current.iter().any(|c| *c == name) {
                continue;
            }
            if in_use(&path, now) {
                kept += 1;
                continue;
            }
            match remove(&path) {
                Ok(()) => removed += 1,
                Err(e) => fail(&path, e),
            }
        } else if kind.is_file()
            && (name.ends_with(".oracle")
                || (name.starts_with(".oracle-") && name.ends_with(".tmp")))
        {
            match std::fs::remove_file(&path) {
                Ok(()) => flat = true,
                Err(e) => fail(&path, e),
            }
        }
    }
    swept.removed = removed + usize::from(flat);
    swept.in_use = kept;
    Ok(swept)
}

/// [`gc`] on every cache under `root`, never through a symlink (a run
/// directory links the install). Only a `root` that cannot be read is an
/// error; a cache or directory below it that fails is recorded in
/// [`Swept::failed`] and the walk goes on.
pub fn gc_tree(root: &Path) -> std::io::Result<Swept> {
    gc_tree_with(root, SystemTime::now(), &mut |p| std::fs::remove_dir_all(p))
}

fn gc_tree_with(
    root: &Path,
    now: SystemTime,
    remove: &mut dyn FnMut(&Path) -> std::io::Result<()>,
) -> std::io::Result<Swept> {
    let mut swept = Swept::default();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if dir.file_name() == Some(std::ffi::OsStr::new(CACHE)) {
            match gc_with(&dir, now, remove) {
                Ok(one) => swept.absorb(one),
                Err(e) => swept.failed.push(format!("{}: {e}", dir.display())),
            }
            continue;
        }
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(e) if dir == root => return Err(e),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => {
                swept.failed.push(format!("{}: {e}", dir.display()));
                continue;
            }
        };
        for entry in entries {
            match entry.and_then(|e| Ok((e.file_type()?, e.path()))) {
                Ok((kind, path)) if kind.is_dir() => stack.push(path),
                Ok(_) => {}
                Err(e) => swept.failed.push(format!("{}: {e}", dir.display())),
            }
        }
    }
    Ok(swept)
}

#[cfg(test)]
use super::{files_under, scratch};

/// Sets `path`'s mtime `hours` into the past.
#[cfg(test)]
fn age(path: &Path, hours: u64) {
    std::fs::File::open(path)
        .unwrap()
        .set_modified(SystemTime::now() - Duration::from_secs(hours * 3600))
        .unwrap();
}

/// A generation directory with one entry, aged past [`IN_USE`] with no mark.
#[cfg(test)]
fn stale(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join("x.oracle"), b"stale").unwrap();
    age(dir, 13);
}

#[test]
fn gc_removes_every_stale_generation_but_the_current() {
    let root = scratch("gc");
    let cache = root.join(CACHE);
    let stale_dir = cache.join("00000000deadbeef");
    let current = cache.join(generation());
    for d in [&stale_dir, &current] {
        std::fs::create_dir_all(d).unwrap();
        std::fs::write(d.join("0123456789abcdef.oracle"), b"entry").unwrap();
        age(d, 13);
    }
    assert_ne!(stale_dir, current);
    let swept = gc(&root).unwrap();
    assert_eq!((swept.caches, swept.removed, swept.in_use), (1, 1, 0));
    assert!(swept.failed.is_empty());
    assert_eq!(
        files_under(&cache),
        [std::path::PathBuf::from(generation()).join("0123456789abcdef.oracle")],
        "a current generation is kept however old"
    );
    assert_eq!(gc(&root).unwrap().removed, 0, "a second gc finds nothing");
    std::fs::remove_dir_all(root).unwrap(); // exclusively this test's authored files
}

#[test]
fn gc_takes_the_flat_layout_as_one_stale_generation_and_keeps_both_current() {
    let root = scratch("gc-flat");
    let cache = root.join(CACHE);
    for current in [generation(), civilian_generation()] {
        std::fs::create_dir_all(cache.join(&current)).unwrap();
        std::fs::write(cache.join(&current).join("a.oracle"), b"kept").unwrap();
        age(&cache.join(&current), 13);
    }
    assert_ne!(generation(), civilian_generation());
    std::fs::write(cache.join("0123456789abcdef.oracle"), b"flat").unwrap();
    std::fs::write(cache.join("fedcba9876543210.oracle"), b"flat").unwrap();
    std::fs::write(cache.join(".oracle-1-2-3.tmp"), b"torn publish").unwrap();
    std::fs::write(cache.join("README"), b"not ours").unwrap();
    std::fs::create_dir_all(cache.join("1111111111111111-2222222222222222")).unwrap();
    age(&cache.join("1111111111111111-2222222222222222"), 13);
    assert_eq!(
        gc(&root).unwrap().removed,
        2,
        "the flat files and one stale directory"
    );
    let mut kept = vec![
        std::path::PathBuf::from("README"),
        std::path::PathBuf::from(generation()).join("a.oracle"),
        std::path::PathBuf::from(civilian_generation()).join("a.oracle"),
    ];
    kept.sort();
    assert_eq!(files_under(&cache), kept);
    // The cache directory itself is accepted too, and a dump without one is empty.
    assert_eq!(gc(&cache).unwrap().removed, 0);
    let absent = gc(&root.join("absent")).unwrap();
    assert_eq!((absent.caches, absent.removed), (0, 0));
    std::fs::remove_dir_all(root).unwrap(); // exclusively this test's authored files
}

#[test]
fn gc_keeps_a_foreign_generation_in_use_by_its_mark_or_its_mtime() {
    let root = scratch("gc-in-use");
    let cache = root.join(CACHE);
    // A lane's generation, read warm: an old directory, a fresh mark.
    let marked = cache.join("aaaaaaaaaaaaaaaa");
    stale(&marked);
    std::fs::write(marked.join(MARK), b"").unwrap();
    age(&marked, 13);
    // An old-layout generation still being written: no mark, a fresh mtime.
    let written = cache.join("bbbbbbbbbbbbbbbb-cccccccccccccccc");
    std::fs::create_dir_all(&written).unwrap();
    std::fs::write(written.join("x.oracle"), b"live").unwrap();
    // Both foreign and both old: a mark aged past IN_USE, and none at all.
    let old_mark = cache.join("dddddddddddddddd");
    stale(&old_mark);
    std::fs::write(old_mark.join(MARK), b"").unwrap();
    age(&old_mark.join(MARK), 13);
    age(&old_mark, 13);
    let unmarked = cache.join("eeeeeeeeeeeeeeee");
    stale(&unmarked);
    let swept = gc(&root).unwrap();
    assert_eq!((swept.removed, swept.in_use), (2, 2));
    assert!(swept.failed.is_empty(), "{:?}", swept.failed);
    assert!(marked.join("x.oracle").is_file(), "a fresh mark is kept");
    assert!(written.join("x.oracle").is_file(), "a fresh mtime is kept");
    assert!(!old_mark.exists(), "an old mark is swept");
    assert!(!unmarked.exists(), "an old unmarked generation is swept");
    // The same generations a day later: nothing is in use any more.
    let later = SystemTime::now() + Duration::from_secs(24 * 3600);
    let swept = gc_with(&root, later, &mut |p| std::fs::remove_dir_all(p)).unwrap();
    assert_eq!((swept.removed, swept.in_use), (2, 0));
    std::fs::remove_dir_all(root).unwrap(); // exclusively this test's authored files
}

#[test]
fn mark_touches_once_per_process_and_the_store_marks_what_it_reads() {
    let root = scratch("gc-mark");
    let dir = root.join(CACHE).join("ffffffffffffffff");
    mark(&dir);
    let fresh = std::fs::metadata(dir.join(MARK))
        .unwrap()
        .modified()
        .unwrap();
    assert!(SystemTime::now().duration_since(fresh).unwrap() < IN_USE);
    age(&dir.join(MARK), 13);
    age(&dir, 13);
    mark(&dir);
    let swept = gc(&root).unwrap();
    assert_eq!(
        (swept.removed, swept.in_use),
        (1, 0),
        "a second mark in one process touches nothing"
    );
    // A store's read marks the generation its entry sits in.
    let path = root.join("capture.txt");
    let text = "BEGIN GAME\n BEGIN FRAME 3\n  BEGIN UNITDATA\n   BEGIN OBJECT\n    BEGIN SUBOBJECT\n     flags 1\n     o 1\n     who 0\n";
    std::fs::write(&path, text).unwrap();
    let mut source = super::IndexedCapture::open(&path).unwrap();
    let mut store = super::Store::new(true);
    store.enabled = true;
    store.directory = None;
    store.read(&mut source, 0);
    let generation_dir = root.join(CACHE).join(generation());
    assert!(generation_dir.join(MARK).is_file());
    drop(source);
    std::fs::remove_dir_all(root).unwrap(); // exclusively this test's authored files
}

#[test]
fn a_failed_removal_is_recorded_and_the_sweep_goes_on() {
    let root = scratch("gc-failed");
    let caches = [root.join("a"), root.join("b"), root.join("c")];
    for dump in &caches {
        for g in ["1111111111111111", "2222222222222222"] {
            stale(&dump.join(CACHE).join(g));
        }
    }
    let refused = caches[1].join(CACHE).join("1111111111111111");
    let mut remove = |p: &Path| {
        if p == refused {
            Err(std::io::Error::from_raw_os_error(66)) // ENOTEMPTY, as raced
        } else {
            std::fs::remove_dir_all(p)
        }
    };
    let swept = gc_tree_with(&root, SystemTime::now(), &mut remove).unwrap();
    assert_eq!((swept.caches, swept.removed, swept.in_use), (3, 5, 0));
    assert_eq!(swept.failed.len(), 1, "{:?}", swept.failed);
    assert!(swept.failed[0].starts_with(&refused.display().to_string()));
    assert!(refused.is_dir());
    for dump in &caches {
        assert_eq!(
            files_under(&dump.join(CACHE)),
            if dump == &caches[1] {
                vec![std::path::PathBuf::from("1111111111111111/x.oracle")]
            } else {
                vec![]
            }
        );
    }
    std::fs::remove_dir_all(root).unwrap(); // exclusively this test's authored files
}

#[test]
fn gc_tree_finds_every_cache_and_never_follows_a_link() {
    let root = scratch("gc-tree");
    let stale_in = |dump: &Path| stale(&dump.join(CACHE).join("00000000deadbeef"));
    stale_in(&root.join("ch1/map-14"));
    stale_in(&root.join("logs"));
    let outside = scratch("gc-tree-outside");
    stale_in(&outside);
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside, root.join("ch1/install")).unwrap();
    let swept = gc_tree(&root).unwrap();
    assert_eq!((swept.caches, swept.removed), (2, 2));
    assert!(
        outside
            .join(CACHE)
            .join("00000000deadbeef/x.oracle")
            .is_file()
    );
    let swept = gc_tree(&root).unwrap();
    assert_eq!((swept.caches, swept.removed), (2, 0));
    std::fs::remove_dir_all(root).unwrap(); // exclusively this test's authored files
    std::fs::remove_dir_all(outside).unwrap(); // exclusively this test's authored files
}
