//! Reads and writes the original's recorded-game container, `.rcx`.
//!
//! The format is `docs/RECGAME.md`: a gzip stream whose contents are the
//! engine's own `walk_data` serialization — raw little-endian struct ranges,
//! one-byte section markers, and length-prefixed UTF-16 strings — followed by
//! the command-package stream that is the point of the exercise. There is no
//! schema and no framing; the PDB's struct layouts are the format, and every
//! offset here cites the walker that produced it.
//!
//! What comes out: the lobby (seed, options, the eight player slots), the
//! embedded rules tables (a recording carries the loaded `Constants`, the
//! composed 493×493 combat table and the 24 tribes, so it doubles as a rules
//! ground-truth source), and every command package with its frame, issuer,
//! stamp and payload. The payload's own encoding is `CommandManager`'s
//! command format, decoded by [`crate::commands`] (`docs/COMMANDS.md`).
//!
//! [`write`] is the inverse of [`parse`] (`docs/RECGAME.md` §8): every field
//! the reader decodes is re-encoded from the decoded value, and the spans it
//! does not decode — the Types stretch, the `Constants` block, the tribes'
//! bodies, the run-state stretch of `Game` — are carried as bytes. Every
//! recording on disk parses and writes back equal byte for byte.

use crate::Error;
use std::io::Read;

/// One of the eight lobby slots (`GameInfo.player[n]`, stride 0x8c).
///
/// The walk writes two bytes of `flags` for every slot and the rest only when
/// bit 0 says the slot is occupied (`GameInfo::walk_data`). The occupied
/// slot's record is `Player` +0x00..+0x39, field by field below.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Slot {
    /// `Player.flags` (+0x30), as walked on its own ahead of the record.
    /// Bit 0 = active; the sample's human slot holds 0x0007 and its AI slots
    /// 0x2001.
    pub flags: u16,
    /// The ten `synced_*` input counters, +0x00..+0x28, in `Player`'s order
    /// (`synced_used_zoom_control` … `synced_control_groups_activated`).
    pub synced: [i32; 10],
    /// `caravan_frame` (+0x28).
    pub caravan_frame: i32,
    /// `pop_cap_frame` (+0x2c).
    pub pop_cap_frame: i32,
    /// `flags` again, inside the raw record (+0x30): the wire carries it
    /// twice per active slot (`docs/RECGAME.md` §3.2).
    pub record_flags: u16,
    /// Nation index into the tribes table (+0x32); 24 is the lobby's
    /// "random", rolled after the header is written.
    pub tribe: u8,
    /// Leader index (+0x33); `read_random_game_info` treats `who < 8` as "has
    /// a leader seed".
    pub who: u8,
    /// Team number (+0x34, signed).
    pub team: i8,
    /// Handicap (+0x35).
    pub handicap: u8,
    /// The play slot the engine addresses this player by (+0x36) — the value
    /// command packages carry in their `play` field.
    pub play: u8,
    /// `pauses` (+0x37).
    pub pauses: u8,
    /// The per-player `diff` byte (+0x38); 2 in every active sample slot.
    /// The lobby's AI-difficulty knob is a different field —
    /// `GameInfo_u_24_s_0.difficulty` in [`RecGame::options`] (index 7),
    /// 5 = Toughest in the sample.
    pub diff: u8,
    /// `Player.name` (+0x40). Empty for AI slots.
    pub name: String,
}

impl Slot {
    /// Whether the slot is occupied (`flags & 1`, the walker's own gate).
    pub fn active(&self) -> bool {
        self.flags & 1 != 0
    }
}

/// One command package, exactly as `RecordGame::write_package` lays it down:
/// frame, play, valid, stamp, a two-byte size, then the payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Package {
    /// The frame the package is consumed on (`read_package` peeks this and
    /// seeks back if it is still in the future).
    pub frame: i32,
    /// The issuing player slot. In the sample — one human, four AIs — every
    /// package is play 0: in lockstep only real input travels.
    pub play: i32,
    pub valid: i32,
    /// `CommandPackage` +0x00.
    pub stamp: u32,
    /// The command payload (`CommandPackage.data`, at most 512 bytes).
    pub data: Vec<u8>,
}

/// Byte offsets into the decompressed stream, for the pieces this reader
/// locates but does not decode. All half-open, `docs/RECGAME.md` §4.2.
#[derive(Clone, Copy, Debug, Default)]
pub struct Spans {
    /// The undecoded Types stretch (806 virtual walkers, unread).
    pub types: (usize, usize),
    /// The loaded `Constants` block: 0xd40 bytes plus the 4-byte re-walk of
    /// `mongol_three_mil_cavalry`.
    pub constants: (usize, usize),
    /// The composed 493×493 combat table, row-major u16.
    pub balance: (usize, usize),
    /// The 24 `Tribe` records, 0x599 bytes each including the marker.
    pub tribes: (usize, usize),
    /// The command-package stream, running to end of file (or to the start
    /// of [`RecGame::truncated`]).
    pub packages: (usize, usize),
}

