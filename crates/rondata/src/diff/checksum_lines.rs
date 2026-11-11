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
/// ranges, bytes, ranges parted)` (`docs/CHECKSUM.md` §13).
const GOODS: (usize, usize, usize, usize) = (36, 180, 792, 0);

/// **The goods line, ours against the original's, range by range**
/// (`docs/CHECKSUM.md` §13, item 4093): our `Sim::checksum_goods` against
/// the instrument's stream of the goods line, every range, then each good's
/// nine fields decoded both sides, then the Adler against the printed
/// `Goods`. One good of 36 has been seen on the frame, good 15 (`AMBER`,
/// type 17) by leader 1: `ever_seen` 2 on both sides, the bit setup's
/// `compute_all_territory` goods scan sets (`Sim::claim_rares_from_owners`).
#[test]
fn the_goods_line_is_the_original_s_range_by_range() {
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
    // One good of 36 has been seen, good 15 (type 17), by leader 1:
    // `ever_seen` 2 on both sides.
    assert_eq!(seen, [(15, 2, 2)], "the goods a leader has seen");
    assert_eq!(ours.bytes(), GOODS.2);
    // The Adler, last: ours against the line the original printed.
    assert_eq!(ours.adler(), want, "the goods line's Adler");

    // **It can part.** Good 15's `ever_seen` back at the unported 0 parts
    // on its own range alone, and the Adler with it.
    let mut unseen = ours.clone();
    unseen.ranges[15 * 5].bytes[0] = 0;
    let p = parting(&unseen, &theirs);
    assert_eq!(
        p.iter()
            .map(|p| (p.0, p.1, p.2.as_str(), p.3.as_str(), p.4.as_str()))
            .collect::<Vec<_>>(),
        [(75, 15, "ever_seen", "00", "02")],
        "the setup scan's byte parts on its range"
    );
    assert_ne!(unseen.adler(), want);
    // One byte elsewhere perturbed, good 3's `x` low byte, parts on that
    // range alone.
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
    assert_eq!(parting(&Line::default(), &theirs).len(), GOODS.1);
}

/// One fixed-width field of a builds record, decoded: `char`s and the
/// named signed bytes signed, every other byte unsigned, `ulong`s and
/// `ushort`s unsigned, every other word signed.
fn build_field(name: &str, b: &[u8], at: usize, width: usize) -> i64 {
    let signed_byte = matches!(
        name,
        "infiltrated"
            | "damage_frac"
            | "mylos"
            | "targeted"
            | "inside_down_who"
            | "up_who"
            | "visible"
            | "launch_frames"
            | "stance"
            | "founder"
            | "gather_max"
            | "attack_whom"
    );
    match width {
        1 if signed_byte => i64::from(b[at] as i8),
        1 => i64::from(b[at]),
        2 if matches!(name, "uid" | "hold_frames") => {
            i64::from(u16::from_le_bytes([b[at], b[at + 1]]))
        }
        2 => i64::from(i16::from_le_bytes([b[at], b[at + 1]])),
        _ if matches!(name, "job_counter" | "job_counter_2" | "constr_time") && at >= 0x48 => {
            i64::from(u32::from_le_bytes(b[at..at + 4].try_into().unwrap()))
        }
        _ => i64::from(i32::from_le_bytes(b[at..at + 4].try_into().unwrap())),
    }
}

/// A builds range decoded field by field: by the type record's table where
/// the range is one, else as its local (`(name, value)` pairs).
fn build_range_fields(field: &str, b: &[u8]) -> Vec<(String, i64)> {
    use sim::checksum::builds::{BUILD_FIELDS, OBJECT_FIELDS, QUEUE_ITEM_FIELDS, WALL_FIELDS};
    let table = |t: &[(&'static str, usize, usize)], base: usize| -> Vec<(String, i64)> {
        t.iter()
            .map(|&(n, at, w)| (n.to_string(), build_field(n, b, at - base, w)))
            .collect()
    };
    match field {
        "myhits..launch_frames" => table(OBJECT_FIELDS, 0x20),
        "job_counter..demolition" => table(WALL_FIELDS, 0x48),
        "gather_down..infiltrate2" => table(BUILD_FIELDS, 0x70),
        "queue[k]" => table(QUEUE_ITEM_FIELDS, 0),
        "who..y_internal" => {
            let w = |at: usize| {
                i64::from(i32::from_le_bytes(b[at..at + 4].try_into().unwrap()) ^ COORD_MASK)
            };
            vec![
                ("who".into(), i64::from(b[0])),
                ("o".into(), i64::from(i16::from_le_bytes([b[1], b[2]]))),
                ("z".into(), w(3)),
                ("x".into(), w(7)),
                ("y".into(), w(11)),
            ]
        }
        "mtn..cliff" => vec![
            ("mtn".into(), i64::from(b[0] as i8)),
            ("cliff".into(), i64::from(b[1] as i8)),
        ],
        "gather_from[k]" => vec![
            (
                "tx".into(),
                i64::from(i32::from_le_bytes(b[0..4].try_into().unwrap())),
            ),
            (
                "ty".into(),
                i64::from(i32::from_le_bytes(b[4..8].try_into().unwrap())),
            ),
        ],
        _ => {
            let v = match b.len() {
                1 => i64::from(b[0]),
                2 => i64::from(i16::from_le_bytes([b[0], b[1]])),
                _ => i64::from(i32::from_le_bytes(b[..4].try_into().unwrap())),
            };
            vec![(field.to_string(), v)]
        }
    }
}

/// The stream's builds rows, each named as ours names the range it is:
/// the call site settles which walker's range a row is (`docs/CHECKSUM.md`
/// §14), independently of our image.
fn build_site_field(site: &str) -> &'static str {
    match site {
        "0062f28c" => "founder",
        "0062f2a0" => "max_age",
        "006621fe" => "flags",
        "00647965" | "00642570" | "0062f2fc" => "must_walk",
        "0066221c" => "who..y_internal",
        "00662287" => "type",
        "00647885" => "myhits..launch_frames",
        "00647910" => "launching",
        "00642586" => "job_counter..demolition",
        "0062f316" => "gather_down..infiltrate2",
        "0063061b" => "queue_size",
        "00630653" => "queue[k]",
        "0062f336" => "mtn..cliff",
        "00471c66" => "gather_from.length",
        "00471cc7" => "gather_from.size",
        "00471cd5" => "gather_from.increment",
        "00471ced" => "gather_from.flags",
        "00471df3" => "gather_from[k]",
        "004708c1" => "gather.length",
        "0062f35a" => "orig_type",
        _ => "?",
    }
}

/// The renderer's piece of every building of `sim`, by handle: the data
/// layer's `GraphicPieces::get_build_gpiece` walk
/// ([`crate::free::placed_buildings`]), the `WallData::gpiece` the builds
/// line folds.
fn build_pieces(sim: &sim::Sim) -> std::collections::BTreeMap<usize, i32> {
    let inst = crate::testenv::install().expect("the install");
    let loaded = crate::load::load(&inst).unwrap();
    let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
    let tribes = inst.tribe_defs(&inst.rules().unwrap()).unwrap();
    let scene = sim.observe(None);
    let placed = crate::free::placed_buildings(&scene, &look, &tribes);
    let mut out = std::collections::BTreeMap::new();
    for (i, b) in sim.buildings.iter().enumerate() {
        if let Some(p) = placed
            .iter()
            .find(|p| p.who == b.owner && p.o == b.index && b.alive)
        {
            out.insert(i, p.piece);
        }
    }
    out
}

/// The pinned outcome of the builds line on run 1562's packet:
/// `(buildings, ranges, bytes, ranges parted, bytes in the ranges that
/// agree)` (`docs/CHECKSUM.md` §14, §15): 519 of 519 ranges and 4,923 of
/// 4,923 bytes. Item 4094 left it at 516 and 4,845, held out on both camps'
/// `recharging` (item 4097, `docs/ORDERS.md` §6.11) and the British camp's
/// `up_who`, ours −1 against 1: the chain of objects' stale word, held since
/// item 4098 (`docs/COLLISION.md` §3.2).
const BUILDS: (usize, usize, usize, usize, usize) = (15, 519, 4923, 0, 4923);

/// The range of the British Woodcutter's Camp `1/2001`'s `ObjectData`
/// (`myhits..launch_frames`) in the builds line, building 9 of 15, and the
/// offset of its `up_who` (+0x3f) in that range.
const CAMP_OBJECT_RANGE: (usize, usize) = (334, 0x3f - 0x20);

