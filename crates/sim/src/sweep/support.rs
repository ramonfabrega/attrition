//! The sweeps' one door to the emulator, `emu_rows`, and its cache: a run
//! is keyed on the SHA-256 of every input that decides its rows — the
//! executable, the script, the `tools/emu/` modules it can import — and the
//! argument list, so a hit skips `uv` and a changed byte anywhere is a
//! miss. Each cache file carries its receipt (`docs/EMULATOR.md` §1.1;
//! `tools/emu_receipts.py` lists and verifies them). A result over the cap
//! is stored gzip-compressed, `<key>.rows.gz`, the same text inside.

use std::io::{self, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::process::Output;

/// What one run of a sweep script depends on, and what to do with the cache.
pub(crate) struct EmuSetup<'a> {
    /// `tools/emu/`, where the script and its shared modules live.
    pub(crate) emu_dir: &'a Path,
    /// The install's `riseofnations.exe`.
    pub(crate) exe: &'a Path,
    /// The arguments after the executable.
    pub(crate) args: &'a [String],
    /// The cache directory, or `None` when `RON_EMU_CACHE=off`.
    pub(crate) cache: Option<PathBuf>,
    /// `RON_VERIFY_EMU`: run the script even on a hit and hold the two.
    pub(crate) verify: bool,
    /// Ask `uv` its version and its Python's for the receipt (off in the
    /// unit tests, whose runner is not `uv`).
    pub(crate) probe: bool,
    /// The most bytes of rows a result may have and still be stored plain
    /// (`RON_EMU_CACHE_MAX_BYTES`; [`DEFAULT_CACHE_CAP`]); a result over it
    /// is stored gzip-compressed as `<key>.rows.gz`, and either is a hit
    /// under any cap. The forty-fifth pass found 136 of 1,523 entries
    /// holding 20.3 of the cache's 21.7 GiB raw after one gate, the largest
    /// 1.3 GB of rows, and stopped storing them: the script ran each time.
    /// Loop 3777 measured the price — 273 over-cap sweeps re-ran `uv` and
    /// unicorn on every gate, 2,486 of the sim crate's 2,496 test-seconds
    /// at four threads (98 % of its lane-gate minutes) — and the rows are
    /// hex text that gzip packs about five to one.
    pub(crate) cap: usize,
    /// How many over-cap results may be live at once (`RON_EMU_BIG_ROWS`;
    /// [`DEFAULT_BIG_ROWS`]; 0 is no bound), counted on `big`.
    pub(crate) big_rows: usize,
    /// The count `big_rows` bounds: [`BIG_ROWS`], process-wide.
    pub(crate) big: &'static BigRows,
}

/// Two over-cap results live at once. The sixty-third pass's whole gate
/// (eight threads, the cache warm) was killed on `sweep_u_z` (137): with
/// every giant a `.rows.gz` hit, `unit_detect_boat_collision`'s 1.7 GB of
/// rows and the 1.1–1.4 GB entries were all held in the same seconds, where
/// unicorn's 40–130 s runs had staggered them. The bound is a count, not
/// bytes: the place is taken before the rows exist.
pub(crate) const DEFAULT_BIG_ROWS: usize = 2;

/// A count of the over-cap results live in the process, and the threads
/// waiting for a place.
///
/// Measured on `sweep_u_z` alone as the isolated gate runs it (eight
/// threads, `MallocLargeCache=0`, peaks sampled every 2 s; Loop 3777's
/// build). Warm and not verifying: bound 2 ran 30 s at a 1.8–2.1 GB peak,
/// with no bound it was killed at 4,159 MiB. Under `RON_VERIFY_EMU` (every
/// whole gate), a re-run takes its place when it ends: 174 s at 3.9 GB,
/// where taking the place before it starts serialized the giants' unicorn
/// minutes, 577 s at 1.9 GB — the pass chose the first, and widened the
/// sweep targets' reservation to match (`tools/isolated_tests.py`).
pub(crate) struct BigRows {
    held: std::sync::Mutex<usize>,
    freed: std::sync::Condvar,
}

impl BigRows {
    pub(crate) const fn new() -> Self {
        BigRows {
            held: std::sync::Mutex::new(0),
            freed: std::sync::Condvar::new(),
        }
    }

    /// Takes one of `limit` places (0: no bound) for the rest of this
    /// thread's life, waiting for one if all are held. The place is the
    /// thread's, not the call's: a sweep holds its rows for its whole test
    /// body, and libtest runs each test on a thread of its own, so the place
    /// goes back when the test is done — after its rows are dropped — and a
    /// thread that holds one asks again without waiting (a test that reads
    /// two giants holds one place, as a thread reused for another test
    /// would). The callers keep `Option<String>`, the 1,400 sweeps
    /// unchanged.
    fn hold(&'static self, limit: usize) {
        if limit == 0 || PLACE.with(|p| p.borrow().is_some()) {
            return;
        }
        let mut held = self.held.lock().unwrap_or_else(|p| p.into_inner());
        while *held >= limit {
            held = self.freed.wait(held).unwrap_or_else(|p| p.into_inner());
        }
        *held += 1;
        drop(held);
        PLACE.with(|p| *p.borrow_mut() = Some(Place(self)));
    }
}

