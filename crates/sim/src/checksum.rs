//! `checksum/` — the original's sync checksum as byte images of our state,
//! traced as `ChecksumSync` (`docs/CHECKSUM.md` §11; `docs/ORACLE.md`'s
//! category 31).
//!
//! `CheckSums::check_all@00936560` folds fifteen walks of the original's
//! memory through Adler-32, one line each (`docs/CHECKSUM.md` §1). A line
//! of ours is the same walk over this crate's state: the ranges, in the
//! original's order and at its widths, holding the bytes the original's
//! memory holds there. The image is the unit of comparison, range by range
//! against the instrument's stream (`tools/recomp/checksum.py`); the Adler
//! is read last, and only says whether a whole line agrees.
//!
//! Ported so far: **items** (item 4088), **cities** and **goods** (item
//! 4091), **builds** ([`builds`], items 4094 to 4098, `docs/CHECKSUM.md`
//! §14, §15), **guys** ([`guys`], item 4104, §16), **units** ([`units`],
//! items 4109, 4113, §17, §18) and **groups** ([`groups`], item 4116,
//! §19). Every other line is
//! its own item, cheapest
//! first by `docs/CHECKSUM.md` §4's ledger.

use crate::Sim;

/// The `SyncDefine` category the original files these lines under
/// (`docs/ORACLE.md`, number 31, `Chksm`).
pub const TRACE: &str = "ChecksumSync";

/// The mask a `Coord` is stored under (`docs/FORMATS.md`, "Coordinates are
/// XOR-masked"): `SubObjectData`'s `z_internal`, `x_internal` and
/// `y_internal` hold `value ^ 0x63637`, and the walk folds the masked word.
pub const COORD_MASK: i32 = 0x63637;

/// `BASE_ITEMTYPES`, the goody box's `TypeIndex` (`docs/TECH.md`, "The type
/// space"): the 4-byte local `SubObject::walk_data@006621d0` folds for an
/// item is its type's `TypeIndex` (`ptype +4`), 543 on every item of both
/// frames read (`docs/CHECKSUM.md` §11).
pub const BASE_ITEMTYPES: i32 = 0x21f;

/// `SubObjectData::who` of an object no player owns, as the dump prints an
/// item's (`who 255`).
pub const NO_OWNER: u8 = 0xff;

/// One range of a line, as the fold receives it: the record it belongs to
/// (the object's `o`), the field or local it is, and its bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Range {
    pub record: i16,
    pub field: &'static str,
    pub bytes: Vec<u8>,
}

/// One line's byte image, in walk order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Line {
    pub name: &'static str,
    pub ranges: Vec<Range>,
}

impl Line {
    /// The bytes the line hands over.
    pub fn bytes(&self) -> usize {
        self.ranges.iter().map(|r| r.bytes.len()).sum()
    }

    /// The line's sum as `check_all` logs it: Adler-32 seeded with 1 over
    /// every range in order.
    pub fn adler(&self) -> u32 {
        self.ranges.iter().fold(1, |a, r| adler32(a, &r.bytes))
    }
}

/// zlib's Adler-32 (`adler32@00a46830`, the fold's; modulus 65521), the sum
/// carried in `seed`.
pub fn adler32(seed: u32, bytes: &[u8]) -> u32 {
    const MOD: u32 = 65521;
    let (mut a, mut b) = (seed & 0xffff, seed >> 16);
    for &x in bytes {
        a = (a + u32::from(x)) % MOD;
        b = (b + a) % MOD;
    }
    (b << 16) | a
}

/// A `Coord` as the walk folds it: the value under [`COORD_MASK`], little
/// endian.
fn coord(v: i32) -> [u8; 4] {
    (v ^ COORD_MASK).to_le_bytes()
}

/// **An item's `z_internal`**: the last writer is
/// `Terrain::refresh_good_z@00850b60`, which runs at the map's close and
/// after each building's terraform and sets every live good's and item's
/// `z` to `TerrainOut::find_data_z@00866560` at its point with the fourth
/// argument **1** (`pushl $0x1` at `00850bfa`): [`crate::World::data_z`],
/// except that a point on an ocean tile answers 0 and a negative height
/// answers 0. `SubObject::init@00662300`'s `find_tcoord_z` is overwritten
/// by it, so [`crate::World::object_z`] — the tile's height — is not the
/// item's (`docs/CHECKSUM.md` §11: it parted on 20 of run 1562's 22).
fn refreshed_z(world: &crate::World, at: crate::world::Pos) -> i32 {
    use crate::world::tile::{SURFACE, SURFACE_OCEAN};
    let t = at.tile();
    if world.tile_in_bounds(t) && world.tile_mask(t) & SURFACE == SURFACE_OCEAN {
        return 0;
    }
    world.data_z(at.x, at.y).0.max(0)
}