/// **The builds line, ours against the original's, range by range**
/// (`docs/CHECKSUM.md` §14, §15; items 4094, 4097, 4098): our
/// `Sim::checksum_builds` against the instrument's stream of the builds
/// line, every range, each range decoded field by field on both sides —
/// ours by our range names, theirs by each row's call site — then the
/// Adler against the printed `Builds`, with no substitution. And it can
/// part: the British camp's `up_who` at the −1 a derived chain gave it
/// parts on its range alone, one perturbed byte on its own, and an empty
/// image on every range.
#[test]
fn the_builds_line_is_the_original_s_range_by_range() {
    let Some((theirs, want, sim)) = line_inputs("builds", "Builds") else {
        return;
    };
    let pieces = build_pieces(&sim);
    let ours = sim.checksum_builds(&|b| pieces.get(&b).copied().unwrap_or(-1));
    // Both sides' records, field by field: ours by our range names, theirs
    // by the call site of each row.
    let mut fields_parted: Vec<(usize, i32, String, i64, i64)> = Vec::new();
    let mut building = -1i32;
    for (i, t) in theirs.iter().enumerate() {
        let tf = build_site_field(&t.1);
        if tf == "founder" {
            building += 1;
        }
        let tv = build_range_fields(tf, &t.2);
        let Some(o) = ours.ranges.get(i) else {
            fields_parted.push((i, building, format!("{tf}: ours -"), 0, 0));
            continue;
        };
        if o.field != tf {
            fields_parted.push((i, building, format!("{tf}: ours walks {}", o.field), 0, 0));
            continue;
        }
        let ov = build_range_fields(o.field, &o.bytes);
        for ((n, a), (_, b)) in ov.iter().zip(&tv) {
            if a != b {
                fields_parted.push((i, building, n.clone(), *a, *b));
            }
        }
    }
    for f in &fields_parted {
        eprintln!("FIELD {f:?}");
    }
    let parted = parting(&ours, &theirs);
    let bytes_equal: usize = ours
        .ranges
        .iter()
        .zip(&theirs)
        .filter(|(o, t)| o.bytes == t.2)
        .map(|(o, _)| o.bytes.len())
        .sum();
    eprintln!(
        "builds: ours {} ranges {} bytes adler {}; theirs {} ranges {} bytes; printed {want}; ranges parted {}, bytes in equal ranges {bytes_equal}, fields parted {}",
        ours.ranges.len(),
        ours.bytes(),
        ours.adler(),
        theirs.len(),
        theirs.iter().map(|t| t.2.len()).sum::<usize>(),
        parted.len(),
        fields_parted.len(),
    );
    let buildings = ours.ranges.iter().filter(|r| r.field == "founder").count();
    assert_eq!(
        (
            buildings,
            theirs.len(),
            theirs.iter().map(|t| t.2.len()).sum::<usize>(),
            parted.len(),
            bytes_equal
        ),
        BUILDS,
        "the builds line: buildings, the stream's ranges and bytes, ranges parted, bytes agreeing"
    );
    assert_eq!(ours.bytes(), BUILDS.2);
    assert!(fields_parted.is_empty(), "every field of every building");
    // The camp's chain words on both sides, for a reader: `up` −1, `down`
    // −1, `down_who` 0, `up_who` 1 — a British unit stood above it and left.
    let (r, at) = CAMP_OBJECT_RANGE;
    let camp = build_range_fields("myhits..launch_frames", &ours.ranges[r].bytes);
    let word = |n: &str| camp.iter().find(|(k, _)| k == n).map(|(_, v)| *v);
    assert_eq!(
        (word("up"), word("down"), word("down_who"), word("up_who")),
        (Some(-1), Some(-1), Some(0), Some(1)),
        "the British camp's four chain words"
    );
    // The Adler, last: ours against the line the original printed.
    assert_eq!(ours.adler(), want, "the builds line's Adler");

    // **It can part.** The camp's `up_who` at the −1 a chain of units
    // alone derives parts on its range alone, and the Adler with it.
    let mut derived = ours.clone();
    derived.ranges[r].bytes[at] = 0xff;
    let p: Vec<usize> = parting(&derived, &theirs).iter().map(|p| p.0).collect();
    assert_eq!(p, vec![r], "the derived up_who parts on the camp's range");
    assert_ne!(derived.adler(), want);
    // One byte elsewhere, the fourth building's `x` (low byte), parts on
    // that range alone.
    let mut bent = ours.clone();
    let sub = bent
        .ranges
        .iter()
        .enumerate()
        .filter(|(_, r)| r.field == "who..y_internal")
        .nth(3)
        .map(|(i, _)| i)
        .unwrap();
    bent.ranges[sub].bytes[7] ^= 1;
    let p: Vec<usize> = parting(&bent, &theirs).iter().map(|p| p.0).collect();
    assert_eq!(p, vec![sub], "one perturbed byte parts on its range");
    assert_eq!(parting(&Line::default(), &theirs).len(), BUILDS.1);
}

/// The units' four chain words agreeing on run 1562's packet, of its 52
/// units: `(units, up, down, down_who, up_who)` (`docs/COLLISION.md` §3.2,
/// item 4098). Before the item a chain of units alone gave 52, 31, 28 and
/// 50: no building below a unit, no good's −2 at a herd's tail, the cell's
/// `down_who` nowhere, and two stale `up_who`s (`0/1` 0, `1/1` 1) lost.
/// Reported for the units line, which is not ported; not a score.
const UNIT_CHAIN_WORDS: (usize, usize, usize, usize, usize) = (52, 52, 52, 52, 52);

/// The four chain words of every unit of the stream's units line, by
/// `(who, o)`: `(up, down, down_who, up_who)`. The unit's `who` and `o`
/// are its `SubObject` range (`0066221c`); its words, the `ObjectData`
/// range after it (`00647885`).
fn stream_unit_chain_words(
    theirs: &[(String, String, Vec<u8>)],
) -> std::collections::BTreeMap<(u8, i16), (i16, i16, i16, i8)> {
    let mut out = std::collections::BTreeMap::new();
    let mut who_o = None;
    for (_, site, b) in theirs {
        match site.as_str() {
            "0066221c" => who_o = Some((b[0], i16::from_le_bytes([b[1], b[2]]))),
            "00647885" => {
                let w = |at: usize| i16::from_le_bytes([b[at], b[at + 1]]);
                if let Some(k) = who_o.take() {
                    out.insert(
                        k,
                        (
                            w(0x2a - 0x20),
                            w(0x2c - 0x20),
                            w(0x2e - 0x20),
                            b[0x3f - 0x20] as i8,
                        ),
                    );
                }
            }
            _ => {}
        }
    }
    out
}

/// **The units' four chain words, reported** (`docs/COLLISION.md` §3.2,
/// item 4098): every unit's `up`, `down`, `down_who` and `up_who` on the
/// packet's frame, ours (`Sim::unit_chain_words`, the whole chain of
/// objects with its stale words) against the stream's units line. The
/// units line is not ported; this is the count it starts from. It can
/// part: a unit's chain read from the units alone parts where a building
/// stands below it.
#[test]
fn the_units_four_chain_words_are_the_original_s() {
    let Some((theirs, _, sim)) = line_inputs("units", "Units") else {
        return;
    };
    let words = stream_unit_chain_words(&theirs);
    let ours: std::collections::BTreeMap<(u8, i16), (i16, i16, i16, i8)> = sim
        .units
        .iter()
        .enumerate()
        .filter(|(_, u)| u.alive())
        .map(|(i, u)| ((u.owner, u.index), sim.unit_chain_words(i)))
        .collect();
    let mut agree = [0usize; 4];
    for (k, t) in &words {
        let o = ours.get(k).copied();
        let pairs = [
            (o.map(|o| i64::from(o.0)), i64::from(t.0)),
            (o.map(|o| i64::from(o.1)), i64::from(t.1)),
            (o.map(|o| i64::from(o.2)), i64::from(t.2)),
            (o.map(|o| i64::from(o.3)), i64::from(t.3)),
        ];
        for (n, (a, b)) in pairs.iter().enumerate() {
            if *a == Some(*b) {
                agree[n] += 1;
            } else {
                let name = ["up", "down", "down_who", "up_who"][n];
                eprintln!("UNIT {}/{} {name}: ours {a:?} theirs {b}", k.0, k.1);
            }
        }
    }
    eprintln!(
        "units' chain words of {}: up, down, down_who, up_who agreeing {agree:?}",
        words.len()
    );
    assert_eq!(
        (words.len(), agree[0], agree[1], agree[2], agree[3]),
        UNIT_CHAIN_WORDS,
        "the units' four chain words agreeing"
    );
    // It can part: `0/2`'s `down` from the units' chain alone is −1, where
    // the human's Woodcutter's Camp `0/2001` stands below it.
    let u = sim
        .units
        .iter()
        .position(|u| u.owner == 0 && u.index == 2 && u.alive())
        .expect("0/2");
    let units_only = sim.units[u].down.map_or(-1, |d| sim.units[d].index);
    assert_eq!((units_only, words[&(0, 2)].1), (-1, 2001));
}

