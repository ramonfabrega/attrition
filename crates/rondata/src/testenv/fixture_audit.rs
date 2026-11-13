//! Opt-in fixture-request evidence for the local release gate. No game contents.
use std::io::{self, Write};
use std::path::Path;
use std::sync::Mutex;

static WRITER: Mutex<()> = Mutex::new(());

fn hex(text: &str) -> String {
    text.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}

fn append(path: &Path, name: &str, present: bool, test: &str) -> io::Result<()> {
    let line = format!("{}\t{}\t{}\n", u8::from(present), hex(name), hex(test));
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    file.write_all(line.as_bytes())
}

/// One fixture request on a tape: the cache whose fill made it (`0` for a
/// request made outside any fill), the fixture, and whether it was there.
type Request = (usize, String, bool);

thread_local! {
    /// The tapes open on this thread, innermost last, each with the cache
    /// it is filling: a fill of a [`Shared`] cache records what it
    /// requested, and a fill inside a fill is on both tapes under the
    /// inner cache's name.
    static TAPES: std::cell::RefCell<Vec<(usize, Vec<Request>)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Which reader has been credited with which cache's requests, by the
/// cache's address: once a process each.
static CREDITED: Mutex<std::collections::BTreeSet<(usize, String)>> =
    Mutex::new(std::collections::BTreeSet::new());

fn credit(cache: usize, reader: &str) -> bool {
    CREDITED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert((cache, reader.to_owned()))
}

fn onto_tapes(requests: &[Request]) {
    TAPES.with(|tapes| {
        for (_, tape) in tapes.borrow_mut().iter_mut() {
            tape.extend_from_slice(requests);
        }
    });
}

/// `body`'s value, and every fixture this thread requested while it ran,
/// as the fill of `cache`.
fn taped<T>(cache: usize, body: impl FnOnce() -> T) -> (T, Vec<Request>) {
    struct Open;
    impl Drop for Open {
        fn drop(&mut self) {
            // An unwinding fill leaves no tape behind it.
            TAPES.with(|tapes| tapes.borrow_mut().pop());
        }
    }
    TAPES.with(|tapes| tapes.borrow_mut().push((cache, Vec::new())));
    let open = Open;
    let value = body();
    let (_, tape) =
        TAPES.with(|tapes| tapes.borrow_mut().pop().expect("the tape this fill opened"));
    std::mem::forget(open);
    (value, tape)
}

/// The fixture requests one stand-up made, kept beside the state it
/// built: a test that resumes that state is credited with them
/// ([`replay`]), as a test that reads a [`Shared`] cache is with its fill's.
#[derive(Clone, Debug, Default)]
pub(crate) struct Tape(Vec<Request>);

/// `body`'s value, and the fixtures this thread requested while it ran.
pub(crate) fn taped_requests<T>(body: impl FnOnce() -> T) -> (T, Tape) {
    let (value, tape) = taped(0, body);
    (value, Tape(tape))
}

/// Write `tape`'s requests again under this thread's name: every one, each
/// time, since a test that resumes a kept state twice would have stood it
/// up twice alone. A request a [`Shared`] cache's fill made inside the
/// stand-up is written only where this thread does not hold that cache's
/// credit, as reading the cache would.
pub(crate) fn replay(tape: &Tape) {
    let reader = std::thread::current()
        .name()
        .unwrap_or("unnamed")
        .to_owned();
    let mut fresh = std::collections::BTreeMap::new();
    for (made_by, name, present) in &tape.0 {
        let owed = *fresh
            .entry(*made_by)
            .or_insert_with(|| *made_by == 0 || credit(*made_by, &reader));
        if owed {
            write(name, *present);
        }
    }
    onto_tapes(&tape.0);
}

/// Credit the current thread with `tape`, the requests of `cache`'s fill:
/// each request is written under this thread's name unless the thread
/// already holds the credit of the cache that made it, and all of them go
/// onto any tape open here, so an enclosing fill carries them whoever
/// reads it next.
fn credit_reader(cache: usize, tape: &[Request]) {
    let reader = std::thread::current()
        .name()
        .unwrap_or("unnamed")
        .to_owned();
    let mut fresh = std::collections::BTreeMap::new();
    for (made_by, _, _) in tape {
        fresh
            .entry(*made_by)
            .or_insert_with(|| *made_by == 0 || credit(*made_by, &reader));
    }
    // A request made outside any fill belongs to the cache being read.
    let own = credit(cache, &reader) || fresh.get(&cache).copied().unwrap_or(false);
    for (made_by, name, present) in tape {
        let owed = if *made_by == 0 || *made_by == cache {
            own
        } else {
            fresh[made_by]
        };
        if owed {
            write(name, *present);
        }
    }
    onto_tapes(tape);
}

/// **A process-wide cache of something read from fixtures, with every
/// reader credited for the requests the fill made** (item 4216). The
/// release gate ran one process a test until the group unit, so a
/// `OnceLock` over the archive was filled by every test and each test's
/// requests were its own. In a process that runs several tests the first
/// fills it and the rest read it, and the fixture audit — which fixture
/// was asked for by which test, how many times — would lose every row but
/// the first test's. A `Shared` keeps the fill's requests and writes them
/// again, once, under the name of each other thread that reads the cache,
/// so the audit's rows are the rows one process a test wrote. A cache
/// filled inside another's fill is credited once a reader however the
/// reader came to it.
pub(crate) struct Shared<T> {
    cell: std::sync::OnceLock<(T, Vec<Request>)>,
}

impl<T> Shared<T> {
    pub(crate) const fn new() -> Self {
        Self {
            cell: std::sync::OnceLock::new(),
        }
    }

    pub(crate) fn get_or_init(&self, fill: impl FnOnce() -> T) -> &T {
        let cache = std::ptr::from_ref(self) as usize;
        let mut filled = false;
        let (value, tape) = self.cell.get_or_init(|| {
            filled = true;
            taped(cache, fill)
        });
        if filled {
            // The fill's requests are this thread's rows already, and on
            // any tape open around it.
            let reader = std::thread::current()
                .name()
                .unwrap_or("unnamed")
                .to_owned();
            credit(cache, &reader);
        } else {
            credit_reader(cache, tape);
        }
        value
    }
}

pub(super) fn record(name: &str, present: bool) {
    let made_by = TAPES.with(|tapes| tapes.borrow().last().map_or(0, |(cache, _)| *cache));
    onto_tapes(&[(made_by, name.to_owned(), present)]);
    write(name, present);
}

thread_local! {
    /// What this thread would have written to the audit, kept when a test
    /// of this module turns it on: the rows, with no file and no
    /// environment.
    static WRITTEN: std::cell::RefCell<Option<Vec<(String, bool)>>> =
        const { std::cell::RefCell::new(None) };
}

/// Keep this thread's rows in memory from here on (this module's tests).
#[cfg(test)]
fn quiet() {
    WRITTEN.with(|written| {
        written.borrow_mut().get_or_insert_with(Vec::new);
    });
}

fn write(name: &str, present: bool) {
    // A thread of this module's own tests keeps its rows and writes none:
    // their fixtures are authored names, and a row for one in a gate's
    // audit would read as a missing fixture.
    let kept = WRITTEN.with(|written| {
        written
            .borrow_mut()
            .as_mut()
            .map(|rows| rows.push((name.to_owned(), present)))
            .is_some()
    });
    if kept {
        return;
    }
    let Some(directory) = std::env::var_os("RON_FIXTURE_AUDIT_DIR") else {
        return;
    };
    let _guard = WRITER.lock().expect("fixture audit writer poisoned");
    let path = Path::new(&directory).join(format!("{}.tsv", std::process::id()));
    append(
        &path,
        name,
        present,
        std::thread::current().name().unwrap_or("unnamed"),
    )
    .expect("cannot persist fixture-request audit");
}

#[test]
fn audit_rows_escape_names_and_record_both_outcomes() {
    let path = std::env::temp_dir().join(format!(
        "attrition-fixture-audit-{}.tsv",
        std::process::id()
    ));
    std::fs::write(&path, "").unwrap();
    append(&path, "a\tb\n", false, "test").unwrap();
    append(&path, "ok", true, "test").unwrap();
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "0\t6109620a\t74657374\n1\t6f6b\t74657374\n"
    );
    std::fs::remove_file(path).unwrap();
}

