//! Units — `Leader::create_units@006c40a0`, `Leader::upgrade_units@006c6430`,
//! `Leader::produce_unit@006cb9e0`, `Leader::queued_units@006ce000`,
//! `Leader::check_income@006cc800`, `Leader::unit_prod_value@006cc580`;
//! the specification is `~/ghidra-projects/reports/ai/create-units.md`
//! (ratified in `docs/AI.md` §11).
//!
//! `create_units` is not "a pass over peasants then a pass over scouts": it
//! is one pass over every unit type, per city, and the type's `role` word
//! picks the branch that values it. Every branch ends in the same tail — the
//! count-cap multiplier, the civilian ceiling, `check_income`, `make_me` —
//! so the module reads as gates, a branch, a tail.
//!
//! Two sites draw from the sync stream, both `Random::get(0, 0xffff)` used
//! as `% 3`: the matchup bias in `create_units`' military gate (one per
//! (city, military type) reaching it) and the same bias in `upgrade_units`
//! (one per eligible type). Neither draws on difficulty 2. The order is the
//! specification; the tests count the draws.
//!
//! **Seams.** Several `UnitTypeData` columns the original reads are not on
//! [`crate::UnitType`] yet — `unit_flags`, `unit_flags2`, `role`, `carry`,
//! `cat` — and neither are armies or the leader-flag that says "human".
//! Every place one is read goes through a private `seam_*`/`named_*` helper
//! documented at its definition, which answers as a plain unit with no flags
//! would. They are listed in the worker's report.

use crate::ai::Census;
use crate::ai_load::{role, uflags, uflags2};
use crate::build::{self, Ident, flags};
use crate::economy::RESOURCES;
use crate::orders::Worker;
use crate::tech::{self, TypeId};
use crate::world::Terrain;
use crate::{Player, Sim, combat};

/// The two matchup-bias draw sites, under the original's own offsets. Both
/// are `Random::get(0, 0xffff)` taken `% 3` over `unit_prod_value`, and the
/// module doc above says which branch each sits in; the mark is what lets
/// the trace's sequence name them instead of the sweep they run under
/// (`crate::mark`, `rondata::trace::SITES`). The addresses are the return
/// addresses of the two `call Random::get` in the listing — `6c46dd` and
/// `6c69cf`, both five bytes long.
pub const SITE_UNIT_BIAS: &str = "Leader::create_units+0x642";
pub const SITE_UPGRADE_BIAS: &str = "Leader::upgrade_units+0x5a4";

/// The goods, in `TypeIndex` order — the order `econ`, `income` and the
/// gather-slot arrays are kept in.
const FOOD: usize = 0;
const KNOWLEDGE: usize = 3;

/// `BUILD_FLAGS` digit `5` — `BuildTypeData::is_military_trainer`, `+0x2c0
/// & 0x40000000`. `build::flags::parse` already folds it; it has no name in
/// [`crate::build::flags`] because no other mechanic reads it.
const MILITARY_TRAINER: u32 = 0x4000_0000;

/// The `role` word (`UnitTypeData+0x2c8`, `UnitType::determine_roles`) and
/// the type columns `create_units` reads, gathered once per (city, type).
///
/// `determine_roles` derives the word at load from the domain, `cat`,
/// `attack` and `carry`; what this reconstructs is exactly the six bits the
/// producers test — `0x200` citizen/scholar, `0x10` scout, `0x8000` carry,
/// `0x10000` military, `0x40000` land, `0x80000` sea (air is neither).
#[derive(Clone, Copy, Debug)]
struct Facts {
    /// The index into [`crate::Sim::unit_types`].
    rec: usize,
    /// `role & 0x200`.
    citizen: bool,
    /// `role & 0x10000` — the loader's `combat_role`.
    military: bool,
    /// `role & 0x40000`, `& 0x80000`; air is neither.
    land: bool,
    sea: bool,
    air: bool,
    /// `role & 0x10`.
    scout: bool,
    /// `role & 0x8000`: the type carries other units.
    carry: bool,
    /// `is_siege` (`unit_flags & 0x20000`).
    siege: bool,
    /// `is_caravan` (`unit_flags2 & 8`) and `TypeData::is_merchant`.
    caravan: bool,
    merchant: bool,
    /// `TypeData::is_worker`, `is_peasant`, `is_scholar`.
    worker: bool,
    peasant: bool,
    scholar: bool,
    /// `+0x1e8 attack`, in tenths.
    attack: i32,
    /// `+0x2f0 control_cost` — the `POP` column.
    control_cost: i32,
    /// `get_age(t)`.
    age: i32,
    /// `is_missile`.
    missile: bool,
    /// `unit_flags & 4`, `& 8`, `& 0x8000`; `unit_flags2 & 0x20`, `& 0x40`.
    flag_c: bool,
    flag_d: bool,
    flag_p: bool,
    special_forces: bool,
    special_forces_elite: bool,
}

impl Sim {
    // ---- the type facts, and the seams under them ----

    /// The `role` word and the columns, for a unit tree id.
    fn unit_facts(&self, t: TypeId) -> Option<Facts> {
        let rec = self.unit_record(t)?;
        let u = &self.unit_types[rec];
        let p = &u.combat;
        let citizen = u.worker != Worker::None;
        let (land, sea, air) = match p.domain {
            crate::attrition::Domain::Land => (true, false, false),
            crate::attrition::Domain::Sea => (false, true, false),
            crate::attrition::Domain::Air => (false, false, true),
        };
        // The derived words the producers read are on the type now —
        // `docs/DATALAYER.md`, "The derived words no column carries" — so
        // every one of these is the original's own bit rather than a
        // stand-in, whenever the loader filled them.
        let c = u.cols;
        let role = self.role_word(t, rec);
        Some(Facts {
            rec,
            citizen: role & role::CITIZEN != 0,
            military: role & role::MILITARY != 0,
            land,
            sea,
            air,
            scout: role & role::SCOUT != 0,
            carry: role & role::CARRY != 0,
            siege: p.siege,
            caravan: c.flag2(uflags2::CARAVAN) || p.roles & combat::role::CARAVAN != 0,
            merchant: p.roles & combat::role::MERCHANT != 0,
            worker: citizen,
            peasant: u.worker == Worker::Citizen,
            scholar: u.worker == Worker::Scholar,
            attack: p.attack,
            control_cost: u.price.pop,
            age: p.age,
            missile: self.seam_missile(t),
            flag_c: c.flag(0x4),
            flag_d: c.flag(0x8),
            flag_p: c.flag(uflags::NO_PRODUCE),
            special_forces: c.flag2(uflags2::SPECIAL_FORCES),
            special_forces_elite: c.flag2(0x40),
        })
    }

    /// The type's `role` word (`UnitTypeData+0x2c8`).
    ///
    /// The loader derives it — [`crate::ai_load::determine_roles`], checked
    /// against the original's own dump for all 364 types — and a type that
    /// carries one is authoritative. A type built by hand in a test carries
    /// zero, and this rebuilds a word from what such a type *does* have:
    /// `worker` for the citizen bit, `combat_role` for the military one, the
    /// combat profile's `BARK` role and the `Scout` lineage for the scout bit,
    /// the garrison layer's `transport` for `carry`. That is exactly what the
    /// five `seam_*` helpers used to answer one at a time.
    pub(crate) fn role_word(&self, t: TypeId, rec: usize) -> u32 {
        let u = &self.unit_types[rec];
        if u.cols.role != 0 {
            return u.cols.role;
        }
        let p = &u.combat;
        let land = p.domain == crate::attrition::Domain::Land;
        let sea = p.domain == crate::attrition::Domain::Sea;
        let mut r = crate::ai_load::determine_roles(&crate::ai_load::RoleFacts {
            citizen_id: u.worker != Worker::None,
            domain: p.domain,
            cat: if u.worker == Worker::None {
                crate::ai_load::cat::FOOT
            } else {
                crate::ai_load::cat::CIVILIAN
            },
            attack: if p.combat_role { p.attack.max(1) } else { 0 },
            max_range: p.max_range,
            carry: i32::from(u.garrison.transport),
            scout: land && self.named_is(t, "Scout"),
            bark: sea && p.roles & combat::role::BARK != 0,
            hoplite: false,
        });
        // `determine_roles` reads `CARRY`, which a hand-built type has no
        // column for; the garrison layer's `transport` stands in.
        if u.garrison.transport {
            r |= role::CARRY;
        }
        r
    }

    /// The same word for a unit **record**, which is all a sweep over the
    /// live objects has to hand ([`Sim::census_units`] reads it through
    /// `UnitData::get_type`, not through the tree).
    ///
    /// A record whose type is in the tree defers to [`Self::role_word`];
    /// one outside it — a type stood up by hand in a test — has only its
    /// own column, and the scout bit of the reconstruction is then out of
    /// reach because that arm resolves the name through the tree.
    pub(crate) fn role_word_of_rec(&self, rec: usize) -> u32 {
        match self.unit_types[rec].tree {
            Some(t) => self.role_word(t, rec),
            None => self.unit_types[rec].cols.role,
        }
    }

    /// **Seam** — `ObjectTypeData::is(t, <named root>)`. The original tests a
    /// `TypeIndex` constant; per `docs/DECISIONS.md` entry 18 the simulation
    /// wants a role bit on the profile instead, and the handful this module
    /// needs (Scout, Spy, Bomber, Fishermen, Anti-Aircraft Gun, Fireship,
    /// Aircraft Carrier, Merchant Fleet, the missiles) have none yet. Until
    /// they do, the lineage test runs against the tree entry whose `NAME`
    /// matches — which is how `rondata`'s loader resolves the same names.
    fn named_is(&self, t: TypeId, name: &str) -> bool {
        let Some(root) = self
            .tech_tree
            .types
            .iter()
            .position(|d| d.kind.is_unit() && d.name.eq_ignore_ascii_case(name))
        else {
            return false;
        };
        self.tech_tree.is(t, root, false)
    }

    /// **Seam** — `UnitTypeData::is_missile` (vslot `+0x114`, a `unit_flags2`
    /// bit). Stands in as the four missile lineages by name plus the
    /// `V2ROCKET` role and the tree's `nuclearmissile`/`icbm` roles.
    fn seam_missile(&self, t: TypeId) -> bool {
        if let Some(rec) = self.unit_record(t)
            && self.unit_types[rec].combat.roles & combat::role::V2ROCKET != 0
        {
            return true;
        }
        for r in [
            self.tech_tree.roles.nuclearmissile,
            self.tech_tree.roles.icbm,
        ] {
            if let Some(x) = r
                && self.tech_tree.is(t, x, false)
            {
                return true;
            }
        }
        self.named_is(t, "Cruise Missile")
    }

    /// ~~**Seams** — `carry`, `unit_flags`, `unit_flags2`.~~ **Closed,
    /// 2026-08-25**: all three are on [`crate::UnitType::cols`], derived by
    /// the loader and checked against the original's own type dump
    /// (`docs/DATALAYER.md`). What replaced them is [`Sim::role_word`] and
    /// [`crate::ai_load::UnitCols::flag`]/`flag2`.
    ///
    /// **Seam** — `leaders[j].leader_flags & 4`, "is a human player"
    /// (`docs/audit/2026-08-23-pathfinder.md`). Stands in as "has no
    /// production script", which `Leader::init` gives every computer leader.
    fn seam_is_human(&self, who: Player) -> bool {
        self.ai.get(who as usize).is_none_or(|l| l.script.is_none())
    }

    /// `Armies::find_army(who, city.x, city.y, −1, ·, −1)` — the slot of
    /// the nearest army in the city's region, or −1 (`docs/ARMY.md` §15).
    /// The sea branch's `init_navy` asks its own `find_army` around the
    /// dock's sea tile ([`Sim::sea_value`]).
    fn army_at(&self, who: Player, city: usize) -> i32 {
        self.find_army(who, self.cities[city].pos, -1, None)
            .map_or(-1, |(s, _)| s as i32)
    }

    // ---- small helpers over the tree and the buildings ----

    /// The root of a building record's `from` chain.
    fn build_root(&self, rec: usize) -> usize {
        let mut r = rec;
        for _ in 0..32 {
            match self.build_types[r].from {
                Some(f) => r = f,
                None => break,
            }
        }
        r
    }

    /// `ObjectTypeData::is(bt, ident)` for a building record — the lineage,
    /// not equality.
    fn build_is_ident(&self, rec: usize, ident: Ident) -> bool {
        let mut r = Some(rec);
        for _ in 0..32 {
            let Some(x) = r else { break };
            if self.build_types[x].ident == ident {
                return true;
            }
            r = self.build_types[x].from;
        }
        false
    }

    /// `BuildTypeData::is_military_trainer` — the flag on the base type.
    fn is_military_trainer(&self, rec: usize) -> bool {
        self.build_types[self.build_root(rec)].flags & MILITARY_TRAINER != 0
    }

    /// Whether a live building's type is in `wt`'s lineage — the
    /// `is(where, is_build_type(t))` every producer test uses.
    fn building_of_type(&self, b: usize, wt: TypeId) -> bool {
        self.buildings[b]
            .ty
            .and_then(|rec| self.build_types[rec].tree)
            .is_some_and(|bt| self.tech_tree.is(bt, wt, false))
    }

    /// `LeaderData::get_units(t, 0)` / `get_queued(t, …)`: the count of `t`
    /// plus every type whose grafted `from` chain reaches it — the line's
    /// count including the nation's variants and later upgrades.
    fn line_count(&self, who: Player, t: TypeId, queued: bool) -> i32 {
        let w = who as usize;
        let mut n = 0;
        for (u, d) in self.tech_tree.types.iter().enumerate() {
            if !d.kind.is_unit() {
                continue;
            }
            let mut p = Some(u);
            let mut hit = false;
            for _ in 0..64 {
                let Some(x) = p else { break };
                if x == t {
                    hit = true;
                    break;
                }
                p = self.tech_tree.get_graft(
                    &self.setup,
                    &self.tech[w],
                    self.tech_tree.types[x].from,
                );
            }
            if !hit {
                continue;
            }
            if let Some(rec) = self.unit_record(u) {
                n += if queued {
                    self.muster[w].queued_by_type[rec]
                } else {
                    self.muster[w].by_type[rec]
                };
            }
        }
        n
    }

    /// The raw `num_units[t]` and `num_queued[t]` — not the graft-aware
    /// `get_*`; the count cap and several branches read these.
    fn raw_counts(&self, who: Player, rec: usize) -> (i32, i32) {
        let m = &self.muster[who as usize];
        (m.by_type[rec], m.queued_by_type[rec])
    }

