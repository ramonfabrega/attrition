//! **The checksum's state lines, ours against the original's stream**
//! (item 4088, `docs/CHECKSUM.md` §11). Our side is `sim::checksum`'s byte
//! image of a line, emitted from the minted game's walk on the packet's
//! frame (the built start under the pump, [`super::minted`]). The other side
//! is the instrument's stream of that line on run 1562's packet
//! (`tools/recomp/checksum.py --stream`, kept outside git): every range
//! the original's walk handed its fold, with its bytes. The two are held
//! range by range — record, field, both sides' values — and the Adler is
//! read last, against the line the original printed in record
//! [`CHECKSUM_PACKET`]'s `.2`.

use super::minted::{built_start, minted, script, tick_to};
use crate::checksum::CHECKSUM_PACKET;
use sim::checksum::{COORD_MASK, Line};

/// The minted game's playback with the packet and the `@dumpall` record
/// (`docs/RUNS.md` run1562).
const RUN1562: &str = "gamelog-run1562-greatlakes-playback-of-minted-packet-91.txt";

/// The instrument's stream of run 1562's packet, if this machine has it:
/// the rows of `line`, as `(walker, call site, bytes)`.
fn stream(line: &str) -> Option<Vec<(String, String, Vec<u8>)>> {
    let dir = std::env::var("RON_CHECKSUM_DIR").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        format!("{home}/ron-data/lab-experiments/2026-11-10-item-4060")
    });
    let path = format!(
        "{dir}/{}/stream-{}.tsv",
        CHECKSUM_PACKET.0, CHECKSUM_PACKET.2
    );
    let text = std::fs::read_to_string(path).ok()?;
    let hex = |s: &str| -> Vec<u8> {
        (0..s.len() / 2)
            .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("the stream's hex"))
            .collect()
    };
    Some(
        text.lines()
            .skip(1)
            .map(|l| l.split('\t').collect::<Vec<_>>())
            .filter(|c| c.len() == 8 && c[0] == line)
            .map(|c| (c[2].to_string(), c[4].to_string(), hex(c[7])))
            .collect(),
    )
}

/// The printed value of `label` in record [`CHECKSUM_PACKET`]'s `.2` of
/// run 1562's archived log.
fn printed(label: &str) -> Option<u32> {
    let path = crate::testenv::dump(RUN1562)?;
    let file = std::fs::File::open(path).expect("a kept dump opens");
    let recs = crate::checksum::records(std::io::BufReader::new(file)).expect("a kept dump reads");
    recs.iter()
        .find(|r| r.number == CHECKSUM_PACKET.2)
        .and_then(|r| r.get(label))
}

/// Our walk of the minted game on the packet's frame, walked once for
/// every line's test.
fn ours_at_packet() -> Option<sim::Sim> {
    static WALK: std::sync::Mutex<Option<Option<sim::Sim>>> = std::sync::Mutex::new(None);
    let mut walk = WALK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    walk.get_or_insert_with(|| {
        let (_, mut built, _) = built_start()?;
        let mut stream = crate::input::Stream::pump(&minted(&script())?);
        tick_to(&mut built, Some(&mut stream), CHECKSUM_PACKET.1);
        Some(built.sim.clone())
    })
    .clone()
}

/// Skips a line's test, saying why, on a machine without the stream, the
/// archived record or the install: `Some((theirs, printed, ours))` else.
/// One line's rows of the instrument's stream: `(walker, call site, bytes)`.
type StreamRows = Vec<(String, String, Vec<u8>)>;

fn line_inputs(line: &str, label: &str) -> Option<(StreamRows, u32, sim::Sim)> {
    let Some(theirs) = stream(line) else {
        eprintln!(
            "skipping: no instrument stream for {} record {} (RON_CHECKSUM_DIR)",
            CHECKSUM_PACKET.0, CHECKSUM_PACKET.2
        );
        return None;
    };
    if crate::testenv::dump(RUN1562).is_none() {
        eprintln!("skipping: no {RUN1562} (set RON_GAMELOG_DIR)");
        return None;
    }
    let want = printed(label).expect("record 3201 prints the line");
    let Some(sim) = ours_at_packet() else {
        eprintln!("skipping: no install or lobby dump (RON_INSTALL, RON_GAMELOG_DIR)");
        return None;
    };
    assert_eq!(
        sim.frame, CHECKSUM_PACKET.1,
        "our walk stands on the packet's frame"
    );
    Some((theirs, want, sim))
}

