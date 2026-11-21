//! **The units line** (`CheckSums::check_units@009371d0`,
//! `docs/CHECKSUM.md` §6 and §17): per active leader in slot order, per
//! live unit in its object band in `o` order, `Unit::walk_data@0060cf40`
//! with the walk's flags at −1 — the sub-object's and the object's ranges
//! with their locals, the unit record `+0x48..+0xb7`, the path stack, the
//! order list, the figure array (the guys line's bytes, emitted by
//! [`Sim::checksum_guy_array`]) and, on an animal, its own three fields.
//!
//! The records' names, offsets and widths are [`UNIT_FIELDS`] and the
//! builds line's [`super::builds::OBJECT_FIELDS`] (`types.txt`, `UnitData`,
//! `ObjectData`) and the order classes' PDB field lists ([`MOVE_FIELDS`],
//! [`TARGET_FIELDS`], [`GATHER_FIELDS`]); `rondata::diff::checksum_lines`
//! decodes both sides by them. A `UnitData`, `PathData` or order `Coord`
//! is stored **plain**; a `SubObjectData`'s is masked ([`super::coord`]).

use super::{Line, Range};
use crate::Sim;
use crate::combat::Obj;
use crate::orders::Body;

/// **`UnitData`'s walked 111 bytes** (`+0x48..+0xb7`, `collide_frame`..
/// `play`): `(name, offset, width)`, 49 fields. The two unions are named by
/// their offset: `+0x54` (`rare`, `air_alt`, `former_type`) and `+0x86`
/// (`hero`, `cara`, `doober`, `special`, `herd`).
pub const UNIT_FIELDS: &[(&str, usize, usize)] = &[
    ("collide_frame", 0x48, 4),
    ("damage_frame", 0x4c, 4),
    ("angle", 0x50, 4),
    ("u54", 0x54, 4),
    ("dest_angle", 0x58, 4),
    ("trench_angle", 0x5c, 4),
    ("tolerance", 0x60, 4),
    ("queue_time", 0x64, 4),
    ("unit_masks", 0x68, 4),
    ("unit_masks2", 0x6c, 4),
    ("orders_x", 0x70, 4),
    ("orders_y", 0x74, 4),
    ("los_x", 0x78, 4),
    ("los_y", 0x7c, 4),
    ("group", 0x80, 2),
    ("inside_up", 0x82, 2),
    ("supply", 0x84, 2),
    ("u86", 0x86, 2),
    ("collide", 0x88, 2),
    ("collide_o", 0x8a, 2),
    ("collide_guy", 0x8c, 2),
    ("o_up", 0x8e, 2),
    ("o_down", 0x90, 2),
    ("gather_down", 0x92, 2),
    ("good_obj", 0x94, 2),
    ("mana_burn", 0x96, 2),
    ("spell_time", 0x98, 2),
    ("myspeed", 0x9a, 2),
    ("myarmor", 0x9c, 2),
    ("attrition", 0x9e, 2),
    ("num_queued", 0xa0, 2),
    ("cavarch_o", 0xa2, 2),
    ("damage_o", 0xa4, 2),
    ("cavarch_uid", 0xa6, 2),
    ("cavarch_who", 0xa8, 1),
    ("damage_who", 0xa9, 1),
    ("form", 0xaa, 1),
    ("form_mod", 0xab, 1),
    ("full", 0xac, 1),
    ("waiting", 0xad, 1),
    ("recharging", 0xae, 1),
    ("path_recursion", 0xaf, 1),
    ("idle", 0xb0, 1),
    ("stance", 0xb1, 1),
    ("safe", 0xb2, 1),
    ("collide_who", 0xb3, 1),
    ("inside_up_who", 0xb4, 1),
    ("guy_mark", 0xb5, 1),
    ("play", 0xb6, 1),
];

/// **`MoveOrder`'s walked 76 bytes** (`+0x4..+0x50`, `x`..`off_y`, the
/// PDB's field list; `AttackToOrder::walk_data@00482e70`, which a
/// `MOVE_TO`, `ATTACK_TO`, `EXPLORE_TO` and `FLEE_TO` share).
pub const MOVE_FIELDS: &[(&str, usize, usize)] = &[
    ("x", 0x4, 4),
    ("y", 0x8, 4),
    ("angle", 0xc, 4),
    ("dest", 0x10, 4),
    ("tolerance", 0x14, 4),
    ("pause", 0x18, 4),
    ("retry", 0x1c, 4),
    ("attempts", 0x20, 4),
    ("timer", 0x24, 4),
    ("facing", 0x28, 4),
    ("dest_x", 0x2c, 4),
    ("dest_y", 0x30, 4),
    ("last_x", 0x34, 4),
    ("last_y", 0x38, 4),
    ("coll_x", 0x3c, 4),
    ("coll_y", 0x40, 4),
    ("orig_x", 0x44, 4),
    ("orig_y", 0x48, 4),
    ("off_x", 0x4c, 2),
    ("off_y", 0x4e, 2),
];

/// **`TargetOrder`'s walked 10 bytes** (`+0x8..+0x12`, `ox`, `whom`, `uid`,
/// the PDB's field list; `TargetOrder::walk_data@0047f220` for a
/// `BUILD_AT`, and the head of `GatherOrder::walk_data@00486e60`).
pub const TARGET_FIELDS: &[(&str, usize, usize)] =
    &[("ox", 0x8, 4), ("whom", 0xc, 4), ("uid", 0x10, 2)];

/// `AttackOrder::walk_data@0047f260`'s final range, after flags and target.
pub const ATTACK_FIELDS: &[(&str, usize, usize)] = &[
    ("def_x", 0x14, 4),
    ("def_y", 0x18, 4),
    ("mandatory", 0x1c, 1),
    ("defensive", 0x1d, 1),
    ("in_range", 0x1e, 1),
    ("ever_in_range", 0x1f, 1),
    ("new_ord", 0x20, 1),
];

/// **`GatherOrder`'s own walked 20 bytes** (`+0x14..+0x28`, `tx`..
/// `been_there`).
pub const GATHER_FIELDS: &[(&str, usize, usize)] = &[
    ("tx", 0x14, 4),
    ("ty", 0x18, 4),
    ("build_type", 0x1c, 4),
    ("wait", 0x20, 4),
    ("goto_build", 0x24, 1),
    ("non_flat_gather", 0x25, 1),
    ("dist_mod", 0x26, 1),
    ("been_there", 0x27, 1),
];