impl Sim {
    /// **The items line** (`CheckSums::check_items@00937790`,
    /// `docs/CHECKSUM.md` §8 and §11): per live goody box, in the items
    /// array's order, five ranges and 22 bytes —
    ///
    /// 1. `ItemData::ever_seen` (`+0x20`), the leaders' bits;
    /// 2. `SubObjectData::flags` (`+0x8`): only a box with `flags & 1` is
    ///    walked, and this crate holds no other bit of it, so `1`;
    /// 3. `SubObject::must_walk@006623a0`'s byte: 1 for a box with a
    ///    `ptype` or `flags & 1`, so 1 for every walked one;
    /// 4. `who`..`y_internal` (`+0x9..+0x18`): [`NO_OWNER`], `o` (the box's
    ///    index), then `z`, `x`, `y` as masked `Coord`s — `z` the ground
    ///    at the box's point as `refresh_good_z` last wrote it
    ///    ([`refreshed_z`]);
    /// 5. the type local, [`BASE_ITEMTYPES`].
    ///
    /// A box the harness took from a dump keeps the dump's order, and the
    /// built start lays them in the original's (`rondata::diff::start`), so
    /// the index is `o`.
    pub fn checksum_items(&self) -> Line {
        let mut ranges = Vec::new();
        for (o, it) in self.gaia.goody_items.iter().enumerate() {
            if !it.alive {
                continue;
            }
            self.sub_object_record(&mut ranges, o as i16, it.ever_seen, it.pos, BASE_ITEMTYPES);
        }
        Line {
            name: "items",
            ranges,
        }
    }

    /// One live goody box's or good's five ranges (`docs/CHECKSUM.md` §11):
    /// the record's `ever_seen` (`+0x20`), `SubObjectData::flags` (1),
    /// `must_walk`'s byte (1), `who`..`y_internal` and the type local.
    fn sub_object_record(
        &self,
        ranges: &mut Vec<Range>,
        o: i16,
        ever_seen: u8,
        at: crate::world::Pos,
        ty: i32,
    ) {
        let mut sub = vec![NO_OWNER];
        sub.extend_from_slice(&o.to_le_bytes());
        sub.extend_from_slice(&coord(refreshed_z(&self.world, at)));
        sub.extend_from_slice(&coord(at.x));
        sub.extend_from_slice(&coord(at.y));
        let mut put = |field, bytes: Vec<u8>| {
            ranges.push(Range {
                record: o,
                field,
                bytes,
            })
        };
        put("ever_seen", vec![ever_seen]);
        put("flags", vec![1]);
        put("must_walk", vec![1]);
        put("who..y_internal", sub);
        put("type", ty.to_le_bytes().to_vec());
    }

    /// `GoodData::ever_seen` (`+0x20`) of good `gi` as this crate holds
    /// it: the bits [`Sim::mark_good_seen`] has set — `World::reveal_fog`'s
    /// rare arm (`:87`) and `World::compute_reg_territory`'s goods scan
    /// (`:651`), each right beside its `Leader::new_rare`. `Good::init`
    /// zeroes it. Of its other two writers, the census's step 9
    /// (`Leader::plan_strategy@006b9620:337`) sets a leader's bit on a good
    /// already on that leader's `new_rares` — which, outside an alliance,
    /// only the two arms above put there, each with the same bit — and
    /// `Animal::work@005d7330:53`'s herd arm, the bit of every live leader
    /// that `is_seen` a herd animal on its herd's good (`HerdData::good_o`),
    /// is **not modelled** (`docs/CHECKSUM.md` §13).
    pub fn good_ever_seen_byte(&self, gi: usize) -> u8 {
        self.territory.good_seen_bits.get(gi).copied().unwrap_or(0)
    }

