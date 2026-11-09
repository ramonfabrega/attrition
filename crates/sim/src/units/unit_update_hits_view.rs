//! **`Unit::update_hits@0060e930`'s steps**, as a walk over a
//! [`UnitHitsView`]: [`crate::Sim::unit_hits`] is this walk over a unit type
//! of the port, and the sweep's port is this walk over the emulated
//! original's windows (`Build::update_hits` and `Sim::update_hits` of
//! `city.rs` are the building's, a namesake).
//!
//! Order of the terms: a unit that is not a captain hands the whole call to
//! another object (`redirect`); otherwise `Object::update_hits`
//! (`base_hits`), the American marines, the Copper and Bananas leaders'
//! percentages, the Iroquois', the Dutch ages, the Spy table (which
//! *replaces*), the General's `level² × hits / 2`, the Nubians', the supply
//! upgrade's addend; then the result is stored in `+0x20` and down the
//! unit's `+0x90` chain of the owner's list. The patch gate on the Dutch
//! hero exemption is `patch_version() > 8`.

/// The constants record's words this function reads.
#[derive(Clone, Copy, Debug, Default)]
pub struct UnitHitsK {
    pub marine: i32,
    pub copper: i32,
    pub bananas: i32,
    pub iroquois: i32,
    pub dutch: i32,
    pub nubian: i32,
    pub spy: [i32; 4],
    pub supply: [i32; 4],
}

/// What `Unit::update_hits` asks of the unit, its type, its leader and the
/// unit list.
pub trait UnitHitsView {
    /// `UnitData::is_captain` (vslot `+0xe8`).
    fn is_captain(&mut self) -> bool;
    /// A non-captain's whole answer: the captain's own `update_hits`.
    fn redirect(&mut self) -> i32;
    /// `Object::update_hits`.
    fn base_hits(&mut self) -> i32;
    /// `+0x6c & 0x200`.
    fn is_marine(&mut self) -> bool;
    /// The leader's `+0x6da6 & 4` or `+0x6dce & 4`.
    fn copper_bit(&mut self) -> bool;
    /// The unit type's `+0x218`.
    fn type_218(&mut self) -> i32;
    /// The unit type's `+0x40`, the trainer.
    fn trainer(&mut self) -> i32;
    /// The leader's `+0x6da8 & 0x80` or `+0x6dd0 & 0x80`.
    fn bananas_bit(&mut self) -> bool;
    /// `LeaderData::has_tribe_bonus`.
    fn has_tribe(&mut self, n: i32) -> bool;
    /// The unit type's `+0x1e8` (`attack`), as a word.
    fn attack_word(&mut self) -> i32;
    /// `ObjectData::is(n, 0)` (vslot `+0xb8`).
    fn is_type(&mut self, n: i32) -> bool;
    /// The unit type's `+4`.
    fn type_index(&mut self) -> i32;
    /// `UnitData::is_caravan` (vslot `+0xd0`).
    fn is_caravan(&mut self) -> bool;
    /// `UnitData::is_supply` (vslot `+0xcc`).
    fn is_supply(&mut self) -> bool;
    /// `Game::get_patch_version`.
    fn patch_version(&mut self) -> i32;
    /// `UnitData::is_gov_hero` (vslot `+200`).
    fn is_gov_hero(&mut self) -> bool;
    /// The leader's word at `+0xdc` of the record at `+0x6eb8`, the ages
    /// as stored (xor `0x62766`).
    fn ages_stored(&mut self) -> i32;
    /// `LeaderData::get_spy_upgrade`.
    fn spy_upgrade(&mut self) -> i32;
    /// `UnitData::is_hero` (vslot `+0xc4`).
    fn is_hero(&mut self) -> bool;
    /// `LeaderData::get_general_upgrade`.
    fn general_upgrade(&mut self) -> i32;
    /// `LeaderData::get_supply_upgrade`.
    fn supply_upgrade(&mut self) -> i32;
    /// The short at `+0x90`: the next unit of the chain, negative for none.
    fn link_of_self(&mut self) -> i32;
    /// The short at `+0x90` of the unit at index `i` of the owner's list.
    fn link_of(&mut self, i: i32) -> i32;
    /// Store `h` at `+0x20` of `this` (`None`) or of the list's unit `i`.
    fn store(&mut self, at: Option<i32>, h: i32);
    /// `this+0x20`, read back.
    fn stored(&mut self) -> i32;
}

fn pct(p: i32, h: i32) -> i32 {
    p.wrapping_add(100).wrapping_mul(h) / 100
}

/// `Unit::update_hits@0060e930`.
pub fn unit_update_hits<V: UnitHitsView>(k: &UnitHitsK, v: &mut V) -> i32 {
    if !v.is_captain() {
        return v.redirect();
    }
    let mut h = v.base_hits();
    if v.is_marine() {
        h = h.wrapping_add(k.marine);
    }
    if v.copper_bit() && (v.type_218() == 1 || matches!(v.trainer(), 0x1ae | 0x1af)) {
        h = pct(k.copper, h);
    }
    if v.bananas_bit() {
        h = pct(k.bananas, h);
    }
    if v.has_tribe(0x12)
        && k.iroquois != 0
        && v.trainer() == 0x1ab
        && (v.attack_word() != 0 || v.is_type(0x45))
    {
        h = pct(k.iroquois, h);
    }
    if v.has_tribe(0x16) && k.dutch != 0 {
        let ti = v.type_index();
        let kind = matches!(ti, 0x3d | 0x3e | 400) || v.is_caravan() || v.is_supply();
        if kind && !(v.patch_version() > 8 && v.is_gov_hero()) {
            h = h.wrapping_add((v.ages_stored() ^ 0x62766).wrapping_mul(k.dutch));
        }
    }
    if v.is_type(0x3a) {
        let level = v.spy_upgrade().clamp(0, 3) as usize;
        h = k.spy[level];
    }
    if v.is_hero() {
        let g = v.general_upgrade();
        h = h.wrapping_add(g.wrapping_mul(g).wrapping_mul(h) / 2);
    }
    let ti = v.type_index();
    if (matches!(ti, 0x3d | 0x3e | 400) || v.is_caravan()) && v.has_tribe(4) && k.nubian != 0 {
        h = pct(k.nubian, h);
    }
    if v.is_supply() {
        let s = v.supply_upgrade();
        h = h.wrapping_add(k.supply.get(s as usize).copied().unwrap_or(0));
    }
    let mut i = v.link_of_self();
    v.store(None, h);
    while i >= 0 {
        v.store(Some(i), h);
        i = v.link_of(i);
    }
    v.stored()
}