/// The original's playback of the minted file with every building printed
/// on each of its 300 frames (`docs/RUNS.md` run1561a).
const RUN1561A: &str = "gamelog-run1561a-greatlakes-playback-of-minted-300.txt";

/// The last frame of [`RUN1561A`] whose dump is whole, and the pinned count
/// of `(frame, building)` pairs it prints a `recharging` for from frame 1.
const RECHARGING_FRAMES: (i64, usize) = (299, 4394);

/// Every `BUILDDATA` record's `recharging` in `text`, by `(frame, who, o)`.
/// `gamelog::BuildDump` does not parse the key (`docs/ORDERS.md` §6.11), so
/// this reads it here: a record's first `o` and `who` are its
/// `SUBOBJECT`'s, and its one `recharging` line is `BuildData`'s.
fn recharging_by_frame(text: &str) -> std::collections::BTreeMap<(i64, i64, i64), i64> {
    let mut out = std::collections::BTreeMap::new();
    let (mut f, mut inb, mut o, mut w) = (0i64, false, None, None);
    for l in text.lines() {
        let t = l.trim_start();
        if let Some(r) = t.strip_prefix("BEGIN FRAME ") {
            f = r.trim().parse().expect("a frame number");
        } else if t == "BEGIN BUILDDATA" {
            (inb, o, w) = (true, None, None);
        } else if inb {
            if let Some(r) = t.strip_prefix("o ") {
                o = o.or(r.trim().parse().ok());
            } else if let Some(r) = t.strip_prefix("who ") {
                w = w.or(r.trim().parse().ok());
            } else if let Some(r) = t.strip_prefix("recharging ") {
                let k = (
                    f,
                    w.expect("who before recharging"),
                    o.expect("o before recharging"),
                );
                out.insert(k, r.trim().parse().expect("a recharging value"));
                inb = false;
            }
        }
    }
    out
}

/// **Every building's `recharging`, ours against the original's, on every
/// frame of the minted game's playback** (item 4097, `docs/ORDERS.md`
/// §6.11): run1561a prints `BuildData::recharging` on all fifteen buildings
/// for 299 whole frames, and our walk of the minted file holds every one.
/// It was the two Woodcutter's Camps that parted before the carrying walk's
/// bump: the human's from frame 4 (theirs 3, ours 2) and the British from 4
/// too, 83 and 67 against 90 and 89 on 91. It can part: one camp one count
/// off on one frame parts on that pair alone, and the key is read on every
/// frame of the dump, so a quiet field is not a blind one.
#[test]
fn every_building_s_recharging_is_the_original_s_on_every_frame_of_the_playback() {
    let Some(path) = crate::testenv::dump(RUN1561A) else {
        eprintln!("skipping: no {RUN1561A} (set RON_GAMELOG_DIR)");
        return;
    };
    let theirs = recharging_by_frame(&std::fs::read_to_string(path).expect("a kept dump reads"));
    let Some((_, mut built, _)) = built_start() else {
        eprintln!("skipping: no install or lobby dump (RON_INSTALL, RON_GAMELOG_DIR)");
        return;
    };
    let mut stream = crate::input::Stream::pump(&minted(&script()).expect("the minted file"));
    let (last, pairs) = RECHARGING_FRAMES;
    let mut ours = std::collections::BTreeMap::new();
    for frame in 1..=last {
        tick_to(&mut built, Some(&mut stream), frame);
        for b in built.sim.buildings.iter().filter(|b| b.alive) {
            let k = (frame, i64::from(b.owner), i64::from(b.index));
            ours.insert(k, i64::from(b.recharging));
        }
    }
    let theirs: std::collections::BTreeMap<_, _> =
        theirs.into_iter().filter(|(k, _)| k.0 <= last).collect();
    let parted = |ours: &std::collections::BTreeMap<(i64, i64, i64), i64>| {
        let keys: std::collections::BTreeSet<_> = ours.keys().chain(theirs.keys()).collect();
        keys.into_iter()
            .filter(|k| ours.get(k) != theirs.get(k))
            .map(|k| (*k, ours.get(k).copied(), theirs.get(k).copied()))
            .collect::<Vec<_>>()
    };
    let p = parted(&ours);
    for x in p.iter().take(20) {
        eprintln!("PARTED {x:?}");
    }
    assert_eq!(
        (
            theirs.len(),
            (1..=last)
                .filter(|f| theirs.keys().any(|k| k.0 == *f))
                .count()
        ),
        (pairs, last as usize),
        "the dump prints a recharging on every frame"
    );
    assert_eq!(
        theirs.get(&(91, 0, 2001)),
        Some(&90),
        "the human's camp on 91"
    );
    assert_eq!(
        theirs.get(&(91, 1, 2001)),
        Some(&89),
        "the British camp on 91"
    );
    assert!(p.is_empty(), "every building's recharging on every frame");
    // It can part.
    let mut bent = ours.clone();
    *bent
        .get_mut(&(16, 1, 2001))
        .expect("the British camp on 16") += 1;
    assert_eq!(parted(&bent), vec![((16, 1, 2001), Some(15), Some(14))]);
}

/// **The score**: the state lines ported are the ones `CHECKSUM_STATE`
/// counts, their bytes its bytes (`docs/CHECKSUM.md` §5). Each line's own
/// test holds it against the stream; this holds the sum.
#[test]
fn the_ported_lines_are_checksum_state() {
    let Some((_, _, sim)) = line_inputs("items", "Items") else {
        return;
    };
    let pieces = build_pieces(&sim);
    let lines = [
        sim.checksum_items(),
        sim.checksum_cities(),
        sim.checksum_goods(),
        sim.checksum_builds(&|b| pieces.get(&b).copied().unwrap_or(-1)),
        sim.checksum_guys(),
        sim.checksum_units(),
        sim.checksum_groups(),
    ];
    let bytes: usize = lines.iter().map(sim::checksum::Line::bytes).sum();
    let (n, _, b, _) = crate::checksum::CHECKSUM_STATE;
    assert_eq!((n, b), (lines.len(), bytes as u64), "CHECKSUM_STATE");
}

/// The guys line's range name for a stream row, by its call site
/// (`PtrArray<Guy>::walk_data@0046df30`, `GuyData::walk_data@005e0210`).
fn guy_site_field(site: &str) -> &'static str {
    match site {
        "0046df67" => "guys.length",
        "0046dfc8" => "guys.size",
        "0046dfd6" => "guys.increment",
        "0046dfee" => "guys.flags",
        "0046e199" => "guys[k].present",
        "0046e220" => "guys.size..increment",
        "005e0228" => "guy",
        _ => "?",
    }
}

/// One guys-line range decoded field by field: the figure record by
/// [`sim::checksum::guys::GUY_FIELDS`] (each field little-endian and
/// signed at its width, a turret array as its four words), any other
/// range as its one value.
fn guy_range_fields(field: &str, b: &[u8]) -> Vec<(String, i64)> {
    let le = |s: &[u8]| -> i64 {
        match s.len() {
            1 => i64::from(s[0] as i8),
            2 => i64::from(i16::from_le_bytes([s[0], s[1]])),
            _ => i64::from(i32::from_le_bytes(s[..4].try_into().unwrap())),
        }
    };
    if field == "guy" {
        let mut out = Vec::new();
        for &(n, at, w) in sim::checksum::guys::GUY_FIELDS {
            let s = &b[at - 8..at - 8 + w];
            if w == 16 {
                for k in 0..4 {
                    out.push((format!("{n}[{k}]"), le(&s[4 * k..4 * k + 4])));
                }
            } else {
                out.push((n.to_string(), le(s)));
            }
        }
        return out;
    }
    if field == "guys.size..increment" {
        return vec![
            ("size".into(), le(&b[0..4])),
            ("increment".into(), le(&b[4..6])),
        ];
    }
    vec![(field.to_string(), le(b))]
}

