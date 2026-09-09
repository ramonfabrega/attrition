//! Bounded-memory access to finalized captures' direct GAME/FRAME blocks.
//!
//! The index stores file ranges, not capture text. A requested frame is read
//! into an owned String and wrapped in GAME, so the existing parser and its
//! borrowed accessors can be used unchanged. Drop that String/Log before reading
//! the next frame to keep memory proportional to the largest requested frame.
//! Setup and shutdown siblings are deliberately not included in this API.
use std::fs::{File, Metadata};
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::SystemTime;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameRange {
    pub number: i64,
    pub offset: u64,
    pub length: u64,
}

pub struct IndexedCapture {
    file: File,
    length: u64,
    modified: SystemTime,
    frames: Arc<[FrameRange]>,
}

// Share only offsets, never capture text or parsed state. A process-wide
// eight-MiB/32-entry FIFO bounds retained index storage across test captures.
#[derive(Clone)]
struct Cached {
    path: PathBuf,
    length: u64,
    modified: SystemTime,
    frames: Arc<[FrameRange]>,
}
static INDEX_CACHE: OnceLock<Mutex<Vec<Cached>>> = OnceLock::new();
const CACHE_BYTES: usize = 8 * 1024 * 1024;
fn cache_bytes(entries: &[Cached]) -> usize {
    entries
        .iter()
        .map(|e| std::mem::size_of_val(e.frames.as_ref()) + e.path.as_os_str().len())
        .sum()
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

impl IndexedCapture {
    /// Scan using one reusable line buffer. Requires a single conventional
    /// root GAME and its space-indented FRAME children; rejects ambiguous
    /// trailing-field shapes rather than returning a guessed frame boundary.
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().canonicalize()?;
        let file = File::open(&path)?;
        let meta = file.metadata()?;
        let modified = meta.modified()?;
        let cache = INDEX_CACHE.get_or_init(|| Mutex::new(Vec::new()));
        if let Ok(entries) = cache.lock()
            && let Some(entry) = entries
                .iter()
                .find(|e| e.path == path && e.length == meta.len() && e.modified == modified)
        {
            return Ok(Self {
                file,
                length: meta.len(),
                modified,
                frames: Arc::clone(&entry.frames),
            });
        }
        let mut reader = BufReader::with_capacity(256 * 1024, file);
        let mut line = Vec::new();
        let mut offset = 0u64;
        let mut in_game = false;
        let mut found_game = false;
        let mut active: Option<usize> = None;
        let mut frames: Vec<FrameRange> = Vec::new();
        let mut closed_by_field = false;
        loop {
            line.clear();
            let bytes = reader.read_until(b'\n', &mut line)?;
            if bytes == 0 {
                break;
            }
            let text = std::str::from_utf8(&line).map_err(|_| invalid("capture is not UTF-8"))?;
            let trimmed = text.trim_start_matches(' ');
            let indent = text.len() - trimmed.len();
            let trimmed = trimmed.trim_end();
            if trimmed.is_empty() {
                offset += bytes as u64;
                continue;
            }
            let name = trimmed.strip_prefix("BEGIN ");
            if in_game && (indent == 0 || indent == 1 && name.is_some()) {
                if let Some(i) = active.take() {
                    frames[i].length = offset - frames[i].offset;
                }
                if indent == 0 {
                    in_game = false;
                }
            }
            if indent == 0 && name == Some("GAME") {
                if found_game {
                    return Err(invalid("multiple GAME roots are unsupported"));
                }
                found_game = true;
                in_game = true;
            } else if in_game {
                if indent != 1 && name.is_some_and(|n| n.starts_with("FRAME ")) {
                    return Err(invalid("nonstandard FRAME indentation is unsupported"));
                }
                if indent == 1 {
                    closed_by_field = name.is_none();
                    if let Some(n) = name.and_then(|n| n.strip_prefix("FRAME ")) {
                        let number = n
                            .trim()
                            .parse()
                            .map_err(|_| invalid("invalid FRAME number"))?;
                        active = Some(frames.len());
                        frames.push(FrameRange {
                            number,
                            offset,
                            length: 0,
                        });
                    }
                } else if indent > 1 && name.is_some() && closed_by_field {
                    return Err(invalid(
                        "block after a GAME-level trailing field is unsupported",
                    ));
                }
            }
            offset += bytes as u64;
        }
        if let Some(i) = active {
            frames[i].length = offset - frames[i].offset;
        }
        if !found_game {
            return Err(invalid("no root GAME block"));
        }
        let capture = Self {
            file: reader.into_inner(),
            length: meta.len(),
            modified,
            frames: frames.into(),
        };
        capture.check_metadata(capture.file.metadata()?)?;
        let entry = Cached {
            path,
            length: capture.length,
            modified,
            frames: Arc::clone(&capture.frames),
        };
        if std::mem::size_of_val(entry.frames.as_ref()) + entry.path.as_os_str().len()
            <= CACHE_BYTES
            && let Ok(mut entries) = cache.lock()
        {
            entries.retain(|e| e.path != entry.path);
            entries.push(entry);
            while entries.len() > 32 || cache_bytes(&entries) > CACHE_BYTES {
                entries.remove(0);
            }
        }
        Ok(capture)
    }

    fn check_metadata(&self, meta: Metadata) -> io::Result<()> {
        if meta.len() != self.length || meta.modified()? != self.modified {
            return Err(invalid("capture changed since indexing"));
        }
        Ok(())
    }

    pub fn frames(&self) -> &[FrameRange] {
        &self.frames
    }

    /// Parse and yield one owned frame at a time. Unlike Log::frame_states,
    /// this does not collect the entire capture's decoded state before yielding.
    pub fn frame_states(&mut self) -> impl Iterator<Item = io::Result<crate::gamelog::Frame>> + '_ {
        (0..self.frames.len()).map(|i| {
            let text = self.read_frame(i)?;
            let mut states = crate::gamelog::Log::parse(&text).frame_states();
            if states.len() != 1 || states[0].n != self.frames[i].number {
                return Err(invalid("indexed frame did not parse as one matching FRAME"));
            }
            Ok(states.pop().expect("one frame"))
        })
    }

    pub fn source_bytes(&self) -> u64 {
        self.length
    }

    /// Read one frame by index (not frame number; duplicate numbers may occur).
    /// Metadata checks catch ordinary edits/truncation; use finalized inputs.
    /// This is safe I/O, not a memory mapping or a claim of atomic snapshots.
    pub fn read_frame(&mut self, index: usize) -> io::Result<String> {
        self.check_metadata(self.file.metadata()?)?;
        let range = self
            .frames
            .get(index)
            .ok_or_else(|| invalid("frame index out of range"))?;
        let size = usize::try_from(range.length).map_err(|_| invalid("frame too large"))?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(
                size.checked_add(11)
                    .ok_or_else(|| invalid("frame too large"))?,
            )
            .map_err(|e| io::Error::other(e.to_string()))?;
        bytes.extend_from_slice(b"BEGIN GAME\n");
        bytes.resize(size + 11, 0);
        self.file.seek(SeekFrom::Start(range.offset))?;
        self.file.read_exact(&mut bytes[11..])?;
        self.check_metadata(self.file.metadata()?)?;
        String::from_utf8(bytes).map_err(|_| invalid("frame is not UTF-8"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Input(std::path::PathBuf);
    impl Input {
        fn new(text: &str) -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "attrition-index-{}-{}.txt",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::write(&path, text).unwrap();
            Self(path)
        }
    }
    impl Drop for Input {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    #[test]
    fn streamed_records_equal_full_parse_with_siblings_and_duplicate_frames() {
        let text = "CHECKSUM 0\r\nBEGIN GAME\r\n BEGIN CONSTANTS\r\n  tuning 5\r\n BEGIN FRAME 3\r\n  BEGIN UNITDATA\r\n   BEGIN OBJECT\r\n    BEGIN SUBOBJECT\r\n     flags 1\r\n     o 1\r\n     who 0\r\n     x_internal 234\r\n     y_internal 567\r\n BEGIN FULL DUMP\r\n  ignored 2\r\n BEGIN FRAME 3\r\n  BEGIN UNITDATA\r\n   BEGIN OBJECT\r\n    BEGIN SUBOBJECT\r\n     flags 1\r\n     o 2\r\n     who 1\r\n     x_internal 999\r\n BEGIN UNITDATA\r\n  closing 1\r\nGameInfo closing\r\nBEGIN FRAME 99\r\n";
        let input = Input::new(text);
        let mut source = IndexedCapture::open(&input.0).unwrap();
        let got = source
            .frame_states()
            .collect::<io::Result<Vec<_>>>()
            .unwrap();
        let expected = crate::gamelog::Log::parse(text).frame_states();
        assert_eq!(got, expected);
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].units[0].pos.x, 234);
        assert_eq!(got[1].units[0].pos.x, 999);
    }

    #[test]
    fn eof_is_a_valid_boundary_but_a_changed_file_is_not() {
        let input = Input::new("BEGIN GAME\n BEGIN FRAME 1\n  value 1");
        let mut source = IndexedCapture::open(&input.0).unwrap();
        assert_eq!(source.frame_states().next().unwrap().unwrap().n, 1);
        std::fs::write(&input.0, b"truncated").unwrap();
        assert!(
            source
                .read_frame(0)
                .unwrap_err()
                .to_string()
                .contains("changed")
        );
    }

    #[test]
    fn ambiguous_tail_is_rejected_instead_of_silently_losing_a_frame() {
        let input = Input::new("BEGIN GAME\n BEGIN FRAME 1\n field 1\n  BEGIN FRAME 2\n");
        assert!(
            IndexedCapture::open(&input.0)
                .err()
                .unwrap()
                .to_string()
                .contains("unsupported")
        );
    }

    #[test]
    fn reopening_shares_offsets_and_a_changed_length_invalidates_them() {
        let input = Input::new("BEGIN GAME\n BEGIN FRAME 1\n");
        let first = IndexedCapture::open(&input.0).unwrap();
        let second = IndexedCapture::open(&input.0).unwrap();
        assert!(Arc::ptr_eq(&first.frames, &second.frames));
        std::fs::write(&input.0, "BEGIN GAME\n BEGIN FRAME 12345\n").unwrap();
        let third = IndexedCapture::open(&input.0).unwrap();
        assert_eq!(third.frames[0].number, 12345);
        assert!(!Arc::ptr_eq(&first.frames, &third.frames));
    }

    #[test]
    fn missing_game_and_invalid_utf8_are_errors() {
        let input = Input::new("BEGIN FRAME 1\n");
        assert!(IndexedCapture::open(&input.0).is_err());
        std::fs::write(&input.0, [0xff, b'\n']).unwrap();
        assert!(IndexedCapture::open(&input.0).is_err());
    }
}
