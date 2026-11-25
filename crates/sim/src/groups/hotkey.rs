//! **A player's hot-key groups, as a hot key writes them** —
//! `CommandPackage::process_hotkey@009474d0` (item 4270,
//! `docs/CLIENT_INPUT.md` §22).
//!
//! `HotKeyGroups` is its own array of `0xa2` (162) groups, nine players of
//! eighteen (`HotKeyGroups::clear@00715230`), apart from the pool and
//! dumped apart from it, under `HOTKEYGROUPS` (`GameLog::dump_hotkey_groups
//! @0092f920`). A `hotkey` command names one by its index, `who × 0x12`
//! plus the key (`Console::on_key_down@007cc210`).
//!
//! - **`clear` 0, with a group**: `HotKeyGroups::copy_group@00715120`
//!   copies the package's group — the pool slot `process_group@0094a0c0`'s
//!   `Groups::push_group(…, 1)` handed back — into the slot: `who`, `num`,
//!   `ox`/`oy`, `o_dist`, `o_angle`, `buildings`, `speed` and the six
//!   per-member arrays, with `stamp` the frame; then `valid` (`+0x9d8`) is
//!   0. Nothing else of the slot is written.
//! - **`clear` 1**: `num` 0 and `valid` 0, and with `valid` the camera's
//!   point and zoom. The floats are carried as their bits.
//!
//! A building's object rally also adds its trainee to the target's hot
//! keys (`HotKeyGroups::copy`, item 4646).
//! SEAM: a slot's other writers (`HotKeyGroups::kill`,
//! `find_group`; the capital the original holds in its F1 slot from the
//! first frame) are not carried. A slot no hot key wrote reads `None`.

use crate::groups::group::{Group, GroupState};
use crate::{Player, Sim};

/// One hot-key group as a hot key last wrote it: `copy_group`'s fields,
/// and the camera fields `process_hotkey` writes itself.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HotKeySlot {
    pub who: u8,
    pub num: i32,
    pub ox: i32,
    pub oy: i32,
    pub o_dist: i32,
    pub o_angle: i32,
    pub buildings: bool,
    pub role: u32,
    pub new_speed: i32,
    pub speed: i32,
    pub stamp: i64,
    /// `+0x8cc`: the members' object numbers.
    pub list: Vec<i16>,
    pub off: Vec<(i32, i32)>,
    pub curr: Vec<(i32, i32)>,
    pub angles: Vec<i8>,
    /// `+0x9d8`.
    pub valid: i32,
    /// `+0x9d0`, `+0x9d4`: −1.0's bits until a screen group writes them.
    pub loc: (u32, u32),
    /// `+0x9dc`.
    pub zoom: i32,
}

/// A hot key's fields (`HotKeyCommand`): the slot, `clear`, `valid`, the
/// camera's point as bits, and its zoom.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HotKey {
    pub group: i32,
    pub clear: i32,
    pub valid: i32,
    pub loc: (u32, u32),
    pub zoom: i32,
}

/// The array's length, `HotKeyGroups::clear`'s `0xa2`.
pub const HOTKEY_GROUPS: i32 = 0xa2;

