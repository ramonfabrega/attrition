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

/// Our walk of the minted game on the packet's frame.
fn ours_at_packet() -> Option<sim::Sim> {
    let (_, mut built, _) = built_start()?;
    let mut stream = crate::input::Stream::pump(&minted(&script())?);
    tick_to(&mut built, Some(&mut stream), CHECKSUM_PACKET.1);
    Some(built.sim.clone())
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
    // The score: the items line is the one state line ported, its bytes
    // the ones ours reproduces (`checksum::CHECKSUM_STATE`).
    let (lines, _, bytes, _) = crate::checksum::CHECKSUM_STATE;
    assert_eq!((lines, bytes), (1, ours.bytes() as u64), "CHECKSUM_STATE");
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
