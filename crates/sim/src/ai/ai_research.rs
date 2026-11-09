//! Research — `Leader::research_techs@006c6ba0` and
//! `Leader::produce_tech@006ca980`; `docs/AI.md` §2.14, §2.17.
//!
//! `research_techs` is one pass over the 85 technology types in `TypeIndex`
//! order. Each one that `Leader::tech_avail` calls researchable, and whose
//! research building the leader actually owns, is given a value — a base
//! from population and infrastructure, multiplied by the age or epoch
//! pacing, by the type's eleven `ai[]` weights against the census, by the
//! category, by the research building, by two recency factors and by the
//! affordability of its goods — and offered to the make list. Nothing is
//! bought here; step 8 spends.
//!
//! **The eleven weights.** `TechType::ai[11]` (`+0x1cc`, `short[11]`) is
//! **not** an XML column: `TechType::init` zeroes all eleven and
//! `Types::init@00669cc0:1239` then calls `TechType::compute_ai_values` on
//! every tech in ascending id order, which derives them from what the tech
//! unlocks — and, through `add_preq_ai`, adds into the *prerequisites*'
//! weights too, so the pass is order-dependent and cumulative. Reproducing
//! it needs the `+0xdc` "requires this type" predicate over units,
//! buildings, techs, goods, spells and bonus types, which is a loader job;
//! until `tech::TypeDef` carries the array, [`Sim::tech_ai_weights`] is a
//! seam answering all zeros (so `w = 1` and `val = base`). The worker's
//! report has the full derivation.
//!
//! **Seams**, each named at its definition: the eleven weights;
//! `leader_flags & 8` (a human the AI is driving); the `age_stamp` and
//! human-alive predicates; `village_num`; `get_mod_resource_cap`;
//! `get_gov`'s bonus test; a
//! building's "is upgrading" vslot; and `Build::queue_up`'s escrow argument.
//! (The `starting_resources == 7 && starting_technology == 8` lobby's
//! hard-coded branch is built — [`Sim::tech_race_shield`], item 1496.)

use crate::build::Ident;
use crate::economy::{OverCap, RESOURCES};
use crate::tech::{self, Kind, Line, TypeId};
use crate::{Player, Sim};

/// The Senate arm's two draws, named so the trace's sequence reads them
/// (`crate::mark`, `rondata::trace::SITES`): the coin that picks which
/// government column is dropped, taken only when `pers.raid == 0`, and the
/// survivor's `% 100` scale. Both are `Random::get(0, 0xffff)`; the
/// offsets are the return addresses of the two five-byte `call`s in the
/// listing, `6c77af` and `6c7812`. East Indies spends the second on frame
/// 15378 (item 643), where it read as a bare address until they were named.
pub const SITE_GOV_COIN: &str = "Leader::research_techs+0xc0f";
pub const SITE_GOV_ROLL: &str = "Leader::research_techs+0xc72";

/// Knowledge's slot in every per-good array — the good `research_techs`
/// prices a technology in.
const KNOWLEDGE: usize = 3;

/// How many `ai[]` weights a `TechType` carries.
pub(crate) const AI_WEIGHTS: usize = crate::ai_load::AI_WEIGHTS;

/// `BuildTypeData::is_military_trainer` — the flag is *derived*, not a
/// `BUILD_FLAGS` letter (`docs/DATALAYER.md`), and it is read on the root of
/// the `FROM` chain, which is what [`crate::build::is_military_trainer`] does.
#[cfg(test)]
const MILITARY_TRAINER: u32 = crate::build::flags::MILITARY_TRAINER;

/// `imul`: the original's scores overflow 32 bits in the deep multiplier
/// chains and wrap, so every product here wraps too.
const fn mul(a: i32, b: i32) -> i32 {
    a.wrapping_mul(b)
}

/// `(v + (v >> 31 & 0xff)) >> 8` — divide by 256, truncating toward zero,
/// the way the original's `*_mod` scalings do.
const fn shr8(v: i32) -> i32 {
    (v.wrapping_add((v >> 31) & 0xff)) >> 8
}

/// The census and lobby figures the weight block reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct WeightFacts {
    /// `info.team_style`: 0, 8 and 0xb add `ai[10]`, everything else
    /// subtracts it.
    pub team_style: i32,
    /// `city_num + village_num`.
    pub cities: i32,
    pub full_cities: i32,
    pub my_team_terr: i32,
    pub other_team_terr: i32,
    pub min_other_team_terr: i32,
    pub active_wars: i32,
    /// `world+0x34`, `sea_map` — the style's sea class (`World::sea_map`),
    /// which the weight cubes.
    pub sea_map: i32,
}

