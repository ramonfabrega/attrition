//! **A city's name on every packet** (item 4416, `docs/HUD.md` §3): each
//! live city's `CityData +0x90` off the packet (`tools/recomp/city_names.py`,
//! `<run>/names.tsv` beside the stream, outside git) against the name our
//! walk holds for the city of the same `(who, o)` on the packet's frame.
//!
//! The name is not a checksum line (no `walk_data` of a `String` is
//! summed), so this row stands beside the twelve rather than in them. It
//! is what the plate draws (`BuildOut::draw_overlays@0062cb30`), and what
//! a city reads before anyone renames it.
use super::{Walked, packets_dir};

/// One live city of a packet: the original's `(who, o, name)`, and ours
/// for the city building `(who, o)` on the same frame (`None` where we
/// hold no such city, or hold it without a name).
#[derive(Clone, Debug)]
pub(in crate::diff) struct NameRow {
    pub who: i64,
    pub o: i64,
    pub theirs: String,
    /// Whether our walk holds a live city at `(who, o)` at all.
    pub held: bool,
    pub ours: Option<String>,
}

impl NameRow {
    pub fn agrees(&self) -> bool {
        self.ours.as_deref() == Some(self.theirs.as_str())
    }
}

/// One live city as the names file holds it: `(who, o, name)`.
pub(in crate::diff) type Named = (i64, i64, String);

/// The packet's `names.tsv`: its logger frame and each live city's
/// `(who, o, name)`. `None` on a packet the tool has not read.
pub(in crate::diff) fn packet_names(run: &str) -> Option<(i64, Vec<Named>)> {
    let text = std::fs::read_to_string(format!("{}/{run}/names.tsv", packets_dir())).ok()?;
    let mut lines = text.lines();
    let frame = lines
        .next()?
        .strip_prefix("# packet logger frame ")?
        .trim()
        .parse()
        .ok()?;
    let rows = lines
        .filter(|l| !l.is_empty())
        .map(|l| {
            let f: Vec<&str> = l.splitn(5, '\t').collect();
            assert_eq!(f.len(), 5, "{run}: names.tsv row {l:?}");
            (
                f[0].parse().unwrap(),
                f[2].parse().unwrap(),
                f[4].to_string(),
            )
        })
        .collect();
    Some((frame, rows))
}

/// Ours against `theirs` on `sim`'s frame.
pub(in crate::diff) fn name_rows(sim: &sim::Sim, theirs: &[Named]) -> Vec<NameRow> {
    theirs
        .iter()
        .map(|(who, o, name)| {
            let city = sim.cities.cities.iter().position(|c| {
                c.alive
                    && i64::from(c.owner) == *who
                    && sim
                        .buildings
                        .get(c.building)
                        .is_some_and(|b| i64::from(b.index) == *o)
            });
            NameRow {
                who: *who,
                o: *o,
                theirs: name.clone(),
                held: city.is_some(),
                ours: city
                    .and_then(|c| sim.city_name_written(c))
                    .map(str::to_string),
            }
        })
        .collect()
}

/// `(agree, of)` on one walked packet, `None` where it has no names file.
pub(super) fn tally(w: &Walked) -> Option<(usize, usize)> {
    let rows = w.names.as_ref()?;
    Some((rows.iter().filter(|r| r.agrees()).count(), rows.len()))
}

/// The first row that parts, for the print and the assertion's message.
pub(super) fn first_parting(w: &Walked) -> Option<String> {
    let r = w.names.as_ref()?.iter().find(|r| !r.agrees())?;
    Some(format!(
        "who {} o {}: ours {:?}{} theirs {:?}",
        r.who,
        r.o,
        r.ours,
        if r.held { "" } else { " (no such city)" },
        r.theirs
    ))
}

