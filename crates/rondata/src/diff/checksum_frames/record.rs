//! **Step R: the open lines' ranges, on every frame** (item 4248,
//! `docs/CHECKSUM.md` §25). run1597 played the minted file to 24,000 with
//! the checksum hook and `tools/trace/ranges.h`: per due set, every range
//! the original's fold took on lines 0 to 7 (units to leaders), with its
//! bytes. A frame of it is a packet's stream on those lines with no packet
//! taken, so a parity item's first act is a diff, not a capture.

use super::*;
use crate::trace::ranges::{Range, Record as Ranges, adler};

/// The record, outside git: `RON_RANGES_RECORD`, else run1597's take where
/// the capture left it.
fn record() -> Option<Ranges> {
    let path = std::env::var("RON_RANGES_RECORD").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        format!("{home}/ron-golden/run1597/map-14/rontrace.ranges")
    });
    match Ranges::open(std::path::Path::new(&path)) {
        Ok(r) => Some(r),
        Err(e) => {
            eprintln!("skipping: no step R record at {path} ({e})");
            None
        }
    }
}

/// The record's set for our frame `frame` — the first of trace frame
/// `frame - 1` (§24.1) — as a packet's stream rows, by line name: `(the
/// call site, the call site, bytes)`; the walker is named by its site,
/// since the export's names never enter the crate.
fn stream_at(
    rec: &mut Ranges,
    frame: i64,
) -> Option<std::collections::BTreeMap<String, super::super::checksum_lines::StreamRows>> {
    let set = rec.first_of(frame - 1)?.clone();
    assert_eq!(set.flags, 0, "the set on {frame} fit the hook's buffer");
    let mut out: std::collections::BTreeMap<String, super::super::checksum_lines::StreamRows> =
        Default::default();
    for r in rec.ranges(&set).expect("a set reads") {
        let name = CHECKSUM_LINES[r.line as usize].0.to_string();
        let site = format!("{:08x}", r.site);
        out.entry(name)
            .or_default()
            .push((site.clone(), site, r.bytes));
    }
    Some(out)
}

/// **The take**: `(sets, the first trace frame, the last, frames with a
/// second set, sets that did not fit)` — run1578's shape, every set's
/// sixteen words run1578's.
const RANGES_TAKE: (usize, i64, i64, usize, usize) = (24004, -1, 23999, 3, 0);

/// One set in this many, and every set on a packet's frame, has its kept
/// lines re-summed against its own words.
const RESUM_EVERY: usize = 97;

/// **The record is the hook's take, and its ranges are its words**: every
/// set's frame, place and sixteen words are run1578's ([`RANGES_TAKE`]);
/// on one set in [`RESUM_EVERY`] each kept line's ranges sum, by Adler-32,
/// to the set's own word for that line. A range the hook dropped, or put
/// on the wrong line, parts there.
#[test]
fn the_ranges_record_is_the_hook_s_take() {
    let Some(mut rec) = record() else { return };
    let Some(theirs) = hook(RUN1578_TRACE) else {
        eprintln!("skipping: no {RUN1578_TRACE}");
        return;
    };
    assert!(!rec.truncated, "the record ends on a set");
    let sets = rec.sets.clone();
    let seconds = sets.windows(2).filter(|w| w[0].frame == w[1].frame).count();
    let over = sets.iter().filter(|s| s.flags != 0).count();
    let seen = (
        sets.len(),
        sets.first().map_or(0, |s| s.frame),
        sets.last().map_or(0, |s| s.frame),
        seconds,
        over,
    );
    assert_eq!(seen, RANGES_TAKE, "RANGES_TAKE");
    assert_eq!(sets.len(), theirs.len(), "one set for each of run1578's");
    for (a, b) in sets.iter().zip(&theirs) {
        assert_eq!(
            (a.frame, a.ordinal, a.lines, a.total),
            (b.frame, b.ordinal, b.lines, b.total),
            "the record's set is run1578's"
        );
    }
    let mut resummed = 0;
    for s in sets.iter().step_by(RESUM_EVERY) {
        let rs: Vec<Range> = rec.ranges(s).expect("a set reads");
        for line in 0..8u8 {
            assert!(rec.keeps(line.into()), "the record keeps line {line}");
            assert_eq!(
                adler(&rs, line),
                s.lines[line as usize],
                "frame {} line {line}: the ranges sum to the word",
                s.frame
            );
        }
        resummed += 1;
    }
    assert!(resummed > 200, "{resummed} sets re-summed");
}

