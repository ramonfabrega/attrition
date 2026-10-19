//! The loader's half of the production AI: the derived type words that no
//! column carries, and which every producer reads.
//!
//! `docs/DATALAYER.md`, "The derived words no column carries", is the
//! specification and says how each was established. Four things live here:
//!
//! - [`role`], the bits of `UnitTypeData+0x2c8`, and [`determine_roles`] —
//!   `UnitType::determine_roles@0061c320`, run at the end of `UnitType::init`
//!   over five columns.
//! - [`UnitCols`], the four words a [`crate::UnitType`] carries so that the
//!   producers can stop guessing: `unit_flags`, `unit_flags2`, `cat`, `carry`,
//!   and the `role` word itself.
//! - [`flags2`], the six lineage bits of `UnitType::init_final_flags@0061dc70`
//!   plus the caster bit `SpellType::init` seeds.
//! - [`compute_ai_values`], `TechType::compute_ai_values@0066cdc0` and
//!   `add_preq_ai@0066c350` — the eleven per-tech weights `research_techs`
//!   multiplies its base by, derived from what each tech unlocks.
//!
//! All four are checked field for field against the original's own loaded
//! values, which its `log_data` writes into a `DUMP_ALL` type dump:
//! `rondata --types <dump>` compares 364 roles, 364 `unit_flags2`, 129
//! `build_flags` and 85 × 11 weights and expects no difference.
//!
//! Why here and not in the loader: the words are *fields of the simulation's
//! types*, a test that hand-builds a type wants to set them the way the
//! original would, and none of the derivations needs a file — they are
//! functions of columns the caller already has.

use crate::tech::{Kind, Preq, Setup, TechTree, TypeId};
use crate::{UnitType, attrition::Domain, build::BuildType};

/// The bits of the `role` word (`UnitTypeData+0x2c8`).
///
/// The producers test six of them; the rest are derived and carried so that
/// the word logged by the original can be compared with ours whole.
pub mod role {
    /// A ranged military type — `max_range != 0`.
    pub const RANGED: u32 = 0x400;
    /// A melee military type. Mutually exclusive with [`RANGED`].
    pub const MELEE: u32 = 0x1;
    /// `is(HOPLITES)`, on a land `Foot` military type.
    pub const HOPLITE: u32 = 0x2;
    /// A `Mounted` land type: both bits, always together.
    pub const MOUNTED: u32 = 0xc;
    /// `is(SCOUT)` on land, `is(BARK)` at sea — the bit that keeps a type out
    /// of the census's invader count.
    pub const SCOUT: u32 = 0x10;
    /// A land `Foot` military type.
    pub const FOOT: u32 = 0x800;
    /// An air type, military or not.
    pub const AIR: u32 = 0x1000;
    /// A military sea type.
    pub const SEA_MILITARY: u32 = 0x2000;
    /// A military air type.
    pub const AIR_MILITARY: u32 = 0x4000;
    /// `CARRY != 0`: the type carries other units.
    pub const CARRY: u32 = 0x8000;
    /// The military bit — `combat_role`.
    pub const MILITARY: u32 = 0x10000;
    /// Not military: the `else` of the military test.
    pub const CIVILIAN: u32 = 0x100;
    /// One of the four citizen/scholar ids.
    pub const CITIZEN: u32 = 0x200;
    /// A land `Foot` military type that is not a hoplite.
    pub const INFANTRY: u32 = 0x100_000;
    /// A land type.
    pub const LAND: u32 = 0x40000;
    /// A sea type.
    pub const SEA: u32 = 0x80000;
    /// `create_units`' trainer test: `role & 0x180c0 == 0` scores a building's
    /// output as economic rather than military (`docs/DATALAYER.md`).
    pub const MILITARY_OR_CARRY: u32 = 0x180c0;
}