/// **The packets whose city names part**, `(run, rows parting)`, pinned as
/// found (item 4416's first act, before any name was ported: every live
/// city of every packet, ours `None`). A packet not listed agrees on every
/// live city.
pub(in crate::diff) const NAMES_PARTING: &[(&str, usize)] = &[
    ("ch94p", 2),
    ("ch95pkt", 2),
    ("run1001d", 5),
    ("run1007b", 5),
    ("run1011b", 5),
    ("run1011c", 5),
    ("run1011d", 5),
    ("run1011e", 5),
    ("run1015b", 5),
    ("run1155c", 4),
    ("run1161", 2),
    ("run1164", 5),
    ("run1165", 5),
    ("run1166", 5),
    ("run1169", 4),
    ("run1185", 4),
    ("run1186b", 3),
    ("run1217", 9),
    ("run1220b", 4),
    ("run1221", 6),
    ("run1224b", 9),
    ("run1235b", 5),
    ("run1236", 7),
    ("run1237", 4),
    ("run1238c", 7),
    ("run1247", 6),
    ("run1254c", 4),
    ("run1263", 8),
    ("run1264b", 6),
    ("run1266c", 9),
    ("run1268", 5),
    ("run1293", 4),
    ("run1294", 4),
    ("run1296", 4),
    ("run1304", 5),
    ("run1305b", 5),
    ("run1320b", 7),
    ("run1323b", 4),
    ("run1324c", 4),
    ("run1325c", 7),
    ("run1326", 5),
    ("run1327", 9),
    ("run1332a", 5),
    ("run1351", 5),
    ("run1351c", 5),
    ("run1355", 4),
    ("run1357", 5),
    ("run1359", 7),
    ("run1359b", 7),
    ("run1359c", 7),
    ("run1361", 5),
    ("run1381", 4),
    ("run1382", 4),
    ("run1383", 2),
    ("run1385p", 4),
    ("run1391", 5),
    ("run1416", 5),
    ("run1417", 6),
    ("run1419b", 4),
    ("run1420", 6),
    ("run1424", 4),
    ("run1426b", 4),
    ("run1437", 5),
    ("run1438", 3),
    ("run1440", 3),
    ("run1441", 5),
    ("run1451", 3),
    ("run1457", 3),
    ("run1459", 3),
    ("run1460", 3),
    ("run1469", 3),
    ("run1469b", 3),
    ("run1472", 3),
    ("run1484", 3),
    ("run1487", 5),
    ("run1496", 5),
    ("run1559b", 5),
    ("run1562", 2),
    ("run1564", 5),
    ("run1570", 5),
    ("run785", 6),
    ("run789", 7),
    ("run795", 7),
    ("run810", 9),
    ("run823", 9),
    ("run825", 9),
    ("run839", 2),
    ("run842", 3),
    ("run842b", 3),
    ("run854", 6),
    ("run865", 9),
    ("run870", 2),
    ("run882b", 9),
    ("run883b", 9),
    ("run893b", 3),
    ("run912e", 4),
    ("run938b", 2),
    ("run943c", 4),
    ("run946", 4),
    ("run954b", 4),
    ("run958b", 2),
    ("run959", 3),
    ("run973g", 5),
    ("run984b", 3),
    ("run984c", 3),
    ("run985b", 6),
    ("run991d", 5),
];

/// **The row**: `(names agreeing, live cities)` over every pinned packet
/// with a names file — the original's `CityData +0x90` on each against
/// ours. **0 of 521** at item 4416's first act, on 107 packets: the
/// sim carried no founding name (`sim::cities::rename`'s SEAM).
pub(crate) const CITY_NAMES_SCORE: (usize, usize) = (0, 521);

/// [`CITY_NAMES_SCORE`] is the names files' rows less [`NAMES_PARTING`]'s,
/// over the pinned packets; every listed packet is a pinned one.
#[test]
fn the_city_names_score_is_the_table() {
    let mut of = 0;
    let mut read = 0;
    for (run, frame, _) in super::PINNED {
        let Some((logged, rows)) = packet_names(run) else {
            continue;
        };
        assert_eq!(logged, *frame, "{run}: names.tsv's logger frame");
        read += 1;
        of += rows.len();
    }
    if read == 0 {
        eprintln!(
            "skipping: no names.tsv under {} (tools/recomp/city_names.py)",
            packets_dir()
        );
        return;
    }
    for (run, _) in NAMES_PARTING {
        assert!(
            super::PINNED.iter().any(|p| p.0 == *run),
            "{run}: a parting packet that is not pinned"
        );
    }
    let parted: usize = NAMES_PARTING.iter().map(|p| p.1).sum();
    eprintln!("CITY NAMES: {read} packets read, {of} live cities, {parted} parting");
    assert_eq!((of - parted, of), CITY_NAMES_SCORE, "the city names' score");
}
