//! **The builds line** (`CheckSums::check_builds@00937290`,
//! `docs/CHECKSUM.md` §7 and §14): per active leader in slot order, per
//! live building in its object band in `o` order, `BuildData::walk_data
//! @0062f270`'s ranges — the founder and age bytes, the sub-object's, the
//! object's and the wall record's ranges with their locals, the build
//! record's 22 bytes, the queue, the mining list and the gather points'
//! count, then `orig_type`.
//!
//! Every field is named by its type record (`types.txt`: `SubObjectData`,
//! `ObjectData`, `WallData`, `BuildData`, `QueueItem`, `ArrayBaseMaster`);
//! the tables below are those names, offsets and widths, and
//! `rondata::diff::checksum_lines` decodes both sides by them.

use super::{COORD_MASK, Line, Range};
use crate::Sim;

/// **`ObjectData`'s walked 34 bytes** (`+0x20..+0x42`, `myhits`..
/// `launch_frames`; `Object::walk_data@00647830`): `(name, offset, width)`.
pub const OBJECT_FIELDS: &[(&str, usize, usize)] = &[
    ("myhits", 0x20, 4),
    ("damage", 0x24, 4),
    ("inside_down", 0x28, 2),
    ("up", 0x2a, 2),
    ("down", 0x2c, 2),
    ("down_who", 0x2e, 2),
    ("uid", 0x30, 2),
    ("hold_frames", 0x32, 2),
    ("near_o", 0x34, 2),
    ("near_who", 0x36, 2),
    ("healing", 0x38, 2),
    ("infiltrated", 0x3a, 1),
    ("damage_frac", 0x3b, 1),
    ("mylos", 0x3c, 1),
    ("targeted", 0x3d, 1),
    ("inside_down_who", 0x3e, 1),
    ("up_who", 0x3f, 1),
    ("visible", 0x40, 1),
    ("launch_frames", 0x41, 1),
];

/// **`WallData`'s walked 30 bytes** (`+0x48..+0x66`, `job_counter`..
/// `demolition`; `WallData::walk_data@00642510`).
pub const WALL_FIELDS: &[(&str, usize, usize)] = &[
    ("job_counter", 0x48, 4),
    ("job_counter_2", 0x4c, 4),
    ("constr_time", 0x50, 4),
    ("construct_hits", 0x54, 4),
    ("gpiece", 0x58, 4),
    ("frame_started", 0x5c, 4),
    ("build_masks", 0x60, 2),
    ("ever_seen", 0x62, 1),
    ("ever_seen_completed", 0x63, 1),
    ("helpers", 0x64, 1),
    ("demolition", 0x65, 1),
];

/// **`BuildData`'s walked 22 bytes** (`+0x70..+0x86`, `gather_down`..
/// `infiltrate2`). `+0x78` is `BuildData_u_120`, the union the dump prints
/// as `fort`, `dock` and `oil_well` (and a farm's record slot).
pub const BUILD_FIELDS: &[(&str, usize, usize)] = &[
    ("gather_down", 0x70, 2),
    ("city", 0x72, 2),
    ("city_down", 0x74, 2),
    ("wonder", 0x76, 2),
    ("u78", 0x78, 2),
    ("recharging", 0x7a, 2),
    ("attack_ox", 0x7c, 2),
    ("stance", 0x7e, 1),
    ("founder", 0x7f, 1),
    ("gather_max", 0x80, 1),
    ("attack_whom", 0x81, 1),
    ("queued", 0x82, 1),
    ("max_age", 0x83, 1),
    ("infiltrate", 0x84, 1),
    ("infiltrate2", 0x85, 1),
];

/// **A `QueueItem`'s walked 18 bytes** of its 20 (`BuildQueue::walk_data
/// @006305f0` folds `[item, item + 0x12)`).
pub const QUEUE_ITEM_FIELDS: &[(&str, usize, usize)] = &[
    ("job_counter", 0x0, 4),
    ("type", 0x4, 2),
    ("good[0]", 0x6, 2),
    ("good[1]", 0x8, 2),
    ("good[2]", 0xa, 2),
    ("cost[0]", 0xc, 2),
    ("cost[1]", 0xe, 2),
    ("cost[2]", 0x10, 2),
];