/// A held place, given back when its thread ends.
struct Place(&'static BigRows);

impl Drop for Place {
    fn drop(&mut self) {
        let mut held = self.0.held.lock().unwrap_or_else(|p| p.into_inner());
        *held = held.saturating_sub(1);
        self.0.freed.notify_all();
    }
}

thread_local! {
    static PLACE: std::cell::RefCell<Option<Place>> = const { std::cell::RefCell::new(None) };
}

/// The process's one count of live over-cap results.
pub(crate) static BIG_ROWS: BigRows = BigRows::new();

/// 16 MiB: keeps 1,387 of 1,523 entries plain at 1.35 GiB (measured
/// 2026-10-19); the rest are stored compressed.
pub(crate) const DEFAULT_CACHE_CAP: usize = 16 * 1024 * 1024;

/// The rows `tools/emu/<script>` prints for the install's executable, or
/// `None` (said on stderr) where the install or `uv` is absent. Cached, and
/// verified under `RON_VERIFY_EMU`: see `emu_rows_with`.
pub(crate) fn emu_rows(script: &str) -> Option<String> {
    emu_rows_args(script, &[])
}

/// The rows `tools/emu/sweep_obj.py` prints for `snake`'s row of
/// `tools/emu/sweep_obj.tsv` (item 3731; `<snake>\t<literal>[\t<kept line>]`):
/// the `obj with windows` sweeps' one driver, handed the row's literal so the cache's key, which holds the
/// arguments, follows the row and nothing else's. A name with no row panics:
/// a test whose row is gone tests nothing.
pub(crate) fn emu_obj(snake: &str) -> Option<String> {
    let table = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/emu/sweep_obj.tsv");
    let text = std::fs::read_to_string(table).expect("tools/emu/sweep_obj.tsv");
    let literal = text
        .lines()
        .find_map(|l| l.strip_prefix(snake)?.strip_prefix('\t'))
        .map(|rest| rest.split('\t').next().unwrap_or(rest))
        .unwrap_or_else(|| panic!("no row `{snake}` in tools/emu/sweep_obj.tsv"));
    emu_rows_args("sweep_obj.py", &[snake.to_string(), literal.to_string()])
}

/// `emu_rows` with the arguments after the executable.
fn emu_rows_args(script: &str, args: &[String]) -> Option<String> {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let Some(install) = crate::testenv::install_root() else {
        eprintln!("skipping: set RON_INSTALL (the sweep runs the install's executable)");
        return None;
    };
    let emu_dir = PathBuf::from(format!("{root}/tools/emu"));
    let exe = PathBuf::from(format!("{install}/riseofnations.exe"));
    let get = |k: &str| std::env::var(k).ok();
    let setup = EmuSetup {
        emu_dir: &emu_dir,
        exe: &exe,
        args,
        cache: cache_dir(get, Path::new(root)),
        verify: get("RON_VERIFY_EMU").is_some_and(|v| !v.is_empty() && v != "0"),
        probe: true,
        cap: get("RON_EMU_CACHE_MAX_BYTES")
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(DEFAULT_CACHE_CAP),
        big_rows: get("RON_EMU_BIG_ROWS")
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(DEFAULT_BIG_ROWS),
        big: &BIG_ROWS,
    };
    emu_rows_with(script, &setup, run_uv)
}

/// The real runner: `uv_command(<script>) <exe> <args…>`.
fn run_uv(script: &Path, exe: &Path, args: &[String]) -> io::Result<Output> {
    uv_command(script).arg(exe).args(args).output()
}

/// `uv run --locked --project <emu> python <script>`, `<emu>` the script's
/// directory (item 3354). A bare `uv run <script>` builds the script's PEP
/// 723 environment under `~/.cache/uv/environments-v2`, keyed on the
/// script's path — one a sweep a worktree, 358,890 of them on 2026-10-20 —
/// where `tools/emu/pyproject.toml`'s project is one `tools/emu/.venv` a
/// worktree, made by `uv` on the first run (gitignored; `--locked` holds it
/// to the committed `uv.lock`). Running `python <script>` is what keeps uv
/// from reading the script's inline metadata, which stays for a person's
/// bare `uv run`.
pub(crate) fn uv_command(script: &Path) -> std::process::Command {
    let emu = script.parent().unwrap_or(Path::new("."));
    let mut c = std::process::Command::new("uv");
    c.args(["run", "--locked", "--project"])
        .arg(emu)
        .arg("python")
        .arg(script);
    c
}

/// The cache directory the environment names: `None` for
/// `RON_EMU_CACHE=off`, else `RON_EMU_CACHE_DIR`, else
/// `$HOME/Library/Caches/attrition/emu-cache` — shared by every worktree,
/// since the key carries no path and a lane's gate runs the same scripts
/// on the same executable (the forty-fifth pass) — else, with no `HOME`,
/// `<root>/target/emu-cache`.
pub(crate) fn cache_dir(get: impl Fn(&str) -> Option<String>, root: &Path) -> Option<PathBuf> {
    if get("RON_EMU_CACHE").is_some_and(|v| v.eq_ignore_ascii_case("off")) {
        return None;
    }
    Some(
        get("RON_EMU_CACHE_DIR")
            .filter(|d| !d.is_empty())
            .map_or_else(
                || {
                    get("HOME").filter(|h| !h.is_empty()).map_or_else(
                        || root.join("target/emu-cache"),
                        |h| PathBuf::from(h).join("Library/Caches/attrition/emu-cache"),
                    )
                },
                PathBuf::from,
            ),
    )
}

/// The first line of every cache file, and of the key's own text.
const MAGIC: &str = "emu-cache v1";

/// `emu_rows` with its runner injected (`run(script, exe, args)`). With a
/// cache: a hit — `<key>.rows`, or `<key>.rows.gz` — returns the cached
/// rows and skips the runner (unless `verify`, which runs it and panics on
/// the first line the two part); a miss runs and writes `<key>.rows`, its
/// receipt header then the rows, or the same text gzip-compressed as
/// `<key>.rows.gz` when the rows are over `setup.cap`. An entry that cannot
/// be read whole (a cut or corrupt gzip stream, another key's header) is a
/// miss, and the run's result replaces it.
pub(crate) fn emu_rows_with(
    script: &str,
    setup: &EmuSetup,
    run: impl Fn(&Path, &Path, &[String]) -> io::Result<Output>,
) -> Option<String> {
    let fresh = || match run(&setup.emu_dir.join(script), setup.exe, setup.args) {
        Ok(o) if o.status.success() => Some(String::from_utf8(o.stdout).expect("utf-8 rows")),
        Ok(o) => panic!("{script} failed: {}", String::from_utf8_lossy(&o.stderr)),
        Err(e) => {
            eprintln!("skipping: uv not runnable ({e})");
            None
        }
    };
    let Some(cache) = &setup.cache else {
        return fresh();
    };
    let (key, receipt) = match cache_key(script, setup) {
        Ok(k) => k,
        Err(e) => {
            eprintln!("emu cache: cannot key {script} ({e}); running it uncached");
            return fresh();
        }
    };
    let plain = cache.join(format!("{key}.rows"));
    let packed = cache.join(format!("{key}.rows.gz"));
    // A fresh run's size is known only when it ends, so a fresh giant takes
    // its place then: holding one for every run would serialize a cold
    // gate's 1,400 sweeps, and the emulator's own minutes stagger the
    // giants. Then it is stored, compressed when over the cap.
    let keep = |rows: String| {
        let over = rows.len() > setup.cap;
        if over {
            setup.big.hold(setup.big_rows);
        }
        let file = if over { &packed } else { &plain };
        match store(
            cache,
            file,
            &key,
            &receipt,
            &rows,
            setup.probe.then_some(setup.emu_dir),
            over,
        ) {
            Ok(size) if over => eprintln!(
                "emu cache: {script}'s rows ({} bytes) are over the cap ({}); stored compressed \
                 ({size} bytes)",
                rows.len(),
                setup.cap
            ),
            Ok(_) => {}
            Err(e) => eprintln!("emu cache: cannot write {} ({e})", file.display()),
        }
        rows
    };
    let unreadable = |file: &Path, e: io::Error| {
        if e.kind() != io::ErrorKind::NotFound {
            eprintln!("emu cache: {} is unreadable ({e}); a miss", file.display());
        }
    };
    if setup.verify {
        // The whole gate. The cached copy is streamed against the fresh
        // rows, never held whole beside them (two copies of every giant
        // killed the sixty-third pass's whole gate on `sweep_u_z` at 4,110
        // MiB). A re-run takes its place when it ends, as a miss does, not
        // before it starts: see [`BigRows`].
        let Some(file) = [&plain, &packed].into_iter().find(|f| f.is_file()) else {
            return Some(keep(fresh()?));
        };
        let now = fresh()?;
        match same_as(file, &key, &now) {
            Ok(Some(None)) => {
                if now.len() > setup.cap {
                    setup.big.hold(setup.big_rows);
                }
                return Some(now);
            }
            Ok(Some(Some((n, was, is)))) => panic!(
                "RON_VERIFY_EMU: {script}'s cached rows ({}) part from a fresh run at line {n}: \
                 cached `{was}`, fresh `{is}`",
                file.display()
            ),
            Ok(None) => {}
            Err(e) => unreadable(file, e),
        }
        return Some(keep(now));
    }
    // An entry over the cap takes a place before its rows are read: a plain
    // one written under a larger cap by its length, a compressed one by its
    // trailer's.
    let hold = |size: u64| {
        if usize::try_from(size).map_or(true, |n| n > setup.cap) {
            setup.big.hold(setup.big_rows);
        }
    };
    let hit = std::fs::metadata(&plain)
        .ok()
        .and_then(|m| {
            hold(m.len());
            std::fs::read_to_string(&plain).ok()
        })
        .or_else(|| {
            read_gz(&packed, hold)
                .map_err(|e| unreadable(&packed, e))
                .ok()
        })
        .and_then(|text| rows_of(text, &key));
    if let Some(cached) = hit {
        return Some(cached);
    }
    Some(keep(fresh()?))
}

/// Whether a cache file of `key` holds exactly `now`, read as a stream:
/// `None` when it is not `key`'s file, else the first line the two part on
/// — its number from 1 and each side's line, `<end>` past a side's last —
/// or `None` when none does. An error — a gzip stream cut short or
/// corrupt, not UTF-8 — is the reader's.
fn same_as(
    file: &Path,
    key: &str,
    now: &str,
) -> io::Result<Option<Option<(usize, String, String)>>> {
    let f = std::fs::File::open(file)?;
    let mut r: Box<dyn io::BufRead> = if file.extension().is_some_and(|e| e == "gz") {
        Box::new(io::BufReader::new(flate2::read::GzDecoder::new(f)))
    } else {
        Box::new(io::BufReader::new(f))
    };
    let mut line = String::new();
    let mut next = |line: &mut String| -> io::Result<bool> {
        line.clear();
        let got = r.read_line(line)?;
        let ended = line.ends_with('\n');
        if ended {
            line.pop();
        }
        Ok(got > 0 && ended)
    };
    // The header, as `strip_header` reads it.
    if !next(&mut line)? || line != format!("# {MAGIC}") {
        return Ok(None);
    }
    let mut keyed = false;
    loop {
        if !next(&mut line)? || !line.starts_with('#') {
            return Ok(None);
        }
        if line == "# end" {
            break;
        }
        if let Some(k) = line.strip_prefix("# key ") {
            if k != key {
                return Ok(None);
            }
            keyed = true;
        }
    }
    if !keyed {
        return Ok(None);
    }
    // The rows, piece by piece as `str::split('\n')` cuts them: a piece the
    // reader ends without a newline is the last, and an empty one follows a
    // final newline.
    let mut fresh = now.split('\n');
    let mut n = 0;
    let mut done = false;
    loop {
        n += 1;
        let was = if done {
            None
        } else {
            done = !next(&mut line)?;
            Some(line.as_str())
        };
        let is = fresh.next();
        match (was, is) {
            (None, None) => return Ok(Some(None)),
            (p, q) if p == q => {}
            (p, q) => {
                let side = |l: Option<&str>| l.unwrap_or("<end>").to_string();
                return Ok(Some(Some((n, side(p), side(q)))));
            }
        }
    }
}

/// A cache file's rows, its header removed in place (no second copy of a
/// result that can be a gigabyte and more); `None` when the text is not a
/// cache file of `key`.
fn rows_of(mut text: String, key: &str) -> Option<String> {
    let header = text.len() - strip_header(&text, key)?.len();
    text.drain(..header);
    Some(text)
}

/// The whole text of a gzip-compressed cache file, streamed into one
/// `String` sized from the stream's trailer (its length mod 2^32, a hint
/// held to 64 times the compressed size), so the peak is the text once. An
/// error — not gzip, cut short, a CRC or length that does not match, not
/// UTF-8 — is the reader's, and a miss to the caller. `before` is handed
/// the trailer's length before a byte is decompressed.
fn read_gz(path: &Path, before: impl FnOnce(u64)) -> io::Result<String> {
    let mut f = std::fs::File::open(path)?;
    let len = f.metadata()?.len();
    let size = gz_size(&mut f)?;
    before(size);
    let hint = size.min(len.saturating_mul(64));
    let mut text = String::with_capacity(usize::try_from(hint).unwrap_or(0));
    flate2::read::GzDecoder::new(f).read_to_string(&mut text)?;
    Ok(text)
}

/// A gzip file's trailer length — its text's size mod 2^32, every entry
/// here being under 4 GiB — or 0 for a file too short to have one; the
/// file is left at its start.
fn gz_size(f: &mut std::fs::File) -> io::Result<u64> {
    if f.metadata()?.len() < 18 {
        return Ok(0);
    }
    let mut isize = [0; 4];
    f.seek(io::SeekFrom::End(-4))?;
    f.read_exact(&mut isize)?;
    f.rewind()?;
    Ok(u64::from(u32::from_le_bytes(isize)))
}

/// Writes `file` through a temporary sibling and a rename, so a reader
/// never sees half of it — the header and the rows, gzip-compressed when
/// `compress` — and returns the bytes written.
fn store(
    cache: &Path,
    file: &Path,
    key: &str,
    receipt: &[String],
    rows: &str,
    probe: Option<&Path>,
    compress: bool,
) -> io::Result<u64> {
    std::fs::create_dir_all(cache)?;
    let mut head = format!("# {MAGIC}\n# key {key}\n");
    let mut line = |l: &str| {
        head.push_str("# ");
        head.push_str(l);
        head.push('\n');
    };
    receipt.iter().for_each(|l| line(l));
    line(&format!("date {}", utc_now()));
    if let Some(emu_dir) = probe {
        tool_versions(emu_dir).iter().for_each(|l| line(l));
    }
    let n = rows.lines().filter(|l| !l.trim().is_empty()).count();
    line(&format!("rows {n}"));
    line("end");
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let tmp = cache.join(format!(".{key}.{}.{nanos}.tmp", std::process::id()));
    let written = if compress {
        let gz = || -> io::Result<()> {
            let out = io::BufWriter::new(std::fs::File::create(&tmp)?);
            let mut gz = flate2::write::GzEncoder::new(out, flate2::Compression::default());
            gz.write_all(head.as_bytes())?;
            gz.write_all(rows.as_bytes())?;
            gz.finish()?.flush()
        };
        gz()
    } else {
        std::fs::write(&tmp, format!("{head}{rows}"))
    };
    if let Err(e) = written.and_then(|()| std::fs::rename(&tmp, file)) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(std::fs::metadata(file)?.len())
}

/// `uv --version` and the Python of `emu_dir`'s project, the one
/// [`uv_command`] runs, once a process: the receipt's tool lines.
fn tool_versions(emu_dir: &Path) -> &'static [String] {
    static V: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    V.get_or_init(|| {
        let out = |args: &[&std::ffi::OsStr]| {
            std::process::Command::new("uv")
                .args(args)
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| {
                    let mut t = String::from_utf8_lossy(&o.stdout).into_owned();
                    t.push_str(&String::from_utf8_lossy(&o.stderr));
                    t.trim().to_string()
                })
                .unwrap_or_else(|| "unknown".to_string())
        };
        vec![
            format!(
                "uv {}",
                out(&["--version".as_ref()]).trim_start_matches("uv ")
            ),
            format!(
                "python {} (uv run --project tools/emu)",
                out(&[
                    "run".as_ref(),
                    "--quiet".as_ref(),
                    "--locked".as_ref(),
                    "--project".as_ref(),
                    emu_dir.as_os_str(),
                    "python".as_ref(),
                    "--version".as_ref(),
                ])
                .trim_start_matches("Python ")
            ),
        ]
    })
}

