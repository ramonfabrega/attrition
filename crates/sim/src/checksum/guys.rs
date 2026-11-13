//! **The guys line** (`CheckSums::check_guys@00937430`, `docs/CHECKSUM.md`
//! §6 and §16): per active leader in slot order, per live unit in its
//! object band in `o` order, the unit's figure array through
//! `PtrArray<Guy>::walk_data@0046df30` — its header and locals, one
//! presence byte a figure, the array's `size` and `increment` again, then
//! each figure's `GuyData::walk_data@005e0210` range `[+0x8, +0xa3)`,
//! `type`..`guy_num`.
//!
//! The record's names, offsets and widths are [`GUY_FIELDS`] (`types.txt`,
//! `GuyData`, size 0xbc); `rondata::diff::checksum_lines` decodes both
//! sides by them. A `GuyData` `Coord` is stored **plain**, not under
//! [`super::COORD_MASK`]: `Guy::set_new_location@005d86f0` writes the
//! value it is handed.

use super::{Line, Range};
use crate::Sim;
use crate::territory::attrition::Domain;

/// **`GuyData`'s walked 155 bytes** (`+0x8..+0xa3`, `type`..`guy_num`):
/// `(name, offset, width)`, 42 fields with the two turret arrays one field
/// each. The five at `+0x40..+0x54` are `f32` in the type record and are
/// folded as their bits (§16).
pub const GUY_FIELDS: &[(&str, usize, usize)] = &[
    ("type", 0x8, 4),
    ("x", 0xc, 4),
    ("y", 0x10, 4),
    ("z", 0x14, 4),
    ("angle", 0x18, 4),
    ("last_angle", 0x1c, 4),
    ("turret_angles", 0x20, 16),
    ("des_turret_angles", 0x30, 16),
    ("turret_inc", 0x40, 4),
    ("bank", 0x44, 4),
    ("last_bank", 0x48, 4),
    ("pitch", 0x4c, 4),
    ("last_pitch", 0x50, 4),
    ("track_dx", 0x54, 4),
    ("track_dy", 0x58, 4),
    ("des_x", 0x5c, 4),
    ("des_y", 0x60, 4),
    ("des_angle", 0x64, 4),
    ("last_x", 0x68, 4),
    ("last_y", 0x6c, 4),
    ("last_z", 0x70, 4),
    ("cur_time", 0x74, 4),
    ("end_time", 0x78, 4),
    ("last_time", 0x7c, 4),
    ("last_speed", 0x80, 4),
    ("avg_speed", 0x84, 4),
    ("gpiece", 0x88, 4),
    ("o", 0x8c, 2),
    ("ox", 0x8e, 2),
    ("variation", 0x90, 2),
    ("off_x", 0x92, 2),
    ("off_y", 0x94, 2),
    ("node_flags", 0x96, 2),
    ("des_node_flags", 0x98, 2),
    ("guy_flags", 0x9a, 2),
    ("cur_anim", 0x9c, 1),
    ("stopped", 0x9d, 1),
    ("hold_attack", 0x9e, 1),
    ("whom", 0x9f, 1),
    ("queued_attack", 0xa0, 1),
    ("who", 0xa1, 1),
    ("guy_num", 0xa2, 1),
];

/// Where the walked range starts in the record: `type` (`+0x8`).
const GUY_BASE: usize = 0x8;
/// Where it ends: one past `guy_num` (`+0xa3`).
const GUY_END: usize = 0xa3;

/// The gaia `TypeIndex`es whose pieces come three to a type
/// (`GraphicPieces::get_type@00902540`'s `0x192 + (piece − first) / 3`).
pub(crate) const GAIA_TYPES: std::ops::Range<i32> = 0x192..0x19e;

/// `first_bird_piece`, as `GraphicPieces::init_piece_ranges@008f70e0`
/// writes it: `0xea8d` (`rondata::artdata::FIRST_BIRD_PIECE`).
const FIRST_BIRD_PIECE: i32 = crate::units::anim::FIRST_BIRD_PIECE;

/// `ArrayBaseMaster::increment` (`+0xc` of the figure array) on every unit
/// of both frames read: 1.
const GUYS_INCREMENT: i16 = 1;