/// Where an image parts from the stream, range by range: `(range index,
/// record, field, ours, theirs)`, the bytes in hex. A range one side lacks
/// parts against `-`.
fn parting(
    ours: &Line,
    theirs: &[(String, String, Vec<u8>)],
) -> Vec<(usize, i16, String, String, String)> {
    let hex = |b: &[u8]| b.iter().map(|x| format!("{x:02x}")).collect::<String>();
    let n = ours.ranges.len().max(theirs.len());
    (0..n)
        .filter_map(|i| {
            let o = ours.ranges.get(i);
            let t = theirs.get(i);
            if o.map(|r| &r.bytes) == t.map(|t| &t.2) {
                return None;
            }
            Some((
                i,
                o.map_or(-1, |r| r.record),
                o.map_or_else(
                    || t.map_or("-".into(), |t| t.0.clone()),
                    |r| r.field.to_string(),
                ),
                o.map_or("-".into(), |r| hex(&r.bytes)),
                t.map_or("-".into(), |t| hex(&t.2)),
            ))
        })
        .collect()
}

/// An item's 22 bytes, decoded: `ever_seen`, `flags`, `must_walk`, `who`,
/// `o`, `z`, `x`, `y` (the `Coord`s unmasked) and the type local.
fn item_fields(r: &[&[u8]]) -> [i64; 9] {
    let w = |b: &[u8], at: usize| {
        i64::from(i32::from_le_bytes(b[at..at + 4].try_into().unwrap()) ^ COORD_MASK)
    };
    let s = r[3];
    [
        i64::from(r[0][0]),
        i64::from(r[1][0]),
        i64::from(r[2][0]),
        i64::from(s[0]),
        i64::from(i16::from_le_bytes([s[1], s[2]])),
        w(s, 3),
        w(s, 7),
        w(s, 11),
        i64::from(i32::from_le_bytes(r[4].try_into().unwrap())),
    ]
}

/// The pinned outcome of the items line on run 1562's packet: `(records,
/// ranges, bytes, ranges parted)`.
const ITEMS: (usize, usize, usize, usize) = (22, 110, 484, 0);