/// Now, UTC, as `YYYY-MM-DDTHH:MM:SSZ` (the civil-from-days conversion,
/// integers only).
fn utc_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let (days, rem) = (i64::try_from(secs / 86_400).unwrap_or(0), secs % 86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3_600,
        rem / 60 % 60,
        rem % 60
    )
}

/// The cache key of a run — hex SHA-256 over the inputs' hashes and the
/// arguments — and the receipt lines naming each input. The key's text is
/// `emu-cache v1`, then `executable <sha>`, `script <sha>`, a `module
/// <name> <sha>` per shared module in name order and an `arg <a>` per
/// argument, each line ended by `\n`. No path enters it, so moving the
/// install or the checkout keeps every entry; `tools/emu_receipts.py`
/// rebuilds the same text.
pub(crate) fn cache_key(script: &str, setup: &EmuSetup) -> io::Result<(String, Vec<String>)> {
    let exe = file_sha(setup.exe)?;
    let own = file_sha(&setup.emu_dir.join(script))?;
    let mut text = format!("{MAGIC}\nexecutable {exe}\nscript {own}\n");
    let mut receipt = vec![
        format!("executable {exe} {}", setup.exe.display()),
        format!("script {own} {script}"),
    ];
    for module in shared_modules(setup.emu_dir, script)? {
        let sha = file_sha(&setup.emu_dir.join(&module))?;
        text.push_str(&format!("module {module} {sha}\n"));
        receipt.push(format!("module {sha} {module}"));
    }
    for arg in setup.args {
        text.push_str(&format!("arg {arg}\n"));
        receipt.push(format!("arg {arg}"));
    }
    Ok((hex(&sha256(text.as_bytes())), receipt))
}

