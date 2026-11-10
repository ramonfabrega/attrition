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
//! Ported so far: **items** (item 4088). Every other line is its own item,
//! cheapest first by `docs/CHECKSUM.md` §4's ledger.

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
            let o = o as i16;
            let mut sub = vec![NO_OWNER];
            sub.extend_from_slice(&o.to_le_bytes());
            sub.extend_from_slice(&coord(refreshed_z(&self.world, it.pos)));
            sub.extend_from_slice(&coord(it.pos.x));
            sub.extend_from_slice(&coord(it.pos.y));
            let mut put = |field, bytes: Vec<u8>| {
                ranges.push(Range {
                    record: o,
                    field,
                    bytes,
                })
            };
            put("ever_seen", vec![it.ever_seen]);
            put("flags", vec![1]);
            put("must_walk", vec![1]);
            put("who..y_internal", sub);
            put("type", BASE_ITEMTYPES.to_le_bytes().to_vec());
        }
        Line {
            name: "items",
            ranges,
        }
    }
}

#[cfg(test)]
mod tests;