    /// **The goods line** (`CheckSums::check_goods@00937710`,
    /// `docs/CHECKSUM.md` §8 and §13): the items line's record over the
    /// goods array — per live good, in [`crate::World::goods`]' order (the
    /// original's goods list, `o` its index), `ever_seen`
    /// ([`Sim::good_ever_seen_byte`]), `flags`, `must_walk`, `who`..`y_internal`
    /// with `z` as `refresh_good_z` last wrote it, and the type local — the
    /// good's own `TypeIndex`.
    pub fn checksum_goods(&self) -> Line {
        let mut ranges = Vec::new();
        for (o, g) in self.world.goods().iter().enumerate() {
            if !g.alive {
                continue;
            }
            self.sub_object_record(
                &mut ranges,
                o as i16,
                self.good_ever_seen_byte(o),
                g.pos,
                g.ty as i32,
            );
        }
        Line {
            name: "goods",
            ranges,
        }
    }
}

/// **`CityData`'s walked 108 bytes** (`+0x6..+0x72`, `city`..`ter`), as
/// the type record names them (`types.txt`, `CityData`, size 0xb8): `(name,
/// offset in the record, width)`. `docs/CHECKSUM.md` §12 has the table with
/// what ours holds for each.
pub const CITY_FIELDS: &[(&str, usize, usize)] = &[
    ("city", 0x6, 2),
    ("o", 0x8, 2),
    ("reg", 0xa, 2),
    ("x", 0xc, 4),
    ("y", 0x10, 4),
    ("attack_stamp", 0x14, 4),
    ("raid_stamp", 0x18, 4),
    ("reduce_stamp", 0x1c, 4),
    ("capture_stamp", 0x20, 4),
    ("assimilation_timer", 0x24, 4),
    ("capture_strength", 0x28, 4),
    ("traded_with[0]", 0x2c, 4),
    ("traded_with[1]", 0x30, 4),
    ("traded_with[2]", 0x34, 4),
    ("traded_with[3]", 0x38, 4),
    ("traded_with[4]", 0x3c, 4),
    ("traded_with[5]", 0x40, 4),
    ("traded_with[6]", 0x44, 4),
    ("traded_with[7]", 0x48, 4),
    ("scouted", 0x4c, 2),
    ("in_port", 0x4e, 2),
    ("peasant_dist", 0x50, 2),
    ("trade_val", 0x52, 2),
    ("conquest_node", 0x54, 2),
    ("granary", 0x56, 1),
    ("lumber_mill", 0x57, 1),
    ("smelter", 0x58, 1),
    ("refinery", 0x59, 1),
    ("free", 0x5a, 1),
    ("busy", 0x5b, 1),
    ("gatherers", 0x5c, 1),
    ("pop", 0x5d, 1),
    ("who", 0x5e, 1),
    ("race", 0x5f, 1),
    ("founder", 0x60, 1),
    ("plundered", 0x61, 1),
    ("ocean", 0x62, 1),
    ("land", 0x63, 1),
    ("filled", 0x64, 1),
    ("bordering", 0x65, 1),
    ("ocean_filled", 0x66, 1),
    ("dock_tile", 0x67, 1),
    ("was_capital_flags", 0x68, 1),
    ("space[0]", 0x69, 1),
    ("space[1]", 0x6a, 1),
    ("space[2]", 0x6b, 1),
    ("ter[0]", 0x6c, 1),
    ("ter[1]", 0x6d, 1),
    ("ter[2]", 0x6e, 1),
    ("ter[3]", 0x6f, 1),
    ("ter[4]", 0x70, 1),
    ("ter[5]", 0x71, 1),
];

/// Where `check_cities`' second range starts in the record: `city` (+0x6).
const CITY_BASE: usize = 0x6;

/// `CityData::conquest_node` (+0x54) as `City::init@00737050` writes it
/// (`0xffff`): the Conquer the World node, which v1 cuts.
const NO_CONQUEST_NODE: i16 = -1;

/// The `size` an `Array<CaravanLink>` reports with links in it: 10 on every
/// non-empty link array of run1100's frame 5408 and in every `CITY` record
/// of run 1562's dump. **A reading of two frames, not a rule**: the
/// array's growth is not read (`docs/CHECKSUM.md` §12).
const VANS_SIZE: i32 = 10;

