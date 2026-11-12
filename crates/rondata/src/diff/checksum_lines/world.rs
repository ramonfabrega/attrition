//! **The world line, ours against the original's stream, section by
//! section and field by field** (item 4131, `docs/CHECKSUM.md` §21). The
//! stream's rows are named by their call site and their place in the walk
//! (`World::walk_data@006b5cf0`, part −1); ours are `Sim::world_image`'s
//! leaves. A range is matched by `(section, field, record)`, so a section
//! one side walks differently parts there alone. Each leaf element is
//! agreeing, parting, or unheld (ours holds no value, or ours has no such
//! range).

use super::{StreamRows, line_inputs};
use sim::checksum::world::{
    CELL_FIELDS, HEADER_FIELDS, OIL_ARRAYS, START_ARRAYS, TERRAIN_ARRAYS, WORLD_SECTIONS,
    WorldRange,
};
use std::collections::BTreeMap;

/// One stream row of the world line, named: `(section, field, record,
/// bytes)`.
type Named = (u8, &'static str, i16, Vec<u8>);

/// Sections 2 and 3's `a`-th array (from 1), range `k`.
fn arr(a: usize, k: usize) -> (u8, &'static str, i16) {
    if a <= 4 {
        (2, START_ARRAYS[a - 1][k], -1)
    } else {
        (3, OIL_ARRAYS[a - 5][k], -1)
    }
}

/// The stream's rows named by site and walk position.
fn name_rows(theirs: &StreamRows) -> Vec<Named> {
    let mut out = Vec::new();
    let (mut arrays, mut lists) = (0usize, 0usize);
    let (mut cell, mut block_cell, mut danger, mut loc) = (0i16, -1i16, 0i16, 0i16);
    for (_, site, b) in theirs {
        let (s, f, r) = match site.as_str() {
            "006b5d46" => (1, "xs..ys", -1),
            "0047c697" => {
                arrays += 1;
                arr(arrays, 0)
            }
            "0047c6f8" => arr(arrays, 1),
            "0047c706" => arr(arrays, 2),
            "0047c71e" => arr(arrays, 3),
            "0047c81d" => arr(arrays, 4),
            "006b5dd6" => (4, "size..seed", -1),
            "006b5e01" => {
                cell += 1;
                (5, "cell", cell - 1)
            }
            "006b5e30" => (6, "mask", -1),
            "006b5e43" => (6, "seen", -1),
            "006b5e56" => (6, "seen2", -1),
            "006b5e69" => (6, "seen3", -1),
            "006b5e85" => (7, "wcoord_seen", -1),
            "006b5eb5" => {
                danger += 1;
                (8, "danger", danger - 1)
            }
            "006b5f92" => {
                block_cell += 1;
                (9, "block.present", block_cell)
            }
            "006b5fb1" => (9, "block.head", block_cell),
            "006b5fc7" => (9, "block.body", block_cell),
            "004789c7" => (10, TERRAIN_ARRAYS[0][0], -1),
            "00478a28" => (10, TERRAIN_ARRAYS[0][1], -1),
            "00478a36" => (10, TERRAIN_ARRAYS[0][2], -1),
            "00478a4e" => (10, TERRAIN_ARRAYS[0][3], -1),
            "00478b54" => {
                loc += 1;
                (10, TERRAIN_ARRAYS[0][4], loc - 1)
            }
            "00473157" => {
                lists += 1;
                (10 + lists as u8, TERRAIN_ARRAYS[lists][0], -1)
            }
            "004731b8" => (10 + lists as u8, TERRAIN_ARRAYS[lists][1], -1),
            "004731c6" => (10 + lists as u8, TERRAIN_ARRAYS[lists][2], -1),
            "004731de" => (10 + lists as u8, TERRAIN_ARRAYS[lists][3], -1),
            "004732dd" => (10 + lists as u8, TERRAIN_ARRAYS[lists][4], -1),
            s => panic!("a world row at an unknown site {s}"),
        };
        out.push((s, f, r, b.clone()));
    }
    out
}

/// A range's leaves as `(name, width)`, repeated to fill its bytes: the
/// layout a stream range is decoded by when ours has no such range.
fn layout(field: &str) -> Vec<(&'static str, usize)> {
    match field {
        "xs..ys" => vec![("xs", 4), ("ys", 4)],
        "size..seed" => HEADER_FIELDS.iter().map(|&n| (n, 4)).collect(),
        "cell" => CELL_FIELDS.to_vec(),
        "mask" => vec![("mask", 2)],
        "seen" => vec![("seen", 1)],
        "seen2" => vec![("seen2", 1)],
        "seen3" => vec![("seen3", 1)],
        "wcoord_seen" => vec![("wcoord_seen", 1)],
        "danger" => vec![("danger", 4)],
        "block.present" => vec![("present", 4)],
        "block.head" => vec![("bits", 4), ("size", 4)],
        "block.body" => vec![("bits", 1)],
        "halfland_locs" => vec![("x", 4), ("y", 4)],
        f if f.ends_with(".num") => vec![("num", 4)],
        f if f.ends_with(".size") => vec![("size", 4)],
        f if f.ends_with(".increment") => vec![("increment", 2)],
        f if f.ends_with(".flags") => vec![("flags", 1)],
        _ => vec![("element", 4)],
    }
}

/// One field's tally: elements and bytes agreeing, parting and unheld, and
/// the first parting elements `(record, element, ours, theirs)`.
#[derive(Default, Debug)]
struct Tally {
    agree: (usize, usize),
    part: (usize, usize),
    unheld: (usize, usize),
    first: Vec<(i16, usize, i64, i64)>,
}

/// The census: per section `(ranges, bytes, ranges agreeing, bytes
/// agreeing)` of the stream, and per field (`section field.leaf`) its tally;
/// ours' ranges the stream has no row for, by field.
struct Census {
    sections: [(usize, usize, usize, usize); WORLD_SECTIONS],
    fields: BTreeMap<String, Tally>,
    extra: BTreeMap<String, usize>,
}

fn le(b: &[u8]) -> i64 {
    let mut v = [0u8; 8];
    v[..b.len()].copy_from_slice(b);
    if b.len() == 4 {
        i64::from(i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    } else {
        i64::from_le_bytes(v)
    }
}

/// How many parting elements a field's tally lists: three, or
/// `WORLD_FIRST` when a probe asks for more.
fn first_cap() -> usize {
    std::env::var("WORLD_FIRST")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3)
}

fn census(ours: &[WorldRange], theirs: &[Named]) -> Census {
    let key = |s: u8, f: &str, r: i16| format!("{s:02} {f} {r}");
    let by_key: BTreeMap<String, &WorldRange> = ours
        .iter()
        .map(|r| (key(r.section, r.field, r.record), r))
        .collect();
    let mut seen = std::collections::BTreeSet::new();
    let mut c = Census {
        sections: [(0, 0, 0, 0); WORLD_SECTIONS],
        fields: BTreeMap::new(),
        extra: BTreeMap::new(),
    };
    for (s, f, r, b) in theirs {
        let k = key(*s, f, *r);
        seen.insert(k.clone());
        let sec = &mut c.sections[*s as usize];
        sec.0 += 1;
        sec.1 += b.len();
        let o = by_key.get(&k);
        if let Some(o) = o
            && o.held()
            && o.bytes() == *b
        {
            sec.2 += 1;
            sec.3 += b.len();
        }
        // Each leaf element, ours or the layout's.
        let leaves: Vec<(&str, usize, Option<i64>)> = match o {
            Some(o) if o.bytes().len() == b.len() => o
                .leaves
                .iter()
                .flat_map(|l| {
                    (0..l.count)
                        .map(move |k| (l.name.as_str(), l.width, l.values.as_ref().map(|v| v[k])))
                })
                .collect(),
            _ => {
                let lay = layout(f);
                let per: usize = lay.iter().map(|l| l.1).sum();
                (0..b.len() / per)
                    .flat_map(|_| lay.iter().map(|&(n, w)| (n, w, None)))
                    .collect()
            }
        };
        let mut at = 0;
        for (e, (name, w, v)) in leaves.into_iter().enumerate() {
            let tb = &b[at..at + w];
            at += w;
            let t = c.fields.entry(format!("{s:02} {f}.{name}")).or_default();
            match v {
                None => {
                    t.unheld.0 += 1;
                    t.unheld.1 += w;
                }
                Some(v) if v.to_le_bytes()[..w] == *tb => {
                    t.agree.0 += 1;
                    t.agree.1 += w;
                }
                Some(v) => {
                    t.part.0 += 1;
                    t.part.1 += w;
                    if t.first.len() < first_cap() {
                        t.first.push((*r, e, v, le(tb)));
                    }
                }
            }
        }
    }
    for o in ours {
        if !seen.contains(&key(o.section, o.field, o.record)) {
            *c.extra
                .entry(format!("{:02} {}", o.section, o.field))
                .or_default() += 1;
        }
    }
    c
}

/// **The world line by section on run 1562's frame 91** (item 4131): the
/// stream's ranges and bytes, and ours agreeing, `(ranges, bytes, ranges
/// agreeing, bytes agreeing)` for sections 0 to 14. A section whose ranges
/// all agree is whole, and its Adler is the record's printed line.
pub(super) const WORLD: [(usize, usize, usize, usize); WORLD_SECTIONS] = [
    (0, 0, 0, 0),
    (1, 8, 1, 8),
    (20, 124, 20, 124),
    (2, 8, 2, 8),
    (1, 120, 1, 120),
    (3600, 75600, 3600, 75600),
    (4, 158_400, 4, 158_400),
    (1, 3600, 1, 3600),
    (8, 28800, 8, 28800),
    (3650, 17000, 3650, 17000),
    (696, 5547, 696, 5547),
    (5, 2779, 5, 2779),
    (5, 2779, 5, 2779),
    (1, 4, 1, 4),
    (0, 0, 0, 0),
];

/// **The fields the world line is held out on**, `section range.leaf` with
/// the count of its elements not agreeing (parting or unheld) on frame 91
/// (`docs/CHECKSUM.md` §21): none since item 4150. The cells' `land_sub`
/// and the masks' bit `0x4` are carried since item 4136
/// (`Terrain::object_placed`, `Wall::mark_behind_tiles`),
/// `World::analyze_map`'s ten setup tallies since item 4147
/// (`docs/MAPGEN.md` §36), the fog's two cell words `was_seen` and
/// `wcoord_seen` since item 4148 (`docs/VISION.md` §12), and the collision
/// blocks' presence, heads and bodies since item 4150 (`docs/COLLISION.md`
/// §26: a set allocates, the daemon's sweep frees an empty block).
pub(super) const WORLD_HELD_OUT: &[(&str, usize)] = &[];

/// The fields of a census not agreeing, with their counts.
fn held(c: &Census) -> Vec<(String, usize)> {
    c.fields
        .iter()
        .filter(|(_, t)| t.part.0 + t.unheld.0 > 0)
        .map(|(f, t)| (f.clone(), t.part.0 + t.unheld.0))
        .collect()
}

/// Record 3201's sixteen `World` lines: sections 0 to 14, then the whole.
fn printed_world_parts() -> Vec<u32> {
    printed_world_parts_of(crate::checksum::CHECKSUM_PACKET.2)
}

/// Record `number`'s sixteen `World` lines.
fn printed_world_parts_of(number: u64) -> Vec<u32> {
    let path = crate::testenv::dump(super::RUN1562).expect("the kept dump");
    let file = std::fs::File::open(path).expect("a kept dump opens");
    let recs = crate::checksum::records(std::io::BufReader::new(file)).expect("a kept dump reads");
    recs.iter()
        .find(|r| r.number == number)
        .unwrap_or_else(|| panic!("record {number}"))
        .world_parts()
}

/// One section's Adler over our image.
fn section_adler(ours: &[WorldRange], s: u8) -> u32 {
    ours.iter()
        .filter(|r| r.section == s)
        .fold(1, |a, r| sim::checksum::adler32(a, &r.bytes()))
}

/// **The world line, ours against the original's, section by section and
/// field by field** (`docs/CHECKSUM.md` §21, item 4131): our
/// `Sim::world_image` against the instrument's stream, every range matched
/// by section, field and record and every leaf element tallied; the
/// agreeing ranges and bytes pinned by section in [`WORLD`]; each field
/// left out named in [`WORLD_HELD_OUT`]; a whole section's Adler the
/// record's printed line. And it can part: one bent element of each kind of
/// field item 4131 carried returns that field to the list.
#[test]
fn the_world_line_is_the_original_s_by_section() {
    let Some((theirs, want, sim)) = line_inputs("world", "World") else {
        return;
    };
    let named = name_rows(&theirs);
    let ours = sim.world_image();
    let c = census(&ours, &named);
    for (s, v) in c.sections.iter().enumerate() {
        eprintln!(
            "SECTION {s}: theirs {} ranges {} bytes; agreeing {} ranges {} bytes",
            v.0, v.1, v.2, v.3
        );
    }
    for (f, t) in &c.fields {
        eprintln!(
            "FIELD {f}: agree {:?} part {:?} unheld {:?} first {:?}",
            t.agree, t.part, t.unheld, t.first
        );
    }
    let line = sim.checksum_world();
    eprintln!(
        "world: ours {} ranges {} bytes adler {}; theirs {} ranges {} bytes; printed {want}",
        line.ranges.len(),
        line.bytes(),
        line.adler(),
        theirs.len(),
        theirs.iter().map(|t| t.2.len()).sum::<usize>()
    );
    assert_eq!(c.sections, WORLD, "the world line by section");
    assert!(
        c.extra.is_empty(),
        "ours walks no range the stream lacks: {:?}",
        c.extra
    );
    let h = held(&c);
    let h: Vec<(&str, usize)> = h.iter().map(|(f, n)| (f.as_str(), *n)).collect();
    assert_eq!(
        h, WORLD_HELD_OUT,
        "the fields the world line is held out on"
    );
    // The record prints each section's own sum: a whole section is its line.
    let parts = printed_world_parts();
    assert_eq!(parts.len(), 16, "sections 0 to 14 and the whole");
    assert_eq!(parts[15], want);
    let mut whole = Vec::new();
    for (s, v) in WORLD.iter().enumerate() {
        let a = section_adler(&ours, s as u8);
        if v.0 == v.2 {
            assert_eq!(
                a, parts[s],
                "section {s} is whole: its Adler is the printed line"
            );
            whole.push(s);
        } else {
            assert_ne!(a, parts[s], "a held-out section {s} cannot sum to its line");
        }
    }
    eprintln!("world: sections whole {whole:?}");
    assert_eq!(whole, (0..WORLD_SECTIONS).collect::<Vec<_>>());
    // Whole since item 4150: every section, and so the line.
    assert!(h.is_empty(), "no field holds the line out");
    assert_eq!(line.adler(), want, "the world line's Adler");

    // **It can part.** One element of each kind of field items 4131 and
    // 4136 carried, bent, returns that field to the list with a count of
    // one.
    let bends: &[(u8, &str, i16, &str)] = &[
        (2, "start_x", -1, "02 start_x.start_x"),
        (2, "start_city_y.size", -1, "02 start_city_y.size.size"),
        (3, "oil_y.num", -1, "03 oil_y.num.num"),
        (4, "size..seed", -1, "04 size..seed.map"),
        (4, "size..seed", -1, "04 size..seed.seed"),
        // Item 4147: one of `analyze_map`'s tallies, at its leaf.
        (4, "size..seed", -1, "04 size..seed.goodies"),
        (5, "cell", 1373, "05 cell.light"),
        // Item 4136: a cell under seat 1's farms, `object_placed`'s, and
        // a tile's mask (the behind bit's own bend is below).
        (5, "cell", 1373, "05 cell.land_sub"),
        // Item 4148: a cell seat 1 has seen, `(52, 16)`, its `was_seen` 2.
        (5, "cell", 1012, "05 cell.was_seen"),
        (7, "wcoord_seen", -1, "07 wcoord_seen.wcoord_seen"),
        (6, "mask", -1, "06 mask.mask"),
        (8, "danger", 5, "08 danger.danger"),
        (10, "halfland_locs", 7, "10 halfland_locs.y"),
        (11, "halfland_types", -1, "11 halfland_types.halfland_types"),
        (
            12,
            "halfland_subtypes.size",
            -1,
            "12 halfland_subtypes.size.size",
        ),
        (13, "nuke_hits.num", -1, "13 nuke_hits.num.num"),
        // Item 4150: an empty cell's presence, a block's head, and one bit
        // of a body (cell 1313, `(53, 21)`, nine bits under a unit).
        (9, "block.present", 0, "09 block.present.present"),
        (9, "block.head", 1313, "09 block.head.size"),
        (9, "block.body", 1313, "09 block.body.bits"),
    ];
    for &(s, field, record, key) in bends {
        let leaf = key.rsplit('.').next().expect("a leaf");
        let mut bent = ours.clone();
        let r = bent
            .iter_mut()
            .find(|r| r.section == s && r.field == field && r.record == record)
            .unwrap_or_else(|| panic!("a range to bend for {key}"));
        let l = r
            .leaves
            .iter_mut()
            .find(|l| l.name == leaf)
            .unwrap_or_else(|| panic!("a leaf to bend for {key}"));
        l.values.as_mut().expect("a held leaf")[0] ^= 1;
        let mut want_h: Vec<(String, usize)> = WORLD_HELD_OUT
            .iter()
            .map(|(f, n)| (f.to_string(), *n))
            .collect();
        want_h.push((key.to_string(), 1));
        want_h.sort();
        assert_eq!(held(&census(&bent, &named)), want_h, "a bent {key}");
    }
    eprintln!("world: {} kinds of field bent", bends.len());

    // **The setup's tallies, bent at their input** (item 4147): the state
    // `World::analyze_map` counts, perturbed on a copy and the function
    // run again, returns the tally it feeds and nothing else.
    let standard = crate::testenv::install()
        .and_then(|i| crate::mapgen::standard_map_size(i.root()))
        .expect("rules.xml's Standard map size");
    let retally = |bend: &dyn Fn(&mut sim::Sim), keys: &[&str]| {
        let mut s = sim.clone();
        assert_eq!(
            s.map_analysis(standard).tallies,
            sim.world.record().tallies.expect("the setup's tallies"),
            "frame 91's state recounts to the setup's tallies"
        );
        bend(&mut s);
        s.analyze_map(standard);
        let mut want_h: Vec<(String, usize)> = WORLD_HELD_OUT
            .iter()
            .map(|(f, n)| (f.to_string(), *n))
            .collect();
        want_h.extend(keys.iter().map(|k| (k.to_string(), 1)));
        want_h.sort();
        assert_eq!(
            held(&census(&s.world_image(), &named)),
            want_h,
            "a bent {keys:?}"
        );
    };
    // A land cell that is no forest becomes one: `forest_size` one more,
    // beside the cell's own `flags`.
    retally(
        &|s| {
            let c = (0..s.world.width() * s.world.height())
                .map(|i| sim::world::Cell::new(i % s.world.width(), i / s.world.width()))
                .find(|&c| {
                    let d = s.world.cell_data(c);
                    d.land == 0 && d.flags & sim::world::cell::FOREST == 0
                })
                .expect("a bare land cell");
            let mut d = s.world.cell_data(c);
            d.flags |= sim::world::cell::FOREST;
            s.world.set_cell_data(c, d);
        },
        &["04 size..seed.forest_size", "05 cell.flags"],
    );
    // No leader is seated: the land region (3,053 cells, past the 44 of
    // `scale_by_area(60)`) holds no leader's city, so it is no player
    // region and becomes a resource region.
    retally(
        &|s| s.territory.players.clear(),
        &["04 size..seed.player_reg", "04 size..seed.resource_reg"],
    );
    // The last placed mountain (564 tiles, the top rung's 10) is gone:
    // `total_metal` 79 -> 69, and nothing the image walks moves with it.
    retally(
        &|s| {
            s.terrain.mountains.pop();
        },
        &["04 size..seed.total_metal"],
    );

    // **The sweep's place** (item 4150): one more visit of
    // `process_coll_blocks` on a copy walks cells 1,365 to 1,379 a frame
    // early and frees the empty blocks of 1,377 and 1,378, so their
    // presence parts and their heads and bodies go unheld.
    let mut early = sim.clone();
    early.movement.coll.process_coll_blocks();
    assert_eq!(
        held(&census(&early.world_image(), &named)),
        vec![
            ("09 block.body.bits".to_string(), 192),
            ("09 block.head.bits".to_string(), 2),
            ("09 block.head.size".to_string(), 2),
            ("09 block.present.present".to_string(), 2),
        ],
        "the sweep a frame early"
    );
    // And a block the sweep never frees: a bit set and cleared in cell
    // 0, `(0, 0)`, leaves an empty block the original does not hold.
    let mut stale = sim.clone();
    stale.movement.coll.set(0, 0, true);
    stale.movement.coll.set(0, 0, false);
    let c = census(&stale.world_image(), &named);
    assert_eq!(
        held(&c),
        vec![("09 block.present.present".to_string(), 1)],
        "a stale block in cell 0"
    );
    assert_eq!(
        c.extra.keys().collect::<Vec<_>>(),
        ["09 block.body", "09 block.head"],
        "its head and body are ranges the stream lacks"
    );

    // **The behind bit itself** (item 4136): tile 19175, `(215, 79)`, which
    // seat 1's Small City hides, loses its `0x4` and the mask parts there.
    let mut bent = ours.clone();
    let masks = bent
        .iter_mut()
        .find(|r| r.section == 6 && r.field == "mask")
        .and_then(|r| r.leaves.iter_mut().find(|l| l.name == "mask"))
        .and_then(|l| l.values.as_mut())
        .expect("the masks' leaf");
    assert_eq!(masks[19_175] & 4, 4, "tile 19175 is behind the Small City");
    masks[19_175] &= !4;
    let mut want_h: Vec<(String, usize)> = WORLD_HELD_OUT
        .iter()
        .map(|(f, n)| (f.to_string(), *n))
        .collect();
    want_h.push(("06 mask.mask".to_string(), 1));
    want_h.sort();
    assert_eq!(
        held(&census(&bent, &named)),
        want_h,
        "tile 19175 not behind"
    );
}

/// **The collision blocks on the next frame** (item 4150,
/// `docs/COLLISION.md` §26): record 3202, frame 92, differs from 3201 in
/// the world's section 9 alone, by the units' moves and the daemon's sweep
/// of cells 1,365 to 1,379, which frees the empty blocks of 1,377 and
/// 1,378. Our walk ticked one frame on prints that section's line, and the
/// fourteen others stand as on frame 91.
#[test]
fn the_collision_blocks_are_the_original_s_on_the_next_frame() {
    let Some((_, _, at91)) = line_inputs("world", "World") else {
        return;
    };
    let Some((_, mut built, _)) = super::built_start() else {
        return;
    };
    let Some(script) = super::minted(&super::script()) else {
        return;
    };
    let mut stream = crate::input::Stream::pump(&script);
    let next = crate::checksum::CHECKSUM_PACKET.1 + 1;
    super::tick_to(&mut built, Some(&mut stream), next);
    let sim = built.sim.clone();
    assert_eq!(sim.frame, next, "our walk stands on the next frame");
    let parts = printed_world_parts_of(crate::checksum::CHECKSUM_PACKET.2 + 1);
    let was = printed_world_parts();
    assert_eq!(parts.len(), 16, "sections 0 to 14 and the whole");
    let ours = sim.world_image();
    let then = at91.world_image();
    let blocks = |img: &[WorldRange]| -> Vec<i16> {
        img.iter()
            .filter(|r| r.section == 9 && r.field == "block.head")
            .map(|r| r.record)
            .collect()
    };
    eprintln!(
        "next frame: blocks on 91 {:?}; on 92 {:?}",
        blocks(&then),
        blocks(&ours)
    );
    for s in 0..WORLD_SECTIONS {
        let a = section_adler(&ours, s as u8);
        eprintln!(
            "next frame: section {s} ours {a} printed {} (frame 91's {})",
            parts[s], was[s]
        );
        assert_eq!(a, parts[s], "section {s} on the next frame");
    }
    assert_ne!(parts[9], was[9], "section 9 moved between the frames");
    assert_eq!(
        sim.checksum_world().adler(),
        parts[15],
        "the world line on the next frame"
    );
}