    /// `City::count_gather_slots(city, max[6], free[6])`: the city's finished
    /// gather buildings' slots per good, and how many are free. The
    /// University's slots land in `[KNOWLEDGE]` but are **excluded** from the
    /// returned total, as the original excludes them.
    fn gather_slot_picture(&self, c: usize) -> (i32, [i32; RESOURCES], [i32; RESOURCES]) {
        self.count_gather_slots(c)
    }

    /// `City::could_queue(city, t)`: any member of the city chain can make
    /// `t` and has room.
    fn city_could_queue(&self, c: usize, t: TypeId) -> bool {
        let Some(wt) = self.tech_tree.types[t].where_ else {
            return false;
        };
        self.city_chain(c).into_iter().any(|b| {
            let bd = &self.buildings[b];
            bd.alive
                && bd.active
                && bd.queue.has_room()
                && !self.building_unassimilated(b)
                && self.building_of_type(b, wt)
        })
    }

    /// `BuildData::count_queue(1, t)`: queued entries of `t`'s lineage;
    /// `count_queue(0, _)` counts queued citizens instead.
    fn count_queue(&self, b: usize, of: Option<TypeId>) -> i32 {
        self.buildings[b]
            .queue
            .items
            .iter()
            .filter(|i| {
                if i.tech.is_some() {
                    return false;
                }
                match of {
                    None => self.unit_types[i.ty].worker == Worker::Citizen,
                    Some(t) => self.unit_types[i.ty]
                        .tree
                        .is_some_and(|x| self.tech_tree.is(x, t, false)),
                }
            })
            .count() as i32
    }

    /// The city's region as the per-region census arrays index it.
    fn city_region(&self, c: usize) -> u16 {
        self.cities[c].reg.unwrap_or(0)
    }
}

// ---- the value passes ----

/// The original's value arithmetic is a 32-bit `imul` and wraps; the tail's
/// `val < 0 → 9999999` is the guard it puts on the result, so an overflowed
/// product is a value the original goes on to use. Every product in a value
/// chain therefore wraps rather than panicking.
const fn wm(a: i32, b: i32) -> i32 {
    a.wrapping_mul(b)
}

/// `create_units`' closing tail, as one expression so the claim has
/// something to assert against — `out = wm(fac, wm(want, val) / divisor)
/// / 256`, with the original's own `out < 0 → 9,999,999` after it.
///
/// **It is a 32-bit `imul` and it wraps**, which `docs/AI.md` §45
/// established for `tech_value`'s tail and item 422 measured here. The
/// guard catches a wrap that lands **negative**; a wrap that lands
/// positive passes through it as an ordinary number, and
/// [`the_scholar_offer_on_great_lakes_9380_is_a_positive_wrap`] is that
/// case with its five terms measured off the run.
///
/// **The wrap is fidelity, not a defect.** The original is a 32-bit
/// engine and its own `imul` wraps the same way; computing this tail in
/// 64 bits instead costs **2,727 frames of the Great Lakes word**,
/// 9510 → 6783, measured in a scratch build (`docs/AI.md` §49.4). Do not
/// "fix" the overflow.
///
/// **And the gate already knows.** `rondata::diff`'s
/// `LONG_WORD_GREAT_LAKES` pins 9510 as a floor, so widening this to
/// `i64` does not pass quietly — it **fails the floor**. A reader who
/// does not believe this comment is stopped in one `cargo test` rather
/// than an afternoon.
pub fn offer_value(fac: i32, want: i32, val: i32, divisor: i32) -> i32 {
    if divisor == 0 {
        return 0;
    }
    let out = wm(fac, wm(want, val) / divisor) / 256;
    if out < 0 { 9_999_999 } else { out }
}

/// `create_units`' **scholar arm**, from `k` to the `val` the tail is
/// handed — the listing over `006c528c..006c52f5`, as one function so the
/// claim has something to assert against.
///
/// ```text
/// 006c528c  eax = leader+0x7a4                  ; infra_mod
/// 006c5292  ecx = leader+0x8b0                  ; total
/// 006c5298  eax *= k
/// 006c529b  eax *= 0x2710                       ; 10000
/// 006c52ab  esi = (eax + (eax>>31 & 0xff)) >> 8  ; /256, toward zero
/// 006c52ae  cmp [leader+0x8c8], ecx ; jge  →  esi *= 60   (filled < total)
/// 006c52d6  cmp ecx, (total*2)/3    ; jge  →  esi *= 10, escrow = 1
/// 006c52e6  cmp ecx, [leader+0x8b0] ; jge  →  esi *= 5    (filled < total)
/// 006c52f5  jmp LAB_006c4cfc
/// ```
///
/// **The three tests are independent `if`s, and the first and third are
/// the same test.** The third's `jge` at `006c52ec` skips only its own
/// `lea esi,[esi+4*esi]`, and both arms land on `LAB_006c4cfc` — so a
/// city below two thirds takes **all three** multiplies, ×3000 rather
/// than ×600. Reading the third as an `else if` was half of the defect
/// `docs/AI.md` §51 measured and §53 closed; the decompiler prints it
/// correctly and it reads like a compiler artefact, which is why the
/// listing is quoted here rather than cited.
///
/// The `escrow` the middle arm sets is the caller's; this returns only
/// the value. Every multiply wraps — see [`offer_value`], and
/// [`the_scholar_arm_pays_great_lakes_9380_s_two_offers`] for the two
/// numbers it has to hit.
pub fn scholar_value(infra_mod: i32, k: i32, filled: i32, total: i32) -> i32 {
    let mut v = wm(wm(infra_mod, k), 10000) / 256;
    if filled < total {
        v = wm(v, 60);
    }
    if filled < total * 2 / 3 {
        v = wm(v, 10);
    }
    if filled < total {
        v = wm(v, 5);
    }
    v
}

/// The war multiplier the air and land-military branches put on a base `b`
/// (`create_units` 1385–1422). `attacked_here` is the land branch's extra:
/// a city-trained type in a city under attack takes `b·4` rather than `b·2`.
#[allow(clippy::too_many_arguments)]
fn war_multiplier(
    b: i32,
    reg_wars: i32,
    reg_neutrals: i32,
    active_wars: i32,
    wars: i32,
    strategy: i32,
    reg_attacked: i32,
    attacked_here: bool,
) -> (i32, i32) {
    // `base` itself doubles in the both-wars case — the original writes it
    // back into `local_10`, which the batch size and nothing else reads.
    let mut base = b;
    let v = if reg_wars != 0 && active_wars != 0 {
        base = wm(b, 2);
        base
    } else if reg_wars != 0 || reg_neutrals != 0 || active_wars != 0 {
        wm(b, 3) / 2
    } else if wars != 0 {
        wm(b, 11) / 10
    } else {
        b
    };
    let v = if strategy & 4 == 0 && reg_attacked == 0 {
        if strategy & 2 != 0 { wm(v, 3) / 2 } else { v }
    } else if attacked_here {
        wm(v, 4)
    } else {
        wm(v, 2)
    };
    (v, base)
}

/// The army-size ladder (`create_units` 438–459 for air, 1424–1446 for land).
/// Each rung is an independent `if`, so the first two stack. The land branch
/// opens ×6 / ×2 where the air branch opens ×2 / ×3/2 — the two ladders are
/// **not** the same, which the first reading had as identical.
fn army_ladder(v: i32, combat: i32, target: i32, land: bool, reg_over: bool) -> i32 {
    let mut v = v;
    if combat < target / 2 {
        v = wm(v, if land { 6 } else { 2 });
    }
    if combat < (target << 2) / 5 {
        v = if land { wm(v, 2) } else { wm(v, 3) / 2 };
    }
    if combat > target * 5 / 4 {
        v = wm(v, 2) / 3;
    }
    if combat > target * 3 / 2 {
        v /= 2;
    }
    if combat > target * 2 {
        v /= 2;
    }
    if reg_over {
        v /= 2;
    }
    v
}

/// The batch size for a combat type (`create_units` 460–476): ten
/// population's worth, clipped to what is left of the count cap, then walked
/// down until it leaves a gap of at least one below the population cap.
fn batch_size(control_cost: i32, remaining: i32, effective_pop: i32, pop_cap: i32) -> i32 {
    if control_cost <= 0 {
        return 1;
    }
    let mut num = 10 / control_cost;
    if num < 1 {
        return 1;
    }
    if remaining < num {
        num = remaining;
    }
    while num > 1 && effective_pop >= pop_cap - num - 1 {
        num -= 1;
    }
    num
}