/// The pinned outcome of the guys line on run 1562's packet: `(units,
/// figures, ranges, bytes, ranges parted, bytes in the ranges that agree)`
/// (`docs/CHECKSUM.md` §16): 368 of 368 ranges and 9,308 of 9,308 bytes.
/// The first image (item 4104) parted on 45 ranges: the forty gaia
/// figures' `gpiece` (ours −1, theirs 60063..60074) and five moving
/// figures' `last_x`/`last_y`/`last_z`, one of them its `last_angle`.
const GUYS: (usize, usize, usize, usize, usize, usize) = (52, 54, 368, 9308, 0, 9308);

/// The five `f32` fields of a figure (`GuyData +0x40..+0x54`), by name.
const GUY_FLOATS: [&str; 5] = ["turret_inc", "bank", "last_bank", "pitch", "last_pitch"];

/// **The guys line, ours against the original's, range by range**
/// (`docs/CHECKSUM.md` §16, item 4104): our `Sim::checksum_guys` against
/// the instrument's stream of the guys line, every range, each figure
/// decoded field by field on both sides by `GUY_FIELDS`, then the Adler
/// against the printed `Guys`, with no substitution. The five floats are
/// held as bits, and are `+0.0` on every figure of both sides. And it can
/// part: a herd's piece at the −1 this crate's figure keeps parts on its
/// range alone, a figure's `last_x` at its own `x` parts on that range,
/// a float bit flipped parts on its range, and an empty image on every
/// range.
#[test]
fn the_guys_line_is_the_original_s_range_by_range() {
    let Some((theirs, want, sim)) = line_inputs("guys", "Guys") else {
        return;
    };
    let ours = sim.checksum_guys();
    let mut parted_by_field: std::collections::BTreeMap<String, Vec<(usize, i64, i64)>> =
        std::collections::BTreeMap::new();
    let mut floats_theirs = std::collections::BTreeSet::new();
    let mut figure = 0usize;
    for (i, t) in theirs.iter().enumerate() {
        let tf = guy_site_field(&t.1);
        let Some(o) = ours.ranges.get(i) else {
            parted_by_field.entry(format!("{tf}: ours -")).or_default();
            continue;
        };
        if o.field != tf {
            parted_by_field
                .entry(format!("{tf}: ours walks {}", o.field))
                .or_default()
                .push((i, 0, 0));
            continue;
        }
        let tv = guy_range_fields(tf, &t.2);
        let ov = guy_range_fields(o.field, &o.bytes);
        for ((n, a), (_, b)) in ov.iter().zip(&tv) {
            if GUY_FLOATS.contains(&n.as_str()) {
                floats_theirs.insert((n.clone(), *b as u32));
            }
            if a != b {
                parted_by_field
                    .entry(n.clone())
                    .or_default()
                    .push((figure, *a, *b));
            }
        }
        if tf == "guy" {
            figure += 1;
        }
    }
    for (f, v) in &parted_by_field {
        eprintln!("FIELD {f}: {} parted; {:?}", v.len(), &v[..v.len().min(8)]);
    }
    let parted = parting(&ours, &theirs);
    let bytes_equal: usize = ours
        .ranges
        .iter()
        .zip(&theirs)
        .filter(|(o, t)| o.bytes == t.2)
        .map(|(o, _)| o.bytes.len())
        .sum();
    eprintln!(
        "guys: ours {} ranges {} bytes adler {}; theirs {} ranges {} bytes; printed {want}; ranges parted {}, bytes in equal ranges {bytes_equal}",
        ours.ranges.len(),
        ours.bytes(),
        ours.adler(),
        theirs.len(),
        theirs.iter().map(|t| t.2.len()).sum::<usize>(),
        parted.len(),
    );
    let units = ours
        .ranges
        .iter()
        .filter(|r| r.field == "guys.length")
        .count();
    let figures = ours.ranges.iter().filter(|r| r.field == "guy").count();
    assert_eq!(
        (
            units,
            figures,
            theirs.len(),
            theirs.iter().map(|t| t.2.len()).sum::<usize>(),
            parted.len(),
            bytes_equal
        ),
        GUYS,
        "the guys line: units, figures, the stream's ranges and bytes, ranges parted, bytes agreeing"
    );
    assert_eq!(ours.bytes(), GUYS.3);
    assert!(parted_by_field.is_empty(), "every field of every figure");
    // The five floats, the original's side: one bit pattern each, +0.0,
    // on all 54 figures (`docs/CHECKSUM.md` §16).
    assert_eq!(
        floats_theirs,
        GUY_FLOATS.iter().map(|n| (n.to_string(), 0u32)).collect(),
        "every float of every figure is +0.0 on the original's side"
    );
    // The Adler, last: ours against the line the original printed.
    assert_eq!(ours.adler(), want, "the guys line's Adler");

    // **It can part.** Each perturbation parts on its own range alone.
    let guy_range = |k: usize| {
        ours.ranges
            .iter()
            .enumerate()
            .filter(|(_, r)| r.field == "guy")
            .nth(k)
            .map(|(i, _)| i)
            .unwrap()
    };
    let field_at = |n: &str| {
        sim::checksum::guys::GUY_FIELDS
            .iter()
            .find(|f| f.0 == n)
            .map(|f| f.1 - 8)
            .unwrap()
    };
    let bend = |k: usize, at: usize, v: &[u8]| {
        let mut bent = ours.clone();
        let r = guy_range(k);
        bent.ranges[r].bytes[at..at + v.len()].copy_from_slice(v);
        let p: Vec<usize> = parting(&bent, &theirs).iter().map(|p| p.0).collect();
        (p, r, bent.adler())
    };
    // The first herd figure's piece at the −1 this crate's figure keeps.
    let herd = (0..figures)
        .find(|&k| {
            let r = &ours.ranges[guy_range(k)];
            i32::from_le_bytes(r.bytes[0..4].try_into().unwrap()) >= 0x192
        })
        .expect("a herd figure");
    let (p, r, a) = bend(herd, field_at("gpiece"), &(-1i32).to_le_bytes());
    assert_eq!(p, vec![r], "a herd's piece at -1 parts on its range");
    assert_ne!(a, want);
    // The first moving figure's `last_x` at its own `x`.
    let moving = (0..figures)
        .find(|&k| {
            let b = &ours.ranges[guy_range(k)].bytes;
            b[field_at("x")..field_at("x") + 4] != b[field_at("last_x")..field_at("last_x") + 4]
        })
        .expect("a figure that moved this frame");
    let x = field_at("x");
    let xb: Vec<u8> = ours.ranges[guy_range(moving)].bytes[x..x + 4].to_vec();
    let (p, r, _) = bend(moving, field_at("last_x"), &xb);
    assert_eq!(p, vec![r], "last_x at x parts on the moving figure's range");
    // A float's sign bit.
    let (p, r, _) = bend(3, field_at("bank") + 3, &[0x80]);
    assert_eq!(p, vec![r], "a -0.0 bank parts on its range");
    assert_eq!(parting(&Line::default(), &theirs).len(), GUYS.2);
}

/// The units line's range name for a stream row, by its call site
/// (`docs/CHECKSUM.md` §17): the unit's own walkers, then the figure
/// array's ([`guy_site_field`]).
fn unit_site_field(site: &str) -> &'static str {
    match site {
        "006621fe" => "flags",
        "0060d07f" | "0060cfae" | "005d7f0a" => "must_walk",
        "0066221c" => "who..y_internal",
        "00662287" => "type",
        "00647885" => "myhits..launch_frames",
        "00647910" => "launching",
        "0060cfcd" => "collide_frame..play",
        "0046d8df" => "path.size..increment",
        "0046d980" => "path[k]",
        "00730291" => "orders.count",
        "007303ac" => "order.type",
        "007303be" => "order.node",
        "0047f243" | "00482e93" | "00486e83" => "order.flags",
        "0047f251" | "00486e91" => "order.target",
        "00482ea1" => "order.move",
        "00486e9f" => "order.gather",
        "005d7f26" => "ox..aid",
        s => guy_site_field(s),
    }
}