/// **A frame of the record is the packet's stream**: on run1562's packet
/// (frame 91) and each of [`MINTED_PACKETS`]' (runs 1579 to 1593), every
/// range of lines 0 to 7 on the record's frame is the instrument's on the
/// packet, site and bytes, in order — sixteen packets, no capture.
#[test]
fn the_ranges_record_is_each_packet_s_stream() {
    use super::super::checksum_lines::packets::packet_stream_in;
    let Some(mut rec) = record() else { return };
    let home = std::env::var("HOME").unwrap_or_default();
    let mut packets: Vec<(String, String)> = vec![(
        format!("{home}/ron-data/lab-experiments/2026-11-10-item-4060"),
        "run1562".into(),
    )];
    packets.extend(
        MINTED_PACKETS
            .iter()
            .map(|p| (minted_packets_dir(), p.0.to_string())),
    );
    let mut held = 0;
    for (dir, run) in &packets {
        let theirs = if run == "run1562" {
            let text = std::fs::read_to_string(format!("{dir}/{run}/stream-3201.tsv")).ok();
            text.map(|t| {
                (
                    91,
                    super::super::checksum_lines::stream_lines_of(&t, |l| {
                        STATE_LINES_KEPT.contains(&l)
                    }),
                )
            })
        } else {
            packet_stream_in(dir, run)
        };
        let Some((frame, theirs)) = theirs else {
            eprintln!("skipping {run}: no stream under {dir}");
            continue;
        };
        let ours = stream_at(&mut rec, frame).expect("the record has every frame");
        for line in STATE_LINES_KEPT {
            let empty = Vec::new();
            let (a, b) = (
                ours.get(line).unwrap_or(&empty),
                theirs.get(line).unwrap_or(&empty),
            );
            assert_eq!(a.len(), b.len(), "{run} {line}: ranges");
            for (i, (x, y)) in a.iter().zip(b).enumerate() {
                assert_eq!((&x.1, &x.2), (&y.1, &y.2), "{run} {line} range {i}");
            }
        }
        held += 1;
    }
    eprintln!("RANGES: the record is {held} packets' streams on lines 0 to 7");
    assert!(
        held == 0 || held == packets.len(),
        "every packet on disk is held"
    );
}

