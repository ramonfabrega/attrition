//! Experimental decoded oracle: source observations only, never a verdict.
//!
//! Each decoded frame is cached beside its dump as
//! `.ron-oracle-v1/<generation>/<key>.oracle`, the generation being the hash
//! of the parser sources the key already carries ([`generation`],
//! [`civilian_generation`]). A parser edit therefore leaves its predecessor's
//! entries as one directory, which [`gc`] removes; the flat
//! `.ron-oracle-v1/<key>.oracle` layout before generations is never read and
//! counts as one stale generation. The cache and its gc are the only parts
//! built outside the tests; the decoder is the tests' alone.
#[cfg(test)]
mod codec;
#[cfg(test)]
use crate::capture::indexed::IndexedCapture;
#[cfg(test)]
use crate::gamelog::{Frame, GroupDump, Log};
#[cfg(test)]
use std::collections::BTreeMap;

#[cfg(test)]
#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Oracle {
    pub(super) frame: Frame,
    pub(super) groups: Vec<GroupDump>,
    pub(super) leaders: [Option<BTreeMap<String, i64>>; 2],
    pub(super) civilian: Option<super::golden::observations::Civilian>,
    pub(super) regions: [Option<BTreeMap<String, i64>>; 2],
}
#[cfg(test)]
impl Oracle {
    fn parse(text: &str, n: i64) -> Self {
        Self::parse_with_regions(text, n, false)
    }
    fn parse_with_regions(text: &str, n: i64, include_regions: bool) -> Self {
        Self::parse_modes(text, n, include_regions, None)
    }
    fn parse_modes(text: &str, n: i64, include_regions: bool, civilian: Option<bool>) -> Self {
        let log = Log::parse_eager(text);
        let frame = IndexedCapture::frame_from_log(&log, n).unwrap();
        let block = log.frames().into_iter().find(|(k, _)| *k == n).unwrap().1;
        let groups = crate::gamelog::groups(block);
        let leaders = std::array::from_fn(|who| {
            log.leader_block(n, who as i64)
                .map(|b| super::leader::theirs(&b))
        });
        let regions = std::array::from_fn(|who| {
            include_regions
                .then(|| {
                    log.leader_block(n, who as i64)
                        .map(|b| super::leader::region_theirs(&b))
                })
                .flatten()
        });
        Self {
            frame,
            groups,
            leaders,
            regions,
            civilian: civilian
                .map(|ammo| super::golden::observations::Civilian::parse(text, &log, ammo)),
        }
    }
}

#[test]
#[ignore = "explicit parsed-oracle experiment; requires RON_ORACLE_SOURCE"]
fn decoded_frame_experiment() {
    let path = std::env::var("RON_ORACLE_SOURCE").expect("RON_ORACLE_SOURCE");
    let mut source = IndexedCapture::open(path).unwrap();
    let count = source.frames().len().min(128);
    let mut parse = std::time::Duration::ZERO;
    let mut encode = parse;
    let mut decode = parse;
    let (mut raw_bytes, mut binary_bytes, mut zipped_bytes) = (0, 0, 0);
    for at in 0..count {
        let text = source.read_frame(at).unwrap();
        let n = source.frames()[at].number;
        let started = std::time::Instant::now();
        let oracle = Oracle::parse(&text, n);
        parse += started.elapsed();
        let started = std::time::Instant::now();
        let bytes = codec::encode(&oracle).unwrap();
        let zipped = lz4_flex::compress_prepend_size(&bytes);
        encode += started.elapsed();
        let started = std::time::Instant::now();
        let unpacked = lz4_flex::decompress_size_prepended(&zipped).unwrap();
        let decoded = codec::decode(&unpacked).unwrap();
        decode += started.elapsed();
        assert_eq!(oracle, decoded, "whole oracle frame position {at}");
        raw_bytes += text.len();
        binary_bytes += bytes.len();
        zipped_bytes += zipped.len();
    }
    eprintln!(
        "oracle prototype: frames={count} parse={parse:?} encode_compress={encode:?} decompress_decode={decode:?} text={raw_bytes} binary={binary_bytes} compressed={zipped_bytes}"
    );
}