/// **The items line, ours against the original's, range by range**
/// (`docs/CHECKSUM.md` §11). Our `Sim::checksum_items` on the minted
/// walk's frame 91 against the instrument's stream of the items line on
/// run 1562's packet (logger frame 91, record 3201): every range's bytes,
/// then each record's nine fields decoded both sides, then the Adler of
/// ours against the line the original printed. And it can part: one
/// perturbed byte of ours parts on exactly that range, and the Adler with
/// it; an empty image parts on every range.
#[test]
fn the_items_line_is_the_original_s_range_by_range() {
    let Some(theirs) = stream("items") else {
        eprintln!(
            "skipping: no instrument stream for {} record {} (RON_CHECKSUM_DIR)",
            CHECKSUM_PACKET.0, CHECKSUM_PACKET.2
        );
        return;
    };
    if crate::testenv::dump(RUN1562).is_none() {
        eprintln!("skipping: no {RUN1562} (set RON_GAMELOG_DIR)");
        return;
    }
    let want = printed("Items").expect("record 3201 prints an Items line");
    let Some(sim) = ours_at_packet() else {
        eprintln!("skipping: no install or lobby dump (RON_INSTALL, RON_GAMELOG_DIR)");
        return;
    };
    assert_eq!(
        sim.frame, CHECKSUM_PACKET.1,
        "our walk stands on the packet's frame"
    );
    let ours = sim.checksum_items();
    // Every `z` is read off the corner grid: none of the 22 may rest on a
    // corner that is not known to be the original's own single.
    let inexact = sim
        .gaia
        .goody_items
        .iter()
        .filter(|it| it.alive && !sim.world.data_z(it.pos.x, it.pos.y).1)
        .count();
    assert_eq!(inexact, 0, "an item's z read an inexact corner");
    // The value diff, per record and field.
    let names = [
        "ever_seen",
        "flags",
        "must_walk",
        "who",
        "o",
        "z",
        "x",
        "y",
        "type",
    ];
    for (k, (o, t)) in ours.ranges.chunks(5).zip(theirs.chunks(5)).enumerate() {
        let ob: Vec<&[u8]> = o.iter().map(|r| r.bytes.as_slice()).collect();
        let tb: Vec<&[u8]> = t.iter().map(|r| r.2.as_slice()).collect();
        if ob.len() < 5 || tb.len() < 5 {
            continue;
        }
        let (of, tf) = (item_fields(&ob), item_fields(&tb));
        let row: Vec<String> = names
            .iter()
            .zip(of.iter().zip(&tf))
            .map(|(n, (a, b))| {
                if a == b {
                    format!("{n} {a}")
                } else {
                    format!("{n} {a}|{b}")
                }
            })
            .collect();
        eprintln!("ITEM {k}: {}", row.join(", "));
    }
    let parted = parting(&ours, &theirs);
    for p in &parted {
        eprintln!("PARTS {p:?}");
    }
    eprintln!(
        "items: ours {} ranges {} bytes adler {}; theirs {} ranges {} bytes; printed {want}",
        ours.ranges.len(),
        ours.bytes(),
        ours.adler(),
        theirs.len(),
        theirs.iter().map(|t| t.2.len()).sum::<usize>(),
    );
    assert_eq!(
        (
            ours.ranges.len() / 5,
            theirs.len(),
            theirs.iter().map(|t| t.2.len()).sum::<usize>(),
            parted.len()
        ),
        ITEMS,
        "the items line: records, the stream's ranges and bytes, ranges parted"
    );
    assert_eq!(ours.bytes(), ITEMS.2);
    // The Adler, last: ours against the line the original printed.
    assert_eq!(ours.adler(), want, "the items line's Adler");

    // **It can part.** One byte of ours perturbed — the fourth box's `x`,
    // low byte — parts on that range alone, and the Adler with it.
    let mut bent = ours.clone();
    bent.ranges[3 * 5 + 3].bytes[7] ^= 1;
    let p = parting(&bent, &theirs);
    assert_eq!(
        p.iter()
            .map(|p| (p.0, p.1, p.2.as_str()))
            .collect::<Vec<_>>(),
        [(18, 3, "who..y_internal")],
        "one perturbed byte parts on its range"
    );
    assert_ne!(bent.adler(), want);
    // And an empty image parts on every range of theirs.
    assert_eq!(parting(&Line::default(), &theirs).len(), ITEMS.1);
}

/// One city's 108 bytes decoded by the type record's table
/// ([`sim::checksum::CITY_FIELDS`]): `who`, `race` and `founder` are
/// `char`s, `traded_with` `int`s read unsigned, every other byte a
/// `uchar` and every wider field signed.
fn city_fields(b: &[u8]) -> Vec<(&'static str, i64)> {
    sim::checksum::CITY_FIELDS
        .iter()
        .map(|&(name, at, width)| {
            let at = at - 6;
            let v = match width {
                1 if matches!(name, "who" | "race" | "founder") => i64::from(b[at] as i8),
                1 => i64::from(b[at]),
                2 => i64::from(i16::from_le_bytes([b[at], b[at + 1]])),
                _ if name.starts_with("traded_with") => {
                    i64::from(u32::from_le_bytes(b[at..at + 4].try_into().unwrap()))
                }
                _ => i64::from(i32::from_le_bytes(b[at..at + 4].try_into().unwrap())),
            };
            (name, v)
        })
        .collect()
}

/// The pinned outcome of the cities line on run 1562's packet: `(cities,
/// ranges, bytes, ranges parted)`.
const CITIES: (usize, usize, usize, usize) = (2, 6, 228, 0);