impl Sim {
    /// `create_units`: every unit type the leader could train, valued and
    /// offered to the make list.
    pub fn create_units(&mut self, who: Player) {
        let w = who as usize;
        self.ai[w].unit_offers.clear();
        self.ai[w].unit_offers_frame = self.frame;
        let d = self.ai_difficulty();
        let pop_cap = self.muster[w].cap;
        let army_target = (pop_cap - 40).max(pop_cap / 2);
        let mut mil_level = self.tech[w].epoch[tech::Line::Military.index()];
        let siege_cap_d = if d < 2 {
            2 * d + 1
        } else {
            (2 * d + 2).min(10)
        };
        let civilians = {
            let cn = &self.ai[w].census;
            cn.merchants + cn.caras + cn.scholars + cn.peasants
        };
        let city_num = self.cities_of(who).len() as i32;
        let rush_rules = self.lobby.rush_rules;
        let mut first_city = true;

        let units: Vec<TypeId> = self
            .tech_tree
            .types
            .iter()
            .enumerate()
            .filter(|(_, d)| d.kind.is_unit())
            .map(|(i, _)| i)
            .collect();
        let cities: Vec<usize> = (0..self.cities.len())
            .filter(|&c| self.cities[c].alive && self.cities[c].owner == who)
            .collect();

        for c in cities {
            let city_o = self.cities[c].building;
            let r = self.city_region(c);
            let army_here = self.army_at(who, c);
            let (slots, _gmax, gfree) = self.gather_slot_picture(c);
            // `city_flags & 2`, which `docs/CITIES.md` §1.4 records as
            // "do not heal / auto-repair" with no setter found and the
            // report reads as "attacked". The bit is `no_heal`.
            let city_attacked = self.cities[c].no_heal;

            for &t in &units {
                // 1. The population and the availability.
                if self.ai[w].effective_pop > pop_cap || !self.type_available(who, t) {
                    continue;
                }
                let Some(f) = self.unit_facts(t) else {
                    continue;
                };
                // 2. The trainer buildings, including their upgraded forms.
                let Some(wt) = self.tech_tree.types[t].where_ else {
                    continue;
                };
                let Some(wrec) = self.build_record(wt) else {
                    continue;
                };
                let trainers = self.buildings_of_line(who, wrec);
                // 3. Trainers exist, and the type is not excluded (`p`).
                if trainers == 0 || f.flag_p {
                    continue;
                }
                // 4. City-trained or globally trained.
                let fort = self.build_is_ident(wrec, Ident::Fort);
                let per_city = !fort
                    && (!self.is_military_trainer(wrec)
                        || self.build_is_ident(wrec, Ident::Village));
                if per_city {
                    if !self.city_could_queue(c, t) {
                        continue;
                    }
                } else if !first_city && !f.sea {
                    continue;
                }

                // 5. The count cap.
                let (units_now, queued_now) = self.raw_counts(who, f.rec);
                let owned = units_now + queued_now;
                let non_siege = self.ai[w].census.non_siege;
                let sea_combat = self.ai[w].census.sea_combat;
                let mut cap: i32 = -1;
                let mut capped = false;
                if f.siege {
                    cap = city_num.min(siege_cap_d).min(mil_level);
                    capped = true;
                } else if f.flag_c {
                    cap = city_num.min(6);
                    capped = true;
                } else if f.flag_d {
                    cap = city_num.min(10);
                    capped = true;
                } else if f.military {
                    if d == 1 {
                        cap = 7;
                    } else if d == 0 {
                        cap = 3;
                    }
                }
                if capped {
                    if d < 2 {
                        cap = 2;
                    }
                    if non_siege - 5 * owned - sea_combat < 10 {
                        continue;
                    }
                }
                if self.named_is(t, "Anti-Aircraft Gun") {
                    cap /= 2;
                }
                let mut remaining = 5;
                if cap > 0 {
                    if cap <= owned {
                        continue;
                    }
                    remaining = cap - owned;
                }

                // 6. The base.
                let mut base = self.ai[w].census.pop * 1000 / city_num.max(1);
                let mut escrow = 0;

                // 7. The military gate — the `else` of the `role & 0x10000`
                // test, so it runs for sea and air military too, not only
                // land (the first reading called it the land gate).
                if f.military {
                    if rush_rules == 8 {
                        continue;
                    }
                    let land_army = non_siege - sea_combat;
                    let pass = match d {
                        0 => {
                            land_army <= 3 * city_num
                                && land_army < 2 * mil_level + 3
                                && land_army < 10
                        }
                        1 => {
                            land_army <= 5 * city_num
                                && land_army < (mil_level + 1) * 7 / 2 + 1
                                && land_army < 25
                        }
                        2 => land_army <= 6 * city_num && land_army < 5 * mil_level + 1,
                        _ => true,
                    };
                    if !pass {
                        continue;
                    }
                    if self.ai[w].wonder_mod != 0 {
                        base /= 10;
                    }
                    let dmod = if f.sea {
                        self.ai[w].sea_mod
                    } else if f.air {
                        self.ai[w].air_mod
                    } else {
                        self.ai[w].ground_mod
                    };
                    base = wm(base, dmod) / 256;
                    if d != 2 {
                        let mut pv = self.unit_prod_value(who, t);
                        if d == 0 || d > 2 {
                            pv = pv * 3 / 2;
                        }
                        self.mark(SITE_UNIT_BIAS);
                        let roll = self.rng.roll();
                        pv = (roll % 3 + 4) * pv / 5;
                        if d < 2 {
                            pv = -pv;
                        }
                        pv = pv.clamp(-256, 256);
                        base = wm(base, pv + 256) / 256;
                    }
                }

                // 8. The common body: the near-cap tapering, then `want`.
                if !f.citizen && !f.caravan && !f.military {
                    let eff = self.ai[w].effective_pop;
                    if eff > pop_cap - 4 {
                        base /= 2;
                    }
                    if eff > pop_cap - 2 {
                        base /= 2;
                    }
                    if eff > pop_cap - 1 {
                        continue;
                    }
                }
                let want_civ = if pop_cap * 2 < 25 { 4 } else { pop_cap * 2 / 5 };
                let mut want = 4;
                if self.build_is_ident(wrec, Ident::Stable) {
                    want = if self
                        .tech_tree
                        .has_tribe_bonus(&self.setup, &self.tech[w], 0x13)
                    {
                        5
                    } else {
                        3
                    };
                }
                if self.build_is_ident(wrec, Ident::SiegeFactory) {
                    want = 2;
                }
                if self.build_is_ident(wrec, Ident::Dock) {
                    want = 2;
                }
                if self.build_is_ident(wrec, Ident::Barracks) {
                    want = if pop_cap * 3 < 25 { 4 } else { pop_cap * 3 / 5 };
                }
                if f.flag_d {
                    want = 2;
                }
                if f.flag_c {
                    want = 1;
                }

                // 9. The branch.
                //
                // **`num` is the branch's own answer, not a constant**
                // (item 362). The original's `TVar24` is set by every
                // arm before `LAB_006c4cfc` and reaches `make_me` as the
                // slot's `num` through `local_14 = min(TVar24,
                // remaining)`; `1` is only the arms that want one. This
                // crate carried the `1` everywhere a civilian was
                // offered, because `civilian_value` computed the count
                // and returned three fields that did not include it, and
                // the AI therefore bought its scholars and its caravans
                // one at a time forever. `docs/AI.md` §42.
                let mut num = 1;
                let cat;
                let v;
                if !f.land && !f.sea {
                    // Air. A non-military air type has no branch at all.
                    if !(f.air && f.military) {
                        continue;
                    }
                    cat = 6;
                    let Some(pass) = self.air_value(
                        who,
                        t,
                        &f,
                        base,
                        r,
                        army_target,
                        remaining,
                        &mut escrow,
                        &mut num,
                        pop_cap,
                    ) else {
                        continue;
                    };
                    v = pass;
                } else if f.sea {
                    let Some((sv, sc)) =
                        self.sea_value(who, t, &f, c, base, r, army_here, &mut escrow)
                    else {
                        continue;
                    };
                    v = sv;
                    cat = sc;
                } else if !f.military {
                    // Land civilians and scouts.
                    if self.lobby.elimination == 1 && !self.owns_capital(who) {
                        continue;
                    }
                    if f.scout {
                        let n = units_now + queued_now;
                        if n >= 2 {
                            continue;
                        }
                        cat = 6;
                        escrow = i32::from(n == 0);
                        v = 10000 / (self.frame / 1000).max(1) as i32;
                    } else {
                        if per_city && city_attacked && city_num >= 2 {
                            continue;
                        }
                        let Some((cv, cc, cw)) = self.civilian_value(
                            who,
                            t,
                            &f,
                            c,
                            city_o,
                            base,
                            r,
                            slots,
                            &gfree,
                            per_city,
                            want_civ,
                            remaining,
                            &mut escrow,
                            &mut num,
                        ) else {
                            continue;
                        };
                        v = cv;
                        cat = cc;
                        want = cw;
                    }
                } else {
                    // Land military.
                    if rush_rules == 8
                        || (rush_rules <= 7 && self.leaders_max_age() < rush_rules - 2)
                    {
                        continue;
                    }
                    if self.lobby.conquest {
                        mil_level += 1;
                    }
                    let land_army = non_siege - sea_combat;
                    if land_army < mil_level * 3 && !f.siege {
                        base = wm(base, 100);
                        escrow = 1;
                    }
                    if per_city && !city_attacked {
                        if self.frame < i64::from(2000 / self.ai_speed.max(1)) {
                            continue;
                        }
                        if pop_cap < 24 {
                            base /= 4;
                        }
                        if pop_cap < 36 {
                            base /= 2;
                        }
                    }
                    cat = 6;
                    let cn = &self.ai[w].census;
                    let (wv, wbase) = war_multiplier(
                        base,
                        Census::reg(&cn.reg_wars, r),
                        Census::reg(&cn.reg_neutrals, r),
                        cn.active_wars,
                        cn.wars,
                        Census::reg(&cn.strategy, r),
                        Census::reg(&cn.reg_attacked, r),
                        per_city && city_attacked,
                    );
                    // The original writes the doubled base back over
                    // `local_10`; nothing reads it again this iteration.
                    let _ = wbase;
                    let mut x = wm(f.age + 8, wv) / 8;
                    let reg_over = Census::reg(&cn.reg_combat, r)
                        > Census::reg(&cn.reg_pop, r) * (Census::reg(&cn.reg_wars, r) + 1) * 2;
                    x = army_ladder(x, cn.combat, army_target, true, reg_over);
                    let siege = cn.siege;
                    let non_siege = cn.non_siege;
                    if f.siege {
                        if non_siege > 2 {
                            x = wm(x, 2);
                        }
                        if siege < non_siege / 8 {
                            x = wm(x, 2);
                        }
                    } else if siege < siege_cap_d {
                        if siege < non_siege / 20 {
                            x /= 2;
                        } else if siege < non_siege / 10 {
                            x = wm(x, 2) / 3;
                        }
                    }
                    if per_city && army_here < 0 && !city_attacked && f.land {
                        x = wm(x, 3) / 4;
                    }
                    v = x;
                    num = batch_size(f.control_cost, remaining, self.ai[w].effective_pop, pop_cap);
                }

                // 10. The tail.
                let mut val = v;
                if cap >= 0
                    && !self
                        .tech_tree
                        .roles
                        .machinegun
                        .is_some_and(|m| self.tech_tree.is(t, m, false))
                {
                    let x = wm(val, remaining);
                    val = wm(x, 10);
                    if units_now == 0 && city_num > 2 {
                        val = wm(x, 1000);
                    }
                }
                if num > remaining {
                    num = remaining;
                }
                if f.worker || f.caravan || f.merchant {
                    if civilians > 49 && civilians > pop_cap / 2 {
                        val /= 100;
                    }
                    if self.civilian_ceiling(who) * 3 / 5 <= civilians {
                        continue;
                    }
                }
                let mut fac =
                    self.check_income(who, t, 0x400, None, escrow != 0, city_o as i32, num, escrow);
                let afford = self.type_affordable(who, t, escrow != 0);
                if afford < num && afford != 0 {
                    fac = self.check_income(
                        who,
                        t,
                        0x400,
                        None,
                        escrow != 0,
                        city_o as i32,
                        afford,
                        escrow,
                    );
                    num = afford;
                }
                let divisor = want + queued_now + units_now;
                if divisor == 0 {
                    continue;
                }
                let mut out = offer_value(fac, want, val, divisor);
                if self.ai[w].wonder_mod != 0 {
                    out /= 2;
                }
                self.ai[w].unit_offers.push(crate::ai::MakeObject {
                    t: t as i32,
                    val: out,
                    escrow,
                    city: c as i32,
                    up: 0,
                    o: -1,
                    num,
                    cat,
                    wx: 0,
                    wy: 0,
                });
                self.ai[w]
                    .make_list
                    .make_me(t as i32, out, escrow, cat, c as i32, 0, num, 0, 0);
            }
            first_city = false;
        }
    }

    /// §2.1 and §2.2 — the air military and the missiles. Returns the value,
    /// or `None` to skip the type.
    #[allow(clippy::too_many_arguments)]
    fn air_value(
        &mut self,
        who: Player,
        t: TypeId,
        f: &Facts,
        base: i32,
        r: u16,
        army_target: i32,
        _remaining: i32,
        escrow: &mut i32,
        num: &mut i32,
        pop_cap: i32,
    ) -> Option<i32> {
        let w = who as usize;
        if f.missile {
            return self.missile_value(who, t, f, escrow);
        }
        let airbases = self
            .build_types
            .iter()
            .position(|b| b.ident == Ident::Airbase)
            .map_or(0, |rec| self.num_buildings_of(who, rec));
        if self.ai[w].census.air >= airbases * 8 {
            return None;
        }
        let d = self.ai_difficulty();
        let air_cap = if d < 2 {
            2
        } else if d == 2 {
            4
        } else {
            16
        };
        let remaining = air_cap;
        let n = self.line_count(who, t, true) + self.line_count(who, t, false);
        if n >= air_cap {
            return None;
        }
        let cn = &self.ai[w].census;
        let (mut v, _) = war_multiplier(
            base,
            Census::reg(&cn.reg_wars, r),
            Census::reg(&cn.reg_neutrals, r),
            cn.active_wars,
            cn.wars,
            Census::reg(&cn.strategy, r),
            Census::reg(&cn.reg_attacked, r),
            false,
        );
        v = wm(f.attack, v) / 10;
        if self.named_is(t, "Bomber") {
            let cn = &self.ai[w].census;
            let fighters = cn.fighters;
            let bombers = cn.bombers;
            if fighters != 0 {
                if bombers < fighters / 4 {
                    v = wm(v, 2);
                }
                if bombers < fighters / 8 {
                    v = wm(v, 2);
                }
            }
            if fighters > 7 && bombers < 3 {
                v = wm(v, 2);
            }
        }
        let cn = &self.ai[w].census;
        let reg_over = Census::reg(&cn.reg_combat, r)
            > Census::reg(&cn.reg_pop, r) * (Census::reg(&cn.reg_wars, r) + 1) * 2;
        v = army_ladder(v, cn.combat, army_target, false, reg_over);
        *num = batch_size(f.control_cost, remaining, self.ai[w].effective_pop, pop_cap);
        Some(v)
    }

    /// §2.2 — the V2/cruise line and the nuke.
    fn missile_value(
        &mut self,
        who: Player,
        t: TypeId,
        f: &Facts,
        escrow: &mut i32,
    ) -> Option<i32> {
        let w = who as usize;
        let pers = self.ai[w].pers;
        let nuke = self
            .tech_tree
            .roles
            .nuclearmissile
            .is_some_and(|x| self.tech_tree.is(t, x, false));
        if !nuke {
            // Conventional: never again once the nuke is reachable.
            if let Some(x) = self.tech_tree.roles.nuclearmissile
                && self.tech_tree.has_preq(&self.setup, &self.tech[w], x)
            {
                return None;
            }
            let n = self.line_count(who, t, true) + self.line_count(who, t, false);
            let mut v = 600_000;
            if n == 0 {
                if pers.air > 0 {
                    v = 1_200_000;
                }
            } else {
                v = 600_000 / (n + 1);
            }
            if pers.nukes < 0 {
                v = wm(v, 2);
            }
            return Some(v);
        }
        // The nuke: difficulty 2 and up only.
        if self.ai_difficulty() <= 1 {
            return None;
        }
        let b = if self.ai[w].census.wars != 0 {
            400_000
        } else {
            200_000
        };
        let mine = self.line_count(who, t, false) + self.line_count(who, t, true);
        let mut max_enemy = 0;
        let mut vulnerable = false;
        for j in 0..self.players.len() {
            let jw = j as Player;
            if self.defeated[j] || self.is_ally(who, jw) {
                continue;
            }
            vulnerable = true;
            let theirs = self.line_count(jw, t, false);
            if theirs > max_enemy {
                max_enemy = theirs;
            }
        }
        if !vulnerable {
            return None;
        }
        let silos = self.tech_tree.types[t]
            .where_
            .and_then(|wt| self.build_record(wt))
            .map_or(0, |rec| self.num_buildings_of(who, rec));
        if mine >= silos {
            return None;
        }
        let mut v = b;
        if mine < max_enemy {
            *escrow = 1;
            v = wm(max_enemy - mine, b);
        }
        if self.type_affordable(who, t, true) != 0 {
            *escrow = 1;
        }
        if mine == 0 {
            v = wm(v, 5);
        }
        let pers = self.ai[w].pers;
        v = wm(v, pers.nukes + 2);
        if pers.nukes > 0 {
            v = wm(v, 2);
        }
        if pers.rush < 0 && pers.army_size > 0 {
            v = wm(v, 2);
        }
        let _ = f;
        Some(v)
    }