/// `BASE_BUILDTYPES`: a building type's `TypeIndex` is this plus its index
/// in the build-type table (`docs/TECH.md`, "The type space").
const BASE_BUILDTYPES: i32 = 0x19e;

/// `BASE_TECHTYPES`: the first age's `TypeIndex`, where the techs begin.
const BASE_TECHTYPES: i32 = 0x220;

/// `DISBAND`'s `TypeIndex`, a queue entry's `type` for a disband job.
const DISBAND_TYPE: i16 = 0x29a;

/// The `size` an empty `MiningList` reports (`size 5` on every building
/// record of run 1562's frame-91 dump whose list is empty), and the base its
/// growth doubles from: `ArrayBaseSimpleCopy<WCoordData>::increase_size
/// @004225c0` with the list's `increment` −1 adds the current size.
const MINING_SIZE_EMPTY: i32 = 5;

/// A `Range`-building closure's bytes, little endian, by width.
fn put(b: &mut [u8], base: usize, at: usize, width: usize, v: i64) {
    let bytes = v.to_le_bytes();
    b[at - base..at - base + width].copy_from_slice(&bytes[..width]);
}

impl Sim {
    /// `(o, who)` of a combat object, as the original's `_o`/`_who` pairs
    /// hold it.
    fn obj_o_who(&self, o: crate::combat::Obj) -> (i64, i64) {
        match o {
            crate::combat::Obj::Unit(u) => self
                .units
                .get(u)
                .map_or((-1, -1), |x| (i64::from(x.index), i64::from(x.owner))),
            crate::combat::Obj::Building(b) => self
                .buildings
                .get(b)
                .map_or((-1, -1), |x| (i64::from(x.index), i64::from(x.owner))),
        }
    }

    /// `SubObjectData::flags` of a building: `0x1` in use, `0x2` started
    /// (`Wall::start`), `0x4` active, `0x20` a city's type
    /// (`SubObject::init` ORs it in when the type's vtable `+100`,
    /// `BuildTypeData::is_city`, answers), `0x80` attacking.
    fn build_flags_byte(&self, b: usize) -> u8 {
        let bd = &self.buildings[b];
        let city = bd
            .ty
            .is_some_and(|t| crate::cities::build::is_city(&self.tables.build_types, t));
        [
            (0x1, bd.alive),
            (0x2, bd.started),
            (0x4, bd.active),
            (0x20, city),
            (0x80, bd.attacking),
        ]
        .iter()
        .filter(|(_, on)| *on)
        .fold(0u8, |w, (bit, _)| w | bit)
    }

    /// `WallData::build_masks` of a building, from the bits this crate
    /// keeps (`docs/CHECKSUM.md` §14).
    fn build_masks_word(&self, b: usize) -> u16 {
        let bd = &self.buildings[b];
        [
            (0x10, bd.under_attack & 0x1 != 0),
            (0x20, bd.under_attack & 0x2 != 0),
            (0x40, bd.queue.infinite),
            (0x88, bd.repeat_air),
            (0x100, bd.regen_roads),
            (0x400, bd.had_helpers),
            (0x800, bd.gather_bumped),
            (0x1000, bd.activated),
            (0x4000, bd.eject_pending),
        ]
        .iter()
        .filter(|(_, on)| *on)
        .fold(0u16, |w, (bit, _)| w | bit)
    }

