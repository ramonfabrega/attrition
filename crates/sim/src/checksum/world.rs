//! **The world line** (`World::walk_data@006b5cf0` with part −1,
//! `docs/CHECKSUM.md` §8 and §21): the walker reads `WorldData` (the
//! world at the global `0x00c06188`) section by section, and the record
//! prints each section's own sum, sections 0 to 14, before the line's.
//! Sections 0 and 14 walk nothing. In walk order:
//!
//! | section | ranges |
//! |---|---|
//! | 1 | `xs`, `ys` |
//! | 2 | `start_x`, `start_y`, `start_city_x`, `start_city_y`, each a `SimpleArray<WCoord>` |
//! | 3 | `oil_x`, `oil_y`, the same |
//! | 4 | `size`..`seed`, 30 ints |
//! | 5 | each `WData` cell, `[+0x0, +0x15)`, 16 fields ([`CELL_FIELDS`]) |
//! | 6 | `tdata`'s masks, then `seen`, `seen2`, `seen3` |
//! | 7 | `wcoord_seen` |
//! | 8 | `danger[0..8]` |
//! | 9 | per cell its collision block's presence local; for a cell with a block its `[+0x0, +0x8)` and `[+0xc, +0x6c)` |
//! | 10 | the terrain's `halfland_locs` (`Array<WCoordData>`) |
//! | 11–13 | `halfland_types`, `halfland_subtypes`, `nuke_hits` (`SimpleArray<int>`) |
//!
//! Each range is emitted as leaves ([`super::leaders::Leaf`]), a `None`
//! leaf being one this crate holds no value for, so a test can name the
//! field that holds the line out.

use super::leaders::Leaf;
use super::{Line, Range};
use crate::Sim;
use crate::world::{Cell, Owner, Pos};

/// One range of the world line: its section (the record's index), its
/// record (a cell's index, a leader's slot, else −1), its name and its
/// leaves.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldRange {
    pub section: u8,
    pub record: i16,
    pub field: &'static str,
    pub leaves: Vec<Leaf>,
}

impl WorldRange {
    /// The range's bytes; an unheld leaf's are zero.
    pub fn bytes(&self) -> Vec<u8> {
        self.leaves.iter().flat_map(Leaf::bytes).collect()
    }

    /// Whether the crate holds every leaf.
    pub fn held(&self) -> bool {
        self.leaves.iter().all(|l| l.values.is_some())
    }
}

/// `WData`'s 21 walked bytes as fields, `(name, width)` (`types.txt`,
/// `WData`, size 0x1c; the walk stops before `block` at +0x18).
pub const CELL_FIELDS: &[(&str, usize)] = &[
    ("flags", 2),
    ("land", 1),
    ("land_sub", 1),
    ("region", 2),
    ("region2", 2),
    ("down", 2),
    ("down_who", 2),
    ("val", 1),
    ("goods", 1),
    ("light", 1),
    ("who", 1),
    ("who2", 1),
    ("blocked", 1),
    ("bad", 1),
    ("solid", 1),
    ("was_seen", 1),
];

/// `WorldData` `+0x8..+0x80`, `size`..`seed` (`types.txt`, `WorldData`).
pub const HEADER_FIELDS: [&str; 30] = [
    "size",
    "fog_xs",
    "fog_ys",
    "fog_size",
    "tile_xs",
    "tile_ys",
    "tile_size",
    "reg_xs",
    "reg_ys",
    "reg_size",
    "map",
    "sea_map",
    "player_territory_limit",
    "player_territory_limit_civic",
    "player_territory_limit_city",
    "colonized_territory_limit",
    "colonized_territory_limit_civic",
    "colonized_territory_limit_city",
    "player_reg",
    "resource_reg",
    "forest_size",
    "mountain_size",
    "rock_size",
    "total_metal",
    "total_oil",
    "goodies",
    "land_resources",
    "sea_resources",
    "land_size",
    "seed",
];

