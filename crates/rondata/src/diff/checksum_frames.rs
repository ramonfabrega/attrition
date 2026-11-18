//! **Step H: the original's fifteen checksums, every frame** (item 4214,
//! `docs/CHECKSUM.md` §24; `docs/DECISIONS.md` entry 132 §7, §10).
//!
//! `CheckSums::check_all@00936560` runs on every turn and prints only in a
//! dump window. A `RON_CHECKSUMS` build of `tools/trace/` writes its
//! fifteen lines every frame ([`crate::trace::checksums`]); this module
//! holds them two ways. First against the record the original printed on
//! the frames a dump window took — the instrument is the record, and on a
//! shifted frame it is not. Then against our walk: the minted game's built
//! start under the pump, our nine emitted lines' Adler on every frame
//! beside the hook's, and per line the first frame it parts. That frame is
//! the score `Checksum: frames`.
//!
//! The pairing is the frame label's (`tools/trace/README.md`): a set the
//! hook tags with trace frame `f` was taken after `Game::do_frame` `f`
//! and its `end_frame`, so it is logger block `f + 1`'s end record, and
//! our walk's state at `sim.frame == f + 1` before that frame's package.
//! The hook's frame `-1` is setup's `check_all`, our frame 0.

use super::minted::{built_start, minted, script};
use crate::checksum::state::held;
use crate::checksum::{CHECKSUM_LINES, Record};
use crate::trace::checksums::{Set, per_frame, sets, summary};

mod record;

/// The 120-frame take: the minted file's playback with the hook and an
/// `@dumpall` on 91 (`docs/RUNS.md` run1577).
const RUN1577: &str = "gamelog-run1577-greatlakes-playback-of-minted-checksums-120.txt";
const RUN1577_TRACE: &str = "rontrace-run1577.log";
/// The whole game: the same file and hook to frame 24,000, trace only
/// (`docs/RUNS.md` run1578).
const RUN1578_TRACE: &str = "rontrace-run1578.log";
/// run1562's dump: the same file and window with a packet, record 3201
/// the instrument reproduces (`docs/CHECKSUM.md` §3).
const RUN1562: &str = "gamelog-run1562-greatlakes-playback-of-minted-packet-91.txt";

/// **The first act's score**: `(record, the hook's trace frame, lines of 15
/// that agree)`, per printed record held against the hook's first set of
/// that frame. 3201 and 3202 are run1562's; run1577 prints both again in
/// its own window. Each record against the hook one frame off is the red
/// half: 3201 against frame 89 and 91, 3202 against 90.
const HOOK_RECORDS: &[(&str, u64, i64, usize)] = &[
    ("run1562", 3201, 90, 15),
    ("run1562", 3201, 89, 9),
    ("run1562", 3201, 91, 10),
    ("run1562", 3202, 91, 15),
    ("run1562", 3202, 90, 10),
    ("run1577", 3201, 90, 15),
    ("run1577", 3202, 91, 15),
];

/// The hook's whole take: `(sets, torn, frames, the first frame, the last,
/// frames with a second set)`. Every frame of 121 has a set; the three
/// with two are the frames before an order's package (29, 59, 89): the
/// read loop's `check_all` before `process_all`, then the turn's own
/// after it.
const HOOK_TAKE: (usize, u32, usize, i64, i64, &[i64]) = (124, 0, 121, -1, 119, &[29, 59, 89]);

fn records(name: &str) -> Option<Vec<Record>> {
    let path = crate::testenv::dump(name)?;
    let file = std::fs::File::open(path).expect("a kept dump opens");
    Some(crate::checksum::records(std::io::BufReader::new(file)).expect("a kept dump reads"))
}

fn hook(name: &str) -> Option<Vec<Set>> {
    let path = crate::testenv::dump(name)?;
    let bytes = std::fs::read(path).expect("a kept trace reads");
    let (written, torn, ok) = summary(&bytes).expect("a RON_CHECKSUMS trace closes with I_CSUM");
    assert!(ok, "the hook's stub passed esp");
    let s = sets(&bytes);
    assert_eq!(s.len(), written as usize, "every set written is read");
    assert_eq!(torn, 0, "no torn set");
    Some(s)
}

/// **The hook is the record**: on the frames a dump window printed, the
/// hook's first set of the frame is the printed record line for line, and
/// one frame off it is not ([`HOOK_RECORDS`]).
#[test]
fn the_hook_s_values_are_the_printed_record_s() {
    let (Some(trace), Some(r1562), Some(r1577)) =
        (hook(RUN1577_TRACE), records(RUN1562), records(RUN1577))
    else {
        eprintln!("skipping: run1577 or run1562 not filed (set RON_GAMELOG_DIR)");
        return;
    };
    let first = per_frame(&trace);
    let frames: std::collections::BTreeMap<i64, usize> =
        trace.iter().fold(Default::default(), |mut m, s| {
            *m.entry(s.frame).or_default() += 1;
            m
        });
    let twice: Vec<i64> = frames
        .iter()
        .filter(|(_, n)| **n > 1)
        .map(|(f, _)| *f)
        .collect();
    let take = (
        trace.len(),
        0,
        frames.len(),
        *frames.keys().next().unwrap(),
        *frames.keys().last().unwrap(),
        twice.as_slice(),
    );
    assert_eq!(take, HOOK_TAKE, "the hook's take");
    // The whole game's take is the same trajectory on the frames both hold.
    if let Some(whole) = hook(RUN1578_TRACE) {
        let key = |s: &Set| (s.frame, s.ordinal, s.lines, s.total);
        let short: std::collections::BTreeSet<_> = trace.iter().map(key).collect();
        let common = whole.iter().filter(|s| s.frame <= 119).map(key);
        let (n, equal) = common.fold((0, 0), |(n, e), k| {
            (n + 1, e + usize::from(short.contains(&k)))
        });
        assert_eq!(
            (n, equal),
            (HOOK_TAKE.0, HOOK_TAKE.0),
            "run1578's first sets are run1577's"
        );
    }
    let mut seen = Vec::new();
    for &(run, number, frame, _) in HOOK_RECORDS {
        let recs = if run == "run1562" { &r1562 } else { &r1577 };
        let printed = recs
            .iter()
            .find(|r| r.number == number)
            .and_then(Record::fifteen)
            .expect("the record prints fifteen lines");
        let set = first[&frame];
        let agree = printed
            .iter()
            .zip(set.lines)
            .filter(|(a, b)| **a == *b)
            .count();
        for (i, (p, h)) in printed.iter().zip(set.lines).enumerate() {
            if *p != h {
                eprintln!(
                    "{run} {number} against frame {frame}: {} printed {p}, hook {h}",
                    CHECKSUM_LINES[i].0
                );
            }
        }
        let total = printed.iter().fold(0u32, |a, v| a.wrapping_add(*v));
        if agree == 15 {
            assert_eq!(set.total, total, "the hook's total is the lines' sum");
        }
        seen.push((run, number, frame, agree));
    }
    assert_eq!(seen, HOOK_RECORDS, "the hook against the records");
}