/// **`TradeOrder`'s own walked 18 bytes** (`+0x14..+0x26`, `oxx`..`uid2`,
/// the PDB's field list; `TradeOrder::walk_data@00483260`, item 4230).
pub const TRADE_FIELDS: &[(&str, usize, usize)] = &[
    ("oxx", 0x14, 4),
    ("whose", 0x18, 4),
    ("started", 0x1c, 4),
    ("loaded", 0x20, 4),
    ("uid2", 0x24, 2),
];

/// **`CastOrder`'s own walked 16 bytes** (`+0x14..+0x24`, `x`..`spell`,
/// the PDB's field list; `CastOrder::walk_data@004860a0`, item 4234).
pub const CAST_FIELDS: &[(&str, usize, usize)] = &[
    ("x", 0x14, 4),
    ("y", 0x18, 4),
    ("paid", 0x1c, 4),
    ("spell", 0x20, 4),
];

/// **`AttackGroundOrder`'s walked 16 bytes** (`+0x4..+0x14`, `att_x`..
/// `attack_unit`, the PDB's field list), the first own range of
/// `AirAttackGroundOrder::walk_data@004875a0` (item 4459).
pub const ATTACK_GROUND_FIELDS: &[(&str, usize, usize)] = &[
    ("att_x", 0x4, 4),
    ("att_y", 0x8, 4),
    ("accuracy", 0xc, 4),
    ("attack_unit", 0x10, 4),
];

/// **`AirOrder`'s walked 24 bytes** (`+0x4..+0x1c` of the base, `oxx`..
/// `returning`, the PDB's field list), the second (item 4459).
pub const AIR_FIELDS: &[(&str, usize, usize)] = &[
    ("oxx", 0x4, 4),
    ("whose", 0x8, 4),
    ("cruising_alt", 0xc, 4),
    ("sharp_turn", 0x10, 4),
    ("old", 0x14, 4),
    ("returning", 0x18, 4),
];

/// **`AirAttackGroundOrder`'s own walked 12 bytes** (`+0x30..+0x3c`,
/// `total_time`, `sx`, `sy`, the PDB's field list), the third (item 4459).
pub const AIR_ATTACK_GROUND_FIELDS: &[(&str, usize, usize)] =
    &[("total_time", 0x30, 4), ("sx", 0x34, 4), ("sy", 0x38, 4)];

/// **`GuardOrder`'s own walked 24 bytes** (`+0x14..+0x2c`, `dx`..`retry`,
/// the PDB's field list; `GuardOrder::walk_data@00486550`, item 4266). The
/// post's two `Coord`s go over as stored, unmasked: run1597's record 62 on
/// 12537 hands over 42840, 18408, its post.
pub const GUARD_FIELDS: &[(&str, usize, usize)] = &[
    ("dx", 0x14, 4),
    ("dy", 0x18, 4),
    ("guard_x", 0x1c, 4),
    ("guard_y", 0x20, 4),
    ("idle", 0x24, 4),
    ("retry", 0x28, 4),
];

/// **`GroupOrder`'s walked 20 bytes on a `GroupMoveOrder`** (`+0x54..+0x68`,
/// `oxx`..`group_angle`, the PDB's field list: `MoveOrder`'s non-virtual
/// part is `+0x0..+0x50`, `GroupOrder`'s `+0x50..+0x68`, its vbptr first;
/// `GroupMoveOrder::walk_data@004855c0`, item 4243). `in_group` (`+0x68`)
/// is the walk's next range, four bytes of its own.
pub const GROUP_ORDER_FIELDS: &[(&str, usize, usize)] = &[
    ("oxx", 0x54, 4),
    ("whose", 0x58, 4),
    ("id", 0x5c, 4),
    ("form_id", 0x60, 4),
    ("group_angle", 0x64, 4),
];

/// `Stack<PathData>`'s growth step as `Unit::init` leaves it (`increment`):
/// 10 on every unit of run 1562's frame 91. Its `size` is
/// [`crate::orders::PathStack::size`], the high water a long path left
/// (run1100's 20 and 110; `docs/CHECKSUM.md` §17).
const PATH_BLOCK: i32 = crate::orders::PathStack::INCREMENT;

use crate::ids::order as kind;

fn put(b: &mut [u8], base: usize, at: usize, width: usize, v: i64) {
    let bytes = v.to_le_bytes();
    b[at - base..at - base + width].copy_from_slice(&bytes[..width]);
}

/// **The slots the units and guys lines walk: 0 to 8, never the birds' 9**
/// (`docs/CHECKSUM.md` §24.3, item 4226). `CheckSums::check_units@009371d0`
/// and `CheckSums::check_guys@00937430` step a `Leader` (`0x6eec` bytes) at a
/// time from `leaders` (`0xe3a390`) while below `0xe789dc`, which is
/// `0xe3a390 + 9 · 0x6eec`, though `Leaders::list` holds ten. A bird is
/// `init_unit`'s on slot 9 ([`crate::gaia::BIRD_OWNER`]), so the original
/// makes it, flies it and draws for it, and neither line ever walks it.
pub(super) const WALKED_SLOTS: crate::Player = 9;

/// An object's `(o, who)` as a chain word holds it: its index and owner.
type ObjRef = (i16, crate::Player);

impl Sim {
    /// **The units line** (`CheckSums::check_units@009371d0`): per leader in
    /// slot order, slots 0 to 8 ([`WALKED_SLOTS`]), per live unit in `o`
    /// order, [`Sim::checksum_unit`].
    pub fn checksum_units(&self) -> Line {
        let mut ranges = Vec::new();
        let last = self
            .units
            .iter()
            .filter(|u| u.alive() && u.owner < WALKED_SLOTS)
            .map(|u| u.owner)
            .max();
        if let Some(last) = last {
            for who in 0..=last {
                let mut mine: Vec<usize> = (0..self.units.len())
                    .filter(|&u| self.units[u].alive() && self.units[u].owner == who)
                    .collect();
                mine.sort_by_key(|&u| self.units[u].index);
                for u in mine {
                    self.checksum_unit(&mut ranges, u);
                }
            }
        }
        Line {
            name: "units",
            ranges,
        }
    }

    /// Whether unit `u` is of the `Animal` class (`Animal::work@005d7330`,
    /// `Animal::update_los@005d7d50`, `AnimalData::walk_data@005d7ea0`):
    /// gaia's, the herd animals and the birds — the units
    /// `Sim::process_unit` steps through `Animal::process`.
    fn is_animal_class(&self, u: usize) -> bool {
        self.units[u].is_gaia()
    }

