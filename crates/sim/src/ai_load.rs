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

/// How many weights a tech carries — `TechType::ai[11]`.
pub const AI_WEIGHTS: usize = 11;

/// The prerequisites of a type the tree does not carry: one of the 55 spells
/// (`craftrules.xml`) or the 122 bonuses (`rules.xml`'s `TECHBONUSES`).
/// `compute_ai_values` counts both as dependants and reads nothing else about
/// them.
pub type ExtraPreq = [Preq; 3];

/// `TechType::compute_ai_values@0066cdc0`, over every tech in the tree.
///
/// Writes `TypeDef::ai` for every tech entry. The pass is one loop per class
/// of dependant; it is written here as one loop over the *dependants*, which
/// is the same arithmetic — every contribution is an unconditional `+=` on the
/// tech's own array — at a fraction of the cost.
///
/// `units` and `builds` are the simulation's type tables, whose `tree` links
/// give each record its tree id; `spells` and `bonuses` are the two classes
/// the tree does not hold.
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
    let mut ai = vec![[0i32; AI_WEIGHTS]; n];

    // Every tech scores itself: an epoch by its line, anything else by the
    // building it is researched at. The Fort's arm falls through into the
    // military weight — see `docs/DATALAYER.md`.
    for (t, a) in ai.iter_mut().enumerate() {
        match tree.types[t].kind {
            Kind::Epoch { line, .. } => match line {
                crate::tech::Line::Science => {
                    a[4] += 1;
                    a[10] += 1;
                }
                crate::tech::Line::Civic => a[5] += 1,
                crate::tech::Line::Commerce => a[1] += 1,
                crate::tech::Line::Military => a[0] += 1,
            },
            k if k.is_tech() => {
                let w = tree.types[t].where_;
                if w.is_some() && w == tree.roles.temple {
                    a[5] += 2;
                } else if w.is_some() && w == tree.roles.fortx {
                    a[5] += 1;
                    a[0] += 1;
                }
            }
            _ => {}
        }
    }

    // `Type::find_preq`: the distinct techs this type names, at most three.
    let preqs_of = |tree: &TechTree, t: TypeId, num: usize| -> Vec<TypeId> {
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
    };

    // Units, without the twelve gaia types: the original's loop stops at
    // `BASE_GAIATYPES`.
    for u in units.iter().filter(|u| !u.gaia) {
        let Some(id) = u.tree else { continue };
        let r = u.cols.role;
        for t in preqs_of(tree, id, 2) {
            let a = &mut ai[t];
            if r & role::MILITARY != 0 {
                a[0] += 1;
            }
            if r & role::CITIZEN != 0 {
                a[1] += 1;
                a[9] += 1;
            }
            match u.combat.domain {
                Domain::Land => {
                    if r & role::MILITARY != 0 {
                        a[8] += 1;
                    }
                }
                d => {
                    if u.cols.carry != 0 {
                        a[3] += 1;
                    }
                    if d == Domain::Sea {
                        a[2] += 1;
                    } else {
                        a[6] += 1;
                    }
                }
            }
        }
    }

    // Buildings. `fundamental` is `from` walked to its root.
    let fundamental = |b: usize| -> usize {
        let mut cur = b;
        for _ in 0..builds.len() {
            match builds[cur].from {
                Some(f) => cur = f,
                None => break,
            }
        }
        cur
    };
    let is_dock = |tree: &TechTree, b: usize| -> bool {
        builds[b]
            .tree
            .is_some_and(|id| tree.roles.dock.is_some_and(|root| tree.is(id, root, false)))
    };
    let mut queued: Vec<(TypeId, usize, i32, i32)> = Vec::new();
    for (b, bt) in builds.iter().enumerate() {
        let Some(id) = bt.tree else { continue };
        let s = if bt.from.is_none() { 2 } else { 1 };
        let village = bt.tree.is_some_and(|x| {
            tree.roles
                .village
                .is_some_and(|root| tree.is(x, root, false))
        });
        let dock = is_dock(tree, b);
        let f = fundamental(b);
        let fb = &builds[f];
        for t in preqs_of(tree, id, 3) {
            let a = &mut ai[t];
            if village {
                a[1] += 1;
                if bt.ident == crate::build::Ident::Town {
                    a[5] += 1;
                    a[1] += 1;
                    queued.push((t, 1, 1, 0));
                }
            }
            if bt.from.is_none() {
                if bt.has(crate::build::flags::GATHER) {
                    a[1] += 2 * s;
                    a[9] += 2 * s;
                }
                if bt.has(crate::build::flags::RESEARCH_HERE) {
                    a[1] += 2 * s;
                    a[4] += 2 * s;
                    if enhancer(bt.ident) {
                        a[9] += 2 * s;
                    }
                }
                if dock {
                    a[2] += 1;
                    queued.push((t, 2, 1, -1));
                }
            }
            if dock {
                a[2] += 1;
                queued.push((t, 2, 1, 2));
            }
            if fb.has(crate::build::flags::GATHER) {
                a[1] += s;
                a[9] += s;
            } else if fb.has(crate::build::flags::RESEARCH_HERE) {
                a[1] += s;
                a[4] += s;
            } else if defensive(tree, builds, f) || fb.attack != 0 {
                a[0] += s;
            } else if enhancer(fb.ident) {
                a[1] += 8;
                a[9] += 8;
            }
            if fb.has(crate::build::flags::TRAINS) {
                for u in units.iter().filter(|u| !u.gaia) {
                    if u.garrison.trained_at != Some(b) {
                        continue;
                    }
                    if u.cols.role & role::MILITARY_OR_CARRY == 0 {
                        a[1] += 1;
                    } else {
                        a[0] += s;
                    }
                }
            }
        }
    }

    // Techs, goods, and the two classes the tree does not carry.
    for t in 0..n {
        let kind = tree.types[t].kind;
        if !(kind.is_tech() || matches!(kind, Kind::Good)) {
            continue;
        }
        let num = if matches!(kind, Kind::Good) { 2 } else { 3 };
        for p in preqs_of(tree, t, num) {
            let a = &mut ai[p];
            if kind.is_tech() {
                a[4] += 1;
                if matches!(kind, Kind::Epoch { .. }) {
                    a[4] += 1;
                    a[0] += 1;
                }
                if matches!(kind, Kind::Age(_)) {
                    a[4] += 2;
                    a[0] += 2;
                }
            } else {
                let s = if t < 6 { 4 } else { 1 };
                a[9] += s;
                a[1] += s;
            }
        }
    }
    for pq in spells {
        for p in extra_preqs(tree, pq) {
            ai[p][0] += 4;
            ai[p][1] += 2;
        }
    }
    for (i, pq) in bonuses.iter().enumerate() {
        for p in extra_preqs(tree, pq) {
            if i == 0 {
                ai[p][1] += 1;
            }
            ai[p][1] += 1;
            ai[p][4] += 2;
        }
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

/// `is(TOWER) || is(FORTX) || is(AIRBASE)` on a building record.
fn defensive(tree: &TechTree, builds: &[BuildType], b: usize) -> bool {
    let Some(id) = builds[b].tree else {
        return false;
    };
    [tree.roles.tower, tree.roles.fortx, tree.roles.airbase]
        .into_iter()
        .flatten()
        .any(|root| tree.is(id, root, false))
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