/// The lines our walk emits, by [`CHECKSUM_LINES`] index: the twelve state
/// lines. Rules and the two script lines are not the simulation's
/// (`docs/DECISIONS.md` entry 119). Walls, ammo and deaths (2, 3, 4) are
/// emitted as the empty walk's Adler alone and refuse a frame that holds an
/// element (item 4355). The deaths line also emits unarmed closes (item 4363).
const EMITTED: [usize; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

/// **The three empty lines' emitter** (item 4355, `docs/CHECKSUM.md` §8):
/// the Adler of no bytes where our walk holds no element of the line, and
/// `None` — a refusal, which the score counts as a parting — where it
/// holds one, since no record shape is emitted. Never
/// [`packets::our_lines`]' three `Line::default()`s, which read 1 whatever
/// the walk holds.
///
/// [`packets::our_lines`]: super::checksum_lines::packets::our_lines
fn empty_walk(held: usize) -> Option<u32> {
    (held == 0).then(|| sim::checksum::Line::default().adler())
}

/// **The score, `Checksum: frames`**, per take: `(trace, frames walked,
/// per emitted line (line, the first of our walk's frames whose Adler parts
/// from the hook's — the take's length when none does — and the frames it
/// parts on inside [`DIAG`]))`. Seen red first against zeros.
///
/// Item 4222 moved two on both takes: groups 61 → 2746 (run1577 whole on
/// 120), a building push normalizing `last_group`'s slot as
/// `equals_group` does; builds 100 → 7383 (run1577 whole on 120), a
/// queue's vacated slots walked as the array holds them (100 → 159), a
/// removed last entry's `job_counter` zeroed (159 → 202), and the tech
/// arm's unqueue after `gain_tech` (202 → 7383).
///
/// Item 4226 moved two on run1578: units 129 → 189 and guys 129 → 6537,
/// both lines walking slots 0 to 8 as `check_units@009371d0` and
/// `check_guys@00937430` do, so the two Wild Birds (`TypeIndex` 402, slot
/// 9) the sampling made on 128 are on neither (run1582 on 129: units 1,147
/// ranges against 1,107 and guys 389 against 375 → the stream's counts).
///
/// Item 4228 moved one on run1578: units 189 → 3166 (none of 0..299
/// parted, from 111), a unit's path stack walking the `size` it grew to
/// rather than its length rounded (`crate`'s `sim::orders::PathStack`).
/// Value diff on 189 (run1586): unit `1/6`, `TypeIndex` 50 (a Citizen),
/// `path.size` ours 10, theirs 20, `path.length` 6 on both → 20.
///
/// Item 4229 moved two on run1578, each to whole on all 24,000: goods 3246
/// → 24,000, `Animal::work@005d7330`'s herd arm setting a seen herd's
/// good's `ever_seen` bit (run1584 on 3246: good 19 `ever_seen` ours 0x00,
/// theirs 0x02 → 0x02 on both); cities 4030 → 24,000, `City::new_caravan`'s
/// bit the partner's index in its owner's cities (run1585 on 4030: city 0
/// `traded_with[1]` ours 4, theirs 2 → 2 on both).
/// Item 4230 moved one on run1578: units 3166 → 3596, the emitter walking
/// a `TRADE_ROUTE` (`TradeOrder::walk_data@00483260`'s three ranges) and a
/// caravan's path taking its road stack's doubled `size`. Value diff on
/// 3166 (run1587): record 14 (`1/14`, its first trade order) ours 1,352
/// ranges, theirs 1,355, from range 536 → whole, its order `ox` 2000,
/// `whom` 1, `uid` 0, `oxx` −1, `whose` −1, `started` 0, `loaded` 0,
/// `uid2` −1 on both; on 3246 (run1584) `1/14`'s `path.size` ours 50,
/// theirs 64 → 64.
///
/// Item 4231 moved one on run1578: groups 2746 → 6781, a `QUEUE_FIRST`
/// move's `get_num_cap` on its record (`action_move_near@00704990+0x52e`)
/// normalizing a group of fewer than four. Value diff on 2746 (run1588):
/// group record 65 (the Scout `1/0`, pushed by `Unit::get_goody_box` on
/// 2745) `role` ours 0, theirs 0x40110, `new_speed` and `speed` ours 0,
/// theirs 34 → the original's on both.
///
/// Item 4234 moved one on run1578: units 3596 → 4030, the emitter walking
/// a `CAST_SPELL` (`CastOrder::walk_data@004860a0`'s three ranges). Value
/// diff on 3596 (run1590): record 16's second order, an untargeted
/// `UNPACK_MERCHANT`, ours 1,437 ranges, theirs 1,440, from range 566 →
/// whole, its `ox` −1, `whom` −1, `uid` 0xffff, `x` −1, `y` −1, `paid` 0,
/// `spell` 0x290 on both. And 4030 → 6537, a caravan's `unit_masks`
/// taking `do_trade@005ed270:470`'s `| 0x200` on its unload (cleared by
/// `add_trade_order` and `end_trade_route`). Value diff on 4030 (run1585):
/// record 14 (`1/14`, the caravan) `unit_masks` ours 262152, theirs 262664
/// → 262664 on both, the line whole.
///
/// Item 4238 moved two on run1578: units 6537 → 6907 and guys 6537 → 7273,
/// one rule, the age's seating (`Leader::gain_tech@006dcb60:2376`'s
/// `Unit::set_new_location(u, x, y, 1, 1)`) writing every figure's `last`
/// through `Guy::set_new_location(…, 1)`. Value diff on 6537 (run1591):
/// the same six figure records on both lines — `1/0` (Scout, TI 69, two
/// figures), `1/12` (Citizen, TI 50), `1/14` (Caravan, TI 59, three) —
/// walking on player 1's age tick 6536; range 187 (guys 58) `last_x` ours
/// 7824, theirs 7848, `last_y` 21648|21672; `1/12` 36584|36607,
/// 23806|23798, `last_z` 339|341; `1/14` 39056|39074, 18478|18460, 44|45
/// → the original's on all six, both lines whole.
///
/// Item 4242 moved one on run1578: groups 6781 → 8206, an army's pushed
/// group taking `Army::add_group@006f8c00`'s `get_num_cap` (`6f8dae`) and
/// with it `normalize` on a group of fewer than four. Value diff on 6781
/// (run1592): group record 64 (army 1/1, the Longbowmen `1/24..1/26`, TI
/// 177, pushed on 6780) `role` ours 0, theirs 0x150c00, `new_speed` and
/// `speed` ours 0, theirs 26 → the original's on both.
///
/// Item 4243 moved one on run1578: units 6907 → 7195, a grouped move's
/// walk (`GroupMoveOrder::walk_data@004855c0`, which
/// `GroupAttackToOrder::walk_data` is whole) handing three ranges after
/// `order.move` the emitter left out. Value diff on 6907 (run1593): theirs
/// 1648 ranges, ours 1639, 781 parted from range 784, record 24 (army 1/1's
/// Longbowman, `GROUP_ATTACK_TO`): the flags byte again 05, `oxx` 24,
/// `whose` 1, `id` 0x697990, `form_id` 0, `group_angle` the move's,
/// `in_group` 0, on each of three members — ours had none of the nine →
/// 1648 on both, the line whole.
///
/// Item 4251 moved one on run1578: units 7195 → 7273, `unit_masks` bit
/// `0x4` (`Unit::set_in_danger@005fcfb0`'s latch, which the crate held
/// and the emitter left out). Value diff on 7195 (run1600): 3 of 1697
/// ranges from 832, records 27, 28, 29 (player 1's Longbowmen, TI 177)
/// `unit_masks` ours 0x40000, 0x4000a, 0x4000a, theirs 0x40004, 0x4000e,
/// 0x4000e → the original's on all three, the line whole.
///
/// Item 4244 moved one on run1578: builds 7383 → 8281, a mine keeping
/// `MiningList::mtn`/`cliff` (`find_gather_tcoords@0063bdc0`'s store,
/// `Build::close`'s reset). Value diff on 7383 (run1597's record): builds
/// range 903, record 2019 (1/2019, a Mine, TI 0x1a3) `mtn` ours −1, theirs
/// 0 → 0 on both.
///
/// Item 4253 moved two on run1578: units 7273 → 8281 and guys 7273 →
/// 10541, one rule, a scholar's seating (`Unit::go_inside@0061a2e0`'s
/// `set_angle(0, _, 1)`, `61a547`) setting its figure's `angle` and
/// `last_angle` rather than turning it. Value diff on 7273 (run1597's
/// record): units range 891 and guys range 279, record 30 (`1/30`, a
/// Scholar, TI 52, seated in its University on 7272) `angle` and
/// `last_angle` ours 0x55555555, theirs 0 → 0 on both, both lines whole.
///
/// Item 4255 moved one on run1578: groups 8206 → 9810, a push running
/// `Group::equals_group@00708000`'s `get_num` (vslot `+0x4`) on
/// `last_group`'s record after its `normalize`, whose empty arm zeroes
/// `ox`/`oy`. Value diff on 8206 (run1597's record): groups range 91,
/// record 67 (player 1's slot 3, emptied of the Longbowmen `1/39..1/41`,
/// TI 177, since their move on 8171) `ox`/`oy` ours 30552/21144, theirs
/// 0/0 (theirs 30552/21144 on 8205, when player 1's push into slot 65
/// follows) → 0/0 on both.
///
/// Item 4257 moved two on run1578: units 8281 → 10541 and builds 8281 →
/// 14312, one rule, the garrison chain's order
/// (`Object::insert_inside@00647e90` appends at the bottom). Value diff on
/// 8281 (run1597's record): the University 1/2018 holds two Scholars (TI
/// 52), 1/30 then 1/42. Builds range 856, record 2018 `inside_down` ours
/// 42, theirs 30; units range 842, record 30 `inside_down` ours −1, theirs
/// 42 (and record 42's `inside_up`, ours 2018, theirs 30) → the
/// original's on both, both lines whole there.
///
/// Item 4258 moved one on run1578: groups 9810 → 24000, the line whole on
/// every frame of the minted game. `Group::add@00714350`'s in-use arm ORs
/// the new member's type `role` into the record's `+0x34`; `seat_add` did
/// not. Value diff on 9810 (run1597's record): groups range 76, record 64
/// (army 1/1, 18 Longbowmen joined on 9809 by the Hoplites `1/56..1/58`, TI 132)
/// `role` ours 0x150c00, theirs 0x150c03 (theirs 0x150c00 on 9809) →
/// 0x150c03 on both.
///
/// Item 4260 moved two on run1578: units 10541 → 11884 and guys 10541 →
/// 11884, one rule, a trackless crew figure's `Guy::move` head running
/// after guy 0's, whose `turn_towards → Guy::do_turn@005d97a0` has
/// already turned it. Value diff on 10541 (run1597's record): units
/// ranges 570/571 and guys 160/161, the Caravan 1/14 (TI 59, land),
/// guys 1 and 2 `last_angle` ours 682461312, theirs 968792448 (guy 0
/// 682461312 on both) → 968792448 on both, both lines whole there.
///
/// Item 4262 moved one on run1578: builds 14312 → 24000, the line whole on
/// every frame of the minted game. `BuildData::wonder` (`+0x76`) is the
/// entry `Wonders::init_wonder@0073c860` returned to `Build::activate`'s
/// wonder arm; the emitter wrote −1 on every building. Value diff on 14312
/// (run1597's record): builds range 1258, record 2023 (player 1's
/// Pyramids, TI 0x20e) `wonder` ours −1, theirs 0 → 0 on both.
///
/// Item 4263 moved two on run1578: units 11884 → 12537 and guys 11884 →
/// whole on all 24,000, one rule, `Unit::set_type@00612fa0`'s seating
/// (`Guy::set_new_location(guy i, x, y, 1)`) writing each figure's `last`
/// on its point. Value diff on 11884 (run1597's record): units ranges
/// 1570/1595/1618 and guys 463/470/477, the Hoplites 1/56..1/58 (TI 132,
/// land) upgraded to Phalanx (TI 133) on a frame they walked; record 56's
/// `last_x`/`last_y` ours (38275, 21337), theirs (38297, 21325), its own
/// point → (38297, 21325) on both, both lines whole there.
///
/// Item 4266 moved one on run1578: units 12537 → 24000, the line whole on
/// every frame of the minted game. `GuardOrder::walk_data@00486550`'s three
/// ranges were not handed over (the order walker's `GUARD` arm). Value
/// diff on 12537 (run1597's record): units range 1539, record 62 (the
/// Longbowmen 1/62, TI 177, land, guarding 1/61), theirs `00486573` `04`
/// (the flags byte) against ours' `guys.length` 1; theirs' three ranges
/// `04`, (61, 1, uid 84), (0, 264, 42840, 18408, 0, 0) → the same three
/// on both, equal to the order this crate already held.
///
/// Item 4298 moved one on both takes: leaders parts on 11 of run1577's
/// 120 frames (0 and 100–109) where it parted on all 120, and on 152 of
/// run1578's first 300 (0, 100–109, 159–299) where it parted on all 300,
/// its first still frame 0 on both. The release of `hire_stamp`, `hire_who` and `tributes`: emitting
/// `hire_who`'s −1 per slot (`Leader::init`'s fill) makes each leader's
/// long range agree where nothing else in it parts; the record's first
/// parting was leader 0's `+0x1cc` (ours 0, theirs `ffffffff`) on every
/// frame of [`RECORD_PARTINGS`]. The line is still not counted whole while
/// a leaf is held (`LEADERS_HELD_OUT`).
///
/// Item 4320 moved leaders inside 300 on both: run1577 11 → 2 (0, 101),
/// run1578 152 → 142 (0, 101, 160–299), the score's dirty bits emitted.
///
/// Item 4332 moved leaders inside 300 on both: run1577 2 → 1 (0), run1578
/// 142 → 4 (0, 167, 202, 297), `Unit::think`'s citizen bit (`0x80000`)
/// emitted; 167, 202 and 297 part on byte 3 alone, `0x8000000` and
/// `0x4000000`, which the `0x80000` parting had hidden.
type FramesRow = (&'static str, i64, [(&'static str, i64, usize); 12]);
const CHECKSUM_FRAMES: &[FramesRow] = &[
    (
        RUN1577_TRACE,
        120,
        [
            ("units", 120, 0),
            ("builds", 120, 0),
            // Item 4355: emitted as the empty walk, none held on any frame.
            ("walls", 120, 0),
            ("ammo", 120, 0),
            ("deaths", 120, 0),
            ("groups", 120, 0),
            ("guys", 120, 0),
            // Item 4340: whole (0 parted; it was frame 0 alone).
            ("leaders", 120, 0),
            ("cities", 120, 0),
            ("items", 120, 0),
            ("goods", 120, 0),
            ("world", 120, 0),
        ],
    ),
    (
        RUN1578_TRACE,
        24000,
        [
            ("units", 24000, 0),
            ("builds", 24000, 0),
            // Item 4355: emitted as the empty walk, none held on any frame.
            ("walls", 24000, 0),
            ("ammo", 24000, 0),
            ("deaths", 24000, 0),
            ("groups", 24000, 0),
            ("guys", 24000, 0),
            // Item 4333: inside 300, 4 → 1 (0 alone; 167, 202, 297 whole).
            // Item 4340: whole (frame 0's setup census and stats bits).
            ("leaders", 24000, 0),
            ("cities", 24000, 0),
            ("items", 24000, 0),
            ("goods", 24000, 0),
            ("world", 24000, 0),
        ],
    ),
];