/// The modules of `emu_dir` a script's rows can depend on, by file name:
/// every `*.py` that is not itself a `sweep_*.py` (conservatively — every
/// library a script may reach), and every `sweep_*.py` the script, or any
/// of those, imports, transitively.
fn shared_modules(emu_dir: &Path, script: &str) -> io::Result<Vec<String>> {
    let mut found = std::collections::BTreeSet::new();
    for entry in std::fs::read_dir(emu_dir)? {
        let name = entry?.file_name().to_string_lossy().into_owned();
        if name.ends_with(".py") && !name.starts_with("sweep_") {
            found.insert(name);
        }
    }
    let mut todo: Vec<String> = found.iter().cloned().collect();
    todo.push(script.to_string());
    let mut seen: std::collections::BTreeSet<String> = todo.iter().cloned().collect();
    while let Some(file) = todo.pop() {
        let text = std::fs::read_to_string(emu_dir.join(&file))?;
        for module in imported(&text) {
            let name = format!("{module}.py");
            if module.starts_with("sweep_")
                && emu_dir.join(&name).is_file()
                && seen.insert(name.clone())
            {
                found.insert(name.clone());
                todo.push(name);
            }
        }
    }
    found.remove(script);
    Ok(found.into_iter().collect())
}

/// The top-level module names a Python text's `import` and `from … import`
/// lines name.
fn imported(text: &str) -> Vec<&str> {
    fn top(m: &str) -> &str {
        m.split('.').next().unwrap_or("")
    }
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim_start().split('#').next().unwrap_or("");
        if let Some(rest) = line.strip_prefix("from ") {
            out.extend(rest.split_whitespace().next().map(top));
        } else if let Some(rest) = line.strip_prefix("import ") {
            for part in rest.split(',') {
                out.extend(part.split_whitespace().next().map(top));
            }
        }
    }
    out.retain(|m| !m.is_empty());
    out
}

/// A file's hex SHA-256, remembered for the process by path, length and
/// modification time — the executable is ten megabytes, and every sweep in
/// a test binary keys on it.
fn file_sha(path: &Path) -> io::Result<String> {
    type Memo = std::collections::BTreeMap<(PathBuf, u64, u128), String>;
    static MEMO: std::sync::Mutex<Memo> = std::sync::Mutex::new(Memo::new());
    let meta = std::fs::metadata(path)?;
    let mtime = meta
        .modified()?
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let at = (path.to_path_buf(), meta.len(), mtime);
    if let Some(sha) = MEMO.lock().unwrap_or_else(|p| p.into_inner()).get(&at) {
        return Ok(sha.clone());
    }
    let sha = hex(&sha256(&std::fs::read(path)?));
    MEMO.lock()
        .unwrap_or_else(|p| p.into_inner())
        .insert(at, sha.clone());
    Ok(sha)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The rows of a cache file, its header stripped; `None` when the text is
/// not a cache file of `key`.
pub(crate) fn strip_header<'t>(text: &'t str, key: &str) -> Option<&'t str> {
    let mut rest = text
        .strip_prefix("# ")?
        .strip_prefix(MAGIC)?
        .strip_prefix('\n')?;
    let mut keyed = false;
    loop {
        let (line, after) = rest.split_once('\n')?;
        if !line.starts_with('#') {
            return None;
        }
        if line == "# end" {
            return keyed.then_some(after);
        }
        if let Some(k) = line.strip_prefix("# key ") {
            if k != key {
                return None;
            }
            keyed = true;
        }
        rest = after;
    }
}

/// SHA-256 of `data` (FIPS 180-4).
pub(crate) fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let full = data.len() / 64 * 64;
    let mut tail = data[full..].to_vec();
    tail.push(0x80);
    while tail.len() % 64 != 56 {
        tail.push(0);
    }
    let bits = u64::try_from(data.len())
        .unwrap_or(u64::MAX)
        .wrapping_mul(8);
    tail.extend_from_slice(&bits.to_be_bytes());
    for block in data[..full].chunks_exact(64).chain(tail.chunks_exact(64)) {
        compress(&mut h, block);
    }
    let mut out = [0; 32];
    for (o, w) in out.chunks_exact_mut(4).zip(h) {
        o.copy_from_slice(&w.to_be_bytes());
    }
    out
}