#[test]
fn audit_write_errors_are_not_successes() {
    assert!(append(&std::env::temp_dir(), "fixture", false, "test").is_err());
}

#[test]
fn a_shared_cache_credits_each_reader_with_the_fill_s_requests() {
    quiet();
    // What a reader is credited with is what lands on a tape open around
    // its read, so the test reads tapes: it sets no environment and
    // writes no file.
    static CACHE: Shared<u32> = Shared::new();
    let cache = std::ptr::from_ref(&CACHE) as usize;
    let names = |tape: &[Request]| -> Vec<(String, bool)> {
        tape.iter().map(|(_, n, p)| (n.clone(), *p)).collect()
    };
    let fill = || {
        record("a", true);
        record("b", false);
        7
    };
    let (value, tape) = taped(0, || *CACHE.get_or_init(fill));
    assert_eq!(value, 7);
    assert_eq!(
        tape,
        [
            (cache, "a".to_owned(), true),
            (cache, "b".to_owned(), false)
        ]
    );
    // The same thread again: credited already, and an enclosing fill is
    // still handed the requests.
    let (_, again) = taped(0, || *CACHE.get_or_init(|| unreachable!()));
    assert_eq!(names(&again), names(&tape));
    // Another test's thread reads the cache: the credit is taken once.
    let other = std::thread::Builder::new()
        .name("another_test".into())
        .spawn(move || {
            quiet();
            let before = credit(cache, "another_test_probe");
            let first = taped(0, || *CACHE.get_or_init(|| unreachable!())).1;
            let held = !credit(cache, "another_test");
            (before, first, held)
        })
        .unwrap()
        .join()
        .unwrap();
    assert!(other.0, "a name that never read holds no credit");
    assert_eq!(names(&other.1), names(&tape));
    assert!(other.2, "the reading thread holds the cache's credit");
}