/// The bits of `unit_flags` (`UnitTypeData+0x2b4`) the producers read. The
/// word is the `FLAGS` column folded a letter to a bit, plus `0x10` from
/// `init_final_flags`.
pub mod uflags {
    /// `b` — "Unit is a horse-drawn cart type thing". The one exemption from
    /// the instant turn from a standstill (`Sim::turning_for`): a cart does
    /// not pivot.
    pub const CART: u32 = 0x2;
    /// `e` — a sea transport.
    pub const TRANSPORT: u32 = 0x10;
    /// `f` — `unitrules.xml`'s own legend: "Unit flies like a helicopter".
    /// Exactly three types carry it — `Helicopter` and the two
    /// `Attack Helicopter`s — and all three are `<DOMAIN>Air`. It is the
    /// exemption in `UnitData::is_plane@0046ce40`
    /// (`domain == 2 && !(unit_flags & 0x20)`), so a helicopter is **not** a
    /// plane: it is halted, stanced and given group orders like a ground
    /// unit (`docs/GROUPS.md` §7, §8, §6.6).
    pub const HELICOPTER: u32 = 0x20;
    /// `c` — the class the AI caps by city count.
    pub const CITY_CAPPED: u32 = 0x4;
    /// `d` — the second such class.
    pub const CITY_CAPPED2: u32 = 0x8;
    /// `p` — never produced, and never marks its building a trainer.
    pub const NO_PRODUCE: u32 = 0x8000;
    /// `h` — priced by `TypeData::get_cost@00664090`'s train arm even when
    /// the player does not own it, never by its research (`:425`). 92 of
    /// the 364 records: the citizens, the starting lines, the carts.
    pub const NO_RESEARCH_PRICE: u32 = 0x80;
    /// `r` — `is_siege`.
    pub const SIEGE: u32 = 0x20000;
    /// `v` — **fires on the move** (the Dragoon's `majv`, the Horse
    /// Archer's `mhv`). `Unit::do_move@005f7b30:60` calls
    /// `Unit::cavarch_fight` for such a type, and `Unit::move_step` walks
    /// it under `CHAR_ATTACKWALK` while it is aimed (`docs/AI.md` §134,
    /// [`crate::cavarch`]).
    pub const FIRES_ON_MOVE: u32 = 0x20_0000;
    /// `UnitTypeData::blocks_while_dead` (`unit_flags & 0x800000`, the type's
    /// vslot `+0x120`): a combat death of such a type leaves its occupancy
    /// disc in the world (`Object::close@00647160`'s arm that skips
    /// `remove_from_world`), so the Tank's wreck is something to walk
    /// around (`docs/GOLDEN.md` §125).
    pub const BLOCKS_WHILE_DEAD: u32 = 0x80_0000;
    /// `t` — `unitrules.xml`'s own legend: "Unit is a tank".
    /// `UnitTypeData::is_tank@00470450` reads it, the type's vslot `+0x110`
    /// (the PDB's `ObjectTypeData` method list), and a land pusher will not
    /// shove one (`docs/COLLISION.md` §13.3).
    pub const TANK: u32 = 0x80000;
    /// `z` — `unitrules.xml`'s own legend: "Unit rocks left/right when it
    /// attacks (attack1 is left, attack2 is right)". The eighteen ship
    /// types carry it, and it is the one arm of `Unit::fight`'s swing
    /// animation that picks its slot from an angle (`docs/ANIM.md` §6.2).
    pub const ROCKS: u32 = 0x200_0000;
    /// `g` — `unitrules.xml`'s own legend: "Unit attacks sideways (most
    /// ships)". `Unit::fight@005fd4d0:698–714` turns such a unit a quarter
    /// turn off the bearing to its target (`docs/COMBAT.md` §49).
    pub const SIDEWAYS: u32 = 0x40;
    /// `w` — `unitrules.xml`'s own legend: "Unit \"strafes\" targets". The
    /// Fighter line carries it and the Bomber does not.
    /// `Unit::pitch_aircraft@005e8de0` reads it at `0x5e9009`: a striking
    /// plane of the kind wants half its cruising altitude
    /// (`docs/ORDERS.md` §39).
    pub const STRAFES: u32 = 0x40_0000;
    /// `o` — "Unit is cloaked" (the Spy, the submarines). `UnitData::is_seen`'s
    /// stealth arm tests the type's `0x4000` (`docs/GOLDEN.md` §101).
    pub const CLOAKED: u32 = 0x4000;
    /// `s` — "Unit is stealth (cloaked when not moving)": the Explorer and
    /// the commando lines. The same arm tests the type's `0x40000` and an
    /// empty order list.
    pub const STEALTH: u32 = 0x4_0000;
}

/// The bits of `unit_flags2` (`+0x2b8`).
pub mod uflags2 {
    /// `is(MACHINEGUN)` or `is(FLAMETHROWER)`.
    pub const EMPLACED: u32 = 0x1;
    /// `is_spellcaster` — seeded by `craftrules.xml` and walked down the
    /// `graft`/`from` chains.
    pub const CASTER: u32 = 0x2;
    /// `needs_packing` — the sim's `Profile::packs`.
    pub const PACKS: u32 = 0x4;
    /// `is_caravan`.
    pub const CARAVAN: u32 = 0x8;
    /// `is(SCOUT)`.
    pub const SCOUT: u32 = 0x10;
    /// `is(GENERAL)`.
    pub const GENERAL: u32 = 0x20;
    /// The supply and government-hero lineages.
    pub const SUPPLY_OR_HERO: u32 = 0x40;
    /// `create_units`' special-forces class: `& 0x60`.
    pub const SPECIAL_FORCES: u32 = 0x60;
}

/// The unit `CAT` column — `rules.xml`'s `<CATEGORIES id="unit_cats">`, in
/// file order. `determine_roles` reads three of the nine.
pub mod cat {
    pub const FOOT: i32 = 0;
    pub const MOUNTED: i32 = 1;
    pub const COMMAND: i32 = 4;
    pub const CIVILIAN: i32 = 5;
}

/// The derived words a unit type carries. Every field is the original's,
/// under the original's name; a type built by hand leaves them zero, which
/// reads as "no flags, `Foot`, carries nothing" and gives a `role` of zero.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UnitCols {
    /// `+0x2b4`, the `FLAGS` column folded.
    pub unit_flags: u32,
    /// `+0x2b8`, six lineage bits and the caster bit.
    pub unit_flags2: u32,
    /// `+0x14`, the `CAT` column as an index into `unit_cats`.
    pub cat: i32,
    /// `+0x2d4`, the `CARRY` column.
    pub carry: i32,
    /// `+0x2c8`, [`determine_roles`]' word.
    pub role: u32,
    /// `+0x2e0`, the `RESEARCH_PREMIUM_COST` column through
    /// `String::fraction(…, 0x100)` (`UnitType::init@0061ab50:606`–`613`):
    /// 8.8, so the shipped `2` is 512. What researching the type costs over
    /// one of it — `get_cost`'s research arm, [`crate::cost::Research`].
    pub research_premium_cost: i32,
}