    /// §2.4 — the sea branch: fishermen and warships. Returns `(val, cat)`.
    #[allow(clippy::too_many_arguments)]
    fn sea_value(
        &mut self,
        who: Player,
        t: TypeId,
        f: &Facts,
        c: usize,
        base: i32,
        r: u16,
        army_here: i32,
        escrow: &mut i32,
    ) -> Option<(i32, i32)> {
        let w = who as usize;
        let _ = army_here;
        if self.frame < i64::from(1000 / self.ai_speed.max(1)) {
            return None;
        }
        // The dock: the nearest of mine around the city's centre, whichever
        // city it belongs to (§57).
        let dock = self.find_dock_near(who, self.cities[c].pos)?;
        let (sea_reg, sea_tile) = self.dock_sea_tile(dock)?;
        if self.world.cells_in(sea_reg).count() <= 19 {
            return None;
        }
        let n = self.line_count(who, t, false) + self.line_count(who, t, true);
        let sea_map = self.world.sea_map();
        let bark = self.unit_types[f.rec].combat.roles & combat::role::BARK != 0;
        let reg_combat = Census::reg(&self.ai[w].census.reg_combat, sea_reg);
        if !(sea_map > 2 || !bark || reg_combat < sea_map * 2) {
            return None;
        }
        // A type that carries other units is never queued here.
        if f.carry {
            return None;
        }
        if !f.military {
            if !self.named_is(t, "Fishermen") {
                return None;
            }
            let cn = &self.ai[w].census;
            let mil_level = self.tech[w].epoch[tech::Line::Military.index()];
            let attacked = self.buildings[dock].under_attack != 0;
            let commerce = self.tech[w].epoch[tech::Line::Commerce.index()];
            if attacked
                || !(reg_combat != 0 || mil_level < 3)
                || cn.idle_fishermen != 0
                || !(n < commerce || n < 4)
            {
                return None;
            }
            // The `econ[FOOD]` multipliers here are dead in the listing —
            // they scale a local nothing reads (report §9.1).
            let mut b = wm(base, 100) / (n + 1);
            if reg_combat < 10 {
                b /= 10 - reg_combat;
            }
            return Some((wm(sea_map, b), 4));
        }
        // Warships.
        let mil_level = self.tech[w].epoch[tech::Line::Military.index()];
        let combat = self.ai[w].census.combat;
        if sea_map <= 1
            || !(sea_map > 2 || bark)
            || !(mil_level < 5 || reg_combat <= combat / 3)
            || reg_combat > combat / 2
        {
            return None;
        }
        let mut max_e = 0;
        let mut sum = 0;
        let mut count = 0;
        for j in 0..self.players.len() {
            let jw = j as Player;
            if self.defeated[j] || jw == who || self.is_ally(jw, who) {
                continue;
            }
            let e = Census::reg(&self.ai[j].census.reg_combat, sea_reg);
            if e > max_e {
                max_e = e;
            }
            sum += e;
            count += 1;
        }
        let avg_e = if count == 0 { 0 } else { sum / count };
        if reg_combat > avg_e + 5 || reg_combat > max_e + 3 {
            return None;
        }
        // **`Armies::init_navy`** (`create_units@006c40a0:823`, listing
        // `6c5a52`–`6c5aa3`): around the centre of the tile the sea was
        // read on, no army of mine in its region (`find_army(who, x, y, −1,
        // ·, −1)`) and a city of mine found → a new army at that city with
        // `navy = 1` and `reg = the sea`. It is what the dock's first
        // warship joins when it comes out (`Unit::add_to_army`'s sea arm),
        // and on East Indies it is run139's 9981 (`docs/ORDERS.md` §25).
        let half = crate::world::UNITS_PER_TILE / 2;
        let at = crate::world::Pos::new(
            sea_tile.x * crate::world::UNITS_PER_TILE + half,
            sea_tile.y * crate::world::UNITS_PER_TILE + half,
        );
        if self.find_army(who, at, -1, None).is_none()
            && let Some(city) = self.navy_city(who, at)
        {
            let s = self.init_army(who, Some(city));
            let a = &mut self.armies[w].list[s];
            a.navy = true;
            a.reg = Some(sea_reg);
        }
        let carrier = self.named_is(t, "Aircraft Carrier");
        let attacked = self.buildings[dock].under_attack != 0;
        if sea_map > 2 {
            let mut b = wm(base, 500) / (n + 1);
            if carrier {
                if n > 1 || reg_combat < 8 || reg_combat < avg_e {
                    return None;
                }
                *escrow = 1;
            } else {
                *escrow = i32::from(n < mil_level + 3);
                if reg_combat < avg_e {
                    b = if bark { b / 10 } else { wm(b, 10) };
                }
            }
            if attacked {
                *escrow = 1;
                if self.named_is(t, "Fireship") {
                    return Some((wm(wm(wm(sea_map, avg_e + 1), b), 100), 6));
                }
                b /= 1000;
            }
            return Some((wm(wm(sea_map, avg_e + 1), b), 6));
        }
        let city_reg_size = self.world.cells_in(r).count() as i32;
        if !(reg_combat <= city_reg_size / 20 || avg_e != 0) || reg_combat >= 2 {
            return None;
        }
        Some((wm(wm(sea_map, avg_e + 1), wm(base, 10) / (n + 1)), 6))
    }

    /// `ObjectsData::find_building(city.x, city.y, SEARCH_FRIENDLY, who,
    /// 0x1800, 0, FILTER_BASE_TYPE, DOCK, 0)@0065d260` — the sea branch's
    /// dock (`create_units@006c40a0:612`, `docs/AI.md` §57).
    ///
    /// **It is a search around the city, not the city's own chain.** A Dock
    /// need not belong to any city — East Indies' `1/2010` is `city -1` on
    /// every block run99 and run139 print — and it is still the dock the
    /// city's ships are offered at. The radius `0x1800` takes the bounded
    /// arm (`(0x1800 + 0x2ff) / 0x300 = 8 ≤ 9`): the object chains of the
    /// cells `circle_x/y[..circle_radius[8]]` around the city's cell. A
    /// candidate is a live, finished building (vslots `+0xc` and `+0x4c`,
    /// `SubObjectData::is_active` and `WallData::is_active`) whose base type
    /// is the Dock. The distance is `vector_dist` in **tiles** (listing
    /// `0065d471`–`0065d4b5`: `sar $6` through `div_3_table` on both
    /// operands) and the winner test is `<=`, so a tie goes to the last
    /// candidate in walk order. The `<= 0x1800` bound is in those tiles
    /// and never binds inside eight cells.
    fn find_dock_near(&self, who: Player, at: crate::world::Pos) -> Option<usize> {
        const RADIUS: i32 = 0x1800;
        let circ = crate::ai_place::circle();
        let home = at.cell();
        let a = at.tile();
        let mut best: Option<(i32, usize)> = None;
        for k in 0..circ.radius[((RADIUS + 0x2ff) / 0x300) as usize] {
            let cell = crate::world::Cell::new(home.x + circ.x[k], home.y + circ.y[k]);
            for (b, bd) in self.buildings.iter().enumerate() {
                if bd.owner != who
                    || !bd.alive
                    || !bd.active
                    || bd.pos.cell() != cell
                    || !bd
                        .ty
                        .is_some_and(|rec| self.build_is_ident(rec, Ident::Dock))
                {
                    continue;
                }
                let p = bd.pos.tile();
                let d = crate::world::vector_dist((p.x - a.x).abs(), (p.y - a.y).abs());
                if d <= RADIUS && best.is_none_or(|(e, _)| d <= e) {
                    best = Some((d, b));
                }
            }
        }
        best.map(|(_, b)| b)
    }

    /// The dock's sea region and the tile it was read on —
    /// `create_units@006c40a0:627`'s footprint walk. `WallData::tile_corner`,
    /// then the footprint's **tiles**, `x` outer and `y` inner: the first
    /// ocean tile of a column (`mask & 0x30 == 0x20`) answers
    /// `WorldData::get_tregion`, with the coastal `region2` refinement, and
    /// ends that column; a sea region ends the walk. The tile is the point
    /// the navy's army and city are looked for around (`docs/ORDERS.md`
    /// §25). This walked the region grid from the centre cell until item
    /// 579.
    fn dock_sea_tile(&self, dock: usize) -> Option<(u16, crate::world::Pos)> {
        let bd = &self.buildings[dock];
        let rec = bd.ty?;
        let corner = self.tile_corner(rec, bd.pos);
        let (xs, ys) = (self.build_types[rec].x_size, self.build_types[rec].y_size);
        let sea = |r: u16| self.world.terrain(r) == Terrain::Sea;
        let mut found: Option<(u16, crate::world::Pos)> = None;
        for tx in corner.x..corner.x + xs {
            for ty in corner.y..corner.y + ys {
                let t = crate::world::Pos::new(tx, ty);
                if self.world.tile_mask(t) & crate::world::tile::SURFACE
                    == crate::world::tile::SURFACE_OCEAN
                {
                    found = self.world.tregion_alt(t).map(|r| (r, t));
                    break;
                }
            }
            if found.is_some_and(|(r, _)| sea(r)) {
                break;
            }
        }
        found.filter(|&(r, _)| sea(r))
    }

    /// `ObjectsData::find_city(x, y, SEARCH_FRIENDLY, who, ·, 0, FILTER_ALL,
    /// 0, 0)@0065ba90` — the navy's city: the nearest live city of `who`,
    /// **anywhere**. The flag word is 0, so the region filter (`0x200`) is
    /// off, and the fifth argument is whatever `ecx` held after
    /// `find_army` returned (listing `6c5a83`); `find_city` never reads it.
    /// The distance starts at 99,999,999 and the test is `<=`, so a tie
    /// goes to the later city.
    fn navy_city(&self, who: Player, at: crate::world::Pos) -> Option<usize> {
        let mut best: Option<(usize, i32)> = None;
        for c in self.cities_of(who) {
            let p = self.cities[c].pos;
            let d = crate::world::vector_dist(p.x - at.x, p.y - at.y);
            if best.is_none_or(|(_, bd)| d <= bd) {
                best = Some((c, d));
            }
        }
        best.map(|(c, _)| c)
    }

    /// §2.3 — the land civilians: merchant, caravan, citizen, scholar, spy
    /// and the special-forces class. Returns `(val, cat, want)`.
    #[allow(clippy::too_many_arguments)]
    fn civilian_value(
        &mut self,
        who: Player,
        t: TypeId,
        f: &Facts,
        c: usize,
        city_o: usize,
        base: i32,
        r: u16,
        slots: i32,
        gfree: &[i32; RESOURCES],
        per_city: bool,
        want_civ: i32,
        remaining: i32,
        escrow: &mut i32,
        num_out: &mut i32,
    ) -> Option<(i32, i32, i32)> {
        let w = who as usize;
        let (units_now, queued_now) = self.raw_counts(who, f.rec);
        let unlimited = self.lobby.resources_unlimited();

        if !f.citizen && !f.caravan && !f.merchant && !f.special_forces && self.named_is(t, "Spy") {
            // Spy.
            if per_city || self.ai_difficulty() <= 2 {
                return None;
            }
            let combat = self.ai[w].census.combat;
            if combat <= 19 {
                return None;
            }
            let mut v = wm(remaining, 2000);
            let n = self.line_count(who, t, false) + queued_now;
            if n == 0 {
                *escrow = 1;
                v = wm(remaining, 200_000);
            }
            if combat < 15 * n {
                return None;
            }
            return Some((v, 6, want_civ));
        }
        if f.special_forces {
            if per_city {
                return None;
            }
            let combat = self.ai[w].census.combat;
            if combat <= 7 {
                return None;
            }
            let mut v = wm(wm(combat, remaining), 2000);
            let n = self.line_count(who, t, false) + queued_now;
            if n == 0 {
                *escrow = 1;
                if f.special_forces_elite {
                    v = wm(wm(combat, remaining), 20000);
                }
            }
            if combat < 8 * n {
                return None;
            }
            return Some((v, 6, want_civ));
        }
        if f.merchant {
            let market = self.city_chain(c).into_iter().any(|b| {
                let bd = &self.buildings[b];
                bd.alive
                    && bd.active
                    && bd
                        .ty
                        .is_some_and(|rec| self.build_is_ident(rec, Ident::Market))
            });
            if !market {
                return None;
            }
            // `has_tech(MATHEMATICS) || type_avail(KNOWLEDGE, 1)` — the
            // listing at `006c54c4..006c54e0` pushes `0x228` and then
            // `(3, 1)`, which the decompile prints as enum names.
            let roles = &self.tech_tree.roles;
            let maths = roles
                .mathematics
                .is_some_and(|m| self.tech_tree.has_tech(&self.setup, &self.tech[w], m));
            let knowledge = roles
                .knowledge
                .is_some_and(|k| self.type_avail(who, k) != tech::NOT_AVAILABLE);
            if !maths && !knowledge {
                return None;
            }
            // `known_rares` (`+0x6d4`), the sum `calc_gather` writes from
            // the census's step 9 — `docs/AI.md` §55. Signed: `jle` at
            // `006c5505`.
            if self.ai[w].known_rares - queued_now - units_now <= 0 {
                return None;
            }
            if self.ai[w].effective_pop >= self.muster[w].cap - 1 {
                return None;
            }
            *escrow = 1;
            return Some((1_000_000, 4, want_civ));
        }
        if f.caravan {
            if unlimited || self.named_is(t, "Merchant Fleet") || !per_city {
                return None;
            }
            let mut v = wm(self.ai[w].infra_mod, 1_000_000) / 256;
            let econ = self.ai[w].econ[2];
            let mut num = 1;
            if econ & 1 != 0 {
                num = 3;
                v = wm(v, 120);
            } else if econ & 2 != 0 {
                num = 2;
                v = wm(v, 60);
            } else if econ & 4 != 0 {
                v = wm(v, 30);
            }
            let limit = self.caravan_limit(who);
            num = num.min(limit - queued_now - units_now);
            if num <= 0 {
                return None;
            }
            *escrow = 1;
            // The caravan arm's own batch — `TVar24` is 3, 2 or 1 by
            // `econ[WEALTH]`'s bits and then capped by the caravan
            // limit. **Untested against a capture**: no run on disk has
            // an AI caravan offer on a frame the trace covers, so this
            // arm rests on `create_units@006c40a0:1080–1100` alone.
            *num_out = num;
            return Some((v, 4, want_civ));
        }
        if f.peasant {
            if !per_city {
                return None;
            }
            return self.citizen_value(
                who, t, f, c, city_o, base, r, slots, gfree, want_civ, escrow, num_out,
            );
        }
        if f.scholar {
            if unlimited || gfree[KNOWLEDGE] == 0 {
                return None;
            }
            // **The raw cap, not the difficulty-modified one.** The gate is
            // `create_units@006c40a0:1122–1127`, and both its operands are
            // fields of `LeaderDataEncrypt` read directly:
            // `+0xc ^ 0x8221` is `bucket[3]` and `+0x30 ^ 0x1281` is
            // `resource_cap[0]` (the type record: `bucket` is `int[6]` at
            // `+0x0`, `resource_cap` `int[7]` at `+0x30`, so the block is a
            // struct of arrays and the two offsets are good 3's bucket and
            // good 0's cap). `LeaderData::get_mod_resource_cap` is called
            // **twice** in the whole of `create_units`, at :1229 and :1232,
            // and both are the citizen branch's — neither is this one. On
            // Easiest the getter halves the cap, and halving it here is what
            // refused Great Lakes' scholar at sim-frame 8180: 159 knowledge
            // against a food cap of 2000 passes `(2000/16)*3/2 = 187` and
            // fails the halved `(1000/16)*3/2 = 93`. `docs/AI.md` §38.
            let cap_food = self.ledgers[w].cap[FOOD];
            if self.ledgers[w].bucket[KNOWLEDGE] > (cap_food / 16) * 3 / 2 {
                return None;
            }
            let uni = self.city_chain(c).into_iter().find(|&b| {
                let bd = &self.buildings[b];
                bd.alive
                    && bd.active
                    && bd
                        .ty
                        .is_some_and(|rec| self.build_is_ident(rec, Ident::University))
            })?;
            let k = gfree[KNOWLEDGE] - self.count_queue(uni, Some(t));
            if k <= 0 {
                return None;
            }
            // The scholar's batch is the free knowledge slots the
            // University's own queue has not already claimed
            // (`create_units@006c40a0:1156`). Great Lakes 8985 is the
            // frame that pins it: the original queues **three** scholars
            // there in one `make_this` — three `make_stuff+0x221` draws
            // and no `+0x63d` on either side — and this crate queued one
            // per pass until `k` reached `make_me`. `docs/AI.md` §42.
            *num_out = k;
            let filled = self.ai[w].census.filled_gather_slots[KNOWLEDGE];
            let total = self.ai[w].census.gather_slots[KNOWLEDGE];
            // The multiplier chain is [`scholar_value`], which quotes the
            // listing: **three independent `if`s**, the first and third
            // the same `+0x8c8 < +0x8b0` test, so a city below two thirds
            // takes ×60 ×10 ×5. `escrow` is the middle arm's and stays
            // here. `docs/AI.md` §53.
            if filled < total * 2 / 3 {
                *escrow = 1;
            }
            let v = scholar_value(self.ai[w].infra_mod, k, filled, total);
            return Some((v, 4, want_civ));
        }
        None
    }