/// One units-line range decoded field by field: a record by its table
/// (each field little-endian and signed at its width), the sub-object's
/// `Coord`s unmasked, a figure by [`guy_range_fields`], anything else as
/// its one value.
fn unit_range_fields(field: &str, b: &[u8]) -> Vec<(String, i64)> {
    use sim::checksum::units::{GATHER_FIELDS, MOVE_FIELDS, TARGET_FIELDS, UNIT_FIELDS};
    let le = |s: &[u8]| -> i64 {
        match s.len() {
            1 => i64::from(s[0] as i8),
            2 => i64::from(i16::from_le_bytes([s[0], s[1]])),
            _ => i64::from(i32::from_le_bytes(s[..4].try_into().unwrap())),
        }
    };
    let table = |t: &[(&str, usize, usize)], base: usize| -> Vec<(String, i64)> {
        t.iter()
            .map(|&(n, at, w)| (n.to_string(), le(&b[at - base..at - base + w])))
            .collect()
    };
    match field {
        "myhits..launch_frames" => table(sim::checksum::builds::OBJECT_FIELDS, 0x20),
        "collide_frame..play" => table(UNIT_FIELDS, 0x48),
        "order.move" => table(MOVE_FIELDS, 0x4),
        "order.target" => table(TARGET_FIELDS, 0x8),
        "order.gather" => table(GATHER_FIELDS, 0x14),
        "who..y_internal" => {
            let c = |at: usize| le(&b[at..at + 4]) ^ i64::from(COORD_MASK);
            vec![
                ("who".into(), i64::from(b[0])),
                ("o".into(), le(&b[1..3])),
                ("z".into(), c(3)),
                ("x".into(), c(7)),
                ("y".into(), c(11)),
            ]
        }
        "path.size..increment" => vec![
            ("path.size".into(), le(&b[0..4])),
            ("path.length".into(), le(&b[4..8])),
            ("path.increment".into(), le(&b[8..9])),
        ],
        "path[k]" => vec![
            ("to_x".into(), le(&b[0..4])),
            ("to_y".into(), le(&b[4..8])),
            ("path.tolerance".into(), le(&b[8..12])),
            ("path.flags".into(), le(&b[12..16])),
        ],
        "ox..aid" => vec![
            ("animal.ox".into(), le(&b[0..2])),
            ("animal.whom".into(), le(&b[2..4])),
            ("animal.aid".into(), le(&b[4..5])),
        ],
        "guy" | "guys.size..increment" => guy_range_fields(field, b),
        _ => vec![(field.to_string(), le(b))],
    }
}

/// The pinned outcome of the units line on run 1562's packet (`docs/CHECKSUM.md`
/// §17, §18): `(units, ranges, bytes, ranges parted, bytes in the ranges
/// that agree)` — the stream's 52 units, 1,084 ranges and 19,642 bytes,
/// every one agreeing since item 4113. Item 4109 held it at 1,078 ranges
/// and 18,976 bytes, the six `UnitData` records of `0/0`, `0/1`, `0/2`,
/// `1/0`, `1/1` and `1/2` parting; its first image parted on 88 (the forty
/// animals' `mylos`, `orders_x/y` and `dest_angle`, the two scouts'
/// `flags`).
const UNITS: (usize, usize, usize, usize, usize) = (52, 1084, 19642, 0, 19642);

/// **The fields the units line is held out on**, each with its count of
/// parting units on run 1562's frame 91 (`docs/CHECKSUM.md` §17): none
/// since item 4113 (§18). Item 4109 held it out on three, each now carried:
///
/// - `play` (`+0xb6`) on 2, −1 against 0 on `0/0` and `0/1`:
///   `CommandPackage::process_group`'s write (`docs/ORDERS.md` §8.1);
/// - `u86` (`+0x86`) on 4, −1 against 0 on the scouts `0/0`, `1/0` and on
///   `0/2`, 2 on `1/2`: the specials' slot (`docs/SCOUT.md` §16) and the
///   hold doober's (`docs/ORDERS.md` §6.12);
/// - `unit_masks` bit `0x8` on 6: [`sim::Unit::line_ok`], which the crate
///   already kept (`docs/MOVEMENT.md`).
const UNITS_HELD_OUT: &[(&str, usize)] = &[];

/// **The units line, ours against the original's, range by range**
/// (`docs/CHECKSUM.md` §17, item 4109): our `Sim::checksum_units` against
/// the instrument's stream of the units line, every range in order, each
/// decoded field by field on both sides by the same tables, then the
/// Adler against the printed `Units`. Held out on [`UNITS_HELD_OUT`], each
/// field named with its count of parting units; the agreeing ranges and
/// bytes pinned in [`UNITS`]. And it can part: one perturbed byte of each
/// kind of range parts on that range alone, and the empty image on all.
#[test]
fn the_units_line_is_the_original_s_range_by_range() {
    let Some((theirs, want, sim)) = line_inputs("units", "Units") else {
        return;
    };
    let ours = sim.checksum_units();
    let mut parted_by_field: std::collections::BTreeMap<String, Vec<(String, i64, i64)>> =
        std::collections::BTreeMap::new();
    let mut unit = (0u8, 0i16);
    for (i, t) in theirs.iter().enumerate() {
        let tf = unit_site_field(&t.1);
        let Some(o) = ours.ranges.get(i) else {
            parted_by_field
                .entry(format!("{tf}: ours -"))
                .or_default()
                .push((format!("range {i}"), 0, 0));
            continue;
        };
        if o.field != tf {
            parted_by_field
                .entry(format!("{tf}: ours walks {}", o.field))
                .or_default()
                .push((format!("range {i}"), 0, 0));
            continue;
        }
        // A unit's first range is its `flags`; its `who` and `o` two rows on.
        if tf == "flags"
            && let Some(w) = theirs.get(i + 2).filter(|w| w.2.len() == 15)
        {
            unit = (w.2[0], i16::from_le_bytes([w.2[1], w.2[2]]));
        }
        let tv = unit_range_fields(tf, &t.2);
        let ov = unit_range_fields(o.field, &o.bytes);
        for ((n, a), (_, b)) in ov.iter().zip(&tv) {
            if a != b {
                parted_by_field.entry(n.clone()).or_default().push((
                    format!("{}/{}", unit.0, unit.1),
                    *a,
                    *b,
                ));
            }
        }
    }
    for (f, v) in &parted_by_field {
        eprintln!("FIELD {f}: {} parted; {:?}", v.len(), &v[..v.len().min(8)]);
    }
    let parted = parting(&ours, &theirs);
    let bytes_equal: usize = ours
        .ranges
        .iter()
        .zip(&theirs)
        .filter(|(o, t)| o.bytes == t.2)
        .map(|(o, _)| o.bytes.len())
        .sum();
    eprintln!(
        "units: ours {} ranges {} bytes adler {}; theirs {} ranges {} bytes; printed {want}; ranges parted {}, bytes in equal ranges {bytes_equal}",
        ours.ranges.len(),
        ours.bytes(),
        ours.adler(),
        theirs.len(),
        theirs.iter().map(|t| t.2.len()).sum::<usize>(),
        parted.len(),
    );
    let units = ours.ranges.iter().filter(|r| r.field == "flags").count();
    assert_eq!(
        (
            units,
            theirs.len(),
            theirs.iter().map(|t| t.2.len()).sum::<usize>(),
            parted.len(),
            bytes_equal
        ),
        UNITS,
        "the units line: units, the stream's ranges and bytes, ranges parted, bytes agreeing"
    );
    assert_eq!(ours.ranges.len(), UNITS.1, "our walk's ranges");
    assert_eq!(ours.bytes(), UNITS.2, "our walk's bytes");
    let held: Vec<(&str, usize)> = parted_by_field
        .iter()
        .map(|(f, v)| (f.as_str(), v.len()))
        .collect();
    assert_eq!(held, UNITS_HELD_OUT, "the fields the line is held out on");
    if UNITS_HELD_OUT.is_empty() {
        assert_eq!(ours.adler(), want, "the units line's Adler");
    } else {
        assert_ne!(
            ours.adler(),
            want,
            "a held-out line cannot sum to the printed value"
        );
    }

    // **It can part.** One byte of each kind of range, perturbed, parts on
    // that range alone.
    let agreeing: Vec<usize> = (0..ours.ranges.len())
        .filter(|&i| ours.ranges[i].bytes == theirs[i].2)
        .collect();
    let mut kinds = std::collections::BTreeSet::new();
    for &i in &agreeing {
        if !kinds.insert(ours.ranges[i].field) {
            continue;
        }
        let mut bent = ours.clone();
        bent.ranges[i].bytes[0] ^= 0x01;
        let p: Vec<usize> = parting(&bent, &theirs).iter().map(|p| p.0).collect();
        let base: Vec<usize> = parted.iter().map(|p| p.0).collect();
        let mut want_p = base.clone();
        want_p.push(i);
        want_p.sort_unstable();
        assert_eq!(
            p, want_p,
            "a bent {} parts on range {i} alone",
            ours.ranges[i].field
        );
    }
    eprintln!("units: {} kinds of range bent", kinds.len());
    assert_eq!(parting(&Line::default(), &theirs).len(), UNITS.1);
}

