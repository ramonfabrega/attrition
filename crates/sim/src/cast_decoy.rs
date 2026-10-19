//! `SpellType::cast_create_decoy@00674370` as steps over a view (item 3320,
//! `docs/GOLDEN.md` §48): the General `g` copies the armed land squads
//! standing near it. [`cast_create_decoy_steps`] is the original's control
//! flow, call for call and in its order; [`DecoyView`] is the state it reads
//! and the callees it makes, which [`SimDecoy`] answers from the crate's
//! [`Sim`] and `sweep/spell_type_cast_create_decoy.rs` from the emulated
//! original's windows. The steps run the sweep's 1500 rows against
//! `tools/emu/sweep_spell_type_cast_create_decoy.py`.
//!
//! ```text
//! limit = (get_general_upgrade + 2) × 5
//! is(PORUS):   limit = (PORUS_DECOYS × limit) / 256
//! is(KUTOSOV): limit = KUTOSOV_DECOYS × limit / 100
//! made  = the player's captains already decoys
//! army  = Object::get_army(g)      when g is a computer's unit
//! for o from the player's last object down to its first:
//!     alive, a captain, not a decoy, of the land (`+0x218` 0), armed,
//!     type index outside PEASANTS..=SCHOLARSKOREAN, not a caravan, not
//!     is(MERCHANT), not `g` itself, and within `get_radius × 0xc0` of `g`:
//!         find_nearby_spot(g's type, g, 0x180, -1, ..)    nonzero: stop
//!         init_unit(who, its type, spot)                   negative: stop
//!         Army::add_unit                                   when army >= 0
//!         population: track_unit_type −1, the leader's three counters
//!         every figure: |= 1, mana_burn 0, the two slots, speed, armor
//!         made += 1; stop at `limit`
//! some made: the sound         none made: unpay_cost, mana handed back
//! ```
//!
//! The Porus and Kutosov multiples were a SEAM until the sweep (the
//! constants are `PORUS_DECOYS` 2/1 and `KUTOSOV_DECOYS` 300 % in
//! `rules.xml`). Still a seam, in [`SimDecoy`]: `is(PORUS)` and
//! `is(KUTOSOV)` are read as the type itself (as `hero_radius` reads
//! Kutosov), the leader's `+0x93c`, `+0x940` and `+0x808` counters, the
//! sound, `Type::unpay_cost`'s resources, and `update_speed` and
//! `update_armor`.

use crate::{Player, Pos, Sim};

/// `TypeIndex` rows `is` is asked.
pub(crate) const PORUS: i32 = 0x16f;
pub(crate) const KUTOSOV: i32 = 0x176;
pub(crate) const MERCHANT: i32 = 0x3d;

/// What `cast_create_decoy` reads and does, by the object's index in its
/// player's list (`o`); `nth` maps a list position to one.
pub trait DecoyView {
    /// `LeaderData::get_general_upgrade@006e0830` of the caster's leader.
    fn general_upgrade(&mut self) -> i32;
    /// `ObjectData::is(ty, 0)` of `o`: the vslot `+0xb8`, or the type's `+0x60`.
    fn is(&mut self, o: i32, ty: i32) -> bool;
    /// `GameAccess::constants->porus_decoys`, 8.8.
    fn porus_decoys(&self) -> i32;
    /// `GameAccess::constants->kutosov_decoys`, a percentage.
    fn kutosov_decoys(&self) -> i32;
    /// The player's object count and the object at a position of its list.
    fn count(&self) -> i32;
    fn nth(&self, k: i32) -> i32;
    /// `Object::flags & 1`.
    fn alive(&self, o: i32) -> bool;
    /// The object's vslot `+0xe8`.
    fn is_captain(&mut self, o: i32) -> bool;
    /// `Unit::unit_masks & 1`.
    fn is_decoy(&self, o: i32) -> bool;
    /// `Unit::unit_masks & 0x40000`: a computer's unit.
    fn is_computer(&self, o: i32) -> bool;
    /// `Object::get_army@00649d70`.
    fn army_of(&mut self, o: i32) -> i32;
    /// The type's `+0x218` is 0.
    fn land(&self, o: i32) -> bool;
    /// The unit's vslot `+0x120`.
    fn armed(&mut self, o: i32) -> bool;
    /// The type's `+0x4`.
    fn type_index(&self, o: i32) -> i32;
    /// `UnitData::is_caravan`, at the unit's vslot `+0xd0` or the type's `+0x130`.
    fn is_caravan(&mut self, o: i32) -> bool;
    /// The unit's position.
    fn pos(&self, o: i32) -> Pos;
    /// `HeroData::get_radius@00739e50` of the hero the caster names.
    fn hero_radius(&mut self, caster: i32) -> i32;
    /// `UnitType::find_nearby_spot@0061de70` asked for the caster's type
    /// and place; `None` is a nonzero answer.
    fn find_spot(&mut self, caster: i32, src: i32) -> Option<Pos>;
    /// `Objects::init_unit@0065e0c0` of the source's type; negative fails.
    fn init_unit(&mut self, src: i32, at: Pos) -> i32;
    /// `Army::add_unit@006f9f40`.
    fn army_add(&mut self, army: i32, o: i32);
    /// The type's `+0x2f0`.
    fn type_pop(&self, o: i32) -> i32;
    /// `Leader::track_unit_type@006e0dd0(type, -1, o)`.
    fn track_unit_type(&mut self, src: i32, o: i32);
    /// The leader's `+0x940 -= pop`, `+0x93c -= 1`, `+0x808 -= 1`.
    fn leader_counts(&mut self, pop: i32);
    /// `unit_masks |= 1` and `mana_burn = 0`.
    fn mark(&mut self, o: i32);
    /// Vslots `+0x15c(0)` and `+0x160`, `update_speed`, `update_armor`.
    fn refresh(&mut self, o: i32);
    /// The short at `+0x90`: the next figure or negative.
    fn o_down(&self, o: i32) -> i32;
    /// The sound, when the caster is the console's.
    fn sound(&mut self);
    /// `Type::unpay_cost(who, caster, -1)`, vslot `+0xd8`.
    fn unpay_cost(&mut self, caster: i32);
    fn mana_burn(&self, o: i32) -> i16;
    fn set_mana_burn(&mut self, o: i32, v: i16);
    /// The craft's `MANA` (`this+0x1d0`).
    fn spell_mana(&self) -> Option<i32>;
}