    /// The citizen — the branch that decides whether a city wants another
    /// gatherer, and how badly (`create_units` 1181–1330).
    #[allow(clippy::too_many_arguments)]
    fn citizen_value(
        &mut self,
        who: Player,
        t: TypeId,
        f: &Facts,
        c: usize,
        city_o: usize,
        base: i32,
        r: u16,
        slots: i32,
        gfree: &[i32; RESOURCES],
        want_civ: i32,
        escrow: &mut i32,
        num: &mut i32,
    ) -> Option<(i32, i32, i32)> {
        let w = who as usize;
        let (units_now, queued_now) = self.raw_counts(who, f.rec);
        let mut b = wm(self.ai[w].infra_mod, base) / 256;
        let q = self.count_queue(city_o, None);
        let cn = &self.ai[w].census;
        if Census::reg(&cn.reg_free_peasants, r) >= Census::reg(&cn.reg_cities, r) {
            return None;
        }
        let ca = self.ai[w].city_ai.get(c).copied().unwrap_or_default();
        let (free, busy, gatherers) = (i32::from(ca.free), ca.busy, ca.gatherers);
        if busy == 0 && free == 0 && q == 0 {
            b = wm(b, 30);
        }
        let assigned = free + busy + q;
        let mut v;
        if assigned < slots - 1 {
            let mut deficit = slots - assigned;
            let mut room = 0;
            for (g, &gf) in gfree.iter().enumerate() {
                if g == KNOWLEDGE || !self.holdings[w].available[g] {
                    continue;
                }
                let inc = self.ledgers[w].income[g];
                let cap = self.mod_resource_cap(who, g);
                if inc < cap {
                    room += ((cap - inc) / 160).min(gf);
                } else {
                    deficit -= gf;
                }
            }
            *escrow = 1;
            let k = deficit.min(room).max(0);
            // `TVar24` again (`create_units@006c40a0:1215–1257`): the
            // gatherers this city is short of, bounded by the room the
            // caps leave. It is the same number the value squares.
            *num = k;
            v = wm(wm(wm(k, k), b), 80);
            if k == 0 && !(gatherers == 0 && busy == 0 && free == 0 && q == 0) {
                return None;
            }
        } else {
            if !(assigned < slots || free + busy < 3) {
                return None;
            }
            v = b;
        }
        // The multipliers.
        if !(free + q < 2 || slots != 0) {
            return None;
        }
        if self.lobby.victory == 8 {
            v = wm(v, 10);
        }
        let cn = &self.ai[w].census;
        let gath = Census::reg(&cn.reg_gatherers, r);
        let gslots = Census::reg(&cn.reg_gather_slots, r);
        if gath < gslots {
            v = wm(v, 3) / 2;
        }
        if gath * 2 < gslots {
            v = wm(v, 2);
        }
        if gatherers == 0 {
            v = wm(v, 2);
        }
        if free + busy <= slots / 2 {
            v = wm(v, 3) / 2;
        }
        if Census::reg(&cn.reg_peasants, r) == 0 {
            v = wm(v, 5);
        }
        if Census::reg(&cn.reg_free_peasants, r) == 0 {
            let town = self
                .tech_tree
                .roles
                .town
                .is_some_and(|x| self.type_avail(who, x) == tech::AVAILABLE);
            let cn = &self.ai[w].census;
            v = wm(
                v,
                if town && Census::reg(&cn.reg_land, r) < 2 {
                    4
                } else {
                    2
                },
            );
        }
        let n = queued_now + units_now;
        if n > self.muster[w].cap / 2 {
            v /= 2;
        }
        if n > 40 {
            v /= 2;
        }
        let _ = t;
        Some((v, 5, want_civ))
    }

    /// `Leaders::max_age()` — the highest age among leaders still in the
    /// game, which the rush-rules gate compares against.
    fn leaders_max_age(&self) -> i32 {
        (0..self.players.len())
            .filter(|&j| !self.defeated[j])
            .map(|j| self.tech[j].ages)
            .max()
            .unwrap_or(0)
    }

    /// `LeaderData::get_caravan_limit(1)`: the Commerce level plus one,
    /// bounded by the cities' pairings. The wonder, rare-resource and Nubian
    /// bonuses are inputs this does not carry.
    fn caravan_limit(&self, who: Player) -> i32 {
        let w = who as usize;
        let commerce = self.tech[w].epoch[tech::Line::Commerce.index()] + 1;
        let n = self.cities_of(who).len() as i32;
        99.min(commerce).min(n * (n - 1) / 2)
    }

    /// `elimination == 1` → the capital's owner is me.
    fn owns_capital(&self, who: Player) -> bool {
        self.cities
            .iter()
            .any(|c| c.alive && c.capital && c.owner == who)
    }

    /// `pop_limits[info.pop_limit].data[0]` — the lobby's population row, the
    /// bound three fifths of which caps civilians. It is the *row's* value,
    /// not the leader's modified cap, and [`crate::Muster::limit`] is the
    /// same number.
    fn civilian_ceiling(&self, who: Player) -> i32 {
        self.muster
            .get(who as usize)
            .map_or(crate::DEFAULT_POP_LIMIT, |m| m.limit)
    }

    /// `upgrade_units`: the unit upgrades, valued and offered.
    pub fn upgrade_units(&mut self, who: Player) {
        let w = who as usize;
        if self.lobby.elimination == 1 && !self.owns_capital(who) {
            return;
        }
        let d = self.ai_difficulty();
        let sea_map = self.world.sea_map();
        // `enemy_combat`: the strongest of those I am actively at war with,
        // else the strongest of the rest I am at war with at all.
        let mut active = 0;
        let mut other = 0;
        for j in 0..self.players.len() {
            let jw = j as Player;
            if jw == who || self.defeated[j] {
                continue;
            }
            if !(self.at_war[w][j] || self.at_war[j][w]) {
                continue;
            }
            let combat = self.ai[j].census.combat;
            if self.ai[w].census.active_wars_with & (1 << j) != 0 {
                active = active.max(combat);
            } else {
                other = other.max(combat);
            }
        }
        let enemy = if active != 0 { active } else { other };

        let units: Vec<TypeId> = self
            .tech_tree
            .types
            .iter()
            .enumerate()
            .filter(|(_, d)| d.kind.is_unit())
            .map(|(i, _)| i)
            .collect();
        // `LeaderData::city_num + LeaderData::village_num`, the two fields
        // the original adds (`upgrade_units@006c6430`, `+0x3f8` and
        // `+0x3fc`) — **not** a recount of which cities are still
        // villages. `village_num` is a declared seam and answers 0 here,
        // which is also the original's answer on run91's own frames
        // (`docs/AI.md` §36); the recount this used to do answered **2**
        // for Great Lakes' leader 1 and halved every value in the list.
        let settlements = (self.city_num(who) + self.village_num(who)).max(1);
        let pop = self.ai[w].census.pop;

        for t in units {
            if self.type_avail(who, t) != tech::RESEARCHABLE {
                continue;
            }
            let Some(f) = self.unit_facts(t) else {
                continue;
            };
            let Some(wt) = self.tech_tree.types[t].where_ else {
                continue;
            };
            let Some(wrec) = self.build_record(wt) else {
                continue;
            };
            if self.buildings_of_line(who, wrec) == 0 || self.researching(who, t) {
                continue;
            }
            // The predecessor chain: what I own of it and whether any of it
            // is available, and the age it came in at.
            let age_t = f.age;
            // **`age_p` is an accumulator the original zeroes, not a
            // default of my own age.** `6c660d xor edi,edi` sits between
            // the chain's head and the walk, and the walk's own guard is
            // `6c6643 test edi,edi / jne` — the accumulator itself, not a
            // "have I found one" flag. Two consequences this crate had
            // wrong: a type with **no available predecessor** gets `gap =
            // age_t - 0`, its own age, where defaulting `age_p = age_t`
            // gives zero; and a predecessor whose own age is zero leaves
            // the walk still looking. `avail` (`local_20`) is a separate
            // flag and is raised by *every* available predecessor.
            let mut age_p = 0;
            let mut owned = 0;
            let mut avail = false;
            let mut p =
                self.tech_tree
                    .get_graft(&self.setup, &self.tech[w], self.tech_tree.types[t].from);
            for _ in 0..64 {
                let Some(x) = p else { break };
                if let Some(rec) = self.unit_record(x) {
                    owned += self.muster[w].by_type[rec];
                }
                if self.type_avail(who, x) == tech::AVAILABLE {
                    avail = true;
                    if age_p == 0 {
                        age_p = self
                            .unit_record(x)
                            .map_or(0, |r| self.unit_types[r].combat.age);
                    }
                }
                p = self.tech_tree.get_graft(
                    &self.setup,
                    &self.tech[w],
                    self.tech_tree.types[x].from,
                );
            }
            if !(owned != 0 || !avail || f.siege) || self.named_is(t, "Merchant Fleet") {
                continue;
            }
            // The value. `k` is the recency factor, four for every
            // non-negative elapsed time (report §9.5).
            let base = pop * 1000 / settlements;
            let k = 4;
            let gap = age_t - age_p;
            let m = (5 * gap + 2).max(1);
            let mut v = wm(m, wm(wm(k, base) / 2, owned + 2) / 2);
            if self.ai[w].wonder_mod != 0 {
                v /= 100;
            }
            v = if f.sea {
                wm(self.ai[w].sea_mod, v) / 256
            } else if f.air {
                wm(self.ai[w].air_mod, v) / 256
            } else {
                wm(self.ai[w].ground_mod, v) / 256
            };
            if self
                .tech_tree
                .roles
                .nuclearmissile
                .is_some_and(|x| self.tech_tree.is(t, x, false))
            {
                let pers = self.ai[w].pers;
                v = wm(v, wm(sea_map / 2 + 1, pers.nukes + 2));
                if pers.nukes > 0 {
                    v = wm(v, 2);
                }
                if pers.rush < 0 && pers.army_size > 0 {
                    v = wm(v, 2);
                }
                if d > 2 && self.max_enemy_age(who) < self.tech[w].ages {
                    v = wm(v, 100);
                }
            }
            let combat = self.ai[w].census.combat;
            if combat >= 2 * enemy {
                v = wm(v, 4);
            } else if combat >= 3 * enemy / 2 {
                v = wm(v, 3);
            } else if combat > enemy {
                v = wm(v, 2);
            }
            if combat < 16 {
                if combat > 7 && !f.siege && owned == 0 {
                    v = wm(v, 5) / 4;
                }
            } else if combat < 24 {
                v = if !f.siege && owned == 0 {
                    wm(v, 3) / 2
                } else {
                    wm(v, 3)
                };
            } else {
                v = wm(v, if f.siege || owned != 0 { 4 } else { 2 });
            }
            v = v.clamp(0, 9_000_000);
            if d != 2 {
                let mut pv = self.unit_prod_value(who, t);
                if d == 0 || d > 2 {
                    pv = pv * 3 / 2;
                }
                self.mark(SITE_UPGRADE_BIAS);
                let roll = self.rng.roll();
                pv = (roll % 3 + 4) * pv / 5;
                if d < 2 {
                    pv = -pv;
                }
                pv = pv.clamp(-256, 256);
                v = wm(v / 256, pv + 256);
            }
            let cat;
            if f.military {
                if self.lobby.rush_rules == 8 {
                    continue;
                }
                cat = 7;
                let cn = &self.ai[w].census;
                if cn.active_wars != 0 {
                    v = wm(v, owned / 3 + 1);
                } else if cn.wars != 0 {
                    v = wm(v, owned / 6 + 6);
                }
                v = if f.siege {
                    wm(v, gap + 1)
                } else {
                    wm(gap + 2, v) / 2
                };
            } else {
                cat = 8;
                if self.ai[w].census.active_wars == 0 {
                    v = wm(v, 5) / 4;
                }
            }
            if self.ai[w].shortages == 0 {
                v = wm(v, 2);
            }
            if f.sea {
                v = wm(v, wm(sea_map, sea_map));
            }
            if self.ai[w].effective_pop > self.muster[w].cap - 4 {
                v = wm(v, 2);
            }
            let aff = if self.type_affordable(who, t, true) >= 1 {
                0x100
            } else {
                0x40
            };
            self.ai[w]
                .make_list
                .make_me(t as i32, wm(aff, v / 256), 1, cat, -1, 0, 1, 0, 0);
        }
    }

    /// `Leaders::max_enemy_age(who)`: the highest age among leaders not
    /// allied to me.
    fn max_enemy_age(&self, who: Player) -> i32 {
        (0..self.players.len())
            .filter(|&j| j as Player != who && !self.defeated[j] && !self.is_ally(who, j as Player))
            .map(|j| self.tech[j].ages)
            .max()
            .unwrap_or(0)
    }

    // ------------------------------------------------------------------
    // The military-trainer registry — `docs/AI.md` §29
    // ------------------------------------------------------------------

    /// `Wall::increment_stats@00643270`'s military-trainer arm: an **active**
    /// building whose type `is_military_trainer` joins its leader's
    /// `mil_trainers` (`LeaderData+0x6e50`), appended in activation order.
    ///
    /// The outer guard is the caller's — `Wall::activate@0063e4b0` line 62
    /// calls it only when the type carries `NO_CITY` or the building is
    /// complete and in a city — and `increment_stats` itself takes this arm
    /// only on the `is_active` side of its own split, which is why
    /// [`Sim::activate`] calls this *after* setting the flag.
    ///
    /// The predicate is [`build::is_military_trainer`], read on the root of
    /// the `FROM` chain, not the placement code's `Ident` list: an upgraded
    /// trainer is still one.
    pub(crate) fn mil_trainer_open(&mut self, b: usize) {
        let Some(ty) = self.buildings[b].ty else {
            return;
        };
        let bt = &self.build_types[ty];
        if !bt.has(flags::NO_CITY) && self.buildings[b].city.is_none() {
            return;
        }
        if !build::is_military_trainer(&self.build_types, ty) {
            return;
        }
        let w = self.buildings[b].owner as usize;
        if w >= self.ai.len() || self.ai[w].mil_trainers.contains(&b) {
            return;
        }
        self.ai[w].mil_trainers.push(b);
    }