    /// The building's 34 `ObjectData` bytes ([`OBJECT_FIELDS`]).
    fn object_bytes(&self, b: usize) -> Vec<u8> {
        let bd = &self.buildings[b];
        let mut v = vec![0u8; 0x42 - 0x20];
        let mut p = |at, w, x: i64| put(&mut v, 0x20, at, w, x);
        p(0x20, 4, i64::from(bd.hits));
        p(0x24, 4, i64::from(bd.damage));
        let inside = bd.garrison.last().map(|&u| {
            (
                i64::from(self.units[u].index),
                i64::from(self.units[u].owner),
            )
        });
        p(0x28, 2, inside.map_or(-1, |x| x.0));
        // The world cell's object chain (`Object::add_to_world@0064d8c0`):
        // this crate threads units only, so the building stands below
        // every unit on its cell.
        let chain = self.cell_chain(bd.pos.cell());
        let up = chain.last().map(|&u| {
            (
                i64::from(self.units[u].index),
                i64::from(self.units[u].owner),
            )
        });
        p(0x2a, 2, up.map_or(-1, |x| x.0));
        p(0x2c, 2, -1);
        p(0x2e, 2, 0);
        p(0x30, 2, i64::from(bd.uid));
        p(0x32, 2, i64::from(bd.hold_frames));
        let near = bd.near.map_or((-1, -1), |o| self.obj_o_who(o));
        p(0x34, 2, near.0);
        p(0x36, 2, near.1);
        p(0x38, 2, i64::from(bd.healing));
        p(0x3a, 1, i64::from(bd.infiltrated));
        p(0x3b, 1, i64::from(bd.damage_frac));
        p(0x3c, 1, i64::from(bd.mylos));
        p(0x3d, 1, i64::from(bd.targeted));
        p(0x3e, 1, inside.map_or(i64::from(bd.owner), |x| x.1));
        p(0x3f, 1, up.map_or(-1, |x| x.1));
        p(0x40, 1, i64::from(bd.visible));
        p(0x41, 1, i64::from(bd.launch_frames));
        v
    }

    /// The building's 30 `WallData` bytes ([`WALL_FIELDS`]); `gpiece` is the
    /// renderer's piece, handed in.
    fn wall_bytes(&self, b: usize, gpiece: i32) -> Vec<u8> {
        let bd = &self.buildings[b];
        let mut v = vec![0u8; 0x66 - 0x48];
        let mut p = |at, w, x: i64| put(&mut v, 0x48, at, w, x);
        p(0x48, 4, i64::from(bd.job_counter));
        p(0x4c, 4, i64::from(bd.job_counter_2));
        p(0x50, 4, i64::from(bd.constr_time));
        p(0x54, 4, i64::from(bd.construct_hits));
        p(0x58, 4, i64::from(gpiece));
        p(0x5c, 4, i64::from(bd.frame_started));
        p(0x60, 2, i64::from(self.build_masks_word(b)));
        p(0x62, 1, i64::from(bd.ever_seen));
        p(0x63, 1, i64::from(bd.ever_seen_completed));
        p(0x64, 1, i64::from(bd.helpers));
        // `demolition` (+0x65): `Wall::init` zeroes it; no writer here.
        p(0x65, 1, 0);
        v
    }

    /// The building's 22 `BuildData` bytes ([`BUILD_FIELDS`]).
    fn build_bytes(&self, b: usize) -> Vec<u8> {
        let bd = &self.buildings[b];
        let mut v = vec![0u8; 0x86 - 0x70];
        let mut p = |at, w, x: i64| put(&mut v, 0x70, at, w, x);
        let gather_down = bd
            .gatherers
            .first()
            .map_or(-1, |&u| i64::from(self.units[u].index));
        p(0x70, 2, gather_down);
        let (city, city_down) = match bd.city {
            Some(c) => {
                let index = self
                    .cities_of(self.cities.cities[c].owner)
                    .iter()
                    .position(|&x| x == c)
                    .map_or(-1, |i| i as i64);
                let chain = self.city_chain(c);
                let next = chain
                    .iter()
                    .position(|&x| x == b)
                    .and_then(|i| chain.get(i + 1))
                    .map_or(-1, |&n| i64::from(self.buildings[n].index));
                (index, next)
            }
            None => (-1, -1),
        };
        p(0x72, 2, city);
        p(0x74, 2, city_down);
        p(0x76, 2, -1);
        let u78 = self
            .cities
            .farm_order
            .iter()
            .position(|&x| x == b)
            .map(|i| i as i64)
            .or(bd.dock_slot.map(|d| d as i64))
            .unwrap_or(-1);
        p(0x78, 2, u78);
        p(0x7a, 2, i64::from(bd.recharging));
        let target = bd.target.map_or((-1, -1), |o| self.obj_o_who(o));
        p(0x7c, 2, target.0);
        p(0x7e, 1, i64::from(self.build_stance(b)));
        p(0x7f, 1, i64::from(bd.founder));
        p(0x80, 1, i64::from(bd.gather_max.unwrap_or(0)));
        p(0x81, 1, target.1);
        p(0x82, 1, bd.queue.items.len() as i64);
        p(0x83, 1, self.build_max_age(b));
        p(0x84, 1, 0);
        p(0x85, 1, 0);
        v
    }