/// The one-byte section markers (`DataWalk::walk_test`): the low byte of the
/// section name's case-insensitive hash, observed rather than derived
/// (`docs/RECGAME.md` §2, §5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Markers {
    /// The Game section (`0x16` in every recording on disk).
    pub game: u8,
    /// The GameInfo section (`0x42`).
    pub info: u8,
    /// The player section, shared by all eight slots (`0x50`; the 2017
    /// sample's reads `0x0f`).
    pub player: u8,
}

/// The GameInfo tail under `sGameSaveVersion >= 0x10` (`docs/RECGAME.md`
/// §3.3), which `Game::run_playback` forces.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InfoTail {
    /// The workshop-mod id (u64).
    pub workshop_id: u64,
    pub scenario_script: String,
    pub scenario_path: String,
    /// The resolved scenario/mod directory.
    pub mod_dir: String,
    /// The dropdown-mod id (u64).
    pub dropdown_id: u64,
    /// The dropdown-mod directory.
    pub dropdown_dir: String,
}

/// The rules section (`Game::walk_rules_data`, `docs/RECGAME.md` §4.2),
/// present on random-map recordings.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Rules {
    /// The rules section marker (`0x92`).
    pub marker: u8,
    /// The Types stretch: 806 virtual walkers whose class sequence is the
    /// loader's knowledge, not the file's (§5). **Carried, never parsed.**
    pub types: Vec<u8>,
    /// `Constants` +0x00..+0xd40, the loaded tuning block. **Carried.** The
    /// 4-byte re-walk of +0x804..+0x808 that follows it on the wire is not
    /// stored: [`write`] derives it from this block, so a recording whose
    /// re-walk disagreed with the block would fail the round trip.
    pub constants: Vec<u8>,
    /// The composed 493×493 combat table, row-major, each cell the raw u16
    /// `Balance::walk_rules_data` writes.
    pub balance: Vec<u16>,
    /// The tribe-record marker, shared by all 24 (`0x8f`): the landmark the
    /// reader finds the section's end by.
    pub tribe_marker: u8,
    /// The 24 `Tribe` records' walked bytes, +0x54..+0x6c then
    /// +0x70..+0x5f0 (0x598 each). **Carried.**
    pub tribes: Vec<Vec<u8>>,
}

/// A parsed recording.
#[derive(Clone, Debug, Default)]
pub struct RecGame {
    /// The section markers.
    pub markers: Markers,
    /// The wrapped version string, e.g. `(Version: 00.2017.08.2100)` —
    /// `Version::get_string` in localized parens, the first thing in the
    /// stream after the two section markers.
    pub version: String,
    /// `GameInfo.version` (+0x00).
    pub info_version: u32,
    /// `GameInfo.seed` (+0x04) — the random-map seed playback regenerates
    /// the world from.
    pub seed: u32,
    /// `checksum_deep`, `checksum_window_size`, `checksum_failure_threshold`
    /// (+0x08..+0x14).
    pub checksum: [u32; 3],
    /// `GameInfo.flags` (+0x14).
    pub flags: u32,
    /// The lobby options union `GameInfo_u_24`, +0x18..+0x36, as walked:
    /// twenty-nine single bytes and `mods`.
    pub options: [u8; 30],
    /// All eight lobby slots.
    pub slots: Vec<Slot>,
    /// The GameInfo tail.
    pub tail: InfoTail,
    /// `Game.frame` at the moment the header was written.
    pub start_frame: i32,
    /// `Game` +0x554..+0x6e4: the run-state stretch after `frame`, up to
    /// `balance`. **Carried, never parsed** — 0x190 bytes, identical in
    /// every recording on disk (`docs/RECGAME.md` §8).
    pub run_state: Vec<u8>,
    /// `semaphore.bits` (the BitMask's width; 256).
    pub semaphore_bits: u32,
    /// The semaphore mask bytes; their count is the walked `size`.
    pub semaphore: Vec<u8>,
    /// `Game.graphic_tick` (+0x844): the render clock at record time.
    pub graphic_tick: u32,
    /// `info.save_name`. Empty means a random-map recording; non-empty means
    /// a scenario recording, which carries no rules section on modern
    /// patches.
    pub save_name: String,
    /// `write_random_game_info`'s blob (§4.1): `(team, rolled nation)` for
    /// each active slot with `who < 8`, in slot order.
    pub random_info: Vec<(i8, u32)>,
    /// The blob's closing `World` +0x30 word — the lobby's map style in
    /// every recording on disk.
    pub world_style: u32,
    /// The rules section, when the recording carries one.
    pub rules: Option<Rules>,
    /// Where the undecoded sections sit.
    pub spans: Spans,
    /// The command packages, in stream order (frames nondecreasing).
    pub packages: Vec<Package>,
    /// A recording whose game died before `RecordGame::finalize` stops
    /// wherever stdio last flushed, mid-record: the trailing bytes of the
    /// partial package, carried. Empty for a finished recording.
    pub truncated: Vec<u8>,
}

impl RecGame {
    /// The occupied slots.
    pub fn active(&self) -> impl Iterator<Item = &Slot> {
        self.slots.iter().filter(|s| s.active())
    }
}

/// A cursor over the decompressed stream that turns overruns into [`Error`]s
/// that name the byte.
struct Cur<'a> {
    data: &'a [u8],
    off: usize,
    path: &'a str,
}