impl UnitCols {
    /// `role & r`.
    pub const fn is(&self, r: u32) -> bool {
        self.role & r != 0
    }
    /// `unit_flags & f`.
    pub const fn flag(&self, f: u32) -> bool {
        self.unit_flags & f != 0
    }
    /// `unit_flags2 & f`.
    pub const fn flag2(&self, f: u32) -> bool {
        self.unit_flags2 & f != 0
    }
}

/// What `determine_roles` reads: five columns and three lineage tests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RoleFacts {
    /// One of the four citizen/scholar ids `0x32..=0x35` — **by identity**,
    /// which is record index `0..=3`.
    pub citizen_id: bool,
    pub domain: Domain,
    /// The `CAT` column; see [`cat`].
    pub cat: i32,
    /// `ATTACK`, in tenths as the loader stores it.
    pub attack: i32,
    /// The *stored* `max_range` — after `FLAGS k` has zeroed it.
    pub max_range: i32,
    /// `CARRY`.
    pub carry: i32,
    /// `is(SCOUT)`, read only for a land type.
    pub scout: bool,
    /// `is(BARK)`, read only for a sea type.
    pub bark: bool,
    /// `is(HOPLITES)`, read only for a land `Foot` military type.
    pub hoplite: bool,
}

/// `UnitType::determine_roles@0061c320` — the `role` word, whole.
///
/// The order matters and is the original's: the citizen bit first, then the
/// domain, then `carry`, then the military test whose `else` is the civilian
/// bit. The hoplite and infantry bits are **exclusive**, not a pair.
pub fn determine_roles(f: &RoleFacts) -> u32 {
    let mut r = 0;
    if f.citizen_id {
        r = role::CITIZEN;
    }
    match f.domain {
        Domain::Air => r |= role::AIR,
        Domain::Land => {
            r |= role::LAND;
            if f.scout {
                r |= role::SCOUT;
            }
            if f.cat == cat::MOUNTED {
                r |= role::MOUNTED;
            }
        }
        Domain::Sea => {
            r |= role::SEA;
            if f.bark {
                r |= role::SCOUT;
            }
        }
    }
    if f.carry != 0 {
        r |= role::CARRY;
    }
    let military =
        f.attack != 0 && f.cat != cat::CIVILIAN && f.cat != cat::COMMAND && r & role::CITIZEN == 0;
    if !military {
        return r | role::CIVILIAN;
    }
    r |= role::MILITARY;
    match f.domain {
        Domain::Sea => r |= role::SEA_MILITARY,
        Domain::Air => r |= role::AIR_MILITARY,
        Domain::Land => {
            if f.cat == cat::FOOT {
                r |= role::FOOT;
                r |= if f.hoplite {
                    role::HOPLITE
                } else {
                    role::INFANTRY
                };
            }
        }
    }
    r | if f.max_range != 0 {
        role::RANGED
    } else {
        role::MELEE
    }
}

/// What `init_final_flags` tests: the lineages, by name, that the six
/// `unit_flags2` bits and the one `unit_flags` bit are derived from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Flags2Facts {
    /// `is(MACHINEGUN)`.
    pub machinegun: bool,
    /// `is(FLAMETHROWER)`.
    pub flamethrower: bool,
    /// `is(CATAPULT)`, `is(FLAMINGARROW)`, `is(FISHERMEN)`, `is(KATYUSHA)`.
    pub packs_lineage: bool,
    /// The **exact ids** `MERCHANT`, `MERCHANTDUTCH`, `FURTRAPPER` — the one
    /// place `init_final_flags` compares an id instead of walking a lineage.
    pub trader_id: bool,
    /// `is(SUPPLYWAGON)`, `is(BASE_GOV_HEROTYPES)`, `is(THEMONARCH)`,
    /// `is(THECITIZEN)`.
    pub supply_or_hero: bool,
    /// `is(GENERAL)`.
    pub general: bool,
    /// `is(CARA)` or `is(MERCHANTFLEET)`.
    pub caravan: bool,
    /// `is(SCOUT)`.
    pub scout: bool,
    /// `is(TRANSPORTBARGE)` or `is(MERCHANTFLEET)` — this one sets a bit in
    /// `unit_flags`, not `unit_flags2`.
    pub transport: bool,
}

/// `UnitType::init_final_flags@0061dc70`: `(unit_flags2, unit_flags extra)`.
///
/// The caster bit ([`uflags2::CASTER`]) is **not** here — it is seeded by the
/// craft table and propagated by [`spread_casters`].
pub fn flags2(f: &Flags2Facts) -> (u32, u32) {
    let mut v = 0;
    if f.packs_lineage || f.trader_id || f.machinegun {
        v |= uflags2::PACKS;
    }
    if f.supply_or_hero {
        v |= uflags2::SUPPLY_OR_HERO;
    }
    if f.general {
        v |= uflags2::GENERAL;
    }
    if f.caravan {
        v |= uflags2::CARAVAN;
    }
    if f.scout {
        v |= uflags2::SCOUT;
    }
    if f.machinegun || f.flamethrower {
        v |= uflags2::EMPLACED;
    }
    (v, if f.transport { uflags::TRANSPORT } else { 0 })
}