/// The frames on which every emitted line is taken and each parting is
/// counted; past them a line is taken only until it first parts, so a
/// 24,000-frame take costs the lines still whole — and the leaders line
/// ([`LEADERS_K`]), taken on every frame for [`LEADERS_AGREE`].
const DIAG: i64 = 300;

/// The leaders line's place in [`EMITTED`].
const LEADERS_K: usize = 7;

/// **Parity's leaders row** (item 4317, `docs/CHECKSUM.md` §25.4;
/// `docs/DECISIONS.md` entry 138 §2): per take, `(trace, the frames on
/// which our leaders line's Adler is the hook's, the frames walked)`.
/// [`DIAG`] counts the first 300 only; this counts the take. Minted as
/// found on the tree after item 4298: run1578's line agrees on 1..99,
/// 110..158 and islands between partings to 3375, and parts on 0 and on
/// every frame from 3376 (`reg_unpack_merch[1]` from 3380,
/// `resources_controlled` from 3780; the census is item 4317's journal).
/// Item 4320: run1578 2,941 → 3,006, run1577 109 → 118, the score's two
/// dirty bits (`0x800000`, `0x1000000`) emitted; 100–109 and 159 agree
/// but 101 and 160, which part on theirs' `0x80000` alone.
/// Item 4324: run1578 3,006 → 19,006, `resources_controlled` (`+0x850`)
/// counted off `rare` (it was 0); run1577 stands at 118.
/// Item 4328: run1578 19,006 → 23,594, `reg_unpack_merch` (`+0x5d4`)
/// leaves a packed merchant out (it counted every merchant); run1577
/// stands at 118.
/// Item 4332: run1578 23,594 → 23,965, run1577 118 → 119, `Unit::think`'s
/// citizen bit (`leader_flags & 0x80000`) held and emitted (it was not).
/// Item 4333: run1578 23,965 → 23,999, the stats bits (`0x8000000`,
/// `0x4000000`) emitted, `gain_tech`'s unit arm raising `0x4000000` for
/// the Militia line alone and the setup's tree raising neither; every
/// frame but 0, whose census leaves stand; run1577 stands at 119.
const LEADERS_AGREE: &[(&str, usize, usize)] =
    &[(RUN1577_TRACE, 120, 120), (RUN1578_TRACE, 24000, 24000)];

