//! **The groups line** (`CheckSums::check_groups@00937530`,
//! `docs/CHECKSUM.md` §7 and §19): every slot of the groups array, 512 —
//! 64 a player, the slot `who·64 + s` — through `Group::walk_data@00708400`:
//! the record's `[+0x4, +0x4c)`, `id`..`march` ([`GROUP_RECORD_FIELDS`]), and for a
//! slot whose `num` is not 0, `num` entries each of `list`, `off_x`,
//! `off_y`, `curr_x`, `curr_y` and `angles`, one range an array. Then the
//! eight words of `GroupsData::last_group` (`const_last_group` points at
//! `+0x1c`), each folded as a 4-byte stack copy.
//!
//! This crate keeps a pool slot's record on the army that holds it or on a
//! [`crate::groups::group::Pushed`] entry (`docs/GROUPS.md` §3, §19, §29);
//! a slot neither holds is `Groups::clear@00713f20`'s image. A `GroupData`
//! `Coord` is stored **plain**.

use super::{Line, Range};
use crate::Player;
use crate::Sim;
use crate::groups::group::GroupState;

/// **`GroupData`'s walked 72 bytes** (`+0x4..+0x4c`, `id`..`march`):
/// `(name, offset, width)`, 21 fields (`types.txt`, `GroupData`, size 0x9cc).
pub const GROUP_RECORD_FIELDS: &[(&str, usize, usize)] = &[
    ("id", 0x4, 4),
    ("army", 0x8, 4),
    ("num", 0xc, 4),
    ("form", 0x10, 4),
    ("stamp", 0x14, 4),
    ("ox", 0x18, 4),
    ("oy", 0x1c, 4),
    ("o_dist", 0x20, 4),
    ("o_angle", 0x24, 4),
    ("disband", 0x28, 4),
    ("order_num", 0x2c, 4),
    ("priority", 0x30, 4),
    ("role", 0x34, 4),
    ("think_frame", 0x38, 4),
    ("new_speed", 0x3c, 4),
    ("speed", 0x40, 4),
    ("form_num", 0x44, 4),
    ("facing", 0x48, 1),
    ("buildings", 0x49, 1),
    ("who", 0x4a, 1),
    ("march", 0x4b, 1),
];

/// The groups array's length, `Groups::clear`'s `0x200`: 64 slots for
/// each of the eight leaders.
const GROUP_SLOTS: usize = 512;

/// A player's slots in the array (`get_open_slot` walks `who·64 + 0..45`).
const SLOTS_PER_PLAYER: usize = 64;

/// One slot as the walk reads it: the 21 fields' values in
/// [`GROUP_RECORD_FIELDS`] order, then per member `(list, off_x, off_y, curr_x,
/// curr_y, angles)`.
type SlotImage = ([i64; 21], Vec<(i16, i32, i32, i32, i32, i8)>);

impl Sim {
    /// **The groups line** (`docs/CHECKSUM.md` §19).
    pub fn checksum_groups(&self) -> Line {
        let mut ranges = Vec::new();
        for id in 0..GROUP_SLOTS {
            let (f, members) = self.group_slot_image(id);
            let record = id as i16;
            let mut b = Vec::with_capacity(0x48);
            for (&(_, _, w), &v) in GROUP_RECORD_FIELDS.iter().zip(&f) {
                b.extend_from_slice(&(v as i32).to_le_bytes()[..w]);
            }
            ranges.push(Range {
                record,
                field: "id..march",
                bytes: b,
            });
            if members.is_empty() {
                continue;
            }
            let mut put = |field, bytes: Vec<u8>| {
                ranges.push(Range {
                    record,
                    field,
                    bytes,
                })
            };
            put(
                "list",
                members.iter().flat_map(|m| m.0.to_le_bytes()).collect(),
            );
            put(
                "off_x",
                members.iter().flat_map(|m| m.1.to_le_bytes()).collect(),
            );
            put(
                "off_y",
                members.iter().flat_map(|m| m.2.to_le_bytes()).collect(),
            );
            put(
                "curr_x",
                members.iter().flat_map(|m| m.3.to_le_bytes()).collect(),
            );
            put(
                "curr_y",
                members.iter().flat_map(|m| m.4.to_le_bytes()).collect(),
            );
            put("angles", members.iter().map(|m| m.5 as u8).collect());
        }
        for who in 0..8 {
            ranges.push(Range {
                record: -1,
                field: "last_group",
                bytes: self.group_last_group(who).to_le_bytes().to_vec(),
            });
        }
        Line {
            name: "groups",
            ranges,
        }
    }