/// `UnitType::init_spellcasters@0061aae0`, over every type in id order.
///
/// `seeded[i]` is "this type is a craft's `FROM` or `FROM2`"; a type inherits
/// the bit when the walk self → `graft` → `from` reaches a type that already
/// has it. Marks [`uflags2::CASTER`] in place.
pub fn spread_casters(
    cols: &mut [UnitCols],
    seeded: &[bool],
    graft: &[Option<usize>],
    from: &[Option<usize>],
) {
    for (i, &s) in seeded.iter().enumerate().take(cols.len()) {
        if s {
            cols[i].unit_flags2 |= uflags2::CASTER;
        }
    }
    for i in 0..cols.len() {
        if cols[i].flag2(uflags2::CASTER) {
            continue;
        }
        let mut cur = i;
        for _ in 0..cols.len() {
            if cols[cur].flag2(uflags2::CASTER) {
                cols[i].unit_flags2 |= uflags2::CASTER;
                break;
            }
            let mut holder = cur;
            if let Some(g) = graft.get(cur).copied().flatten() {
                cur = g;
                holder = g;
                if cols[cur].flag2(uflags2::CASTER) {
                    cols[i].unit_flags2 |= uflags2::CASTER;
                    break;
                }
            }
            match from.get(holder).copied().flatten() {
                Some(f) => cur = f,
                None => break,
            }
        }
    }
}

/// A class of type that depends on a tech: the six `compute_ai_values` loops
/// ask in this order, one `find_preq` an element.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Unit,
    Build,
    Tech,
    Good,
    Spell,
    Bonus,
}

/// The type ids `compute_ai_values` names. A building is asked `is(root)`;
/// the tech's own `WHERE` is compared with [`Root::Temple`] and
/// [`Root::Fort`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Root {
    Village = 0x19e,
    Granary = 0x1a7,
    Lumbermill = 0x1a8,
    Smelter = 0x1a9,
    Refinery = 0x1aa,
    Dock = 0x1b0,
    Temple = 0x1b5,
    Tower = 0x1b7,
    Fort = 0x1bb,
    Airbase = 0x1bf,
}

/// What `TechType::compute_ai_values@0066cdc0` reads, per tech (`this`). The
/// original computes one tech's eleven words by asking every unit, building,
/// tech, good, spell and bonus whether it names the tech; a building is
/// addressed by its position in the building table, a unit by its position
/// in the non-gaia units.
pub trait AiValuesView {
    /// How many of a class the loop walks.
    fn count(&self, c: Class) -> usize;
    /// `find_preq(this.id)` on element `i` of the class.
    fn depends(&mut self, c: Class, i: usize) -> bool;
    /// `this.ai[w] += amt`, a 16-bit wrapping add.
    fn add(&mut self, w: usize, amt: i32);
    /// `add_preq_ai(this, idx, amt, depth)`.
    fn add_preq_ai(&mut self, idx: usize, amt: i32, depth: i32);
    /// `this` is an epoch tech.
    fn self_is_epoch(&mut self) -> bool;
    /// `this.line` (`+0x14`): 0 military, 1 civic, 2 commerce, 3 science.
    fn self_line(&self) -> i32;
    /// `this.where == root`.
    fn self_where_is(&self, r: Root) -> bool;
    /// `UnitTypeData+0x2c8`.
    fn unit_role(&self, u: usize) -> u32;
    /// `UnitTypeData+0x218`: 0 land, 1 sea, else air.
    fn unit_domain(&self, u: usize) -> i32;
    /// `UnitTypeData+0x2d4 != 0`.
    fn unit_carry(&self, u: usize) -> bool;
    /// `UnitTypeData+0x40 == building b`.
    fn unit_trained_at(&self, u: usize, b: usize) -> bool;
    /// `building.is(root)`.
    fn build_is(&mut self, b: usize, r: Root) -> bool;
    /// The loop's own id is `TOWN` (0x19f).
    fn build_is_town(&self, b: usize) -> bool;
    /// The loop's own id is one of the four gather enhancers.
    fn build_is_enhancer(&self, b: usize) -> bool;
    /// `building.from < 0`.
    fn build_from_root(&self, b: usize) -> bool;
    /// The gatherer flag (`+0x2c0 & 0x40`).
    fn build_gather(&mut self, b: usize) -> bool;
    /// `+0x2c0 & 0x8000000`.
    fn build_research_here(&self, b: usize) -> bool;
    /// `+0x1e8 != 0`.
    fn build_attack(&self, b: usize) -> bool;
    /// `+0x2c0 < 0`.
    fn build_trains(&self, b: usize) -> bool;
    /// `BuildTypeData::fundamental_type` of `b`, or `b` itself when it has no `FROM`.
    fn fundamental(&mut self, b: usize) -> usize;
    /// `BuildTypeData::basic_type` of `f`.
    fn basic(&mut self, f: usize) -> usize;
    /// `is_epoch_type` of tech `i`.
    fn tech_is_epoch(&mut self, i: usize) -> bool;
    /// `is_age_type` of tech `i`.
    fn tech_is_age(&mut self, i: usize) -> bool;
    /// The type id of good `i`.
    fn good_id(&self, i: usize) -> usize;
}