    /// `BuildData::max_age` (+0x83): `Build::init` writes the owner's age
    /// (`LeaderDataEncrypt +0xdc`) and `Leader::gain_tech` rewrites it on
    /// every building of the player, so it is the owner's ages owned.
    fn build_max_age(&self, b: usize) -> i64 {
        let owner = self.buildings[b].owner as usize;
        self.tech.get(owner).map_or(0, |p| i64::from(p.ages))
    }

    /// A technology's `TypeIndex` from its tree id: the techs run from
    /// `BASE_TECHTYPES` (0x220, the first age) in tree order, and the tree
    /// holds no entry for the goody box's 0x21f, so a tech's id is not its
    /// `TypeIndex` (`docs/CHECKSUM.md` §14: Library entries 551 and 565 on
    /// run 1562's frame 91, tree ids 550 and 564).
    fn tech_type_index(&self, t: usize) -> i64 {
        let first = self
            .tables
            .tech_tree
            .types
            .iter()
            .position(|d| matches!(d.kind, crate::tech::Kind::Age(_)))
            .unwrap_or(0);
        i64::from(BASE_TECHTYPES) + t as i64 - first as i64
    }

    /// One `QueueItem`'s 18 walked bytes ([`QUEUE_ITEM_FIELDS`]).
    fn queue_item_bytes(&self, it: &crate::cities::production::Item) -> Vec<u8> {
        let ty: i64 = if it.disband {
            i64::from(DISBAND_TYPE)
        } else if let Some(t) = it.tech {
            self.tech_type_index(t)
        } else {
            self.tables
                .unit_types
                .get(it.ty)
                .map_or(-1, |t| i64::from(t.type_index))
        };
        let mut v = vec![0u8; 0x12];
        let mut p = |at, w, x: i64| put(&mut v, 0, at, w, x);
        p(0x0, 4, i64::from(it.job_counter));
        p(0x4, 2, ty);
        for k in 0..3 {
            p(
                0x6 + 2 * k,
                2,
                i64::from(it.good.get(k).copied().unwrap_or(0)),
            );
            p(
                0xc + 2 * k,
                2,
                i64::from(it.cost.get(k).copied().unwrap_or(0)),
            );
        }
        v
    }

    /// **The builds line** (`docs/CHECKSUM.md` §14): per leader in slot
    /// order, per live building in `o` order, `BuildData::walk_data`'s
    /// ranges. `gpiece(b)` is building `b`'s `WallData::gpiece`, the piece
    /// `GraphicPieces::get_build_gpiece@0090bce0` chose — art this crate
    /// does not load, so the data layer supplies it.
    pub fn checksum_builds(&self, gpiece: &dyn Fn(usize) -> i32) -> Line {
        let mut ranges = Vec::new();
        let owners = self
            .buildings
            .iter()
            .filter(|b| b.alive)
            .map(|b| b.owner)
            .max();
        let Some(last) = owners else {
            return Line {
                name: "builds",
                ranges,
            };
        };
        for who in 0..=last {
            let mut mine: Vec<usize> = (0..self.buildings.len())
                .filter(|&b| self.buildings[b].alive && self.buildings[b].owner == who)
                .collect();
            mine.sort_by_key(|&b| self.buildings[b].index);
            for b in mine {
                self.checksum_build_record(&mut ranges, b, gpiece(b));
            }
        }
        Line {
            name: "builds",
            ranges,
        }
    }

