//! **The original's sync checksum record, read whole** (item 4060,
//! `docs/CHECKSUM.md` §2).
//!
//! `GameLog::say_checksum@00930b30` prints a `CHECKSUM n` record: its
//! number, the `FILE` and `LINE` of the call, then one labelled Adler-32
//! per walked subsystem, as many as the effective level asks — 67 lines in
//! all at the level a `DUMP_ALL` frame forces. [`crate::gamelog::Log::frame_seeds`]
//! reads the record's `game_random seed` alone; this module reads every
//! labelled line, both records of a frame, and says which hook printed
//! each.
//!
//! It reads lines rather than the block tree. A frame's end record sits
//! beside its frame block, not under it, and a window the tracer opens
//! mid-frame prints that record with no `BEGIN FULL DUMP` above it
//! (run1100's record 3502), so the tree has no one place for it. The
//! labels are matched in print order, so a line that merely looks like
//! `label value` past the record's last line is not taken into it.
//!
//! This file is not one of `diff::oracle`'s `SCHEMA_SOURCES`: editing it
//! writes no new oracle-cache generation.

use std::io::BufRead;

/// Every label the record prints, in print order (`docs/CHECKSUM.md` §2):
/// the eleven subsystem lines with the sixteen `World` sections between
/// them, then `Rules`, the seed, `Game`, and the 34 lines of level 16. A
/// record at level L prints a prefix of this list.
pub const RECORD_LABELS: [&str; 64] = [
    "Ammo",
    "Deaths",
    "Groups",
    "Leaders",
    "Cities",
    "Items",
    "Goods",
    "World",
    "World",
    "World",
    "World",
    "World",
    "World",
    "World",
    "World",
    "World",
    "World",
    "World",
    "World",
    "World",
    "World",
    "World",
    "World",
    "Walls",
    "Guys",
    "Units",
    "Builds",
    "Rules",
    "game_random seed",
    "Game",
    "tribes",
    "mountains",
    "constants",
    "armies",
    "forms",
    "heroes",
    "herds",
    "specials",
    "wonders",
    "forts",
    "docks",
    "oil_wells",
    "supplies",
    "caravans",
    "lands",
    "leader_options",
    "option_info",
    "pathfinder",
    "hotkey_groups",
    "game_daemon",
    "graphic_events",
    "searcher",
    "scene",
    "message_win",
    "storm",
    "farms",
    "unbuilt_wonders",
    "unbuilt_cities",
    "unbuilt_forts",
    "regions",
    "achieve",
    "scenario_data",
    "script_run_time",
    "turn_control",
];

/// The fifteen subsystems of the sync checksum
/// (`CheckSums::check_all@00936560`), in its order: check_all's own name,
/// the record's label, and whether the line is the simulation's state —
/// the twelve step 5 counts (`docs/DECISIONS.md` entry 119). The record's
/// `World` is its sixteenth `World` line, the whole world.
pub const CHECKSUM_LINES: [(&str, &str, bool); 15] = [
    ("units", "Units", true),
    ("builds", "Builds", true),
    ("walls", "Walls", true),
    ("ammo", "Ammo", true),
    ("deaths", "Deaths", true),
    ("groups", "Groups", true),
    ("guys", "Guys", true),
    ("leaders", "Leaders", true),
    ("cities", "Cities", true),
    ("items", "Items", true),
    ("goods", "Goods", true),
    ("world", "World", true),
    ("rules", "Rules", false),
    ("scenario_data", "scenario_data", false),
    ("script_run_time", "script_run_time", false),
];

/// **The instrument's score** (`docs/CHECKSUM.md` §5): of the fifteen
/// lines, how many `tools/recomp/checksum.py` reproduces from the packet
/// of [`CHECKSUM_PACKET`], held against the record the original printed
/// on that frame. Re-pinned by the item that moves it; the queue's
/// `Checksum:` line is the driver's.
pub const CHECKSUM_REPRODUCED: (usize, usize) = (15, 15);

/// The run and logger frame the instrument is proven on, and the record it
/// reproduces: run 1562's packet (`GameLog::end_frame` entry of logger
/// frame 91, the minted game's playback) and record 3201, the first of its
/// `@dumpall` window (`docs/CHECKSUM.md` §3). Item 4060 proved it first on
/// run1100's 5408 and record 3502.
pub const CHECKSUM_PACKET: (&str, i64, u64) = ("run1562", 91, 3201);

/// **The port's score**: of the twelve state lines, how many our
/// simulation emits equal to the original's on that frame, and of the
/// bytes the original's twelve walks hand over there (the ledger,
/// `docs/CHECKSUM.md` §4), how many ours reproduces, on
/// [`CHECKSUM_PACKET`]'s frame. Item 4088: items, 484 of run 1562's
/// 425,499 (`diff::checksum_lines`). Item 4091: cities, 228 more; goods
/// is held out on good 15's `ever_seen` (`docs/CHECKSUM.md` §13).
pub const CHECKSUM_STATE: (usize, usize, u64, u64) = (2, 12, 712, 425_499);

/// Which logger hook printed a record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hook {
    /// A setup record (`FILE` another than `gamelog.cpp`): mapgen, init.
    Setup,
    /// `GameLog::begin_game`'s, before the first frame block.
    Game,
    /// `GameLog::begin_frame`'s for frame n: the first child of its block.
    Begin(i64),
    /// `GameLog::end_frame`'s for frame n: after the frame's tick.
    End(i64),
}