/// `TechType::compute_ai_values@0066cdc0` for one tech: the original's body,
/// over a view of what it reads.
pub fn compute_ai_values_steps<V: AiValuesView>(v: &mut V) {
    // Every tech scores itself: an epoch by its line, anything else by the
    // building it is researched at. The Fort's arm falls through into the
    // military weight — see `docs/DATALAYER.md`.
    if v.self_is_epoch() {
        match v.self_line() {
            3 => {
                v.add(4, 1);
                v.add(10, 1);
            }
            1 => v.add(5, 1),
            2 => v.add(1, 1),
            0 => v.add(0, 1),
            _ => {}
        }
    } else if v.self_where_is(Root::Temple) {
        v.add(5, 2);
    } else if v.self_where_is(Root::Fort) {
        v.add(5, 1);
        v.add(0, 1);
    }

    for u in 0..v.count(Class::Unit) {
        if !v.depends(Class::Unit, u) {
            continue;
        }
        let r = v.unit_role(u);
        if r & role::MILITARY != 0 {
            v.add(0, 1);
        }
        if r & role::CITIZEN != 0 {
            v.add(1, 1);
            v.add(9, 1);
        }
        match v.unit_domain(u) {
            0 => {
                if r & role::MILITARY != 0 {
                    v.add(8, 1);
                }
            }
            d => {
                if v.unit_carry(u) {
                    v.add(3, 1);
                }
                if d == 1 {
                    v.add(2, 1);
                } else {
                    v.add(6, 1);
                }
            }
        }
    }

    for b in 0..v.count(Class::Build) {
        if !v.depends(Class::Build, b) {
            continue;
        }
        let root = v.build_from_root(b);
        let s = if root { 2 } else { 1 };
        if v.build_is(b, Root::Village) {
            v.add(1, 1);
            if v.build_is_town(b) {
                v.add(5, 1);
                v.add(1, 1);
                v.add_preq_ai(1, 1, 0);
            }
        }
        if root {
            if v.build_gather(b) {
                v.add(1, 2 * s);
                v.add(9, 2 * s);
            }
            if v.build_research_here(b) {
                v.add(1, 2 * s);
                v.add(4, 2 * s);
                if v.build_is_enhancer(b) {
                    v.add(9, 2 * s);
                }
            }
            if v.build_is(b, Root::Dock) {
                v.add(2, 1);
                v.add_preq_ai(2, 1, -1);
            }
        }
        if v.build_is(b, Root::Dock) {
            v.add(2, 1);
            v.add_preq_ai(2, 1, 2);
        }
        let f = v.fundamental(b);
        if v.build_gather(f) {
            v.add(1, s);
            v.add(9, s);
        } else if v.build_research_here(f) {
            v.add(1, s);
            v.add(4, s);
        } else if v.build_is(f, Root::Tower)
            || v.build_is(f, Root::Fort)
            || v.build_is(f, Root::Airbase)
            || v.build_attack(f)
        {
            v.add(0, s);
        } else if v.build_is(f, Root::Granary)
            || v.build_is(f, Root::Lumbermill)
            || v.build_is(f, Root::Smelter)
            || v.build_is(f, Root::Refinery)
        {
            v.add(1, 8);
            v.add(9, 8);
        }
        let basic = v.basic(f);
        if v.build_trains(basic) {
            for u in 0..v.count(Class::Unit) {
                if !v.unit_trained_at(u, b) {
                    continue;
                }
                if v.unit_role(u) & role::MILITARY_OR_CARRY == 0 {
                    v.add(1, 1);
                } else {
                    v.add(0, s);
                }
            }
        }
    }

    for i in 0..v.count(Class::Tech) {
        if !v.depends(Class::Tech, i) {
            continue;
        }
        v.add(4, 1);
        if v.tech_is_epoch(i) {
            v.add(4, 1);
            v.add(0, 1);
        }
        if v.tech_is_age(i) {
            v.add(4, 2);
            v.add(0, 2);
        }
    }
    for i in 0..v.count(Class::Good) {
        if !v.depends(Class::Good, i) {
            continue;
        }
        let s = if v.good_id(i) < 6 { 4 } else { 1 };
        v.add(9, s);
        v.add(1, s);
    }
    for i in 0..v.count(Class::Spell) {
        if v.depends(Class::Spell, i) {
            v.add(0, 4);
            v.add(1, 2);
        }
    }
    for i in 0..v.count(Class::Bonus) {
        if v.depends(Class::Bonus, i) {
            if i == 0 {
                v.add(1, 1);
            }
            v.add(1, 1);
            v.add(4, 2);
        }
    }
}

/// How many weights a tech carries — `TechType::ai[11]`.
pub const AI_WEIGHTS: usize = 11;

/// The prerequisites of a type the tree does not carry: one of the 55 spells
/// (`craftrules.xml`) or the 122 bonuses (`rules.xml`'s `TECHBONUSES`).
/// `compute_ai_values` counts both as dependants and reads nothing else about
/// them.
pub type ExtraPreq = [Preq; 3];

/// `Type::find_preq`: the distinct techs a type names in its first `num`
/// prerequisite slots.
fn preqs_of(tree: &TechTree, setup: &Setup, t: TypeId, num: usize) -> Vec<TypeId> {
    let mut v: Vec<TypeId> = Vec::with_capacity(num);
    for i in 0..num {
        if let Preq::Of(p) = tree.get_preq(setup, None, t, i)
            && tree.types[p].kind.is_tech()
            && !v.contains(&p)
        {
            v.push(p);
        }
    }
    v
}

/// The simulation's tables, read by [`compute_ai_values_steps`] one tech at a
/// time.
struct TreeView<'a> {
    tree: &'a TechTree,
    units: &'a [&'a UnitType],
    builds: &'a [BuildType],
    /// The techs each element names, by [`Class`].
    deps: &'a [Vec<Vec<TypeId>>; 6],
    techs: &'a [TypeId],
    goods: &'a [TypeId],
    /// The tech being computed.
    t: TypeId,
    out: [i32; AI_WEIGHTS],
    queued: Vec<(usize, i32, i32)>,
}

