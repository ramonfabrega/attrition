//! Immutable source-keyed setup observations. Stream bounded records, not text.
use super::Observations;
use crate::capture::owned_codec::{self as codec, *};
use crate::gamelog::{FrameUnit, Initial};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{
    OnceLock,
    atomic::{AtomicU64, Ordering},
};
use std::time::{SystemTime, UNIX_EPOCH};

const MAGIC: &[u8; 8] = b"RONOBS\0\x01";
const MAX_FILE: u64 = 512 * 1024 * 1024;
const MAX_HEAP: usize = 512 * 1024 * 1024;
const MAX_FRAMES: usize = 100_000;
const HASH_START: u64 = 0xcbf29ce484222325;
fn hash(mut state: u64, bytes: &[u8]) -> u64 {
    for b in bytes {
        state = (state ^ u64::from(*b)).wrapping_mul(0x100000001b3);
    }
    state
}
fn invalid(s: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, s)
}
fn schema() -> u64 {
    static SCHEMA: OnceLock<u64> = OnceLock::new();
    *SCHEMA.get_or_init(|| {
        hash(
            HASH_START,
            concat!(
                include_str!("../../../gamelog.rs"),
                include_str!("../../owned_codec.rs"),
                include_str!("../../indexed.rs"),
                include_str!("../observations.rs"),
                include_str!("persistent.rs"),
                include_str!("../../source.rs"),
                include_str!("../../archive.rs")
            )
            .as_bytes(),
        )
    })
}
fn identity(path: &Path, length: u64, stored: u64, modified: SystemTime) -> String {
    format!(
        "setup-v1 schema={:016x} path={path:?} length={length} stored={stored} mtime={modified:?}",
        schema()
    )
}
fn entry(path: &Path, identity: &str) -> PathBuf {
    std::env::var_os("RON_SETUP_OBSERVATIONS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| path.parent().unwrap().join(".ron-observations-v1"))
        .join(format!(
            "{:016x}.observations",
            hash(HASH_START, identity.as_bytes())
        ))
}
fn note(event: &str, path: &Path) {
    if std::env::var_os("RON_PROFILE_SETUP_OBSERVATIONS").is_some() {
        eprintln!("setup observations: {event} {path:?}");
    }
}

struct Output<W> {
    file: W,
    hash: u64,
    bytes: u64,
}
impl<W: Write> Output<W> {
    fn put(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.bytes = self
            .bytes
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| invalid("size overflow"))?;
        if self.bytes > MAX_FILE - 8 {
            return Err(invalid("entry too large"));
        }
        self.file.write_all(bytes)?;
        self.hash = hash(self.hash, bytes);
        Ok(())
    }
    fn chunk<T: Wire>(&mut self, value: &T) -> io::Result<()> {
        let bytes = codec::encode(value).map_err(invalid)?;
        let compressed = lz4_flex::compress_prepend_size(&bytes);
        self.put(&(compressed.len() as u32).to_le_bytes())?;
        self.put(&compressed)
    }
}
fn write_entry(file: impl Write, identity: &str, init: &Initial<'_>) -> io::Result<()> {
    if identity.len() > 65536 || init.frame_guys.len() > MAX_FRAMES {
        return Err(invalid("entry limits"));
    }
    let mut out = Output {
        file,
        hash: HASH_START,
        bytes: 0,
    };
    out.put(MAGIC)?;
    out.put(&(identity.len() as u32).to_le_bytes())?;
    out.put(identity.as_bytes())?;
    out.chunk(&init.frame_seeds)?;
    out.chunk(&init.anim_lengths)?;
    out.put(&(init.frame_guys.len() as u32).to_le_bytes())?;
    // Encode borrowed rows separately; do not clone or encode the entire sequence.
    for row in &init.frame_guys {
        out.chunk(row)?;
    }
    out.file.write_all(&out.hash.to_le_bytes())?;
    out.file.flush()
}
struct Input<R> {
    file: R,
    hash: u64,
    bytes: u64,
    compressed: Vec<u8>,
    decoded: Vec<u8>,
    heap: usize,
}
impl<R: Read> Input<R> {
    fn take(&mut self, bytes: &mut [u8]) -> io::Result<()> {
        self.bytes = self
            .bytes
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| invalid("size overflow"))?;
        if self.bytes > MAX_FILE - 8 {
            return Err(invalid("entry too large"));
        }
        self.file.read_exact(bytes)?;
        self.hash = hash(self.hash, bytes);
        Ok(())
    }
    fn number(&mut self) -> io::Result<usize> {
        let mut bytes = [0; 4];
        self.take(&mut bytes)?;
        Ok(u32::from_le_bytes(bytes) as usize)
    }
    fn chunk<T: Wire>(&mut self) -> io::Result<T> {
        let n = self.number()?;
        if !(4..=codec::LIMIT + 1024 * 1024).contains(&n) {
            return Err(invalid("compressed size"));
        }
        self.bytes = self
            .bytes
            .checked_add(n as u64)
            .ok_or_else(|| invalid("size overflow"))?;
        if self.bytes > MAX_FILE - 8 {
            return Err(invalid("entry too large"));
        }
        self.compressed.resize(n, 0);
        self.file.read_exact(&mut self.compressed)?;
        self.hash = hash(self.hash, &self.compressed);
        let size = u32::from_le_bytes(self.compressed[..4].try_into().unwrap()) as usize;
        if size > codec::LIMIT {
            return Err(invalid("decoded size"));
        }
        self.decoded.resize(size, 0);
        if lz4_flex::decompress_into(&self.compressed[4..], &mut self.decoded)
            .map_err(|_| invalid("compression"))?
            != size
        {
            return Err(invalid("decoded length"));
        }
        let value: T = codec::decode(&self.decoded).map_err(invalid)?;
        self.heap = self
            .heap
            .checked_add(value.heap_bytes())
            .ok_or_else(|| invalid("heap overflow"))?;
        if self.heap > MAX_HEAP {
            return Err(invalid("observation heap limit"));
        }
        Ok(value)
    }
}
fn read_entry(file: impl Read, identity: &str) -> io::Result<Observations> {
    let mut input = Input {
        file,
        hash: HASH_START,
        bytes: 0,
        compressed: Vec::new(),
        decoded: Vec::new(),
        heap: 0,
    };
    let mut magic = [0; 8];
    input.take(&mut magic)?;
    if &magic != MAGIC {
        return Err(invalid("schema magic"));
    }
    let n = input.number()?;
    if n > 65536 || n != identity.len() {
        return Err(invalid("identity size"));
    }
    let mut key = vec![0; n];
    input.take(&mut key)?;
    if key != identity.as_bytes() {
        return Err(invalid("source identity"));
    }
    let seeds = input.chunk()?;
    let animations = input.chunk()?;
    let count = input.number()?;
    if count > MAX_FRAMES {
        return Err(invalid("frame count"));
    }
    input.heap += count * std::mem::size_of::<(i64, Vec<FrameUnit>)>();
    let mut guys = Vec::with_capacity(count);
    for _ in 0..count {
        guys.push(input.chunk()?);
    }
    let mut checksum = [0; 8];
    input.file.read_exact(&mut checksum)?;
    if u64::from_le_bytes(checksum) != input.hash {
        return Err(invalid("checksum"));
    }
    let mut tail = [0];
    if input.file.read(&mut tail)? != 0 {
        return Err(invalid("trailing bytes"));
    }
    Ok(Observations {
        seeds,
        animations,
        guys,
    })
}