    /// `GroupsData::last_group[who]` as the original holds it, the array
    /// index `who·64 + s` — `Groups::Groups`' and `Groups::clear`'s `who·64`,
    /// then each `push_group`'s slot (§19).
    pub fn group_last_group(&self, who: Player) -> i32 {
        i32::from(who) * SLOTS_PER_PLAYER as i32 + i32::from(self.orders.last_group[who as usize])
    }

    /// Slot `id`'s fields and members, from the record this crate holds on
    /// it (`docs/CHECKSUM.md` §19 has the rule for each field).
    fn group_slot_image(&self, id: usize) -> SlotImage {
        let who = (id / SLOTS_PER_PLAYER) as Player;
        let s = (id % SLOTS_PER_PLAYER) as u8;
        let w = who as usize;
        let army = self
            .ai
            .armies
            .get(w)
            .and_then(|x| x.list.iter().find(|a| a.valid && a.group.pool == Some(s)));
        let pushed = self
            .orders
            .pushed
            .iter()
            .find(|p| p.who == who && p.state.pool == Some(s));
        // The record: an army's while it has members, else the pushed
        // entry's, else an army's empty one.
        let (st, units, builds): (&GroupState, &[usize], &[usize]) = match (army, pushed) {
            (Some(a), _) if !a.units.is_empty() || pushed.is_none() => (&a.group, &a.units, &[]),
            (_, Some(p)) => (&p.state, &p.list, &p.builds),
            (Some(a), None) => (&a.group, &a.units, &[]),
            (None, None) => {
                // `Groups::clear@00713f20`: `Group::clear(id)` — `army`
                // and `form` −1, every other field 0 — and then `stamp`
                // and `priority` zeroed again.
                let mut f = [0i64; 21];
                f[0] = id as i64;
                f[1] = -1;
                f[3] = -1;
                return (f, Vec::new());
            }
        };
        let num = units.len() + builds.len();
        // `army` (`+0x8`): `Army::add_group@006f8c00` writes the army's
        // number; `Group::clear` −1, and `copy_group` leaves it.
        let army_no = army
            .filter(|a| !a.group.unlinked)
            .map_or(-1, |a| i64::from(a.army));
        // `who` (`+0x4a`): `Group::clear` writes 0, `copy_group` and the
        // first `Group::add` the owner — so an emptied, cleared record
        // reads 0 until its slot is taken again.
        let who_byte = if num == 0 && st.unlinked { 0 } else { w as i64 };
        let f = [
            id as i64,
            army_no,
            num as i64,
            i64::from(st.form),
            st.stamp,
            i64::from(st.o.x),
            i64::from(st.o.y),
            i64::from(st.o_dist),
            i64::from(st.o_angle.0),
            // `disband` (`+0x28`): every writer in the export stores 0.
            0,
            i64::from(st.order_num),
            // `priority` (`+0x30`): only the two clears write it, with 0.
            0,
            i64::from(st.role as i32),
            st.opportunity,
            i64::from(st.new_speed),
            i64::from(st.speed),
            i64::from(st.form_num),
            i64::from(st.facing),
            i64::from(!builds.is_empty()),
            who_byte,
            i64::from(st.march),
        ];
        let names = units
            .iter()
            .map(|&u| self.units[u].index)
            .chain(builds.iter().map(|&b| self.buildings[b].index));
        let members = names
            .enumerate()
            .map(|(k, o)| {
                let (ox, oy) = st.off.get(k).copied().unwrap_or((0, 0));
                let c = st.curr.get(k).copied().unwrap_or_default();
                let a = st.angles.get(k).copied().unwrap_or(0);
                (o, ox, oy, c.x, c.y, a)
            })
            .collect();
        (f, members)
    }
}