/// The weight multiplier `val = w × base` uses, from `research_techs`
/// 006c7300–006c7383.
///
/// `w = 1 + ai[0]/10 + Σ ai[1..=10]`, then the team-style sign on `ai[10]`,
/// the city and territory terms on `ai[5]`, the war term, and the landmass
/// term on `ai[2]`.
pub(crate) fn weight_total(ai: &[i32; AI_WEIGHTS], f: &WeightFacts) -> i32 {
    let mut w: i32 = 1;
    for (i, &a) in ai.iter().enumerate() {
        w = w.wrapping_add(if i == 0 { a / 10 } else { a });
    }
    // ± ai[10] by team style.
    w = w.wrapping_add(if matches!(f.team_style, 0 | 8 | 0xb) {
        ai[10]
    } else {
        -ai[10]
    });
    // Full cities.
    if f.cities / 2 < f.full_cities {
        w = w.wrapping_add(mul(ai[5], 2));
    } else if f.full_cities != 0 {
        w = w.wrapping_add(ai[5]);
    }
    // Behind on territory.
    if f.my_team_terr < f.other_team_terr {
        let t = if f.my_team_terr < f.min_other_team_terr {
            mul(ai[5], 2)
        } else {
            ai[5]
        };
        w = w.wrapping_add(t);
    }
    // At peace, or at war.
    let war = if f.active_wars == 0 {
        ai[5].wrapping_add(ai[1])
    } else {
        ai[0] / 3
    };
    // The sea term: no sea at all subtracts the naval weight.
    let m = f.sea_map;
    let sea = if m == 0 {
        -ai[2]
    } else {
        mul(mul(mul(ai[2], m), m), m)
    };
    war.wrapping_add(sea).wrapping_add(w)
}

/// The value's last step, and the **only** route a technology's offer takes
/// to the `9,999,999` ceiling: `income × val` in 32 bits, then `>> 8`, then
/// the original's own `val < 0 → 9,999,999` guard.
///
/// The multiply **wraps** (`imul`, `docs/audit/2026-08-25-ai.md` note 4), so
/// the ceiling is an *overflow* rather than a saturation — the guard catches
/// the wrapped sign. With `income` at its `0x40` commonest value the
/// threshold is `val > 33,554,431`, and Great Lakes' block 9179 sits either
/// side of it: the original's Mercenaries offer is `34,560,000` and clamps,
/// this crate's is `26,400,000` and does not (`docs/AI.md` §45).
pub(crate) fn income_scaled(income: i32, val: i32) -> i32 {
    let v = shr8(mul(income, val));
    if v < 0 { 9_999_999 } else { v }
}

/// `f1` and `f2`, the two recency factors: `min(cap, dt × ai_speed / div +
/// add + 1)`, written by the original as `dt/div + add < cap ? … + 1 : cap`.
///
/// Both stamps are only ever written by `Leader::init`, to zero, so on any
/// real game `dt ≥ 0` and these are the constants `f1 = 4`, `f2 = 3` —
/// `val ×= 3` net. Kept whole for fidelity.
pub(crate) fn recency(dt: i32, speed: i32, div: i32, add: i32, cap: i32) -> i32 {
    let q = mul(dt, speed) / div;
    if q.wrapping_add(add) < cap {
        q.wrapping_add(add).wrapping_add(1)
    } else {
        cap
    }
}

/// The make-list slot a category competes for.
pub(crate) const fn slot_for(cat: usize) -> i32 {
    match cat {
        0 => 10,
        1 => 9,
        2 => 4,
        _ => 8,
    }
}

/// The epoch quota by difficulty: `true` when the tech passes.
///
/// `n` is `techs_per_age` plus the category's bonus, `epochs` the total
/// library techs owned and `level` the line's own. Difficulty 3 and above
/// is uncapped.
pub(crate) const fn epoch_gate(diff: i32, n: i32, epochs: i32, level: i32) -> bool {
    match diff {
        0 | 1 => epochs < n + 2 && level < n,
        2 => epochs < n + 6 && level < n + 1,
        _ => true,
    }
}

/// What the human-pacing block does to one age tech.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AgePace {
    /// No human, or the leader is a human the AI drives: no gate.
    Unpaced,
    /// Too far ahead of the humans.
    Skip,
    /// Allowed once `frame − age_stamp ≥ delay`; an unstamped age passes.
    Delay(i32),
}