impl TreeView<'_> {
    fn is_root(&self, b: usize, root: Option<TypeId>) -> bool {
        match (self.builds[b].tree, root) {
            (Some(id), Some(r)) => self.tree.is(id, r, false),
            _ => false,
        }
    }
}

impl AiValuesView for TreeView<'_> {
    fn count(&self, c: Class) -> usize {
        match c {
            Class::Unit => self.units.len(),
            Class::Build => self.builds.len(),
            _ => self.deps[c as usize].len(),
        }
    }
    fn depends(&mut self, c: Class, i: usize) -> bool {
        self.deps[c as usize][i].contains(&self.t)
    }
    fn add(&mut self, w: usize, amt: i32) {
        self.out[w] += amt;
    }
    fn add_preq_ai(&mut self, idx: usize, amt: i32, depth: i32) {
        self.queued.push((idx, amt, depth));
    }
    fn self_is_epoch(&mut self) -> bool {
        matches!(self.tree.types[self.t].kind, Kind::Epoch { .. })
    }
    fn self_line(&self) -> i32 {
        match self.tree.types[self.t].kind {
            Kind::Epoch { line, .. } => line.index() as i32,
            _ => -1,
        }
    }
    fn self_where_is(&self, r: Root) -> bool {
        let def = &self.tree.types[self.t];
        if !def.kind.is_tech() || def.where_.is_none() {
            return false;
        }
        match r {
            Root::Temple => def.where_ == self.tree.roles.temple,
            Root::Fort => def.where_ == self.tree.roles.fortx,
            _ => false,
        }
    }
    fn unit_role(&self, u: usize) -> u32 {
        self.units[u].cols.role
    }
    fn unit_domain(&self, u: usize) -> i32 {
        match self.units[u].combat.domain {
            Domain::Land => 0,
            Domain::Sea => 1,
            _ => 2,
        }
    }
    fn unit_carry(&self, u: usize) -> bool {
        self.units[u].cols.carry != 0
    }
    fn unit_trained_at(&self, u: usize, b: usize) -> bool {
        self.units[u].garrison.trained_at == Some(b)
    }
    fn build_is(&mut self, b: usize, r: Root) -> bool {
        use crate::build::Ident;
        let roles = &self.tree.roles;
        match r {
            Root::Village => self.is_root(b, roles.village),
            Root::Dock => self.is_root(b, roles.dock),
            Root::Tower => self.is_root(b, roles.tower),
            Root::Fort => self.is_root(b, roles.fortx),
            Root::Airbase => self.is_root(b, roles.airbase),
            Root::Granary => self.builds[b].ident == Ident::Granary,
            Root::Lumbermill => self.builds[b].ident == Ident::Lumbermill,
            Root::Smelter => self.builds[b].ident == Ident::Smelter,
            Root::Refinery => self.builds[b].ident == Ident::Refinery,
            Root::Temple => false,
        }
    }
    fn build_is_town(&self, b: usize) -> bool {
        self.builds[b].ident == crate::build::Ident::Town
    }
    fn build_is_enhancer(&self, b: usize) -> bool {
        enhancer(self.builds[b].ident)
    }
    fn build_from_root(&self, b: usize) -> bool {
        self.builds[b].from.is_none()
    }
    fn build_gather(&mut self, b: usize) -> bool {
        self.builds[b].has(crate::build::flags::GATHER)
    }
    fn build_research_here(&self, b: usize) -> bool {
        self.builds[b].has(crate::build::flags::RESEARCH_HERE)
    }
    fn build_attack(&self, b: usize) -> bool {
        self.builds[b].attack != 0
    }
    fn build_trains(&self, b: usize) -> bool {
        self.builds[b].has(crate::build::flags::TRAINS)
    }
    fn fundamental(&mut self, b: usize) -> usize {
        let mut cur = b;
        for _ in 0..self.builds.len() {
            match self.builds[cur].from {
                Some(f) => cur = f,
                None => break,
            }
        }
        cur
    }
    fn basic(&mut self, f: usize) -> usize {
        f
    }
    fn tech_is_epoch(&mut self, i: usize) -> bool {
        matches!(self.tree.types[self.techs[i]].kind, Kind::Epoch { .. })
    }
    fn tech_is_age(&mut self, i: usize) -> bool {
        matches!(self.tree.types[self.techs[i]].kind, Kind::Age(_))
    }
    fn good_id(&self, i: usize) -> usize {
        self.goods[i]
    }
}