impl Sim {
    /// `City::flags`' word (`CityData::city_flags`, +0x4) from the state this
    /// crate keeps for it (`docs/CITIES.md` §1.4): the city's own bits, the
    /// building marks `0x80`/`0x200`/`0x400`/`0x800` (an active Temple,
    /// Granary, Lumbermill, Market in the city — `Build::add_to_city`'s, read
    /// here by the building chain), and the owner's `near_border` mark
    /// (`0x1000`). Bit `0x20` is not held (none of the frames read sets it).
    fn city_flags_word(&self, c: usize) -> u16 {
        use crate::cities::build::Ident;
        let city = &self.cities.cities[c];
        let near_border = self
            .ai
            .ai
            .get(city.owner as usize)
            .and_then(|a| a.city_ai.get(c))
            .is_some_and(|a| a.near_border);
        let bits = [
            (0x1, city.alive),
            (0x2, city.no_heal),
            (0x4, city.attacking),
            (0x8, city.ever_attacked),
            (0x10, city.capital),
            (0x40, city.alarm),
            (0x80, self.city_has(c, Ident::Temple)),
            (0x100, city.unassimilated),
            (0x200, self.city_has(c, Ident::Granary)),
            (0x400, self.city_has(c, Ident::Lumbermill)),
            (0x800, self.city_has(c, Ident::Market)),
            (0x1000, near_border),
            (0x2000, city.no_muster),
            (0x4000, city.founding_capital),
            (0x8000, city.was_founding_capital),
        ];
        bits.iter()
            .filter(|(_, on)| *on)
            .fold(0u16, |w, (b, _)| w | b)
    }

    /// The 108 bytes `+0x6..+0x72` of one city's record, from this crate's
    /// state: the city record ([`crate::cities::city::City`]), the owner's
    /// copy of the AI's half ([`crate::ai::CityAi`]), the economy's enhancer
    /// bytes (`City::calc_gather`'s, [`Sim::city_holdings`]) and the
    /// world's region numbering. `index` is the city's place in its owner's
    /// list (`CityData::city`, `City::init`'s first argument).
    fn city_record_bytes(&self, c: usize, index: usize) -> Vec<u8> {
        let city = &self.cities.cities[c];
        let ai = self
            .ai
            .ai
            .get(city.owner as usize)
            .and_then(|a| a.city_ai.get(c))
            .copied()
            .unwrap_or_default();
        let levels = crate::economy::holdings::Levels::for_player(self, city.owner);
        let held = self.city_holdings(c, levels);
        let mut b = vec![0u8; 0x72 - CITY_BASE];
        let mut put = |at: usize, v: &[u8]| {
            b[at - CITY_BASE..at - CITY_BASE + v.len()].copy_from_slice(v);
        };
        let i16b = |v: i64| (v as i16).to_le_bytes();
        let i32b = |v: i64| (v as i32).to_le_bytes();
        put(0x6, &i16b(index as i64));
        put(0x8, &self.buildings[city.building].index.to_le_bytes());
        // `reg` in the original's numbering, which a loaded map keeps beside
        // ours (`World::region_number`); `-1` off the map.
        let reg = city
            .reg
            .map_or(-1, |r| i64::from(self.world.region_number(r)));
        put(0xa, &i16b(reg));
        // A city's `Coord`s are stored plain: `City::init` unmasks the
        // building's (`^ 0x63637`) before it writes them.
        put(0xc, &city.pos.x.to_le_bytes());
        put(0x10, &city.pos.y.to_le_bytes());
        put(0x14, &i32b(city.attack_stamp));
        put(0x18, &i32b(city.raid_stamp));
        put(0x1c, &i32b(city.reduce_stamp));
        put(0x20, &i32b(city.capture_stamp));
        put(0x24, &i32b(city.assimilation_timer));
        put(0x28, &city.capture_strength.to_le_bytes());
        for (k, w) in city.traded_with.iter().enumerate() {
            put(0x2c + 4 * k, &w.to_le_bytes());
        }
        // `scouted` (+0x4c): no writer in this crate; `City::init` zeroes it.
        put(0x4c, &0i16.to_le_bytes());
        put(0x4e, &i16b(i64::from(ai.in_port)));
        put(0x50, &i16b(i64::from(ai.peasant_dist)));
        put(0x52, &i16b(i64::from(city.trade_val)));
        put(0x54, &NO_CONQUEST_NODE.to_le_bytes());
        // `City::calc_gather@00737c60`'s four bytes; the fourth (oil) it
        // writes as zero.
        put(
            0x56,
            &[
                held.granary as u8,
                held.lumber_mill as u8,
                held.smelter as u8,
                0,
            ],
        );
        put(
            0x5a,
            &[ai.free, ai.busy as u8, ai.gatherers, city.pop as u8],
        );
        put(
            0x5e,
            &[
                city.owner,
                city.race.map_or(0xff, |r| r),
                city.founder,
                // `plundered` (+0x61): `City::init` zeroes it; no writer here.
                0,
                ai.ocean as u8,
                ai.land as u8,
                ai.filled as u8,
                ai.bordering as u8,
                ai.ocean_filled as u8,
                ai.dock_tile as u8,
                city.was_capital as u8,
            ],
        );
        let space: Vec<u8> = ai.space.iter().map(|&v| v as u8).collect();
        put(0x69, &space);
        let ter: Vec<u8> = ai.ter.iter().map(|&v| v as u8).collect();
        put(0x6c, &ter);
        b
    }

