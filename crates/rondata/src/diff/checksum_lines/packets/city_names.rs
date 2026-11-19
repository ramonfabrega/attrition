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
                ours: city.and_then(|c| sim.city_name(c)).map(str::to_string),
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

/// **The packets whose city names part**, `(run, rows parting)`. Item
/// 4416's first act pinned all 107 (ours `None` on every live city, 0 of
/// 521); item 4418's port of `City::generate_name@00735c90` empties it.
/// Its first walk, with the tribe files' `<CITY>` texts untrimmed, parted
/// six (`run1011b` to `e`, `run1015b`, `run1323b`: ours `"Huaraz  "`,
/// theirs `"Huaraz"`). A packet not listed agrees on every live city.
pub(in crate::diff) const NAMES_PARTING: &[(&str, usize)] = &[];

/// **The row**: `(names agreeing, live cities)` over every pinned packet
/// with a names file — the original's `CityData +0x90` on each against
/// ours. **0 of 521** at item 4416's first act, on 107 packets: the
/// sim carried no founding name. **521 of 521** at item 4418 (+521), the
/// port (`sim::leaders::words::founding_name`).
pub(crate) const CITY_NAMES_SCORE: (usize, usize) = (521, 521);

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