/// SHA-256's round constants: the first 32 bits of the fractional parts of
/// the cube roots of the first 64 primes.
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// One 64-byte block into the running state.
fn compress(h: &mut [u32; 8], block: &[u8]) {
    let mut w = [0u32; 64];
    for (word, bytes) in w.iter_mut().zip(block.chunks_exact(4)) {
        *word = u32::from_be_bytes(bytes.try_into().expect("four bytes"));
    }
    for i in 16..64 {
        let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
        let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
        w[i] = w[i - 16]
            .wrapping_add(s0)
            .wrapping_add(w[i - 7])
            .wrapping_add(s1);
    }
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = *h;
    for (k, wi) in K.iter().zip(w) {
        let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let ch = (e & f) ^ (!e & g);
        let t1 = hh
            .wrapping_add(s1)
            .wrapping_add(ch)
            .wrapping_add(*k)
            .wrapping_add(wi);
        let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let maj = (a & b) ^ (a & c) ^ (b & c);
        let t2 = s0.wrapping_add(maj);
        hh = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }
    for (x, y) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
        *x = x.wrapping_add(y);
    }
}

/// One row, `<args…> -> <result>`, split into its integers.
pub(crate) fn row(line: &str) -> (Vec<i64>, i64) {
    let (lhs, rhs) = line
        .split_once(" -> ")
        .expect("a row is `<args…> -> <result>`");
    let args = lhs
        .split_whitespace()
        .map(|s| s.parse().expect("an integer argument"))
        .collect();
    (args, rhs.trim().parse().expect("an integer result"))
}