/// The original's body over a view; `g` is the caster's object.
pub fn cast_create_decoy_steps<V: DecoyView>(v: &mut V, g: i32) {
    let mut limit = v.general_upgrade().wrapping_add(2).wrapping_mul(5);
    if v.is(g, PORUS) {
        // `cdq; and $0xff; add; sar $8`: toward zero.
        limit = v.porus_decoys().wrapping_mul(limit) / 256;
    }
    if v.is(g, KUTOSOV) {
        limit = v.kutosov_decoys().wrapping_mul(limit) / 100;
    }
    let mut made = 0i32;
    for k in 0..v.count() {
        let o = v.nth(k);
        if v.alive(o) && v.is_captain(o) && v.is_decoy(o) {
            made += 1;
        }
    }
    let army = if v.is_computer(g) { v.army_of(g) } else { -1 };
    let mut any = false;
    'scan: {
        for k in (0..v.count()).rev() {
            let o = v.nth(k);
            if !v.alive(o) || !v.is_captain(o) || v.is_decoy(o) || !v.land(o) || !v.armed(o) {
                continue;
            }
            if (0x32..=0x35).contains(&v.type_index(o)) || v.is_caravan(o) || v.is(o, MERCHANT) {
                continue;
            }
            if o == g {
                continue;
            }
            let (a, b) = (v.pos(g), v.pos(o));
            let d = crate::world::vector_dist(a.x.wrapping_sub(b.x), a.y.wrapping_sub(b.y));
            if d >= v.hero_radius(g).wrapping_mul(0xc0) {
                continue;
            }
            let Some(spot) = v.find_spot(g, o) else {
                break 'scan;
            };
            let head = v.init_unit(o, spot);
            if head < 0 {
                break 'scan;
            }
            if army >= 0 {
                v.army_add(army, head);
            }
            any = true;
            if v.type_pop(head) != 0 {
                v.track_unit_type(o, head);
                let pop = if v.is_decoy(head) {
                    0
                } else {
                    v.type_pop(head)
                };
                v.leader_counts(pop);
            }
            let mut at = head;
            while at >= 0 {
                v.mark(at);
                v.refresh(at);
                at = v.o_down(at);
            }
            made += 1;
            if limit <= made {
                break 'scan;
            }
        }
    }
    if any {
        v.sound();
        return;
    }
    v.unpay_cost(g);
    let burn = v.mana_burn(g);
    if let Some(mana) = v.spell_mana() {
        let back = mana.min(i32::from(burn));
        v.set_mana_burn(g, (i32::from(burn) - back) as i16);
    }
}

/// The crate's [`Sim`] as the view: `o` is a unit id, a player's list
/// the units it owns in `index` order (the original's array).
pub(crate) struct SimDecoy<'a> {
    pub(crate) sim: &'a mut Sim,
    pub(crate) who: Player,
    pub(crate) list: Vec<usize>,
}

