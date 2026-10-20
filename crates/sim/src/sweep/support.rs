//! The sweeps' one door to the emulator, `emu_rows`, and its cache: a run
//! is keyed on the SHA-256 of every input that decides its rows — the
//! executable, the script, the `tools/emu/` modules it can import — and the
//! argument list, so a hit skips `uv` and a changed byte anywhere is a
//! miss. Each cache file carries its receipt (`docs/EMULATOR.md` §1.1;
//! `tools/emu_receipts.py` lists and verifies them).

use std::io;
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
    /// The most bytes of rows a result may have and still be stored
    /// (`RON_EMU_CACHE_MAX_BYTES`; [`DEFAULT_CACHE_CAP`]): the forty-fifth
    /// pass found 136 of 1,523 entries holding 20.3 of the cache's 21.7 GiB
    /// after one gate, the largest 1.3 GB of rows. A result over the cap is
    /// returned and never written; the script runs each time, as before.
    pub(crate) cap: usize,
}

/// 16 MiB: keeps 1,387 of 1,523 entries at 1.35 GiB (measured 2026-10-19).
pub(crate) const DEFAULT_CACHE_CAP: usize = 16 * 1024 * 1024;

/// The rows `tools/emu/<script>` prints for the install's executable, or
/// `None` (said on stderr) where the install or `uv` is absent. Cached, and
/// verified under `RON_VERIFY_EMU`: see `emu_rows_with`.
pub(crate) fn emu_rows(script: &str) -> Option<String> {
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
        args: &[],
        cache: cache_dir(get, Path::new(root)),
        verify: get("RON_VERIFY_EMU").is_some_and(|v| !v.is_empty() && v != "0"),
        probe: true,
        cap: get("RON_EMU_CACHE_MAX_BYTES")
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(DEFAULT_CACHE_CAP),
    };
    emu_rows_with(script, &setup, run_uv)
}

/// The real runner: `uv run <script> <exe> <args…>`.
fn run_uv(script: &Path, exe: &Path, args: &[String]) -> io::Result<Output> {
    std::process::Command::new("uv")
        .arg("run")
        .arg(script)
        .arg(exe)
        .args(args)
        .output()
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
/// cache: a hit returns the cached rows and skips the runner (unless
/// `verify`, which runs it and panics on the first line the two part); a
/// miss runs and writes `<key>.rows`, its receipt header then the rows.
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
    let file = cache.join(format!("{key}.rows"));
    if let Ok(text) = std::fs::read_to_string(&file)
        && let Some(cached) = strip_header(&text, &key)
    {
        if !setup.verify {
            return Some(cached.to_string());
        }
        let now = fresh()?;
        if let Some((n, was, is)) = first_difference(cached, &now) {
            panic!(
                "RON_VERIFY_EMU: {script}'s cached rows ({}) part from a fresh run at line {n}: \
                 cached `{was}`, fresh `{is}`",
                file.display()
            );
        }
        return Some(now);
    }
    let rows = fresh()?;
    if rows.len() > setup.cap {
        eprintln!(
            "emu cache: {script}'s rows ({} bytes) are over the cap ({}); not stored",
            rows.len(),
            setup.cap
        );
        return Some(rows);
    }
    if let Err(e) = store(cache, &file, &key, &receipt, &rows, setup.probe) {
        eprintln!("emu cache: cannot write {} ({e})", file.display());
    }
    Some(rows)
}

/// The first line two texts part on: its number (from 1) and each side's
/// line, `<end>` past a side's last.
fn first_difference(a: &str, b: &str) -> Option<(usize, String, String)> {
    let (mut x, mut y) = (a.split('\n'), b.split('\n'));
    let mut n = 0;
    loop {
        n += 1;
        match (x.next(), y.next()) {
            (None, None) => return None,
            (p, q) if p == q => {}
            (p, q) => {
                let side = |l: Option<&str>| l.unwrap_or("<end>").to_string();
                return Some((n, side(p), side(q)));
            }
        }
    }
}

/// Writes `file` through a temporary sibling and a rename, so a reader
/// never sees half of it.
fn store(
    cache: &Path,
    file: &Path,
    key: &str,
    receipt: &[String],
    rows: &str,
    probe: bool,
) -> io::Result<()> {
    std::fs::create_dir_all(cache)?;
    let mut head = format!("# {MAGIC}\n# key {key}\n");
    let mut line = |l: &str| {
        head.push_str("# ");
        head.push_str(l);
        head.push('\n');
    };
    receipt.iter().for_each(|l| line(l));
    line(&format!("date {}", utc_now()));
    if probe {
        tool_versions().iter().for_each(|l| line(l));
    }
    let n = rows.lines().filter(|l| !l.trim().is_empty()).count();
    line(&format!("rows {n}"));
    line("end");
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let tmp = cache.join(format!(".{key}.{}.{nanos}.tmp", std::process::id()));
    std::fs::write(&tmp, format!("{head}{rows}"))?;
    std::fs::rename(&tmp, file)
}

/// `uv --version` and the Python `uv run` resolves by default, once a
/// process: the receipt's tool lines.
fn tool_versions() -> &'static [String] {
    static V: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    V.get_or_init(|| {
        let out = |args: &[&str]| {
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
            format!("uv {}", out(&["--version"]).trim_start_matches("uv ")),
            format!(
                "python {} (uv run --no-project)",
                out(&["run", "--no-project", "python", "--version"]).trim_start_matches("Python ")
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

    #[test]
    fn a_result_over_the_cap_is_returned_and_not_stored() {
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
        let (k, _) = cache_key("sweep_a.py", &set).expect("key");
        assert!(
            !cache.join(format!("{k}.rows")).exists(),
            "over the cap: not stored"
        );
        emu_rows_with("sweep_a.py", &set, run).expect("rows");
        assert_eq!(calls.get(), 2, "uncached, it runs each time");
        set.cap = DEFAULT_CACHE_CAP;
        emu_rows_with("sweep_a.py", &set, run).expect("rows");
        assert!(
            cache.join(format!("{k}.rows")).exists(),
            "under the cap: stored"
        );
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