#[test]
fn a_cache_filled_inside_another_is_credited_once_a_reader() {
    quiet();
    static INNER: Shared<u32> = Shared::new();
    static OUTER: Shared<u32> = Shared::new();
    let inner = std::ptr::from_ref(&INNER) as usize;
    let outer = std::ptr::from_ref(&OUTER) as usize;
    // One test fills the outer cache, and the inner inside it.
    std::thread::Builder::new()
        .name("fills_both".into())
        .spawn(|| {
            quiet();
            OUTER.get_or_init(|| {
                record("outer", true);
                *INNER.get_or_init(|| {
                    record("inner", true);
                    1
                })
            });
        })
        .unwrap()
        .join()
        .unwrap();
    let outer_tape = &OUTER.cell.get().unwrap().1;
    assert_eq!(
        outer_tape,
        &[
            (outer, "outer".to_owned(), true),
            (inner, "inner".to_owned(), true)
        ]
    );
    // A second test reads the inner cache and then the outer: alone it
    // would have asked for `inner` once, in the inner fill, so the outer's
    // credit must not write it again. Seen red on a tape that did not say
    // which cache made a request: the rows read inner, outer, inner.
    let rows = std::thread::Builder::new()
        .name("reads_inner_then_outer".into())
        .spawn(|| {
            quiet();
            WRITTEN.with(|written| written.borrow_mut().as_mut().unwrap().clear());
            INNER.get_or_init(|| unreachable!());
            OUTER.get_or_init(|| unreachable!());
            OUTER.get_or_init(|| unreachable!());
            WRITTEN.with(|written| written.borrow_mut().take().unwrap())
        })
        .unwrap()
        .join()
        .unwrap();
    assert_eq!(
        rows,
        [("inner".to_owned(), true), ("outer".to_owned(), true)]
    );
    // A third reads the outer alone: both are its rows, once.
    let rows = std::thread::Builder::new()
        .name("reads_outer".into())
        .spawn(|| {
            quiet();
            WRITTEN.with(|written| written.borrow_mut().as_mut().unwrap().clear());
            OUTER.get_or_init(|| unreachable!());
            INNER.get_or_init(|| unreachable!());
            WRITTEN.with(|written| written.borrow_mut().take().unwrap())
        })
        .unwrap()
        .join()
        .unwrap();
    assert_eq!(
        rows,
        [("outer".to_owned(), true), ("inner".to_owned(), true)]
    );
}

#[test]
fn a_resumed_stand_up_s_requests_are_written_each_time() {
    quiet();
    static INSIDE: Shared<u32> = Shared::new();
    let rows = std::thread::Builder::new()
        .name("stands_up_and_resumes".into())
        .spawn(|| {
            quiet();
            let ((), tape) = taped_requests(|| {
                record("start", true);
                INSIDE.get_or_init(|| {
                    record("cached", true);
                    1
                });
                record("trace", false);
            });
            WRITTEN.with(|written| written.borrow_mut().as_mut().unwrap().clear());
            replay(&tape);
            replay(&tape);
            (
                tape,
                WRITTEN.with(|written| written.borrow_mut().take().unwrap()),
            )
        })
        .unwrap()
        .join()
        .unwrap();
    // The stand-up's own requests twice; the cache's once a process a
    // reader, and this reader filled it.
    let own = [("start".to_owned(), true), ("trace".to_owned(), false)];
    assert_eq!(rows.1, [own.clone(), own].concat());
    // Another test resumes the same state: the cache's request is its row
    // once, the stand-up's each time.
    let tape = rows.0;
    let other = std::thread::Builder::new()
        .name("resumes_twice".into())
        .spawn(move || {
            quiet();
            WRITTEN.with(|written| written.borrow_mut().as_mut().unwrap().clear());
            replay(&tape);
            replay(&tape);
            WRITTEN.with(|written| written.borrow_mut().take().unwrap())
        })
        .unwrap()
        .join()
        .unwrap();
    let names: Vec<&str> = other.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["start", "cached", "trace", "start", "trace"]);
}

#[test]
fn a_fill_inside_a_fill_is_on_both_tapes_and_a_panic_closes_its_tape() {
    quiet();
    let (inner, outer) = taped(1, || {
        record("outer", true);
        taped(2, || record("inner", true)).1
    });
    assert_eq!(inner, [(2, "inner".to_owned(), true)]);
    assert_eq!(
        outer,
        [(1, "outer".to_owned(), true), (2, "inner".to_owned(), true)]
    );
    let panicked = std::panic::catch_unwind(|| taped(3, || panic!("a fill that fails")));
    assert!(panicked.is_err());
    assert_eq!(TAPES.with(|tapes| tapes.borrow().len()), 0);
}