impl Sim {
    /// **The guys line** (`CheckSums::check_guys@00937430`): per leader in
    /// slot order, per live unit in `o` order, [`Sim::checksum_guy_array`].
    pub fn checksum_guys(&self) -> Line {
        let mut ranges = Vec::new();
        let last = self
            .units
            .iter()
            .filter(|u| u.alive())
            .map(|u| u.owner)
            .max();
        if let Some(last) = last {
            for who in 0..=last {
                let mut mine: Vec<usize> = (0..self.units.len())
                    .filter(|&u| self.units[u].alive() && self.units[u].owner == who)
                    .collect();
                mine.sort_by_key(|&u| self.units[u].index);
                for u in mine {
                    self.checksum_guy_array(&mut ranges, u);
                }
            }
        }
        Line {
            name: "guys",
            ranges,
        }
    }

    /// One unit's figure array, in `PtrArray<Guy>::walk_data@0046df30`'s
    /// order: `length` (a 4-byte local), `size` (a 4-byte local),
    /// `increment` (`+0xc`, 2), `flags & 0xbf` (a 1-byte local), one byte a
    /// figure (its pointer is not null), `size`..`increment` again
    /// (`+0x8..+0xe`, 6), then each figure's 155 bytes.
    pub(super) fn checksum_guy_array(&self, ranges: &mut Vec<Range>, u: usize) {
        let unit = &self.units[u];
        let record = unit.index;
        let n = unit.guys.len() as i32;
        let mut put = |field, bytes: Vec<u8>| {
            ranges.push(Range {
                record,
                field,
                bytes,
            })
        };
        put("guys.length", n.to_le_bytes().to_vec());
        if n == 0 {
            return;
        }
        // `size`, the allocation: the figure count on every unit of both
        // frames read (`docs/CHECKSUM.md` §16).
        let size = n;
        put("guys.size", size.to_le_bytes().to_vec());
        put("guys.increment", GUYS_INCREMENT.to_le_bytes().to_vec());
        put("guys.flags", vec![0]);
        for _ in 0..n {
            put("guys[k].present", vec![1]);
        }
        let mut si = size.to_le_bytes().to_vec();
        si.extend_from_slice(&GUYS_INCREMENT.to_le_bytes());
        put("guys.size..increment", si);
        for g in 0..unit.guys.len() {
            put("guy", self.guy_record_bytes(u, g));
        }
    }

    /// `GuyData::guy_flags` (`+0x9a`) of figure `g`, from what this crate
    /// keeps: the bits `Guy::init_real@005db6b0` sets from the type and the
    /// piece, and `Unit::set_in_danger`'s `0x20`.
    fn guy_flags_word(&self, u: usize, g: usize) -> u16 {
        let unit = &self.units[u];
        let ty = unit.type_index;
        let pivot = self
            .roster
            .art
            .pivots
            .get(&ty)
            .is_some_and(|nodes| !nodes.is_empty());
        let bits = [
            (0x8, self.guy_turns(u, g)),
            (0x10, unit.movement.turning.instant_from_stop),
            (
                0x20,
                unit.guy_flag_0x20 && g < crate::units::anim::SQUAD_SIZE,
            ),
            (0x40, self.is_plane(u)),
            (0x80, ty == 0x34 || ty == 0x35),
            (0x100, pivot),
        ];
        bits.iter()
            .filter(|(_, on)| *on)
            .fold(0u16, |w, (b, _)| w | b)
    }

    /// A figure's `z` (`+0x14`) at `at`, or its `last_z` (`+0x70`), as
    /// `Guy::set_new_location@005d86f0` writes it by the type's domain: an
    /// aircraft's guy 0 climbs ([`crate::air::Airframe`]); a ship's figure
    /// stands at 0; every other figure takes the ground under its point,
    /// `TerrainOut::find_data_z(x, y, 0)` ([`crate::World::data_z`]).
    /// `last_z` is the `z` `Guy::move`'s head copies (`005d9240:47`), the
    /// ground under the point it copies beside it.
    fn guy_z(&self, u: usize, g: usize, at: crate::world::Pos, last: bool) -> i32 {
        let unit = &self.units[u];
        match unit.kind.domain {
            Domain::Air if g == 0 => {
                if last {
                    unit.airframe.last_z
                } else {
                    unit.airframe.z
                }
            }
            Domain::Sea => 0,
            _ => self.world.data_z(at.x, at.y).0,
        }
    }

    /// A figure's `gpiece` (`+0x88`): the piece this crate keeps, and for a
    /// gaia type (`TypeIndex` `0x192..0x19e`, the birds and the herd
    /// animals) `GraphicPieces::get_unit_gpiece@0090c030`'s own gaia arm,
    /// `first_bird_piece + 3·(type − 0x192) + (seed + o) % 3`
    /// (`docs/ANIM.md` §3), its variant walked down to one the install
    /// ships ([`Sim::gaia_piece`], item 4208) where the install was read.
    fn guy_gpiece(&self, u: usize, g: usize) -> i32 {
        let ty = self.units[u].type_index;
        if let Some(p) = self.gaia_piece(ty, self.units[u].index) {
            return p;
        }
        if (GAIA_TYPES).contains(&ty) {
            return FIRST_BIRD_PIECE
                + 3 * (ty - GAIA_TYPES.start)
                + i32::from(self.gaia_variant(u));
        }
        self.units[u].guys[g].gpiece
    }