    /// `Wall::decrement_stats@00642da0`'s arm, through
    /// `SimpleArray<int>::remove@00462e70` — which finds the first slot
    /// holding the value and **shifts the tail down**, so the list keeps its
    /// order rather than swapping with the last.
    pub(crate) fn mil_trainer_close(&mut self, b: usize) {
        let w = self.buildings[b].owner as usize;
        if w >= self.ai.len() {
            return;
        }
        if let Some(i) = self.ai[w].mil_trainers.iter().position(|&x| x == b) {
            self.ai[w].mil_trainers.remove(i);
        }
    }

    /// `produce_unit(t, city, num, escrow)`: `true` when queued (the
    /// original's 0). `city` is a sim city index, `None` for −1.
    pub fn produce_unit(
        &mut self,
        who: Player,
        t: TypeId,
        city: Option<usize>,
        num: i32,
        escrow: i32,
    ) -> bool {
        let Some(wt) = self.tech_tree.types[t].where_ else {
            return false;
        };
        let Some(wrec) = self.build_record(wt) else {
            return false;
        };
        let Some(rec) = self.unit_record(t) else {
            return false;
        };
        let fort = self.build_is_ident(wrec, Ident::Fort);
        if !self.is_military_trainer(wrec) && !fort {
            // City-trained: the best building of the city chain.
            let Some(c) = city else { return false };
            let level = self.city_level_of(c);
            let mut best = None;
            let mut best_score = 0;
            for b in self.city_chain(c) {
                let bd = &self.buildings[b];
                if !bd.alive || !bd.active || !bd.queue.has_room() {
                    continue;
                }
                let ok = self.building_of_type(b, wt)
                    || (self.tech_tree.roles.town == Some(wt) && self.building_is_city(b));
                if !ok {
                    continue;
                }
                let score = self.trainer_score(b, level * 1_000_000, true);
                if score > best_score {
                    best_score = score;
                    best = Some(b);
                }
            }
            return self.queue_batch(best, rec, num, escrow);
        }
        if fort {
            // Fort-trained: the leader's forts.
            let mut best = None;
            let mut best_score = 0;
            for b in 0..self.buildings.len() {
                let bd = &self.buildings[b];
                if bd.owner != who || !bd.alive || !bd.active || !bd.queue.has_room() {
                    continue;
                }
                if !self.building_of_type(b, wt) {
                    continue;
                }
                let score = self.trainer_score(b, 1_000_000, false);
                if score > best_score {
                    best_score = score;
                    best = Some(b);
                }
            }
            return self.queue_batch(best, rec, num, escrow);
        }
        // A military trainer: a harder AI widens the batch to what it can
        // afford, and each round re-picks the best trainer, so a batch
        // spreads over trainers as their queues fill.
        let mut num = num;
        if self.unit_types[rec].combat.attack != 0
            && !self.seam_missile(t)
            && self.ai_difficulty() > 1
        {
            num = self.type_affordable(who, t, escrow != 0).min(7);
        }
        if num < 1 {
            return false;
        }
        let mut first = None;
        for round in 0..num {
            let mut best = None;
            let mut best_score = 0;
            for &b in &self.ai[who as usize].mil_trainers.clone() {
                let Some(bd) = self.buildings.get(b) else {
                    continue;
                };
                if !bd.alive || !bd.active || bd.owner != who {
                    continue;
                }
                if !self.building_of_type(b, wt) || bd.queue.items.len() >= 4 {
                    continue;
                }
                let score = self.trainer_score(b, 1_000_000, false);
                if score > best_score {
                    best_score = score;
                    best = Some(b);
                }
            }
            let Some(b) = best else {
                // No candidate at all on the first round is the original's
                // "nothing could be queued"; later rounds keep the first
                // round's answer.
                return round != 0 && first == Some(true);
            };
            let ok = self.queue_up(b, rec).is_ok();
            if round == 0 {
                first = Some(ok);
            }
        }
        first == Some(true)
    }

    /// The trainer score both walks share: the city level or a flat million,
    /// halved (city) or fifthed (fort) when unassimilated, divided by the
    /// queue depth, the damage and the danger at its own half-cell.
    fn trainer_score(&self, b: usize, base: i32, city: bool) -> i32 {
        let mut s = base;
        if self.building_unassimilated(b) {
            s /= if city { 2 } else { 5 };
        }
        let bd = &self.buildings[b];
        let queued = bd.queue.items.len() as i32;
        let damage = if city { bd.damage / 50 } else { bd.damage };
        let danger = self.world.danger_at(bd.owner, bd.pos).max(0);
        s / (queued + 1) / (damage + 1) / (danger + 1)
    }

    /// `num` orders at the one building the walk chose; the first order's
    /// success is the result.
    ///
    /// **Seam** — `Build::queue_up(b, t, escrow)` spends from the escrow
    /// reservation when `escrow != 0`; [`Sim::queue_up`] takes no such
    /// argument and always spends from the ordinary purse.
    fn queue_batch(&mut self, at: Option<usize>, rec: usize, num: i32, escrow: i32) -> bool {
        let _ = escrow;
        let Some(b) = at else { return false };
        let mut first = false;
        for i in 0..num.max(1) {
            let ok = self.queue_up(b, rec).is_ok();
            if i == 0 {
                first = ok;
            }
        }
        first
    }

    /// `queued_units`: the population the leader's queues will add.
    pub fn queued_units(&self, who: Player) -> i32 {
        let mut n = 0;
        for b in &self.buildings {
            if b.owner != who || !b.alive || !b.active {
                continue;
            }
            for item in &b.queue.items {
                if item.tech.is_some() {
                    continue;
                }
                let u = &self.unit_types[item.ty];
                let avail = u
                    .tree
                    .is_none_or(|t| self.type_avail(who, t) >= tech::AVAILABLE);
                if avail {
                    n += u.price.pop;
                }
            }
        }
        n
    }

    /// `check_income(t, scale, o, escrow, city, num, &afford)` — the
    /// affordability factor in 256ths that every producer multiplies its
    /// value by. Each good the type costs removes `scale/256 × q` percent of
    /// the value, where `q` is the price in sixteenths-of-income units capped
    /// at twenty; a good with no income quarters it. An escrow purchase skips
    /// the income discount entirely, and a batch it cannot pay for at all is
    /// worth nothing unless escrow will cover it.
    ///
    /// The parameter names came from the first reading and sit one place off
    /// the original's: positionally this is `(t, scale, o, escrow, city, num,
    /// out)`, so `city` here is the original's object context, `o` its city
    /// context, `flag` its escrow and `escrow` the same escrow as an integer.
    /// Both contexts are seams — the cost model carries no per-object or
    /// per-city price — and `escrow_on` is `flag || escrow != 0`, which every
    /// caller satisfies either way.
    #[allow(clippy::too_many_arguments)]
    pub fn check_income(
        &self,
        who: Player,
        t: TypeId,
        scale: i32,
        city: Option<usize>,
        flag: bool,
        o: i32,
        num: i32,
        escrow: i32,
    ) -> i32 {
        let _ = (city, o);
        let w = who as usize;
        let escrow_on = flag || escrow != 0;
        let n = self.type_affordable(who, t, escrow_on);
        let mut f = 0x100;
        if n < num {
            if !escrow_on {
                return 0;
            }
            f = 0x40;
        }
        if escrow_on || self.lobby.resources_unlimited() {
            return f;
        }
        let Some(price) = self.type_price(who, t) else {
            return f;
        };
        for (g, &p) in price.iter().enumerate() {
            if !self.holdings[w].available[g] || p == 0 {
                continue;
            }
            let inc = self.ledgers[w].income[g] / 16;
            if inc < 1 {
                f /= 4;
            } else {
                let q = (p / inc).clamp(0, 20);
                f = (100 - (q * scale / 256)) * f / 100;
            }
        }
        f
    }

    /// `unit_prod_value(t)`: how much better `t` is against what the other
    /// leaders field than they are against it, weighted by their counts and
    /// capped at ten of each type. Zero for anything that is not a
    /// non-siege military type.
    fn unit_prod_value(&self, who: Player, t: TypeId) -> i32 {
        let Some(f) = self.unit_facts(t) else {
            return 0;
        };
        if !f.military || f.siege {
            return 0;
        }
        let d = self.ai_difficulty();
        let anti_air = self.unit_types[f.rec].combat.obj_masks & 0x8000_0000 != 0;
        let mut sum: i32 = 0;
        let mut total = 0;
        for j in 0..self.players.len() {
            let jw = j as Player;
            if jw == who || self.defeated[j] {
                continue;
            }
            if self.allied[w_index(who)][j] && self.allied[j][w_index(who)] {
                continue;
            }
            if !(d > 1 || self.seam_is_human(jw)) {
                continue;
            }
            for k in 0..self.unit_types.len() {
                let n = self.muster[j].by_type[k];
                if n == 0 {
                    continue;
                }
                let kp = &self.unit_types[k].combat;
                if !kp.combat_role {
                    continue;
                }
                let k_air = kp.domain == crate::attrition::Domain::Air;
                let counts = if !f.land {
                    // Not a land type: the domains must match, or `k` is air
                    // and `t` is anti-air.
                    same_domain(&self.unit_types[f.rec].combat, kp) || (k_air && anti_air)
                } else if anti_air {
                    k_air
                } else {
                    same_domain(&self.unit_types[f.rec].combat, kp) || (k_air && anti_air)
                };
                if !counts {
                    continue;
                }
                let weight = n.min(10);
                sum = sum.wrapping_add(wm(
                    self.table.pct(f.rec, k) - self.table.pct(k, f.rec),
                    weight,
                ));
                total += weight;
            }
        }
        if total == 0 { 0 } else { sum / total }
    }
}

/// `who` as an index — the matrices are `Vec<Vec<bool>>`.
const fn w_index(who: Player) -> usize {
    who as usize
}