// Default beside each canonical source; the explicit experiment root must also
// be outside git because entries contain derived original observations.
#[cfg(test)]
pub(super) struct Store {
    enabled: bool,
    include_regions: bool,
    civilian: Option<bool>,
    directory: Option<std::path::PathBuf>,
    value: Oracle,
    compressed: Vec<u8>,
    decoded: Vec<u8>,
    pub(super) hits: usize,
    pub(super) misses: usize,
    pub(super) writes: usize,
}
#[cfg(test)]
impl Store {
    pub(super) fn new(enabled: bool) -> Self {
        Self {
            include_regions: false,
            civilian: None,
            enabled: enabled && std::env::var_os("RON_DISABLE_ORACLE_CACHE").is_none(),
            directory: std::env::var_os("RON_ORACLE_CACHE_DIR").map(Into::into),
            value: Oracle::default(),
            compressed: Vec::new(),
            decoded: Vec::new(),
            hits: 0,
            misses: 0,
            writes: 0,
        }
    }
    pub(super) fn with_regions(mut self) -> Self {
        self.include_regions = true;
        self
    }
    pub(super) fn with_civilian(mut self, ammo: bool) -> Self {
        self.civilian = Some(ammo);
        self
    }
    pub(super) fn enabled(&self) -> bool {
        self.enabled && !crate::gamelog::reads::active()
    }
    pub(super) fn read(&mut self, source: &mut IndexedCapture, at: usize) -> &Oracle {
        source.validate().unwrap();
        let n = source.frames()[at].number;
        let mut identity = identity(source, at, self.include_regions);
        if let Some(ammo) = self.civilian {
            identity.push_str(&format!(
                " civilian-ammo={ammo} schema={:016x}",
                civilian_schema()
            ));
        }
        let directory = self
            .directory
            .clone()
            .unwrap_or_else(|| source.oracle_identity().0.parent().unwrap().join(CACHE));
        let generation = match self.civilian {
            Some(_) => civilian_generation(),
            None => generation(),
        };
        let path = entry_path(&directory, &generation, &identity);
        if read_entry(
            &path,
            &identity,
            &mut self.compressed,
            &mut self.decoded,
            &mut self.value,
        )
        .is_some()
            && self.value.frame.n == n
        {
            source.validate().unwrap();
            if std::env::var_os("RON_VERIFY_ORACLE").is_some()
                || (self.civilian.is_some()
                    && std::env::var_os("RON_VERIFY_GOLDEN_READERS").is_some())
            {
                let text = source.read_frame(at).unwrap();
                assert_eq!(
                    self.value,
                    Oracle::parse_modes(&text, n, self.include_regions, self.civilian),
                    "whole oracle differs at position {at}"
                );
            }
            self.hits += 1;
            return &self.value;
        }
        self.misses += 1;
        let text = source.read_frame(at).unwrap();
        let value = Oracle::parse_modes(&text, n, self.include_regions, self.civilian);
        source.validate().unwrap();
        if let Ok(bytes) = pack(&identity, &value)
            && publish(&path, &bytes).is_ok()
        {
            self.writes += 1;
        }
        self.value = value;
        &self.value
    }
}
fn hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    })
}
/// The sources an entry's value is a function of: the parser that turns a
/// frame's text into an [`Oracle`] (`gamelog.rs`, `leader.rs`'s dump side,
/// `indexed.rs`'s `frame_from_log`), the codec that writes it
/// (`oracle/codec.rs`, `owned_codec.rs`, this file's `pack`), and the
/// readers that hand the parser its text (`indexed.rs`, `source.rs`,
/// `archive.rs`). Each is read through [`keyed`], not whole.
const SCHEMA_SOURCES: &[(&str, &str)] = &[
    ("gamelog.rs", include_str!("../gamelog.rs")),
    ("leader.rs", include_str!("leader.rs")),
    ("oracle.rs", include_str!("oracle.rs")),
    ("oracle/codec.rs", include_str!("oracle/codec.rs")),
    ("owned_codec.rs", include_str!("../capture/owned_codec.rs")),
    ("indexed.rs", include_str!("../capture/indexed.rs")),
    ("source.rs", include_str!("../capture/source.rs")),
    ("archive.rs", include_str!("../capture/archive.rs")),
];
/// The civilian readers' sources, read the same way.
const CIVILIAN_SOURCES: &[(&str, &str)] = &[
    ("golden.rs", include_str!("golden.rs")),
    (
        "golden/observations.rs",
        include_str!("golden/observations.rs"),
    ),
    ("ammo.rs", include_str!("ammo.rs")),
];
fn schema() -> u64 {
    static SCHEMA: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
    *SCHEMA.get_or_init(|| schema_of(SCHEMA_SOURCES))
}
fn schema_of(sources: &[(&str, &str)]) -> u64 {
    let mut text = String::new();
    for (_, source) in sources {
        text.push_str(&keyed(source));
    }
    hash(text.as_bytes())
}
/// The column-0 items in a keyed source that no entry's value depends on:
/// `leader.rs`'s crate side ([`super::leader::rows`],
/// [`super::leader::region_rows`]) and its widening ledger, which change on
/// nearly every widening and never touch what [`super::leader::theirs`]
/// reads off a block.
const CONSUMERS: &[&str] = &[
    "pub(crate) fn rows(",
    "pub(crate) fn region_rows(",
    "pub(crate) const UNMODELLED:",
];
/// A source as the generation key reads it (Loop 3382): every line but
/// blank ones, comments, `#[test]` functions, `mod …tests` modules and the
/// [`CONSUMERS`]. None of these can change a parsed [`Oracle`] or its bytes,
/// and before this an edit to any of them turned a 1.6 GB generation over
/// — the compared coverage pin went from 121 s warm to 232 s cold. An
/// item is skipped from its first line to the next column-0 line that
/// closes it (`}`, `]`, `)`), or to its own header when that is one line;
/// rustfmt is what makes column 0 a reliable item boundary.
fn keyed(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    // 0: keeping; 1: after `#[test]`, before the item's header; 2: in a
    // skipped item's body.
    let mut skip = 0u8;
    for line in source.lines() {
        let line = line.trim_end();
        let column0 = !line.is_empty() && !line.starts_with(char::is_whitespace);
        match skip {
            1 if column0 && !line.starts_with('#') && !line.starts_with("//") => {
                skip = if line.ends_with('}') || line.ends_with(';') {
                    0
                } else {
                    2
                };
                continue;
            }
            1 => continue,
            2 => {
                if column0 && line.starts_with(['}', ']', ')']) {
                    skip = 0;
                }
                continue;
            }
            _ => {}
        }
        if line == "#[test]" {
            skip = 1;
            continue;
        }
        if (line.starts_with("mod ") && line.ends_with("tests {"))
            || CONSUMERS.iter().any(|c| line.starts_with(c))
        {
            skip = 2;
            continue;
        }
        let code = line.trim_start();
        if code.is_empty() || code.starts_with("//") {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}
/// The cache directory beside a dump; every generation is a directory in it.
pub const CACHE: &str = ".ron-oracle-v1";
/// The generation an entry without civilian observations is written under:
/// the hash of the parser sources its key carries.
pub fn generation() -> String {
    format!("{:016x}", schema())
}
/// The generation an entry with civilian observations is written under: its
/// key carries the civilian readers' sources as well, which change far more
/// often than the parser's, so they move only these entries.
pub fn civilian_generation() -> String {
    format!("{:016x}-{:016x}", schema(), civilian_schema())
}
#[cfg(test)]
fn entry_path(directory: &std::path::Path, generation: &str, identity: &str) -> std::path::PathBuf {
    directory
        .join(generation)
        .join(format!("{:016x}.oracle", hash(identity.as_bytes())))
}
/// Removes every generation in `dump_dir`'s cache but the current ones, and
/// returns how many it removed. `dump_dir` is the directory holding a dump
/// (its cache is `dump_dir/.ron-oracle-v1`) or that cache itself; one with no
/// cache has nothing to remove. A generation is a directory whose name is
/// not [`generation`] or [`civilian_generation`]; the flat entries and torn
/// temporaries of the layout before generations are together one more. A
/// symlink, and any file that is neither, are left where they are. It is a
/// cache: whatever it removes, a read regenerates.
pub fn gc(dump_dir: &std::path::Path) -> std::io::Result<usize> {
    let cache = if dump_dir.file_name() == Some(std::ffi::OsStr::new(CACHE)) {
        dump_dir.to_path_buf()
    } else {
        dump_dir.join(CACHE)
    };
    let entries = match std::fs::read_dir(&cache) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(e) => return Err(e),
    };
    let current = [generation(), civilian_generation()];
    let (mut removed, mut flat) = (0, false);
    for entry in entries {
        let entry = entry?;
        let kind = entry.file_type()?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if kind.is_dir() {
            if !current.iter().any(|c| *c == name) {
                std::fs::remove_dir_all(entry.path())?;
                removed += 1;
            }
        } else if kind.is_file()
            && (name.ends_with(".oracle")
                || (name.starts_with(".oracle-") && name.ends_with(".tmp")))
        {
            std::fs::remove_file(entry.path())?;
            flat = true;
        }
    }
    Ok(removed + usize::from(flat))
}
/// [`gc`] on every cache under `root`, never through a symlink (a run
/// directory links the install): the caches found, and the generations
/// removed from them.
pub fn gc_tree(root: &std::path::Path) -> std::io::Result<(usize, usize)> {
    let (mut caches, mut removed) = (0, 0);
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if dir.file_name() == Some(std::ffi::OsStr::new(CACHE)) {
            caches += 1;
            removed += gc(&dir)?;
            continue;
        }
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                stack.push(entry.path());
            }
        }
    }
    Ok((caches, removed))
}
fn civilian_schema() -> u64 {
    static SCHEMA: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
    *SCHEMA.get_or_init(|| schema_of(CIVILIAN_SOURCES))
}
#[cfg(test)]
fn identity(source: &IndexedCapture, at: usize, include_regions: bool) -> String {
    let (path, length, modified) = source.oracle_identity();
    // Debug path representation is escaped, preserving non-UTF8 paths too.
    format!(
        "oracle-v1 schema={:016x} path={path:?} length={length} mtime={modified:?} at={at} regions={include_regions} range={:?}",
        schema(),
        source.frames()[at]
    )
}
#[cfg(test)]
const MAGIC: &[u8; 8] = b"RONORC\0\x01";
#[cfg(test)]
fn pack(identity: &str, value: &Oracle) -> Result<Vec<u8>, &'static str> {
    let decoded = codec::encode(value)?;
    let compressed = lz4_flex::compress_prepend_size(&decoded);
    let mut bytes = MAGIC.to_vec();
    bytes.extend_from_slice(
        &u32::try_from(identity.len())
            .map_err(|_| "identity too large")?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(identity.as_bytes());
    bytes.extend_from_slice(&compressed);
    bytes.extend_from_slice(&hash(&bytes).to_le_bytes());
    if bytes.len() > codec::LIMIT {
        return Err("entry too large");
    }
    Ok(bytes)
}
#[cfg(test)]
fn unpack(bytes: &[u8], identity: &str) -> Option<Oracle> {
    let mut value = Oracle::default();
    unpack_into(bytes, identity, &mut Vec::new(), &mut value)?;
    Some(value)
}
#[cfg(test)]
fn unpack_into(
    bytes: &[u8],
    identity: &str,
    decoded: &mut Vec<u8>,
    value: &mut Oracle,
) -> Option<()> {
    if bytes.len() < 24 || bytes.len() > codec::LIMIT || &bytes[..8] != MAGIC {
        return None;
    }
    let end = bytes.len() - 8;
    let t = profile::now();
    let sum = hash(&bytes[..end]);
    profile::add(&profile::HASH_NS, t);
    if sum != u64::from_le_bytes(bytes[end..].try_into().ok()?) {
        return None;
    }
    let n = u32::from_le_bytes(bytes[8..12].try_into().ok()?) as usize;
    let payload = 12usize.checked_add(n)?;
    if bytes.get(12..payload)? != identity.as_bytes() {
        return None;
    }
    let compressed = bytes.get(payload..end)?;
    let size = u32::from_le_bytes(compressed.get(..4)?.try_into().ok()?) as usize;
    if size > codec::LIMIT {
        return None;
    }
    let t = profile::now();
    decoded.resize(size, 0);
    if lz4_flex::decompress_into(&compressed[4..], decoded).ok()? != size {
        return None;
    }
    profile::add(&profile::LZ4_NS, t);
    profile::DECODED_BYTES.fetch_add(size as u64, std::sync::atomic::Ordering::Relaxed);
    let t = profile::now();
    let decoded = codec::decode_into(decoded, value).ok();
    profile::add(&profile::DECODE_NS, t);
    decoded
}
/// `RON_PROFILE_ORACLE`: the store's process-wide counters, printed by
/// [`profile_report`]. Off, each costs one relaxed load.
#[cfg(test)]
pub(super) mod profile {
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
    pub(super) static READS: AtomicU64 = AtomicU64::new(0);
    pub(super) static FILE_BYTES: AtomicU64 = AtomicU64::new(0);
    pub(super) static DECODED_BYTES: AtomicU64 = AtomicU64::new(0);
    pub(super) static IO_NS: AtomicU64 = AtomicU64::new(0);
    pub(super) static HASH_NS: AtomicU64 = AtomicU64::new(0);
    pub(super) static LZ4_NS: AtomicU64 = AtomicU64::new(0);
    pub(super) static DECODE_NS: AtomicU64 = AtomicU64::new(0);
    pub(super) static UNIQUE: std::sync::Mutex<
        Option<std::collections::HashSet<std::path::PathBuf>>,
    > = std::sync::Mutex::new(None);
    pub(super) fn on() -> bool {
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ON.get_or_init(|| std::env::var_os("RON_PROFILE_ORACLE").is_some())
    }
    pub(super) fn add(counter: &AtomicU64, since: Option<std::time::Instant>) {
        if let Some(t) = since {
            counter.fetch_add(t.elapsed().as_nanos() as u64, Relaxed);
        }
    }
    pub(super) fn now() -> Option<std::time::Instant> {
        on().then(std::time::Instant::now)
    }
}
/// One line of [`profile`]'s totals, when `RON_PROFILE_ORACLE` is set.
#[cfg(test)]
pub(super) fn profile_report() {
    use std::sync::atomic::Ordering::Relaxed;
    if !profile::on() {
        return;
    }
    let ms = |c: &std::sync::atomic::AtomicU64| c.load(Relaxed) / 1_000_000;
    let unique = profile::UNIQUE
        .lock()
        .unwrap()
        .as_ref()
        .map_or(0, |u| u.len());
    eprintln!(
        "oracle profile: entry reads={} unique entries={unique} file bytes={} decompressed bytes={} io={}ms hash={}ms lz4={}ms decode={}ms",
        profile::READS.load(Relaxed),
        profile::FILE_BYTES.load(Relaxed),
        profile::DECODED_BYTES.load(Relaxed),
        ms(&profile::IO_NS),
        ms(&profile::HASH_NS),
        ms(&profile::LZ4_NS),
        ms(&profile::DECODE_NS),
    );
}
/// **Measured, Loop 3382 (2026-10-21, `RON_PROFILE_ORACLE`, the coverage
/// pin `every_parsed_field_is_compared_by_the_instrument_or_pinned`)**: 159
/// walks, 38,665 entry reads, 38,665 distinct entries — no walk rereads a
/// block another walk read, so an in-process decoded LRU would hit nothing,
/// and a verified-once hash skip would skip nothing. Warm: 1.60 GB read
/// (io 0.97 s), FNV 1.47 s, lz4 1.66 s to 9.73 GB, decode 2.96 s — 7.1 s of
/// the test's 101 s, 0.18 ms a block; the rest is the walks' own ticks and
/// comparisons. Cold (a new [`generation`], i.e. after any edit to a file
/// [`schema`] hashes): every read misses, parses the text and publishes a
/// 1.6 GB generation, and the test took 232 s — ~3.4 ms a block, the
/// "3.7 ms a block" the forty-seventh pass measured. The lever is how often
/// the generation turns over, not this function.
#[cfg(test)]
fn read_entry(
    path: &std::path::Path,
    identity: &str,
    bytes: &mut Vec<u8>,
    decoded: &mut Vec<u8>,
    value: &mut Oracle,
) -> Option<()> {
    use std::io::Read;
    use std::sync::atomic::Ordering::Relaxed;
    if profile::on() {
        profile::READS.fetch_add(1, Relaxed);
        profile::UNIQUE
            .lock()
            .unwrap()
            .get_or_insert_with(Default::default)
            .insert(path.to_path_buf());
    }
    let t = profile::now();
    let file = std::fs::File::open(path).ok()?;
    if file.metadata().ok()?.len() > codec::LIMIT as u64 {
        return None;
    }
    bytes.clear();
    file.take(codec::LIMIT as u64 + 1).read_to_end(bytes).ok()?;
    profile::add(&profile::IO_NS, t);
    profile::FILE_BYTES.fetch_add(bytes.len() as u64, Relaxed);
    unpack_into(bytes, identity, decoded, value)
}
#[cfg(test)]
fn publish(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let parent = path.parent().unwrap();
    std::fs::create_dir_all(parent)?;
    let temp = parent.join(format!(
        ".oracle-{}-{}-{:?}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)?;
    let result = file
        .write_all(bytes)
        .and_then(|()| std::fs::hard_link(&temp, path));
    drop(file);
    // Only this invocation's create_new temporary file is disposable.
    let cleanup = std::fs::remove_file(temp);
    result.and(cleanup)
}

#[test]
fn entry_rejects_corruption_staleness_and_wrong_frame_identity() {
    let value = Oracle {
        frame: Frame::default(),
        groups: vec![],
        leaders: [None, None],
        regions: [None, None],
        civilian: None,
    };
    let bytes = pack("source version frame-position=0", &value).unwrap();
    assert_eq!(
        unpack(&bytes, "source version frame-position=0"),
        Some(value)
    );
    for wrong in [
        "changed source",
        "changed parser",
        "source version frame-position=1",
    ] {
        assert!(unpack(&bytes, wrong).is_none());
    }
    for at in 0..bytes.len() {
        let mut bad = bytes.clone();
        bad[at] ^= 1;
        assert!(unpack(&bad, "source version frame-position=0").is_none());
        assert!(unpack(&bytes[..at], "source version frame-position=0").is_none());
    }
    let mut bad = bytes.clone();
    let at = 12 + "source version frame-position=0".len();
    bad[at..at + 4].copy_from_slice(&u32::MAX.to_le_bytes());
    let end = bad.len() - 8;
    let checksum = hash(&bad[..end]);
    bad[end..].copy_from_slice(&checksum.to_le_bytes());
    assert!(unpack(&bad, "source version frame-position=0").is_none());
}

#[test]
fn store_preserves_duplicate_frames_source_versions_and_existing_entries() {
    use std::fs;
    let root = std::env::temp_dir().join(format!(
        "ron-oracle-test-{}-{:?}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    let path = root.join("capture.txt");
    let text = "BEGIN GAME\n BEGIN FRAME 3\n  BEGIN UNITDATA\n   BEGIN OBJECT\n    BEGIN SUBOBJECT\n     flags 1\n     o 1\n     who 0\n     x_internal 234\n BEGIN FRAME 3\n  BEGIN UNITDATA\n   BEGIN OBJECT\n    BEGIN SUBOBJECT\n     flags 1\n     o 2\n     who 1\n     x_internal 999\n";
    fs::write(&path, text).unwrap();
    let mut source = IndexedCapture::open(&path).unwrap();
    let mut store = Store {
        enabled: true,
        include_regions: false,
        civilian: None,
        directory: Some(root.join("cache")),
        value: Oracle::default(),
        compressed: Vec::new(),
        decoded: Vec::new(),
        hits: 0,
        misses: 0,
        writes: 0,
    };
    let first = Oracle::parse(&source.read_frame(0).unwrap(), 3);
    assert_eq!(store.read(&mut source, 0), &first);
    let second = Oracle::parse(&source.read_frame(1).unwrap(), 3);
    assert_eq!(store.read(&mut source, 1), &second);
    assert_ne!(first, second);
    assert_eq!(store.read(&mut source, 0), &first);
    assert_eq!(store.read(&mut source, 1), &second);
    assert_eq!((store.hits, store.misses, store.writes), (2, 2, 2));
    let key = identity(&source, 0, false);
    let entry = entry_path(store.directory.as_ref().unwrap(), &generation(), &key);
    assert!(publish(&entry, b"must not replace").is_err());
    assert_eq!(store.read(&mut source, 0), &first);
    // This test owns the entry: corrupt it to prove fallback preserves it.
    fs::write(&entry, b"corrupt test entry").unwrap();
    assert_eq!(store.read(&mut source, 0), &first);
    assert_eq!(fs::read(&entry).unwrap(), b"corrupt test entry");
    crate::gamelog::reads::start();
    assert!(
        !store.enabled(),
        "read-key recording must bypass decoded cache"
    );
    crate::gamelog::reads::stop();
    fs::write(&path, format!("{text} BEGIN FRAME 4\n")).unwrap();
    assert!(source.validate().is_err());
    let reopened = IndexedCapture::open(&path).unwrap();
    assert_ne!(identity(&reopened, 0, false), key);
    drop(source);
    drop(reopened);
    fs::remove_dir_all(root).unwrap(); // exclusively this test's synthetic files
}

#[test]
fn region_mode_has_distinct_identity_and_preserves_base_observations() {
    let root = std::env::temp_dir().join(format!(
        "ron-oracle-regions-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    let path = root.join("capture.txt");
    let text = "BEGIN GAME\n BEGIN FRAME 3\n  BEGIN LEADERDATA\n   who 0\n   reg_active[scan] 4\n   reg_active[scan] bad\n   reg_naval[scan] 8\n";
    std::fs::write(&path, text).unwrap();
    let mut source = IndexedCapture::open(&path).unwrap();
    let mut base = Store::new(true);
    base.enabled = true;
    base.directory = Some(root.join("cache"));
    let mut regions = Store::new(true).with_regions();
    regions.enabled = true;
    regions.directory = base.directory.clone();
    let original = Oracle::parse(text, 3);
    let expanded = Oracle::parse_with_regions(text, 3, true);
    assert_eq!(original.frame, expanded.frame);
    assert_eq!(original.groups, expanded.groups);
    assert_eq!(original.leaders, expanded.leaders);
    assert_eq!(original.regions, [None, None]);
    assert_eq!(
        expanded.regions[0],
        Some(BTreeMap::from([
            ("reg_active[0]".into(), 4),
            ("reg_active[1]".into(), i64::MIN),
            ("reg_naval[63]".into(), 8),
        ]))
    );
    assert_ne!(identity(&source, 0, false), identity(&source, 0, true));
    for _ in 0..2 {
        assert_eq!(base.read(&mut source, 0), &original);
        assert_eq!(regions.read(&mut source, 0), &expanded);
    }
    assert_eq!((base.hits, base.misses, base.writes), (1, 1, 1));
    assert_eq!((regions.hits, regions.misses, regions.writes), (1, 1, 1));
    crate::gamelog::reads::start();
    assert!(!regions.enabled());
    crate::gamelog::reads::stop();
    drop(source);
    std::fs::remove_dir_all(root).unwrap(); // only this test's authored files
}

#[test]
fn civilian_modes_do_not_alias_and_keep_every_observation() {
    let root = std::env::temp_dir().join(format!(
        "ron-civilian-modes-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    let path = root.join("capture.txt");
    let text = "BEGIN GAME\n BEGIN FRAME 3\n  BEGIN WORLD\n   seen2[scan] 7\n   seen3[scan] 8\n  BEGIN AMMO\n   who 0\n   o 7\n   index 2\n   flags 2\n   accuracy 17\n";
    std::fs::write(&path, text).unwrap();
    let mut source = IndexedCapture::open(&path).unwrap();
    for mode in [None, Some(false), Some(true)] {
        let mut store = Store::new(true);
        store.enabled = true;
        store.directory = Some(root.join("cache"));
        if let Some(ammo) = mode {
            store = store.with_civilian(ammo);
        }
        let expected = Oracle::parse_modes(text, 3, false, mode);
        for _ in 0..2 {
            assert_eq!(store.read(&mut source, 0), &expected);
        }
        assert_eq!((store.hits, store.misses, store.writes), (1, 1, 1));
        assert_eq!(expected.civilian.is_some(), mode.is_some());
    }
    assert_eq!(files_under(&root.join("cache")).len(), 3);
    let store = Store::new(true).with_civilian(true);
    crate::gamelog::reads::start();
    assert!(!store.enabled());
    crate::gamelog::reads::stop();
    drop(source);
    std::fs::remove_dir_all(root).unwrap(); // exclusively this test's authored files
}

#[cfg(test)]
fn scratch(tag: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "ron-oracle-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    root
}

#[cfg(test)]
fn files_under(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap() {
            let e = e.unwrap();
            if e.file_type().unwrap().is_dir() {
                stack.push(e.path());
            } else {
                out.push(e.path().strip_prefix(root).unwrap().to_path_buf());
            }
        }
    }
    out.sort();
    out
}

#[test]
fn gc_removes_every_generation_but_the_current() {
    let root = scratch("gc");
    let cache = root.join(CACHE);
    let stale = cache.join("00000000deadbeef");
    let current = cache.join(generation());
    for d in [&stale, &current] {
        std::fs::create_dir_all(d).unwrap();
        std::fs::write(d.join("0123456789abcdef.oracle"), b"entry").unwrap();
    }
    assert_ne!(stale, current);
    assert_eq!(gc(&root).unwrap(), 1);
    assert_eq!(
        files_under(&cache),
        [std::path::PathBuf::from(generation()).join("0123456789abcdef.oracle")]
    );
    assert_eq!(gc(&root).unwrap(), 0, "a second gc finds nothing stale");
    std::fs::remove_dir_all(root).unwrap(); // exclusively this test's authored files
}

#[test]
fn gc_takes_the_flat_layout_as_one_stale_generation_and_keeps_both_current() {
    let root = scratch("gc-flat");
    let cache = root.join(CACHE);
    for current in [generation(), civilian_generation()] {
        std::fs::create_dir_all(cache.join(&current)).unwrap();
        std::fs::write(cache.join(&current).join("a.oracle"), b"kept").unwrap();
    }
    assert_ne!(generation(), civilian_generation());
    std::fs::write(cache.join("0123456789abcdef.oracle"), b"flat").unwrap();
    std::fs::write(cache.join("fedcba9876543210.oracle"), b"flat").unwrap();
    std::fs::write(cache.join(".oracle-1-2-3.tmp"), b"torn publish").unwrap();
    std::fs::write(cache.join("README"), b"not ours").unwrap();
    std::fs::create_dir_all(cache.join("1111111111111111-2222222222222222")).unwrap();
    assert_eq!(
        gc(&root).unwrap(),
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
    assert_eq!(gc(&cache).unwrap(), 0);
    assert_eq!(gc(&root.join("absent")).unwrap(), 0);
    std::fs::remove_dir_all(root).unwrap(); // exclusively this test's authored files
}

#[test]
fn cache_path_carries_the_schema_directory_and_ignores_the_flat_layout() {
    let root = scratch("schema-dir");
    let path = root.join("capture.txt");
    let text = "BEGIN GAME\n BEGIN FRAME 3\n  BEGIN UNITDATA\n   BEGIN OBJECT\n    BEGIN SUBOBJECT\n     flags 1\n     o 1\n     who 0\n     x_internal 234\n";
    std::fs::write(&path, text).unwrap();
    let mut source = IndexedCapture::open(&path).unwrap();
    let key = identity(&source, 0, false);
    // A flat entry of the old layout, valid for this very identity but holding
    // a different value: a read that looked for it would return it.
    let wrong = Oracle {
        frame: Frame {
            n: 3,
            ..Frame::default()
        },
        ..Oracle::default()
    };
    let cache = root.join(CACHE);
    std::fs::create_dir_all(&cache).unwrap();
    let flat = cache.join(format!("{:016x}.oracle", hash(key.as_bytes())));
    std::fs::write(&flat, pack(&key, &wrong).unwrap()).unwrap();
    let mut store = Store::new(true);
    store.enabled = true;
    store.directory = None; // the default: beside the capture
    let expected = Oracle::parse(text, 3);
    assert_ne!(expected, wrong);
    assert_eq!(store.read(&mut source, 0), &expected);
    assert_eq!((store.hits, store.misses, store.writes), (0, 1, 1));
    let written = entry_path(&cache, &generation(), &key);
    assert_eq!(
        written.parent().unwrap(),
        cache.join(generation()),
        "the entry sits in its schema's directory"
    );
    assert_eq!(generation(), format!("{:016x}", schema()));
    assert!(written.is_file());
    assert_eq!(store.read(&mut source, 0), &expected);
    assert_eq!((store.hits, store.misses, store.writes), (1, 1, 1));
    let mut civilian = Store::new(true).with_civilian(false);
    civilian.enabled = true;
    civilian.read(&mut source, 0);
    assert_eq!(civilian.writes, 1);
    let mut layout = files_under(&cache);
    layout.retain(|f| f.parent() != Some(std::path::Path::new("")));
    let dirs: std::collections::BTreeSet<_> = layout
        .iter()
        .map(|f| f.parent().unwrap().to_path_buf())
        .collect();
    assert_eq!(
        dirs,
        [generation(), civilian_generation()]
            .into_iter()
            .map(std::path::PathBuf::from)
            .collect()
    );
    assert_eq!(
        gc(&root).unwrap(),
        1,
        "the flat entry is the stale generation"
    );
    assert!(!flat.exists());
    drop(source);
    std::fs::remove_dir_all(root).unwrap(); // exclusively this test's authored files
}

#[test]
fn gc_tree_finds_every_cache_and_never_follows_a_link() {
    let root = scratch("gc-tree");
    let stale = |dump: &std::path::Path| {
        let d = dump.join(CACHE).join("00000000deadbeef");
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("x.oracle"), b"stale").unwrap();
    };
    stale(&root.join("ch1/map-14"));
    stale(&root.join("logs"));
    let outside = scratch("gc-tree-outside");
    stale(&outside);
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside, root.join("ch1/install")).unwrap();
    assert_eq!(gc_tree(&root).unwrap(), (2, 2));
    assert!(
        outside
            .join(CACHE)
            .join("00000000deadbeef/x.oracle")
            .is_file()
    );
    assert_eq!(gc_tree(&root).unwrap(), (2, 0));
    std::fs::remove_dir_all(root).unwrap(); // exclusively this test's authored files
    std::fs::remove_dir_all(outside).unwrap(); // exclusively this test's authored files
}

/// Loop 3382: an edit to a test, a comment, or `leader.rs`'s crate side
/// leaves the generation where it was; an edit to the parser or the codec
/// turns it over.
#[test]
fn the_schema_key_moves_on_the_parser_and_the_codec_only() {
    let leader = include_str!("leader.rs");
    let codec = include_str!("oracle/codec.rs");
    let key = |l: &str, c: &str| schema_of(&[("leader.rs", l), ("oracle/codec.rs", c)]);
    let base = key(leader, codec);
    let rows = "pub(crate) fn rows(loaded: &crate::load::Loaded, built: &Built, who: usize) -> Vec<Row> {\n";
    let theirs =
        "pub(crate) fn theirs(block: &Block<'_>) -> std::collections::BTreeMap<String, i64> {\n";
    assert!(leader.contains(rows) && leader.contains(theirs));
    let consumer = leader.replace(rows, &format!("{rows}    let _crate_side = 1;\n"));
    let commented = leader.replace(theirs, &format!("{theirs}    // a note, no code\n"));
    let tested = format!("{leader}\n#[test]\nfn added() {{\n    assert!(true);\n}}\n");
    let one_line = format!("{leader}\n#[test]\nfn empty() {{}}\n");
    let unmodelled = leader.replace(
        "    (\"score\", \"no score is kept\"),\n",
        "    (\"score\", \"no score is kept\"),\n    (\"extra\", \"a ledger line\"),\n",
    );
    assert_ne!(unmodelled, leader);
    for (what, edited) in [
        ("rows", &consumer),
        ("a comment", &commented),
        ("a test", &tested),
        ("a one-line test", &one_line),
        ("UNMODELLED", &unmodelled),
    ] {
        assert_eq!(key(edited, codec), base, "an edit to {what} moved the key");
    }
    let parser = leader.replace(theirs, &format!("{theirs}    let _parser = 1;\n"));
    assert_ne!(key(&parser, codec), base, "a parser edit must move the key");
    let after_test = format!("{one_line}\nfn after() {{}}\n");
    assert_ne!(
        key(&after_test, codec),
        base,
        "code after a test is still keyed"
    );
    let wire = codec.replacen(
        "use std::collections::BTreeMap;\n",
        "use std::collections::BTreeMap;\nconst _WIRE: u8 = 1;\n",
        1,
    );
    assert_ne!(wire, codec);
    assert_ne!(key(leader, &wire), base, "a codec edit must move the key");
    // What the real sources keep: the parser's entry points, never the
    // crate side or a test.
    let kept = keyed(leader);
    assert!(
        kept.contains("pub(crate) fn theirs(") && kept.contains("pub(crate) fn region_theirs(")
    );
    assert!(
        !kept.contains("fn rows(")
            && !kept.contains("fn region_rows(")
            && !kept.contains("mod tests")
    );
    let kept = keyed(include_str!("../gamelog.rs"));
    for entry in [
        "fn parse_eager(",
        "pub fn groups(",
        "fn leader_block(",
        "pub(crate) mod reads {",
    ] {
        assert!(kept.contains(entry), "gamelog.rs keeps {entry}");
    }
    assert!(keyed(include_str!("oracle.rs")).contains("fn pack("));
    assert!(!keyed(include_str!("oracle.rs")).contains("fn the_schema_key_moves"));
}