/// `TechType::compute_ai_values@0066cdc0`, over every tech in the tree.
///
/// Writes `TypeDef::ai` for every tech entry, by running
/// [`compute_ai_values_steps`] on each; the view answers "does this
/// dependant name the tech" from the preqs of each class, worked out once.
///
/// `units` and `builds` are the simulation's type tables, whose `tree` links
/// give each record its tree id; `spells` and `bonuses` are the two classes
/// the tree does not hold. The twelve gaia types are outside the unit loop:
/// the original's stops at `BASE_GAIATYPES`.
pub fn compute_ai_values(
    tree: &mut TechTree,
    setup: &Setup,
    units: &[UnitType],
    builds: &[BuildType],
    spells: &[ExtraPreq],
    bonuses: &[ExtraPreq],
) {
    for d in &mut tree.types {
        d.ai = [0; AI_WEIGHTS];
    }
    let n = tree.types.len();
    let units: Vec<&UnitType> = units.iter().filter(|u| !u.gaia).collect();
    let techs: Vec<TypeId> = (0..n).filter(|&t| tree.types[t].kind.is_tech()).collect();
    let goods: Vec<TypeId> = (0..n)
        .filter(|&t| matches!(tree.types[t].kind, Kind::Good))
        .collect();
    let deps: [Vec<Vec<TypeId>>; 6] = [
        units
            .iter()
            .map(|u| u.tree.map_or(Vec::new(), |id| preqs_of(tree, setup, id, 2)))
            .collect(),
        builds
            .iter()
            .map(|b| b.tree.map_or(Vec::new(), |id| preqs_of(tree, setup, id, 3)))
            .collect(),
        techs.iter().map(|&t| preqs_of(tree, setup, t, 3)).collect(),
        goods.iter().map(|&t| preqs_of(tree, setup, t, 2)).collect(),
        spells.iter().map(|pq| extra_preqs(tree, pq)).collect(),
        bonuses.iter().map(|pq| extra_preqs(tree, pq)).collect(),
    ];
    let mut ai = vec![[0i32; AI_WEIGHTS]; n];
    let mut queued: Vec<(TypeId, usize, i32, i32)> = Vec::new();
    for &t in &techs {
        let mut v = TreeView {
            tree,
            units: &units,
            builds,
            deps: &deps,
            techs: &techs,
            goods: &goods,
            t,
            out: [0; AI_WEIGHTS],
            queued: Vec::new(),
        };
        compute_ai_values_steps(&mut v);
        ai[t] = v.out;
        queued.extend(v.queued.into_iter().map(|(i, a, d)| (t, i, a, d)));
    }

    // `add_preq_ai`, once the direct scores are in: it adds into the *computed
    // tech's* prerequisites, so it is order-independent with everything above.
    for (t, idx, amt, depth) in queued {
        add_preq_ai(tree, setup, &mut ai, t, idx, amt, depth);
    }

    for (t, a) in ai.into_iter().enumerate() {
        tree.types[t].ai = a.map(|v| v as i16);
    }
}

/// The four gather-enhancer buildings, which `compute_ai_values` names by id.
const fn enhancer(i: crate::build::Ident) -> bool {
    matches!(
        i,
        crate::build::Ident::Granary
            | crate::build::Ident::Lumbermill
            | crate::build::Ident::Smelter
            | crate::build::Ident::Refinery
    )
}

/// The techs an [`ExtraPreq`] names.
fn extra_preqs(tree: &TechTree, pq: &ExtraPreq) -> Vec<TypeId> {
    let mut v = Vec::new();
    for q in pq {
        if let Preq::Of(p) = *q
            && tree.types[p].kind.is_tech()
            && !v.contains(&p)
        {
            v.push(p);
        }
    }
    v
}