    /// One unit's ranges, in the stream's order (`docs/CHECKSUM.md` §17).
    fn checksum_unit(&self, ranges: &mut Vec<Range>, u: usize) {
        let unit = &self.units[u];
        let record = unit.index;
        {
            let mut put = |field, bytes: Vec<u8>| {
                ranges.push(Range {
                    record,
                    field,
                    bytes,
                })
            };
            put("flags", vec![self.unit_flags_byte(u)]);
            // `Unit::must_walk@0060d040` behind `SubObject::walk_data`'s
            // vcall: 1 for a live unit (`flags & 1`).
            put("must_walk", vec![1]);
            let mut sub = vec![unit.owner];
            sub.extend_from_slice(&record.to_le_bytes());
            sub.extend_from_slice(&super::coord(self.world.object_z(unit.pos.tile())));
            sub.extend_from_slice(&super::coord(unit.pos.x));
            sub.extend_from_slice(&super::coord(unit.pos.y));
            put("who..y_internal", sub);
            put("type", unit.type_index.to_le_bytes().to_vec());
            // The same vcall again, from `Object::walk_data`.
            put("must_walk", vec![1]);
            put("myhits..launch_frames", self.unit_object_bytes(u));
            // `Object::walk_data`'s own local: `launching` (`+0x44`) is not
            // null — a building's launcher; never a unit's here.
            put("launching", vec![0]);
            // `Unit::walk_data`'s inlined `must_walk`.
            put("must_walk", vec![1]);
            put("collide_frame..play", self.unit_record_bytes(u));
            // `size` is the stack's high water, not its length (item 4228).
            let mut stack = unit.path.size().to_le_bytes().to_vec();
            stack.extend_from_slice(&(unit.path.len() as i32).to_le_bytes());
            stack.push(PATH_BLOCK as u8);
            put("path.size..increment", stack);
            for n in &unit.path {
                let mut b = n.to.x.to_le_bytes().to_vec();
                b.extend_from_slice(&n.to.y.to_le_bytes());
                b.extend_from_slice(&n.tolerance.to_le_bytes());
                b.extend_from_slice(&i32::from(n.flags).to_le_bytes());
                put("path[k]", b);
            }
            put(
                "orders.count",
                (unit.orders.len() as i32).to_le_bytes().to_vec(),
            );
        }
        for k in 0..unit.orders.len() {
            self.checksum_order(ranges, u, k);
        }
        self.checksum_guy_array(ranges, u);
        if self.is_animal_class(u) {
            let mut put = |field, bytes: Vec<u8>| {
                ranges.push(Range {
                    record,
                    field,
                    bytes,
                })
            };
            put("must_walk", vec![1]);
            // `AnimalData` `ox`, `whom`, `aid` as `Animal::init@005d7da0`
            // writes them; this crate keeps no animal target.
            put("ox..aid", vec![0xff; 5]);
        }
    }

    /// `SubObjectData::flags` (`+0x8`) of a unit: `1` (walked only when
    /// set), `0x08` [`crate::Unit::idle_latch`] (`Unit::check_idle@006032c0:64`
    /// raises it with `0x10`, `Unit::work@0060d180:276` drops it; item
    /// 4218: unit 0 read 0x41 ours, 0x49 theirs on the minted game's
    /// frames 1 to 30 without it), `0x10` [`crate::Unit::cant_reach`], and
    /// `0x40`, which `Object::init@00647750:39` sets when the type's
    /// `objmask` carries `0x2000000` — [`crate::combat::mask::DETECT`], the
    /// type's `obj_masks` (`docs/CHECKSUM.md` §17). The attack latch is
    /// `0x80` (window A, state 9416).
    fn unit_flags_byte(&self, u: usize) -> u8 {
        let un = &self.units[u];
        let detect = un.ty.is_some_and(|t| {
            self.tables.unit_types[t].combat.obj_masks & crate::combat::mask::DETECT != 0
        });
        1 | if un.idle_latch { 0x08 } else { 0 }
            | if un.cant_reach { 0x10 } else { 0 }
            | if detect { 0x40 } else { 0 }
            | if un.attacking { 0x80 } else { 0 }
    }

    /// A garrisoned captain's two neighbours in its building's chain, as
    /// `(index, owner)`: the object above it (the building for the head)
    /// and the captain below it (`None` at the tail). `None` when `u` is
    /// not a captain in any building's [`garrison`](crate::cities::building::Building::garrison),
    /// whose order is the chain's: `Object::insert_inside@00647e90`
    /// appends at the bottom and `Object::remove_from_inside@006480f0`
    /// unlinks in place (`docs/CITIES.md` §6, item 4257).
    pub(crate) fn garrison_links(&self, u: usize) -> Option<(ObjRef, Option<ObjRef>)> {
        let un = &self.units[u];
        let b = un.inside?;
        let bd = &self.buildings[b];
        let k = bd.garrison.iter().position(|&c| c == u)?;
        let unit = |x: usize| (self.units[x].index, self.units[x].owner);
        let above = match k {
            0 => (bd.index, bd.owner),
            _ => unit(bd.garrison[k - 1]),
        };
        Some((above, bd.garrison.get(k + 1).map(|&x| unit(x))))
    }

    /// The unit's 34 `ObjectData` bytes ([`super::builds::OBJECT_FIELDS`]).
    pub(crate) fn unit_object_bytes(&self, u: usize) -> Vec<u8> {
        let un = &self.units[u];
        let mut v = vec![0u8; 0x42 - 0x20];
        let mut p = |at, w, x: i64| put(&mut v, 0x20, at, w, x);
        p(0x20, 4, i64::from(un.max_health));
        p(0x24, 4, i64::from(un.max_health - un.health));
        // `inside_down`: the next object down the chain this one is in.
        // For a captain inside a building, the captain that entered after
        // it — `Object::insert_inside@00647e90` appends at the bottom
        // (`ObjectData::inside_bottom@00644000`), so the bottom's `+0x28`
        // names the newcomer; −1 at the tail (item 4257). Otherwise the
        // last unit seated inside this one (a carrier's chain; no unit
        // holds another on the scored frame).
        let inside = match self.garrison_links(u) {
            Some((_, below)) => below.map(|x| (i64::from(x.0), i64::from(x.1))),
            None => self
                .units
                .iter()
                .filter(|x| x.alive() && x.inside_unit == Some(u))
                .map(|x| (i64::from(x.index), i64::from(x.owner)))
                .next_back(),
        };
        p(0x28, 2, inside.map_or(-1, |x| x.0));
        let (up, down, down_who, up_who) = self.unit_chain_words(u);
        p(0x2a, 2, i64::from(up));
        p(0x2c, 2, i64::from(down));
        p(0x2e, 2, i64::from(down_who));
        p(0x30, 2, i64::from(un.uid));
        p(0x32, 2, i64::from(un.hold_frames));
        let near = un.near.map_or((-1, -1), |o| self.obj_o_who(o));
        p(0x34, 2, near.0);
        p(0x36, 2, near.1);
        p(0x38, 2, i64::from(un.healing));
        p(0x3a, 1, i64::from(un.infiltrated));
        p(0x3b, 1, i64::from(un.damage_frac));
        p(0x3c, 1, i64::from(un.mylos));
        p(0x3d, 1, i64::from(un.combat.targeted));
        // `inside_down_who`: `Object::init`'s owner while nothing is seated.
        p(0x3e, 1, inside.map_or(i64::from(un.owner), |x| x.1));
        p(0x3f, 1, i64::from(up_who));
        p(0x40, 1, i64::from(un.visible));
        // `launch_frames`: a launcher's clock (`Object::do_launch`); 0 on a
        // unit (`Object::Object`).
        p(0x41, 1, 0);
        v
    }