/// `research_techs` 006c6e90–006c6f28: difficulty 0 and 1 hold the AI one
/// age behind the best human and wait `(who + 16 | 8) × 225` frames after
/// the human got there; difficulty 2 allows the same age with a
/// `(who + 20) × 225` wait; 3 and above are unpaced.
pub(crate) const fn age_pace(diff: i32, a: i32, h: i32, who: i32) -> AgePace {
    match diff {
        // The original writes `a <= h − 1`; `h ≥ 0` here, so this is it.
        0 | 1 => {
            if a < h {
                AgePace::Delay((who + if diff == 0 { 16 } else { 8 }) * 225)
            } else {
                AgePace::Skip
            }
        }
        2 => {
            if a <= h {
                AgePace::Delay((who + 20) * 225)
            } else {
                AgePace::Skip
            }
        }
        _ => AgePace::Unpaced,
    }
}

/// The four gather-rate lines the hardest difficulties weight by shortage
/// (`research_techs` 006c7532–006c7645), by `NAME`. The original names six
/// `TypeIndex` runs; `Supercomputers`, the fifth knowledge tech, is
/// deliberately not among them.
const GATHER_LINES: [(usize, &[&str]); 4] = [
    (0, &["Agriculture", "Crop Rotation", "Food Industry"]),
    (1, &["Carpentry", "Logging Industry", "Papermill"]),
    (
        3,
        &[
            "Literacy",
            "Printing Press",
            "Scientific Method",
            "Institutional Research",
        ],
    ),
    (4, &["Metal Alloys", "Cold Casting", "Steel"]),
];

/// `0x24d..=0x250` — the Temple's taxation line, which takes a further
/// ×100 on top of every Temple tech's ×4. The Temple's other three
/// (Religion, Monotheism, Existentialism) do not.
const TAXATION: [&str; 4] = ["Taxation", "Vassalage", "Social Contract", "Income Tax"];

impl Sim {
    // ---- the seams ----

    /// `TechType::ai[11]`, derived at load by
    /// [`crate::ai_load::compute_ai_values`] from what each tech unlocks and
    /// checked against the program's own dump — `docs/DATALAYER.md`, "The
    /// derived words no column carries". A tree built by hand leaves them
    /// zero, which is the old seam's answer: `w` collapses to 1.
    fn tech_ai_weights(&self, t: TypeId) -> [i32; AI_WEIGHTS] {
        self.tables.tech_tree.types[t].ai.map(i32::from)
    }

    /// **Seam** — `leader_flags & 8`, a human leader the AI is driving.
    /// Only the human-pacing block reads it, and only to *disable* itself;
    /// answering `false` keeps every computer leader paced, which is what
    /// an ordinary game does.
    const fn ai_driven_human(&self, who: Player) -> bool {
        let _ = who;
        false
    }

    /// `Leaders::max_human_age`: the highest age any live human holds, −1
    /// if there is none. The original's predicate is `leader_flags & 6 ==
    /// 6`; `nation.human` is bit 2 and `!defeated` is bit 1.
    fn max_human_age(&self) -> i32 {
        let mut best = -1;
        for w in 0..self.tech.len() {
            if self.cities_state.nation[w].human
                && !self.leaders.defeated[w]
                && self.tech[w].ages >= best
            {
                best = self.tech[w].ages;
            }
        }
        best
    }

    /// `Leaders::best_human_age_stamp(t)`: the *earliest* frame any live
    /// human reached this age, −1 if none has. **Seam** — the original
    /// wants `leader_flags & 7 == 7`; bit 0 has no model here, so this uses
    /// the same human-and-alive pair `max_human_age` does.
    fn best_human_age_stamp(&self, n: usize) -> i64 {
        let mut best = -1i64;
        for w in 0..self.tech.len() {
            if !self.cities_state.nation[w].human || self.leaders.defeated[w] {
                continue;
            }
            if let Some(s) = self.tech[w].age_stamp[n]
                && s >= 0
                && (best < 0 || s <= best)
            {
                best = s;
            }
        }
        best
    }

    /// `types[0x2ae]->find_preq(t)` (`TRANSPORT_BONUS`, vslot `+0xdc`,
    /// `Type::find_preq@00668380`), which multiplies an epoch tech by 30 on
    /// a many-landmass map (`research_techs` `6c71a2`–`6c71c9`). `find_preq`
    /// reads slot 0 as written, so the bonus needs its `preq0` — Written
    /// Word in the shipped `rules.xml`, the tree's `roles.transport_preq`.
    /// Its slots 1 and 2 (the lobby's age remaps) are empty in the shipped
    /// row and not carried (`docs/AI.md` §304; item 3515, run1161's packet).
    fn transport_bonus_needs(&self, t: TypeId) -> bool {
        self.tables.tech_tree.roles.transport_preq == Some(t)
    }