    /// Figure `g` of unit `u`'s 155 bytes `+0x8..+0xa3`.
    fn guy_record_bytes(&self, u: usize, g: usize) -> Vec<u8> {
        let unit = &self.units[u];
        let guy = unit.guys[g];
        let (body, facing, des, des_angle) = match guy.follow {
            Some(f) => (f.body, f.facing, f.des, f.des_angle),
            None if g == 0 => (
                unit.movement.body,
                unit.movement.facing,
                unit.pos,
                unit.movement.heading,
            ),
            None => (
                unit.movement.body,
                unit.movement.facing,
                unit.pos,
                unit.movement.facing,
            ),
        };
        let track = guy.follow.map_or((0, 0), |f| f.track);
        let (last, last_facing) = guy.last.unwrap_or((body.pos, facing));
        let z = self.guy_z(u, g, body.pos, false);
        let last_z = self.guy_z(u, g, last, true);
        let aim = guy.aim.map(|o| match o {
            crate::combat::Obj::Unit(v) => (self.units[v].index, self.units[v].owner as i8),
            crate::combat::Obj::Building(b) => {
                (self.buildings[b].index, self.buildings[b].owner as i8)
            }
        });
        let (af_bank, af_last_bank, af_pitch, af_last_pitch) = if g == 0 {
            let af = unit.airframe;
            (
                af.bank.bits(),
                af.last_bank.bits(),
                af.pitch.bits(),
                af.last_pitch.bits(),
            )
        } else {
            (0, 0, 0, 0)
        };
        let mut b = vec![0u8; GUY_END - GUY_BASE];
        let mut put = |at: usize, v: &[u8]| {
            b[at - GUY_BASE..at - GUY_BASE + v.len()].copy_from_slice(v);
        };
        put(0x8, &unit.type_index.to_le_bytes());
        put(0xc, &body.pos.x.to_le_bytes());
        put(0x10, &body.pos.y.to_le_bytes());
        put(0x14, &z.to_le_bytes());
        put(0x18, &facing.0.to_le_bytes());
        put(0x1c, &last_facing.0.to_le_bytes());
        for k in 0..4 {
            put(0x20 + 4 * k, &guy.turret.angles[k].to_le_bytes());
            put(0x30 + 4 * k, &guy.turret.des[k].to_le_bytes());
        }
        put(0x40, &0u32.to_le_bytes());
        put(0x44, &af_bank.to_le_bytes());
        put(0x48, &af_last_bank.to_le_bytes());
        put(0x4c, &af_pitch.to_le_bytes());
        put(0x50, &af_last_pitch.to_le_bytes());
        put(0x54, &track.0.to_le_bytes());
        put(0x58, &track.1.to_le_bytes());
        put(0x5c, &des.x.to_le_bytes());
        put(0x60, &des.y.to_le_bytes());
        put(0x64, &des_angle.0.to_le_bytes());
        put(0x68, &last.x.to_le_bytes());
        put(0x6c, &last.y.to_le_bytes());
        put(0x70, &last_z.to_le_bytes());
        put(0x74, &guy.cur_time.to_le_bytes());
        put(0x78, &guy.end_time.to_le_bytes());
        put(0x7c, &guy.last_time.to_le_bytes());
        put(0x80, &body.last_speed.to_le_bytes());
        put(0x84, &body.avg_speed.to_le_bytes());
        put(0x88, &self.guy_gpiece(u, g).to_le_bytes());
        put(0x8c, &unit.index.to_le_bytes());
        put(0x8e, &aim.map_or(-1, |a| a.0).to_le_bytes());
        put(0x96, &guy.turret.node_flags.to_le_bytes());
        put(0x98, &guy.turret.des_flags.to_le_bytes());
        put(0x9a, &self.guy_flags_word(u, g).to_le_bytes());
        put(
            0x9c,
            &[
                guy.anim as u8,
                guy.stopped as u8,
                guy.pending_attack as u8,
                aim.map_or(-1, |a| a.1) as u8,
                guy.queued_attack as u8,
                unit.owner,
                g as u8,
            ],
        );
        b
    }
}
