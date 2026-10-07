//! Bounded-memory access to finalized captures' direct GAME/FRAME blocks.
//!
//! The index stores file ranges, not capture text. A requested frame is read
//! into an owned String and wrapped in GAME, so the existing parser and its
//! borrowed accessors can be used unchanged. Drop that String/Log before reading
//! the next frame to keep memory proportional to the largest requested frame.
//! Setup is separate; `read_shutdown` retains the last frame and its siblings.
mod observations;

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
    path: PathBuf,
    length: u64,
    modified: SystemTime,
    frames: Arc<[FrameRange]>,
    setup: Arc<[(u64, u64)]>,
}

// Share only offsets, never capture text or parsed state. A process-wide
// eight-MiB FIFO bounds retained index storage across test captures. **By
// bytes alone** (item 1571): a 32-entry bound beside it evicted the suite's
// indexes long before the bytes did — every distinct capture's index
// together is 4.0 MB — so the release suite built 623 indexes for 329
// captures, 145 of its 290 index seconds rebuilding what it had dropped;
// without it, 336 for 328, and the rondata suite 641 s → 611 s.
#[derive(Clone)]
struct Cached {
    path: PathBuf,
    length: u64,
    modified: SystemTime,
    frames: Arc<[FrameRange]>,
    setup: Arc<[(u64, u64)]>,
}
static INDEX_CACHE: OnceLock<Mutex<Vec<Cached>>> = OnceLock::new();
const CACHE_BYTES: usize = 8 * 1024 * 1024;
fn cache_bytes(entries: &[Cached]) -> usize {
    entries.iter().map(Cached::bytes).sum()
}

impl Cached {
    fn bytes(&self) -> usize {
        std::mem::size_of_val(self.frames.as_ref())
            + std::mem::size_of_val(self.setup.as_ref())
            + self.path.as_os_str().len()
    }
}

// Select setup while the index already has each line in hand. Adjacent kept
// lines coalesce; no source text is retained in the cache.
struct SetupRanges {
    ranges: Vec<(u64, u64)>,
    in_game: bool,
    keep: bool,
    seen: std::collections::BTreeSet<String>,
}
impl SetupRanges {
    fn new() -> Self {
        Self {
            ranges: Vec::new(),
            in_game: false,
            keep: true,
            seen: Default::default(),
        }
    }
    fn line(&mut self, offset: u64, bytes: u64, indent: usize, trimmed: &str, before: bool) {
        if !trimmed.is_empty() {
            let name = trimmed.strip_prefix("BEGIN ");
            if indent == 0 {
                self.in_game = name == Some("GAME");
            }
            let relevant = indent == 1
                && self.in_game
                && matches!(name, Some("FULL DUMP" | "WORLD" | "CITIES" | "CONSTANTS"));
            if before {
                if relevant {
                    self.seen.insert(name.unwrap().to_owned());
                }
            } else if !self.in_game || indent == 0 {
                self.keep = true;
            } else if indent == 1
                && let Some(name) = name
            {
                self.keep = relevant && self.seen.insert(name.to_owned());
            } else if indent == 1 {
                self.keep = true;
            }
        }
        if before || self.keep {
            if let Some((start, length)) = self.ranges.last_mut()
                && *start + *length == offset
            {
                *length += bytes;
            } else {
                self.ranges.push((offset, bytes));
            }
        }
    }
}