/// One line's parting as the test sees it: [`PacketRow`] with the text owned.
type SeenRow = (&'static str, usize, usize, usize, String);

/// Lines 0 to 7, the record's mask.
const STATE_LINES_KEPT: [&str; 8] = [
    "units", "builds", "walls", "ammo", "deaths", "groups", "guys", "leaders",
];

/// **Where each open line first parts, off the record** (§25): per frame,
/// as [`MINTED_PACKETS`] pins a packet's — `(our frame, per kept line that
/// parts (line, the record's ranges, ours, ranges parted, the first
/// parting))`. The frames are [`CHECKSUM_FRAMES`]' first partings of the
/// open lines; a walker the record names by its call site. The leaders
/// line's first parting moved from `score` at `+0x10` to `+0x1cc` on all
/// ten frames when item 4159's sixteenth residue put `score` on the line
/// (ours equal to theirs there, 0x124 on 7195 to 0x16b on 14312), and off
/// `+0x1cc` (leader 0's `hire_who`, theirs −1) when item 4298 emitted it:
/// one range fewer on every frame, the first parting now `flags` or
/// `resources_controlled` (`+0x850`) or `reg_unpack_merch[1]` (`+0x5d8`).
const RECORD_PARTINGS: &[(i64, &[PacketRow])] = &[
    // Item 4317: the leaders line's first three parted frames on run1578
    // (`CHECKSUM_FRAMES`' 0, 100 and 159), every other line whole there.
    // `leader_flags` byte 2's `0x80` is bit `0x800000`, the score's units
    // dirty bit (`Unit::init`, `Unit::close`; `compute_score` clears it),
    // which the crate keeps (`Score::dirty_units`) and the emitter does not
    // write; on 0 byte 3's `0x08` too (`0x8000000`), and fifteen more
    // ranges, the setup's (item 4317's journal).
    (
        0,
        &[(
            "leaders",
            267,
            267,
            16,
            // Item 4320: byte 2 agrees (`0x800000` emitted); before it
            // `+0x2 of 8|8: 00020000|800a0000`. `0x8000000` stands.
            "0 0 flags: +0x3 of 8|8: 02000000|0a000000",
        )],
    ),
    (
        100,
        &[
            // Item 4320: leaders whole (`0x800000` emitted); before it, 1 of
            // 267 parted, `128 1 flags: +0x2 of 8|8: 00000000|80000000`.
        ],
    ),
    (
        101,
        // Item 4320: the next parted frame once `0x800000` is emitted.
        // Byte 2's `0x08` is bit `0x80000`, theirs alone on leader 1:
        // `Unit::think`'s carrying store, cleared at the ends of
        // `Leader::process` and `Leaders::process_all`; not held.
        &[(
            "leaders",
            267,
            267,
            1,
            "128 1 flags: +0x2 of 8|8: 80000000|88000000",
        )],
    ),
    (
        159,
        &[
            // Item 4320: leaders whole (`0x800000` emitted); before it, 1 of
            // 267 parted, `0 0 flags: +0x2 of 8|8: 00000000|80000000`.
        ],
    ),
    (
        160,
        // Item 4320: the next parted frame once `0x800000` is emitted.
        // Byte 2's `0x08` is bit `0x80000`, theirs alone on leader 0:
        // `Unit::think`'s carrying store, cleared at the ends of
        // `Leader::process` and `Leaders::process_all`; not held.
        &[(
            "leaders",
            267,
            267,
            1,
            "0 0 flags: +0x2 of 8|8: 80000000|88000000",
        )],
    ),
    (
        7195,
        // Item 4251: units whole; before it, 3 of 1697 ranges from 832,
        // records 27..29 `unit_masks` 0x40000|0x40004 (bit 0x4,
        // `in_danger`, left out of the emitter; run1600's packet agrees).
        &[
            // Item 4298: 3 → 2 parted (`hire_who`'s −1s emitted); before it, first `1 0
            // who..ally_mask: +0x1cc of 26914|26914: 00000000|ffffffff`.
            // Item 4320: 2 → 1 (the score's units bit `0x800000` emitted);
            // before it, first `128 1 flags: +0x2 of 8|8: 00020000|80020000`.
            (
                "leaders",
                271,
                271,
                1,
                "129 1 who..ally_mask: +0x848 of 26914|26914: 00000000|03000000",
            ),
        ],
    ),
    (
        7273,
        // Item 4253: units and guys whole; before it, 1 of 1692 and 1 of
        // 560 ranges, 891 and 279, record 30 (`1/30`, a Scholar, TI 52,
        // seated in its University on 7272) guy `angle` and `last_angle`
        // 0x55555555|0 (`Unit::go_inside`'s `set_angle(0, _, 1)` snap).
        &[
            // Item 4298: 3 → 2 parted (`hire_who`'s −1s emitted); before it, first `1 0
            // who..ally_mask: +0x1cc of 26914|26914: 00000000|ffffffff`.
            // Item 4320: 2 → 1 (the score's units bit `0x800000` emitted);
            // before it, first `128 1 flags: +0x2 of 8|8: 00020000|80020000`.
            (
                "leaders",
                271,
                271,
                1,
                "129 1 who..ally_mask: +0x848 of 26914|26914: 00000000|03000000",
            ),
        ],
    ),
    (
        7383,
        &[
            // Item 4298: 2 → 1 parted (`hire_who`'s −1s emitted); before it, first `1 0
            // who..ally_mask: +0x1cc of 26914|26914: 00000000|ffffffff`.
            (
                "leaders",
                271,
                271,
                1,
                "129 1 who..ally_mask: +0x848 of 26914|26914: 00000000|03000000",
            ),
        ],
    ),
    (
        8206,
        // Item 4255: groups whole; before it, 1 of 544 ranges from 91,
        // record 67 `ox`/`oy` 30552/21144 ours, 0/0 theirs (here and on
        // 8281): `equals_group`'s `get_num` on an emptied last slot.
        &[
            // Item 4298: 2 → 1 parted (`hire_who`'s −1s emitted); before it, first `1 0
            // who..ally_mask: +0x1cc of 26914|26914: 00000000|ffffffff`.
            (
                "leaders",
                271,
                271,
                1,
                "129 1 who..ally_mask: +0x848 of 26914|26914: 00000000|03000000",
            ),
        ],
    ),
    (
        8281,
        // Item 4257 took units and builds here: the garrison chain's order
        // in the University 1/2018 (record 30 `inside_down` -1|42, the
        // building's 42|30), whole since; item 4255 took groups.
        &[
            // Item 4298: 3 → 2 parted (`hire_who`'s −1s emitted); before it, first `1 0
            // who..ally_mask: +0x1cc of 26914|26914: 00000000|ffffffff`.
            // Item 4320: 2 → 1 (the score's units bit `0x800000` emitted);
            // before it, first `128 1 flags: +0x2 of 8|8: 00020000|80020000`.
            (
                "leaders",
                271,
                271,
                1,
                "129 1 who..ally_mask: +0x848 of 26914|26914: 00000000|03000000",
            ),
        ],
    ),
    (
        9810,
        // Item 4255's groups row (record 64 `role` 0x150c00 ours, 0x150c03
        // theirs) went with item 4258 (`Group::add`'s OR of the Hoplites'
        // role, TI 132); its units and builds rows (the University's chain)
        // went with item 4257.
        &[
            // Item 4298: 2 → 1 parted (`hire_who`'s −1s emitted); before it, first `1 0
            // who..ally_mask: +0x1cc of 26914|26914: 00000000|ffffffff`.
            (
                "leaders",
                271,
                271,
                1,
                "129 1 who..ally_mask: +0x5d0 of 26914|26914: 04000000|03000000",
            ),
        ],
    ),
    (
        10541,
        // Item 4260 took units and guys here: the Caravan `1/14`'s crew
        // (guys 1 and 2) `last_angle` 682461312|968792448, read before guy
        // 0's turn; whole since.
        &[
            // Item 4298: 2 → 1 parted (`hire_who`'s −1s emitted); before it, first `1 0
            // who..ally_mask: +0x1cc of 26914|26914: 00000000|ffffffff`.
            (
                "leaders",
                271,
                271,
                1,
                "129 1 who..ally_mask: +0x5d0 of 26914|26914: 05000000|03000000",
            ),
        ],
    ),
    (
        11884,
        // Item 4263 took units and guys here: the Hoplites `1/56..1/58`
        // (TI 132) upgraded to Phalanx on a frame they walked, record 56's
        // `last_x`/`last_y` 38275|38297, 21337|21325 (`Unit::set_type`'s
        // seating writes `last` on the point); whole since.
        &[
            // Item 4298: 3 → 2 parted (`hire_who`'s −1s emitted); before it, first `1 0
            // who..ally_mask: +0x1cc of 26914|26914: 00000000|ffffffff`.
            // Item 4320: 2 → 1 (`gain_tech`'s `0x1000000` emitted); before
            // it, first `128 1 flags: +0x3 of 8|8: 02000000|03000000`.
            (
                "leaders",
                271,
                271,
                1,
                "129 1 who..ally_mask: +0x5d0 of 26914|26914: 06000000|05000000",
            ),
        ],
    ),
    (
        12537,
        // Item 4266 took units here: record 62's second order, a `GUARD`
        // of 1/61 (the Longbowmen 1/62..1/64, TI 177), whose
        // `GuardOrder::walk_data` ranges the emitter did not hand over —
        // range 1539 `00486573`'s flags byte `04`, read as our guy list's
        // `length` 1 (the booking's "4 figures"); whole since.
        &[
            // Item 4298: 2 → 1 parted (`hire_who`'s −1s emitted); before it, first `1 0
            // who..ally_mask: +0x1cc of 26914|26914: 00000000|ffffffff`.
            (
                "leaders",
                271,
                271,
                1,
                "129 1 who..ally_mask: +0x848 of 26914|26914: 00000000|06000000",
            ),
        ],
    ),
    (
        14312,
        // Item 4257's next builds word, record 2023's `wonder` -1|0, left
        // with item 4262: builds is whole on all 24,000. Its units row, the
        // same guard as 12537's, left with item 4266.
        &[
            // Item 4298: 3 → 2 parted (`hire_who`'s −1s emitted); before it, first `1 0
            // who..ally_mask: +0x1cc of 26914|26914: 00000000|ffffffff`.
            (
                "leaders",
                271,
                271,
                2,
                "128 1 flags: +0x3 of 8|8: 02000000|0e000000",
            ),
        ],
    ),
];

/// **The record names the partings**: our walk under the pump to each of
/// [`RECORD_PARTINGS`]' frames, lines 0 to 7 held range by range against
/// the record's set there.
#[test]
fn the_ranges_record_names_each_first_parting() {
    use super::super::checksum_lines::packets::{STATE_LINES, compare, our_lines};
    let Some(mut rec) = record() else { return };
    let (Some((_, mut built, _)), Some(pieces)) = (
        built_start(),
        super::super::checksum_lines::packets::Pieces::read(),
    ) else {
        eprintln!("skipping: no install or lobby dump");
        return;
    };
    let mut stream = crate::input::Stream::pump(&minted(&script()).unwrap());
    let mut seen_all = Vec::new();
    for &(frame, _) in RECORD_PARTINGS {
        super::super::minted::tick_to(&mut built, Some(&mut stream), frame);
        let theirs = stream_at(&mut rec, frame).expect("the record has the frame");
        let outcome = compare(&our_lines(&built.sim, &pieces), &theirs);
        let mut seen: Vec<SeenRow> = Vec::new();
        for (name, o) in STATE_LINES.iter().zip(&outcome).take(8) {
            if let Some(f) = &o.first {
                eprintln!(
                    "RECORD frame {frame} {name}: theirs {} ours {} parted {}; first {} {} {}:\n  ours   {}\n  theirs {}",
                    o.theirs, o.ours, o.parted, f.0, f.1, f.2, f.3, f.4
                );
                seen.push((name, o.theirs, o.ours, o.parted, first_text(name, f)));
            }
        }
        seen_all.push((frame, seen));
    }
    let pinned: Vec<(i64, Vec<SeenRow>)> = RECORD_PARTINGS
        .iter()
        .map(|(f, rows)| {
            (
                *f,
                rows.iter()
                    .map(|&(l, t, o, p, s)| (l, t, o, p, s.to_string()))
                    .collect(),
            )
        })
        .collect();
    if seen_all != pinned {
        eprintln!("RECORD_PARTINGS moved: {seen_all:#?}");
    }
    assert_eq!(seen_all, pinned, "RECORD_PARTINGS");
}