    /// `UnitData::unit_masks` (`+0x68`) from the bits this crate keeps
    /// (`docs/CHECKSUM.md` §17, §18). `0x8` is [`crate::Unit::line_ok`],
    /// the verified line `do_move`, `find_path`, `move_step` and
    /// `go_around_building` already keep (`docs/MOVEMENT.md`, "The verified
    /// line's lifecycle"). `0x4` is [`crate::Unit::in_danger`],
    /// `Unit::set_in_danger`'s latch (item 4251, run1600's records 27..29). `0x200` is [`crate::Unit::trade_unloaded`]
    /// (`Unit::do_trade`'s, item 4234). `0x4000000` is
    /// [`crate::Unit::explore_queued`], `add_move_facing_order`'s queued
    /// explore (item 4351; on the word since item 4405, the 2017 sample's
    /// scout `3/0` on 92). `0x200000` (`Unit::init`'s
    /// `can_carry(AIR)`) is not held.
    fn unit_masks_word(&self, u: usize) -> u32 {
        let un = &self.units[u];
        let bits = [
            (0x1, un.decoy),
            (0x2, un.movement.mirror),
            (0x4, un.in_danger),
            (0x8, un.line_ok),
            (0x10, un.contact),
            (0x20, un.gather_here),
            (0x40, un.waiting_on),
            (0x200, un.trade_unloaded),
            (0x400, un.was_builder),
            (0x800, un.cloak),
            (0x1000, un.blown),
            (0x2000, un.ambushing.is_some()),
            (0x8000, un.marching.is_some()),
            (0x10000, un.fought),
            (0x20000, un.casting),
            (0x40000, self.unit_ai_bit(un.owner)),
            (0x80000, un.combat.packed),
            (0x100000, un.half_step),
            (0x800000, un.auto_transport),
            (0x2000000, un.combat.entrenched),
            (0x4000000, un.explore_queued),
        ];
        bits.iter()
            .filter(|(_, on)| *on)
            .fold(un.carry, |w, (b, _)| w | b)
    }

    /// `UnitData::+0x86`, the union: a caravan's slot, an animal's herd, a
    /// hero's or a special's slot (`docs/SCOUT.md` §16), a citizen's hold
    /// doober (`docs/ORDERS.md` §6.12); −1 (`Unit::init`'s) otherwise
    /// (`docs/CHECKSUM.md` §18).
    fn unit_union_86(&self, u: usize) -> i64 {
        let un = &self.units[u];
        un.caravan
            .or(un.herd)
            .or(self.caster_or_doober_slot(u))
            .and_then(|s| i64::try_from(s).ok())
            .unwrap_or(-1)
    }

    /// The unit's 111 `UnitData` bytes ([`UNIT_FIELDS`]).
    pub(crate) fn unit_record_bytes(&self, u: usize) -> Vec<u8> {
        let un = &self.units[u];
        let mut v = vec![0u8; 0xb7 - 0x48];
        let mut p = |at, w, x: i64| put(&mut v, 0x48, at, w, x);
        // `collide_frame` and `damage_o`: `Unit::init` writes −1, which this
        // crate keeps as 0 — "never collided", "never hit" (the harness's
        // own note on both).
        p(
            0x48,
            4,
            if un.collide_frame == 0 {
                -1
            } else {
                un.collide_frame
            },
        );
        p(0x4c, 4, un.combat.damage_frame);
        p(0x50, 4, i64::from(un.movement.heading.0));
        p(0x54, 4, i64::from(un.rare));
        p(0x58, 4, i64::from(un.movement.des_angle.0));
        p(0x5c, 4, i64::from(un.combat.trench_angle.0));
        p(0x60, 4, i64::from(un.tolerance));
        p(0x64, 4, i64::from(un.queue_time));
        p(0x68, 4, i64::from(self.unit_masks_word(u)));
        p(0x6c, 4, i64::from(un.unit_masks2));
        p(0x70, 4, i64::from(un.orders_pos.x));
        p(0x74, 4, i64::from(un.orders_pos.y));
        p(0x78, 4, i64::from(un.birth_cell.x));
        p(0x7c, 4, i64::from(un.birth_cell.y));
        p(
            0x80,
            2,
            un.group_ptr
                .map_or(-1, |g| i64::from(un.owner) * 64 + i64::from(g)),
        );
        // `inside_up`/`inside_up_who`: the object above this one in its
        // container's chain. `insert_inside` writes the bottom it appends
        // under — the building for the first captain in, the captain
        // before it for the rest (item 4257). A squad's followers are not
        // linked (`Unit::go_inside@0061a2e0` climbs to the captain before
        // its one `insert_inside`) and keep the container.
        let inside = match self.garrison_links(u) {
            Some((above, _)) => Some((i64::from(above.0), above.1)),
            None => un
                .inside
                .map(|b| (i64::from(self.buildings[b].index), self.buildings[b].owner))
                .or(un
                    .inside_unit
                    .map(|b| (i64::from(self.units[b].index), self.units[b].owner))),
        };
        p(0x82, 2, inside.map_or(-1, |x| x.0));
        p(
            0x84,
            2,
            un.supply_slot
                .and_then(|s| i64::try_from(s).ok())
                .unwrap_or(-1),
        );
        p(0x86, 2, self.unit_union_86(u));
        p(0x88, 2, i64::from(un.collide));
        p(0x8a, 2, i64::from(un.collide_o));
        p(0x8c, 2, i64::from(un.collide_guy));
        let link = |l: Option<usize>| l.map_or(-1, |x| i64::from(self.units[x].index));
        p(0x8e, 2, link(un.o_up));
        p(0x90, 2, link(un.o_down));
        p(0x92, 2, self.unit_gather_down(u));
        p(0x94, 2, i64::from(un.good_obj));
        p(0x96, 2, i64::from(un.mana_burn));
        p(0x98, 2, i64::from(un.spell_time));
        p(0x9a, 2, i64::from(un.movement.speed));
        // `myarmor`: the cache `Unit::update_armor` wrote (item 4464).
        p(0x9c, 2, i64::from(un.myarmor));
        p(0x9e, 2, i64::from(un.attrition));
        p(0xa0, 2, i64::from(un.num_queued));
        // `cavarch_o`: the held target's `o`, else −1, or −2 once a
        // fire-on-the-move search has come back empty twice running
        // (`Unit::cavarch_fight@005ff4b0:34`, `cavarch_idle`; item 4443).
        let cast = un.cast_target.map(|o| self.obj_o_who(o));
        let none = if un.cavarch_idle { -2 } else { -1 };
        p(0xa2, 2, cast.map_or(none, |x| x.0));
        p(
            0xa4,
            2,
            if un.combat.damage_frame == 0 {
                -1
            } else {
                i64::from(un.combat.damage_o)
            },
        );
        // `cavarch_uid`: the carried word, kept past a dropped target
        // (item 4476).
        p(0xa6, 2, i64::from(un.cavarch_uid));
        p(0xa8, 1, i64::from(un.cavarch_who));
        p(0xa9, 1, i64::from(un.combat.damage_who));
        p(0xaa, 1, i64::from(un.form));
        p(0xab, 1, i64::from(un.form_width));
        p(0xac, 1, i64::from(un.full));
        p(0xad, 1, i64::from(un.waiting));
        p(0xae, 1, i64::from(un.combat.recharging));
        p(0xaf, 1, i64::from(un.path_recursion));
        p(0xb0, 1, i64::from(un.idle));
        p(0xb1, 1, i64::from(un.stance));
        p(0xb2, 1, i64::from(un.safe));
        p(0xb3, 1, i64::from(un.collide_who));
        // `inside_up_who`: `Unit::init:278`'s owner, the container's inside.
        p(
            0xb4,
            1,
            inside.map_or(i64::from(un.owner), |x| i64::from(x.1)),
        );
        p(0xb5, 1, crate::units::anim::SQUAD_SIZE as i64);
        // `play`: `process_group`'s write (`docs/ORDERS.md` §8.1).
        p(0xb6, 1, i64::from(un.play));
        v
    }