    /// **Seam** — `LeaderData::get_gov() < 0`, which the original answers by
    /// testing the five government *bonus* types (`0x31d..0x325`) with
    /// `has_preq`. The tree has no bonus types; owning a government tech is
    /// the same answer in every ordinary game.
    fn has_no_government(&self, who: Player) -> bool {
        self.tables
            .tech_tree
            .govs_taken(&self.tables.setup, &self.tech[who as usize])
            == 0
    }

    /// `danger[who][half-cell of the building]`, clamped at zero, which
    /// divides a candidate producer's score in `produce_tech@006ca980:138`
    /// and `:236`. [`crate::danger`] writes it.
    fn danger_at(&self, who: Player, b: usize) -> i32 {
        self.world.danger_at(who, self.buildings[b].pos).max(0)
    }

    // ---- the small readings ----

    /// `LeaderData::get_lowest_epoch`: the least of the four library
    /// levels, capped at 99.
    fn lowest_epoch(&self, who: Player) -> i32 {
        let e = self.tech[who as usize].epoch;
        e.iter().fold(99, |a, &b| a.min(b))
    }

    /// The number of available goods whose `over_cap` marker is not the
    /// clean value — goods whose rate is pinned at the cap.
    fn goods_over_cap(&self, who: Player) -> i32 {
        (0..RESOURCES)
            .filter(|&g| {
                self.type_available(who, g)
                    && self.economy.ledgers[who as usize].over_cap[g] != OverCap::Under
            })
            .count() as i32
    }

    /// `TechType+0x14`, set by `TechType::set_research`: an epoch tech's
    /// line, and 3 for everything else — every age, plain, final and
    /// government tech.
    pub(crate) fn tech_cat(&self, t: TypeId) -> usize {
        match self.tables.tech_tree.kind(t) {
            Kind::Epoch { line, .. } => line.index(),
            _ => Line::Science.index(),
        }
    }

    /// `get_cost(KNOWLEDGE, who, …)` for a technology.
    fn knowledge_cost(&self, who: Player, t: TypeId) -> i32 {
        self.tech_price(who, t)[KNOWLEDGE]
    }

    /// The gate both the age and the epoch branches open with: a technology
    /// costing more than a third again the knowledge in hand is not
    /// considered at all. Skipped entirely when knowledge is unavailable.
    fn knowledge_out_of_reach(&self, who: Player, t: TypeId) -> bool {
        if !self.type_available(who, KNOWLEDGE) {
            return false;
        }
        let stock = self.economy.ledgers[who as usize].bucket[KNOWLEDGE];
        mul(stock, 4) / 3 < self.knowledge_cost(who, t)
    }

    /// A tech's name against a list, the way `rondata`'s loader resolves
    /// the same `TypeIndex` constants (`docs/DECISIONS.md` entry 18).
    fn tech_named_in(&self, t: TypeId, names: &[&str]) -> bool {
        let name = &self.tables.tech_tree.types[t].name;
        names.iter().any(|n| name.eq_ignore_ascii_case(n))
    }

    /// The good a gather-rate line raises, if `t` is on one.
    fn gather_line_good(&self, t: TypeId) -> Option<usize> {
        GATHER_LINES
            .iter()
            .find(|(_, names)| self.tech_named_in(t, names))
            .map(|(g, _)| *g)
    }

    /// `Leader::tech_avail@006da060`: 0 when the tech is masked off for me,
    /// had, or being researched — otherwise `LeaderData::type_avail(t, 1)`.
    /// The government-pair rule the original spells out here is already
    /// `type_avail`'s (`tech::TechTree::type_avail`, `Kind::Gov`).
    fn ai_tech_avail(&self, who: Player, t: TypeId) -> i32 {
        let def = &self.tables.tech_tree.types[t];
        if def.kind.is_tech() && def.leader_off & (1u8 << (who & 7)) != 0 {
            return 0;
        }
        if self
            .tables
            .tech_tree
            .has_tech(&self.tables.setup, &self.tech[who as usize], t)
            || self.researching(who, t)
        {
            return 0;
        }
        self.type_avail(who, t)
    }

    /// `LeaderData::find_capital` compared with me: the capital-countdown
    /// lobby only researches while the leader still holds its capital.
    fn holds_capital(&self, who: Player) -> bool {
        self.cities_state
            .cities
            .iter()
            .any(|c| c.alive && c.owner == who && c.capital)
    }

