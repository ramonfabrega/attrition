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
                    if t.first.len() < 3 {
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

/// **The world line's census on run 1562's frame 91** (item 4131): per
/// section the stream's ranges and bytes and ours agreeing, and per field
/// the elements agreeing, parting and unheld.
#[test]
fn the_world_line_s_census_by_section() {
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
    for (f, n) in &c.extra {
        eprintln!("EXTRA {f}: {n}");
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
}
