//! The `AMMO` record, whole (`docs/COMBAT.md` §24).
//!
//! `AmmoData::log_data@00679c00` prints **twenty-seven** fields across five
//! detail levels, and item 396 compared five of them. This is the rest, on
//! the same evidence: run109, `AMMO=5` over `[9420, 9480)` on the Great
//! Lakes seed, 183 blocks carrying nine arrows.
//!
//! The rule this module exists for is `CLAUDE.md`'s — *when the original
//! dumps a record, diff the whole record* — and what the widening found is
//! that fourteen of the twenty-seven are things this crate holds and can be
//! compared by value, while thirteen are engine bookkeeping, a `z` axis the
//! simulation does not have, or one of the four floats. Those thirteen are
//! not left silent: they are asserted as the original's own values, so a
//! row that is residue today is a row that says so when the archive changes.
//!
//! # The levels, from the decompile
//!
//! `log_data` opens a level before each group (`Log`'s vtable `+0x28`):
//!
//! | level | fields |
//! |---|---|
//! | 1 | `cur_time`, `total_time`, `who`, `o`, `whom`, `ox` |
//! | 2 | `sx`, `sy`, `sz`, `ex`, `ey`, `ez`, `angle` |
//! | 3 | `traj`, `v1z`, `dx`, `start_roll_angle`, `bank_dx`, `bank_dy`, and `SplineData::log_data` when `ammo_path` is set |
//! | 4 | `flags`, `rolling`, `gpiece`, `graph_index`, `splash_area`, `index`, `num_guys` |
//! | 5 | `accuracy` |
//!
//! and the whole record is gated on `this->flags & 3`, so an ammo with
//! neither bit prints nothing at any level.

use std::collections::BTreeMap;

/// One `BEGIN AMMO` block, every field it printed.
///
/// The four floats are kept as the printed decimal scaled by a million
/// rather than parsed — this crate reads the original's numbers, it does
/// not do arithmetic in them, and `133.951630` is exactly `133_951_630`
/// micro-units with nothing lost.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) struct Ammo {
    // level 1
    pub cur_time: i64,
    pub total_time: i64,
    pub who: i64,
    pub o: i64,
    pub whom: i64,
    pub ox: i64,
    // level 2
    pub sx: i64,
    pub sy: i64,
    pub sz: i64,
    pub ex: i64,
    pub ey: i64,
    pub ez: i64,
    pub angle: i64,
    // level 3
    pub traj: i64,
    pub v1z: i64,
    pub dx: i64,
    pub start_roll_angle: i64,
    pub bank_dx: i64,
    pub bank_dy: i64,
    // level 4
    pub flags: i64,
    pub rolling: i64,
    pub gpiece: i64,
    pub graph_index: i64,
    pub splash_area: i64,
    pub index: i64,
    pub num_guys: i64,
    // level 5
    pub accuracy: i64,
}

/// The twenty-seven names, in the order `log_data` prints them.
pub(crate) const FIELDS: &[&str] = &[
    "cur_time",
    "total_time",
    "who",
    "o",
    "whom",
    "ox",
    "sx",
    "sy",
    "sz",
    "ex",
    "ey",
    "ez",
    "angle",
    "traj",
    "v1z",
    "dx",
    "start_roll_angle",
    "bank_dx",
    "bank_dy",
    "flags",
    "rolling",
    "gpiece",
    "graph_index",
    "splash_area",
    "index",
    "num_guys",
    "accuracy",
];

/// The four `log_data` hands the float printer (vtable `+0x14`) rather than
/// the integer one (`+0x1c`/`+0x18`); they arrive with six decimals.
pub(crate) const FLOATS: &[&str] = &["v1z", "dx", "bank_dx", "bank_dy"];

/// `"133.951630"` → `133_951_630`. Six decimals exactly, or `None`.
fn micro(s: &str) -> Option<i64> {
    let (neg, s) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s),
    };
    let (whole, frac) = s.split_once('.')?;
    if frac.len() != 6 || !frac.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let v = whole.parse::<i64>().ok()? * 1_000_000 + frac.parse::<i64>().ok()?;
    Some(if neg { -v } else { v })
}

