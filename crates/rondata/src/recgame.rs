//! Reads the original's recorded-game container, `.rcx`.
//!
//! The format is `docs/RECGAME.md`: a gzip stream whose contents are the
//! engine's own `walk_data` serialization — raw little-endian struct ranges,
//! one-byte section markers, and length-prefixed UTF-16 strings — followed by
//! the command-package stream that is the point of the exercise. There is no
//! schema and no framing; the PDB's struct layouts are the format, and every
//! offset here cites the walker that produced it.
//!
//! What comes out: the lobby (seed, options, the eight player slots), the
//! byte spans of the embedded rules tables (a recording carries the loaded
//! `Constants`, the composed 493×493 combat table and the 24 tribes, so it
//! doubles as a rules ground-truth source), and every command package with
//! its frame, issuer, stamp and payload. The payload's own encoding is
//! `CommandManager`'s command format, decoded by [`crate::commands`]
//! (`docs/COMMANDS.md`).

use crate::Error;
use std::io::Read;

/// One of the eight lobby slots (`GameInfo.player[n]`, stride 0x8c).
///
/// The walk writes two bytes of `flags` for every slot and the rest only when
/// bit 0 says the slot is occupied (`GameInfo::walk_data`).
#[derive(Clone, Debug, Default)]
pub struct Slot {
    /// `Player.flags` (+0x30). Bit 0 = active; the sample's human slot holds
    /// 0x0007 and its AI slots 0x2001.
    pub flags: u16,
    /// Nation index into the tribes table (+0x32).
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
#[derive(Clone, Debug)]
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
    /// The command-package stream, running to end of file.
    pub packages: (usize, usize),
}

/// A parsed recording.
#[derive(Clone, Debug, Default)]
pub struct RecGame {
    /// The wrapped version string, e.g. `(Version: 00.2017.08.2100)` —
    /// `Version::get_string` in localized parens, the first thing in the
    /// stream after the two section markers.
    pub version: String,
    /// `GameInfo.version` (+0x00).
    pub info_version: u32,
    /// `GameInfo.seed` (+0x04) — the random-map seed playback regenerates
    /// the world from.
    pub seed: u32,
    /// `GameInfo.flags` (+0x14).
    pub flags: u32,
    /// The lobby options union `GameInfo_u_24`, +0x18..+0x36, as walked:
    /// twenty-nine single bytes and one more.
    pub options: [u8; 30],
    /// All eight lobby slots.
    pub slots: Vec<Slot>,
    /// `info.save_name`. Empty means a random-map recording; non-empty means
    /// a scenario recording, which carries no rules section on modern
    /// patches.
    pub save_name: String,
    /// `Game.frame` at the moment the header was written.
    pub start_frame: i32,
    /// Where the undecoded sections sit.
    pub spans: Spans,
    /// The command packages, in stream order (frames nondecreasing).
    pub packages: Vec<Package>,
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
    c.u8("Game section marker")?;
    c.u8("GameInfo section marker")?;
    rec.version = c.string("version string")?;
    if !rec.version.contains("Version") {
        return Err(c.bad(format!("version string reads {:?}", rec.version)));
    }
    rec.info_version = c.u32("GameInfo.version")?;
    rec.seed = c.u32("GameInfo.seed")?;
    c.raw(12, "GameInfo checksum knobs")?;
    rec.flags = c.u32("GameInfo.flags")?;
    rec.options.copy_from_slice(c.raw(30, "lobby options")?);

    // The eight player records share one marker byte (§3.2); active slots
    // walk Player +0x00..+0x39 raw and then the name.
    let mut slot_marker = None;
    for n in 0..8 {
        let m = c.u8("player section marker")?;
        if *slot_marker.get_or_insert(m) != m {
            return Err(c.bad(format!("player {n} marker {m:#x} breaks the run")));
        }
        let mut s = Slot {
            flags: c.u16("player flags")?,
            ..Slot::default()
        };
        if s.active() {
            let r = c.raw(0x39, "player record")?;
            // +0x30 is the flags field again; the interesting tail follows.
            s.tribe = r[0x32];
            s.who = r[0x33];
            s.team = r[0x34] as i8;
            s.handicap = r[0x35];
            s.play = r[0x36];
            s.diff = r[0x38];
            s.name = c.string("player name")?;
        }
        rec.slots.push(s);
    }