    /// One building's ranges, in `BuildData::walk_data@0062f270`'s order.
    fn checksum_build_record(&self, ranges: &mut Vec<Range>, b: usize, gpiece: i32) {
        let bd = &self.buildings[b];
        let record = bd.index;
        let mut put = |field, bytes: Vec<u8>| {
            ranges.push(Range {
                record,
                field,
                bytes,
            })
        };
        put("founder", vec![bd.founder]);
        put("max_age", vec![self.build_max_age(b) as u8]);
        // `SubObject::walk_data@006621d0`.
        put("flags", vec![self.build_flags_byte(b)]);
        put("must_walk", vec![1]);
        let mut sub = vec![bd.owner];
        sub.extend_from_slice(&bd.index.to_le_bytes());
        sub.extend_from_slice(&(bd.z ^ COORD_MASK).to_le_bytes());
        sub.extend_from_slice(&(bd.pos.x ^ COORD_MASK).to_le_bytes());
        sub.extend_from_slice(&(bd.pos.y ^ COORD_MASK).to_le_bytes());
        put("who..y_internal", sub);
        let ty = bd.ty.map_or(0, |t| BASE_BUILDTYPES + t as i32);
        put("type", ty.to_le_bytes().to_vec());
        // `Object::walk_data@00647830`.
        put("must_walk", vec![1]);
        put("myhits..launch_frames", self.object_bytes(b));
        put("launching", vec![u8::from(!bd.launching.is_empty())]);
        // `WallData::walk_data@00642510`.
        put("must_walk", vec![1]);
        put("job_counter..demolition", self.wall_bytes(b, gpiece));
        // `BuildData::walk_data@0062f270`'s own.
        put("must_walk", vec![1]);
        put("gather_down..infiltrate2", self.build_bytes(b));
        // `BuildQueue::walk_data@006305f0`: `queue_size`, then 18 bytes of
        // each of its slots. `BuildQueue::init@006307d0` zeroes the array
        // and writes slot 0's `type` −1; `set_queue` writes a slot.
        let size = bd.queue.capacity;
        put("queue_size", (size as i32).to_le_bytes().to_vec());
        for k in 0..size {
            let bytes = match bd.queue.items.get(k) {
                Some(it) => self.queue_item_bytes(it),
                None => {
                    let mut v = vec![0u8; 0x12];
                    if k == 0 {
                        v[4..6].copy_from_slice(&(-1i16).to_le_bytes());
                    }
                    v
                }
            };
            put("queue[k]", bytes);
        }
        // `MiningList::mtn`, `cliff` (+0xb4, +0xb5): −1 until a mine's list
        // is filled from a range or a cliff.
        put("mtn..cliff", vec![0xff, 0xff]);
        // `Array<TCoordData>::walk_data@00471c30`.
        let n = bd.gather_from.len() as i32;
        put("gather_from.length", n.to_le_bytes().to_vec());
        if n > 0 {
            let mut size = MINING_SIZE_EMPTY;
            while size < n {
                size *= 2;
            }
            put("gather_from.size", size.to_le_bytes().to_vec());
            put("gather_from.increment", (-1i16).to_le_bytes().to_vec());
            put("gather_from.flags", vec![0]);
            for t in &bd.gather_from {
                let mut v = t.x.to_le_bytes().to_vec();
                v.extend_from_slice(&t.y.to_le_bytes());
                put("gather_from[k]", v);
            }
        }
        // `PtrLinkListAbstract<GatherPoint>::walk_data@004708a0`.
        put(
            "gather.length",
            (bd.gather.len() as i32).to_le_bytes().to_vec(),
        );
        for g in &bd.gather {
            put("gather[k].type", 0i32.to_le_bytes().to_vec());
            put("gather[k].metric", vec![0]);
            let mut v = g.pos.x.to_le_bytes().to_vec();
            v.extend_from_slice(&g.pos.y.to_le_bytes());
            v.push(g.action);
            put("gather[k]", v);
        }
        let orig = bd
            .orig_ty
            .or(bd.ty)
            .map_or(0, |t| BASE_BUILDTYPES + t as i32);
        put("orig_type", orig.to_le_bytes().to_vec());
    }
}