    /// The tech `0x243`, **Missile Shield**, when the tech-race branch of
    /// `research_techs` is live for `who`: the lobby is `STARTING_RESOURCES
    /// 7` with `STARTING_TECHNOLOGY 8` and the leader's raw bit for the tech
    /// is clear (the branch's own test is the bit, not `has_tech`'s
    /// prerequisite half).
    fn tech_race_shield(&self, who: Player) -> Option<TypeId> {
        if self.ai_state.lobby.starting_resources != 7
            || self.ai_state.lobby.starting_technology != 8
        {
            return None;
        }
        let shield = self
            .tables
            .tech_tree
            .types
            .iter()
            .position(|d| d.kind.is_tech() && d.name.eq_ignore_ascii_case("Missile Shield"))?;
        (!self.tech[who as usize].tech[shield]).then_some(shield)
    }

    // ---- research_techs ----

    /// `research_techs`: every eligible tech valued and offered to the make
    /// list. Draws from the sync stream for governments only (§2.14).
    pub fn research_techs(&mut self, who: Player) {
        let w = who as usize;
        if self.ai_state.lobby.elimination == 1 && !self.holds_capital(who) {
            return;
        }
        let low = self.lowest_epoch(who);
        let over = self.goods_over_cap(who);
        let techs: Vec<TypeId> = (0..self.tables.tech_tree.types.len())
            .filter(|&t| self.tables.tech_tree.kind(t).is_tech())
            .collect();
        for t in techs {
            if self.ai_tech_avail(who, t) < tech::AVAILABLE {
                continue;
            }
            // The research building, and one of it (or of what it upgrades
            // to) standing: `num_buildings[where] + get_buildings(upgrade)`.
            let Some(where_) = self.tables.tech_tree.types[t].where_ else {
                continue;
            };
            let Some(rec) = self.build_record(where_) else {
                continue;
            };
            if self.buildings_of_line(who, rec) == 0 {
                continue;
            }
            // **The tech-race lobby's hard-coded branch** (`research_techs`
            // `6c6ccb`..`6c6de7`, item 1496, `docs/AI.md` §129): under
            // `STARTING_RESOURCES 7` with `STARTING_TECHNOLOGY 8`, while the
            // leader's own bit for tech `0x243` (Missile Shield, the first
            // of the four final techs — `field_0x6c60 & 8` is bit 579 of the
            // tech mask) is clear, every candidate but that one is skipped
            // outright, and that one is **bought on the spot** when
            // `can_pay_cost(who, −1, −1, 1)` holds — `produce_tech(0x243,
            // 1)` — and valued as any other when it does not.
            if let Some(shield) = self.tech_race_shield(who) {
                if t != shield {
                    continue;
                }
                if self.type_affordable(who, shield, true) > 0 {
                    self.produce_tech(who, shield, 1);
                    continue;
                }
            }
            if let Some((val, slot)) = self.tech_value(who, t, rec, low, over) {
                self.ai_state.ai[w]
                    .make_list
                    .make_me(t as i32, val, 1, slot, -1, 0, 1, 0, 0);
            }
        }
    }