/// `TechType::add_preq_ai@0066c350`: add `amt` to `ai[idx]` of `t`'s own
/// prerequisites, and recurse `depth` levels — `0` none, `−1` until the chain
/// ends.
fn add_preq_ai(
    tree: &TechTree,
    setup: &Setup,
    ai: &mut [[i32; AI_WEIGHTS]],
    t: TypeId,
    idx: usize,
    amt: i32,
    depth: i32,
) {
    if depth < -64 {
        return;
    }
    for i in 0..3 {
        let Preq::Of(p) = tree.get_preq(setup, None, t, i) else {
            continue;
        };
        if !tree.types[p].kind.is_tech() {
            continue;
        }
        ai[p][idx] += amt;
        if depth != 0 {
            add_preq_ai(tree, setup, ai, p, idx, amt, depth - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts() -> RoleFacts {
        RoleFacts {
            domain: Domain::Land,
            ..RoleFacts::default()
        }
    }

    #[test]
    fn the_citizen_s_word_is_the_original_s() {
        // Unit type 0x32, `role 262912` in the type dump: land, citizen,
        // civilian — an `ATTACK` of 40 that the `Civilian` category refuses.
        let r = determine_roles(&RoleFacts {
            citizen_id: true,
            cat: cat::CIVILIAN,
            attack: 40,
            ..facts()
        });
        assert_eq!(r, 0x40300);
        assert_eq!(r, role::LAND | role::CITIZEN | role::CIVILIAN);
    }

    #[test]
    fn a_hoplite_takes_the_hoplite_bit_and_not_the_infantry_one() {
        let h = determine_roles(&RoleFacts {
            cat: cat::FOOT,
            attack: 60,
            hoplite: true,
            ..facts()
        });
        assert!(h & role::HOPLITE != 0 && h & role::INFANTRY == 0);
        let s = determine_roles(&RoleFacts {
            cat: cat::FOOT,
            attack: 60,
            ..facts()
        });
        assert!(s & role::INFANTRY != 0 && s & role::HOPLITE == 0);
        assert!(h & role::MELEE != 0 && s & role::MELEE != 0);
    }

    #[test]
    fn command_and_civilian_categories_are_never_military() {
        for c in [cat::COMMAND, cat::CIVILIAN] {
            let r = determine_roles(&RoleFacts {
                cat: c,
                attack: 90,
                ..facts()
            });
            assert!(r & role::MILITARY == 0);
            assert!(r & role::CIVILIAN != 0);
        }
    }

    #[test]
    fn the_sea_and_air_military_bits_come_with_the_domain() {
        let sea = determine_roles(&RoleFacts {
            domain: Domain::Sea,
            attack: 40,
            max_range: 4,
            ..facts()
        });
        assert_eq!(
            sea,
            role::SEA | role::MILITARY | role::SEA_MILITARY | role::RANGED
        );
        let air = determine_roles(&RoleFacts {
            domain: Domain::Air,
            attack: 40,
            ..facts()
        });
        assert_eq!(
            air,
            role::AIR | role::MILITARY | role::AIR_MILITARY | role::MELEE
        );
    }

    #[test]
    fn a_carrier_takes_the_carry_bit_whatever_else_it_is() {
        let r = determine_roles(&RoleFacts {
            domain: Domain::Sea,
            carry: 8,
            ..facts()
        });
        assert!(r & role::CARRY != 0 && r & role::CIVILIAN != 0);
    }

    #[test]
    fn the_caster_bit_walks_down_a_from_chain() {
        // Militia → Minuteman → Partisan: only the root is a craft's owner.
        let mut cols = vec![UnitCols::default(); 3];
        let from = [None, Some(0), Some(1)];
        let graft = [None, None, None];
        spread_casters(&mut cols, &[true, false, false], &graft, &from);
        assert!(cols.iter().all(|c| c.flag2(uflags2::CASTER)));
    }

    /// A tree with one age, one epoch and one plain tech, a unit that needs
    /// the age and a gather building that needs the plain tech.
    fn small_tree() -> (TechTree, Vec<UnitType>, Vec<BuildType>) {
        use crate::build::{BuildType, Ident, flags};
        use crate::tech::{Kind, Line, Preq, TypeDef};

        let mut tree = TechTree::new();
        let age = tree.add(TypeDef::new("Classical Age", Kind::Age(0)));
        let epoch = tree.add(TypeDef::new(
            "Science 1",
            Kind::Epoch {
                line: Line::Science,
                level: 0,
            },
        ));
        let plain = tree.add(TypeDef::new("Agriculture", Kind::Plain));
        let mut ud = TypeDef::unit("Hoplite", crate::tech::UnitTraits::default());
        ud.preq[0] = Preq::Of(age);
        let unit_id = tree.add(ud);
        let mut bd = TypeDef::new(
            "Farm",
            Kind::Building {
                auto: false,
                wonder: false,
            },
        );
        bd.preq[0] = Preq::Of(plain);
        let build_id = tree.add(bd);
        // The epoch names the age, so the age gains an epoch dependant.
        tree.types[epoch].preq[0] = Preq::Of(age);

        let unit = UnitType {
            tree: Some(unit_id),
            cols: UnitCols {
                role: role::LAND | role::MILITARY | role::FOOT | role::INFANTRY | role::MELEE,
                ..UnitCols::default()
            },
            ..UnitType::default()
        };
        let build = BuildType {
            ident: Ident::Farm,
            tree: Some(build_id),
            flags: flags::GATHER,
            ..BuildType::default()
        };
        (tree, vec![unit], vec![build])
    }

    #[test]
    fn the_weights_count_what_a_tech_unlocks() {
        let (mut tree, units, builds) = small_tree();
        compute_ai_values(&mut tree, &Setup::STANDARD, &units, &builds, &[], &[]);
        let age = 0;
        let plain = 2;
        // The age is named by a military unit (`ai[0]`, `ai[8]`) and by an
        // epoch tech (`ai[4] + 2`, `ai[0] + 1`).
        assert_eq!(tree.types[age].ai[0], 2);
        assert_eq!(tree.types[age].ai[8], 1);
        assert_eq!(tree.types[age].ai[4], 2);
        // The plain tech is named by a rootless gather building: `2s` with
        // `s = 2`, then the fundamental arm's `s` again.
        assert_eq!(tree.types[plain].ai[1], 6);
        assert_eq!(tree.types[plain].ai[9], 6);
        // `ai[7]` is never written by anything.
        assert!(tree.types.iter().all(|d| d.ai[7] == 0));
    }

    #[test]
    fn a_gaia_type_is_outside_the_unit_loop() {
        let (mut tree, mut units, builds) = small_tree();
        units[0].gaia = true;
        compute_ai_values(&mut tree, &Setup::STANDARD, &units, &builds, &[], &[]);
        // Only the epoch's contribution to the age is left.
        assert_eq!(tree.types[0].ai[0], 1);
        assert_eq!(tree.types[0].ai[8], 0);
    }

    #[test]
    fn a_spell_and_the_first_bonus_score_where_the_original_puts_them() {
        use crate::tech::Preq;
        let (mut tree, units, builds) = small_tree();
        let plain = 2;
        let spell = [Preq::Of(plain), Preq::None, Preq::None];
        let bonus = [Preq::Of(plain), Preq::None, Preq::None];
        compute_ai_values(
            &mut tree,
            &Setup::STANDARD,
            &units,
            &builds,
            &[spell],
            &[bonus, bonus],
        );
        // The building's 6 plus the spell's 2, the first bonus's 2 and the
        // second's 1; `ai[0]` is the spell's 4 alone.
        assert_eq!(tree.types[plain].ai[1], 6 + 2 + 2 + 1);
        assert_eq!(tree.types[plain].ai[0], 4);
        assert_eq!(tree.types[plain].ai[4], 4);
    }

    #[test]
    fn the_caster_bit_does_not_walk_up() {
        let mut cols = vec![UnitCols::default(); 3];
        let from = [None, Some(0), Some(1)];
        let graft = [None, None, None];
        spread_casters(&mut cols, &[false, false, true], &graft, &from);
        assert_eq!(cols.iter().filter(|c| c.flag2(uflags2::CASTER)).count(), 1);
    }
}