/// One `CHECKSUM n` record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    /// The record number (`checksum_count`).
    pub number: u64,
    /// The `FILE` of the call.
    pub file: String,
    /// The `LINE` of the call.
    pub line: u32,
    /// Which hook printed it.
    pub hook: Hook,
    /// Every labelled line, in print order: a prefix of [`RECORD_LABELS`].
    pub values: Vec<(&'static str, u32)>,
}

impl Record {
    /// A line by label: the first with it, except `World`, which is the
    /// sixteenth `World` (the whole world, `World::walk_data(-1)`).
    pub fn get(&self, label: &str) -> Option<u32> {
        if label == "World" {
            return self.world_parts().get(15).copied();
        }
        self.values
            .iter()
            .find(|(l, _)| *l == label)
            .map(|(_, v)| *v)
    }

    /// The sixteen `World` lines: sections 0 to 14, then the whole.
    pub fn world_parts(&self) -> Vec<u32> {
        self.values
            .iter()
            .filter(|(l, _)| *l == "World")
            .map(|(_, v)| *v)
            .collect()
    }

    /// The fifteen of [`CHECKSUM_LINES`], if the record prints them all.
    pub fn fifteen(&self) -> Option<[u32; 15]> {
        let mut out = [0; 15];
        for (slot, (_, label, _)) in out.iter_mut().zip(CHECKSUM_LINES) {
            *slot = self.get(label)?;
        }
        Some(out)
    }
}

/// A record still being read.
struct Pending {
    number: u64,
    file: Option<String>,
    line: Option<u32>,
    values: Vec<(&'static str, u32)>,
    frame: Option<i64>,
    under_frame_head: bool,
}

impl Pending {
    /// Takes `text` into the record if it is the record's next line.
    fn take(&mut self, text: &str) -> bool {
        if self.file.is_none() {
            return match text.strip_prefix("FILE ") {
                Some(f) => {
                    self.file = Some(f.trim().to_string());
                    true
                }
                None => false,
            };
        }
        if self.line.is_none() {
            return match text
                .strip_prefix("LINE ")
                .and_then(|l| l.trim().parse().ok())
            {
                Some(l) => {
                    self.line = Some(l);
                    true
                }
                None => false,
            };
        }
        let Some(label) = RECORD_LABELS.get(self.values.len()) else {
            return false;
        };
        match text
            .strip_prefix(label)
            .and_then(|r| r.strip_prefix(' '))
            .and_then(|v| v.trim().parse::<u32>().ok())
        {
            Some(v) => {
                self.values.push((label, v));
                true
            }
            None => false,
        }
    }

    fn finish(self) -> Option<Record> {
        let file = self.file?;
        let line = self.line?;
        let hook = match self.frame {
            _ if file != "gamelog.cpp" => Hook::Setup,
            None => Hook::Game,
            Some(n) if self.under_frame_head => Hook::Begin(n),
            Some(n) => Hook::End(n),
        };
        Some(Record {
            number: self.number,
            file,
            line,
            hook,
            values: self.values,
        })
    }
}

/// Every `CHECKSUM` record of a log, in log order.
pub fn records(mut input: impl BufRead) -> std::io::Result<Vec<Record>> {
    let mut out = Vec::new();
    let mut buf = Vec::new();
    let (mut prev, mut prev2) = (String::new(), String::new());
    let mut frame: Option<i64> = None;
    let mut pending: Option<Pending> = None;
    loop {
        buf.clear();
        let eof = input.read_until(b'\n', &mut buf)? == 0;
        if eof {
            break;
        }
        let text = String::from_utf8_lossy(&buf);
        let text = text.trim();
        if let Some(p) = pending.as_mut() {
            if p.take(text) {
                continue;
            }
            out.extend(pending.take().and_then(Pending::finish));
        }
        if let Some(n) = text
            .strip_prefix("BEGIN FRAME ")
            .and_then(|n| n.trim().parse().ok())
        {
            frame = Some(n);
        } else if let Some(n) = text
            .strip_prefix("CHECKSUM ")
            .and_then(|n| n.trim().parse().ok())
        {
            let head = frame.map(|f| format!("BEGIN FRAME {f}"));
            pending = Some(Pending {
                number: n,
                file: None,
                line: None,
                values: Vec::new(),
                frame,
                under_frame_head: prev == "BEGIN FULL DUMP" && head.as_deref() == Some(&prev2),
            });
        }
        prev2 = std::mem::replace(&mut prev, text.to_string());
    }
    out.extend(pending.and_then(Pending::finish));
    Ok(out)
}

/// The begin and end records of frame `n`.
pub fn frame(records: &[Record], n: i64) -> (Option<&Record>, Option<&Record>) {
    let begin = records.iter().find(|r| r.hook == Hook::Begin(n));
    let end = records.iter().find(|r| r.hook == Hook::End(n));
    (begin, end)
}

/// The instrument's fifteen, as `tools/recomp/checksum.py --values` writes
/// them: `(check_all name, value)` per line, comments skipped.
pub fn instrument_values(text: &str) -> Vec<(String, u32)> {
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with("line\t"))
        .filter_map(|l| {
            let mut it = l.split('\t');
            let name = it.next()?.to_string();
            let _label = it.next()?;
            Some((name, it.next()?.parse().ok()?))
        })
        .collect()
}

/// The lines of a record that part from the instrument's values: each
/// `(name, printed, instrument)`.
pub fn parting(record: &Record, instrument: &[(String, u32)]) -> Vec<(&'static str, u32, u32)> {
    CHECKSUM_LINES
        .iter()
        .filter_map(|(name, label, _)| {
            let ours = instrument.iter().find(|(n, _)| n == name)?.1;
            let printed = record.get(label)?;
            (printed != ours).then_some((*name, printed, ours))
        })
        .collect()
}

#[cfg(test)]
mod tests;