impl<'a> SimDecoy<'a> {
    pub(crate) fn new(sim: &'a mut Sim, g: usize) -> Self {
        let who = sim.units[g].owner;
        let mut list: Vec<usize> = (0..sim.units.len())
            .filter(|&u| sim.units[u].owner == who)
            .collect();
        list.sort_by_key(|&u| sim.units[u].index);
        SimDecoy { sim, who, list }
    }

    fn ty(&self, o: i32) -> usize {
        self.sim.units[o as usize].ty.expect("a typed unit")
    }
}

impl DecoyView for SimDecoy<'_> {
    fn general_upgrade(&mut self) -> i32 {
        self.sim.general_upgrade_level(self.who)
    }
    fn is(&mut self, o: i32, ty: i32) -> bool {
        match ty {
            MERCHANT => self.sim.is_merchant(o as usize),
            _ => self.sim.units[o as usize].type_index == ty,
        }
    }
    fn porus_decoys(&self) -> i32 {
        self.sim.tuning.porus_decoys
    }
    fn kutosov_decoys(&self) -> i32 {
        self.sim.tuning.kutosov_decoys
    }
    fn count(&self) -> i32 {
        self.list.len() as i32
    }
    fn nth(&self, k: i32) -> i32 {
        self.list[k as usize] as i32
    }
    fn alive(&self, o: i32) -> bool {
        self.sim.units[o as usize].alive()
    }
    fn is_captain(&mut self, o: i32) -> bool {
        self.sim.units[o as usize].captain
    }
    fn is_decoy(&self, o: i32) -> bool {
        self.sim.units[o as usize].decoy
    }
    fn is_computer(&self, o: i32) -> bool {
        self.sim.ai_driven(self.sim.units[o as usize].owner)
    }
    fn army_of(&mut self, o: i32) -> i32 {
        self.sim.army_of(o as usize).map_or(-1, |a| a as i32)
    }
    fn land(&self, o: i32) -> bool {
        // A unit without a type is no one's captain here.
        self.sim.units[o as usize].ty.is_some()
            && self.sim.unit_domain_of(o as usize) == crate::attrition::Domain::Land
    }
    fn armed(&mut self, o: i32) -> bool {
        self.sim
            .profile(crate::combat::Obj::Unit(o as usize))
            .attack
            != 0
    }
    fn type_index(&self, o: i32) -> i32 {
        self.sim.units[o as usize].type_index
    }
    fn is_caravan(&mut self, o: i32) -> bool {
        matches!(self.sim.units[o as usize].type_index, 0x3b | 0x3c)
    }
    fn pos(&self, o: i32) -> Pos {
        self.sim.units[o as usize].pos
    }
    fn hero_radius(&mut self, caster: i32) -> i32 {
        self.sim.hero_radius(caster as usize)
    }
    fn find_spot(&mut self, caster: i32, src: i32) -> Option<Pos> {
        let g = caster as usize;
        let ty = self.sim.decoy_search_type(g, self.ty(src));
        self.sim.find_nearby_spot_type_for(
            ty,
            g,
            self.sim.units[g].pos,
            0x180,
            -1,
            crate::movement::Angle(0),
        )
    }
    fn init_unit(&mut self, src: i32, at: Pos) -> i32 {
        let ty = self.ty(src);
        self.sim.init_unit(self.who, ty, at) as i32
    }
    fn army_add(&mut self, army: i32, o: i32) {
        self.sim.army_add_unit(self.who, army as usize, o as usize);
    }
    fn type_pop(&self, o: i32) -> i32 {
        self.sim.unit_types[self.ty(o)].price.pop
    }
    fn track_unit_type(&mut self, src: i32, _o: i32) {
        let ty = self.ty(src);
        self.sim.track_unit_type(self.who, ty, -1);
    }
    fn leader_counts(&mut self, _pop: i32) {}
    fn mark(&mut self, o: i32) {
        let unit = &mut self.sim.units[o as usize];
        unit.decoy = true;
        unit.mana_burn = 0;
    }
    fn refresh(&mut self, o: i32) {
        // `update_los`: term 12's one tile, on the cache.
        self.sim.update_los(o as usize);
    }
    fn o_down(&self, o: i32) -> i32 {
        self.sim.units[o as usize].o_down.map_or(-1, |x| x as i32)
    }
    fn sound(&mut self) {}
    fn unpay_cost(&mut self, _caster: i32) {}
    fn mana_burn(&self, o: i32) -> i16 {
        self.sim.units[o as usize].mana_burn
    }
    fn set_mana_burn(&mut self, o: i32, v: i16) {
        self.sim.units[o as usize].mana_burn = v;
    }
    fn spell_mana(&self) -> Option<i32> {
        self.sim
            .spell(crate::orders::spell::CREATE_DECOY)
            .map(|d| d.mana)
    }
}