impl<'a> Cur<'a> {
    fn bad(&self, what: impl Into<String>) -> Error {
        Error::Format {
            path: self.path.to_string(),
            at: self.off,
            what: what.into(),
        }
    }

    fn raw(&mut self, n: usize, what: &str) -> Result<&'a [u8], Error> {
        let end = self.off.checked_add(n).filter(|&e| e <= self.data.len());
        let Some(end) = end else {
            return Err(self.bad(format!("{what}: {n} bytes past end of stream")));
        };
        let s = &self.data[self.off..end];
        self.off = end;
        Ok(s)
    }

    fn u8(&mut self, what: &str) -> Result<u8, Error> {
        Ok(self.raw(1, what)?[0])
    }

    fn u16(&mut self, what: &str) -> Result<u16, Error> {
        let b = self.raw(2, what)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    fn u32(&mut self, what: &str) -> Result<u32, Error> {
        let b = self.raw(4, what)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn u64(&mut self, what: &str) -> Result<u64, Error> {
        let b = self.raw(8, what)?;
        Ok(u64::from_le_bytes(b.try_into().expect("8 bytes")))
    }

    fn i32(&mut self, what: &str) -> Result<i32, Error> {
        Ok(self.u32(what)? as i32)
    }

    /// `String::walk_data`: a u32 character count then UTF-16LE code units,
    /// no terminator, count 0 = empty.
    fn string(&mut self, what: &str) -> Result<String, Error> {
        let n = self.u32(what)? as usize;
        // Nothing walked here is a paragraph; a huge count means the cursor
        // is misaligned, and saying so beats allocating gigabytes.
        if n > 4096 {
            return Err(self.bad(format!("{what}: implausible string length {n}")));
        }
        let b = self.raw(2 * n, what)?;
        let units: Vec<u16> = b
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16(&units).map_err(|_| self.bad(format!("{what}: not UTF-16")))
    }
}

/// `Tribe` records are 0x18 + 0x580 walked bytes behind a marker.
const TRIBE_BLOCK: usize = 1 + 0x18 + 0x580;
/// The composed combat table: 493 × 493 u16 (`Balance::walk_rules_data`).
const BALANCE_BYTES: usize = 493 * 493 * 2;
/// The loaded `Constants`: +0x00..+0xd40 then the 4-byte re-walk.
const CONSTANTS_BYTES: usize = 0xd40 + 4;
/// The re-walked `Constants` field, `mongol_three_mil_cavalry`.
const CONSTANTS_REWALK: std::ops::Range<usize> = 0x804..0x808;
/// `Game` +0x550..+0x6e4, `frame` first.
const RUN_STATE_BYTES: usize = 0x194;

/// Reads and parses a `.rcx` recording.
pub fn read(path: &str) -> Result<RecGame, Error> {
    parse(&decompress(path)?, path)
}

/// Reads a `.rcx` to the raw walked stream, for callers that want to slice
/// the [`Spans`] a parse hands back — the embedded rules tables live there.
///
/// The game records uncompressed and gzips once in `RecordGame::finalize`
/// (`docs/RECGAME.md` §1), so a recording whose game died before finalize is
/// the same record with no gzip header — sniffed rather than assumed.
pub fn decompress(path: &str) -> Result<Vec<u8>, Error> {
    let io = |source| Error::Io {
        path: path.to_string(),
        source,
    };
    let raw = std::fs::read(path).map_err(io)?;
    if !raw.starts_with(&[0x1f, 0x8b]) {
        return Ok(raw);
    }
    let mut data = Vec::new();
    flate2::read::GzDecoder::new(&raw[..])
        .read_to_end(&mut data)
        .map_err(io)?;
    Ok(data)
}

/// Parses an already-decompressed stream. Split out so tests can feed bytes.
pub fn parse(data: &[u8], path: &str) -> Result<RecGame, Error> {
    let mut c = Cur { data, off: 0, path };
    let mut rec = RecGame::default();

    // Game::walk_data then GameInfo::walk_data: two section markers, the
    // version string, and GameInfo's fixed front (docs/RECGAME.md §3.1).
    rec.markers.game = c.u8("Game section marker")?;
    rec.markers.info = c.u8("GameInfo section marker")?;
    rec.version = c.string("version string")?;
    if !rec.version.contains("Version") {
        return Err(c.bad(format!("version string reads {:?}", rec.version)));
    }
    rec.info_version = c.u32("GameInfo.version")?;
    rec.seed = c.u32("GameInfo.seed")?;
    for k in &mut rec.checksum {
        *k = c.u32("GameInfo checksum knobs")?;
    }
    rec.flags = c.u32("GameInfo.flags")?;
    rec.options.copy_from_slice(c.raw(30, "lobby options")?);

    // The eight player records share one marker byte (§3.2); active slots
    // walk Player +0x00..+0x39 raw and then the name.
    for n in 0..8 {
        let m = c.u8("player section marker")?;
        if n == 0 {
            rec.markers.player = m;
        } else if rec.markers.player != m {
            return Err(c.bad(format!("player {n} marker {m:#x} breaks the run")));
        }
        let mut s = Slot {
            flags: c.u16("player flags")?,
            ..Slot::default()
        };
        if s.active() {
            let r = c.raw(0x39, "player record")?;
            let word = |at: usize| i32::from_le_bytes([r[at], r[at + 1], r[at + 2], r[at + 3]]);
            for (k, v) in s.synced.iter_mut().enumerate() {
                *v = word(4 * k);
            }
            s.caravan_frame = word(0x28);
            s.pop_cap_frame = word(0x2c);
            s.record_flags = u16::from_le_bytes([r[0x30], r[0x31]]);
            s.tribe = r[0x32];
            s.who = r[0x33];
            s.team = r[0x34] as i8;
            s.handicap = r[0x35];
            s.play = r[0x36];
            s.pauses = r[0x37];
            s.diff = r[0x38];
            s.name = c.string("player name")?;
        }
        rec.slots.push(s);
    }

    // The GameInfo tail under sGameSaveVersion >= 0x10, which run_playback
    // forces (§3.3): workshop id, three strings, second id, mod name.
    rec.tail = InfoTail {
        workshop_id: c.u64("workshop id")?,
        scenario_script: c.string("scenario_script")?,
        scenario_path: c.string("scenario_path")?,
        mod_dir: c.string("mod install dir")?,
        dropdown_id: c.u64("second workshop id")?,
        dropdown_dir: c.string("mod name")?,
    };

    // Back in Game::walk_data (§3.4): frame..balance, the semaphore header
    // and its mask (its own size field says how many bytes), graphic_tick.
    let game = c.raw(RUN_STATE_BYTES, "Game.frame..balance")?;
    rec.start_frame = i32::from_le_bytes([game[0], game[1], game[2], game[3]]);
    rec.run_state = game[4..].to_vec();
    rec.semaphore_bits = c.u32("semaphore bits")?;
    let sem = c.u32("semaphore size")? as usize;
    if sem > 4096 {
        return Err(c.bad(format!("implausible semaphore size {sem}")));
    }
    rec.semaphore = c.raw(sem, "semaphore mask")?.to_vec();
    rec.graphic_tick = c.u32("graphic_tick")?;
    rec.save_name = c.string("save_name")?;

    // The random-game info blob: one team byte and four leader bytes per
    // occupied slot with a leader, plus World +0x30 (§4.1). Playback itself
    // reads this into a buffer and frees it unused.
    let seeded = rec.slots.iter().filter(|s| s.active() && s.who < 8).count();
    for _ in 0..seeded {
        let team = c.u8("random-info team")? as i8;
        rec.random_info.push((team, c.u32("random-info nation")?));
    }
    rec.world_style = c.u32("random-info World +0x30")?;

    // The rules section (§4.2): present only for random-map recordings.
    // Types is undecoded, so the tribes train is found by landmark — the
    // earliest nonzero byte repeating at stride 0x599 twenty-four times from
    // which the package stream parses clean to end of file. A recording
    // whose game died before finalize ends mid-record; only when no
    // candidate parses clean is a truncated tail accepted (§8).
    let mut packages_end = data.len();
    if rec.save_name.is_empty() {
        let marker = c.u8("rules section marker")?;
        let rules_start = c.off;
        let limit = data.len().saturating_sub(24 * TRIBE_BLOCK);
        let train = |pos: usize| {
            let b = data[pos];
            b != 0 && (1..24).all(|k| data[pos + k * TRIBE_BLOCK] == b)
        };
        let mut found = (rules_start..limit)
            .find(|&pos| train(pos) && packages_parse(data, pos + 24 * TRIBE_BLOCK))
            .map(|pos| (pos, data.len()));
        if found.is_none() {
            found = (rules_start..limit).find_map(|pos| {
                if !train(pos) {
                    return None;
                }
                packages_parse_truncated(data, pos + 24 * TRIBE_BLOCK).map(|end| (pos, end))
            });
        }
        let Some((tribes, end)) = found else {
            return Err(c.bad("no tribe train found; rules section unrecognised"));
        };
        packages_end = end;
        let balance = tribes - BALANCE_BYTES;
        let constants = balance - CONSTANTS_BYTES;
        if constants < rules_start {
            return Err(c.bad(format!(
                "tribe train at {tribes:#x} leaves no room for constants+balance"
            )));
        }
        rec.spans = Spans {
            types: (rules_start, constants),
            constants: (constants, balance),
            balance: (balance, tribes),
            tribes: (tribes, tribes + 24 * TRIBE_BLOCK),
            packages: (tribes + 24 * TRIBE_BLOCK, end),
        };
        rec.rules = Some(Rules {
            marker,
            types: data[rules_start..constants].to_vec(),
            constants: data[constants..constants + 0xd40].to_vec(),
            balance: data[balance..tribes]
                .chunks_exact(2)
                .map(|b| u16::from_le_bytes([b[0], b[1]]))
                .collect(),
            tribe_marker: data[tribes],
            tribes: (0..24)
                .map(|k| {
                    data[tribes + k * TRIBE_BLOCK + 1..tribes + (k + 1) * TRIBE_BLOCK].to_vec()
                })
                .collect(),
        });
        c.off = rec.spans.packages.0;
    } else {
        rec.spans.packages = (c.off, data.len());
    }

    // The package stream (§4.3): fixed 18-byte head then the payload, to end
    // of file. read_package confirms there is no count — EOF is the stop.
    while c.off < packages_end {
        let frame = c.i32("package frame")?;
        let play = c.i32("package play")?;
        let valid = c.i32("package valid")?;
        let stamp = c.u32("package stamp")?;
        let size = c.u16("package size")? as usize;
        if size > 512 {
            return Err(c.bad(format!("package size {size} exceeds the 512-byte buffer")));
        }
        let data = c.raw(size, "package data")?.to_vec();
        if let Some(prev) = rec.packages.last()
            && frame < prev.frame
        {
            return Err(c.bad(format!("package frame {frame} after {}", prev.frame)));
        }
        rec.packages.push(Package {
            frame,
            play,
            valid,
            stamp,
            data,
        });
    }
    rec.truncated = data[packages_end..].to_vec();

    Ok(rec)
}

/// Whether a well-formed package stream runs from `off` to exactly end of
/// file: sizes within the 512-byte `CommandPackage` buffer, plays in the
/// eight slots (or the -1 broadcast), frames nondecreasing. This is the
/// validator behind the tribe-train landmark.
pub(crate) fn packages_parse(data: &[u8], off: usize) -> bool {
    clean_prefix(data, off) == Some(data.len())
}

/// The well-formed packages from `off`, and where they stop: `Some(end)`
/// when what follows `end` is nothing or a well-formed *partial* record —
/// a head cut short, or a head whose payload runs past end of file. The
/// stream a game killed before `RecordGame::finalize` leaves behind.
pub(crate) fn packages_parse_truncated(data: &[u8], off: usize) -> Option<usize> {
    let end = clean_prefix(data, off)?;
    let rest = &data[end..];
    if rest.len() < 18 {
        return Some(end);
    }
    let play = i32::from_le_bytes([rest[4], rest[5], rest[6], rest[7]]);
    let size = u16::from_le_bytes([rest[16], rest[17]]) as usize;
    ((-1..=7).contains(&play) && size <= 512 && rest.len() < 18 + size).then_some(end)
}

/// Walks well-formed packages from `off` and returns the offset where the
/// last whole one ends, or `None` if a malformed head stands before end of
/// file. A record cut short by end of file stops the walk there.
fn clean_prefix(data: &[u8], mut off: usize) -> Option<usize> {
    let mut prev = i32::MIN;
    while off < data.len() {
        let Some(head) = data.get(off..off + 18) else {
            return Some(off);
        };
        let frame = i32::from_le_bytes([head[0], head[1], head[2], head[3]]);
        let play = i32::from_le_bytes([head[4], head[5], head[6], head[7]]);
        let size = u16::from_le_bytes([head[16], head[17]]) as usize;
        if size > 512 || !(-1..=7).contains(&play) || frame < prev {
            return None;
        }
        if off + 18 + size > data.len() {
            return Some(off);
        }
        prev = frame;
        off += 18 + size;
    }
    Some(off)
}

/// Writes a recording back to the decompressed stream `parse` reads — every
/// decoded field from its decoded value, every carried span as its bytes
/// (`docs/RECGAME.md` §8). `parse(&write(r))` gives `r` back, and over every
/// recording on disk `write(&parse(d)) == d`.
pub fn write(rec: &RecGame) -> Vec<u8> {
    fn string(o: &mut Vec<u8>, s: &str) {
        let units: Vec<u16> = s.encode_utf16().collect();
        o.extend((units.len() as u32).to_le_bytes());
        for u in units {
            o.extend(u.to_le_bytes());
        }
    }
    let mut o = Vec::new();
    o.extend([rec.markers.game, rec.markers.info]);
    string(&mut o, &rec.version);
    o.extend(rec.info_version.to_le_bytes());
    o.extend(rec.seed.to_le_bytes());
    for k in rec.checksum {
        o.extend(k.to_le_bytes());
    }
    o.extend(rec.flags.to_le_bytes());
    o.extend(rec.options);
    for s in &rec.slots {
        o.push(rec.markers.player);
        o.extend(s.flags.to_le_bytes());
        if s.active() {
            for v in s.synced {
                o.extend(v.to_le_bytes());
            }
            o.extend(s.caravan_frame.to_le_bytes());
            o.extend(s.pop_cap_frame.to_le_bytes());
            o.extend(s.record_flags.to_le_bytes());
            o.extend([
                s.tribe,
                s.who,
                s.team as u8,
                s.handicap,
                s.play,
                s.pauses,
                s.diff,
            ]);
            string(&mut o, &s.name);
        }
    }
    o.extend(rec.tail.workshop_id.to_le_bytes());
    string(&mut o, &rec.tail.scenario_script);
    string(&mut o, &rec.tail.scenario_path);
    string(&mut o, &rec.tail.mod_dir);
    o.extend(rec.tail.dropdown_id.to_le_bytes());
    string(&mut o, &rec.tail.dropdown_dir);
    o.extend(rec.start_frame.to_le_bytes());
    o.extend(&rec.run_state);
    o.extend(rec.semaphore_bits.to_le_bytes());
    o.extend((rec.semaphore.len() as u32).to_le_bytes());
    o.extend(&rec.semaphore);
    o.extend(rec.graphic_tick.to_le_bytes());
    string(&mut o, &rec.save_name);
    for &(team, nation) in &rec.random_info {
        o.push(team as u8);
        o.extend(nation.to_le_bytes());
    }
    o.extend(rec.world_style.to_le_bytes());
    if let Some(r) = &rec.rules {
        o.push(r.marker);
        o.extend(&r.types);
        o.extend(&r.constants);
        o.extend(r.constants.get(CONSTANTS_REWALK).unwrap_or(&[0; 4]));
        for v in &r.balance {
            o.extend(v.to_le_bytes());
        }
        for t in &r.tribes {
            o.push(r.tribe_marker);
            o.extend(t);
        }
    }
    for p in &rec.packages {
        o.extend(p.frame.to_le_bytes());
        o.extend(p.play.to_le_bytes());
        o.extend(p.valid.to_le_bytes());
        o.extend(p.stamp.to_le_bytes());
        o.extend((p.data.len() as u16).to_le_bytes());
        o.extend(&p.data);
    }
    o.extend(&rec.truncated);
    o
}

/// [`write`], gzipped the way `RecordGame::finalize` leaves a finished
/// recording (`docs/RECGAME.md` §1) — the bytes of a `.rcx` file.
pub fn write_gz(rec: &RecGame) -> Vec<u8> {
    use std::io::Write as _;
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    gz.write_all(&write(rec))
        .expect("writing to a Vec cannot fail");
    gz.finish().expect("writing to a Vec cannot fail")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The external sample, if this machine has it: the one explicitly
    /// EE-era recording on `ron.heavengames.com` (fileid 2170, 2018, build
    /// 00.2017.08.2100), kept under the gitignored install directory so it
    /// never enters the repository. A machine without it skips, loudly.
    fn sample() -> Option<String> {
        let root = crate::testenv::install_root()?;
        let dir = std::path::Path::new(&root).join("external-recgames");
        let rcx = std::fs::read_dir(dir).ok()?.find_map(|e| {
            let p = e.ok()?.path();
            (p.extension()? == "rcx").then(|| p.display().to_string())
        });
        if rcx.is_none() {
            eprintln!("no .rcx under the install's external-recgames; skipping");
        }
        rcx
    }

    /// Every fact here was first established by hand against the hexdump
    /// (docs/RECGAME.md) — the test pins the reader to them.
    #[test]
    fn the_heavengames_sample_parses_end_to_end() {
        let Some(path) = sample() else { return };
        let rec = read(&path).expect("the sample should parse");

        assert_eq!(rec.version, "(Version: 00.2017.08.2100)");
        assert_eq!(rec.seed, 340513920);
        assert_eq!(rec.save_name, "");
        assert_eq!(rec.start_frame, 0);

        // One human and four Toughest AIs, teams 0 vs 1.
        let active: Vec<_> = rec.active().collect();
        assert_eq!(active.len(), 5);
        assert_eq!(active[0].name, "Seifer008");
        assert_eq!(active[0].team, 0);
        assert!(active[1..].iter().all(|s| s.name.is_empty()));
        assert!(active[1..].iter().all(|s| s.team == 1));
        assert!(active[1..].iter().all(|s| s.diff == 2));

        // The command stream: only the human's input travels in lockstep.
        assert_eq!(rec.packages.len(), 21884);
        assert!(rec.packages.iter().all(|p| p.play == 0));
        assert_eq!(rec.packages.first().unwrap().frame, 0);
        assert_eq!(rec.packages.last().unwrap().frame, 21044);
        // Stamps are a contiguous 1..N sequence and only the first record
        // carries valid = 1 (the blind second reading's observation).
        assert!(
            rec.packages
                .iter()
                .enumerate()
                .all(|(i, p)| p.stamp as usize == i + 1)
        );
        assert!(
            rec.packages
                .iter()
                .enumerate()
                .all(|(i, p)| (p.valid == 1) == (i == 0))
        );
        // The lobby options decode: difficulty (index 7) is 5 = Toughest,
        // exactly what the uploader promised.
        assert_eq!(rec.options[7], 5);

        // Replay-oracle preflight (2026-10-15): this external SP recording
        // has no embedded sync commands. A successful playback therefore
        // cannot be certified against recording checksums that do not exist.
        let commands: Vec<_> = rec
            .packages
            .iter()
            .flat_map(|p| crate::commands::decode(&p.data).unwrap())
            .collect();
        assert_eq!(commands.len(), 25_779);
        assert_eq!(
            commands
                .iter()
                .filter(|c| matches!(
                    c,
                    crate::commands::Command::CheckRandom { .. }
                        | crate::commands::Command::CheckSums { .. }
                        | crate::commands::Command::NextCheckSum { .. }
                ))
                .count(),
            0
        );

        // The landmark put the embedded rules where the walkers say they
        // are: tribes end where packages begin, balance is the 493x493
        // table, constants the 0xd44 before it.
        assert_eq!(rec.spans.tribes.1, rec.spans.packages.0);
        assert_eq!(rec.spans.balance.1 - rec.spans.balance.0, BALANCE_BYTES);
        assert_eq!(
            rec.spans.constants.1 - rec.spans.constants.0,
            CONSTANTS_BYTES
        );
        assert_eq!(rec.spans.tribes.0, 0xf1e51);
    }

    /// The recording embeds the loaded 493×493 combat table
    /// (`Balance::walk_rules_data`), and it equals the one the loader
    /// composes from this install cell for cell — across a seven-year build
    /// gap. This is the same guard `rondata --types` gets from a `DUMP_ALL`
    /// dump, from a source that does not hang the game; it also proves the
    /// tribe-train landmark found the true span, since a misplaced span
    /// would shred the comparison.
    #[test]
    fn the_embedded_combat_table_is_the_composed_one() {
        let Some(path) = sample() else { return };
        let Some(root) = crate::testenv::install_root() else {
            return;
        };
        let data = decompress(&path).unwrap();
        let rec = parse(&data, &path).unwrap();
        let loaded = crate::load::load(&crate::Install::new(&root)).unwrap();
        let n = loaded.kinds.len();
        let side = n + loaded.build_kinds.len();
        let table = &data[rec.spans.balance.0..rec.spans.balance.1];
        assert_eq!(table.len(), 2 * side * side);
        let mut mismatches = 0;
        for a in 0..side {
            for b in 0..side {
                let cell = 2 * (a * side + b);
                let want = i32::from(i16::from_le_bytes([table[cell], table[cell + 1]]));
                let at = |i: usize| {
                    if i < n {
                        sim::combat::TypeRef::Unit(i)
                    } else {
                        sim::combat::TypeRef::Build(i - n)
                    }
                };
                if want != loaded.table.pct_of(at(a), at(b)) {
                    mismatches += 1;
                }
            }
        }
        assert_eq!(mismatches, 0);
    }

    #[test]
    fn garbage_is_refused_with_an_offset() {
        let err = parse(&[0u8; 64], "x").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("at byte"), "{msg}");
    }

    #[test]
    fn a_truncated_stream_is_refused() {
        // A plausible opening (two markers, a version string) that ends
        // mid-header.
        let mut d = vec![0x16, 0x42];
        let s: Vec<u16> = "(Version: test)".encode_utf16().collect();
        d.extend((s.len() as u32).to_le_bytes());
        for u in s {
            d.extend(u.to_le_bytes());
        }
        assert!(parse(&d, "x").is_err());
    }

    #[test]
    fn package_validation_requires_exact_eof() {
        // frame 0, play 0, valid 0, stamp 0, size 1, one byte: 19 bytes.
        let mut d = Vec::new();
        d.extend(0i32.to_le_bytes());
        d.extend(0i32.to_le_bytes());
        d.extend(0i32.to_le_bytes());
        d.extend(0u32.to_le_bytes());
        d.extend(1u16.to_le_bytes());
        d.push(0xaa);
        assert!(packages_parse(&d, 0));
        // One trailing byte breaks the exact-EOF contract.
        d.push(0);
        assert!(!packages_parse(&d, 0));
        // A play outside -1..=7 is not a package.
        let mut bad = d.clone();
        bad.truncate(19);
        bad[4] = 9;
        assert!(!packages_parse(&bad, 0));
    }

    /// **Item 4058's first score**: recordings on disk that parse and write
    /// back equal byte for byte, payloads through the command encoder.
    pub(crate) const RECORDINGS_ROUND_TRIPPED: usize = 50;
    /// The denominator, measured: this install's 49 kept recordings in the
    /// bottle's `Logs\` and the 2017 sample (`docs/RECGAME.md` §8).
    pub(crate) const RECORDINGS_ON_DISK: usize = 50;

    /// Every `.rcx` this machine keeps where the tests may look without a
    /// consent dialog: `$RON_RECGAME_DIR`, else the dumps' directory
    /// (`$RON_GAMELOG_DIR` or the bottle's `Logs\`), and the install's
    /// `external-recgames/`. Never `~/Documents`, which is consent-gated.
    pub(crate) fn recordings_on_disk() -> Vec<String> {
        let home = std::env::var("HOME").unwrap_or_default();
        let logs = std::env::var("RON_RECGAME_DIR")
            .or_else(|_| std::env::var("RON_GAMELOG_DIR"))
            .unwrap_or_else(|_| {
                format!("{home}/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs")
            });
        let mut dirs = vec![std::path::PathBuf::from(logs)];
        if let Some(root) = crate::testenv::install_root() {
            dirs.push(std::path::Path::new(&root).join("external-recgames"));
        }
        let mut out: Vec<String> = dirs
            .iter()
            .filter_map(|d| std::fs::read_dir(d).ok())
            .flatten()
            .filter_map(|e| {
                let p = e.ok()?.path();
                (p.extension()? == "rcx").then(|| p.display().to_string())
            })
            .collect();
        out.sort();
        out
    }

    /// One recording through the round trip: decompress, parse, re-encode
    /// every payload from its decoded commands, write, compare. `Err` names
    /// the first byte where the two streams part, or the parse's own error.
    fn round_trip(path: &str) -> Result<(usize, usize), String> {
        let data = decompress(path).map_err(|e| e.to_string())?;
        let mut rec = parse(&data, path).map_err(|e| e.to_string())?;
        let mut commands = 0;
        for p in &mut rec.packages {
            let cmds = crate::commands::decode(&p.data)
                .map_err(|e| format!("frame {} stamp {}: {e}", p.frame, p.stamp))?;
            commands += cmds.len();
            p.data = crate::commands::encode(&cmds);
        }
        let out = write(&rec);
        if out != data {
            let at = out
                .iter()
                .zip(&data)
                .position(|(a, b)| a != b)
                .unwrap_or(out.len().min(data.len()));
            return Err(format!(
                "parts at byte {at:#x} of {:#x} (ours {:#x}): ours {:02x?}, theirs {:02x?}",
                data.len(),
                out.len(),
                &out[at..(at + 8).min(out.len())],
                &data[at..(at + 8).min(data.len())]
            ));
        }
        Ok((rec.packages.len(), commands))
    }

    /// **Every recording on disk, parsed and written back byte for byte**
    /// (item 4058, `docs/RECGAME.md` §8) — the header from its decoded
    /// fields, the rules' combat table from its cells, every package's head
    /// from its fields and its payload through [`crate::commands::encode`];
    /// only the spans §8 names as carried ride as bytes. Install-gated, and
    /// loud when the corpus is absent.
    #[test]
    fn every_recording_on_disk_round_trips_byte_for_byte() {
        let files = recordings_on_disk();
        if files.is_empty() {
            eprintln!("skipping: no .rcx on disk (set RON_RECGAME_DIR / RON_INSTALL)");
            return;
        }
        let mut ok = 0;
        let mut failed = Vec::new();
        let (mut packages, mut commands) = (0, 0);
        for f in &files {
            match round_trip(f) {
                Ok((p, c)) => {
                    ok += 1;
                    packages += p;
                    commands += c;
                }
                Err(e) => failed.push(format!("{f}: {e}")),
            }
        }
        eprintln!(
            "ROUND TRIP: {ok} of {} recordings, {packages} packages, {commands} commands",
            files.len()
        );
        assert_eq!(files.len(), RECORDINGS_ON_DISK, "the corpus moved");
        assert_eq!(ok, RECORDINGS_ROUND_TRIPPED, "{failed:#?}");
    }

    /// The two recordings whose games died before `finalize` (uncompressed,
    /// cut mid-record) parse, and say where they stop.
    #[test]
    fn a_recording_cut_before_finalize_keeps_its_partial_record() {
        let files = recordings_on_disk();
        let cut: Vec<(String, usize, usize)> = files
            .iter()
            .filter_map(|f| {
                let rec = read(f).ok()?;
                (!rec.truncated.is_empty()).then(|| {
                    let name = f.rsplit('/').next().unwrap_or(f).to_string();
                    (name, rec.packages.len(), rec.truncated.len())
                })
            })
            .collect();
        if files.is_empty() {
            eprintln!("skipping: no .rcx on disk");
            return;
        }
        assert_eq!(
            cut,
            vec![
                (
                    "Playback - 2026.08.20 01'34'39 (Thu).rcx".to_string(),
                    834,
                    21
                ),
                (
                    "Playback - 2026.08.24 17'20'55 (Mon).rcx".to_string(),
                    3902,
                    1
                ),
            ]
        );
    }

    /// A built recording writes and parses back to itself, and a cut tail
    /// survives the trip.
    #[test]
    fn a_built_recording_parses_back_to_itself() {
        let mut rec = RecGame {
            markers: Markers {
                game: 0x16,
                info: 0x42,
                player: 0x50,
            },
            version: "(Version: test)".into(),
            seed: 12345,
            checksum: [0, 256, 0],
            run_state: vec![0; RUN_STATE_BYTES - 4],
            semaphore_bits: 256,
            semaphore: vec![0; 32],
            ..RecGame::default()
        };
        // Only an active slot's record is walked, so an inactive one is
        // its flags alone.
        rec.slots = (0..8)
            .map(|n| match n {
                0 | 1 => Slot {
                    flags: 1,
                    record_flags: 1,
                    who: n,
                    play: n,
                    team: 8,
                    name: if n == 0 {
                        "Player 1".into()
                    } else {
                        String::new()
                    },
                    ..Slot::default()
                },
                _ => Slot::default(),
            })
            .collect();
        rec.random_info = vec![(8, 4), (8, 11)];
        rec.world_style = 14;
        rec.packages = vec![Package {
            frame: 0,
            play: 0,
            valid: 1,
            stamp: 1,
            data: crate::commands::encode(&[crate::commands::Command::Pause { state: 1 }]),
        }];
        // No rules: a save-name recording carries none, and parses without
        // the landmark.
        rec.save_name = "x".into();
        let bytes = write(&rec);
        let back = parse(&bytes, "built").unwrap();
        assert_eq!(write(&back), bytes);
        assert_eq!(back.slots, rec.slots);
        assert_eq!(back.packages, rec.packages);
        assert_eq!(back.random_info, rec.random_info);
    }
}
