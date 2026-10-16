//! Experimental decoded oracle: source observations only, never a verdict.
mod codec;
use crate::capture::indexed::IndexedCapture;
use crate::gamelog::{Frame, GroupDump, Log};
use std::collections::BTreeMap;

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Oracle {
    pub(super) frame: Frame,
    pub(super) groups: Vec<GroupDump>,
    pub(super) leaders: [Option<BTreeMap<String, i64>>; 2],
}
impl Oracle {
    fn parse(text: &str, n: i64) -> Self {
        let log = Log::parse_eager(text);
        let frame = IndexedCapture::frame_from_log(&log, n).unwrap();
        let block = log.frames().into_iter().find(|(k, _)| *k == n).unwrap().1;
        let groups = crate::gamelog::groups(block);
        let leaders = std::array::from_fn(|who| {
            log.leader_block(n, who as i64)
                .map(|b| super::leader::theirs(&b))
        });
        Self {
            frame,
            groups,
            leaders,
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
pub(super) struct Store {
    enabled: bool,
    directory: Option<std::path::PathBuf>,
    value: Oracle,
    compressed: Vec<u8>,
    decoded: Vec<u8>,
    pub(super) hits: usize,
    pub(super) misses: usize,
    pub(super) writes: usize,
}
impl Store {
    pub(super) fn new(enabled: bool) -> Self {
        Self {
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
    pub(super) fn enabled(&self) -> bool {
        self.enabled && !crate::gamelog::reads::active()
    }
    pub(super) fn read(&mut self, source: &mut IndexedCapture, at: usize) -> &Oracle {
        source.validate().unwrap();
        let n = source.frames()[at].number;
        let identity = identity(source, at);
        let directory = self.directory.clone().unwrap_or_else(|| {
            source
                .oracle_identity()
                .0
                .parent()
                .unwrap()
                .join(".ron-oracle-v1")
        });
        let path = directory.join(format!("{:016x}.oracle", hash(identity.as_bytes())));
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
            if std::env::var_os("RON_VERIFY_ORACLE").is_some() {
                let text = source.read_frame(at).unwrap();
                assert_eq!(
                    self.value,
                    Oracle::parse(&text, n),
                    "whole oracle differs at position {at}"
                );
            }
            self.hits += 1;
            return &self.value;
        }
        self.misses += 1;
        let text = source.read_frame(at).unwrap();
        let value = Oracle::parse(&text, n);
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
fn schema() -> u64 {
    static SCHEMA: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
    *SCHEMA.get_or_init(|| {
        hash(
            concat!(
                include_str!("../gamelog.rs"),
                include_str!("leader.rs"),
                include_str!("oracle.rs"),
                include_str!("oracle/codec.rs"),
                include_str!("../capture/owned_codec.rs"),
                include_str!("../capture/indexed.rs"),
                include_str!("../capture/source.rs"),
                include_str!("../capture/archive.rs")
            )
            .as_bytes(),
        )
    })
}
fn identity(source: &IndexedCapture, at: usize) -> String {
    let (path, length, modified) = source.oracle_identity();
    // Debug path representation is escaped, preserving non-UTF8 paths too.
    format!(
        "oracle-v1 schema={:016x} path={path:?} length={length} mtime={modified:?} at={at} range={:?}",
        schema(),
        source.frames()[at]
    )
}
const MAGIC: &[u8; 8] = b"RONORC\0\x01";
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
fn unpack(bytes: &[u8], identity: &str) -> Option<Oracle> {
    let mut value = Oracle::default();
    unpack_into(bytes, identity, &mut Vec::new(), &mut value)?;
    Some(value)
}
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
    if hash(&bytes[..end]) != u64::from_le_bytes(bytes[end..].try_into().ok()?) {
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
    decoded.resize(size, 0);
    if lz4_flex::decompress_into(&compressed[4..], decoded).ok()? != size {
        return None;
    }
    codec::decode_into(decoded, value).ok()
}
fn read_entry(
    path: &std::path::Path,
    identity: &str,
    bytes: &mut Vec<u8>,
    decoded: &mut Vec<u8>,
    value: &mut Oracle,
) -> Option<()> {
    use std::io::Read;
    let file = std::fs::File::open(path).ok()?;
    if file.metadata().ok()?.len() > codec::LIMIT as u64 {
        return None;
    }
    bytes.clear();
    file.take(codec::LIMIT as u64 + 1).read_to_end(bytes).ok()?;
    unpack_into(bytes, identity, decoded, value)
}
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
    let key = identity(&source, 0);
    let entry = store
        .directory
        .as_ref()
        .unwrap()
        .join(format!("{:016x}.oracle", hash(key.as_bytes())));
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
    assert_ne!(identity(&reopened, 0), key);
    drop(source);
    drop(reopened);
    fs::remove_dir_all(root).unwrap(); // exclusively this test's synthetic files
}