/// Opens, counted (item 1571): `RON_INDEX_STATS=<file>` appends one line
/// per open — `hit` or `build`, the build's seconds, the index's bytes, the
/// path — so a suite's re-indexing can be measured after it ran, as
/// `RON_READ_STATS` measures its reads. One `write` a line: the suite's
/// threads append at once, and `writeln!` interleaved them.
fn index_stat(kind: &str, seconds: f64, path: &Path, bytes: usize) {
    let Ok(stats) = std::env::var("RON_INDEX_STATS") else {
        return;
    };
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(stats)
    {
        use std::io::Write;
        let line = format!("{kind} {seconds:.4} {bytes} {}\n", path.display());
        let _ = f.write_all(line.as_bytes());
    }
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
            index_stat("hit", 0.0, &path, 0);
            return Ok(Self {
                file,
                path: path.clone(),
                length: meta.len(),
                modified,
                frames: Arc::clone(&entry.frames),
                setup: Arc::clone(&entry.setup),
            });
        }
        let built = std::time::Instant::now();
        let mut reader = BufReader::with_capacity(256 * 1024, file);
        let mut line = Vec::new();
        let mut offset = 0u64;
        let mut in_game = false;
        let mut found_game = false;
        let mut active: Option<usize> = None;
        let mut frames: Vec<FrameRange> = Vec::new();
        let mut closed_by_field = false;
        let mut setup = SetupRanges::new();
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
                setup.line(offset, bytes as u64, indent, trimmed, frames.is_empty());
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
            setup.line(offset, bytes as u64, indent, trimmed, frames.is_empty());
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
            path: path.clone(),
            length: meta.len(),
            modified,
            frames: frames.into(),
            setup: setup.ranges.into(),
        };
        capture.check_metadata(capture.file.metadata()?)?;
        let bytes = std::mem::size_of_val(capture.frames.as_ref())
            + std::mem::size_of_val(capture.setup.as_ref());
        index_stat("build", built.elapsed().as_secs_f64(), &path, bytes);
        let entry = Cached {
            path,
            length: capture.length,
            modified,
            frames: Arc::clone(&capture.frames),
            setup: Arc::clone(&capture.setup),
        };
        if entry.bytes() <= CACHE_BYTES
            && let Ok(mut entries) = cache.lock()
        {
            entries.retain(|e| e.path != entry.path);
            entries.push(entry);
            while cache_bytes(&entries) > CACHE_BYTES {
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

    /// Check that the finalized source still has its indexed length and mtime.
    pub fn validate(&self) -> io::Result<()> {
        self.check_metadata(self.file.metadata()?)
    }

    pub fn frames(&self) -> &[FrameRange] {
        &self.frames
    }

    /// Parse and yield one owned frame at a time. Unlike Log::frame_states,
    /// this does not collect the entire capture's decoded state before yielding.
    pub fn frame_states(&mut self) -> impl Iterator<Item = io::Result<crate::gamelog::Frame>> + '_ {
        (0..self.frames.len()).map(|i| self.frame_state(i))
    }

    /// Decode one frame by index, preserving duplicate labels by position.
    pub fn frame_state(&mut self, index: usize) -> io::Result<crate::gamelog::Frame> {
        let text = self.read_frame(index)?;
        // This is one indexed frame, and decoding visits its whole body.
        let mut states = crate::gamelog::Log::parse_eager(&text).frame_states();
        if states.len() != 1 || states[0].n != self.frames[index].number {
            return Err(invalid("indexed frame did not parse as one matching FRAME"));
        }
        Ok(states.pop().expect("one frame"))
    }

    /// Preserve the prefix, GAME-level fields, first setup-relevant children
    /// and non-GAME roots. A marker keeps late children outside the initial
    /// record range. Uses the index's validated conventional indentation.
    pub fn read_replay_setup(&mut self) -> io::Result<String> {
        self.validate()?;
        let mut bytes = Vec::new();
        for (i, &(offset, length)) in self.setup.iter().enumerate() {
            let size = usize::try_from(length).map_err(|_| invalid("setup too large"))?;
            bytes
                .try_reserve(size)
                .map_err(|e| io::Error::other(e.to_string()))?;
            let start = bytes.len();
            bytes.resize(start + size, 0);
            self.file.seek(SeekFrom::Start(offset))?;
            self.file.read_exact(&mut bytes[start..])?;
            if i == 0 && !self.frames.is_empty() {
                // The first range is the complete prefix, ending at FRAME.
                bytes.extend_from_slice(b" BEGIN FRAME 0\n");
            }
        }
        self.validate()?;
        String::from_utf8(bytes).map_err(|_| invalid("setup is not UTF-8"))
    }

    /// Build setup with borrowed strings confined to this call. Whole-capture
    /// observations are accumulated from one frame plus its siblings at a time.
    pub fn with_replay_initial<R>(
        &mut self,
        use_initial: impl FnOnce(crate::gamelog::Initial<'_>) -> R,
    ) -> io::Result<R> {
        let text = self.read_replay_setup()?;
        let log = crate::gamelog::Log::parse(&text);
        let mut init = log
            .replay_initial()
            .ok_or_else(|| invalid("no initial state"))?;
        if let Some(cached) = observations::get(&self.path, self.length, self.modified) {
            cached.apply(&mut init);
        } else {
            for i in 0..self.frames.len() {
                let chunk = self.read_frame_and_siblings(i)?;
                // The bounded chunk is immediately traversed in full.
                crate::gamelog::Log::parse_eager(&chunk).append_observations(&mut init, false);
            }
            self.validate()?;
            observations::insert(&self.path, self.length, self.modified, &init);
        }
        self.validate()?;
        Ok(use_initial(init))
    }

    pub fn source_bytes(&self) -> u64 {
        self.length
    }

    /// Read one frame by index (not frame number; duplicate numbers may occur).
    /// Metadata checks catch ordinary edits/truncation; use finalized inputs.
    /// This is safe I/O, not a memory mapping or a claim of atomic snapshots.
    pub fn read_frame(&mut self, index: usize) -> io::Result<String> {
        let range = self
            .frames
            .get(index)
            .ok_or_else(|| invalid("frame index out of range"))?;
        self.read_wrapped(range.offset, range.length)
    }

    /// Last GAME frame and everything following it, wrapped in GAME. Keeping
    /// the frame itself preserves trailing fields and the closing reader's
    /// sibling boundary. No frames yields an empty GAME, hence no final state.
    pub fn read_shutdown(&mut self) -> io::Result<String> {
        match self.frames.last() {
            Some(last) => self.read_wrapped(last.offset, self.length - last.offset),
            None => self.read_wrapped(0, 0),
        }
    }

    /// One frame plus its following GAME siblings, stopping at the next frame.
    /// Used for FULL DUMP classification; duplicate frame numbers remain distinct.
    pub fn read_frame_and_siblings(&mut self, index: usize) -> io::Result<String> {
        let start = self
            .frames
            .get(index)
            .ok_or_else(|| invalid("frame index out of range"))?
            .offset;
        let end = self.frames.get(index + 1).map_or(self.length, |f| f.offset);
        self.read_wrapped(start, end - start)
    }

    fn read_wrapped(&mut self, offset: u64, length: u64) -> io::Result<String> {
        self.check_metadata(self.file.metadata()?)?;
        let size = usize::try_from(length).map_err(|_| invalid("frame too large"))?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(
                size.checked_add(11)
                    .ok_or_else(|| invalid("frame too large"))?,
            )
            .map_err(|e| io::Error::other(e.to_string()))?;
        bytes.extend_from_slice(b"BEGIN GAME\n");
        bytes.resize(size + 11, 0);
        self.file.seek(SeekFrom::Start(offset))?;
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
    fn replay_setup_preserves_late_lookups_fields_and_roots() {
        for text in [
            "BEGIN GAME\n BEGIN LEADERDATA\n  who 0\n BEGIN FRAME 9\n\n  BEGIN CITIES\n   BEGIN CITY\n    who 1\n    o 2000\n leader_flags 7\n BEGIN CITIES\n  BEGIN CITY\n   who 0\n   o 2001\nBEGIN GAME INFO\n BEGIN GAMEINFO\n  map late\n",
            "BEGIN GAME\n BEGIN FRAME 1\n BEGIN FULL DUMP\n  BEGIN WORLD\n   cell_x 4\n  who 0\n  o 2001\n  valid 1\n  farm_type 2\n BEGIN FULL DUMP\n  BEGIN WORLD\n   cell_x 99\n BEGIN FRAME 1\n",
            "BEGIN GAME\n who 0\n o 2001\n BEGIN FRAME 1\n farm_type 2\n",
            "BEGIN GAME\n BEGIN WORLD\n  cell_x 3\n",
        ] {
            let input = Input::new(text);
            let log = crate::gamelog::Log::parse(text);
            let mut expected = log.initial().unwrap();
            expected.frame_bodies.clear();
            for _ in 0..2 {
                IndexedCapture::open(&input.0)
                    .unwrap()
                    .with_replay_initial(|mut actual| {
                        assert_eq!(actual, expected);
                        // A caller's edits must not poison the cached observation set.
                        actual.frame_seeds.push((-1, 123));
                        actual.anim_lengths.clear();
                        actual.frame_guys.clear();
                    })
                    .unwrap();
            }
        }
    }

    #[test]
    fn setup_ranges_skip_frame_bodies_and_survive_cached_reopen() {
        let prefix = "BEGIN GAME\r\n BEGIN WORLD\r\n  value head\r\n";
        let late = " leader_flags 7\r\n BEGIN CITIES\r\n  value late\r\n";
        let tail = "BEGIN GAME INFO\r\n BEGIN GAMEINFO\r\n  value root";
        let text = format!(
            "{prefix} BEGIN FRAME 1\r\n  ignored {}\r\n\r\n{late} BEGIN FRAME 1\r\n  ignored second\r\n{tail}",
            "x".repeat(100_000)
        );
        let input = Input::new(&text);
        let mut first = IndexedCapture::open(&input.0).unwrap();
        let expected = format!("{prefix} BEGIN FRAME 0\n{late}{tail}");
        assert_eq!(first.read_replay_setup().unwrap(), expected);
        assert_eq!(
            first.setup.iter().map(|r| r.1).sum::<u64>() as usize,
            prefix.len() + late.len() + tail.len()
        );
        let mut cached = IndexedCapture::open(&input.0).unwrap();
        assert!(Arc::ptr_eq(&first.setup, &cached.setup));
        assert_eq!(cached.read_replay_setup().unwrap(), expected);
        let entry = Cached {
            path: input.0.clone(),
            length: first.length,
            modified: first.modified,
            frames: Arc::clone(&first.frames),
            setup: Arc::clone(&first.setup),
        };
        assert_eq!(
            entry.bytes(),
            std::mem::size_of_val(first.frames.as_ref())
                + std::mem::size_of_val(first.setup.as_ref())
                + input.0.as_os_str().len()
        );
    }

    #[test]
    fn changed_source_refuses_setup_scan() {
        let input = Input::new("BEGIN GAME\n BEGIN FRAME 1\n");
        let mut source = IndexedCapture::open(&input.0).unwrap();
        source.with_replay_initial(|_| ()).unwrap();
        std::fs::write(&input.0, "BEGIN GAME\n").unwrap();
        assert!(source.with_replay_initial(|_| ()).is_err());
        assert!(source.read_replay_setup().is_err());
        assert!(source.validate().is_err());
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
    fn shutdown_keeps_trailing_flags_but_not_setup_records() {
        let text = "BEGIN GAME\n leader_flags 101\n BEGIN LEADERDATA\n  who 0\n BEGIN FRAME 1\n  value 1\n BEGIN FRAME 2\n  value 2\n leader_flags 202\n BEGIN LEADERDATA\n  who 1\n BEGIN UNITDATA\n  BEGIN OBJECT\n   BEGIN SUBOBJECT\n    flags 1\n    o 9\n    who 1\n    x_internal 234\nGameInfo closing\n";
        let input = Input::new(text);
        let mut source = IndexedCapture::open(&input.0).unwrap();
        let tail = source.read_shutdown().unwrap();
        let state = crate::gamelog::Log::parse(&tail).final_state().unwrap();
        assert_eq!(
            Some(state.clone()),
            crate::gamelog::Log::parse(text).final_state()
        );
        assert_eq!(state.n, 2);
        assert_eq!(state.units[0].o, 9);
        assert_eq!(state.leaders.len(), 1);
        assert_eq!(state.leaders[0].leader_flags, 202);
        assert!(!tail.contains("leader_flags 101"));
        std::fs::write(&input.0, "BEGIN GAME\n").unwrap();
        assert!(source.read_shutdown().is_err());
    }

    #[test]
    fn duplicate_labels_and_full_dump_siblings_keep_their_pairing() {
        let text =
            "BEGIN GAME\n BEGIN FRAME 2\n BEGIN FULL DUMP\n  value 1\n BEGIN FRAME 2\n  value 2\n";
        let input = Input::new(text);
        let mut source = IndexedCapture::open(&input.0).unwrap();
        for (index, count) in [(0, 1), (1, 0)] {
            let slice = source.read_frame_and_siblings(index).unwrap();
            let log = crate::gamelog::Log::parse(&slice);
            assert_eq!(log.frames().len(), 1);
            assert_eq!(log.dumps().len(), count);
        }
        let tail = source.read_shutdown().unwrap();
        assert!(crate::gamelog::Log::parse(&tail).final_state().is_none());
        let empty = Input::new("BEGIN GAME\n value 1\n");
        let tail = IndexedCapture::open(&empty.0)
            .unwrap()
            .read_shutdown()
            .unwrap();
        assert!(crate::gamelog::Log::parse(&tail).final_state().is_none());
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
    fn the_cache_is_bounded_by_bytes_not_by_entries() {
        // Item 1571: a 32-entry bound evicted half the release suite's
        // indexes while every one of them together was 4.0 MB. Forty tiny
        // captures fit the bytes, so the first is still shared.
        let inputs: Vec<Input> = (0..40)
            .map(|i| Input::new(&format!("BEGIN GAME\n BEGIN FRAME {i}\n")))
            .collect();
        let first = IndexedCapture::open(&inputs[0].0).unwrap();
        for input in &inputs[1..] {
            IndexedCapture::open(&input.0).unwrap();
        }
        let again = IndexedCapture::open(&inputs[0].0).unwrap();
        assert!(Arc::ptr_eq(&first.frames, &again.frames));
    }

    #[test]
    fn missing_game_and_invalid_utf8_are_errors() {
        let input = Input::new("BEGIN FRAME 1\n");
        assert!(IndexedCapture::open(&input.0).is_err());
        std::fs::write(&input.0, [0xff, b'\n']).unwrap();
        assert!(IndexedCapture::open(&input.0).is_err());
    }
}