    /// One technology's value and make-list slot, or `None` when the pass
    /// rejects it. The order of the multipliers is the original's.
    #[allow(clippy::too_many_lines)]
    fn tech_value(
        &mut self,
        who: Player,
        t: TypeId,
        rec: usize,
        low: i32,
        over: i32,
    ) -> Option<(i32, i32)> {
        let w = who as usize;
        let diff = self.ai_difficulty();
        let kind = self.tables.tech_tree.kind(t);
        let cat = self.tech_cat(t);

        // `base = (pop × 200 / max(1, cities + villages)) × infra_mod / 256`.
        // **Seam** — `village_num` has no model, so this is the city count.
        let cities = self.city_num(who);
        let mut base = self.ai_state.ai[w].census.pop.wrapping_mul(200) / cities.max(1);
        base = shr8(mul(base, self.ai_state.ai[w].infra_mod));

        // *(The `starting_resources == 7 && starting_technology == 8` lobby's
        // branch stands in [`Sim::research_techs`], before this call — item
        // 1496.)*

        match kind {
            Kind::Age(n) => {
                if self.knowledge_out_of_reach(who, t) {
                    return None;
                }
                let a = i32::from(n);
                if !self.ai_driven_human(who) {
                    let h = self.max_human_age();
                    if h >= 0 {
                        match age_pace(diff, a, h, who as i32) {
                            AgePace::Skip => return None,
                            AgePace::Delay(delay) => {
                                // Difficulty 0 and 1 pass `t`; difficulty 2
                                // passes `t − 1` (`0x6c6e94`, `leal
                                // −0x1(%esi)`, called at `0x6c6eac`), so the
                                // harder setting waits on the **previous**
                                // age's stamp. For the first age `t − 1` is
                                // not an age type and the original's call
                                // returns −1, which passes the gate.
                                // (`docs/audit/2026-08-25-ai.md`, B3-d.)
                                let stamp = if diff == 2 {
                                    if a >= 1 {
                                        self.best_human_age_stamp((a - 1) as usize)
                                    } else {
                                        -1
                                    }
                                } else {
                                    self.best_human_age_stamp(n as usize)
                                };
                                if stamp >= 0 && i64::from(delay) > self.frame - stamp {
                                    return None;
                                }
                            }
                            AgePace::Unpaced => {}
                        }
                    }
                }
                base = mul(base, if a < low { 600 } else { 10 });
            }
            Kind::Epoch { line, level } => {
                if self.knowledge_out_of_reach(who, t) {
                    return None;
                }
                // The quota, unless this is The Art of War, the game ends
                // here, or the Spanish are on the Science line.
                let art_of_war = line == Line::Military && level == 0;
                let spanish = cat == 3 && self.tech[w].tribe == crate::ai::tribe::SPANISH;
                let ages = self.tech[w].ages;
                if ages < self.tables.setup.ending && !art_of_war && !spanish {
                    let age_ty = usize::try_from(ages)
                        .ok()
                        .and_then(|i| self.tables.tech_tree.ages.get(i).copied().flatten());
                    if let Some(age_ty) = age_ty {
                        let mut n = self.tables.tech_tree.techs_per_age(
                            &self.tables.setup,
                            &self.tech[w],
                            age_ty,
                        );
                        if cat == 0 {
                            if self.roster.muster[w].cap.wrapping_mul(7) / 8
                                <= self.ai_state.ai[w].effective_pop
                            {
                                n += 4;
                            }
                        } else if cat == 2 && over > 1 {
                            n += 4;
                        }
                        if !epoch_gate(diff, n, self.tech[w].epochs, self.tech[w].epoch[cat]) {
                            return None;
                        }
                    }
                }
                // Keep the four lines within one of each other.
                let level_now = self.tech[w].epoch[cat];
                if low == level_now {
                    base = mul(base, 10);
                } else if level_now > low + 1 {
                    return None;
                }
                if matches!(cat, 1 | 2) && matches!(self.ai_state.lobby.victory, 8 | 9) {
                    base = mul(base, 30);
                }
                if self.world.sea_map() > 1 && self.transport_bonus_needs(t) {
                    base = mul(base, 30);
                }
                let second_of_line = level == 1 && matches!(line, Line::Commerce | Line::Civic);
                if second_of_line && self.has_classical_age(who) {
                    base = mul(base, 20);
                }
            }
            _ => {}
        }

        // Knowledge comfortably in hand.
        if self.knowledge_cost(who, t) < self.economy.ledgers[w].bucket[KNOWLEDGE] - 100 {
            base = mul(base, 10);
        }

        // The eleven weights against the census.
        let ai = self.tech_ai_weights(t);
        let facts = WeightFacts {
            team_style: self.ai_state.lobby.team_style,
            cities,
            full_cities: self.ai_state.ai[w].census.full_cities,
            my_team_terr: self.ai_state.ai[w].census.my_team_terr,
            other_team_terr: self.ai_state.ai[w].census.other_team_terr,
            min_other_team_terr: self.ai_state.ai[w].census.min_other_team_terr,
            active_wars: self.ai_state.ai[w].census.active_wars,
            sea_map: self.world.sea_map(),
        };
        let mut val = mul(weight_total(&ai, &facts), base);

        // The category.
        match cat {
            0 => {
                val /= 10;
                let cap = self.roster.muster[w].cap;
                if cap < 200 {
                    if cap.wrapping_mul(5) / 6 < self.ai_state.ai[w].effective_pop {
                        val = mul(val, 20);
                    } else if self.tech[w].epoch[0] > 1 {
                        val /= 10;
                    }
                }
            }
            2 => {
                if self.tech[w].epoch[2] == 0 {
                    val = mul(val, 3);
                }
                // Only an epoch or an age tech is scaled by how many goods
                // sit at their cap — the Commerce line and the ages are
                // what raise the caps.
                if matches!(kind, Kind::Epoch { .. } | Kind::Age(_)) {
                    let k = self.goods_near_cap(who);
                    val = if k == 0 {
                        val / 10
                    } else {
                        mul(mul(val, k), k)
                    };
                }
            }
            _ => {
                if diff >= 4 && cat == 3 {
                    if let Some(g) = self.gather_line_good(t) {
                        let e = self.ai_state.ai[w].econ[g];
                        if e & 4 != 0 {
                            val = mul(val, 2);
                        }
                        if e & 2 != 0 {
                            val = mul(val, 4);
                        }
                        if e & 1 != 0 {
                            val = mul(val, 8);
                        }
                    }
                    let d = self.tech[w].epoch[3] - self.tables.tech_tree.types[t].age;
                    if d > 0 {
                        val = mul(mul(val, d + 1), d);
                    }
                }
            }
        }

        // The Tech Race.
        if self.ai_state.lobby.victory == 9 {
            val = mul(val, if matches!(kind, Kind::Age(_)) { 9 } else { 3 });
        }

        let mut slot = slot_for(cat);

        // The research building.
        let ident = self.tables.build_types[rec].ident;
        if ident == Ident::Tower {
            if diff < 2 {
                return None;
            }
            val = mul(val, 100);
            slot = 6;
        }
        if ident == Ident::Temple {
            val = mul(val, 4);
            if self.tech_named_in(t, &TAXATION) {
                val = mul(val, 100);
            }
        }
        if ident == Ident::Library {
            let e = self.tech[w].epoch;
            let m = e.iter().fold(0, |a, &b| a.max(b));
            val = mul(m, val).wrapping_add(1) / (e[cat] + 1);
        }
        if ident == Ident::University {
            val = mul(val, 1000);
        }
        if ident == Ident::Senate {
            // The coin (one draw, and only when `pers.raid == 0`) picks
            // which column of the three government pairs is thrown away;
            // whichever survives is then multiplied by a second draw.
            let raid = self.ai_state.ai[w].pers.raid;
            let drop_first = raid < 0
                || (raid == 0 && {
                    self.mark(SITE_GOV_COIN);
                    (self.rng.roll() & 1) != 0
                });
            let dropped = u8::from(!drop_first);
            if let Kind::Gov { column, .. } = kind
                && column == dropped
            {
                return None;
            }
            self.mark(SITE_GOV_ROLL);
            val = mul(val, self.rng.roll() % 100);
            if self.has_no_government(who) {
                val = mul(val, 20);
            }
        }

        // The two recency factors — both pinned constants in practice.
        let f1 = recency(
            self.frame_delta(self.ai_state.ai[w].tech_frame),
            self.ai_state.ai_speed,
            14400,
            12,
            4,
        );
        let f2 = recency(
            self.frame_delta(self.ai_state.ai[w].tech_cat_frame[cat]),
            self.ai_state.ai_speed,
            25600,
            16,
            3,
        );
        val = mul(f2, mul(f1, val) / 2) / 2;
        if self.ai_state.ai[w].shortages == 0 {
            val = mul(val, 2);
        }

        // Coverage: the goods the price is in that the leader can neither
        // gather nor reach.
        let cost = self.tables.tech_tree.types[t].cost;
        let mut total: i32 = 0;
        let mut missing: i32 = 0;
        for (g, &c) in cost.iter().enumerate() {
            total = total.wrapping_add(c);
            if !self.type_available(who, g)
                && !self
                    .tables
                    .tech_tree
                    .has_preq(&self.tables.setup, &self.tech[w], g)
            {
                missing = missing.wrapping_add(mul(c, 2));
            }
        }
        if missing != 0 {
            val = mul(mul(total, 2).wrapping_sub(missing), val) / mul(total, 2);
        }
        if val < 0 {
            val = 9_999_999;
        }
        let no_wonder = self.ai_state.ai[w].wonder_mod == 0;
        if !no_wonder {
            val /= 2;
        }
        let income = self.check_income(who, t, 0x400, None, no_wonder, -1, 1, 0);
        val = income_scaled(income, val);
        Some((val, slot))
    }