/// One row with a tail, `<args…> -> <result> | <tail> | …` (item 2061): its
/// integers and the tail's fields, each trimmed, in order.
#[allow(dead_code)] // Some standalone partitions use only the scalar row helper.
pub(crate) fn tail_row(line: &str) -> (Vec<i64>, i64, Vec<&str>) {
    let (head, tail) = line.split_once(" | ").unwrap_or((line, ""));
    let (args, eax) = row(head);
    let tail = if tail.is_empty() && !line.contains(" | ") {
        Vec::new()
    } else {
        tail.split(" | ").map(str::trim).collect()
    };
    (args, eax, tail)
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_CACHE_CAP, EmuSetup, cache_dir, cache_key, emu_rows_with, sha256, strip_header,
    };
    use std::cell::Cell;
    use std::io;
    use std::os::unix::process::ExitStatusExt;
    use std::path::{Path, PathBuf};
    use std::process::{ExitStatus, Output};
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A scratch directory, removed on drop.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(tag: &str) -> Self {
            static N: AtomicUsize = AtomicUsize::new(0);
            let n = N.fetch_add(1, Ordering::Relaxed);
            let dir = std::env::temp_dir()
                .join(format!("emu-cache-test-{}-{tag}-{n}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).expect("a scratch directory");
            Scratch(dir)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn write(path: &Path, text: &str) {
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("mkdir");
        std::fs::write(path, text).expect("write");
    }

    /// An emulator directory and an executable under `root`: the script
    /// imports one sibling sweep, and one sweep it does not import sits
    /// beside it.
    fn lay_out(root: &Path) -> (PathBuf, PathBuf) {
        let emu = root.join("tools/emu");
        write(
            &emu.join("sweep_a.py"),
            "import sys\nfrom hooks import Harness  # noqa: E402\nfrom sweep_b import run\nrun()\n",
        );
        write(&emu.join("sweep_b.py"), "def run():\n    print('1 -> 2')\n");
        write(&emu.join("sweep_unrelated.py"), "print('unrelated')\n");
        write(&emu.join("hooks.py"), "class Harness: pass\n");
        write(&emu.join("callfn.py"), "STACK_BASE = 0\n");
        write(&emu.join("README.md"), "not a module\n");
        let exe = root.join("install/riseofnations.exe");
        write(&exe, "MZ the executable");
        (emu, exe)
    }

    fn setup<'a>(
        emu: &'a Path,
        exe: &'a Path,
        args: &'a [String],
        cache: Option<PathBuf>,
    ) -> EmuSetup<'a> {
        EmuSetup {
            emu_dir: emu,
            exe,
            args,
            cache,
            verify: false,
            probe: false,
            cap: DEFAULT_CACHE_CAP,
            big_rows: super::DEFAULT_BIG_ROWS,
            // Each test its own count, so one test's giants never wait on
            // another's.
            big: Box::leak(Box::new(super::BigRows::new())),
        }
    }

    impl<'a> EmuSetup<'a> {
        fn clone_for_test(&self) -> EmuSetup<'a> {
            EmuSetup {
                emu_dir: self.emu_dir,
                exe: self.exe,
                args: self.args,
                cache: self.cache.clone(),
                verify: self.verify,
                probe: self.probe,
                cap: self.cap,
                big_rows: self.big_rows,
                big: self.big,
            }
        }
    }

    fn key(emu: &Path, exe: &Path, args: &[String]) -> String {
        cache_key("sweep_a.py", &setup(emu, exe, args, None))
            .expect("a key")
            .0
    }

    fn ok(stdout: &str) -> io::Result<Output> {
        Ok(Output {
            status: ExitStatus::from_raw(0),
            stdout: stdout.as_bytes().to_vec(),
            stderr: Vec::new(),
        })
    }

    fn hex(d: [u8; 32]) -> String {
        super::hex(&d)
    }

    #[test]
    fn sha256_matches_the_standard_vectors() {
        assert_eq!(
            hex(sha256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex(sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            hex(sha256(
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
            )),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        assert_eq!(
            hex(sha256(&[b'a'; 1_000_000])),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }

    #[test]
    fn the_key_moves_with_every_input_and_not_with_a_path() {
        let s = Scratch::new("key");
        let (emu, exe) = lay_out(&s.0);
        let k0 = key(&emu, &exe, &[]);
        // `tools/explore/test_emu_receipts.py` pins the same literal for the
        // same layout: the receipt tool rebuilds this key text.
        assert_eq!(
            k0, "6cd8297036e1e4c4fbe9175973c94410c17929c99222fdb7f81ab42c8fe4d2df",
            "the key text is `tools/emu_receipts.py`'s"
        );
        assert_eq!(
            key(&emu, &exe, &[]),
            k0,
            "the key is a function of the inputs"
        );

        // Moving everything leaves the key where it was.
        let moved = s.0.join("elsewhere");
        let (emu2, exe2) = lay_out(&moved);
        assert_eq!(key(&emu2, &exe2, &[]), k0, "a path is not an input");

        // A byte of any input moves it.
        let touch = |path: &Path, extra: &str| {
            let was = std::fs::read_to_string(path).expect("read");
            write(path, &format!("{was}{extra}"));
            let k = key(&emu2, &exe2, &[]);
            write(path, &was);
            k
        };
        for input in [
            &exe2,
            &emu2.join("sweep_a.py"),
            &emu2.join("hooks.py"),
            &emu2.join("callfn.py"),
            &emu2.join("sweep_b.py"),
        ] {
            assert_ne!(touch(input, "#"), k0, "{} is an input", input.display());
        }
        // A sweep the script does not import, and a file that is not a
        // module, are not inputs.
        assert_eq!(touch(&emu2.join("sweep_unrelated.py"), "#"), k0);
        assert_eq!(touch(&emu2.join("README.md"), "#"), k0);
        // A new shared module is.
        write(&emu2.join("newlib.py"), "X = 1\n");
        assert_ne!(
            key(&emu2, &exe2, &[]),
            k0,
            "a new shared module is an input"
        );
        std::fs::remove_file(emu2.join("newlib.py")).expect("rm");
        // And so are the arguments.
        assert_eq!(key(&emu2, &exe2, &[]), k0);
        assert_ne!(
            key(&emu2, &exe2, &["7".to_string()]),
            k0,
            "an argument is an input"
        );

        // The receipt names each input with its hash.
        let (_, receipt) = cache_key("sweep_a.py", &setup(&emu, &exe, &[], None)).expect("key");
        let text = receipt.join("\n");
        for name in [
            "sweep_a.py",
            "sweep_b.py",
            "hooks.py",
            "callfn.py",
            "riseofnations.exe",
        ] {
            assert!(text.contains(name), "the receipt names {name}: {text}");
        }
        assert!(!text.contains("sweep_unrelated.py"), "{text}");
    }

    /// Item 3354: a bare `uv run <script>` builds a PEP 723 environment per
    /// script *path* under `~/.cache/uv/environments-v2`, so each worktree
    /// built one per sweep. The runner runs the script on `tools/emu`'s
    /// project python instead, and the project's own files leave every
    /// cache key where it was.
    #[test]
    fn the_runner_runs_the_shared_project_and_its_files_key_nothing() {
        let s = Scratch::new("uv");
        let (emu, exe) = lay_out(&s.0);
        let script = emu.join("sweep_a.py");
        let c = super::uv_command(&script);
        assert_eq!(c.get_program(), "uv");
        let args: Vec<_> = c.get_args().collect();
        assert_eq!(
            args,
            [
                "run".as_ref(),
                "--locked".as_ref(),
                "--project".as_ref(),
                emu.as_os_str(),
                "python".as_ref(),
                script.as_os_str(),
            ],
            "uv run --locked --project <tools/emu> python <script>"
        );

        let k0 = key(&emu, &exe, &[]);
        write(&emu.join("pyproject.toml"), "[project]\nname = \"x\"\n");
        write(&emu.join("uv.lock"), "version = 1\n");
        write(&emu.join(".venv/bin/python"), "#!/bin/sh\n");
        assert_eq!(
            key(&emu, &exe, &[]),
            k0,
            "the project's files are not inputs"
        );
    }

    #[test]
    fn a_miss_runs_and_writes_and_a_hit_skips_the_runner() {
        let s = Scratch::new("hit");
        let (emu, exe) = lay_out(&s.0);
        let cache = s.0.join("cache");
        let calls = Cell::new(0);
        let run = |script: &Path, e: &Path, _: &[String]| {
            calls.set(calls.get() + 1);
            assert!(script.ends_with("sweep_a.py"));
            assert_eq!(e, exe.as_path());
            ok("1 2 -> 3\n4 5 -> 9\n\n6 7 -> 13\n")
        };
        let set = setup(&emu, &exe, &[], Some(cache.clone()));
        let first = emu_rows_with("sweep_a.py", &set, run).expect("rows");
        assert_eq!(first, "1 2 -> 3\n4 5 -> 9\n\n6 7 -> 13\n");
        assert_eq!(calls.get(), 1);

        let (k, _) = cache_key("sweep_a.py", &set).expect("key");
        let file = cache.join(format!("{k}.rows"));
        let text = std::fs::read_to_string(&file).expect("the cache file is written");
        assert!(text.starts_with("# emu-cache v1\n"), "{text}");
        assert!(text.contains(&format!("# key {k}\n")), "{text}");
        assert!(text.contains("# rows 3\n"), "three rows: {text}");
        assert!(text.contains("# date "), "{text}");
        assert_eq!(strip_header(&text, &k), Some(first.as_str()));

        let second = emu_rows_with("sweep_a.py", &set, run).expect("rows");
        assert_eq!(second, first, "a hit returns the rows verbatim");
        assert_eq!(calls.get(), 1, "a hit skips the runner");

        // A changed script is a miss.
        write(&emu.join("sweep_a.py"), "print('changed')\n");
        emu_rows_with("sweep_a.py", &set, run).expect("rows");
        assert_eq!(calls.get(), 2, "a changed input runs again");
    }

    /// The text of a gzip file, or the error that stops its reading.
    fn gunzip(path: &Path) -> io::Result<String> {
        use std::io::Read;
        let mut text = String::new();
        flate2::read::GzDecoder::new(std::fs::File::open(path)?).read_to_string(&mut text)?;
        Ok(text)
    }

    /// Loop 3777: 273 sweeps' rows were over the cap and re-ran `uv` and
    /// unicorn on every gate, 98 % of the crate's lane-gate minutes. A
    /// result over the cap is stored gzip-compressed as `<key>.rows.gz`,
    /// the same header and rows inside, and read back on the next call.
    #[test]
    fn a_result_over_the_cap_is_stored_compressed_and_read_back() {
        let s = Scratch::new("cap");
        let (emu, exe) = lay_out(&s.0);
        let cache = s.0.join("cache");
        let calls = Cell::new(0);
        let run = |_: &Path, _: &Path, _: &[String]| {
            calls.set(calls.get() + 1);
            ok("1 2 -> 3\n4 5 -> 9\n")
        };
        let mut set = setup(&emu, &exe, &[], Some(cache.clone()));
        set.cap = 8;
        let rows = emu_rows_with("sweep_a.py", &set, run).expect("rows");
        assert_eq!(
            rows, "1 2 -> 3\n4 5 -> 9\n",
            "the rows come back whatever the cap"
        );
        let (k, receipt) = cache_key("sweep_a.py", &set).expect("key");
        let plain = cache.join(format!("{k}.rows"));
        let packed = cache.join(format!("{k}.rows.gz"));
        assert!(!plain.exists(), "over the cap: no plain entry");
        let text = gunzip(&packed).expect("over the cap: stored compressed");
        assert!(text.starts_with("# emu-cache v1\n"), "{text}");
        for line in &receipt {
            assert!(text.contains(&format!("# {line}\n")), "{line}: {text}");
        }
        assert!(text.contains("# rows 2\n"), "{text}");
        assert_eq!(strip_header(&text, &k), Some(rows.as_str()));

        let again = emu_rows_with("sweep_a.py", &set, run).expect("rows");
        assert_eq!(again, rows, "a compressed hit returns the rows verbatim");
        assert_eq!(calls.get(), 1, "a compressed hit skips the runner");

        // The cap names where an entry is compressed, not whether it is
        // read: a compressed entry is a hit under any cap.
        set.cap = DEFAULT_CACHE_CAP;
        assert_eq!(
            emu_rows_with("sweep_a.py", &set, run).as_deref(),
            Some(rows.as_str())
        );
        assert_eq!(calls.get(), 1, "read under the default cap too");
        assert!(!plain.exists(), "a hit writes nothing");

        // And `RON_VERIFY_EMU` holds a compressed hit to a fresh run.
        set.verify = true;
        emu_rows_with("sweep_a.py", &set, run).expect("rows");
        assert_eq!(calls.get(), 2, "verify runs the script on a compressed hit");
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            emu_rows_with("sweep_a.py", &set, |_: &Path, _: &Path, _: &[String]| {
                ok("1 2 -> 3\n4 5 -> 8\n")
            })
        }));
        let err = caught.expect_err("a mismatch with a compressed hit panics");
        let msg = err.downcast_ref::<String>().cloned().unwrap_or_default();
        assert!(msg.contains("line 2") && msg.contains("rows.gz"), "{msg}");
    }

    /// The pass's whole gate (eight threads, warm cache) was killed on
    /// `sweep_u_z` (137): with every giant a hit, they all held their rows
    /// in the same seconds. With the bound at one, a second thread asking
    /// for an over-cap hit waits until the first thread — its test — is
    /// done; and a thread that holds the place may ask again.
    #[test]
    fn over_cap_results_wait_for_a_place_and_hold_it_for_the_test() {
        use std::sync::atomic::AtomicUsize;
        let s = Scratch::new("big");
        let (emu, exe) = lay_out(&s.0);
        let cache = s.0.join("cache");
        let mut set = setup(&emu, &exe, &[], Some(cache));
        set.cap = 8;
        set.big_rows = 1;
        // Written on a thread of its own: the place it takes goes back
        // when that thread ends.
        std::thread::scope(|scope| {
            scope.spawn(|| {
                emu_rows_with("sweep_a.py", &set, |_: &Path, _: &Path, _: &[String]| {
                    ok("1 2 -> 3\n4 5 -> 9\n")
                })
                .expect("rows");
            });
        });
        let no_run = |_: &Path, _: &Path, _: &[String]| -> io::Result<Output> {
            panic!("a hit runs nothing")
        };
        let (live, peak, done) = (
            AtomicUsize::new(0),
            AtomicUsize::new(0),
            AtomicUsize::new(0),
        );
        std::thread::scope(|scope| {
            for _ in 0..2 {
                scope.spawn(|| {
                    let rows = emu_rows_with("sweep_a.py", &set, no_run).expect("a hit");
                    let now = live.fetch_add(1, Ordering::SeqCst) + 1;
                    peak.fetch_max(now, Ordering::SeqCst);
                    std::thread::sleep(std::time::Duration::from_millis(150));
                    // The place is the thread's: a second ask does not wait.
                    let again = emu_rows_with("sweep_a.py", &set, no_run).expect("a hit");
                    assert_eq!(again, rows);
                    live.fetch_sub(1, Ordering::SeqCst);
                    done.fetch_add(1, Ordering::SeqCst);
                });
            }
        });
        assert_eq!(done.load(Ordering::SeqCst), 2, "both threads finished");
        assert_eq!(
            peak.load(Ordering::SeqCst),
            1,
            "with the bound at one, one over-cap result is live at a time"
        );
    }

    /// The whole gate verifies (`RON_VERIFY_EMU`), and there a giant hit
    /// held two copies — the cached rows and the fresh run's — where the
    /// uncached path held one; the gate was killed at 4,110 MiB on
    /// `sweep_u_z`. Verify streams the cached entry against the fresh rows,
    /// and a cached entry cut short is a miss that is rewritten, not a
    /// panic. A re-run does not wait for a place before it starts (the
    /// sixty-third pass's choice): every whole gate verifies, and taking the
    /// place first serialized the giants' unicorn minutes — `sweep_u_z` 174
    /// → 577 s at eight threads — for a sampled peak of 1.9 GB against 3.9.
    #[test]
    fn verify_does_not_wait_for_a_place_before_a_giant_run_and_streams_the_cached_copy() {
        use std::sync::mpsc;
        let s = Scratch::new("verify-big");
        let (emu, exe) = lay_out(&s.0);
        let cache = s.0.join("cache");
        let mut set = setup(&emu, &exe, &[], Some(cache.clone()));
        set.cap = 8;
        set.big_rows = 1;
        let rows = "1 2 -> 3\n4 5 -> 9\n";
        let make = |_: &Path, _: &Path, _: &[String]| ok(rows);
        std::thread::scope(|scope| {
            scope.spawn(|| emu_rows_with("sweep_a.py", &set, make).expect("rows"));
        });
        let mut check = set.clone_for_test();
        check.verify = true;
        let (ran, seen) = mpsc::channel();
        let waited = std::thread::scope(|scope| {
            let (hold, held) = mpsc::channel();
            let set = &set;
            let holder = scope.spawn(move || {
                // A test holding the one place for its whole body.
                emu_rows_with("sweep_a.py", set, make).expect("a hit");
                hold.send(()).expect("send");
                seen.recv_timeout(std::time::Duration::from_secs(1))
                    .is_err()
            });
            held.recv().expect("the place is held");
            let check = &check;
            scope.spawn(move || {
                let rows =
                    emu_rows_with("sweep_a.py", check, |_: &Path, _: &Path, _: &[String]| {
                        let _ = ran.send(());
                        ok("1 2 -> 3\n4 5 -> 9\n")
                    });
                assert_eq!(rows.as_deref(), Some("1 2 -> 3\n4 5 -> 9\n"));
            });
            holder.join().expect("the holder")
        });
        assert!(
            !waited,
            "verify does not wait for a place before re-running a known giant"
        );

        // A cached copy cut short is a miss under verify: no panic, and the
        // entry is rewritten whole.
        let (k, _) = cache_key("sweep_a.py", &set).expect("key");
        let packed = cache.join(format!("{k}.rows.gz"));
        let good = std::fs::read(&packed).expect("a compressed entry");
        std::fs::write(&packed, &good[..good.len() - 6]).expect("write");
        let again = std::thread::scope(|scope| {
            scope
                .spawn(|| emu_rows_with("sweep_a.py", &check, make))
                .join()
                .expect("no panic")
        });
        assert_eq!(again.as_deref(), Some(rows));
        let text = gunzip(&packed).expect("rewritten whole");
        assert_eq!(strip_header(&text, &k), Some(rows));
    }

    /// Under the cap nothing changed: the file is today's, byte for byte —
    /// the header, the receipt, the date, the row count, `# end`, the
    /// rows — so no entry already in the cache is invalidated.
    #[test]
    fn a_result_under_the_cap_is_the_plain_format_byte_for_byte() {
        let s = Scratch::new("plain");
        let (emu, exe) = lay_out(&s.0);
        let cache = s.0.join("cache");
        let rows = "1 2 -> 3\n\n4 5 -> 9\n";
        let set = setup(&emu, &exe, &[], Some(cache.clone()));
        emu_rows_with("sweep_a.py", &set, |_: &Path, _: &Path, _: &[String]| {
            ok(rows)
        })
        .expect("rows");
        let (k, receipt) = cache_key("sweep_a.py", &set).expect("key");
        let bytes = std::fs::read(cache.join(format!("{k}.rows"))).expect("a plain entry");
        let text = String::from_utf8(bytes).expect("utf-8");
        let date = text
            .lines()
            .find_map(|l| l.strip_prefix("# date "))
            .expect("a date line");
        let mut want = format!("# emu-cache v1\n# key {k}\n");
        for line in &receipt {
            want.push_str(&format!("# {line}\n"));
        }
        want.push_str(&format!("# date {date}\n# rows 2\n# end\n{rows}"));
        assert_eq!(text, want, "the plain format, byte for byte");
        assert!(
            !cache.join(format!("{k}.rows.gz")).exists(),
            "under the cap: nothing compressed"
        );
    }

    /// A compressed entry that is not gzip, is cut short (its CRC and
    /// length never reached), or holds another key's file is a miss: the
    /// script runs, and its result replaces the entry.
    #[test]
    fn a_corrupt_compressed_entry_is_a_miss_not_a_panic() {
        let s = Scratch::new("corrupt");
        let (emu, exe) = lay_out(&s.0);
        let cache = s.0.join("cache");
        let calls = Cell::new(0);
        let run = |_: &Path, _: &Path, _: &[String]| {
            calls.set(calls.get() + 1);
            ok("1 2 -> 3\n4 5 -> 9\n")
        };
        let mut set = setup(&emu, &exe, &[], Some(cache.clone()));
        set.cap = 8;
        emu_rows_with("sweep_a.py", &set, run).expect("rows");
        let (k, _) = cache_key("sweep_a.py", &set).expect("key");
        let packed = cache.join(format!("{k}.rows.gz"));
        let good = std::fs::read(&packed).expect("a compressed entry");
        let other = {
            let text = gunzip(&packed).expect("gzip");
            let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            std::io::Write::write_all(&mut gz, text.replace(&k, &"0".repeat(64)).as_bytes())
                .expect("gzip");
            gz.finish().expect("gzip")
        };
        let cases: [(&str, Vec<u8>); 4] = [
            ("not gzip", b"not a gzip stream at all".to_vec()),
            ("cut short", good[..good.len() - 6].to_vec()),
            ("a flipped byte", {
                let mut b = good.clone();
                let mid = b.len() / 2;
                b[mid] ^= 0x55;
                b
            }),
            ("another key's file", other),
        ];
        for (n, (what, bytes)) in cases.into_iter().enumerate() {
            std::fs::write(&packed, bytes).expect("write");
            let rows = emu_rows_with("sweep_a.py", &set, run);
            assert_eq!(rows.as_deref(), Some("1 2 -> 3\n4 5 -> 9\n"), "{what}");
            assert_eq!(calls.get(), n + 2, "{what}: a miss runs the script");
            let text = gunzip(&packed).unwrap_or_else(|e| panic!("{what}: rewritten whole ({e})"));
            assert_eq!(
                strip_header(&text, &k),
                Some("1 2 -> 3\n4 5 -> 9\n"),
                "{what}"
            );
        }
    }

    #[test]
    fn a_header_strips_to_the_rows_and_a_stranger_is_a_miss() {
        let body = "# not a header\n1 -> 2\n";
        let text = format!("# emu-cache v1\n# key abc\n# rows 2\n# end\n{body}");
        assert_eq!(
            strip_header(&text, "abc"),
            Some(body),
            "only the header goes"
        );
        assert_eq!(strip_header(&text, "abd"), None, "another key's file");
        assert_eq!(
            strip_header("# emu-cache v1\n# key abc\n1 -> 2\n", "abc"),
            None,
            "no end"
        );
        assert_eq!(strip_header("1 -> 2\n", "abc"), None, "no header");
    }

    #[test]
    fn verify_mode_reruns_on_a_hit_and_panics_on_a_mismatch() {
        let s = Scratch::new("verify");
        let (emu, exe) = lay_out(&s.0);
        let cache = s.0.join("cache");
        let mut set = setup(&emu, &exe, &[], Some(cache));
        emu_rows_with("sweep_a.py", &set, |_: &Path, _: &Path, _: &[String]| {
            ok("1 -> 1\n2 -> 4\n")
        })
        .expect("rows");
        set.verify = true;
        let calls = Cell::new(0);
        let same = emu_rows_with("sweep_a.py", &set, |_: &Path, _: &Path, _: &[String]| {
            calls.set(calls.get() + 1);
            ok("1 -> 1\n2 -> 4\n")
        });
        assert_eq!(same.as_deref(), Some("1 -> 1\n2 -> 4\n"));
        assert_eq!(calls.get(), 1, "verify runs the script on a hit");

        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            emu_rows_with("sweep_a.py", &set, |_: &Path, _: &Path, _: &[String]| {
                ok("1 -> 1\n2 -> 5\n")
            })
        }));
        let err = caught.expect_err("a mismatch panics");
        let msg = err.downcast_ref::<String>().cloned().unwrap_or_default();
        assert!(msg.contains("line 2"), "{msg}");
        assert!(
            msg.contains("2 -> 4") && msg.contains("2 -> 5"),
            "both lines: {msg}"
        );
    }

    #[test]
    fn cache_off_runs_every_time_and_writes_nothing() {
        let root = Path::new("/r");
        let env = |pairs: &'static [(&'static str, &'static str)]| {
            move |k: &str| {
                pairs
                    .iter()
                    .find(|(n, _)| *n == k)
                    .map(|(_, v)| v.to_string())
            }
        };
        assert_eq!(
            cache_dir(env(&[]), root),
            Some(PathBuf::from("/r/target/emu-cache"))
        );
        assert_eq!(
            cache_dir(env(&[("HOME", "/Users/x")]), root),
            Some(PathBuf::from("/Users/x/Library/Caches/attrition/emu-cache"))
        );
        assert_eq!(
            cache_dir(env(&[("RON_EMU_CACHE_DIR", "/c")]), root),
            Some(PathBuf::from("/c"))
        );
        assert_eq!(cache_dir(env(&[("RON_EMU_CACHE", "off")]), root), None);
        assert_eq!(
            cache_dir(
                env(&[("RON_EMU_CACHE", "off"), ("RON_EMU_CACHE_DIR", "/c")]),
                root
            ),
            None
        );

        let s = Scratch::new("off");
        let (emu, exe) = lay_out(&s.0);
        let calls = Cell::new(0);
        let run = |_: &Path, _: &Path, _: &[String]| {
            calls.set(calls.get() + 1);
            ok("1 -> 1\n")
        };
        let set = setup(&emu, &exe, &[], None);
        emu_rows_with("sweep_a.py", &set, run).expect("rows");
        emu_rows_with("sweep_a.py", &set, run).expect("rows");
        assert_eq!(calls.get(), 2, "with the cache off every call runs");
    }

    #[test]
    fn a_runner_that_cannot_start_is_none_and_writes_nothing() {
        let s = Scratch::new("nouv");
        let (emu, exe) = lay_out(&s.0);
        let cache = s.0.join("cache");
        let set = setup(&emu, &exe, &[], Some(cache.clone()));
        let none = emu_rows_with("sweep_a.py", &set, |_: &Path, _: &Path, _: &[String]| {
            Err(io::Error::new(io::ErrorKind::NotFound, "no uv"))
        });
        assert_eq!(none, None);
        let written = std::fs::read_dir(&cache).map(|d| d.count()).unwrap_or(0);
        assert_eq!(written, 0, "nothing cached for a run that never ran");
    }
}