/// **Parity's empty-lines row** (item 4355, `docs/CHECKSUM.md` §24.3;
/// `docs/DECISIONS.md` entry 140 §1): per take, `(trace, the frames on
/// which our walk holds no wall, no shot in flight and no death object
/// when the other lines are taken, the frames walked)`. The hook's walls,
/// ammo and deaths read 1 (the Adler of no bytes) on every set of both
/// takes. Minted at 0, red first, and read as none on every frame of both:
/// run1577 0 → 120 of 120, run1578 0 → 24,000 of 24,000 (theirs 1 on 124
/// and 24,004 sets).
const EMPTY_HELD: &[(&str, usize, usize)] =
    &[(RUN1577_TRACE, 120, 120), (RUN1578_TRACE, 24000, 24000)];

/// The Adler of each emitted line `want` names, on `sim`, in [`EMITTED`]'s
/// order (`None` where not wanted). The builds line needs the data layer's
/// pieces, so it takes every line through [`packets::our_lines`].
///
/// [`packets::our_lines`]: super::checksum_lines::packets::our_lines
fn our_adlers(
    sim: &sim::Sim,
    pieces: &super::checksum_lines::packets::Pieces,
    want: &[bool; 12],
) -> [Option<Option<u32>>; 12] {
    let h = held(sim);
    let lines = want[1].then(|| super::checksum_lines::packets::our_lines(sim, pieces));
    let mut out = [None; 12];
    for (k, &i) in EMITTED.iter().enumerate() {
        if !want[k] {
            continue;
        }
        out[k] = Some(match (i, &lines) {
            (2..=3, _) => empty_walk(h[i - 2]),
            (4, _) => sim.checksum_deaths().map(|line| line.adler()),
            (_, Some(l)) => Some(l[i].adler()),
            _ => Some(
                match i {
                    0 => sim.checksum_units(),
                    5 => sim.checksum_groups(),
                    6 => sim.checksum_guys(),
                    7 => sim.checksum_leaders(),
                    8 => sim.checksum_cities(),
                    9 => sim.checksum_items(),
                    10 => sim.checksum_goods(),
                    11 => sim.checksum_world(),
                    _ => unreachable!("builds goes through our_lines"),
                }
                .adler(),
            ),
        });
    }
    out
}

/// One emitted line's walk against a hook: the first frame it parts, both
/// sides' Adler there, the frames it parts on inside [`DIAG`], and the
/// frames it was taken on and agreed.
type LineWalk = (&'static str, i64, u32, u32, Vec<i64>, usize);

/// What a walk held of the three empty lines' elements: the frames it held
/// none on, and the first frame it held one with [`held`]'s counts there.
type HeldWalk = (usize, Option<(i64, [usize; 3])>);

/// The walk under the pump against a hook's sets, per emitted line, and
/// [`held`] on every frame a set pairs with.
fn first_partings(trace: &[Set], length: i64) -> Option<(Vec<LineWalk>, HeldWalk)> {
    let (_, mut built, _) = built_start()?;
    let pieces = super::checksum_lines::packets::Pieces::read()?;
    let mut stream = crate::input::Stream::pump(&minted(&script())?);
    let first = per_frame(trace);
    let mut out: Vec<Option<(i64, u32, u32)>> = vec![None; EMITTED.len()];
    let mut parted: Vec<Vec<i64>> = vec![Vec::new(); EMITTED.len()];
    let mut agreed = vec![0usize; EMITTED.len()];
    let mut empty = 0usize;
    let mut first_held = None;
    while built.sim.frame < length {
        let f = built.sim.frame;
        let want: [bool; 12] =
            std::array::from_fn(|k| f < DIAG || out[k].is_none() || k == LEADERS_K);
        if !want.iter().any(|w| *w) {
            break;
        }
        if let Some(set) = first.get(&(f - 1)) {
            let h = held(&built.sim);
            if h == [0; 3] {
                empty += 1;
            } else if first_held.is_none() {
                first_held = Some((f, h));
            }
            let ours = our_adlers(&built.sim, &pieces, &want);
            for (k, &i) in EMITTED.iter().enumerate() {
                let Some(o) = ours[k] else { continue };
                // A refusal (an empty line on a walk that holds an element)
                // parts, and prints as ours 0.
                if o == Some(set.lines[i]) {
                    agreed[k] += 1;
                } else {
                    if f < DIAG {
                        parted[k].push(f);
                    }
                    if out[k].is_none() {
                        out[k] = Some((f, o.unwrap_or(0), set.lines[i]));
                    }
                }
            }
        }
        stream.apply(f as i32, &mut built);
        built.tick();
    }
    Some((
        EMITTED
            .iter()
            .zip(out)
            .zip(parted)
            .zip(agreed)
            .map(|(((&i, o), p), n)| {
                let (f, a, b) = o.unwrap_or((length, 0, 0));
                (CHECKSUM_LINES[i].0, f, a, b, p, n)
            })
            .collect(),
        (empty, first_held),
    ))
}

/// Frames as runs, `a-b` for a run of more than one.
fn spans(frames: &[i64]) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < frames.len() {
        let mut j = i;
        while j + 1 < frames.len() && frames[j + 1] == frames[j] + 1 {
            j += 1;
        }
        out.push(if i == j {
            frames[i].to_string()
        } else {
            format!("{}-{}", frames[i], frames[j])
        });
        i = j + 1;
    }
    out.join(" ")
}