/// The groups line's range name for a stream row, by its call site
/// (`Group::walk_data@00708400`, `CheckSums::check_groups@00937530`).
fn group_site_field(site: &str) -> &'static str {
    match site {
        "00708419" => "id..march",
        "0070843a" => "list",
        "0070844b" => "off_x",
        "00708464" => "off_y",
        "0070847d" => "curr_x",
        "00708496" => "curr_y",
        "007084ae" => "angles",
        "00937587" => "last_group",
        _ => "?",
    }
}

/// One groups-line range decoded field by field: a record by
/// [`sim::checksum::groups::GROUP_RECORD_FIELDS`] (each little-endian and signed
/// at its width), a member array entry by entry, `last_group` as its word.
fn group_range_fields(field: &str, b: &[u8]) -> Vec<(String, i64)> {
    let le = |s: &[u8]| -> i64 {
        match s.len() {
            1 => i64::from(s[0] as i8),
            2 => i64::from(i16::from_le_bytes([s[0], s[1]])),
            _ => i64::from(i32::from_le_bytes(s[..4].try_into().unwrap())),
        }
    };
    let width = match field {
        "id..march" => {
            return sim::checksum::groups::GROUP_RECORD_FIELDS
                .iter()
                .map(|&(n, at, w)| (n.to_string(), le(&b[at - 4..at - 4 + w])))
                .collect();
        }
        "list" => 2,
        "angles" => 1,
        _ => 4,
    };
    b.chunks(width)
        .enumerate()
        .map(|(k, c)| (format!("{field}[{k}]"), le(c)))
        .collect()
}

/// The pinned outcome of the groups line on run 1562's packet
/// (`docs/CHECKSUM.md` §19): `(slots, slots in use, ranges, bytes, ranges
/// parted, bytes in the ranges that agree)` — 512 slots, six in use (each
/// one member), 556 ranges and 37,010 bytes, every one agreeing. The first
/// image of item 4116 parted on the two scout groups' `role` (slots 1 and
/// 65, 0 against `0x40110`).
const GROUPS: (usize, usize, usize, usize, usize, usize) = (512, 6, 556, 37010, 0, 37010);

/// **The fields the groups line is held out on**, each with its count of
/// parting slots on run 1562's frame 91: none since item 4116 carried
/// `role` (`Group::find_role`, `docs/GROUPS.md` §3.4).
const GROUPS_HELD_OUT: &[(&str, usize)] = &[];

/// **The groups line, ours against the original's, range by range**
/// (`docs/CHECKSUM.md` §19, item 4116): our `Sim::checksum_groups` against
/// the instrument's stream, every range in order, each decoded field by
/// field on both sides by the same table, then the Adler against the
/// printed `Groups`. Held out on [`GROUPS_HELD_OUT`]; the agreeing ranges
/// and bytes pinned in [`GROUPS`]. And it can part: one perturbed byte of
/// each kind of range parts on that range alone, and the empty image on
/// all.
#[test]
fn the_groups_line_is_the_original_s_range_by_range() {
    let Some((theirs, want, sim)) = line_inputs("groups", "Groups") else {
        return;
    };
    let ours = sim.checksum_groups();
    let mut parted_by_field: std::collections::BTreeMap<String, Vec<(i16, i64, i64)>> =
        std::collections::BTreeMap::new();
    for (i, t) in theirs.iter().enumerate() {
        let tf = group_site_field(&t.1);
        let Some(o) = ours.ranges.get(i) else {
            parted_by_field
                .entry(format!("{tf}: ours -"))
                .or_default()
                .push((-1, i as i64, 0));
            continue;
        };
        if o.field != tf {
            parted_by_field
                .entry(format!("{tf}: ours walks {}", o.field))
                .or_default()
                .push((o.record, i as i64, 0));
            continue;
        }
        let tv = group_range_fields(tf, &t.2);
        let ov = group_range_fields(o.field, &o.bytes);
        for ((n, a), (_, b)) in ov.iter().zip(&tv) {
            if a != b {
                parted_by_field
                    .entry(n.clone())
                    .or_default()
                    .push((o.record, *a, *b));
            }
        }
    }
    for (f, v) in &parted_by_field {
        eprintln!("FIELD {f}: {} parted; {:?}", v.len(), &v[..v.len().min(8)]);
    }
    let parted = parting(&ours, &theirs);
    let bytes_equal: usize = ours
        .ranges
        .iter()
        .zip(&theirs)
        .filter(|(o, t)| o.bytes == t.2)
        .map(|(o, _)| o.bytes.len())
        .sum();
    eprintln!(
        "groups: ours {} ranges {} bytes adler {}; theirs {} ranges {} bytes; printed {want}; ranges parted {}, bytes in equal ranges {bytes_equal}",
        ours.ranges.len(),
        ours.bytes(),
        ours.adler(),
        theirs.len(),
        theirs.iter().map(|t| t.2.len()).sum::<usize>(),
        parted.len(),
    );
    let slots = ours
        .ranges
        .iter()
        .filter(|r| r.field == "id..march")
        .count();
    let in_use = ours.ranges.iter().filter(|r| r.field == "list").count();
    assert_eq!(
        (
            slots,
            in_use,
            theirs.len(),
            theirs.iter().map(|t| t.2.len()).sum::<usize>(),
            parted.len(),
            bytes_equal
        ),
        GROUPS,
        "the groups line: slots, slots in use, the stream's ranges and bytes, ranges parted, bytes agreeing"
    );
    assert_eq!(ours.ranges.len(), GROUPS.2, "our walk's ranges");
    assert_eq!(ours.bytes(), GROUPS.3, "our walk's bytes");
    let held: Vec<(&str, usize)> = parted_by_field
        .iter()
        .map(|(f, v)| (f.as_str(), v.len()))
        .collect();
    assert_eq!(held, GROUPS_HELD_OUT, "the fields the line is held out on");
    if GROUPS_HELD_OUT.is_empty() {
        assert_eq!(ours.adler(), want, "the groups line's Adler");
    } else {
        assert_ne!(
            ours.adler(),
            want,
            "a held-out line cannot sum to the printed value"
        );
    }

    // **It can part.** One byte of each kind of range — a slot's record, a
    // member's entry, a `last_group` word — perturbed, parts on that range
    // alone.
    let mut kinds = std::collections::BTreeSet::new();
    for i in 0..ours.ranges.len() {
        if ours.ranges[i].bytes != theirs[i].2 || !kinds.insert(ours.ranges[i].field) {
            continue;
        }
        let mut bent = ours.clone();
        bent.ranges[i].bytes[0] ^= 0x01;
        let p: Vec<usize> = parting(&bent, &theirs).iter().map(|p| p.0).collect();
        let mut want_p: Vec<usize> = parted.iter().map(|p| p.0).collect();
        want_p.push(i);
        want_p.sort_unstable();
        assert_eq!(
            p, want_p,
            "a bent {} parts on range {i} alone",
            ours.ranges[i].field
        );
        assert_ne!(
            bent.adler(),
            want,
            "a bent {} moves the Adler",
            ours.ranges[i].field
        );
    }
    eprintln!("groups: {} kinds of range bent", kinds.len());
    assert_eq!(kinds.len(), 8, "every kind of range was bent");
    // The first image: slot 1's `role` back at 0 parts on that slot's
    // record alone.
    let scout = ours
        .ranges
        .iter()
        .position(|r| r.field == "id..march" && r.record == 1)
        .expect("slot 1");
    let mut bent = ours.clone();
    bent.ranges[scout].bytes[0x34 - 4..0x38 - 4].copy_from_slice(&[0; 4]);
    assert_eq!(
        parting(&bent, &theirs)
            .iter()
            .map(|p| p.0)
            .collect::<Vec<_>>(),
        vec![scout],
        "slot 1 without its role"
    );
    assert_eq!(parting(&Line::default(), &theirs).len(), GROUPS.2);
}