/// **The cities line, ours against the original's, range by range**
/// (`docs/CHECKSUM.md` §12). Our `Sim::checksum_cities` on the minted
/// walk's frame 91 against the instrument's stream of the cities line on
/// run 1562's packet: every range's bytes, then each city's `city_flags`
/// and its 52 fields decoded both sides, then the Adler against the
/// printed `Cities`. It can part: one perturbed byte of ours parts on its
/// range alone, and an empty image on every range.
#[test]
fn the_cities_line_is_the_original_s_range_by_range() {
    let Some((theirs, want, sim)) = line_inputs("cities", "Cities") else {
        return;
    };
    let ours = sim.checksum_cities();
    // The value diff, per city and field: a city is its flags, its 108
    // bytes and its links' header, so the 108-byte ranges are the records.
    let recs = |l: Vec<&Vec<u8>>| -> Vec<(u16, Vec<(&'static str, i64)>)> {
        let mut out = Vec::new();
        for w in l.windows(2) {
            if w[0].len() == 2 && w[1].len() == 108 {
                out.push((u16::from_le_bytes([w[0][0], w[0][1]]), city_fields(w[1])));
            }
        }
        out
    };
    let o = recs(ours.ranges.iter().map(|r| &r.bytes).collect());
    let t = recs(theirs.iter().map(|t| &t.2).collect());
    let mut fields_parted = 0;
    for (k, (a, b)) in o.iter().zip(&t).enumerate() {
        let mut row = vec![if a.0 == b.0 {
            format!("city_flags {:#x}", a.0)
        } else {
            fields_parted += 1;
            format!("city_flags {:#x}|{:#x}", a.0, b.0)
        }];
        for ((n, x), (_, y)) in a.1.iter().zip(&b.1) {
            if x == y {
                row.push(format!("{n} {x}"));
            } else {
                fields_parted += 1;
                row.push(format!("{n} {x}|{y}"));
            }
        }
        eprintln!("CITY {k}: {}", row.join(", "));
    }
    let parted = parting(&ours, &theirs);
    for p in &parted {
        eprintln!("PARTS {p:?}");
    }
    eprintln!(
        "cities: ours {} ranges {} bytes adler {}; theirs {} ranges {} bytes; printed {want}; fields parted {fields_parted}",
        ours.ranges.len(),
        ours.bytes(),
        ours.adler(),
        theirs.len(),
        theirs.iter().map(|t| t.2.len()).sum::<usize>(),
    );
    assert_eq!(
        (
            o.len(),
            theirs.len(),
            theirs.iter().map(|t| t.2.len()).sum::<usize>(),
            parted.len()
        ),
        CITIES,
        "the cities line: cities, the stream's ranges and bytes, ranges parted"
    );
    assert_eq!(o.len(), t.len(), "both sides walk the same cities");
    // The two flag words (`docs/CHECKSUM.md` §12): the human's city holds
    // a Market (`0x800`), the British does not.
    assert_eq!(
        t.iter().map(|c| c.0).collect::<Vec<_>>(),
        [0x4811, 0x4011],
        "the original's city_flags"
    );
    assert_eq!(fields_parted, 0, "a city's field parts");
    assert_eq!(ours.bytes(), CITIES.2);
    // The Adler, last: ours against the line the original printed.
    assert_eq!(ours.adler(), want, "the cities line's Adler");

    // **It can part.** The second city's `land` (+0x63), perturbed, parts
    // on its range alone, and the Adler with it.
    let mut bent = ours.clone();
    bent.ranges[4].bytes[0x63 - 6] ^= 1;
    let p = parting(&bent, &theirs);
    assert_eq!(
        p.iter().map(|p| (p.0, p.2.as_str())).collect::<Vec<_>>(),
        [(4, "city..ter")],
        "one perturbed byte parts on its range"
    );
    assert_ne!(bent.adler(), want);
    assert_eq!(
        parting(&sim::checksum::Line::default(), &theirs).len(),
        CITIES.1
    );
}

/// The pinned outcome of the goods line on run 1562's packet: `(goods,
/// ranges, bytes, ranges parted)`. **Held out on one byte** (`docs/CHECKSUM.md`
/// §13): good 15's `ever_seen`.
const GOODS: (usize, usize, usize, usize) = (36, 180, 792, 1);

/// **The goods line, ours against the original's, range by range, held
/// out on one byte** (`docs/CHECKSUM.md` §13): our `Sim::checksum_goods`
/// against the instrument's stream of the goods line, every range, then
/// each good's nine fields decoded both sides, then the Adler against the
/// printed `Goods`. 179 of 180 ranges agree; the one that parts is good
/// 15's `GoodData::ever_seen` — the SILK at (40320, 14208), 2 in the
/// original (leader 1's bit, set by setup's `compute_all_territory`
/// goods scan) and 0 in ours until our first in-play border pass sets it
/// on frame 387. The pin can fail both ways: a fix of the setup pass turns
/// it red, and so does any other byte that parts.
#[test]
fn the_goods_line_is_held_out_on_good_15_s_ever_seen() {
    let Some((theirs, want, sim)) = line_inputs("goods", "Goods") else {
        return;
    };
    let ours = sim.checksum_goods();
    let names = [
        "ever_seen",
        "flags",
        "must_walk",
        "who",
        "o",
        "z",
        "x",
        "y",
        "type",
    ];
    let mut seen = Vec::new();
    for (k, (o, t)) in ours.ranges.chunks(5).zip(theirs.chunks(5)).enumerate() {
        let ob: Vec<&[u8]> = o.iter().map(|r| r.bytes.as_slice()).collect();
        let tb: Vec<&[u8]> = t.iter().map(|r| r.2.as_slice()).collect();
        if ob.len() < 5 || tb.len() < 5 {
            continue;
        }
        let (of, tf) = (item_fields(&ob), item_fields(&tb));
        if of[0] != 0 || tf[0] != 0 {
            seen.push((k, of[0], tf[0]));
        }
        let row: Vec<String> = names
            .iter()
            .zip(of.iter().zip(&tf))
            .map(|(n, (a, b))| {
                if a == b {
                    format!("{n} {a}")
                } else {
                    format!("{n} {a}|{b}")
                }
            })
            .collect();
        eprintln!("GOOD {k}: {}", row.join(", "));
    }
    let parted = parting(&ours, &theirs);
    for p in &parted {
        eprintln!("PARTS {p:?}");
    }
    eprintln!(
        "goods: ours {} ranges {} bytes adler {}; theirs {} ranges {} bytes; printed {want}; ever_seen not 0 {seen:?}",
        ours.ranges.len(),
        ours.bytes(),
        ours.adler(),
        theirs.len(),
        theirs.iter().map(|t| t.2.len()).sum::<usize>(),
    );
    assert_eq!(
        (
            ours.ranges.len() / 5,
            theirs.len(),
            theirs.iter().map(|t| t.2.len()).sum::<usize>(),
            parted.len()
        ),
        GOODS,
        "the goods line: goods, the stream's ranges and bytes, ranges parted"
    );
    // One good of 36 has been seen in the original, good 15 (the SILK,
    // type 17), by leader 1: `ever_seen` 2 there, 0 in ours.
    assert_eq!(seen, [(15, 0, 2)], "the goods a leader has seen");
    assert_eq!(
        parted
            .iter()
            .map(|p| (p.0, p.1, p.2.as_str(), p.3.as_str(), p.4.as_str()))
            .collect::<Vec<_>>(),
        [(75, 15, "ever_seen", "00", "02")],
        "the one byte the goods line is held out on"
    );
    assert_eq!(ours.bytes(), GOODS.2);
    assert_ne!(ours.adler(), want, "the goods line is held out");
    // The rest of the line is the original's: with that one byte as the
    // original holds it, ours sums to the printed `Goods`. A test-side
    // substitution, never the image.
    let mut held = ours.clone();
    held.ranges[15 * 5].bytes[0] = 2;
    assert_eq!(held.adler(), want, "791 of 792 bytes are the original's");

    // **It can part.** One more byte of ours perturbed — good 3's `x`,
    // low byte — parts on that range as well, and only there.
    let mut bent = ours.clone();
    bent.ranges[3 * 5 + 3].bytes[7] ^= 1;
    let p = parting(&bent, &theirs);
    assert_eq!(
        p.iter()
            .map(|p| (p.0, p.1, p.2.as_str()))
            .collect::<Vec<_>>(),
        [(18, 3, "who..y_internal"), (75, 15, "ever_seen")],
        "one perturbed byte parts on its range"
    );
    assert_eq!(
        parting(&sim::checksum::Line::default(), &theirs).len(),
        GOODS.1
    );
}

/// **The score**: the state lines ported are the ones `CHECKSUM_STATE`
/// counts, their bytes its bytes (`docs/CHECKSUM.md` §5). Each line's own
/// test holds it against the stream; this holds the sum.
#[test]
fn the_ported_lines_are_checksum_state() {
    let Some((_, _, sim)) = line_inputs("items", "Items") else {
        return;
    };
    // Goods is held out (`the_goods_line_is_held_out_on_good_15_s_ever_seen`).
    let lines = [sim.checksum_items(), sim.checksum_cities()];
    let bytes: usize = lines.iter().map(sim::checksum::Line::bytes).sum();
    let (n, _, b, _) = crate::checksum::CHECKSUM_STATE;
    assert_eq!((n, b), (lines.len(), bytes as u64), "CHECKSUM_STATE");
}