/// Whether two profiles share a domain.
fn same_domain(a: &combat::Profile, b: &combat::Profile) -> bool {
    a.domain == b.domain
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::Leader;
    use crate::combat::Rng;

    #[test]
    fn the_war_multiplier_stacks_the_region_and_the_strategy() {
        // No war anywhere: the base, unchanged.
        assert_eq!(war_multiplier(100, 0, 0, 0, 0, 0, 0, false).0, 100);
        // A war somewhere but not here: 11/10.
        assert_eq!(war_multiplier(100, 0, 0, 0, 1, 0, 0, false).0, 110);
        // A war in this region, and one active: double, and the base itself
        // doubles with it.
        let (v, base) = war_multiplier(100, 1, 0, 1, 1, 0, 0, false);
        assert_eq!((v, base), (200, 200));
        // Only one of the three: three halves.
        assert_eq!(war_multiplier(100, 0, 1, 0, 0, 0, 0, false).0, 150);
        // The strategy word: bit 2 is three halves, bit 4 (or an attacked
        // region) is double, and a city-trained type in an attacked city is
        // quadruple.
        assert_eq!(war_multiplier(100, 0, 0, 0, 0, 2, 0, false).0, 150);
        assert_eq!(war_multiplier(100, 0, 0, 0, 0, 4, 0, false).0, 200);
        assert_eq!(war_multiplier(100, 0, 0, 0, 0, 0, 1, true).0, 400);
    }

    #[test]
    fn the_two_army_ladders_are_not_the_same() {
        // A tiny army: land opens ×6 then ×2, air ×2 then ×3/2.
        assert_eq!(army_ladder(100, 0, 100, true, false), 1200);
        assert_eq!(army_ladder(100, 0, 100, false, false), 300);
        // An army at four fifths of target clears the first rung only.
        assert_eq!(army_ladder(100, 60, 100, true, false), 200);
        assert_eq!(army_ladder(100, 60, 100, false, false), 150);
        // A big army: the last three rungs stack.
        assert_eq!(army_ladder(900, 300, 100, true, false), 900 * 2 / 3 / 2 / 2);
        // The region rung halves again.
        assert_eq!(army_ladder(100, 100, 100, true, true), 50);
    }

    #[test]
    fn the_batch_leaves_a_gap_below_the_population_cap() {
        // Ten population's worth of a one-pop unit, clipped by the cap left.
        assert_eq!(batch_size(1, 5, 0, 100), 5);
        assert_eq!(batch_size(1, 20, 0, 100), 10);
        assert_eq!(batch_size(2, 20, 0, 100), 5);
        // A control cost above ten is one at a time.
        assert_eq!(batch_size(20, 20, 0, 100), 1);
        // Near the cap the batch walks down to leave one free.
        assert_eq!(batch_size(1, 20, 95, 100), 3);
        assert_eq!(batch_size(1, 20, 99, 100), 1);
        assert_eq!(batch_size(0, 5, 0, 100), 1);
    }

    /// A sim with one player on a small land world, its AI leader armed.
    fn bare() -> Sim {
        let mut w = crate::world::World::new(16, 16);
        w.fill_region(
            Terrain::Land,
            crate::world::Cell::new(0, 0),
            crate::world::Cell::new(15, 15),
        );
        let mut sim = Sim::new(crate::Tuning::RON, w, 1);
        sim.ai = vec![Leader::new()];
        sim.ai[0].census.resize(sim.world.region_count(), 1);
        sim.ai[0].city_ai.resize(4, crate::ai::CityAi::default());
        sim
    }

    #[test]
    fn queued_units_sums_the_control_cost_of_what_is_ordered() {
        let mut sim = bare();
        let mut ty = crate::UnitType::default();
        ty.price.pop = 2;
        let a = sim.add_unit_type(ty);
        let mut ty = crate::UnitType::default();
        ty.price.pop = 3;
        let b = sim.add_unit_type(ty);
        let at = sim.add_building(0, crate::Pos::default(), 8);
        assert_eq!(sim.queued_units(0), 0);
        sim.queue_up(at, a).expect("queued");
        sim.queue_up(at, b).expect("queued");
        sim.queue_up(at, a).expect("queued");
        // Two of a two-pop type and one of a three-pop type.
        assert_eq!(sim.queued_units(0), 7);
        // A dead building's queue does not count.
        sim.buildings[at].alive = false;
        assert_eq!(sim.queued_units(0), 0);
    }

    #[test]
    fn check_income_is_a_factor_in_two_hundred_and_fifty_sixths() {
        let mut sim = bare();
        let mut ty = crate::UnitType::default();
        ty.price.base[0] = 100;
        let a = sim.add_unit_type(ty);
        let t = {
            let mut tree = tech::TechTree::new();
            let good = tree.add(tech::TypeDef::good("Food"));
            let u = tree.add(tech::TypeDef::unit("A", tech::UnitTraits::default()));
            let _ = good;
            tree.finalize();
            sim.set_tech_tree(tree);
            u
        };
        sim.unit_types[a].tree = Some(t);
        // Owned, so the price is the train arm's: an unowned type is priced
        // by its research (`docs/AI.md` §56).
        sim.tech[0].tech[t] = true;
        sim.holdings[0].available = [true; RESOURCES];
        sim.ledgers[0].bucket[0] = 1000;
        // No income at all: the factor is quartered once for the one good.
        sim.ledgers[0].income[0] = 0;
        assert_eq!(sim.check_income(0, t, 0x400, None, false, -1, 1, 0), 0x40);
        // With an income the discount is `4q` percent of the value, `q` the
        // price in sixteenths-of-income units, capped at twenty.
        sim.ledgers[0].income[0] = 10 * 16;
        let price = sim.type_price(0, t).expect("a unit has a price")[0];
        let q = (price / 10).clamp(0, 20);
        assert_eq!(
            sim.check_income(0, t, 0x400, None, false, -1, 1, 0),
            (100 - 4 * q) * 0x100 / 100
        );
        // A tenth of the price as income puts `q` under the cap.
        sim.ledgers[0].income[0] = price * 16;
        assert_eq!(
            sim.check_income(0, t, 0x400, None, false, -1, 1, 0),
            (100 - 4) * 0x100 / 100
        );
        // Escrow skips the discount entirely.
        assert_eq!(sim.check_income(0, t, 0x400, None, true, -1, 1, 0), 0x100);
        // A batch it cannot pay for is worth nothing without escrow, and a
        // quarter with it.
        assert_eq!(sim.check_income(0, t, 0x400, None, false, -1, 50, 0), 0);
        assert_eq!(sim.check_income(0, t, 0x400, None, true, -1, 50, 0), 0x40);
    }

    /// How many draws a closure takes out of the sync stream.
    fn draws(sim: &mut Sim, before: Rng) -> usize {
        let mut probe = before;
        let mut n = 0;
        while probe != sim.rng {
            probe.roll();
            n += 1;
            assert!(n < 10_000, "diverged");
        }
        n
    }

    /// **Great Lakes 9380's Scholar offer is a positive wrap** — item
    /// 422, `docs/AI.md` §49.
    ///
    /// `create_units` re-offers the Scholar on this frame and the number
    /// it files is **45,568**, which ranks below a Citizen's 234,782 and
    /// loses the purchase three frames later — the purchase that costs
    /// Great Lakes its sixth scholar at 9510 (§47, §48).
    ///
    /// 45,568 is not a valuation. It is the remainder of a 32-bit
    /// overflow, and the five terms are measured off the run rather than
    /// fitted:
    ///
    /// | term | value |
    /// | --- | --- |
    /// | `val` | 42,000,000 |
    /// | `fac` | 256 |
    /// | `want` | 20 |
    /// | `divisor` | 25 (`want` + 0 queued + 5 standing) |
    /// | `out` | **45,568** |
    ///
    /// `20 × 42,000,000 / 25` is 33,600,000; `256 ×` that is
    /// **8,601,600,000**, which does not fit an `i32`; it wraps to
    /// **11,665,408**, and `/256` is 45,568 exactly.
    ///
    /// **And the sign is the sting.** `docs/AI.md` §45's `out < 0 →
    /// 9,999,999` catches a wrap that lands negative — the ceiling that
    /// section is named for. This one lands **positive**, so the guard
    /// never fires and the overflow leaves behind a small, plausible
    /// number instead of an obvious one. The same defect as §45, in the
    /// arm next door, wearing the one disguise the guard cannot see.
    ///
    /// The original's own offer on that frame is **5,755,741**, and
    /// `5,755,741 × 256` is 1,473,469,696 — inside an `i32` with room, so
    /// it does not wrap. ~~Which of its three terms differs from this
    /// crate's is **not** established (§49).~~ **Closed by §53**: the term
    /// is `val`, and it was wrong twice over — `k` was 7 where the
    /// original's is 6 and 3 (§51's `count_gather_slots`), and the arm's
    /// third multiply was read as an `else if` where the listing has an
    /// independent `if`. With both corrected the arm hands the tail
    /// 180,000,000 and 90,000,000, and the tail returns the original's own
    /// two numbers exactly: [`the_scholar_arm_pays_great_lakes_9380_s_two_offers`].
    ///
    /// **`VAL = 42,000,000` below is therefore historical**, the value the
    /// arm produced before item 442. It is kept because the *wrap* it
    /// demonstrates is unchanged and load-bearing — the corrected chain
    /// wraps too, twice, and lands on the right answer.
    #[test]
    fn the_scholar_offer_on_great_lakes_9380_is_a_positive_wrap() {
        const VAL: i32 = 42_000_000;
        const FAC: i32 = 256;
        const WANT: i32 = 20;
        const DIVISOR: i32 = 25;
        // The offer as filed, and the arithmetic that produces it.
        assert_eq!(offer_value(FAC, WANT, VAL, DIVISOR), 45_568);
        let x = WANT * VAL / DIVISOR;
        assert_eq!(x, 33_600_000);
        // The product that does not fit, stated as the 64-bit number it
        // would be — this is the line that says "overflow" rather than
        // "small value".
        assert_eq!(i64::from(FAC) * i64::from(x), 8_601_600_000);
        assert!(i64::from(FAC) * i64::from(x) > i64::from(i32::MAX));
        // Where it lands, and that it lands **positive** — so the
        // original's own guard does not fire.
        assert_eq!(wm(FAC, x), 11_665_408);
        assert!(
            wm(FAC, x) > 0,
            "a negative wrap would be caught by the guard"
        );
        assert_eq!(wm(FAC, x) / 256, 45_568);
        // **The guard still works where the wrap is negative**, which is
        // what says this test is about the sign and not about the
        // expression being broken in general: §45's own row.
        assert_eq!(offer_value(FAC, 1, i32::MAX, 1), 9_999_999);
        // **The Citizen it has to beat, and does not.** Both sides offer
        // the Citizen at this value on the same block (run111), so the
        // ranking is what the wrap decides — asserted through
        // `offer_value` rather than between literals, which would be a
        // tautology that says nothing.
        const CITIZEN: i32 = 234_782;
        const THEIRS: i32 = 5_755_741;
        assert!(
            offer_value(FAC, WANT, VAL, DIVISOR) < CITIZEN,
            "the wrapped offer does not lose to the Citizen — then the \
             purchase at 9382 is decided by something else"
        );
        // **And the original's number does not wrap**, said through `wm`
        // rather than between constants: 5,755,741 × 256 survives the
        // same multiply that eats this crate's.
        assert_eq!(wm(FAC, THEIRS), 1_473_469_696);
        assert!(
            wm(FAC, THEIRS) > 0 && i64::from(wm(FAC, THEIRS)) == i64::from(FAC) * i64::from(THEIRS)
        );
    }

    /// **The scholar arm pays Great Lakes 9380's two offers, to the
    /// unit** — item 442, `docs/AI.md` §53. This is the item's whole
    /// claim as one assertion, and it is not a fit: both numbers were
    /// read off run114's `RON_LEADER_PROBE` trace (§52.2) before the
    /// arithmetic was done, and both fall out of the same two corrections.
    ///
    /// | | city 0 | city 1 |
    /// | --- | --- | --- |
    /// | seated scholars (run111's `inside_down`) | 4 | 1 |
    /// | `k = gfree[KNOWLEDGE] − count_queue` | **3** | **6** |
    /// | `scholar_value` | 90,000,000 | 180,000,000 |
    /// | `offer_value` | **4,891,136** | **5,755,741** |
    /// | the original's own offer (run114) | **4,891,136** | **5,755,741** |
    ///
    /// **Two corrections, and neither alone reaches either number.**
    /// `count_gather_slots` through [`crate::Sim::num_gatherers`] turns
    /// `k = 7` into 3 and 6 (§51); the third multiply as an independent
    /// `if` turns ×600 into ×3000 (§53.1). With only the first, the chain
    /// hands the tail 18,000,000 and 36,000,000, and **both clamp to
    /// 9,999,999** — which is the 925-frame loss item 438 measured and
    /// declined to land. With only the second, `k` is 7 for both cities
    /// and the two offers are equal, which is the tie §50 found buying the
    /// Scholar twice. The rows below assert each of those three worlds, so
    /// the test says *why* the pair is a pair.
    ///
    /// **Both corrected values still wrap.** 6's chain wraps negative at
    /// `want × val` and back positive at `× fac`; 3's wraps once. The
    /// ratio 5,755,741 / 4,891,136 = 1.1768 that §51.2 recorded as
    /// evidence against `val ∝ k` was an artefact of exactly that — the
    /// inputs *are* 2:1, and §49.2's lesson held: a number downstream of a
    /// wrap has no arithmetic relationship to its inputs.
    #[test]
    fn the_scholar_arm_pays_great_lakes_9380_s_two_offers() {
        // The frame's shared terms, measured off the run (§49.1) and
        // unchanged by this item: `infra_mod` 256, and the tail's
        // `fac`/`want`/`divisor`.
        const INFRA: i32 = 256;
        const FAC: i32 = 256;
        const WANT: i32 = 20;
        const DIVISOR: i32 = 25;
        // The arm's two branch tests are `Leader` fields, so they are
        // leader-global: both cities take all three multiplies on this
        // frame. `filled < total * 2 / 3` is the strictest of them.
        const FILLED: i32 = 0;
        const TOTAL: i32 = 14;
        const { assert!(FILLED < TOTAL * 2 / 3 && FILLED < TOTAL) };
        // City 0: four scholars seated in its University, so `k` is 3.
        assert_eq!(scholar_value(INFRA, 3, FILLED, TOTAL), 90_000_000);
        assert_eq!(
            offer_value(FAC, WANT, scholar_value(INFRA, 3, FILLED, TOTAL), DIVISOR),
            4_891_136,
            "the original's city-0 Scholar, off run114's own trace"
        );
        // City 1: one seated, so `k` is 6.
        assert_eq!(scholar_value(INFRA, 6, FILLED, TOTAL), 180_000_000);
        assert_eq!(
            offer_value(FAC, WANT, scholar_value(INFRA, 6, FILLED, TOTAL), DIVISOR),
            5_755_741,
            "the original's city-1 Scholar, off run114's own trace"
        );
        // **The wrap is still there, and it is why the ratio was a red
        // herring.** The inputs are exactly 2:1; the outputs are 1.1768:1.
        assert_eq!(
            scholar_value(INFRA, 6, FILLED, TOTAL),
            2 * scholar_value(INFRA, 3, FILLED, TOTAL)
        );
        assert!(i64::from(WANT) * 180_000_000_i64 > i64::from(i32::MAX));
        // **Half a fix reaches neither number.** `k = 7` for both cities
        // is the tree before §51's correction; with the third multiply in
        // place it clamps, and without it the two cities tie.
        assert_eq!(
            offer_value(FAC, WANT, scholar_value(INFRA, 7, FILLED, TOTAL), DIVISOR),
            9_999_999,
            "the uncorrected `k`, with the arm right — §45's ceiling"
        );
        // The `else if` reading, which is `scholar_value` without its
        // third multiply: both corrected `k`s clamp, which is item 438's
        // 36,000,000 / 18,000,000 and its 925 lost frames.
        let without_third = |k: i32| wm(wm(wm(wm(INFRA, k), 10000) / 256, 60), 10);
        assert_eq!(without_third(6), 36_000_000);
        assert_eq!(without_third(3), 18_000_000);
        for k in [3, 6] {
            assert_eq!(
                offer_value(FAC, WANT, without_third(k), DIVISOR),
                9_999_999,
                "the `else if` reading clamps for both cities — the 925 frames"
            );
        }
        // And with neither correction, the tie §50 measured: one number
        // for two cities, below the Citizen's 234,782.
        assert_eq!(offer_value(FAC, WANT, without_third(7), DIVISOR), 45_568);
    }

    #[test]
    fn create_units_offers_nothing_without_a_tree() {
        let mut sim = bare();
        let before = sim.rng;
        sim.create_units(0);
        assert_eq!(draws(&mut sim, before), 0);
        assert_eq!(sim.ai[0].make_list.head().t, -1);
    }

    /// The ids a [`barracks_sim`] hands back: the unit's tree entry, its
    /// record, the barracks' record, and the barracks building.
    struct Barracks {
        unit: TypeId,
        rec: usize,
        brec: usize,
        b: usize,
    }

    /// `find_building(city, SEARCH_FRIENDLY, who, 0x1800, …, DOCK)` — the
    /// sea branch's dock is the nearest of mine around the city, whichever
    /// city it belongs to, inside eight cells (`docs/AI.md` §57).
    #[test]
    fn the_sea_branch_s_dock_is_the_nearest_of_mine_around_the_city() {
        use crate::world::{Pos, UNITS_PER_CELL, UNITS_PER_TILE};
        let mut sim = bare();
        let dock = sim.build_types.len();
        sim.build_types.push(crate::build::BuildType {
            ident: Ident::Dock,
            ..crate::build::BuildType::default()
        });
        let add = |sim: &mut Sim, who: Player, tiles: (i32, i32)| {
            let b = sim.add_building(
                who,
                Pos::new(tiles.0 * UNITS_PER_TILE, tiles.1 * UNITS_PER_TILE),
                8,
            );
            sim.buildings[b].ty = Some(dock);
            b
        };
        let city = Pos::new(40 * UNITS_PER_TILE, 40 * UNITS_PER_TILE);
        assert_eq!(sim.find_dock_near(0, city), None, "no dock at all");
        // A dock of no city, twelve tiles out: found, although no city's
        // chain holds it — East Indies' `1/2010`.
        let far = add(&mut sim, 0, (52, 40));
        assert_eq!(sim.find_dock_near(0, city), Some(far));
        // A nearer one of mine wins; a nearer one of another leader's does
        // not; an unfinished one is not a candidate.
        let rival = add(&mut sim, 1, (42, 40));
        let near = add(&mut sim, 0, (46, 40));
        let site = add(&mut sim, 0, (44, 40));
        sim.buildings[site].active = false;
        assert_eq!(sim.find_dock_near(0, city), Some(near));
        let _ = rival;
        // Nine cells out is past `circle_radius[8]`: not found, however
        // near it would be in tiles to nothing else.
        for b in [far, near] {
            sim.buildings[b].alive = false;
        }
        let beyond = add(&mut sim, 0, (40 + 9 * UNITS_PER_CELL / UNITS_PER_TILE, 40));
        assert_eq!(sim.find_dock_near(0, city), None);
        sim.buildings[beyond].pos.x -= UNITS_PER_CELL;
        assert_eq!(
            sim.find_dock_near(0, city),
            Some(beyond),
            "eight cells is in"
        );
    }

    /// One leader with one city whose building is a Barracks — a military
    /// trainer — and one available land military type made there. Enough for
    /// a type to walk the whole of `create_units` and reach `make_me`.
    fn barracks_sim(upgradeable: bool) -> (Sim, Barracks) {
        let mut sim = bare();
        // The tree: six goods, one unit, one building.
        let mut tree = tech::TechTree::new();
        for n in ["Food", "Timber", "Wealth", "Knowledge", "Metal", "Oil"] {
            tree.add(tech::TypeDef::good(n));
        }
        let unit = tree.add(tech::TypeDef::unit(
            "Hoplites",
            tech::UnitTraits {
                combat: true,
                ..tech::UnitTraits::default()
            },
        ));
        let build = tree.add(tech::TypeDef::building("Barracks"));
        tree.types[unit].where_ = Some(build);
        tree.finalize();
        sim.set_tech_tree(tree);
        sim.tech[0].tech[unit] = !upgradeable;
        sim.tech[0].tech[build] = true;

        // The unit type: land, military, one population.
        let mut ty = crate::UnitType {
            tree: Some(unit),
            ..crate::UnitType::default()
        };
        ty.price.pop = 1;
        ty.combat.combat_role = true;
        ty.combat.attack = 100;
        ty.combat.domain = crate::attrition::Domain::Land;
        // Every shipped fighting unit belongs to a production group; the
        // muster's group counters are sized from this at registration.
        ty.group = Some(0);
        let rec = sim.add_unit_type(ty);

        // The building type: a military trainer (`BUILD_FLAGS 5`).
        let brec = sim.build_types.len();
        sim.build_types.push(crate::build::BuildType {
            ident: Ident::Barracks,
            tree: Some(build),
            flags: MILITARY_TRAINER,
            x_size: 2,
            y_size: 2,
            hits: 100,
            ..crate::build::BuildType::default()
        });

        let b = sim.add_building(0, crate::Pos::new(4 * 256, 4 * 256), 8);
        sim.buildings[b].ty = Some(brec);
        sim.cities.push(crate::city::City {
            alive: true,
            owner: 0,
            race: Some(0),
            founder: 0,
            building: b,
            members: Vec::new(),
            reg: sim.world.region_of(sim.buildings[b].pos.cell()),
            pos: sim.buildings[b].pos,
            capital: true,
            founding_capital: true,
            was_founding_capital: false,
            unassimilated: false,
            no_heal: false,
            alarm: false,
            no_muster: false,
            was_capital: 0,
            capture_stamp: 0,
            assimilation_timer: 0,
            attack_stamp: 0,
            capture_strength: 0,
            pop: 1,
            has_citizen: false,
            source: None,
            trade_val: 0,
            traded_with: [0; 8],
        });
        sim.buildings[b].city = Some(sim.cities.len() - 1);
        sim.ai[0]
            .city_ai
            .resize(sim.cities.len(), crate::ai::CityAi::default());
        sim.ai[0].mil_trainers = vec![b];
        sim.muster[0].cap = 100;
        sim.ai[0].census.pop = 10;
        for l in &mut sim.ledgers {
            l.bucket = [10_000; RESOURCES];
        }
        sim.holdings[0].available = [true; RESOURCES];
        (sim, Barracks { unit, rec, brec, b })
    }

    /// The registry `produce_unit` and `produce_tech` both read, and which
    /// nothing in this crate wrote until 2026-09-04 — `docs/AI.md` §29.
    #[test]
    fn a_trainer_joins_the_leader_s_list_on_activation_and_leaves_on_close() {
        let (mut sim, ids) = barracks_sim(false);
        let city = sim.buildings[ids.b].city;
        // The fixture files its own barracks by hand; this is about the
        // registry the building lifecycle keeps.
        sim.ai[0].mil_trainers.clear();
        sim.buildings[ids.b].active = false;

        // A second trainer, to make the order and the removal observable.
        let trainer = |sim: &mut Sim, x: i32| {
            let b = sim.add_building(0, crate::Pos::new(x * 256, 4 * 256), 8);
            sim.buildings[b].ty = Some(ids.brec);
            sim.buildings[b].city = city;
            b
        };
        let b2 = trainer(&mut sim, 8);
        let b3 = trainer(&mut sim, 12);
        let b4 = trainer(&mut sim, 12);

        // `Wall::increment_stats` files an *active* trainer, in activation
        // order, and `SimpleArray::add` never checks for a duplicate — but
        // `Wall::activate` only reaches it on the transition.
        sim.activate(ids.b, false, true);
        assert_eq!(sim.ai[0].mil_trainers, vec![ids.b]);
        sim.activate(b2, false, true);
        sim.activate(b3, false, true);
        sim.activate(b4, false, true);
        assert_eq!(sim.ai[0].mil_trainers, vec![ids.b, b2, b3, b4]);

        // `SimpleArray<int>::remove@00462e70` finds the first slot holding
        // the value and shifts the tail down, so closing one from the middle
        // leaves the order of the rest — it is not a swap with the last.
        sim.close_building(b2, false);
        assert_eq!(sim.ai[0].mil_trainers, vec![ids.b, b3, b4]);
        sim.close_building(b4, false);
        assert_eq!(sim.ai[0].mil_trainers, vec![ids.b, b3]);

        // Not a military trainer: never filed. `bare`'s type table has one
        // record with no flags at all.
        let plain = sim.add_build_type(crate::build::BuildType {
            ident: Ident::Granary,
            x_size: 2,
            y_size: 2,
            hits: 100,
            ..crate::build::BuildType::default()
        });
        let g = sim.add_building(0, crate::Pos::new(4 * 256, 12 * 256), 8);
        sim.buildings[g].ty = Some(plain);
        sim.buildings[g].city = city;
        sim.activate(g, false, true);
        assert_eq!(sim.ai[0].mil_trainers, vec![ids.b, b3]);

        // A trainer outside a city, with no `NO_CITY` on its type, is what
        // `Wall::activate`'s own guard refuses.
        let out = trainer(&mut sim, 4);
        sim.buildings[out].city = None;
        sim.activate(out, false, true);
        assert_eq!(sim.ai[0].mil_trainers, vec![ids.b, b3]);
        // With `NO_CITY` the same building would have joined.
        sim.build_types[ids.brec].flags |= crate::build::flags::NO_CITY;
        let free = trainer(&mut sim, 6);
        sim.buildings[free].city = None;
        sim.activate(free, false, true);
        assert_eq!(sim.ai[0].mil_trainers, vec![ids.b, b3, free]);
    }

    #[test]
    fn the_army_size_gate_draws_once_per_military_type_and_never_on_moderate() {
        for d in [0, 1, 2, 3, 4] {
            let (mut sim, ids) = barracks_sim(false);
            sim.lobby.difficulty = d;
            let before = sim.rng;
            sim.create_units(0);
            let n = draws(&mut sim, before);
            assert_eq!(n, usize::from(d != 2), "difficulty {d}");
            // And the type reached the make list, in the military category.
            assert_eq!(sim.ai[0].make_list.head().t, ids.unit as i32);
            assert_eq!(sim.ai[0].make_list.list[6].t, ids.unit as i32);
            assert_eq!(sim.ai[0].make_list.head().city, 0);
        }
    }

    #[test]
    fn upgrade_units_draws_once_per_eligible_type_and_never_on_moderate() {
        for d in [0, 1, 2, 3, 4] {
            // `type_avail == 2`: eligible, bit clear — an upgrade is bought
            // as research.
            let (mut sim, ids) = barracks_sim(true);
            assert_eq!(sim.type_avail(0, ids.unit), tech::RESEARCHABLE);
            sim.lobby.difficulty = d;
            let before = sim.rng;
            sim.upgrade_units(0);
            assert_eq!(
                draws(&mut sim, before),
                usize::from(d != 2),
                "difficulty {d}"
            );
            // A military upgrade lands in category 7.
            assert_eq!(sim.ai[0].make_list.list[7].t, ids.unit as i32);
            assert_eq!(sim.ai[0].make_list.list[7].escrow, 1);
        }
        // A type that is already available is not an upgrade at all.
        let (mut sim, _) = barracks_sim(false);
        let before = sim.rng;
        sim.upgrade_units(0);
        assert_eq!(draws(&mut sim, before), 0);
        assert_eq!(sim.ai[0].make_list.list[7].t, -1);
    }

    #[test]
    fn queued_units_ignores_a_type_the_tree_has_taken_away() {
        let (mut sim, ids) = barracks_sim(false);
        sim.queue_up(ids.b, ids.rec).expect("queued");
        assert_eq!(sim.queued_units(0), 1);
        sim.tech[0].tech[ids.unit] = false;
        assert_eq!(sim.queued_units(0), 0, "type_avail below 4 does not count");
    }

    /// **A batch's second unit pays the first one's ramp step**
    /// (2026-09-06).
    ///
    /// `LeaderData::get_support_count@006da110` answers
    /// `<group>_queued + <group>_units` for a type whose `PROGRESSION` is
    /// by-group — the same `queued + built` it answers by type — and
    /// `Leader::produce_unit` queues its `num` units **one at a time**, so
    /// the second is priced with the first already in the queue.
    ///
    /// [`Muster::by_group`] carried only the built half, so the group ramp
    /// stood still inside a batch. run76's `BUILDQUEUE` at dump-frame 6783
    /// is the receipt: the AI's Barracks holds two Longbowmen at
    /// `cost 31, 51` and `cost 33, 53`, and this crate charged 31/51
    /// twice. Two timber and two wealth, and two hundred frames later they
    /// were the difference between affording a University and not
    /// (`docs/AI.md` §30).
    #[test]
    fn a_batch_s_second_unit_pays_the_first_one_s_group_ramp_step() {
        let (mut sim, ids) = barracks_sim(false);
        // Triangular by group, one timber a step — the shape nearly every
        // fighting unit ships with.
        {
            let price = &mut sim.unit_types[ids.rec].price;
            price.progression = crate::cost::Progression::PROGRESSIVE_BY_GROUP;
            price.support = [Some((crate::economy::Resource::Timber, 1)), None];
        }
        sim.ledgers[0].bucket = [10_000; crate::economy::RESOURCES];
        let timber = crate::economy::Resource::Timber.index();

        let first = sim.price_of(0, ids.rec)[timber];
        sim.queue_up(ids.b, ids.rec).expect("the first is queued");
        let second = sim.price_of(0, ids.rec)[timber];
        sim.queue_up(ids.b, ids.rec).expect("the second is queued");
        let third = sim.price_of(0, ids.rec)[timber];
        assert!(
            first < second && second < third,
            "the queue is half the count: {first}, {second}, {third}"
        );
        assert_eq!(sim.muster[0].queued_by_group[0], 2, "and it is the queue");

        // And it comes back off. `Build::close`'s refund path is one of the
        // six that move `queued_by_type`, and every one of them moves this
        // beside it.
        let mut ledger = crate::economy::Ledger::default();
        sim.buildings[ids.b].queue.unqueue(0, false, &mut ledger);
        sim.muster[0].queued_by_type[ids.rec] -= 1;
        sim.track_queued_for_test(0, ids.rec, -1);
        assert_eq!(sim.muster[0].queued_by_group[0], 1);
        assert_eq!(sim.price_of(0, ids.rec)[timber], second);
    }

    #[test]
    fn produce_unit_queues_at_the_military_trainer_with_the_shallowest_queue() {
        let (mut sim, ids) = barracks_sim(false);
        // A second trainer, three deep; the first is empty.
        let b2 = sim.add_building(0, crate::Pos::new(8 * 256, 8 * 256), 8);
        sim.buildings[b2].ty = Some(ids.brec);
        sim.ai[0].mil_trainers = vec![b2, ids.b];
        for _ in 0..3 {
            sim.queue_up(b2, ids.rec).expect("queued");
        }
        assert!(sim.produce_unit(0, ids.unit, None, 1, 0));
        assert_eq!(sim.buildings[ids.b].queue.items.len(), 1, "the empty one");
        assert_eq!(sim.buildings[b2].queue.items.len(), 3);
        // A trainer already four deep is not a candidate at all.
        sim.ai[0].mil_trainers = vec![b2];
        sim.queue_up(b2, ids.rec).expect("queued");
        assert!(!sim.produce_unit(0, ids.unit, None, 1, 0));
        assert_eq!(sim.buildings[b2].queue.items.len(), 4);
    }

    /// **A batch goes in whole, in one call, down the city walk** — item
    /// 362. The military trainer's batch has been pinned since the
    /// barracks tests above; the *city-chain* walk, which is the one a
    /// scholar or a citizen takes, had none — and it is the walk the
    /// make list's `num` reaches. Great Lakes 8985 is what this is
    /// written for: the original queues **three** scholars there in a
    /// single `make_this` — three `make_stuff+0x221` draws and no
    /// `+0x63d` on either side — and until item 362 every civilian
    /// `make_me` carried a `num` of one, so this crate queued them one
    /// production cycle apart. `docs/AI.md` §42.
    ///
    /// Made to fail by handing `produce_unit` a `num` of 1: the queue
    /// then holds one entry, which is exactly the shape run97's
    /// `BUILDDATA` showed for 1,320 frames.
    #[test]
    fn produce_unit_queues_the_whole_batch_down_the_city_walk() {
        let (mut sim, ids) = barracks_sim(false);
        // Not a military trainer any more, so `produce_unit` takes the
        // city-chain branch; the building is still the type the unit is
        // made at, and it is the capital's own.
        sim.build_types[ids.brec].flags &= !MILITARY_TRAINER;
        sim.ai[0].mil_trainers.clear();
        assert!(sim.produce_unit(0, ids.unit, Some(0), 3, 0));
        assert_eq!(
            sim.buildings[ids.b].queue.items.len(),
            3,
            "the city walk queued {} of a batch of three",
            sim.buildings[ids.b].queue.items.len()
        );
        // A batch of one is still a batch of one: the plumbing the fix
        // added carries the number, it does not invent one.
        assert!(sim.produce_unit(0, ids.unit, Some(0), 1, 0));
        assert_eq!(sim.buildings[ids.b].queue.items.len(), 4);
    }
}