/// One leaf's verdict on the leaders line: `(slot, range field, leaf, ours,
/// theirs)`, the values as the leaf's elements, and whether the crate holds
/// it.
type LeafRow = (i16, &'static str, String, Option<Vec<i64>>, Vec<i64>, usize);

/// The leaders line, ours as leaves beside the stream's bytes decoded by the
/// same leaves' widths (signed at 4 bytes, unsigned below): every leaf of
/// every range, in walk order. A range whose length differs is one row
/// named `<field>: length` with both lengths.
fn leader_leaf_rows(sim: &sim::Sim, theirs: &StreamRows) -> Vec<LeafRow> {
    let ours: Vec<sim::checksum::leaders::LeaderRange> = (0..sim::checksum::leaders::LEADER_SLOTS)
        .flat_map(|s| sim.leader_image(s as sim::Player))
        .collect();
    let mut out = Vec::new();
    for (i, r) in ours.iter().enumerate() {
        let Some(t) = theirs.get(i) else {
            out.push((
                r.record,
                r.field,
                format!("{}: theirs -", r.field),
                None,
                vec![],
                0,
            ));
            continue;
        };
        let len: usize = r.leaves.iter().map(|l| l.width * l.count).sum();
        if len != t.2.len() {
            out.push((
                r.record,
                r.field,
                format!("{}: length", r.field),
                Some(vec![len as i64]),
                vec![t.2.len() as i64],
                0,
            ));
            continue;
        }
        let mut at = 0;
        for l in &r.leaves {
            let tv: Vec<i64> = (0..l.count)
                .map(|k| {
                    let b = &t.2[at + k * l.width..at + (k + 1) * l.width];
                    match l.width {
                        1 => i64::from(b[0]),
                        2 => i64::from(u16::from_le_bytes([b[0], b[1]])),
                        _ => i64::from(i32::from_le_bytes(b[..4].try_into().unwrap())),
                    }
                })
                .collect();
            let ov = l.values.as_ref().map(|v| {
                v.iter()
                    .map(|&x| match l.width {
                        1 => x & 0xff,
                        2 => x & 0xffff,
                        _ => i64::from(x as i32),
                    })
                    .collect()
            });
            out.push((r.record, r.field, l.name.clone(), ov, tv, l.width * l.count));
            at += l.width * l.count;
        }
    }
    if theirs.len() > ours.len() {
        for t in &theirs[ours.len()..] {
            out.push((-1, "-", format!("{}: ours -", t.1), None, vec![], 0));
        }
    }
    out
}

/// A leaf's values in short: one value, or its non-zero entries.
fn leaf_text(v: &[i64]) -> String {
    if v.len() == 1 {
        return v[0].to_string();
    }
    let nz: Vec<String> = v
        .iter()
        .enumerate()
        .filter(|(_, x)| **x != 0)
        .map(|(i, x)| format!("{i}:{x}"))
        .collect();
    if nz.is_empty() {
        format!("0x{}", v.len())
    } else {
        format!("{}/{}", nz.join(","), v.len())
    }
}

/// The pinned outcome of the leaders line on run 1562's packet
/// (`docs/CHECKSUM.md` §20): `(slots walked whole, ranges, bytes, ranges
/// parted, bytes in the ranges that agree)` — two leaders walked whole of
/// eight slots, 267 ranges and 58,343 bytes. Four ranges part: each walked
/// leader's long range `[+0x8, +0x692a)` (26,914 bytes, on the leaves
/// [`LEADERS_HELD_OUT`] names) and its `tech` mask's array (101 bytes, the
/// spell types' bits). Item 4121's first image agreed on 403 and 410 leaves.
const LEADERS: (usize, usize, usize, usize, usize) = (2, 267, 58343, 4, 4313);

/// **The leaves the leaders line is held out on**, each with its count of
/// walked leaders on run 1562's frame 91 (`docs/CHECKSUM.md` §20): a leaf
/// named bare parts, a leaf marked `(unheld)` is one this crate holds no
/// value for (emitted as zeros, and counted whether or not the original's
/// value is zero). `docs/CHECKSUM.md` §20 sorts each into its pile.
const LEADERS_HELD_OUT: &[(&str, usize)] = &[
    ("air_queued (unheld)", 2),
    ("ally_stamp (unheld)", 2),
    ("anti_att (unheld)", 2),
    ("att (unheld)", 2),
    ("attack_stamp (unheld)", 2),
    ("attrition_stamp (unheld)", 2),
    ("attrition_stamp2 (unheld)", 2),
    ("attrition_stamp3 (unheld)", 2),
    ("average_damage_rate (unheld)", 2),
    ("average_death_rate (unheld)", 2),
    ("average_hit_rate (unheld)", 2),
    ("average_kill_rate (unheld)", 2),
    ("barracks_queued (unheld)", 2),
    ("blacken (unheld)", 2),
    ("broke_alliance (unheld)", 2),
    ("buildings_lost (unheld)", 2),
    ("buildings_razed (unheld)", 2),
    ("cannon_time (unheld)", 2),
    ("city_name (unheld)", 2),
    ("combat_queued (unheld)", 2),
    ("counteroffer (unheld)", 2),
    ("ctw_hero_stamp (unheld)", 2),
    ("damage_current_frame (unheld)", 2),
    ("damage_fifteen_seconds (unheld)", 2),
    ("deaths_current_frame (unheld)", 2),
    ("deaths_fifteen_seconds (unheld)", 2),
    ("defeat_stamp (unheld)", 2),
    ("defeat_type (unheld)", 2),
    ("defeated_by (unheld)", 2),
    ("dock_queued (unheld)", 2),
    ("dow (unheld)", 2),
    ("explored", 2),
    ("factory_queued (unheld)", 2),
    ("fort_mark (unheld)", 2),
    ("forts_built (unheld)", 2),
    ("frame_battle (unheld)", 2),
    ("gift_stamp (unheld)", 2),
    ("give_att_disabled (unheld)", 2),
    ("good_deeds (unheld)", 2),
    ("handicap (unheld)", 2),
    ("high_buildings (unheld)", 2),
    ("hire_stamp (unheld)", 2),
    ("hire_who (unheld)", 2),
    ("hits_current_frame (unheld)", 2),
    ("hits_fifteen_seconds (unheld)", 2),
    ("incoming (unheld)", 2),
    ("incoming_taunt (unheld)", 2),
    ("incoming_taunt_frame (unheld)", 2),
    ("incoming_taunt_who (unheld)", 2),
    ("kills_current_frame (unheld)", 2),
    ("kills_fifteen_seconds (unheld)", 2),
    ("last_building_finished (unheld)", 2),
    ("last_spoke (unheld)", 2),
    ("last_taunt (unheld)", 2),
    ("last_unit_finished (unheld)", 2),
    ("lost_capital_modifier (unheld)", 2),
    ("lost_capital_stamp (unheld)", 2),
    ("lost_capital_timer (unheld)", 2),
    ("made_peace (unheld)", 2),
    ("misery (unheld)", 2),
    ("missiles_used (unheld)", 2),
    ("multi_diff (unheld)", 2),
    ("nukes_launched (unheld)", 2),
    ("oil_well_mark (unheld)", 2),
    ("pop_issues (unheld)", 2),
    ("raid_stamp (unheld)", 2),
    ("reg_terr (unheld)", 2),
    ("repair_stamp (unheld)", 2),
    ("scholar_militia (unheld)", 2),
    ("score (unheld)", 2),
    ("score_buildings (unheld)", 2),
    ("score_combat (unheld)", 2),
    ("score_economy (unheld)", 2),
    ("score_explored (unheld)", 2),
    ("score_pop (unheld)", 2),
    ("score_research (unheld)", 2),
    ("score_territory (unheld)", 2),
    ("score_unit_upgrades (unheld)", 2),
    ("score_units (unheld)", 2),
    ("score_units_2 (unheld)", 2),
    ("score_wonders (unheld)", 2),
    ("senates_built (unheld)", 2),
    ("stable_queued (unheld)", 2),
    ("supply_mark (unheld)", 2),
    ("taunt_frame (unheld)", 2),
    ("tech.ptr", 2),
    ("territory_high (unheld)", 2),
    ("tribute_demanded (unheld)", 2),
    ("tribute_received (unheld)", 2),
    ("tribute_sent (unheld)", 2),
    ("tribute_stamp (unheld)", 2),
    ("tributes (unheld)", 2),
    ("units_bribed (unheld)", 2),
    ("units_killed (unheld)", 2),
    ("units_lost (unheld)", 2),
    ("victory_type (unheld)", 2),
    ("wonders_built (unheld)", 2),
    ("wonders_held (unheld)", 2),
    ("wonderwin_stamp (unheld)", 2),
    ("wonderwin_timer (unheld)", 2),
];

/// The long range's leaves the test bends, one of each kind carried
/// (`docs/CHECKSUM.md` §20).
const LEADERS_BENT: &[&str] = &[
    "tribe",
    "units_built",
    "buildings_built",
    "cities_built",
    "city_mark",
    "chat_status",
    "team_color",
    "ctw_hero_retreat_stamp",
    "best_pop",
    "base_rate",
    "cities_lost",
    "nukes_used",
    "dock_mark",
    "reg_forts",
    "combat_units",
    "air_units",
    "aggression",
];

/// One leaf's held-out name: a long-range leaf bare, a tail's leaf under its
/// range, and `(unheld)` for one the crate holds no value for.
fn leader_held_name(field: &str, leaf: &str, unheld: bool) -> String {
    let name = if field == "who..ally_mask" {
        leaf.to_string()
    } else if field.ends_with(leaf) {
        field.to_string()
    } else {
        format!("{field}.{leaf}")
    };
    if unheld {
        format!("{name} (unheld)")
    } else {
        name
    }
}

/// The held-out leaves of a set of leaf rows, sorted, with their counts.
fn leader_held(rows: &[LeafRow]) -> Vec<(String, usize)> {
    let mut held: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for (_, field, leaf, ours, t, _) in rows {
        if ours.as_ref() == Some(t) {
            continue;
        }
        *held
            .entry(leader_held_name(field, leaf, ours.is_none()))
            .or_default() += 1;
    }
    held.into_iter().collect()
}

/// **The leaders line, ours against the original's, range by range and leaf
/// by leaf** (`docs/CHECKSUM.md` §20, item 4121): our `Sim::checksum_leaders`
/// against the instrument's stream of the leaders line on run 1562's packet,
/// every range in order, each decoded leaf by leaf on both sides by the same
/// leaves' widths, then the Adler against the printed `Leaders`. Held out on
/// [`LEADERS_HELD_OUT`], each leaf named with its count; the agreeing ranges
/// and bytes pinned in [`LEADERS`]. And it can part: one perturbed byte of
/// each kind of agreeing range parts on that range alone, a perturbed leaf
/// of the long range adds that leaf to the held-out list, and the empty
/// image parts on all.
#[test]
fn the_leaders_line_is_the_original_s_range_by_range() {
    let Some((theirs, want, sim)) = line_inputs("leaders", "Leaders") else {
        return;
    };
    let rows = leader_leaf_rows(&sim, &theirs);
    let mut counts: std::collections::BTreeMap<(i16, String, &str), (usize, usize)> =
        std::collections::BTreeMap::new();
    for (slot, field, leaf, ours, t, bytes) in &rows {
        let verdict = match ours {
            None => "unheld",
            Some(o) if o == t => "agree",
            Some(_) => "part",
        };
        let tail = field.split(['.', '[']).next().unwrap_or(field).to_string();
        let e = counts.entry((*slot, tail, verdict)).or_default();
        e.0 += 1;
        e.1 += bytes;
        eprintln!(
            "LEAF\t{slot}\t{field}\t{leaf}\t{verdict}\t{}\t{}",
            leaf_text(t),
            ours.as_deref().map_or("-".into(), leaf_text)
        );
    }
    for ((slot, tail, verdict), (n, b)) in &counts {
        eprintln!("COUNT\t{slot}\t{tail}\t{verdict}\t{n}\t{b}");
    }
    let ours = sim.checksum_leaders();
    let parted = parting(&ours, &theirs);
    let bytes_equal: usize = ours
        .ranges
        .iter()
        .zip(&theirs)
        .filter(|(o, t)| o.bytes == t.2)
        .map(|(o, _)| o.bytes.len())
        .sum();
    eprintln!(
        "leaders: ours {} ranges {} bytes adler {}; theirs {} ranges {} bytes; printed {want}; ranges parted {}, bytes in equal ranges {bytes_equal}",
        ours.ranges.len(),
        ours.bytes(),
        ours.adler(),
        theirs.len(),
        theirs.iter().map(|t| t.2.len()).sum::<usize>(),
        parted.len(),
    );
    let walked = ours
        .ranges
        .iter()
        .filter(|r| r.field == "who..ally_mask")
        .count();
    assert_eq!(
        (
            walked,
            theirs.len(),
            theirs.iter().map(|t| t.2.len()).sum::<usize>(),
            parted.len(),
            bytes_equal
        ),
        LEADERS,
        "the leaders line: leaders walked, the stream's ranges and bytes, ranges parted, bytes agreeing"
    );
    assert_eq!(ours.ranges.len(), LEADERS.1, "our walk's ranges");
    assert_eq!(ours.bytes(), LEADERS.2, "our walk's bytes");
    let held = leader_held(&rows);
    let held: Vec<(&str, usize)> = held.iter().map(|(f, n)| (f.as_str(), *n)).collect();
    assert_eq!(held, LEADERS_HELD_OUT, "the leaves the line is held out on");
    if LEADERS_HELD_OUT.is_empty() {
        assert_eq!(ours.adler(), want, "the leaders line's Adler");
    } else {
        assert_ne!(
            ours.adler(),
            want,
            "a held-out line cannot sum to the printed value"
        );
    }

    // **It can part.** One byte of each kind of agreeing range — a flag
    // word, a diplomacy record, the personality, a mask, an array's
    // element, the string, a decrypted word — perturbed, parts on that range
    // alone and moves the Adler.
    let mut kinds = std::collections::BTreeSet::new();
    for i in 0..ours.ranges.len() {
        if ours.ranges[i].bytes != theirs[i].2 || !kinds.insert(ours.ranges[i].field) {
            continue;
        }
        let mut bent = ours.clone();
        bent.ranges[i].bytes[0] ^= 0x01;
        let p: Vec<usize> = parting(&bent, &theirs).iter().map(|p| p.0).collect();
        let mut want_p: Vec<usize> = parted.iter().map(|p| p.0).collect();
        want_p.push(i);
        want_p.sort_unstable();
        assert_eq!(
            p, want_p,
            "a bent {} parts on range {i} alone",
            ours.ranges[i].field
        );
        assert_ne!(
            bent.adler(),
            ours.adler(),
            "a bent {} moves the Adler",
            ours.ranges[i].field
        );
    }
    eprintln!("leaders: {} kinds of range bent", kinds.len());
    for k in [
        "flags",
        "dip",
        "pers",
        "rare.ptr",
        "sites[k]",
        "make_list[k]",
        "encrypt",
        "prod_script",
    ] {
        assert!(kinds.contains(k), "a {k} range was bent");
    }
    // A leaf of the long range bent on leader 0 joins the list: `tribe`
    // (item 4121), and one of each kind item 4124 carried — a counter
    // (`units_built`), a city slot mark (`city_mark`), a setup rule
    // (`chat_status`, `team_color`).
    for leaf in LEADERS_BENT {
        let mut bent_rows = rows.clone();
        let row = bent_rows
            .iter_mut()
            .find(|r| r.0 == 0 && r.2 == *leaf)
            .unwrap_or_else(|| panic!("leader 0's {leaf}"));
        let v = row
            .3
            .as_mut()
            .unwrap_or_else(|| panic!("{leaf} is carried"));
        v[0] ^= 1;
        let bent_held = leader_held(&bent_rows);
        assert!(
            bent_held.iter().any(|(f, n)| f == leaf && *n == 1),
            "a bent {leaf} joins the held-out leaves"
        );
        assert_eq!(bent_held.len(), LEADERS_HELD_OUT.len() + 1, "{leaf}");
    }
    assert_eq!(parting(&Line::default(), &theirs).len(), LEADERS.1);
}
