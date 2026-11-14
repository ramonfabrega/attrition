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

/// `Stack<PathData>`'s allocation and growth as `Unit::init` leaves them
/// (`size`, `increment`): 10 and 10 on every unit of run 1562's frame 91;
/// run1100's carry sizes 20 and 110 where a long path grew the stack
/// (`docs/CHECKSUM.md` §17).
const PATH_BLOCK: i32 = 10;

use crate::ids::order as kind;

fn put(b: &mut [u8], base: usize, at: usize, width: usize, v: i64) {
    let bytes = v.to_le_bytes();
    b[at - base..at - base + width].copy_from_slice(&bytes[..width]);
}

impl Sim {
    /// **The units line** (`CheckSums::check_units@009371d0`): per leader in
    /// slot order, per live unit in `o` order, [`Sim::checksum_unit`].
    pub fn checksum_units(&self) -> Line {
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
            let mut stack = PATH_BLOCK
                .max(unit.path.len().div_ceil(PATH_BLOCK as usize) as i32 * PATH_BLOCK)
                .to_le_bytes()
                .to_vec();
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
    /// type's `obj_masks` (`docs/CHECKSUM.md` §17). `0x20` and `0x80` are
    /// not emitted.
    fn unit_flags_byte(&self, u: usize) -> u8 {
        let un = &self.units[u];
        let detect = un.ty.is_some_and(|t| {
            self.tables.unit_types[t].combat.obj_masks & crate::combat::mask::DETECT != 0
        });
        1 | if un.idle_latch { 0x08 } else { 0 }
            | if un.cant_reach { 0x10 } else { 0 }
            | if detect { 0x40 } else { 0 }
    }

    /// The unit's 34 `ObjectData` bytes ([`super::builds::OBJECT_FIELDS`]).
    fn unit_object_bytes(&self, u: usize) -> Vec<u8> {
        let un = &self.units[u];
        let mut v = vec![0u8; 0x42 - 0x20];
        let mut p = |at, w, x: i64| put(&mut v, 0x20, at, w, x);
        p(0x20, 4, i64::from(un.max_health));
        p(0x24, 4, i64::from(un.max_health - un.health));
        // `inside_down`: the last unit seated inside this one, −1 for none
        // (a carrier's chain; no unit holds another on the scored frame).
        let inside = self
            .units
            .iter()
            .filter(|x| x.alive() && x.inside_unit == Some(u))
            .map(|x| (i64::from(x.index), i64::from(x.owner)))
            .next_back();
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
    /// line's lifecycle"). `0x200` (`Unit::do_trade`'s) and `0x200000`
    /// (`Unit::init`'s `can_carry(AIR)`) are not held.
    fn unit_masks_word(&self, u: usize) -> u32 {
        let un = &self.units[u];
        let bits = [
            (0x1, un.decoy),
            (0x2, un.movement.mirror),
            (0x8, un.line_ok),
            (0x10, un.contact),
            (0x20, un.gather_here),
            (0x40, un.waiting_on),
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
    fn unit_record_bytes(&self, u: usize) -> Vec<u8> {
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
        let inside = un
            .inside
            .map(|b| (i64::from(self.buildings[b].index), self.buildings[b].owner))
            .or(un
                .inside_unit
                .map(|b| (i64::from(self.units[b].index), self.units[b].owner)));
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
        p(0x9c, 2, i64::from(self.object_armor(Obj::Unit(u))));
        p(0x9e, 2, i64::from(un.attrition));
        p(0xa0, 2, i64::from(un.num_queued));
        let cast = un.cast_target.map(|o| self.obj_o_who(o));
        p(0xa2, 2, cast.map_or(-1, |x| x.0));
        p(
            0xa4,
            2,
            if un.combat.damage_frame == 0 {
                -1
            } else {
                i64::from(un.combat.damage_o)
            },
        );
        p(
            0xa6,
            2,
            un.cast_target.map_or(0, |o| i64::from(self.obj_uid(o))),
        );
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
    fn obj_uid(&self, o: Obj) -> u16 {
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
    /// `tx`..`been_there`); any other kind hands over its type and node
    /// byte alone here, its walker not ported (`docs/CHECKSUM.md` §17).
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
            }
            Body::Build(b) if o.index() == kind::BUILD_AT => {
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

fn put_raw(b: &mut [u8], at: usize, width: usize, v: i64) {
    b[at..at + width].copy_from_slice(&v.to_le_bytes()[..width]);
}