/// **Our lines, every frame, against the hook's.** The score: per take
/// and emitted line, the first frame it parts and the frames it parts on
/// inside [`DIAG`] ([`CHECKSUM_FRAMES`]), each first parting printed with
/// both sides' Adler.
#[test]
fn our_lines_part_from_the_hook_s_on_their_first_frame() {
    for &(name, length, pinned) in CHECKSUM_FRAMES {
        let Some(trace) = hook(name) else {
            eprintln!("skipping: {name} not filed (set RON_GAMELOG_DIR)");
            continue;
        };
        let Some((walked, (empty, first_held))) = first_partings(&trace, length) else {
            eprintln!("skipping: no install or lobby dump (RON_INSTALL, RON_GAMELOG_DIR)");
            return;
        };
        // The hook's three empty lines are the Adler of no bytes on every
        // set of the take (entry 140 §1, the pass's read), so the row is
        // ours alone.
        let ones = trace
            .iter()
            .filter(|s| s.lines[2..5].iter().all(|&a| a == 1))
            .count();
        eprintln!(
            "EMPTY HELD {name}: ours none on {empty} of {length} frames, first held {first_held:?} \
             ([walls, ammo, deaths]); theirs 1 on {ones} of {} sets",
            trace.len()
        );
        assert_eq!(ones, trace.len(), "theirs walk no wall, shot or death");
        for (line, f, ours, theirs, p, _) in &walked {
            eprintln!(
                "CHECKSUM FRAMES {name} {line}: first parts on {f} (ours {ours}, theirs {theirs}); \
                 inside {DIAG}: {} frames, {}",
                p.len(),
                spans(p)
            );
        }
        let seen: Vec<(&str, i64, usize)> = walked
            .iter()
            .map(|(l, f, _, _, p, _)| (*l, *f, p.len()))
            .collect();
        let agree = (walked[LEADERS_K].5, length as usize);
        eprintln!("LEADERS AGREE {name}: {} of {} frames", agree.0, agree.1);
        let want = LEADERS_AGREE
            .iter()
            .find(|r| r.0 == name)
            .map(|r| (r.1, r.2));
        assert_eq!(walked[LEADERS_K].0, "leaders", "LEADERS_K names the line");
        assert_eq!(seen, pinned, "CHECKSUM_FRAMES {name}");
        assert_eq!(Some(agree), want, "LEADERS_AGREE {name}");
        let row = EMPTY_HELD.iter().find(|r| r.0 == name).map(|r| (r.1, r.2));
        assert_eq!(
            Some((empty, length as usize)),
            row,
            "EMPTY_HELD {name}: first held {first_held:?}"
        );
    }
}

/// **The held count can part** (item 4355): the classical melee
/// (`docs/GOLDEN.md` §150, run1449's start under its staged script) holds
/// a shot — the Kushite Archers `0/14`'s round at the Phalanx `1/19`,
/// loosed on 673 (`classical_melee_s_kushite_archers_loose_from_the_bow`)
/// — and [`held`] reads it. Per count, the first frame of `[0, MELEE_TO)`
/// it reads non-zero, `None` where it never does. A count that cannot
/// part on a walk that holds one is a blind instrument. Minted at none;
/// read as a shot from 642 (fourteen at most inside the window) and a
/// death object from 692 (two at most). No wall stands in the window, so
/// the walls arm is held by a building moved into the 3000 band instead.
const MELEE_HELD: [Option<i64>; 3] = [None, Some(642), Some(692)];

/// The melee walked to this frame.
const MELEE_TO: i64 = 720;

#[test]
fn the_held_count_reads_the_melee_s_shot() {
    use super::classical_melee::{MELEE_LONG, MELEE_START, melee_script};
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(sib), Some(tr)) = (
        crate::testenv::dump(MELEE_LONG.0),
        crate::testenv::dump(MELEE_START),
        super::testkit::trace(MELEE_LONG.1),
    ) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", MELEE_LONG.0);
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let sib_text = crate::capture::read(&sib);
    let log = crate::gamelog::Log::parse(&text);
    let sib_log = crate::gamelog::Log::parse(&sib_text);
    let sib_init = sib_log.initial().expect("a start dump");
    let mut init = log.initial().unwrap();
    super::borrow_from_siblings(&mut init, &[&sib_init]);
    super::setup::borrow_pasture(&mut init, &tr);
    let mut built = super::setup::with_staged_script(melee_script(), || {
        super::build_sim(&loaded, &init, sim::Tuning::RON)
    });
    let mut seen: [Option<i64>; 3] = [None; 3];
    let mut most = [0usize; 3];
    // The emitter on the frames each count first reads: the empty walk's
    // Adler is refused there, as a parting.
    let mut emitted: [Option<Option<u32>>; 3] = [None; 3];
    while built.sim.frame < MELEE_TO {
        let h = held(&built.sim);
        for k in 0..3 {
            if h[k] > 0 && seen[k].is_none() {
                seen[k] = Some(built.sim.frame);
                emitted[k] = Some(if k == 2 {
                    built.sim.checksum_deaths().map(|line| line.adler())
                } else {
                    empty_walk(h[k])
                });
            }
            most[k] = most[k].max(h[k]);
        }
        built.tick();
    }
    eprintln!("MELEE HELD: first {seen:?}, most {most:?} ([walls, ammo, deaths])");
    assert_eq!(seen, MELEE_HELD, "MELEE_HELD");
    assert_eq!(
        emitted,
        [None, Some(None), Some(None)],
        "the empty lines refuse a walk that holds a shot or a death"
    );
    assert_eq!(
        empty_walk(0),
        Some(1),
        "the empty walk is the Adler of no bytes"
    );
    // The walls arm: a live building in the wall band is one wall, and a
    // dead one is none (`check_walls`' `flags & 1`).
    let mut b = built.sim.buildings[0].clone();
    assert!(
        b.alive && b.index < sim::WALL_BASE,
        "a city building to move"
    );
    b.index = sim::WALL_BASE;
    built.sim.buildings.push(b);
    let walls = held(&built.sim)[0];
    built.sim.buildings.last_mut().unwrap().alive = false;
    assert_eq!((walls, held(&built.sim)[0]), (1, 0), "the walls arm");
}

/// The instrument's streams of the minted game's packets (runs 1579 to
/// 1586), outside git: `<run>/values.tsv` and `<run>/stream.tsv`.
fn minted_packets_dir() -> String {
    std::env::var("RON_MINTED_PACKETS_DIR").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        format!("{home}/ron-data/lab-experiments/2026-11-13-item-4214")
    })
}