/// Every `BEGIN AMMO` block in one frame body, whole.
///
/// **The flush has to come before the open**, and this is the trap item 396
/// fell into and caught only because it had asserted a count first:
/// `Objects::dump_ammo` walks the pool, so the blocks of one frame are
/// *consecutive siblings*, and a version of this that reset the pending
/// record on `BEGIN AMMO` threw away every arrow but the last of each run —
/// six of nine, silently, and the survivors came back labelled two frames
/// early. Hence [`blocks`] returns a count its callers assert before they
/// assert a value.
///
/// **Nesting is by indentation, because the log writes no `END`.** `BEGIN
/// AMMO` sits at one depth and its fields one deeper; `log_data` can emit a
/// nested `SplineData` block between `bank_dy` and `flags` when `ammo_path`
/// is set, and a flat walk would either swallow the spline's fields or drop
/// the seven that follow it. No block in run109 carries one — all 183 print
/// `traj 1` — so that branch is written from the decompile and **is not
/// exercised by any capture on this disk**. The count assertion is what
/// would catch it: a record that lost or gained a field is not 27 long.
pub(crate) fn blocks(body: &str) -> Vec<(Ammo, usize)> {
    let indent = |l: &str| l.len() - l.trim_start().len();
    let mut out: Vec<(Ammo, usize)> = Vec::new();
    // The pending record: the named fields it has, the *total* count of
    // `key value` lines seen inside it (so a field this crate does not know
    // still moves the count), and the depth its `BEGIN AMMO` sat at.
    let mut cur: Option<(BTreeMap<&str, &str>, usize, usize)> = None;
    // While `Some`, we are inside a nested block opened at this depth and
    // every line deeper than it belongs to the nested record, not ours.
    let mut nested: Option<usize> = None;
    let flush = |cur: &mut Option<(BTreeMap<&str, &str>, usize, usize)>,
                 out: &mut Vec<(Ammo, usize)>| {
        let Some((done, n, _)) = cur.take() else {
            return;
        };
        let int = |k: &str| done.get(k).and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
        let flt = |k: &str| done.get(k).and_then(|v| micro(v)).unwrap_or(0);
        out.push((
            Ammo {
                cur_time: int("cur_time"),
                total_time: int("total_time"),
                who: int("who"),
                o: int("o"),
                whom: int("whom"),
                ox: int("ox"),
                sx: int("sx"),
                sy: int("sy"),
                sz: int("sz"),
                ex: int("ex"),
                ey: int("ey"),
                ez: int("ez"),
                angle: int("angle"),
                traj: int("traj"),
                v1z: flt("v1z"),
                dx: flt("dx"),
                start_roll_angle: int("start_roll_angle"),
                bank_dx: flt("bank_dx"),
                bank_dy: flt("bank_dy"),
                flags: int("flags"),
                rolling: int("rolling"),
                gpiece: int("gpiece"),
                graph_index: int("graph_index"),
                splash_area: int("splash_area"),
                index: int("index"),
                num_guys: int("num_guys"),
                accuracy: int("accuracy"),
            },
            n,
        ));
    };
    for line in body.lines() {
        let line = line.trim_end_matches('\r').trim_end();
        if line.trim().is_empty() {
            continue;
        }
        let depth = indent(line);
        let t = line.trim_start();
        if let Some(d) = nested {
            if depth > d {
                continue;
            }
            nested = None;
        }
        if t == "BEGIN AMMO" {
            flush(&mut cur, &mut out);
            cur = Some((BTreeMap::new(), 0, depth));
            continue;
        }
        let Some((map, seen, open)) = cur.as_mut() else {
            continue;
        };
        if depth <= *open {
            // Out of the record entirely — a sibling block, or the frame's end.
            flush(&mut cur, &mut out);
            continue;
        }
        if t.starts_with("BEGIN ") {
            // `SplineData::log_data`, inside our record. Skip its body and
            // keep collecting ours after it.
            nested = Some(depth);
            continue;
        }
        let mut it = t.split_whitespace();
        if let (Some(k), Some(v), None) = (it.next(), it.next(), it.next()) {
            // A line counts towards the record's width when it is a field
            // this crate knows **and** it arrived in the type `log_data`
            // prints it in — the float printer at the `Log` vtable's `+0x14`
            // for [`FLOATS`], the integer one at `+0x18`/`+0x1c` for the
            // rest. So a field that changed type, or vanished, makes the
            // width short, and a field nobody here knows makes it long.
            // Either way the callers' `== FIELDS.len()` is what says so.
            match FIELDS.iter().find(|f| **f == k) {
                Some(name) if FLOATS.contains(name) => {
                    if let Some(u) = micro(v) {
                        map.insert(*name, v);
                        let _ = u;
                        *seen += 1;
                    }
                }
                Some(name) => {
                    if v.parse::<i64>().is_ok() {
                        map.insert(*name, v);
                        *seen += 1;
                    }
                }
                None => *seen += 1,
            }
        }
    }
    flush(&mut cur, &mut out);
    out
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::diff::testkit::{first_parting, sibling_texts, trace};
    use crate::diff::{borrow_from_siblings, borrow_pasture, build_sim};
    use crate::gamelog::{Initial, Log};
    use crate::testenv::{dump, install};
    use sim::Tuning;
    use sim::combat::Obj;

    /// run109's window: `AMMO=5` over `[9420, 9480)`, and an arrow launched
    /// on sim-frame `L` first prints in block `L + 1`.
    pub(crate) const FIRST: i64 = 9_420;
    pub(crate) const LAST: i64 = 9_480;

    /// Every `AMMO` block in run109's window, tagged with the block it was
    /// printed in and the number of `key value` lines it held.
    pub(crate) fn run109() -> Option<Vec<(i64, Ammo, usize)>> {
        let path = dump("gamelog-run109-greatlakes-ammolaunch.txt")?;
        let mut ix = crate::capture::indexed::IndexedCapture::open(&path).unwrap();
        let mut out = Vec::new();
        for f in FIRST..LAST {
            let Some(at) = ix.frames().iter().position(|x| x.number == f) else {
                continue;
            };
            let body = ix.read_frame(at).unwrap();
            for (a, n) in blocks(&body) {
                out.push((f, a, n));
            }
        }
        Some(out)
    }

    /// **The count before the value.** 183 blocks, each of them the whole
    /// twenty-seven fields, nine distinct arrows.
    ///
    /// This is the assertion item 396's parser bug was caught by, kept and
    /// tightened: the old walk lost six of nine arrows and mislabelled the
    /// survivors by two frames, and nothing but `theirs.len() == 9` said so.
    /// A count of *fields* is the same guard one level down — a record that
    /// gained a field, lost one, or swallowed a nested `SplineData` block is
    /// not twenty-seven lines long.
    #[test]
    fn run109_s_ammo_record_is_twenty_seven_fields_on_every_block() {
        let Some(all) = run109() else {
            eprintln!("skipping: no run109 capture (set RON_GAMELOG_DIR)");
            return;
        };
        assert_eq!(
            all.len(),
            183,
            "run109's window holds 183 AMMO blocks; it holds {}",
            all.len()
        );
        let odd: Vec<_> = all.iter().filter(|(_, _, n)| *n != FIELDS.len()).collect();
        assert!(
            odd.is_empty(),
            "AmmoData::log_data prints {} fields and these blocks did not: {odd:?}",
            FIELDS.len()
        );
        // Identity: an arrow is its shooter, its endpoints and its clock.
        let mut ids: Vec<(i64, i64, i64, i64, i64, i64)> = all
            .iter()
            .map(|(_, a, _)| (a.o, a.sx, a.sy, a.ex, a.ey, a.total_time))
            .collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 9, "nine arrows over the window: {ids:?}");
        // And the blocks of one frame are consecutive siblings, which is
        // what the flush-before-open in [`blocks`] is for: 9451 carries
        // four at once.
        let at_9451 = all.iter().filter(|(f, _, _)| *f == 9_451).count();
        assert_eq!(at_9451, 4, "block 9451 carries four arrows at once");
    }

    /// One residue field that never moves: its name, how to read it, and
    /// the value it holds on every block of run109's window.
    type Constant = (&'static str, fn(&Ammo) -> i64, i64);

    /// **The thirteen fields `sim::combat::Projectile` has no room for**,
    /// pinned as the original's own values (`docs/COMBAT.md` §24.3).
    ///
    /// Eight of them are constant across all 183 blocks and are asserted as
    /// constants; five vary and are asserted by their per-arrow value. None
    /// of this is a claim about the port — it is a claim about the record,
    /// and its job is to make the residue *visible*: the day one of these
    /// becomes something the simulation produces, this is the test that says
    /// what it has to produce.
    ///
    /// Three of the thirteen are nonetheless tied back to this crate's own
    /// integers elsewhere, and those are the stronger claims:
    /// [`run100_and_run109_agree_on_where_the_bow_hand_is_off_the_ground`]
    /// derives `sz`, and
    /// [`run109_s_ammo_floats_are_the_trajectory_this_crate_does_not_fly`]
    /// derives `v1z` and `dx`. What is left with nothing but this test
    /// behind it is the ten in §24.3's own table.
    #[test]
    fn run109_s_ammo_record_pins_the_thirteen_fields_nothing_here_models() {
        let Some(all) = run109() else {
            eprintln!("skipping: no run109 capture (set RON_GAMELOG_DIR)");
            return;
        };
        // ---- the eight constants.
        let konst: &[Constant] = &[
            // The landing height: the ground under the farm, and the one
            // target in this window.
            ("ez", |a| a.ez, 548),
            // `AmmoTrajectory`: 1 on every arrow here. `log_data` prints a
            // nested `SplineData` block when `ammo_path` is set, and none
            // of the 183 has one.
            ("traj", |a| a.traj, 1),
            // The roll/bank triple is for a model that spins in flight.
            ("start_roll_angle", |a| a.start_roll_angle, 0),
            ("bank_dx", |a| a.bank_dx, 0),
            ("bank_dy", |a| a.bank_dy, 0),
            // `log_data`'s own gate is `flags & 3`, so a record that prints
            // at all has one of the low two bits; these carry bit 1.
            ("flags", |a| a.flags, 2),
            ("rolling", |a| a.rolling, 0),
            // The *ammo's* graphic piece, not the shooter's: 60157 against
            // the Longbowman's own 127.
            ("gpiece", |a| a.gpiece, 60_157),
        ];
        for (name, get, want) in konst {
            let bad: Vec<_> = all
                .iter()
                .filter(|(_, a, _)| get(a) != *want)
                .map(|(f, a, _)| (*f, get(a)))
                .collect();
            assert!(
                bad.is_empty(),
                "run109's `{name}` is not {want} on every block: {bad:?}"
            );
        }
        // ---- the five that vary, per arrow, at its first block.
        let mut first: Vec<(i64, Ammo)> = Vec::new();
        for (f, a, _) in &all {
            let id = (a.o, a.sx, a.sy, a.ex, a.ey, a.total_time);
            if !first
                .iter()
                .any(|(_, b)| (b.o, b.sx, b.sy, b.ex, b.ey, b.total_time) == id)
            {
                first.push((*f, *a));
            }
        }
        let rows: Vec<(i64, i64, i64, i64, i64, i64)> = first
            .iter()
            .map(|(f, a)| (f - 1, a.sz, a.v1z, a.dx, a.graph_index, a.index))
            .collect();
        eprintln!("  run109 residue (launch, sz, v1z, dx, graph_index, index):");
        for r in &rows {
            eprintln!("    {r:?}");
        }
        assert_eq!(
            rows,
            vec![
                // launch      sz        v1z          dx     graph  index
                (9_425, 754, 133_951_630, 100_541_908, 0, 0),
                (9_426, 722, 129_645_203, 103_700_104, 1, 1),
                (9_439, 753, 128_452_896, 100_408_539, 2, 2),
                (9_444, 754, 128_414_429, 101_260_757, 3, 3),
                (9_451, 738, 129_029_816, 101_397_415, 4, 4),
                (9_456, 722, 124_133_751, 102_119_942, 5, 0),
                (9_462, 675, 126_013_748, 103_900_818, 6, 1),
                (9_467, 738, 123_493_752, 103_771_881, 7, 2),
                (9_474, 754, 122_853_752, 101_929_306, 8, 3),
            ],
            "run109's residue fields are not what item 402 read out of it"
        );
    }

    /// The fourteen fields of the record this crate holds, as one row.
    ///
    /// `who`/`o` is the shooter and `whom`/`ox` the target, the same
    /// `(player, object)` pair every other record here is keyed by
    /// (`SubObjectData::who`/`o`). A ground shot has no target and the
    /// original prints `-1`/`-1` for it; nothing in this window is one.
    type Row = [i64; 15];

    /// [`Row`]'s columns, for the failure message.
    const COLS: &[&str] = &[
        "block",
        "who",
        "o",
        "whom",
        "ox",
        "sx",
        "sy",
        "ex",
        "ey",
        "angle",
        "cur_time",
        "total_time",
        "splash_area",
        "num_guys",
        "accuracy",
    ];

    /// One row, named. A fifteen-column tuple prints as a wall of integers
    /// and the reader then has to count commas to find which field moved.
    fn show(r: &Row) -> String {
        COLS.iter()
            .zip(r)
            .map(|(c, v)| format!("{c} {v}"))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// **The whole record, every block, both sides** (`docs/COMBAT.md`
    /// §24.2).
    ///
    /// Item 396 compared five numbers on nine arrows at their first block.
    /// This compares the fourteen the simulation holds on all **183**
    /// blocks — every arrow, every frame of its flight — which is the rule
    /// `CLAUDE.md` states and the reason it states it.
    ///
    /// What the widening adds over 396 beyond the extra fields is the
    /// arrow's **lifetime**: the original's `cur_time` runs 1 to
    /// `total_time - 1` and the record then stops, because `Ammo::inc_time`
    /// lands and frees the shot on the frame its clock would reach
    /// `total_time`. `crates/sim`'s `process_projectiles` is the same
    /// predicate (`cur_time < total_time`), and this is what says so:
    /// a0 launches on 9425 with `total_time` 27, prints 9426..9451, and its
    /// hit is the first half of the farm's 26-sixteenth step on 9452 that
    /// [`super::super::build::tests::run100_says_great_lakes_9451_is_two_arrows_on_one_farm`]
    /// asserts from the other end.
    #[test]
    fn run109_says_every_ammo_field_this_crate_models_is_the_original_s() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(all)) = (dump("gamelog-run53-greatlakes-24k-trace.txt"), run109())
        else {
            eprintln!("skipping: no run53/run109 capture (set RON_GAMELOG_DIR)");
            return;
        };
        assert_eq!(all.len(), 183, "run109's window holds 183 AMMO blocks");
        let mut theirs: Vec<Row> = all
            .iter()
            .map(|(f, a, _)| {
                [
                    *f,
                    a.who,
                    a.o,
                    a.whom,
                    a.ox,
                    a.sx,
                    a.sy,
                    a.ex,
                    a.ey,
                    a.angle,
                    a.cur_time,
                    a.total_time,
                    a.splash_area,
                    a.num_guys,
                    a.accuracy,
                ]
            })
            .collect();
        theirs.sort_unstable();

        // ---- this crate, over the same frames. `built.tick()` at 0-based
        // `f` leaves the state the dump numbers `f + 1`.
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        if let Some(t) = trace("rontrace-run53.log") {
            borrow_pasture(&mut init, &t);
        }
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let mut ours: Vec<Row> = Vec::new();
        for f in 0..LAST - 1 {
            built.tick();
            let block = f + 1;
            if block < FIRST {
                continue;
            }
            let ident = |s: &sim::Sim, o: Obj| match o {
                Obj::Unit(u) => (i64::from(s.units[u].owner), i64::from(s.units[u].index)),
                Obj::Building(b) => (
                    i64::from(s.buildings[b].owner),
                    i64::from(s.buildings[b].index),
                ),
            };
            for p in &built.sim.projectiles {
                let (who, o) = ident(&built.sim, p.shooter);
                let (whom, ox) = p.target.map_or((-1, -1), |t| ident(&built.sim, t));
                ours.push([
                    block,
                    who,
                    o,
                    whom,
                    ox,
                    i64::from(p.launch.x),
                    i64::from(p.launch.y),
                    i64::from(p.landing.x),
                    i64::from(p.landing.y),
                    i64::from(p.angle.0),
                    i64::from(p.cur_time),
                    i64::from(p.total_time),
                    i64::from(p.splash_area),
                    i64::from(p.num_guys),
                    i64::from(p.accuracy),
                ]);
            }
        }
        ours.sort_unstable();
        eprintln!(
            "  run109: {} blocks, ours: {} blocks",
            theirs.len(),
            ours.len()
        );
        // **The count before the value**, on this side too: a crate that
        // launched the right arrows and kept them a frame too long would
        // otherwise fail on a value and read as an aim bug.
        assert_eq!(
            ours.len(),
            theirs.len(),
            "this crate holds {} ammo records over [{FIRST}, {LAST}) where \
             run109 printed {} — the arrows' *lifetimes* differ, not their \
             numbers",
            ours.len(),
            theirs.len()
        );
        if let Some((at, s)) = first_parting(
            &ours.iter().map(show).collect::<Vec<_>>(),
            &theirs.iter().map(show).collect::<Vec<_>>(),
        ) {
            panic!(
                "this crate's AMMO records are not the original's \
                 (block, who, o, whom, ox, sx, sy, ex, ey, angle, cur_time, \
                 total_time, splash_area, num_guys, accuracy)\n{s}\nat {at}"
            );
        }
    }

    /// A single as `%f` prints it, in the millionths [`Ammo`] keeps. The
    /// host's `{:.6}` is correctly rounded, as the original's printf is;
    /// this is test code, where `no_float.rs` does not reach and an oracle
    /// in the host's arithmetic is the point.
    pub(crate) fn printed(s: sim::single::Single) -> i64 {
        micro(&format!("{:.6}", f32::from_bits(s.bits()))).expect("six decimals")
    }

    /// **`v1z` to the last printed digit** (`docs/COMBAT.md` §46.1).
    ///
    /// The test below holds the printed `v1z` to its formula within three
    /// ulp, because until item 495 this crate had no single-precision
    /// arithmetic to hold it to more. [`sim::combat::arc_v1z`] is that
    /// arithmetic — `Ammo::init`'s own four SSE steps in
    /// [`sim::single::Single`] — and on every one of run109's 183 records
    /// its single prints as exactly the six decimals the original printed.
    #[test]
    fn run109_s_v1z_is_arc_v1z_to_the_last_digit() {
        let Some(all) = run109() else {
            eprintln!("skipping: no run109 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let wrong: Vec<_> = all
            .iter()
            .filter(|(_, a, _)| {
                let v = sim::combat::arc_v1z(a.sz as i32, a.ez as i32, a.total_time as i32);
                printed(v) != a.v1z
            })
            .map(|(f, a, _)| (*f, a.sz, a.ez, a.total_time, a.v1z))
            .collect();
        assert!(wrong.is_empty(), "v1z is not arc_v1z's single: {wrong:?}");
        assert_eq!(all.len(), 183);
    }

    /// `GraphicPieces::init@008ffcc0+0x??` sets `GRAV_Z = -10.4875`, and
    /// this is it scaled by a million, the same way [`Ammo::v1z`] is.
    const GRAV_Z: i64 = -10_487_500;

    /// **The two floats, derived — and both halves confirmed** (§24.5).
    ///
    /// `Ammo::init@0067bbf0` ends with the trajectory:
    ///
    /// ```text
    /// v1z = ((ez - sz) - GRAV_Z * 0.5 * T * T) / T
    /// dx  = sqrtf(dx*dx + dy*dy) / T
    /// ```
    ///
    /// (the decompiler drops `sqrtf`'s result into a discarded temporary —
    /// the FPU-stack artefact `tools/ghidra/README.md` warns about — and
    /// `field_0x58 = fVar22 / local_28` with `fVar22` still the *sum of
    /// squares* is that artefact, not the engine squaring a distance.)
    ///
    /// Until item 495 neither was a number this crate could hold — no float
    /// in the simulation — so the assertion here is the weaker one; `v1z`
    /// is now held exactly by [`run109_s_v1z_is_arc_v1z_to_the_last_digit`]
    /// and this stays for `dx`. The weaker one — that the original's own printed
    /// floats *are* those formulae over the integers this crate does
    /// reproduce — computed in integers at a millionth, to within a float32
    /// ulp at their own magnitude.
    ///
    /// It earns its place twice. It is the only check on `total_time` that
    /// does not go through the launch point, and it is the check that the
    /// flight is a **plan** distance: `dx * T` is `hypot(ex - sx, ey - sy)`
    /// and not the 3-D length, which for a0 would be 2722 against 2714.
    #[test]
    fn run109_s_ammo_floats_are_the_trajectory_this_crate_does_not_fly() {
        let Some(all) = run109() else {
            eprintln!("skipping: no run109 capture (set RON_GAMELOG_DIR)");
            return;
        };
        assert_eq!(all.len(), 183, "run109's window holds 183 AMMO blocks");
        let mut checked = 0;
        for (f, a, _) in &all {
            let t = a.total_time;
            // `v1z`: three multiplies and a subtract in float32, so three
            // ulp of headroom on a value the size of `v1z` itself.
            let want = ((a.ez - a.sz) * 2_000_000 - GRAV_Z * t * t).div_euclid(2 * t);
            let slack = 3 * ((a.v1z.abs() >> 23) + 1);
            assert!(
                (a.v1z - want).abs() <= slack,
                "block {f}: v1z {} is not ((ez {} - sz {}) - GRAV_Z * T*T/2) / T {t} = {want} \
                 (slack {slack})",
                a.v1z,
                a.ez,
                a.sz
            );
            // `dx`: one float32 ulp of `dx`, times the flight time it is
            // multiplied back up by, plus the integer square root's floor.
            let d2 = (a.ex - a.sx).pow(2) + (a.ey - a.sy).pow(2);
            let dist = i64::try_from(
                u128::from(d2.unsigned_abs())
                    .checked_mul(1_000_000_000_000)
                    .unwrap()
                    .isqrt(),
            )
            .unwrap();
            let slack = ((a.dx >> 23) + 1) * t + t;
            assert!(
                (a.dx * t - dist).abs() <= slack,
                "block {f}: dx {} over {t} frames is {} where the plan distance \
                 from ({}, {}) to ({}, {}) is {dist} (slack {slack})",
                a.dx,
                a.dx * t,
                a.sx,
                a.sy,
                a.ex,
                a.ey
            );
            checked += 1;
        }
        assert_eq!(checked, 183, "every block, not a subset");
    }

    /// **`index` is the lowest free slot of the ammo pool, `graph_index`
    /// is the game's running count of shots, and the record is printed in
    /// slot order** (§24.4). Read first, then measured.
    ///
    /// `Objects::add_ammo@00658b10` scans its pool from slot 0 and
    /// **breaks at the first whose `flags & 3` is clear** — the lowest free
    /// one, growing the array only when every slot is taken — then calls
    /// `Ammo::init` with that slot as one argument and `objects+0x1f8` as
    /// the next, incrementing `+0x1f8` after. So the two indices are a
    /// reused slot and a monotone counter, and they are different things.
    /// `GameLog::dump_ammo@0092fe40` then walks the same pool `0..count`,
    /// skipping the slots whose `flags & 3` is clear, which is why a
    /// frame's blocks ascend in `index` with gaps rather than running
    /// 0, 1, 2.
    ///
    /// Each of those is measured here rather than taken on the reading.
    /// Replaying "take the lowest slot no live arrow holds" over the nine
    /// launches gives 0, 1, 2, 3, 4, **0, 1, 2, 3** — four reuses, all four
    /// right; `graph_index` runs 0..8 from the game's very first arrow on
    /// 9425; and no frame's blocks descend.
    ///
    /// **Item 495 made this crate do it** ([`sim::Sim::add_ammo`],
    /// `docs/COMBAT.md` §46.4). Until then `Sim::projectiles` was a `Vec`
    /// that `process_projectiles` `swap_remove`d from, so after the first
    /// landing its order was neither launch order nor slot order — and the
    /// frame this predicted arrived in chapter two: `0/7` and `0/8` both due
    /// on 683, stepped in the wrong order, and the wrong one rolled. What
    /// the crate still does not do is hold a **spent** arrow's slot for its
    /// 200 frames (§46.7).
    #[test]
    fn run109_s_ammo_index_is_the_lowest_free_slot_in_the_pool() {
        let Some(all) = run109() else {
            eprintln!("skipping: no run109 capture (set RON_GAMELOG_DIR)");
            return;
        };
        // Each arrow's first block, in launch order.
        let mut first: Vec<Ammo> = Vec::new();
        for (_, a, _) in &all {
            let id = (a.o, a.sx, a.sy, a.ex, a.ey, a.total_time);
            if !first
                .iter()
                .any(|b| (b.o, b.sx, b.sy, b.ex, b.ey, b.total_time) == id)
            {
                first.push(*a);
            }
        }
        assert_eq!(first.len(), 9, "nine arrows");
        // Replay the allocation. An arrow launched on `L` with flight time
        // `T` holds its slot until it lands on `L + T`; a first block is the
        // one printing `cur_time 1`, so `L` is that block less one.
        let mut held: Vec<Option<i64>> = Vec::new();
        let mut got: Vec<i64> = Vec::new();
        let launches: Vec<i64> = all
            .iter()
            .filter(|(_, a, _)| a.cur_time == 1)
            .map(|(f, _, _)| f - 1)
            .collect();
        assert_eq!(launches.len(), 9, "nine first blocks: {launches:?}");
        for (a, l) in first.iter().zip(&launches) {
            for h in held.iter_mut() {
                if h.is_some_and(|dead| dead <= *l) {
                    *h = None;
                }
            }
            let slot = match held.iter().position(Option::is_none) {
                Some(s) => s,
                None => {
                    held.push(None);
                    held.len() - 1
                }
            };
            held[slot] = Some(l + a.total_time);
            got.push(i64::try_from(slot).unwrap());
        }
        let want: Vec<i64> = first.iter().map(|a| a.index).collect();
        eprintln!("  index: predicted {got:?} printed {want:?}");
        assert_eq!(got, want, "the ammo pool's slot is not the lowest free one");
        assert_eq!(
            want,
            vec![0, 1, 2, 3, 4, 0, 1, 2, 3],
            "and four of the nine are reuses, which is what makes it a claim"
        );
        // `graph_index` is `objects+0x1f8`, incremented once per
        // `add_ammo` and never reused — so it running 0..8 says these are
        // the *first nine arrows of the game*, which is the same thing
        // `golden`'s `Ammo::init` draw count says from the trace's side.
        let g: Vec<i64> = first.iter().map(|a| a.graph_index).collect();
        assert_eq!(g, (0..9).collect::<Vec<_>>(), "graph_index: {g:?}");
        // The blocks of one frame are ascending in `index` — the pool walk.
        let mut by_frame: Vec<(i64, Vec<i64>)> = Vec::new();
        for (f, a, _) in &all {
            match by_frame.last_mut() {
                Some((g, v)) if g == f => v.push(a.index),
                _ => by_frame.push((*f, vec![a.index])),
            }
        }
        let unsorted: Vec<_> = by_frame
            .iter()
            .filter(|(_, v)| v.windows(2).any(|w| w[0] >= w[1]))
            .collect();
        assert!(
            unsorted.is_empty(),
            "dump_ammo walks the pool, so a frame's blocks ascend in `index`; \
             these did not: {unsorted:?}"
        );
        assert!(
            by_frame.iter().any(|(_, v)| v.len() == 5),
            "and at 9463 five arrows are in the air at once: {by_frame:?}"
        );
    }

    /// A hand-built body, for the two shapes no capture on this disk has.
    ///
    /// **Synthetic, and labelled so**: this is not evidence about the
    /// original, it is a test of [`blocks`] against the two ways the walk
    /// has been or could be wrong.
    ///
    /// 1. **Consecutive siblings.** Item 396's first `ammo_blocks` reset
    ///    the pending record on `BEGIN AMMO` instead of flushing it, and
    ///    because `Objects::dump_ammo` walks the pool that is exactly the
    ///    shape the log has: six of nine arrows vanished and the survivors
    ///    came back two frames early. Two adjacent blocks here, with
    ///    different values, catch it.
    /// 2. **A nested `SplineData`.** `log_data` emits one between `bank_dy`
    ///    and `flags` when `ammo_path` is set, which a flat walk would
    ///    either splice into the record or truncate the record before. All
    ///    183 of run109's blocks print `traj 1` and carry none, so this
    ///    branch has no capture behind it and this is the only thing that
    ///    exercises it.
    #[test]
    fn the_walk_survives_a_sibling_and_a_nested_spline() {
        let body = |sx: i64, spline: bool| {
            let mut v = vec![
                "  BEGIN AMMO".to_string(),
                "   cur_time 1".into(),
                "   total_time 27".into(),
                "   who 1".into(),
                "   o 29".into(),
                "   whom 0".into(),
                "   ox 2004".into(),
                format!("   sx {sx}"),
                "   sy 29951".into(),
                "   sz 754".into(),
                "   ex 2192".into(),
                "   ey 31179".into(),
                "   ez 548".into(),
                "   angle -1411055616".into(),
                "   traj 1".into(),
                "   v1z 133.951630".into(),
                "   dx 100.541908".into(),
                "   start_roll_angle 0".into(),
                "   bank_dx 0.000000".into(),
                "   bank_dy 0.000000".into(),
            ];
            if spline {
                v.push("   BEGIN SPLINE".into());
                // A nested field whose *name collides with ours* is the
                // reason the walk skips by depth rather than by name.
                v.push("    sx 999".into());
                v.push("    points 4".into());
            }
            v.extend(
                [
                    "   flags 2",
                    "   rolling 0",
                    "   gpiece 60157",
                    "   graph_index 0",
                    "   splash_area 0",
                    "   index 0",
                    "   num_guys 1",
                    "   accuracy 278",
                ]
                .map(String::from),
            );
            v.join("\r\n")
        };
        let frame = format!(
            " BEGIN FRAME 9426\n{}\n{}\n  BEGIN WORLD\n   forest_size 189\n",
            body(4613, false),
            body(4620, true)
        );
        let got = blocks(&frame);
        assert_eq!(got.len(), 2, "two sibling records, not one: {got:?}");
        assert!(
            got.iter().all(|(_, n)| *n == FIELDS.len()),
            "both records are whole: {got:?}"
        );
        assert_eq!(
            got.iter().map(|(a, _)| a.sx).collect::<Vec<_>>(),
            vec![4613, 4620],
            "the spline's own `sx 999` is not the ammo's, and the seven \
             fields after the spline still belong to the ammo"
        );
        assert_eq!(got[1].0.accuracy, 278, "the tail survived the nesting");
    }

    /// **`sz` is the guy's own `z` plus §22's `dz`** — the one residue row
    /// that a second capture turns back into a check (§24.6).
    ///
    /// `crates/sim` has no `z`: `Pos` is two integers, so `sz` cannot be
    /// compared the way `sx` and `sy` are. But `launch::Node` carries the
    /// `dz` §22 measured and *nothing reads it*, which is the worst state
    /// for a constant to be in. run100 dumps the same game at `GUYS=4`,
    /// so the shooter's guy record — position including `z`, `cur_anim`,
    /// `cur_time`, `gpiece` — is on the disk for every one of the nine
    /// launch blocks, and `guy.z + node(gpiece, cur_anim, cur_time).dz` is
    /// run109's `sz` on all nine.
    ///
    /// What that checks is the whole key as well as the value: a row
    /// looked up under the unit type 177 rather than the piece 127 — the
    /// mistake item 396 actually made, which cost a silent no-op — returns
    /// `None` here and the test names it.
    #[test]
    fn run100_and_run109_agree_on_where_the_bow_hand_is_off_the_ground() {
        let (Some(all), Some(r100)) =
            (run109(), dump("gamelog-run100-greatlakes-valuewindow2.txt"))
        else {
            eprintln!("skipping: no run100/run109 capture (set RON_GAMELOG_DIR)");
            return;
        };
        // Each arrow's first block: `cur_time 1`.
        let firsts: Vec<(i64, Ammo)> = all
            .iter()
            .filter(|(_, a, _)| a.cur_time == 1)
            .map(|(f, a, _)| (*f, *a))
            .collect();
        assert_eq!(firsts.len(), 9, "nine launches in the window");
        let mut ix = crate::capture::indexed::IndexedCapture::open(&r100).unwrap();
        let mut rows: Vec<(i64, i64, i64, i64, i64, i64)> = Vec::new();
        for (f, a) in &firsts {
            let at = ix
                .frames()
                .iter()
                .position(|x| x.number == *f)
                .unwrap_or_else(|| panic!("run100 covers block {f}"));
            let body = ix.read_frame(at).unwrap();
            let parsed = Log::parse(&body);
            let block = parsed
                .frames()
                .into_iter()
                .find(|(n, _)| *n == *f)
                .map(|(_, b)| b)
                .unwrap_or_else(|| panic!("run100 block {f}"));
            let (units, _, _) = crate::gamelog::records(block, false);
            let u = units
                .iter()
                .find(|u| u.who == a.who && u.o == a.o)
                .unwrap_or_else(|| panic!("run100 block {f} has {}/{}", a.who, a.o));
            let g = u.guys.first().expect("a Longbowman has one figure");
            let pos = g.pos.expect("GUYS=4 prints the figure's position");
            let (anim, t, piece) = (
                g.cur_anim.expect("cur_anim"),
                g.cur_time.expect("cur_time"),
                g.gpiece.expect("gpiece"),
            );
            let n = sim::launch::node(
                i32::try_from(piece).unwrap(),
                i8::try_from(anim).unwrap(),
                u32::try_from(t).unwrap(),
            )
            .unwrap_or_else(|| {
                panic!(
                    "block {f}: §22's table has no row for piece {piece}, anim \
                     {anim}, starttime {t} — a shot from a piece with no row \
                     leaves from the unit's own square and says nothing"
                )
            });
            rows.push((f - 1, piece, anim, t, pos.z, i64::from(n.dz)));
            assert_eq!(
                pos.z + i64::from(n.dz),
                a.sz,
                "block {f}: run100's guy is at z {} and §22's node is {} above \
                 it, which is not run109's sz {}",
                pos.z,
                n.dz,
                a.sz
            );
        }
        eprintln!("  (launch, gpiece, cur_anim, starttime, guy z, node dz):");
        for r in &rows {
            eprintln!("    {r:?}");
        }
        // Three shooters, three heights, and every one of the six release
        // events the Longbowman has (§22.1) — so the table is exercised
        // whole rather than on one row.
        let mut z: Vec<i64> = rows.iter().map(|r| r.4).collect();
        z.sort_unstable();
        z.dedup();
        assert_eq!(
            z,
            vec![538, 569, 585],
            "three shooters, three heights: {z:?}"
        );
        let mut ev: Vec<(i64, i64)> = rows.iter().map(|r| (r.2, r.3)).collect();
        ev.sort_unstable();
        ev.dedup();
        assert_eq!(
            ev,
            vec![(11, 6), (11, 22), (12, 4), (12, 22), (13, 10), (13, 24)],
            "all six of the Longbowman's release events: {ev:?}"
        );
    }
}