pub(in crate::capture::indexed) fn load(
    path: &Path,
    length: u64,
    stored: u64,
    modified: SystemTime,
) -> Option<Observations> {
    let key = identity(path, length, stored, modified);
    let file = match File::open(entry(path, &key)) {
        Ok(file) => file,
        Err(_) => {
            note("miss", path);
            return None;
        }
    };
    if file.metadata().ok()?.len() > MAX_FILE {
        note("oversize", path);
        return None;
    }
    match read_entry(file, &key) {
        Ok(value) => {
            note("hit", path);
            Some(value)
        }
        Err(_) => {
            note("invalid", path);
            None
        }
    }
}
fn publish(final_path: &Path, key: &str, init: &Initial<'_>) -> io::Result<()> {
    let parent = final_path.parent().unwrap();
    if final_path.exists() {
        return Err(io::ErrorKind::AlreadyExists.into());
    }
    std::fs::create_dir_all(parent)?;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let temporary = parent.join(format!(
        ".setup-{}-{}-{}.tmp",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    // Explicit flush reports errors before the immutable publication.
    let result = write_entry(io::BufWriter::new(file), key, init)
        .and_then(|()| std::fs::hard_link(&temporary, final_path));
    // Only this invocation's create-new temporary file is disposable.
    let cleanup = std::fs::remove_file(temporary);
    result.and(cleanup)
}
pub(in crate::capture::indexed) fn save(
    path: &Path,
    length: u64,
    stored: u64,
    modified: SystemTime,
    init: &Initial<'_>,
) {
    let key = identity(path, length, stored, modified);
    let final_path = entry(path, &key);
    if final_path.exists() {
        return;
    }
    note(
        if publish(&final_path, &key, init).is_ok() {
            "write"
        } else {
            "write-failed"
        },
        path,
    );
}
record!(FrameUnit { who: i64, o: i64, guys: Vec<crate::gamelog::Guy>, orderless: bool, pos: crate::gamelog::Pos, goal: Option<crate::gamelog::Pos> });

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gamelog::{Guy, Pos};
    use std::io::Cursor;
    fn sample() -> Initial<'static> {
        Initial {
            frame_seeds: vec![(4, u32::MAX), (4, 0), (-1, 9)],
            anim_lengths: vec![(3, 2, 1), (3, 2, 1)],
            frame_guys: vec![
                (
                    4,
                    vec![FrameUnit {
                        who: i64::MIN,
                        o: i64::MAX,
                        orderless: true,
                        pos: Pos { x: 1, y: 2, z: 3 },
                        goal: Some(Pos {
                            x: -1,
                            y: -2,
                            z: -3,
                        }),
                        guys: vec![Guy {
                            cur_time: Some(77),
                            track: Some((-1, 9)),
                            turret_angles: Some([1, 2, 3, 4]),
                            ..Guy::default()
                        }],
                    }],
                ),
                (4, vec![]),
                (-1, vec![FrameUnit::default()]),
            ],
            ..Initial::default()
        }
    }
    #[test]
    fn streamed_observations_preserve_every_field_and_duplicate_order() {
        let init = sample();
        let expected = Observations::from_initial(&init);
        let mut bytes = Vec::new();
        write_entry(&mut bytes, "source", &init).unwrap();
        assert_eq!(read_entry(Cursor::new(&bytes), "source").unwrap(), expected);
        for key in ["other!", "source-v2"] {
            assert!(read_entry(Cursor::new(&bytes), key).is_err());
        }
        for end in 0..bytes.len() {
            assert!(read_entry(Cursor::new(&bytes[..end]), "source").is_err());
        }
        for i in 0..bytes.len() {
            let mut bad = bytes.clone();
            bad[i] ^= 0x80;
            assert!(
                read_entry(Cursor::new(bad), "source").is_err(),
                "corruption {i}"
            );
        }
        bytes.push(0);
        assert!(read_entry(Cursor::new(bytes), "source").is_err());
    }
    #[test]
    fn empty_observations_and_stale_source_identity() {
        let mut bytes = Vec::new();
        write_entry(&mut bytes, "empty", &Initial::default()).unwrap();
        assert_eq!(
            read_entry(Cursor::new(bytes), "empty").unwrap(),
            Observations::default()
        );
        let p = Path::new("/finalized/source");
        let key = identity(p, 100, 50, UNIX_EPOCH);
        assert_ne!(key, identity(p, 101, 50, UNIX_EPOCH));
        assert_ne!(key, identity(p, 100, 51, UNIX_EPOCH));
        assert_ne!(key, identity(Path::new("/other"), 100, 50, UNIX_EPOCH));
        assert_ne!(
            key,
            identity(p, 100, 50, UNIX_EPOCH + std::time::Duration::from_secs(1))
        );
    }
    #[test]
    fn oversized_chunks_are_rejected_before_allocation_and_flush_failure_propagates() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&u32::MAX.to_le_bytes());
        assert!(read_entry(Cursor::new(bytes), "").is_err());
        struct BadFlush;
        impl Write for BadFlush {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                Ok(bytes.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Err(io::Error::other("authored flush failure"))
            }
        }
        assert!(write_entry(BadFlush, "empty", &Initial::default()).is_err());
    }
    #[test]
    fn publication_never_replaces_an_existing_entry() {
        let dir = std::env::temp_dir().join(format!(
            "ron-setup-publication-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&dir).unwrap();
        let file = dir.join("record");
        let init = sample();
        publish(&file, "source", &init).unwrap();
        assert_eq!(
            publish(&file, "source", &Initial::default())
                .unwrap_err()
                .kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(
            read_entry(File::open(&file).unwrap(), "source").unwrap(),
            Observations::from_initial(&init)
        );
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        std::fs::remove_file(file).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }
    #[test]
    fn raw_key_recording_bypasses_both_observation_caches() {
        crate::gamelog::reads::start();
        assert!(!super::super::enabled());
        crate::gamelog::reads::stop();
    }
}