/// **Where each first parting is**: per packet, `(run, its trace, our
/// frame, per state line that parts (line, the stream's ranges, ours,
/// ranges parted, the first parting as `range record field: ours |
/// theirs`, each side's bytes in hex cut to 32 digits))`. Each packet
/// stands on a frame [`CHECKSUM_FRAMES`] names as a line's first parting;
/// a line that agrees there is not listed. run1589 stands on 2745, the
/// frame before groups' 2746, whose tick item 4231 read under `step4.py`.
/// The leaders line's first parting was leader 0's `+0x1cc` (theirs
/// `ffffffff`, ours 0: the unheld `hire_who`'s −1s) from item 4159's
/// sixteenth residue, which put `score` on the line (until then `score` at
/// `+0x10`, ours 0, unheld), to item 4298, which emits `hire_who`: since
/// then each packet parts on one or two ranges fewer, seven agree whole
/// on the line, and the rest name the next parting (`flags`,
/// `resources_controlled` at `+0x850`, `reg_unpack_merch[1]` at `+0x5d8`).
/// The line is still not counted whole while a leaf is held.
type PacketRow = (&'static str, usize, usize, usize, &'static str);
pub(in crate::diff) const MINTED_PACKETS: &[(&str, &str, i64, &[PacketRow])] = &[
    (
        "run1579",
        "rontrace-run1579.log",
        1,
        &[
            // Item 4298: leaders whole; before it, 2 of 267 ranges parted, first `1 0
            // who..ally_mask: +0x1cc of 26914|26914: 00000000|ffffffff` (the unheld `hire_who`'s
            // −1s).
        ],
    ),
    (
        "run1582",
        "rontrace-run1582.log",
        129,
        &[
            // Item 4298: leaders whole; before it, 2 of 267 ranges parted, first `1 0
            // who..ally_mask: +0x1cc of 26914|26914: 00000000|ffffffff` (the unheld `hire_who`'s
            // −1s).
        ],
    ),
    (
        "run1586",
        "rontrace-run1586.log",
        189,
        // Item 4228: units whole; before it, 1 of 1,127 ranges, range 303,
        // unit `1/6` (`TypeIndex` 50, a Citizen) `path.size` 10|20.
        &[
            // Item 4298: 3 → 1 parted (`hire_who`'s −1s emitted); before it, first `0 0 flags: +0x2
            // of 8|8: 00000000|08000000`.
            // Item 4332: leaders whole (`Unit::think`'s citizen bit,
            // `0x80000`, emitted); before it, 1 of 267 parted, `0 0 flags:
            // +0x2 of 8|8: 00000000|08000000` (leader 0's idle citizen).
        ],
    ),
    (
        "run1587",
        "rontrace-run1587.log",
        3166,
        // Item 4230: units whole; before it, 1355 ranges theirs, 1352 ours,
        // 660 parted from range 536: record 14's `TRADE_ROUTE`, whose
        // `TradeOrder::walk_data` hands over three ranges the emitter left
        // out (`order.flags`, `order.target`, `order.trade`).
        &[
            // Item 4298: 3 → 1 parted (`hire_who`'s −1s emitted); before it, first `1 0
            // who..ally_mask: +0x1cc of 26914|26914: 00000000|ffffffff`.
            // Item 4320: leaders whole (the score's units bit `0x800000`
            // emitted); before it, 1 of 267 parted, first `128 1 flags: +0x2
            // of 8|8: 00000000|80000000`.
        ],
    ),
    (
        // Item 4231: the frame before groups' 2746, the push's tick still
        // ahead (`step4.py`'s packet; `docs/RUNS.md` run1589). Every line
        // but leaders agrees.
        "run1589",
        "rontrace-run1589.log",
        2745,
        &[
            // Item 4298: leaders whole; before it, 2 of 267 ranges parted, first `1 0
            // who..ally_mask: +0x1cc of 26914|26914: 00000000|ffffffff` (the unheld `hire_who`'s
            // −1s).
        ],
    ),
    (
        "run1588",
        "rontrace-run1588.log",
        2746,
        // Item 4231: groups whole; before it, 1 of 544 ranges, range 83,
        // group record 65 (the Scout `1/0`, pushed on 2745) `role` 0|0x40110,
        // `new_speed` 0|34, `speed` 0|34.
        &[
            // Item 4298: leaders whole; before it, 2 of 267 ranges parted, first `1 0
            // who..ally_mask: +0x1cc of 26914|26914: 00000000|ffffffff` (the unheld `hire_who`'s
            // −1s).
        ],
    ),
    (
        "run1590",
        "rontrace-run1590.log",
        3596,
        // Item 4234: units whole; before it, 1440 ranges theirs, 1437 ours,
        // 710 parted from range 566: record 16's `CAST_SPELL`
        // (`UNPACK_MERCHANT`), whose `CastOrder::walk_data@004860a0` hands
        // over three ranges the emitter left out (`order.flags`,
        // `order.target`, `order.cast`).
        &[
            // Item 4298: 2 → 1 parted (`hire_who`'s −1s emitted); before it, first `1 0
            // who..ally_mask: +0x1cc of 26914|26914: 00000000|ffffffff`.
            // Item 4328: leaders whole (`reg_unpack_merch`, `+0x5d4`, leaves a
            // packed merchant out); before it, 1 of 267 ranges parted, first
            // `129 1 who..ally_mask: +0x5d0 of 26914|26914: 02000000|00000000`.
        ],
    ),
    (
        "run1591",
        "rontrace-run1591.log",
        6537,
        // Item 4238: units and guys whole; before it, units 6 of 1546
        // ranges from 187, guys 6 of 511 from 58 — the same six figure
        // records on both lines (`1/0` Scout TI 69 ×2, `1/12` Citizen TI 50,
        // `1/14` Caravan TI 59 ×3), `last_x`/`last_y` (and `last_z` on the
        // last two) one step behind: range 187 `last_x` 7824|7848, `last_y`
        // 21648|21672. Player 1's age on tick 6536: `gain_tech:2376`'s
        // seating writes every figure's `last`.
        &[
            // Item 4298: 3 → 2 parted (`hire_who`'s −1s emitted); before it, first `1 0
            // who..ally_mask: +0x1cc of 26914|26914: 00000000|ffffffff`.
            // Item 4324: 2 → 1 (`resources_controlled` counted). Item
            // 4320: ours `0x1000000` (`gain_tech`'s bit), `02000000` before.
            // Item 4333: leaders whole (the stats bits emitted); before it,
            // 1 of 267 parted, `128 1 flags: +0x3 of 8|8:
            // 03000000|0f000000`.
        ],
    ),
    (
        "run1592",
        "rontrace-run1592.log",
        6781,
        // Item 4242: groups whole; before it, 1 of 544 ranges, range 76,
        // group record 64 (army 1/1, the Longbowmen `1/24..1/26`, TI 177,
        // pushed on 6780) `role` 0|0x150c00, `new_speed` 0|26, `speed`
        // 0|26: `Army::add_group@006f8c00`'s `get_num_cap`.
        &[
            // Item 4298: 3 → 2 parted (`hire_who`'s −1s emitted); before it, first `1 0
            // who..ally_mask: +0x1cc of 26914|26914: 00000000|ffffffff`.
            // Item 4324: 2 → 1 (`resources_controlled` counted). Item
            // 4320: byte 2 agrees (`0x800000` emitted). Item 4333: leaders
            // whole (`0x8000000` emitted); before it, 1 of 271 parted,
            // `128 1 flags: +0x3 of 8|8: 02000000|0a000000`.
        ],
    ),
    (
        "run1593",
        "rontrace-run1593.log",
        6907,
        // Item 4243: units whole; before it, theirs 1648 ranges, ours 1639,
        // 781 parted from range 784, record 24 (a Longbowman of army 1/1,
        // `GROUP_ATTACK_TO`): `GroupMoveOrder::walk_data@004855c0` hands
        // three ranges after `order.move` the emitter left out — the flags
        // byte again (05), `oxx`..`group_angle` (24, 1, 0x697990, 0, the
        // move's angle) and `in_group` (0) — on each of three members.
        &[
            // Item 4324: leaders whole (`resources_controlled`, `+0x850`, counted off
            // `rare`); before it, 1 of 271 ranges parted, first `129 1 who..ally_mask:
            // +0x848 of 26914|26914: 00000000|03000000`. Item 4298: 2 → 1 (`hire_who`).
        ],
    ),
    (
        "run1600",
        "rontrace-run1600.log",
        7195,
        // Item 4251: units whole; before it, 3 of 1697 ranges from 832,
        // records 27, 28, 29 (player 1's Longbowmen, TI 177) `unit_masks`
        // 262144|262148 (and 0x4000a|0x4000e on the other two): the
        // emitter left out bit 0x4, `Unit::set_in_danger@005fcfb0`'s latch
        // the crate already held.
        &[
            // Item 4324: leaders whole (`resources_controlled`, `+0x850`, counted off
            // `rare`); before it, 1 of 271 ranges parted, first `129 1 who..ally_mask:
            // +0x848 of 26914|26914: 00000000|03000000`. Item 4320: 2 → 1 (`0x800000`);
            // item 4298: 3 → 2 (`hire_who`).
        ],
    ),
    // Item 4343: the panel's idle arms' packets (`docs/CLIENT_INPUT.md`
    // §32.8), at 13000 (the Pyramids sited) and 14400 (built); on no
    // first parting, wired here so that the playback's lockstep there is
    // held, not assumed.
    ("run1652", "rontrace-run1652.log", 13000, &[]),
    ("run1653", "rontrace-run1653.log", 14400, &[]),
    (
        "run1583",
        "rontrace-run1583.log",
        987,
        // Item 4228: units whole; before it, 4 ranges, range 177 `path.size`
        // 10|50.
        &[
            // Item 4298: leaders whole; before it, 2 of 267 ranges parted, first `1 0
            // who..ally_mask: +0x1cc of 26914|26914: 00000000|ffffffff` (the unheld `hire_who`'s
            // −1s).
        ],
    ),
    (
        "run1584",
        "rontrace-run1584.log",
        3246,
        &[
            // Item 4230: units whole. Item 4228: 661 → 656 parted, range
            // 167 `path.size` 20|60 → agreeing, the first range 524, record
            // 14 `path.size` 50|64 — the road stack's doubled allocation
            // `do_trade`'s `operator=` hands the caravan — and then the
            // `TradeOrder`'s three unwalked ranges (1386 ours, 1389 theirs).
            // Item 4298: leaders whole; before it, 2 of 267 ranges parted, first `1 0
            // who..ally_mask: +0x1cc of 26914|26914: 00000000|ffffffff` (the unheld `hire_who`'s
            // −1s).
        ],
    ),
    (
        "run1585",
        "rontrace-run1585.log",
        4030,
        &[
            // Item 4228: 809 → 804 parted, range 167 `path.size` 10|60 →
            // agreeing. Item 4230: 804 → 1, the `TradeOrder`'s three ranges
            // walked (1504 → 1507 ours). Item 4234: units whole; before it,
            // 1 of 1507, range 492, record 14 (`1/14`, the caravan)
            // `unit_masks` 262152|262664, `do_trade:470`'s `| 0x200` on its
            // unload.
            // Item 4298: 2 → 1 parted (`hire_who`'s −1s emitted); before it, first `1 0
            // who..ally_mask: +0x1cc of 26914|26914: 00000000|ffffffff`.
            // Item 4328: leaders whole (`reg_unpack_merch`, `+0x5d4`, leaves a
            // packed merchant out); before it, 1 of 267 ranges parted, first
            // `129 1 who..ally_mask: +0x5d0 of 26914|26914: 03000000|01000000`.
        ],
    ),
    (
        "run1580",
        "rontrace-run1580.log",
        61,
        &[
            // Item 4298: leaders whole; before it, 2 of 267 ranges parted, first `1 0
            // who..ally_mask: +0x1cc of 26914|26914: 00000000|ffffffff` (the unheld `hire_who`'s
            // −1s).
        ],
    ),
    (
        "run1581",
        "rontrace-run1581.log",
        100,
        &[
            // Item 4298: 3 → 1 parted (`hire_who`'s −1s emitted); before it, first `1 0
            // who..ally_mask: +0x1cc of 26914|26914: 00000000|ffffffff`.
            // Item 4320: leaders whole (the score's units bit `0x800000`
            // emitted); before it, 1 of 267 parted, first `128 1 flags: +0x2
            // of 8|8: 00000000|80000000`.
        ],
    ),
];