/// A walked array's five range names: its count, size, increment and flags
/// locals, and its elements.
type ArrayNames = [&'static str; 5];

/// Sections 2 and 3's arrays, in walk order.
pub const START_ARRAYS: [ArrayNames; 4] = [
    [
        "start_x.num",
        "start_x.size",
        "start_x.increment",
        "start_x.flags",
        "start_x",
    ],
    [
        "start_y.num",
        "start_y.size",
        "start_y.increment",
        "start_y.flags",
        "start_y",
    ],
    [
        "start_city_x.num",
        "start_city_x.size",
        "start_city_x.increment",
        "start_city_x.flags",
        "start_city_x",
    ],
    [
        "start_city_y.num",
        "start_city_y.size",
        "start_city_y.increment",
        "start_city_y.flags",
        "start_city_y",
    ],
];
pub const OIL_ARRAYS: [ArrayNames; 2] = [
    [
        "oil_x.num",
        "oil_x.size",
        "oil_x.increment",
        "oil_x.flags",
        "oil_x",
    ],
    [
        "oil_y.num",
        "oil_y.size",
        "oil_y.increment",
        "oil_y.flags",
        "oil_y",
    ],
];
/// Sections 10 to 13's lists.
pub const TERRAIN_ARRAYS: [ArrayNames; 4] = [
    [
        "halfland_locs.num",
        "halfland_locs.size",
        "halfland_locs.increment",
        "halfland_locs.flags",
        "halfland_locs",
    ],
    [
        "halfland_types.num",
        "halfland_types.size",
        "halfland_types.increment",
        "halfland_types.flags",
        "halfland_types",
    ],
    [
        "halfland_subtypes.num",
        "halfland_subtypes.size",
        "halfland_subtypes.increment",
        "halfland_subtypes.flags",
        "halfland_subtypes",
    ],
    [
        "nuke_hits.num",
        "nuke_hits.size",
        "nuke_hits.increment",
        "nuke_hits.flags",
        "nuke_hits",
    ],
];

/// The fifteen sections the record prints a sum for (0..=14).
pub const WORLD_SECTIONS: usize = 15;

/// A walked list as the crate holds it: its elements (`width` bytes each,
/// `per` leaves an element) and its allocation, or `None` where it holds
/// no image of the list at all.
pub struct ListImage {
    pub elements: Vec<i64>,
    pub size: i64,
    pub increment: i64,
    pub flags: i64,
}

impl ListImage {
    /// An empty list: its count alone.
    pub const EMPTY: ListImage = ListImage {
        elements: Vec::new(),
        size: 0,
        increment: -1,
        flags: 0,
    };

    /// A list `TerrainData::TerrainData@0049bdd0` built with grow −1 and
    /// flags 0, holding `num` entries: `increase_size` at grow −1 takes 4,
    /// then the current size again, so `size` is the first `4·2^k ≥ num`
    /// (`crate::mapgen::terrain_init::HalfLand`).
    pub fn grown(elements: Vec<i64>, num: usize) -> ListImage {
        let mut size = 4;
        while size < num {
            size *= 2;
        }
        ListImage {
            elements,
            size: size as i64,
            increment: -1,
            flags: 0,
        }
    }
}

fn one(name: &'static str, width: usize, v: Option<i64>) -> Leaf {
    Leaf::new(name, width, 1, v.map(|v| vec![v]))
}

/// `SimpleArray<T>::walk_data`, `Array<T>::walk_data`: the count; with
/// elements, the size, the increment (2 bytes), the flags `& 0xbf` (1
/// byte), then the elements — as one range (`SimpleArray`) or one range an
/// element of `per` 4-byte words (`Array<WCoordData>`, `per` 2).
fn list(
    out: &mut Vec<WorldRange>,
    section: u8,
    names: ArrayNames,
    image: Option<ListImage>,
    per: Option<[&'static str; 2]>,
) {
    let put = |out: &mut Vec<WorldRange>, record: i16, field, leaves| {
        out.push(WorldRange {
            section,
            record,
            field,
            leaves,
        })
    };
    let Some(l) = image else {
        put(out, -1, names[0], vec![one("num", 4, None)]);
        return;
    };
    let width = per.map_or(1, |_| 2);
    let num = l.elements.len() / width;
    put(out, -1, names[0], vec![one("num", 4, Some(num as i64))]);
    if num == 0 {
        return;
    }
    put(out, -1, names[1], vec![one("size", 4, Some(l.size))]);
    put(
        out,
        -1,
        names[2],
        vec![one("increment", 2, Some(l.increment))],
    );
    put(
        out,
        -1,
        names[3],
        vec![one("flags", 1, Some(l.flags & 0xbf))],
    );
    match per {
        None => put(
            out,
            -1,
            names[4],
            vec![Leaf::new(names[4], 4, num, Some(l.elements))],
        ),
        Some([a, b]) => {
            for (k, e) in l.elements.chunks(2).enumerate() {
                put(
                    out,
                    k as i16,
                    names[4],
                    vec![one(a, 4, Some(e[0])), one(b, 4, Some(e[1]))],
                );
            }
        }
    }
}

fn owner_byte(o: Owner) -> i64 {
    match o {
        Owner::None => -1,
        Owner::Ambiguous => -2,
        Owner::Player(p) => i64::from(p),
    }
}

impl Sim {
    /// **The world line**, as bytes (`docs/CHECKSUM.md` §21): every range
    /// of [`Sim::world_image`] in order, an unheld leaf's bytes zero.
    pub fn checksum_world(&self) -> Line {
        Line {
            name: "world",
            ranges: self
                .world_image()
                .into_iter()
                .map(|r| Range {
                    record: r.record,
                    field: r.field,
                    bytes: r.bytes(),
                })
                .collect(),
        }
    }

    /// **The world line as leaves**, in walk order (module docs).
    pub fn world_image(&self) -> Vec<WorldRange> {
        let w = &self.world;
        let (xs, ys) = (w.width(), w.height());
        let mut out = Vec::new();
        // 1: `xs`, `ys`.
        out.push(WorldRange {
            section: 1,
            record: -1,
            field: "xs..ys",
            leaves: vec![
                one("xs", 4, Some(i64::from(xs))),
                one("ys", 4, Some(i64::from(ys))),
            ],
        });
        // 2: the start arrays, as the generator left them.
        let starts = w.record().starts.as_ref();
        for (k, names) in START_ARRAYS.into_iter().enumerate() {
            let image = starts.map(|a| ListImage {
                elements: a[k].list.iter().map(|&v| i64::from(v)).collect(),
                size: i64::from(a[k].size),
                increment: i64::from(a[k].increment),
                flags: i64::from(a[k].flags),
            });
            list(&mut out, 2, names, image, None);
        }
        // 3: `oil_x`, `oil_y` — `World::init` and `World::close` empty
        // them and nothing appends (the export and the listing both), so
        // each is its zero count.
        for names in OIL_ARRAYS {
            list(&mut out, 3, names, Some(ListImage::EMPTY), None);
        }
        // 4: `size`..`seed`.
        let header = self.world_header();
        out.push(WorldRange {
            section: 4,
            record: -1,
            field: "size..seed",
            leaves: HEADER_FIELDS
                .iter()
                .zip(header)
                .map(|(&n, v)| one(n, 4, v))
                .collect(),
        });
        // 5: the cells.
        for i in 0..xs * ys {
            let c = Cell::new(i % xs, i / xs);
            out.push(WorldRange {
                section: 5,
                record: i as i16,
                field: "cell",
                leaves: CELL_FIELDS
                    .iter()
                    .zip(self.world_cell(c))
                    .map(|(&(n, wd), v)| one(n, wd, v))
                    .collect(),
            });
        }
        // 6: the tile masks, then the three fog planes.
        let (tw, th) = (4 * xs, 4 * ys);
        let masks: Vec<i64> = (0..tw * th)
            .map(|i| i64::from(w.tile_mask(Pos::new(i % tw, i / tw))))
            .collect();
        out.push(WorldRange {
            section: 6,
            record: -1,
            field: "mask",
            leaves: vec![Leaf::new("mask", 2, masks.len(), Some(masks))],
        });
        let (fw, fh) = (2 * xs, 2 * ys);
        let fog = |f: &dyn Fn(i32, i32) -> Option<u8>| -> Option<Vec<i64>> {
            (0..fw * fh)
                .map(|i| f(i % fw, i / fw).map(i64::from))
                .collect()
        };
        let planes: [(&'static str, Option<Vec<i64>>); 3] = [
            ("seen", fog(&|x, y| w.seen(x, y))),
            ("seen2", fog(&|x, y| w.seen2(x, y))),
            ("seen3", fog(&|x, y| Some(w.seen3(x, y)))),
        ];
        for (name, v) in planes {
            out.push(WorldRange {
                section: 6,
                record: -1,
                field: name,
                leaves: vec![Leaf::new(name, 1, (fw * fh) as usize, v)],
            });
        }
        // 7: `wcoord_seen`.
        out.push(WorldRange {
            section: 7,
            record: -1,
            field: "wcoord_seen",
            leaves: vec![Leaf::new("wcoord_seen", 1, (xs * ys) as usize, None)],
        });
        // 8: the danger maps.
        let reg = ((xs / 2) * (ys / 2)) as usize;
        for who in 0..8u8 {
            let row = w.danger_row(who);
            // An unseated slot's row is `World::init`'s zeros:
            // `GameDaemon::calc_danger` clears and fills a row only for a
            // leader with `leader_flags & 2`, which an unseated slot never
            // holds (`docs/CHECKSUM.md` §20, `Leader::init`).
            let seated = usize::from(who) < self.territory.players.len();
            let v = if row.len() == reg {
                Some(row.iter().map(|&d| i64::from(d)).collect())
            } else if !seated {
                Some(vec![0; reg])
            } else {
                None
            };
            out.push(WorldRange {
                section: 8,
                record: i16::from(who),
                field: "danger",
                leaves: vec![Leaf::new("danger", 4, reg, v)],
            });
        }
        // 9: the collision blocks.
        for i in 0..xs * ys {
            out.push(WorldRange {
                section: 9,
                record: i as i16,
                field: "block.present",
                leaves: vec![one("present", 4, None)],
            });
        }
        // 10–12: `Terrain::init`'s half-land lists. 13: `nuke_hits`, its
        // zero count: `TerrainOut::close` empties it, `rebuild_nuke_craters`
        // (a restore's) reads it, and neither the export nor the listing
        // holds a store that appends to it.
        let half = w.record().halfland.as_ref();
        let locs = half.map(|h| {
            ListImage::grown(
                h.locs
                    .iter()
                    .flat_map(|&(x, y)| [i64::from(x), i64::from(y)])
                    .collect(),
                h.locs.len(),
            )
        });
        list(&mut out, 10, TERRAIN_ARRAYS[0], locs, Some(["x", "y"]));
        let ints = |v: &[i32]| ListImage::grown(v.iter().map(|&n| i64::from(n)).collect(), v.len());
        list(
            &mut out,
            11,
            TERRAIN_ARRAYS[1],
            half.map(|h| ints(&h.types)),
            None,
        );
        list(
            &mut out,
            12,
            TERRAIN_ARRAYS[2],
            half.map(|h| ints(&h.subtypes)),
            None,
        );
        list(
            &mut out,
            13,
            TERRAIN_ARRAYS[3],
            Some(ListImage::EMPTY),
            None,
        );
        out
    }

    /// `WorldData` `size`..`seed` as this crate holds it, `None` where it
    /// holds nothing.
    fn world_header(&self) -> [Option<i64>; 30] {
        let w = &self.world;
        let (xs, ys) = (i64::from(w.width()), i64::from(w.height()));
        let lim = w.territory_limits();
        let mut h = [None; 30];
        h[0] = Some(xs * ys);
        h[1] = Some(2 * xs);
        h[2] = Some(2 * ys);
        h[3] = Some(4 * xs * ys);
        h[4] = Some(4 * xs);
        h[5] = Some(4 * ys);
        h[6] = Some(16 * xs * ys);
        h[7] = Some(xs / 2);
        h[8] = Some(ys / 2);
        h[9] = Some((xs / 2) * (ys / 2));
        h[10] = w.record().map.map(i64::from);
        h[11] = Some(i64::from(w.sea_map()));
        if let Some(l) = lim {
            for k in 0..3 {
                h[12 + k] = Some(i64::from(l.player[k]));
                h[15 + k] = Some(i64::from(l.colonized[k]));
            }
        }
        // `World::analyze_map`'s ten, where the setup ran it (§21): the
        // cell tallies so far.
        if let Some(t) = w.record().tallies {
            for (k, v) in t.words().into_iter().enumerate() {
                if [2, 3, 4, 6, 7, 8, 9].contains(&k) {
                    h[18 + k] = Some(i64::from(v));
                }
            }
        }
        h[28] = Some(i64::from(self.land_size()));
        h[29] = w.record().seed.map(i64::from);
        h
    }

    /// One `WData`'s 16 walked fields as this crate holds them.
    fn world_cell(&self, c: Cell) -> [Option<i64>; 16] {
        let w = &self.world;
        let d = w.cell_data(c);
        let (down, down_who) = self.object_head_words(c);
        let num = |r: Option<u16>| r.map_or(-1, |r| i64::from(w.region_number(r)));
        [
            Some(i64::from(d.flags)),
            Some(i64::from(d.land)),
            Some(i64::from(d.land_sub)),
            Some(num(w.region_of(c))),
            Some(num(d.region2)),
            Some(i64::from(down)),
            Some(i64::from(down_who)),
            Some(i64::from(d.val)),
            Some(i64::from(d.goods)),
            // `light`: `WData::WData` and `World::wipe` write 0, and no
            // other store reaches `+0xe` of a cell (`docs/CHECKSUM.md` §21).
            Some(0),
            Some(owner_byte(w.owner(c))),
            Some(owner_byte(w.second(c))),
            Some(i64::from(d.blocked)),
            Some(i64::from(d.bad)),
            Some(i64::from(d.solid)),
            None,
        ]
    }
}
