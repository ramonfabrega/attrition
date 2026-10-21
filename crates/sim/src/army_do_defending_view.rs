//! **`Army::do_defending@006f4070`'s steps** (item 3396): the defend order of
//! an army, as a pure walk over an [`ArmyDoDefendingView`]. The sweep's port
//! is a view over the emulated original's windows.
//!
//! The crate's `Sim::do_defending` (army.rs, §10) is the same decision tree
//! over the structured state — an army's `target`, `city` and `muster` rather
//! than the record's words — so it cannot be handed raw bytes; this walk is
//! the original's own, call for call, and is **not wired**. It is what the
//! sweep holds the original to; the crate's method is read against it by eye
//! (the same predicates, in the same order) and not by a diff.
//!
//! The army record's words: `+0x4` status (`0x2` marching, `0x4` marched,
//! `0x10` forming, `0x20` defending), `+0x20` the city index, `+0x30`/`+0x34`
//! the target's unit id and owner (−1 for none), `+0x38`/`+0x3c` the army's
//! position, `+0x48`/`+0x4c` the muster cell, `+0x94` the owner (a short).
//!
//! One thing the original does that no port of it would: it calls
//! `ObjectsData::find_city` with the `ecx` that `find_besieged_city` left
//! behind (the decompiler prints it as an uninitialised `this_01`), and
//! pushes the same register as the fifth argument. The sweep's stub leaves
//! `ecx` as it was (the army), so the row's log names it `m+0x0`; the real
//! value is unestablished.

/// `Army::count`'s first argument.
pub const COUNT_ATTACK: i32 = 2;
/// `ObjectsData::find_city`'s search index: friendly cities.
pub const SEARCH_FRIENDLY: i32 = 1;
/// The status bits this walk touches.
pub const MARCHING: u32 = 0x02;
/// Cleared at entry.
pub const MARCHED: u32 = 0x04;
/// Set when the army has somewhere to muster.
pub const FORMING: u32 = 0x10;
/// Cleared whenever the army has a target or a city.
pub const DEFENDING: u32 = 0x20;

/// What `Army::do_defending` asks of the army, the object and city tables
/// and its callees.
pub trait ArmyDoDefendingView {
    /// The word at `+0x4`.
    fn status(&self) -> u32;
    /// Write the word at `+0x4`.
    fn set_status(&mut self, v: u32);
    /// The short at `+0x94`.
    fn who(&self) -> i32;
    /// `+0x30` / `+0x34`: the target's id and owner.
    fn target(&self) -> (i32, i32);
    /// Write `+0x30` / `+0x34`.
    fn set_target(&mut self, id: i32, who: i32);
    /// `+0x38` / `+0x3c`.
    fn pos(&self) -> (i32, i32);
    /// Write `+0x38` / `+0x3c`.
    fn set_pos(&mut self, x: i32, y: i32);
    /// Write `+0x20`: the city index.
    fn set_city(&mut self, c: i32);
    /// The word at `+0x20`.
    fn city(&self) -> i32;
    /// Write `+0x48` / `+0x4c`.
    fn set_muster(&mut self, x: i32, y: i32);
    /// `div_3_table[v >> 8]`.
    fn div3(&self, v: i32) -> i32;
    /// `Army::count(COUNT_ATTACK, 0)`.
    fn count_attack(&mut self) -> i32;
    /// `Army::close`.
    fn close(&mut self);
    /// `Army::is_engaged`.
    fn is_engaged(&mut self) -> bool;
    /// The byte at `+8` of unit `id` of player `who`.
    fn unit_flags(&self, who: i32, id: i32) -> u8;
    /// The short at `+0x72` of what the unit's virtual slot `+0xac` answers.
    fn unit_city_index(&mut self, who: i32, id: i32) -> i32;
    /// The byte at `+4` of city `idx` of player `who`.
    fn city_flags(&self, who: i32, idx: i32) -> u8;
    /// City `idx` of player `who`: the words at `+0xc` and `+0x10`.
    fn city_pos(&self, who: i32, idx: i32) -> (i32, i32);
    /// City `idx` of player `who`: the short at `+8`.
    fn city_building(&self, who: i32, idx: i32) -> i32;
    /// `Army::find_besieged_city`.
    fn find_besieged_city(&mut self) -> bool;
    /// `ObjectsData::find_city(x, y, SEARCH_FRIENDLY, who, .., 0x200, ..)`.
    fn find_city(&mut self, x: i32, y: i32) -> i32;
    /// `WorldData::restrict(&x, &y)` on the army's own position.
    fn restrict(&mut self);
    /// `Army::find_muster_spot(building, who, 1)`.
    fn find_muster_spot(&mut self, building: i32, who: i32) -> bool;
}

/// `Army::do_defending@006f4070`.
pub fn army_do_defending_steps<V: ArmyDoDefendingView>(v: &mut V) {
    if v.count_attack() < 5 {
        v.close();
        return;
    }
    if v.is_engaged() {
        return;
    }
    let st = v.status() & !MARCHED;
    v.set_status(st);
    let (id, owner) = v.target();
    if st & MARCHING != 0 && id >= 0 && owner >= 0 {
        let f = v.unit_flags(owner, id);
        if f & 1 != 0 {
            if f & 0x20 != 0 {
                let ci = v.unit_city_index(owner, id);
                if ci >= 0 && v.city_flags(owner, ci) & 3 == 3 {
                    return;
                }
            }
            v.set_target(-1, -1);
            let (x, y) = v.pos();
            let (mx, my) = (v.div3(x), v.div3(y));
            v.set_muster(mx, my);
            let st = v.status();
            v.set_status((st & !(MARCHING | DEFENDING)) | FORMING);
            return;
        }
    }
    if v.find_besieged_city() {
        let st = v.status();
        v.set_status(st | MARCHING);
        return;
    }
    let (x, y) = v.pos();
    let c = v.find_city(x, y);
    v.set_target(-1, -1);
    v.set_city(c);
    if c >= 0 {
        let st = v.status();
        v.set_status(st & !(MARCHING | DEFENDING));
        let who = v.who();
        let (cx, cy) = v.city_pos(who, c);
        v.set_pos(cx, cy.wrapping_add(0x300));
        v.restrict();
        let building = v.city_building(who, v.city());
        if v.find_muster_spot(building, who) {
            let st = v.status();
            v.set_status(st | FORMING);
        } else {
            v.close();
        }
        return;
    }
    let (x, y) = v.pos();
    let (mx, my) = (v.div3(x), v.div3(y));
    v.set_muster(mx, my);
    let st = v.status();
    v.set_status((st & !(MARCHING | DEFENDING)) | FORMING);
}