/// One line's first parting as [`MINTED_PACKETS`] pins it: `range record
/// field`, then the fields that part as `name ours|theirs` where the line
/// has a decoder, else the first byte that parts and four bytes from it
/// on each side, in hex.
fn first_text(line: &str, f: &(usize, i16, String, String, String)) -> String {
    let bytes = |h: &str| -> Vec<u8> {
        (0..h.len() / 2)
            .filter_map(|i| u8::from_str_radix(&h[2 * i..2 * i + 2], 16).ok())
            .collect()
    };
    let (o, t) = (bytes(&f.3), bytes(&f.4));
    let head = format!("{} {} {}", f.0, f.1, f.2);
    let decoded = (o.len() == t.len() && !o.is_empty())
        .then(|| {
            let a = super::checksum_lines::range_fields(line, &f.2, &o)?;
            let b = super::checksum_lines::range_fields(line, &f.2, &t)?;
            Some(
                a.iter()
                    .zip(&b)
                    .filter(|(x, y)| x.1 != y.1)
                    .map(|(x, y)| format!("{} {}|{}", x.0, x.1, y.1))
                    .collect::<Vec<_>>(),
            )
        })
        .flatten()
        .filter(|d| !d.is_empty());
    if let Some(d) = decoded {
        return format!("{head}: {}", d.join(", "));
    }
    let at = o
        .iter()
        .zip(&t)
        .position(|(x, y)| x != y)
        .unwrap_or(o.len().min(t.len()));
    let cut = |b: &[u8]| {
        b.iter()
            .skip(at)
            .take(4)
            .map(|x| format!("{x:02x}"))
            .collect::<String>()
    };
    format!(
        "{head}: +{at:#x} of {}|{}: {}|{}",
        o.len(),
        t.len(),
        cut(&o),
        cut(&t)
    )
}