    /// `UnitData::gather_down` (`+0x92`): the next unit down its building's
    /// gatherer chain (`Build::add_gatherer`), whose head is the building's
    /// own `gather_down` (the builds line's); −1 at the tail and off it.
    fn unit_gather_down(&self, u: usize) -> i64 {
        for bd in &self.buildings {
            if let Some(k) = bd.gatherers.iter().position(|&x| x == u) {
                return bd
                    .gatherers
                    .get(k + 1)
                    .map_or(-1, |&n| i64::from(self.units[n].index));
            }
        }
        -1
    }

    /// The `uid` (`ObjectData +0x30`) of a combat object.
    pub(crate) fn obj_uid(&self, o: Obj) -> u16 {
        match o {
            Obj::Unit(v) => self.units[v].uid,
            Obj::Building(b) => self.buildings[b].uid,
        }
    }

    /// Order `k` of unit `u`: its type (a 4-byte local, vslot `+0x10`), the
    /// list node's `metric` byte (`+0xc`, 0), then its walker's ranges —
    /// the virtual base's `UnitOrder::flags` byte first. The kinds on run
    /// 1562's frame are a move (`MoveOrder` `x`..`off_y`), a `BUILD_AT`
    /// (`TargetOrder` `ox`, `whom`, `uid`) and a `GATHER` (that, and
    /// `tx`..`been_there`); a `TRADE_ROUTE` (that target, and `oxx`..
    /// `uid2`, item 4230) first held one on the minted game's 3166; a
    /// `CAST_SPELL` (that target, and `x`..`spell`, item 4234) on 3596; a
    /// `GUARD` (that target, and `dx`..`retry`, item 4266) on 12537; a
    /// `GARRISON` (that target, and `search`, item 4440) on the sample's
    /// 2888; a
    /// `GROUP_MOVE` or `GROUP_ATTACK_TO` (the move, then the flags again,
    /// `oxx`..`group_angle` and `in_group`, item 4243) on 6907; a `THINK`
    /// (the flags byte, item 4414) on the 2017 sample's 532; an
    /// `AIR_ATTACK_GROUND` (the flags byte, the ground point, the flags
    /// byte again, the `AirOrder` row and `total_time`..`sy`, item 4459) on
    /// run1721's 2587; a `REPAIR` (that target alone, item 4479) on the
    /// sample's 3921; any other kind hands over its type and node byte
    /// alone here, its walker not ported (`docs/CHECKSUM.md` §17).
    fn checksum_order(&self, ranges: &mut Vec<Range>, u: usize, k: usize) {
        let unit = &self.units[u];
        let record = unit.index;
        let o = unit.orders[k];
        let mut put = |field, bytes: Vec<u8>| {
            ranges.push(Range {
                record,
                field,
                bytes,
            })
        };
        put("order.type", i32::from(o.index()).to_le_bytes().to_vec());
        put("order.node", vec![0]);
        let target = |b: usize| -> Vec<u8> {
            let bd = &self.buildings[b];
            let mut t = vec![0u8; 10];
            put_raw(&mut t, 0, 4, i64::from(bd.index));
            put_raw(&mut t, 4, 4, i64::from(bd.owner));
            put_raw(&mut t, 8, 2, i64::from(bd.uid));
            t
        };
        match o.body {
            Body::Move(m) => {
                put("order.flags", vec![o.flags]);
                let last = m.last.unwrap_or(crate::Pos::new(-1, -1));
                let coll = m.coll.unwrap_or(crate::Pos::new(0, 0));
                let orig = m
                    .orig
                    .or(m.group.map(|g| g.orig))
                    .unwrap_or(crate::Pos::new(-1, -1));
                let mut b = vec![0u8; 0x50 - 0x4];
                let mut p = |at: usize, w, x: i64| put_raw(&mut b, at - 0x4, w, x);
                p(0x4, 4, i64::from(m.dest.x));
                p(0x8, 4, i64::from(m.dest.y));
                p(0xc, 4, i64::from(m.angle.0));
                p(0x10, 4, i64::from(m.has_waypoint));
                // `tolerance` (`+0x14`): no writer in this crate; 0 on every
                // move of run29's field table and of this frame.
                p(0x14, 4, 0);
                p(0x18, 4, i64::from(m.pause));
                p(0x1c, 4, i64::from(m.retry));
                p(0x20, 4, i64::from(m.attempts));
                p(0x24, 4, i64::from(m.timer));
                p(0x28, 4, m.facing.map_or(-1, i64::from));
                p(0x2c, 4, i64::from(m.waypoint.x));
                p(0x30, 4, i64::from(m.waypoint.y));
                p(0x34, 4, i64::from(last.x));
                p(0x38, 4, i64::from(last.y));
                p(0x3c, 4, i64::from(coll.x));
                p(0x40, 4, i64::from(coll.y));
                p(0x44, 4, i64::from(orig.x));
                p(0x48, 4, i64::from(orig.y));
                // `off_x`/`off_y`: the destination's offset in its 0x300
                // block (`diff::order`'s own reading).
                p(0x4c, 2, i64::from(m.dest.x.rem_euclid(0x300)));
                p(0x4e, 2, i64::from(m.dest.y.rem_euclid(0x300)));
                put("order.move", b);
                // `GroupMoveOrder::walk_data@004855c0` (item 4243), which
                // `GroupAttackToOrder::walk_data@00482510` is whole: after
                // `MoveOrder`'s two ranges, `GroupOrder::walk_data`'s two
                // inlined — the virtual base's flags byte again, through
                // `GroupOrder`'s own vbptr, and `oxx`..`group_angle` — then
                // `in_group`. A plain `MOVE_TO`/`ATTACK_TO` walks
                // `MoveOrder::walk_data` and stops at `order.move`.
                if let Some(g) = m.group
                    && matches!(o.index(), kind::GROUP_MOVE | kind::GROUP_ATTACK_TO)
                {
                    put("order.flags", vec![o.flags]);
                    let leader = &self.units[g.leader];
                    let mut b = vec![0u8; 0x68 - 0x54];
                    debug_assert_eq!(GROUP_ORDER_FIELDS.len(), 5);
                    let mut p = |at: usize, w, x: i64| put_raw(&mut b, at - 0x54, w, x);
                    p(0x54, 4, i64::from(leader.index));
                    p(0x58, 4, i64::from(leader.owner));
                    p(0x5c, 4, g.id);
                    p(0x60, 4, g.form_id as i64);
                    p(0x64, 4, i64::from(g.group_angle.0));
                    put("order.group", b);
                    put(
                        "order.in_group",
                        i32::from(g.in_group).to_le_bytes().to_vec(),
                    );
                }
            }
            // `ThinkOrder`'s vslot `+0x8` is the base's own
            // `UnitOrder::walk_data@0047f2b0` (`ThinkOrder::vftable@00b48d88`):
            // the flags byte alone (item 4414, the sample's units on 532).
            Body::Think => put("order.flags", vec![o.flags]),
            // `GarrisonOrder::walk_data@00484690` (item 4440, the sample's
            // units on 2888): the flags byte, `TargetOrder`'s `ox`, `whom`,
            // `uid` (`+0x8..+0x12`, the building it goes into) and its own
            // `search` (`+0x14`, four bytes; the PDB's field list).
            Body::Garrison { building, search } => {
                put("order.flags", vec![o.flags]);
                put("order.target", target(building));
                put("order.search", i32::from(search).to_le_bytes().to_vec());
            }
            Body::Build(b) if o.index() == kind::BUILD_AT => {
                put("order.flags", vec![o.flags]);
                put("order.target", target(b));
            }
            // `RepairOrder` has no walker of its own: its bytes fold at
            // `0047f243` and `0047f251`, both in `TargetOrder::walk_data@
            // 0047f220` — the flags byte, then `ox`, `whom`, `uid` (`+0x8..
            // +0x12`, the building repaired) — and the guys follow (run1724,
            // state 3921: unit `3/11`'s second order, `2010` of player 3,
            // uid 0x31; item 4479, the sample's units on 3921).
            Body::Repair(b) => {
                put("order.flags", vec![o.flags]);
                put("order.target", target(b));
            }
            Body::Gather(g) if o.index() == kind::GATHER => {
                put("order.flags", vec![o.flags]);
                put("order.target", target(g.building));
                let tile = g.tile.unwrap_or(crate::Pos::new(-1, -1));
                let ty = self.building_type_index(g.building);
                let mut b = vec![0u8; 0x28 - 0x14];
                let mut p = |at: usize, w, x: i64| put_raw(&mut b, at - 0x14, w, x);
                p(0x14, 4, i64::from(tile.x));
                p(0x18, 4, i64::from(tile.y));
                p(0x1c, 4, ty);
                p(0x20, 4, i64::from(g.wait));
                p(0x24, 1, i64::from(g.goto_build));
                // `non_flat_gather`: `Unit::add_gather_order@0061a5c0`'s
                // lay — a gather type that is neither flat nor a University.
                p(0x25, 1, i64::from(self.gather_is_non_flat(g.building)));
                p(0x26, 1, i64::from(g.dist_mod));
                p(0x27, 1, i64::from(g.been_there));
                put("order.gather", b);
            }
            Body::Trade(t) => {
                // `TradeOrder::walk_data@00483260` (item 4230): the flags
                // byte, `TargetOrder`'s `ox`, `whom`, `uid` (`+0x8..+0x12`)
                // and its own `oxx`..`uid2` (`+0x14..+0x26`). The two ends
                // are each city's building: `Unit::add_trade_order@005e4dc0`
                // writes the home's object, owner and uid, and `oxx`/`whose`
                // −1 with `uid2` 0xffff; `do_trade@005ed270:276`–`:278`
                // writes the far one's when it picks it.
                put("order.flags", vec![o.flags]);
                put("order.target", target(self.cities.cities[t.home].building));
                let mut b = vec![0u8; 0x26 - 0x14];
                debug_assert_eq!(TRADE_FIELDS.len(), 5);
                let mut p = |at: usize, w, x: i64| put_raw(&mut b, at - 0x14, w, x);
                match t
                    .dest
                    .map(|c| &self.buildings[self.cities.cities[c].building])
                {
                    Some(bd) => {
                        p(0x14, 4, i64::from(bd.index));
                        p(0x18, 4, i64::from(bd.owner));
                        p(0x24, 2, i64::from(bd.uid));
                    }
                    None => {
                        p(0x14, 4, -1);
                        p(0x18, 4, -1);
                        p(0x24, 2, -1);
                    }
                }
                p(0x1c, 4, i64::from(t.started));
                p(0x20, 4, i64::from(t.loaded));
                put("order.trade", b);
            }
            Body::Attack(a) => {
                put("order.flags", vec![o.flags]);
                let mut target = vec![0xff; 10];
                if let Some((object, owner, uid)) = a.target {
                    put_raw(&mut target, 0, 4, i64::from(object));
                    put_raw(&mut target, 4, 4, i64::from(owner));
                    put_raw(&mut target, 8, 2, i64::from(uid));
                }
                put("order.target", target);
                let def = a.def.unwrap_or(crate::Pos::new(-1, -1));
                let mut fields = Vec::with_capacity(13);
                fields.extend_from_slice(&def.x.to_le_bytes());
                fields.extend_from_slice(&def.y.to_le_bytes());
                fields.extend(
                    [
                        a.mandatory,
                        a.defensive,
                        a.in_range,
                        a.ever_in_range,
                        a.new_ord,
                    ]
                    .map(u8::from),
                );
                put("order.attack", fields);
            }
            Body::Cast(c) => {
                // `CastOrder::walk_data@004860a0` (item 4234): the flags
                // byte, `TargetOrder`'s `ox`, `whom`, `uid` (`+0x8..+0x12`)
                // and its own `x`, `y`, `paid`, `spell` (`+0x14..+0x24`).
                // An untargeted cast keeps what `CastOrder::CastOrder@00486390`
                // and `clear@00486350` write: −1, −1, 0xffff and a point of
                // (−1, −1).
                put("order.flags", vec![o.flags]);
                let mut t = vec![0u8; 10];
                match c.target {
                    Some(obj) => {
                        let (index, owner) = match obj {
                            Obj::Unit(v) => (self.units[v].index, self.units[v].owner),
                            Obj::Building(b) => (self.buildings[b].index, self.buildings[b].owner),
                        };
                        put_raw(&mut t, 0, 4, i64::from(index));
                        put_raw(&mut t, 4, 4, i64::from(owner));
                        put_raw(&mut t, 8, 2, i64::from(self.obj_uid(obj)));
                    }
                    None => {
                        put_raw(&mut t, 0, 4, -1);
                        put_raw(&mut t, 4, 4, -1);
                        put_raw(&mut t, 8, 2, -1);
                    }
                }
                put("order.target", t);
                let mut b = vec![0u8; 0x24 - 0x14];
                debug_assert_eq!(CAST_FIELDS.len(), 4);
                let mut p = |at: usize, w, x: i64| put_raw(&mut b, at - 0x14, w, x);
                p(0x14, 4, i64::from(c.at.x));
                p(0x18, 4, i64::from(c.at.y));
                p(0x1c, 4, i64::from(c.paid));
                p(0x20, 4, i64::from(c.spell));
                put("order.cast", b);
            }
            Body::Guard(g) => {
                // `GuardOrder::walk_data@00486550` (item 4266): the flags
                // byte, `TargetOrder`'s `ox`, `whom`, `uid` (`+0x8..+0x12`)
                // — the escorted unit, which `Unit::add_guard_order@005e3e40`
                // writes — and its own `dx`..`retry` (`+0x14..+0x2c`).
                put("order.flags", vec![o.flags]);
                let t = &self.units[g.target];
                let mut tb = vec![0u8; 10];
                put_raw(&mut tb, 0, 4, i64::from(t.index));
                put_raw(&mut tb, 4, 4, i64::from(t.owner));
                put_raw(&mut tb, 8, 2, i64::from(t.uid));
                put("order.target", tb);
                let mut b = vec![0u8; 0x2c - 0x14];
                debug_assert_eq!(GUARD_FIELDS.len(), 6);
                let mut p = |at: usize, w, x: i64| put_raw(&mut b, at - 0x14, w, x);
                p(0x14, 4, i64::from(g.dx));
                p(0x18, 4, i64::from(g.dy));
                p(0x1c, 4, i64::from(g.guard.x));
                p(0x20, 4, i64::from(g.guard.y));
                p(0x24, 4, i64::from(g.idle));
                p(0x28, 4, i64::from(g.retry));
                put("order.guard", b);
            }
            Body::AirAttackGround(g) => {
                // `AirAttackGroundOrder::walk_data@004875a0` (item 4459,
                // run1721's units on 2587, a missile `1/84` laid in its
                // silo): the flags byte through `AttackGroundOrder`'s
                // vbptr, its `att_x`..`attack_unit` (`+0x4..+0x14`), the
                // flags byte again through `AirOrder`'s, the `AirOrder`
                // row (`+0x18..+0x30`) and its own `total_time`, `sx`,
                // `sy` (`+0x30..+0x3c`). `Unit::add_air_attack_ground_order@
                // 005e41c0` writes the point, 0 `accuracy` and
                // `attack_unit`, the home, `cruising_alt` 0x640 and 0
                // `returning` on an order `get_obj` has just cleared;
                // `do_air_physics` rewrites `cruising_alt` and `returning`.
                // SEAM: `old` and the class's own three are written by no
                // function but `clear` (0) and `operator=` in the export,
                // and walk 0 here.
                put("order.flags", vec![o.flags]);
                let mut b = vec![0u8; 0x14 - 0x4];
                put_raw(&mut b, 0, 4, i64::from(g.at.x));
                put_raw(&mut b, 4, 4, i64::from(g.at.y));
                put("order.attack_ground", b);
                put("order.flags", vec![o.flags]);
                let mut b = vec![0u8; 0x1c - 0x4];
                debug_assert_eq!(AIR_FIELDS.len(), 6);
                let (oxx, whose) = g.home.map_or((-1, -1), |h| {
                    let bd = &self.buildings[h];
                    (i64::from(bd.index), i64::from(bd.owner))
                });
                let mut p = |at: usize, w, x: i64| put_raw(&mut b, at - 0x4, w, x);
                p(0x4, 4, oxx);
                p(0x8, 4, whose);
                p(0xc, 4, i64::from(g.cruising_alt));
                p(0x10, 4, i64::from(g.sharp_turn));
                p(0x18, 4, i64::from(g.returning));
                put("order.air", b);
                put("order.air_attack_ground", vec![0u8; 0x3c - 0x30]);
            }
            Body::Strafe(st) => {
                // `StrafeOrder::walk_data@0047f310` (item 4491, run1721's
                // units on 2758, the Jet Fighter `1/91` handed a strafe on
                // 2757 above its patrol): the flags byte through
                // `TargetOrder`'s vbptr, `ox`, `whom`, `uid` (`+0x8..+0x12`),
                // `AttackOrder`'s `def_x`..`new_ord` (`+0x14..+0x21`), the
                // flags byte again through `AirOrder`'s, the `AirOrder` row
                // (`+0x28..+0x40`) and its own `xx`, `yy` (`+0x40..+0x48`).
                // `StrafeOrder::clear@00480040` leaves `def` at (−1, −1),
                // `defensive`, `in_range`, `ever_in_range` 0 and `new_ord` 1;
                // `Unit::add_strafe_order@005e48c0` writes the target, its
                // `uid` (the object's `+0x30`), `mandatory`, the home,
                // `cruising_alt` and `returning`, and the point.
                // SEAM: the attack fields past `mandatory` walk `clear`'s
                // values; a writer through the `get_attack_order` vslot on a
                // strafe is not carried, and none is walked by run1721 to
                // its end.
                put("order.flags", vec![o.flags]);
                let mut t = vec![0u8; 10];
                match st.target {
                    Some(obj) => {
                        let (index, owner) = match obj {
                            Obj::Unit(v) => (self.units[v].index, self.units[v].owner),
                            Obj::Building(b) => (self.buildings[b].index, self.buildings[b].owner),
                        };
                        put_raw(&mut t, 0, 4, i64::from(index));
                        put_raw(&mut t, 4, 4, i64::from(owner));
                        put_raw(&mut t, 8, 2, i64::from(self.obj_uid(obj)));
                    }
                    None => {
                        put_raw(&mut t, 0, 4, -1);
                        put_raw(&mut t, 4, 4, -1);
                        put_raw(&mut t, 8, 2, -1);
                    }
                }
                put("order.target", t);
                let mut fields = Vec::with_capacity(13);
                fields.extend_from_slice(&(-1i32).to_le_bytes());
                fields.extend_from_slice(&(-1i32).to_le_bytes());
                fields.extend([u8::from(st.mandatory), 0, 0, 0, 1]);
                put("order.attack", fields);
                put("order.flags", vec![o.flags]);
                let mut b = vec![0u8; 0x1c - 0x4];
                let (oxx, whose) = st.home.map_or((-1, -1), |h| {
                    let bd = &self.buildings[h];
                    (i64::from(bd.index), i64::from(bd.owner))
                });
                let mut p = |at: usize, w, x: i64| put_raw(&mut b, at - 0x4, w, x);
                p(0x4, 4, oxx);
                p(0x8, 4, whose);
                p(0xc, 4, i64::from(st.cruising_alt));
                p(0x10, 4, i64::from(st.sharp_turn));
                p(0x18, 4, i64::from(st.returning));
                put("order.air", b);
                let at = st.at.unwrap_or(crate::Pos::new(-1, -1));
                let mut xy = Vec::with_capacity(8);
                xy.extend_from_slice(&at.x.to_le_bytes());
                xy.extend_from_slice(&at.y.to_le_bytes());
                put("order.strafe", xy);
            }
            Body::AirPatrol(ap) => {
                // `AirPatrolOrder::walk_data@00483ed0` (item 4474, run1721's
                // units on 2743, the Jet Fighter `1/91` handed a patrol on
                // 2742): the flags byte through `PatrolOrder`'s vbptr, its
                // `waypoint` (`+0x3c`), `x_pos` (`+0x4`) and `y_pos`
                // (`+0x20`) by `SimpleArray<Coord>::walk_data@00483cc0`, the
                // flags byte again through `AirOrder`'s, and the `AirOrder`
                // row (`+0x44..+0x5c`).
                put("order.flags", vec![o.flags]);
                put(
                    "order.waypoint",
                    (ap.waypoint as i32).to_le_bytes().to_vec(),
                );
                let size = patrol_array_size(usize::from(ap.len));
                for (name, axis) in [("order.x_pos", 0usize), ("order.y_pos", 1)] {
                    let n = usize::from(ap.len);
                    put(name, (n as i32).to_le_bytes().to_vec());
                    if n > 0 {
                        put(name, (size as i32).to_le_bytes().to_vec());
                        put(name, (-1i16).to_le_bytes().to_vec());
                        put(name, vec![0]);
                        let list = ap
                            .live()
                            .iter()
                            .flat_map(|p| if axis == 0 { p.x } else { p.y }.to_le_bytes())
                            .collect();
                        put(name, list);
                    }
                }
                put("order.flags", vec![o.flags]);
                let mut b = vec![0u8; 0x1c - 0x4];
                let (oxx, whose) = ap.home.map_or((-1, -1), |h| {
                    let bd = &self.buildings[h];
                    (i64::from(bd.index), i64::from(bd.owner))
                });
                let mut p = |at: usize, w, x: i64| put_raw(&mut b, at - 0x4, w, x);
                p(0x4, 4, oxx);
                p(0x8, 4, whose);
                p(0xc, 4, i64::from(ap.cruising_alt));
                p(0x10, 4, i64::from(ap.sharp_turn));
                p(0x18, 4, i64::from(ap.returning));
                put("order.air", b);
            }
            _ => {}
        }
    }