    /// The caravan links ending at city `c`, `(cara, who)`, in the owner
    /// and slot order of [`crate::economy::caravan::Caravans`]. `cara` is
    /// taken as the caravan's slot in its owner's list and `who` as that
    /// owner: **an assumption**, since no link stands on the scored frame
    /// (`docs/CHECKSUM.md` §12).
    fn city_links(&self, c: usize) -> Vec<(i32, i32)> {
        let mut out = Vec::new();
        for (who, list) in self.economy.caravans.iter().enumerate() {
            for (k, v) in list.slots[..list.mark].iter().enumerate() {
                if v.alive && v.linked && (v.city_a == Some(c) || v.city_b == Some(c)) {
                    out.push((k as i32, who as i32));
                }
            }
        }
        out
    }

    /// **The cities line** (`CheckSums::check_cities@00937600`,
    /// `docs/CHECKSUM.md` §8 and §12): per leader in slot order, per live
    /// city in that leader's list —
    ///
    /// 1. `CityData::city_flags` (`+0x4`), 2 bytes ([`Sim::city_flags_word`]);
    /// 2. `city`..`ter` (`+0x6..+0x72`), 108 bytes ([`CITY_FIELDS`]);
    /// 3. the `vans` `Array<CaravanLink>` (`+0x74`) through
    ///    `Array<CaravanLink>::walk_data@00489040`: its length as a 4-byte
    ///    local, and with links its size (4, a local), `increment` (2),
    ///    the flags byte `& 0xbf` (1, a local) and 8 bytes a link.
    ///
    /// The two `String` names are walked only by an input walk, never here.
    pub fn checksum_cities(&self) -> Line {
        let mut ranges = Vec::new();
        let owners = self
            .cities
            .cities
            .iter()
            .filter(|c| c.alive)
            .map(|c| c.owner)
            .max();
        for who in 0..=owners.unwrap_or(0) {
            if owners.is_none() {
                break;
            }
            for (index, c) in self.cities_of(who).into_iter().enumerate() {
                let record = c as i16;
                let mut put = |field, bytes: Vec<u8>| {
                    ranges.push(Range {
                        record,
                        field,
                        bytes,
                    })
                };
                put("city_flags", self.city_flags_word(c).to_le_bytes().to_vec());
                put("city..ter", self.city_record_bytes(c, index));
                let links = self.city_links(c);
                put("vans.length", (links.len() as i32).to_le_bytes().to_vec());
                if !links.is_empty() {
                    put(
                        "vans.size",
                        VANS_SIZE.max(links.len() as i32).to_le_bytes().to_vec(),
                    );
                    put("vans.increment", (-1i16).to_le_bytes().to_vec());
                    put("vans.flags", vec![0]);
                    for (cara, w) in links {
                        let mut l = cara.to_le_bytes().to_vec();
                        l.extend_from_slice(&w.to_le_bytes());
                        put("vans[k]", l);
                    }
                }
            }
        }
        Line {
            name: "cities",
            ranges,
        }
    }
}

pub mod builds;
pub mod groups;
pub mod guys;
pub mod leaders;
pub mod units;
pub mod world;

#[cfg(test)]
mod tests;