/// **The packets name the partings.** Our walk under the pump to each
/// packet's frame, its twelve lines held range by range against the
/// packet's stream ([`MINTED_PACKETS`]); and the hook's set on the same
/// take is the instrument's values on that frame, 15 of 15, and so is
/// run1578's on the same frame.
#[test]
fn the_minted_packets_name_each_first_parting() {
    use super::checksum_lines::packets::{STATE_LINES, compare, our_lines, packet_stream_in};
    let dir = minted_packets_dir();
    let whole = hook(RUN1578_TRACE);
    let whole = whole.as_deref().map(per_frame);
    let mut moved = Vec::new();
    for &(run, trace, frame, pinned) in MINTED_PACKETS {
        let (Some((logger, theirs)), Some(sets)) = (packet_stream_in(&dir, run), hook(trace))
        else {
            eprintln!("skipping {run}: no stream under {dir} or no {trace}");
            continue;
        };
        assert_eq!(logger, frame, "{run}'s values file names our frame");
        let values = std::fs::read_to_string(format!("{dir}/{run}/values.tsv")).unwrap();
        let instrument = crate::checksum::instrument_values(&values);
        let set = per_frame(&sets)[&(frame - 1)];
        let agree = CHECKSUM_LINES
            .iter()
            .zip(set.lines)
            .filter(|((name, ..), v)| instrument.iter().any(|(n, iv)| n == name && *iv == *v))
            .count();
        assert_eq!(
            agree,
            15,
            "{run}: the hook's set on {} is the instrument's",
            frame - 1
        );
        if let Some(whole) = &whole {
            assert_eq!(
                whole[&(frame - 1)].lines,
                set.lines,
                "run1578's set on {} is {run}'s",
                frame - 1
            );
        }
        let (Some((_, mut built, _)), Some(pieces)) = (
            built_start(),
            super::checksum_lines::packets::Pieces::read(),
        ) else {
            eprintln!("skipping: no install or lobby dump");
            return;
        };
        let mut stream = crate::input::Stream::pump(&minted(&script()).unwrap());
        super::minted::tick_to(&mut built, Some(&mut stream), frame);
        let outcome = compare(&our_lines(&built.sim, &pieces), &theirs);
        let mut seen: Vec<(&str, usize, usize, usize, String)> = Vec::new();
        for (name, o) in STATE_LINES.iter().zip(&outcome) {
            if let Some(f) = &o.first {
                eprintln!(
                    "MINTED PACKET {run} frame {frame} {name}: theirs {} ours {} parted {}; first {} {} {}:\n  ours   {}\n  theirs {}",
                    o.theirs, o.ours, o.parted, f.0, f.1, f.2, f.3, f.4
                );
                seen.push((name, o.theirs, o.ours, o.parted, first_text(name, f)));
            }
        }
        let pinned: Vec<(&str, usize, usize, usize, String)> = pinned
            .iter()
            .map(|&(l, t, o, p, f)| (l, t, o, p, f.to_string()))
            .collect();
        if seen != pinned {
            eprintln!("MINTED_PACKETS {run} moved: {seen:#?}");
            moved.push(run);
        }
    }
    assert!(moved.is_empty(), "MINTED_PACKETS moved on {moved:?}");
}

/// run1579's dump: the minted file's playback with an `@dumpall` on frame
/// 0, whose first record (3201) is printed before frame 0's tick — the
/// hook's frame `-1`, our built start.
const RUN1579: &str = "gamelog-run1579-greatlakes-playback-of-minted-packet-1.txt";

/// **The world on the start, by section** (item 4218): record 3201 of
/// [`RUN1579`]'s section 8, `danger[0..8]`, as `(theirs, ours with every
/// row cleared)`. Theirs is not zero: `Game::run@00584590:721` calls
/// `GameDaemon::calc_danger` once past `build_game`, before the first
/// frame, and the built start runs it (`Setup::calc_danger`). Before item
/// 4218 ours was the zeros' Adler (`0x70800001`, 28,800 zero bytes) and
/// section 8 alone parted.
const WORLD_START_DANGER: (u32, u32) = (2_995_557_468, 1_887_436_801);

/// The world line on the built start, every one of the fifteen sections
/// against record 3201 printed before frame 0's tick (no packet stands
/// there; the record carries each section's own sum). Red half: the same
/// start with every danger row cleared parts on section 8 alone, at the
/// zeros' sum — the state before `Game::run`'s call.
/// **The stats bits on the minted game's tables** (item 4333,
/// `docs/CHECKSUM.md` §25.4): the setup's tree raises neither (item 4340:
/// `Leader::init` lays it down in place; run1578's setup `check_all` reads
/// byte 3 `0a`, `0x8000000` alone, which `Wall::activate` raises for the
/// starting city), a
/// gained unit type raises `0x4000000` only on the Militia line
/// (`gain_tech@006dcb60`, `6dd988`: `is(0x42, 0)`), and any other gain
/// raises both (the tail's `0xc000000`). Then the emitter writes each.
#[test]
fn the_stats_bits_follow_the_militia_line() {
    let Some((_, mut built, _)) = super::minted::built_start() else {
        eprintln!("skipping: no install or lobby dump");
        return;
    };
    let sim = &mut built.sim;
    for who in 0..2u8 {
        let (f, _) = sim.leader_flag_words(who);
        // Theirs' `0a`: `0x8000000` alone. Item 4333's named divergence
        // (the tree raising `0x4000000` too, `0e`) went with item 4340: a
        // unit is born at its leader's speed, so nothing waits on the
        // first `calc_unit_stats`.
        assert_eq!(
            f.map(|f| f & 0xc00_0000),
            Some(0x800_0000),
            "leader {who}: the setup's `0x8000000` alone, theirs' `0a`"
        );
    }
    let tree = sim.tables.tech_tree.clone();
    let lacks = |sim: &sim::Sim, t: usize| !tree.has_tech(&sim.tables.setup, &sim.tech[0], t);
    let other = (0..tree.types.len())
        .find(|&t| tree.kind(t).is_unit() && !tree.is(t, 0x42, false) && lacks(sim, t))
        .expect("a unit type off the Militia line not yet held");
    sim.roster.unit_stats_dirty[0] = false;
    sim.cities.wall_stats_dirty[0] = false;
    sim.gain_tech(0, other);
    assert!(
        !sim.roster.unit_stats_dirty[0] && !sim.cities.wall_stats_dirty[0],
        "type {other:#x}: off the Militia line, neither bit"
    );
    assert_eq!(sim.leader_flag_words(0).0.map(|f| f & 0xc00_0000), Some(0));
    assert!(lacks(sim, 0x42), "the Militia is not held at the start");
    sim.gain_tech(0, 0x42);
    assert!(
        sim.roster.unit_stats_dirty[0],
        "the Militia raises 0x4000000"
    );
    assert!(!sim.cities.wall_stats_dirty[0], "and no wall stats");
    assert_eq!(
        sim.leader_flag_words(0).0.map(|f| f & 0xc00_0000),
        Some(0x400_0000)
    );
}

#[test]
fn the_world_is_the_original_s_on_the_start_by_section() {
    let Some(recs) = records(RUN1579) else {
        eprintln!("skipping: {RUN1579} not filed (set RON_GAMELOG_DIR)");
        return;
    };
    let Some((_, built, _)) = built_start() else {
        eprintln!("skipping: no install or lobby dump");
        return;
    };
    assert_eq!(built.sim.frame, 0, "the built start");
    let record = recs.iter().find(|r| r.number == 3201).expect("record 3201");
    let parts = record.world_parts();
    assert_eq!(parts.len(), 16, "sections 0 to 14 and the whole");
    let parted = |sim: &sim::Sim| -> Vec<(usize, u32, u32)> {
        let image = sim.world_image();
        parts
            .iter()
            .take(15)
            .enumerate()
            .filter_map(|(s, theirs)| {
                let ours = image
                    .iter()
                    .filter(|r| usize::from(r.section) == s)
                    .fold(1, |a, r| sim::checksum::adler32(a, &r.bytes()));
                (ours != *theirs).then_some((s, ours, *theirs))
            })
            .collect()
    };
    let whole = built.sim.checksum_world().adler();
    let seen = parted(&built.sim);
    eprintln!(
        "WORLD START whole ours {whole} theirs {}; sections parted {seen:?}",
        parts[15]
    );
    assert_eq!(
        parts[8], WORLD_START_DANGER.0,
        "record 3201's danger section"
    );
    assert!(seen.is_empty(), "the world parts on the start: {seen:?}");
    assert_eq!(whole, parts[15], "the world line on the start");
    let mut cleared = built.sim.clone();
    for who in 0..8u8 {
        if !cleared.world.danger_row(who).is_empty() {
            cleared.world.clear_danger(who);
        }
    }
    assert_eq!(
        parted(&cleared),
        vec![(8, WORLD_START_DANGER.1, WORLD_START_DANGER.0)],
        "the start without Game::run's calc_danger"
    );
}