    /// `frame − stamp`, in the original's 32 bits.
    fn frame_delta(&self, stamp: i64) -> i32 {
        i32::try_from(self.frame - stamp).unwrap_or(i32::MAX)
    }

    /// Whether the leader holds the first age tech — `has_tech(CLASSICAL_AGE)`.
    fn has_classical_age(&self, who: Player) -> bool {
        self.tables.tech_tree.ages[0].is_some_and(|a| {
            self.tables
                .tech_tree
                .has_tech(&self.tables.setup, &self.tech[who as usize], a)
        })
    }

    /// How many available non-knowledge goods have an income at or above
    /// nine tenths of their cap — `get_mod_resource_cap`'s, which on an
    /// easy difficulty is smaller than the ledger's own
    /// ([`Sim::mod_resource_cap`], item 290).
    fn goods_near_cap(&self, who: Player) -> i32 {
        let l = &self.economy.ledgers[who as usize];
        (0..RESOURCES)
            .filter(|&g| {
                g != KNOWLEDGE
                    && self.type_available(who, g)
                    && self.mod_resource_cap(who, g).wrapping_mul(9) / 10 <= l.income[g]
            })
            .count() as i32
    }

    // ---- produce_tech ----

    /// `produce_tech(t, escrow)`: `true` when queued (the original's 0).
    ///
    /// The research building is `t.where`. A military trainer is searched
    /// in the leader's own `mil_trainers` list; anything else is searched
    /// over the member chain of every live city. **Seam** — `escrow` is
    /// `Build::queue_up`'s fourth argument, which [`Sim::queue_tech`] has
    /// no slot for yet; and a building's "is upgrading" vslot has no model,
    /// so no candidate is excluded for it.
    pub fn produce_tech(&mut self, who: Player, t: TypeId, escrow: i32) -> bool {
        let w = who as usize;
        // SEAM: the original asks `researching(t, -1, 0, 0)` here too
        // (`produce_tech@006ca980`), whose unit arm counts a queued rung of
        // `t`'s lineage ([`Sim::researching_unit`]); only `upgrade_units`
        // takes that arm so far (item 1379), and no walk here holds a unit
        // upgrade bought while a rung below it is in research.
        if self
            .tables
            .tech_tree
            .has_tech(&self.tables.setup, &self.tech[w], t)
            || self.researching(who, t)
        {
            return true;
        }
        let Some(where_) = self.tables.tech_tree.types[t].where_ else {
            return false;
        };
        let Some(rec) = self.build_record(where_) else {
            return false;
        };
        // `BuildTypeData::is_military_trainer` (`produce_tech`:33) reads the
        // flag on the **root** of the `FROM` chain, not on the type itself.
        let trainer = crate::build::is_military_trainer(&self.tables.build_types, rec);

        let mut best: Option<(i32, usize)> = None;
        if trainer {
            for b in self.ai_state.ai[w].mil_trainers.clone() {
                if !self.research_candidate(who, b, where_, t) {
                    continue;
                }
                let queued = self.buildings[b].queue.items.len() as i32;
                let mut score = 1_000_000 / (queued + 1) / (self.buildings[b].damage + 1);
                if self.building_unassimilated(b) {
                    score /= 2;
                }
                score /= self.danger_at(who, b) + 1;
                if best.is_none_or(|(v, _)| v < score) {
                    best = Some((score, b));
                }
            }
        } else {
            for c in self.cities_of(who) {
                let level = self.city_level_of(c);
                for b in self.city_chain(c) {
                    if !self.research_candidate(who, b, where_, t) {
                        continue;
                    }
                    let queued = self.buildings[b].queue.items.len() as i32;
                    let mut score = mul(level, 1_000_000) / (queued + 1);
                    if self.building_unassimilated(b) {
                        score /= 2;
                    }
                    score /= self.danger_at(who, b) + 1;
                    if best.is_none_or(|(v, _)| v < score) {
                        best = Some((score, b));
                    }
                }
            }
        }
        let Some((_, b)) = best else {
            return false;
        };
        // `Build::queue_up(t, escrow)` (`produce_tech:262`) is the one queue
        // for every kind, so a unit type's research — an upgrade the leader
        // does not own yet — is the unit queue's research job, priced by
        // `get_cost`'s research arm (`docs/AI.md` §56). `queue_tech` takes
        // only a technology and refused it.
        if self.tables.tech_tree.kind(t).is_unit() {
            let Some(rec) = self.unit_record(t) else {
                return false;
            };
            return self.queue_up_with(b, rec, escrow != 0).is_ok();
        }
        self.queue_tech_with(b, t, escrow != 0).is_ok()
    }

    /// The gates both searches share: alive, finished, of the research
    /// building's line, mine, and a queue that would accept the job.
    fn research_candidate(&self, who: Player, b: usize, where_: TypeId, t: TypeId) -> bool {
        let bd = &self.buildings[b];
        if !bd.alive || !bd.active || bd.owner != who || !bd.queue.has_room() {
            return false;
        }
        bd.ty
            .and_then(|r| self.tables.build_types[r].tree)
            .is_some_and(|bt| {
                self.tables.tech_tree.is(bt, where_, false)
                    && self.tables.tech_tree.queue_here(bt, t)
            })
    }
}

#[cfg(test)]
mod tests;