impl Sim {
    /// A trainee inherits each of its live rally target's unit hot keys.
    /// A hot key keeps its own offsets and membership independently of the
    /// command group in the pool. Adding a unit leaves its pool link alone.
    pub(crate) fn copy_rally_hotkeys(&mut self, target: i32, player: i32, trainee: usize) {
        let who = self.units[trainee].owner;
        let Ok(o) = i16::try_from(target) else { return };
        if player != i32::from(who) || self.unit_by_o(who, o).is_none() {
            return;
        }
        let first = u16::from(who) * 18;
        for key in first..first + 18 {
            let Some(slot) = self.orders.hotkeys.get(&key) else {
                continue;
            };
            if slot.who != who || slot.buildings || slot.num == 0 || !slot.list.contains(&o) {
                continue;
            }
            let mut updated = slot.clone();
            let mut group = Group::stack(who);
            // Group::get_num drops dead members and shifts their offsets.
            for index in (0..updated.list.len()).rev() {
                if let Some(unit) = self.unit_by_o(who, updated.list[index]) {
                    group.list.push(unit);
                } else {
                    updated.list.remove(index);
                    updated.off.remove(index);
                    updated.curr.remove(index);
                    updated.angles.remove(index);
                }
            }
            group.list.reverse();
            if slot.num < 4 {
                updated.role = self.group_find_role(&group.list);
                updated.speed = self.group_compute_speed(&group);
                updated.new_speed = updated.speed;
            }
            let retained = group.list.len();
            self.group_add(&mut group, trainee);
            let added = &group.list[retained..];
            if !added.is_empty() {
                updated
                    .list
                    .extend(added.iter().map(|&u| self.units[u].index));
                updated.off.resize(group.list.len(), (0, 0));
                updated.curr.resize(group.list.len(), (0, 0));
                updated.angles.resize(group.list.len(), 0);
                // Adding a follower calls get_num again. If dead members
                // left a small group, that call normalizes its living role.
                if added.len() > 1 && retained + 1 < 4 {
                    updated.role = self.group_find_role(&group.list);
                } else {
                    updated.role |= self.group_find_role(added);
                }
                updated.speed = self.group_compute_speed(&group);
                updated.new_speed = updated.speed;
                updated.stamp = self.frame;
            }
            updated.num = updated.list.len() as i32;
            self.orders.hotkeys.insert(key, updated);
        }
    }

    /// `process_hotkey` for `who`'s package: with `clear` 0, the package's
    /// pool slot (the one `who` pushed last, `Groups::last_group`) is copied
    /// into slot `k.group`; with `clear` 1, the slot is emptied and, with
    /// `valid`, given the camera. Returns whether a slot was written.
    pub fn process_hotkey(&mut self, who: Player, k: HotKey) -> bool {
        if !(0..HOTKEY_GROUPS).contains(&k.group) {
            return false;
        }
        let g = k.group as u16;
        if k.clear != 0 {
            let s = self.orders.hotkeys.entry(g).or_default();
            s.num = 0;
            s.valid = 0;
            if k.valid != 0 {
                s.valid = 1;
                s.loc = k.loc;
                s.zoom = k.zoom;
            }
            return true;
        }
        let pool = self.orders.last_group[who as usize];
        let (state, list, buildings) = match self.pool_building_group(who, pool) {
            Some((st, o)) => (st.clone(), o, true),
            None => match self.pool_state(who, pool) {
                Some(st) => (st.clone(), self.pool_list(who, pool), false),
                None => return false,
            },
        };
        let frame = self.frame;
        let s = self.orders.hotkeys.entry(g).or_default();
        copy_group(s, who, &state, list, buildings, frame);
        s.valid = 0;
        true
    }

    /// Hot-key group `g` as a hot key last wrote it; `None` for a slot no
    /// hot key wrote (the SEAM above).
    pub fn hotkey_slot(&self, g: u16) -> Option<&HotKeySlot> {
        self.orders.hotkeys.get(&g)
    }
}

/// `HotKeyGroups::copy_group@00715120`: the source's `who`, `num`, origin,
/// distance, angle, `buildings`, `speed` and arrays; `stamp` the frame.
fn copy_group(
    s: &mut HotKeySlot,
    who: Player,
    g: &GroupState,
    list: Vec<i16>,
    buildings: bool,
    frame: i64,
) {
    let n = list.len();
    s.who = who;
    s.num = n as i32;
    s.ox = g.o.x;
    s.oy = g.o.y;
    s.o_dist = g.o_dist;
    s.o_angle = g.o_angle.0;
    s.buildings = buildings;
    s.speed = g.speed;
    s.stamp = frame;
    s.off = (0..n)
        .map(|k| g.off.get(k).copied().unwrap_or((0, 0)))
        .collect();
    s.curr = (0..n)
        .map(|k| g.curr.get(k).map_or((0, 0), |p| (p.x, p.y)))
        .collect();
    s.angles = (0..n)
        .map(|k| g.angles.get(k).copied().unwrap_or(0))
        .collect();
    s.list = list;
}