    // The GameInfo tail under sGameSaveVersion >= 0x10, which run_playback
    // forces (§3.3): workshop id, three strings, second id, mod name.
    c.raw(8, "workshop id")?;
    c.string("scenario_script")?;
    c.string("scenario_path")?;
    c.string("mod install dir")?;
    c.raw(8, "second workshop id")?;
    c.string("mod name")?;

    // Back in Game::walk_data (§3.4): frame..balance, the semaphore header
    // and its mask (its own size field says how many bytes), graphic_tick.
    let game = c.raw(0x194, "Game.frame..balance")?;
    rec.start_frame = i32::from_le_bytes([game[0], game[1], game[2], game[3]]);
    c.u32("semaphore bits")?;
    let sem = c.u32("semaphore size")? as usize;
    if sem > 4096 {
        return Err(c.bad(format!("implausible semaphore size {sem}")));
    }
    c.raw(sem, "semaphore mask")?;
    c.u32("graphic_tick")?;
    rec.save_name = c.string("save_name")?;

    // The random-game info blob: one team byte and four leader bytes per
    // occupied slot with a leader, plus the world seed (§4.1). Playback
    // itself reads this into a buffer and frees it unused.
    let seeded = rec.slots.iter().filter(|s| s.active() && s.who < 8).count();
    c.raw(5 * seeded + 4, "random-game info blob")?;

    // The rules section (§4.2): present only for random-map recordings.
    // Types is undecoded, so the tribes train is found by landmark — the
    // earliest nonzero byte repeating at stride 0x599 twenty-four times from
    // which the package stream parses clean to end of file.
    if rec.save_name.is_empty() {
        c.u8("rules section marker")?;
        let rules_start = c.off;
        let mut found = None;
        let limit = data.len().saturating_sub(24 * TRIBE_BLOCK);
        for pos in rules_start..limit {
            let b = data[pos];
            if b == 0 {
                continue;
            }
            if (1..24).all(|k| data[pos + k * TRIBE_BLOCK] == b)
                && packages_parse(data, pos + 24 * TRIBE_BLOCK)
            {
                found = Some(pos);
                break;
            }
        }
        let Some(tribes) = found else {
            return Err(c.bad("no tribe train found; rules section unrecognised"));
        };
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
            packages: (tribes + 24 * TRIBE_BLOCK, data.len()),
        };
        c.off = rec.spans.packages.0;
    } else {
        rec.spans.packages = (c.off, data.len());
    }

    // The package stream (§4.3): fixed 18-byte head then the payload, to end
    // of file. read_package confirms there is no count — EOF is the stop.
    while c.off < data.len() {
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

    Ok(rec)
}

/// Whether a well-formed package stream runs from `off` to exactly end of
/// file: sizes within the 512-byte `CommandPackage` buffer, plays in the
/// eight slots (or the -1 broadcast), frames nondecreasing. This is the
/// validator behind the tribe-train landmark.
pub(crate) fn packages_parse(data: &[u8], mut off: usize) -> bool {
    let mut prev = i32::MIN;
    while off < data.len() {
        let Some(head) = data.get(off..off + 18) else {
            return false;
        };
        let frame = i32::from_le_bytes([head[0], head[1], head[2], head[3]]);
        let play = i32::from_le_bytes([head[4], head[5], head[6], head[7]]);
        let size = u16::from_le_bytes([head[16], head[17]]) as usize;
        if size > 512 || !(-1..=7).contains(&play) || frame < prev {
            return false;
        }
        prev = frame;
        off += 18 + size;
    }
    off == data.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The external sample, if this machine has it: the one explicitly
    /// EE-era recording on `ron.heavengames.com` (fileid 2170, 2018, build
    /// 00.2017.08.2100), kept under the gitignored install directory so it
    /// never enters the repository. A machine without it skips, loudly.
    fn sample() -> Option<String> {
        let root = std::env::var("RON_INSTALL").ok().or_else(|| {
            let here = env!("CARGO_MANIFEST_DIR");
            Some(format!("{here}/../../game"))
        })?;
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
        let root = std::env::var("RON_INSTALL").ok().unwrap_or_else(|| {
            let here = env!("CARGO_MANIFEST_DIR");
            format!("{here}/../../game")
        });
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
}