    /// `GatherOrder::non_flat_gather` as `Unit::add_gather_order` lays it on
    /// building `b`: a gather type (`build_flags & 0x40`) that is not flat
    /// and not a University — the same test that gives `dist_mod` its 4 or
    /// 10 (`docs/ORDERS.md` §6.2).
    fn gather_is_non_flat(&self, b: usize) -> bool {
        self.is_gather_type(b)
            && !self.is_flat(b)
            && self.building_ident(b) != crate::cities::build::Ident::University
    }
}

/// **An `AirPatrolOrder` array's `size`** (`SimpleArray<Coord>`, `+0xc` and
/// `+0x28`), the capacity its walk hands over after the length.
/// `PatrolOrder::clear@00483ac0` keeps a capacity of at least 1 (it
/// reallocates one slot only below 1), and `SimpleArray<Coord>::make_valid@
/// 0046edd0` at `increment` −1 grows by the capacity itself when that
/// suffices: 1, 2, 4, 8 as an Airbase's gather list appends. SEAM: an order
/// the original's order pool recycles keeps the capacity its last use grew
/// to, and this crate keeps no pool; this is a fresh object's.
fn patrol_array_size(n: usize) -> usize {
    n.max(1).next_power_of_two()
}

fn put_raw(b: &mut [u8], at: usize, width: usize, v: i64) {
    b[at..at + width].copy_from_slice(&v.to_le_bytes()[..width]);
}
