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
//! `types[0x2ae]`'s many-landmasses predicate; `get_gov`'s bonus test; the
//! `starting_resources == 7 && starting_technology == 8` lobby's hard-coded
//! branch; a building's "is upgrading" vslot; and
//! `Build::queue_up`'s escrow argument.

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
        self.tech_tree.types[t].ai.map(i32::from)
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
            if self.nation[w].human && !self.defeated[w] && self.tech[w].ages >= best {
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
            if !self.nation[w].human || self.defeated[w] {
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

    /// **Seam** — `types[0x2ae]` (`TRANSPORT_BONUS`) asked whether it needs
    /// this tech, which multiplies an epoch tech by 30 on a many-landmass
    /// map. The tree carries no bonus types, so nothing needs it.
    const fn transport_bonus_needs(&self, t: TypeId) -> bool {
        let _ = t;
        false
    }

    /// **Seam** — `LeaderData::get_gov() < 0`, which the original answers by
    /// testing the five government *bonus* types (`0x31d..0x325`) with
    /// `has_preq`. The tree has no bonus types; owning a government tech is
    /// the same answer in every ordinary game.
    fn has_no_government(&self, who: Player) -> bool {
        self.tech_tree
            .govs_taken(&self.setup, &self.tech[who as usize])
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
                    && self.ledgers[who as usize].over_cap[g] != OverCap::Under
            })
            .count() as i32
    }

    /// `TechType+0x14`, set by `TechType::set_research`: an epoch tech's
    /// line, and 3 for everything else — every age, plain, final and
    /// government tech.
    fn tech_cat(&self, t: TypeId) -> usize {
        match self.tech_tree.kind(t) {
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
        let stock = self.ledgers[who as usize].bucket[KNOWLEDGE];
        mul(stock, 4) / 3 < self.knowledge_cost(who, t)
    }

    /// A tech's name against a list, the way `rondata`'s loader resolves
    /// the same `TypeIndex` constants (`docs/DECISIONS.md` entry 18).
    fn tech_named_in(&self, t: TypeId, names: &[&str]) -> bool {
        let name = &self.tech_tree.types[t].name;
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
        let def = &self.tech_tree.types[t];
        if def.kind.is_tech() && def.leader_off & (1u8 << (who & 7)) != 0 {
            return 0;
        }
        if self
            .tech_tree
            .has_tech(&self.setup, &self.tech[who as usize], t)
            || self.researching(who, t)
        {
            return 0;
        }
        self.type_avail(who, t)
    }

    /// `LeaderData::find_capital` compared with me: the capital-countdown
    /// lobby only researches while the leader still holds its capital.
    fn holds_capital(&self, who: Player) -> bool {
        self.cities
            .iter()
            .any(|c| c.alive && c.owner == who && c.capital)
    }

    // ---- research_techs ----

    /// `research_techs`: every eligible tech valued and offered to the make
    /// list. Draws from the sync stream for governments only (§2.14).
    pub fn research_techs(&mut self, who: Player) {
        let w = who as usize;
        if self.lobby.elimination == 1 && !self.holds_capital(who) {
            return;
        }
        let low = self.lowest_epoch(who);
        let over = self.goods_over_cap(who);
        let techs: Vec<TypeId> = (0..self.tech_tree.types.len())
            .filter(|&t| self.tech_tree.kind(t).is_tech())
            .collect();
        for t in techs {
            if self.ai_tech_avail(who, t) < tech::AVAILABLE {
                continue;
            }
            // The research building, and one of it (or of what it upgrades
            // to) standing: `num_buildings[where] + get_buildings(upgrade)`.
            let Some(where_) = self.tech_tree.types[t].where_ else {
                continue;
            };
            let Some(rec) = self.build_record(where_) else {
                continue;
            };
            if self.buildings_of_line(who, rec) == 0 {
                continue;
            }
            if let Some((val, slot)) = self.tech_value(who, t, rec, low, over) {
                self.ai[w]
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
        let kind = self.tech_tree.kind(t);
        let cat = self.tech_cat(t);

        // `base = (pop × 200 / max(1, cities + villages)) × infra_mod / 256`.
        // **Seam** — `village_num` has no model, so this is the city count.
        let cities = self.city_num(who);
        let mut base = self.ai[w].census.pop.wrapping_mul(200) / cities.max(1);
        base = shr8(mul(base, self.ai[w].infra_mod));

        // *(The `starting_resources == 7 && starting_technology == 8` lobby
        // buys tech `0x243` outright here; the option block carries no
        // `starting_technology`, so that branch is a **seam** — never
        // taken.)*

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
                if ages < self.setup.ending && !art_of_war && !spanish {
                    let age_ty = usize::try_from(ages)
                        .ok()
                        .and_then(|i| self.tech_tree.ages.get(i).copied().flatten());
                    if let Some(age_ty) = age_ty {
                        let mut n =
                            self.tech_tree
                                .techs_per_age(&self.setup, &self.tech[w], age_ty);
                        if cat == 0 {
                            if self.muster[w].cap.wrapping_mul(7) / 8 <= self.ai[w].effective_pop {
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
                if matches!(cat, 1 | 2) && matches!(self.lobby.victory, 8 | 9) {
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
        if self.knowledge_cost(who, t) < self.ledgers[w].bucket[KNOWLEDGE] - 100 {
            base = mul(base, 10);
        }

        // The eleven weights against the census.
        let ai = self.tech_ai_weights(t);
        let facts = WeightFacts {
            team_style: self.lobby.team_style,
            cities,
            full_cities: self.ai[w].census.full_cities,
            my_team_terr: self.ai[w].census.my_team_terr,
            other_team_terr: self.ai[w].census.other_team_terr,
            min_other_team_terr: self.ai[w].census.min_other_team_terr,
            active_wars: self.ai[w].census.active_wars,
            sea_map: self.world.sea_map(),
        };
        let mut val = mul(weight_total(&ai, &facts), base);

        // The category.
        match cat {
            0 => {
                val /= 10;
                let cap = self.muster[w].cap;
                if cap < 200 {
                    if cap.wrapping_mul(5) / 6 < self.ai[w].effective_pop {
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
                        let e = self.ai[w].econ[g];
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
                    let d = self.tech[w].epoch[3] - self.tech_tree.types[t].age;
                    if d > 0 {
                        val = mul(mul(val, d + 1), d);
                    }
                }
            }
        }

        // The Tech Race.
        if self.lobby.victory == 9 {
            val = mul(val, if matches!(kind, Kind::Age(_)) { 9 } else { 3 });
        }

        let mut slot = slot_for(cat);

        // The research building.
        let ident = self.build_types[rec].ident;
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
            let raid = self.ai[w].pers.raid;
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
            self.frame_delta(self.ai[w].tech_frame),
            self.ai_speed,
            14400,
            12,
            4,
        );
        let f2 = recency(
            self.frame_delta(self.ai[w].tech_cat_frame[cat]),
            self.ai_speed,
            25600,
            16,
            3,
        );
        val = mul(f2, mul(f1, val) / 2) / 2;
        if self.ai[w].shortages == 0 {
            val = mul(val, 2);
        }

        // Coverage: the goods the price is in that the leader can neither
        // gather nor reach.
        let cost = self.tech_tree.types[t].cost;
        let mut total: i32 = 0;
        let mut missing: i32 = 0;
        for (g, &c) in cost.iter().enumerate() {
            total = total.wrapping_add(c);
            if !self.type_available(who, g)
                && !self.tech_tree.has_preq(&self.setup, &self.tech[w], g)
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
        let no_wonder = self.ai[w].wonder_mod == 0;
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
        self.tech_tree.ages[0].is_some_and(|a| {
            self.tech_tree
                .has_tech(&self.setup, &self.tech[who as usize], a)
        })
    }

    /// How many available non-knowledge goods have an income at or above
    /// nine tenths of their cap — `get_mod_resource_cap`'s, which on an
    /// easy difficulty is smaller than the ledger's own
    /// ([`Sim::mod_resource_cap`], item 290).
    fn goods_near_cap(&self, who: Player) -> i32 {
        let l = &self.ledgers[who as usize];
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
        let _ = escrow;
        let w = who as usize;
        if self.tech_tree.has_tech(&self.setup, &self.tech[w], t) || self.researching(who, t) {
            return true;
        }
        let Some(where_) = self.tech_tree.types[t].where_ else {
            return false;
        };
        let Some(rec) = self.build_record(where_) else {
            return false;
        };
        // `BuildTypeData::is_military_trainer` (`produce_tech`:33) reads the
        // flag on the **root** of the `FROM` chain, not on the type itself.
        let trainer = crate::build::is_military_trainer(&self.build_types, rec);

        let mut best: Option<(i32, usize)> = None;
        if trainer {
            for b in self.ai[w].mil_trainers.clone() {
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
        if self.tech_tree.kind(t).is_unit() {
            let Some(rec) = self.unit_record(t) else {
                return false;
            };
            return self.queue_up(b, rec).is_ok();
        }
        self.queue_tech(b, t).is_ok()
    }

    /// The gates both searches share: alive, finished, of the research
    /// building's line, mine, and a queue that would accept the job.
    fn research_candidate(&self, who: Player, b: usize, where_: TypeId, t: TypeId) -> bool {
        let bd = &self.buildings[b];
        if !bd.alive || !bd.active || bd.owner != who || !bd.queue.has_room() {
            return false;
        }
        bd.ty
            .and_then(|r| self.build_types[r].tree)
            .is_some_and(|bt| {
                self.tech_tree.is(bt, where_, false) && self.tech_tree.queue_here(bt, t)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::{BuildType, flags};
    use crate::tech::{TechTree, TypeDef};
    use crate::world::{Terrain, UNITS_PER_TILE};
    use crate::{Cell, Player, Pos, Tuning, World};

    const TILE: i32 = UNITS_PER_TILE;

    /// **The only route a tech offer takes to the ceiling** — item 382,
    /// `docs/AI.md` §45. Great Lakes' block 9179 measured on both sides:
    /// Mercenaries' weights are `[152, 16, 4, 0, 10, 0, 0, 0, 44, 0, 0]`,
    /// so [`weight_total`] answers **110** at peace (`ai[5] + ai[1]`) and
    /// **144** at war (`ai[0] / 3`), and every other factor on the frame is
    /// shared. The pre-income values that come out of those two are what
    /// the dump shows either side of the overflow.
    #[test]
    fn the_tech_ceiling_is_the_income_multiply_wrapping_not_a_saturation() {
        // `income` on the frame, read off this crate's own `check_income`.
        const INCOME: i32 = 0x40;
        // The war weight and the peace weight, from the same `base`.
        const AT_WAR: i32 = 34_560_000;
        const AT_PEACE: i32 = 26_400_000;

        // The original's offer wraps 32 bits and the guard catches the sign.
        assert!(mul(INCOME, AT_WAR) < 0, "the product must overflow");
        assert_eq!(income_scaled(INCOME, AT_WAR), 9_999_999);
        // This crate's does not, and comes out a quarter of itself.
        assert!(mul(INCOME, AT_PEACE) > 0);
        assert_eq!(income_scaled(INCOME, AT_PEACE), 6_600_000);

        // The threshold, exactly: `2^31 / 0x40`.
        assert_eq!(income_scaled(INCOME, 33_554_431), 8_388_607);
        assert_eq!(income_scaled(INCOME, 33_554_432), 9_999_999);

        // And the two weights are the whole of the parting: 55/72.
        let ai = [152, 16, 4, 0, 10, 0, 0, 0, 44, 0, 0];
        let peace = WeightFacts {
            team_style: 0,
            cities: 2,
            full_cities: 0,
            my_team_terr: 568,
            other_team_terr: 266,
            min_other_team_terr: 266,
            active_wars: 0,
            sea_map: 1,
        };
        let war = WeightFacts {
            active_wars: 1,
            ..peace
        };
        assert_eq!(weight_total(&ai, &peace), 110);
        assert_eq!(weight_total(&ai, &war), 144);
        assert_eq!(i64::from(AT_PEACE) * 144, i64::from(AT_WAR) * 110);
    }

    fn tile_pos(tx: i32, ty: i32) -> Pos {
        Pos::new(tx * TILE + TILE / 2, ty * TILE + TILE / 2)
    }

    struct Types {
        village: usize,
        library: usize,
        temple: usize,
        university: usize,
        senate: usize,
        tower: usize,
        barracks: usize,
        classical: TypeId,
        science1: TypeId,
        commerce2: TypeId,
        military1: TypeId,
        taxation: TypeId,
        religion: TypeId,
        agriculture: TypeId,
        fortification: TypeId,
        gov_a: TypeId,
        gov_b: TypeId,
        /// A unit type at the Barracks that needs the Classical Age — an
        /// upgrade the leader researches before it trains one.
        phalanx: TypeId,
    }

    fn bt(ident: Ident, flag: &str) -> BuildType {
        BuildType {
            ident,
            x_size: 4,
            y_size: 4,
            flags: flags::parse(flag),
            job_time: 100,
            hits: 500,
            ..BuildType::default()
        }
    }

    /// A flat world, two leaders — 0 human, 1 the AI — with every research
    /// building available and at least one tech at each.
    fn sim() -> (Sim, Types) {
        let mut w = World::new(40, 40);
        w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
        let mut sim = Sim::new(Tuning::RON, w, 2);
        // These are the research coin's tests, and the coin's outcome is
        // the *seed's*. A building started here would plan its road
        // (`Wall::start`, `crate::roads` §1) and spend a draw a node before
        // the coin is tossed, which is a different question.
        sim.plan_roads = false;
        for l in &mut sim.ledgers {
            l.bucket = [10_000; RESOURCES];
            l.income = [100 * 16; RESOURCES];
            l.cap = [1000 * 16; RESOURCES];
        }
        let village = sim.add_build_type(bt(Ident::Village, "ean"));
        let library = sim.add_build_type(bt(Ident::Library, "jam"));
        let temple = sim.add_build_type(bt(Ident::Temple, "jam"));
        let university = sim.add_build_type(bt(Ident::University, "jam"));
        let senate = sim.add_build_type(bt(Ident::Senate, "jam"));
        let tower = sim.add_build_type(bt(Ident::Tower, "ean"));
        let barracks = {
            let mut b = bt(Ident::Barracks, "ean");
            b.flags |= MILITARY_TRAINER;
            sim.add_build_type(b)
        };

        let mut tree = TechTree::new().with_tuning(&Tuning::RON);
        for name in ["Food", "Timber", "Wealth", "Knowledge", "Metal", "Oil"] {
            let mut d = TypeDef::good(name);
            d.tribe_mask = u32::MAX;
            tree.add(d);
        }
        let recs = [
            village, library, temple, university, senate, tower, barracks,
        ];
        let names = [
            "Village",
            "Library",
            "Temple",
            "University",
            "Senate",
            "Tower",
            "Barracks",
        ];
        let mut ids = Vec::new();
        for name in names {
            let mut d = TypeDef::building(name);
            d.tribe_mask = u32::MAX;
            ids.push(tree.add(d));
        }
        let (lib_id, temple_id, uni_id, senate_id, tower_id) =
            (ids[1], ids[2], ids[3], ids[4], ids[5]);
        let barracks_id = ids[6];

        let age_names = [
            "Classical Age",
            "Medieval Age",
            "Gunpowder Age",
            "Enlightenment Age",
            "Industrial Age",
            "Modern Age",
            "Information Age",
        ];
        let mut ages = Vec::new();
        for (n, name) in age_names.iter().enumerate() {
            ages.push(tree.add(TypeDef::age(name, n as u8).at(lib_id)));
        }
        let mut epochs = Vec::new();
        for line in Line::ALL {
            for level in 0..7u8 {
                let name = format!("{line:?} {level}");
                epochs.push(tree.add(TypeDef::epoch(&name, line, level).at(lib_id)));
            }
        }
        let taxation = tree.add(TypeDef::plain("Taxation", 0).at(temple_id));
        let religion = tree.add(TypeDef::plain("Religion", 0).at(temple_id));
        let agriculture = tree.add(TypeDef::plain("Agriculture", 0).at(uni_id));
        let fortification = tree.add(TypeDef::plain("Fortification", 0).at(tower_id));
        let mut govs = Vec::new();
        for (i, name) in [
            "Empire",
            "Republic",
            "Monarchy",
            "Democracy",
            "Socialism",
            "Capitalism",
        ]
        .iter()
        .enumerate()
        {
            govs.push(tree.add(TypeDef::gov(name, (i / 2) as u8, (i % 2) as u8).at(senate_id)));
        }
        let mut phalanx = TypeDef::unit("Phalanx", tech::UnitTraits::default())
            .at(barracks_id)
            .needs(0, ages[0]);
        phalanx.tribe_mask = u32::MAX;
        let phalanx = tree.add(phalanx);

        sim.set_tech_tree(tree);
        for (rec, id) in recs.iter().zip(ids.iter()) {
            sim.build_types[*rec].tree = Some(*id);
        }
        sim.nation[0].human = true;
        let t = Types {
            village,
            library,
            temple,
            university,
            senate,
            tower,
            barracks,
            classical: ages[0],
            science1: epochs[Line::Science.index() * 7],
            commerce2: epochs[Line::Commerce.index() * 7 + 1],
            military1: epochs[Line::Military.index() * 7],
            taxation,
            religion,
            agriculture,
            fortification,
            gov_a: govs[0],
            gov_b: govs[1],
            phalanx,
        };
        (sim, t)
    }

    fn city(sim: &mut Sim, t: &Types, who: Player, tx: i32, ty: i32) -> usize {
        let b = sim
            .place_building(who, t.village, tile_pos(tx, ty))
            .expect("the city places");
        while !sim.buildings[b].active && sim.buildings[b].alive {
            sim.do_construct(b, 1_000_000);
        }
        let c = sim.buildings[b].city.expect("a finished city has a record");
        let w = who as usize;
        sim.ai[w]
            .census
            .resize(sim.world.region_count(), sim.players.len());
        sim.ai[w]
            .city_ai
            .resize(sim.cities.len(), Default::default());
        sim.ai[w].census.pop = 20;
        c
    }

    fn building(sim: &mut Sim, who: Player, rec: usize, tx: i32, ty: i32) -> usize {
        let b = sim
            .place_building(who, rec, tile_pos(tx, ty))
            .expect("it places");
        while !sim.buildings[b].active && sim.buildings[b].alive {
            sim.do_construct(b, 1_000_000);
        }
        b
    }

    /// Keeps the scores small enough to read: one head of population and a
    /// shortage, so the deep multiplier chains do not wrap the way the
    /// original's do.
    fn small(sim: &mut Sim, who: Player) {
        sim.ai[who as usize].census.pop = 1;
        sim.ai[who as usize].shortages = 1;
    }

    /// The value one tech would be offered at, straight from the scorer.
    fn value(sim: &mut Sim, who: Player, t: TypeId) -> Option<(i32, i32)> {
        let where_ = sim.tech_tree.types[t].where_.expect("a research building");
        let rec = sim.build_record(where_).expect("a building record");
        let low = sim.lowest_epoch(who);
        let over = sim.goods_over_cap(who);
        sim.tech_value(who, t, rec, low, over)
    }

    fn offered(sim: &Sim, who: Player) -> Vec<i32> {
        sim.ai[who as usize]
            .make_list
            .list
            .iter()
            .map(|m| m.t)
            .filter(|&x| x >= 0)
            .collect()
    }

    // ---- the pure arithmetic ----

    #[test]
    fn the_weight_block_collapses_to_one_with_no_weights() {
        assert_eq!(weight_total(&[0; AI_WEIGHTS], &WeightFacts::default()), 1);
    }

    #[test]
    fn the_weight_block_sums_the_eleven_with_ai_zero_tenthed() {
        let mut ai = [0; AI_WEIGHTS];
        ai[0] = 30;
        ai[3] = 5;
        let f = WeightFacts {
            active_wars: 1,
            ..WeightFacts::default()
        };
        // 1 + 30/10 + 5 = 9, the war term is ai[0]/3 = 10, no sea.
        assert_eq!(weight_total(&ai, &f), 19);
    }

    #[test]
    fn peace_adds_ai_five_and_ai_one_where_war_adds_a_third_of_ai_zero() {
        let mut ai = [0; AI_WEIGHTS];
        ai[0] = 30;
        ai[1] = 2;
        ai[5] = 4;
        let war = WeightFacts {
            active_wars: 1,
            ..WeightFacts::default()
        };
        let peace = WeightFacts {
            active_wars: 0,
            ..WeightFacts::default()
        };
        // The sum is 1 + 3 + 2 + 4 = 10 either way.
        assert_eq!(weight_total(&ai, &war), 10 + 10);
        assert_eq!(weight_total(&ai, &peace), 10 + 4 + 2);
    }

    #[test]
    fn the_team_style_sign_flips_on_ai_ten() {
        let mut ai = [0; AI_WEIGHTS];
        ai[10] = 7;
        let base = WeightFacts {
            active_wars: 1,
            ..WeightFacts::default()
        };
        for style in [0, 8, 0xb] {
            let f = WeightFacts {
                team_style: style,
                ..base
            };
            assert_eq!(weight_total(&ai, &f), 1 + 7 + 7);
        }
        let f = WeightFacts {
            team_style: 1,
            ..base
        };
        assert_eq!(weight_total(&ai, &f), 1 + 7 - 7);
    }

    #[test]
    fn full_cities_and_territory_ride_on_ai_five() {
        let mut ai = [0; AI_WEIGHTS];
        ai[5] = 10;
        let base = WeightFacts {
            team_style: 1,
            active_wars: 1,
            ..WeightFacts::default()
        };
        let f = WeightFacts {
            cities: 4,
            full_cities: 3,
            ..base
        };
        assert_eq!(weight_total(&ai, &f), 1 + 10 + 20);
        let f = WeightFacts {
            cities: 4,
            full_cities: 1,
            ..base
        };
        assert_eq!(weight_total(&ai, &f), 1 + 10 + 10);
        let f = WeightFacts {
            my_team_terr: 1,
            other_team_terr: 9,
            min_other_team_terr: 5,
            ..base
        };
        assert_eq!(weight_total(&ai, &f), 1 + 10 + 20);
    }

    #[test]
    fn the_sea_term_is_cubed_and_negated_on_a_dry_map() {
        let mut ai = [0; AI_WEIGHTS];
        ai[2] = 2;
        let f = WeightFacts {
            team_style: 1,
            active_wars: 1,
            sea_map: 3,
            ..WeightFacts::default()
        };
        assert_eq!(weight_total(&ai, &f), 1 + 2 + 2 * 27);
        let f = WeightFacts { sea_map: 0, ..f };
        assert_eq!(weight_total(&ai, &f), 1 + 2 - 2);
    }

    #[test]
    fn the_recency_factors_are_four_and_three_in_any_real_game() {
        for frame in [0, 1, 100, 100_000] {
            assert_eq!(recency(frame, 1, 14400, 12, 4), 4);
            assert_eq!(recency(frame, 1, 25600, 16, 3), 3);
        }
        // `f2 × (f1 × val / 2) / 2` is `val × 3`.
        let v = 1000;
        assert_eq!(mul(3, mul(4, v) / 2) / 2, 3 * v);
    }

    #[test]
    fn the_recency_factors_only_fall_below_their_cap_on_a_future_stamp() {
        assert_eq!(recency(-8 * 14400, 1, 14400, 12, 4), 4);
        assert_eq!(recency(-9 * 14400, 1, 14400, 12, 4), 4);
        assert_eq!(recency(-10 * 14400, 1, 14400, 12, 4), 3);
        assert_eq!(recency(-13 * 25600, 1, 25600, 16, 3), 3);
        assert_eq!(recency(-14 * 25600, 1, 25600, 16, 3), 3);
        assert_eq!(recency(-15 * 25600, 1, 25600, 16, 3), 2);
    }

    /// `get_cost`'s British taxation term (`docs/AI.md` §74). run227's
    /// who=1, the British, held 106 food and 76 timber on block 16779 and
    /// priced Taxation at 88 each here — `8 × TECH_COST_FACTOR` and the
    /// Science surcharge — so `check_income`'s escrow arm read it
    /// unaffordable (`0x40`) where the original's, at half the price, read
    /// `0x100`: the offer's value 22784 against 91136.
    #[test]
    fn the_british_pay_half_for_the_taxation_line() {
        let (mut sim, t) = sim();
        for x in [t.taxation, t.religion] {
            sim.tech_tree.types[x].cost = [8, 8, 0, 0, 0, 0];
        }
        sim.tech_tree.roles.taxation_line = vec![t.taxation];
        sim.holdings[1].available = [true, true, true, true, true, false];
        sim.ledgers[1].bucket = [106, 76, 89, 299, 20, 0];
        sim.tech[1].has_city = true;
        let income = |sim: &Sim, x| sim.check_income(1, x, 0x400, None, true, -1, 1, 0);
        assert_eq!(sim.tech_price(1, t.taxation), [88, 88, 0, 0, 0, 0]);
        assert_eq!(
            income(&sim, t.taxation),
            0x40,
            "not British: 88 is out of reach"
        );
        sim.tech[1].power = Some(0xb);
        assert_eq!(sim.tech_price(1, t.taxation), [44, 44, 0, 0, 0, 0]);
        assert_eq!(income(&sim, t.taxation), 0x100, "British: 44 is in reach");
        // Only the four taxation techs, and only for the British.
        assert_eq!(sim.tech_price(1, t.religion), [88, 88, 0, 0, 0, 0]);
        sim.tech[1].power = Some(0xc);
        assert_eq!(sim.tech_price(1, t.taxation), [88, 88, 0, 0, 0, 0]);
    }

    #[test]
    fn the_slot_is_the_category() {
        assert_eq!(slot_for(0), 10);
        assert_eq!(slot_for(1), 9);
        assert_eq!(slot_for(2), 4);
        assert_eq!(slot_for(3), 8);
    }

    #[test]
    fn the_epoch_quota_loosens_with_difficulty() {
        assert!(epoch_gate(0, 3, 4, 2));
        assert!(!epoch_gate(0, 3, 5, 2));
        assert!(!epoch_gate(1, 3, 4, 3));
        assert!(epoch_gate(2, 3, 8, 3));
        assert!(!epoch_gate(2, 3, 9, 3));
        assert!(!epoch_gate(2, 3, 8, 4));
        for d in [3, 4, 5] {
            assert!(epoch_gate(d, 0, 99, 99));
        }
    }

    #[test]
    fn the_age_pacing_holds_the_ai_behind_the_humans() {
        assert_eq!(age_pace(0, 2, 3, 1), AgePace::Delay((1 + 16) * 225));
        assert_eq!(age_pace(1, 2, 3, 1), AgePace::Delay((1 + 8) * 225));
        assert_eq!(age_pace(0, 3, 3, 1), AgePace::Skip);
        assert_eq!(age_pace(2, 3, 3, 1), AgePace::Delay((1 + 20) * 225));
        assert_eq!(age_pace(2, 4, 3, 1), AgePace::Skip);
        assert_eq!(age_pace(3, 9, 0, 1), AgePace::Unpaced);
    }

    // ---- research_techs ----

    #[test]
    fn a_leader_with_no_research_building_offers_nothing() {
        let (mut sim, t) = sim();
        city(&mut sim, &t, 1, 5, 5);
        sim.research_techs(1);
        assert_eq!(sim.ai[1].make_list.head().t, -1);
    }

    #[test]
    fn a_library_offers_the_library_lines_in_their_category_slots() {
        let (mut sim, t) = sim();
        city(&mut sim, &t, 1, 5, 5);
        building(&mut sim, 1, t.library, 12, 5);
        sim.research_techs(1);
        let list = &sim.ai[1].make_list;
        assert!(list.head().t >= 0, "something was offered");
        assert_eq!(list.list[8].t, t.science1 as i32, "Science is cat 3");
        assert_eq!(list.list[10].t, t.military1 as i32, "Military is cat 0");
    }

    #[test]
    fn the_capital_countdown_lobby_stops_a_leader_that_lost_its_capital() {
        let (mut sim, t) = sim();
        let c = city(&mut sim, &t, 1, 5, 5);
        building(&mut sim, 1, t.library, 12, 5);
        sim.lobby.elimination = 1;
        sim.cities[c].capital = false;
        sim.research_techs(1);
        assert_eq!(sim.ai[1].make_list.head().t, -1);
        sim.cities[c].capital = true;
        sim.research_techs(1);
        assert!(sim.ai[1].make_list.head().t >= 0);
    }

    #[test]
    fn a_tower_tech_is_refused_below_difficulty_two_and_takes_slot_six() {
        let (mut sim, t) = sim();
        city(&mut sim, &t, 1, 5, 5);
        building(&mut sim, 1, t.tower, 12, 5);
        small(&mut sim, 1);
        sim.lobby.difficulty = 1;
        assert_eq!(value(&mut sim, 1, t.fortification), None);
        sim.lobby.difficulty = 2;
        let (hard, slot) = value(&mut sim, 1, t.fortification).expect("offered at 2");
        assert_eq!(slot, 6, "the Tower overrides the category's slot");
        // The ×100 is the Tower's: the same tech at a Temple takes ×4.
        let temple_id = sim.build_types[t.temple].tree.expect("a tree entry");
        sim.tech_tree.types[t.fortification].where_ = Some(temple_id);
        building(&mut sim, 1, t.temple, 16, 5);
        let (soft, slot) = value(&mut sim, 1, t.fortification).expect("offered");
        assert_eq!(slot, 8);
        assert_eq!(hard / 100, soft / 4);
    }

    #[test]
    fn the_taxation_line_takes_a_further_hundred_at_the_temple() {
        let (mut sim, t) = sim();
        city(&mut sim, &t, 1, 5, 5);
        building(&mut sim, 1, t.temple, 12, 5);
        small(&mut sim, 1);
        let (tax, _) = value(&mut sim, 1, t.taxation).expect("Taxation is valued");
        let (rel, _) = value(&mut sim, 1, t.religion).expect("Religion is valued");
        assert!(rel > 0);
        assert_eq!(tax, mul(rel, 100));
    }

    #[test]
    fn the_university_multiplies_by_a_thousand() {
        let (mut sim, t) = sim();
        city(&mut sim, &t, 1, 5, 5);
        building(&mut sim, 1, t.university, 12, 5);
        small(&mut sim, 1);
        let (uni, _) = value(&mut sim, 1, t.agriculture).expect("valued");
        let temple_id = sim.build_types[t.temple].tree.expect("a tree entry");
        sim.tech_tree.types[t.agriculture].where_ = Some(temple_id);
        building(&mut sim, 1, t.temple, 16, 5);
        let (tem, _) = value(&mut sim, 1, t.agriculture).expect("valued");
        assert_eq!(uni / 1000, tem / 4);
    }

    #[test]
    fn the_government_coin_is_one_draw_and_the_dropped_column_is_not_offered() {
        let (mut sim, t) = sim();
        city(&mut sim, &t, 1, 5, 5);
        building(&mut sim, 1, t.senate, 12, 5);
        // `pers.raid < 0` short-circuits the coin: no draw for it, and
        // column 0 is thrown away.
        sim.ai[1].pers.raid = -1;
        let before = sim.rng;
        sim.research_techs(1);
        // Only the first tier's pair is reachable in this tree: one of it
        // is dropped without a draw, the other takes exactly one.
        let mut check = before;
        check.roll();
        assert_eq!(sim.rng, check, "one draw for the survivor, none for a coin");
        let list = offered(&sim, 1);
        assert!(!list.contains(&(t.gov_a as i32)), "column 0 is dropped");
        assert!(list.contains(&(t.gov_b as i32)));
    }

    #[test]
    fn a_raid_zero_leader_spends_one_extra_draw_on_the_coin() {
        let (mut sim, t) = sim();
        city(&mut sim, &t, 1, 5, 5);
        building(&mut sim, 1, t.senate, 12, 5);
        sim.ai[1].pers.raid = 0;
        let before = sim.rng;
        sim.research_techs(1);
        // The coin is inside the per-tech loop: each of the two reachable
        // governments tosses its own, and the survivor takes a second
        // draw. This seed keeps one of the pair, so three draws.
        let mut check = before;
        for _ in 0..3 {
            check.roll();
        }
        assert_eq!(sim.rng, check, "a coin each, then the survivor's draw");
        let list = offered(&sim, 1);
        let a = list.contains(&(t.gov_a as i32));
        let b = list.contains(&(t.gov_b as i32));
        assert!(a ^ b, "this seed's two coins agreed");
    }

    #[test]
    fn a_raiding_leader_drops_the_second_column_without_a_coin() {
        let (mut sim, t) = sim();
        city(&mut sim, &t, 1, 5, 5);
        building(&mut sim, 1, t.senate, 12, 5);
        sim.ai[1].pers.raid = 1;
        let before = sim.rng;
        sim.research_techs(1);
        let mut check = before;
        check.roll();
        assert_eq!(sim.rng, check);
        let list = offered(&sim, 1);
        assert!(list.contains(&(t.gov_a as i32)));
        assert!(!list.contains(&(t.gov_b as i32)));
    }

    #[test]
    fn an_epoch_line_more_than_one_ahead_of_the_lowest_is_dropped() {
        let (mut sim, t) = sim();
        city(&mut sim, &t, 1, 5, 5);
        building(&mut sim, 1, t.library, 12, 5);
        sim.tech[1].epoch[Line::Commerce.index()] = 2;
        assert_eq!(value(&mut sim, 1, t.commerce2), None);
        sim.tech[1].epoch[Line::Commerce.index()] = 1;
        assert!(value(&mut sim, 1, t.commerce2).is_some());
    }

    #[test]
    fn the_line_at_the_lowest_epoch_is_worth_ten_times_as_much() {
        let (mut sim, t) = sim();
        city(&mut sim, &t, 1, 5, 5);
        // Read the ×10 away from the Library, whose own branch divides the
        // score by the line's level and would confound it.
        let temple_id = sim.build_types[t.temple].tree.expect("a tree entry");
        sim.tech_tree.types[t.science1].where_ = Some(temple_id);
        building(&mut sim, 1, t.temple, 12, 5);
        small(&mut sim, 1);
        sim.lobby.difficulty = 3;
        let (level, _) = value(&mut sim, 1, t.science1).expect("valued");
        sim.tech[1].epoch[Line::Science.index()] = 1;
        let (ahead, _) = value(&mut sim, 1, t.science1).expect("valued");
        assert_eq!(level, mul(ahead, 10));
    }

    #[test]
    fn the_age_gate_holds_the_ai_a_step_behind_the_human() {
        let (mut sim, t) = sim();
        city(&mut sim, &t, 1, 5, 5);
        building(&mut sim, 1, t.library, 12, 5);
        sim.lobby.difficulty = 0;
        assert_eq!(
            value(&mut sim, 1, t.classical),
            None,
            "the human is still in the Ancient age"
        );
        sim.tech[0].ages = 1;
        assert!(
            value(&mut sim, 1, t.classical).is_some(),
            "an unstamped age passes once the human is there"
        );
    }

    #[test]
    fn a_stamped_age_waits_out_the_human_delay() {
        let (mut sim, t) = sim();
        city(&mut sim, &t, 1, 5, 5);
        building(&mut sim, 1, t.library, 12, 5);
        sim.lobby.difficulty = 0;
        sim.tech[0].ages = 1;
        sim.tech[0].age_stamp[0] = Some(0);
        // `(who + 16) × 225` = 3825 frames for leader 1.
        sim.frame = 3824;
        assert_eq!(value(&mut sim, 1, t.classical), None);
        sim.frame = 3825;
        assert!(value(&mut sim, 1, t.classical).is_some());
    }

    /// Difficulty 2 waits on the **previous** age's stamp — the original
    /// passes `t − 1` where 0 and 1 pass `t` (`0x6c6e94`, called at
    /// `0x6c6eac`). The first age has no predecessor, so it is never
    /// delayed however early the human reached it, even though the stamp
    /// difficulty 0 would read is set and recent.
    /// (`docs/audit/2026-08-25-ai.md`, B3-d.)
    #[test]
    fn difficulty_two_waits_on_the_previous_ages_stamp() {
        let (mut sim, t) = sim();
        city(&mut sim, &t, 1, 5, 5);
        building(&mut sim, 1, t.library, 12, 5);
        sim.tech[0].ages = 1;
        sim.tech[0].age_stamp[0] = Some(0);
        sim.frame = 0;

        // Difficulty 0 reads `age_stamp[0]` and holds for `(1 + 16) × 225`.
        sim.lobby.difficulty = 0;
        assert_eq!(value(&mut sim, 1, t.classical), None);

        // Difficulty 2 reads the age before it — there is none, so the
        // original's call returns −1 and the gate opens at once.
        sim.lobby.difficulty = 2;
        assert!(
            value(&mut sim, 1, t.classical).is_some(),
            "the first age has no previous stamp to wait on"
        );
    }

    #[test]
    fn a_difficulty_three_leader_is_unpaced() {
        let (mut sim, t) = sim();
        city(&mut sim, &t, 1, 5, 5);
        building(&mut sim, 1, t.library, 12, 5);
        sim.lobby.difficulty = 3;
        sim.tech[0].ages = 0;
        assert!(value(&mut sim, 1, t.classical).is_some());
    }

    #[test]
    fn the_knowledge_stockpile_gates_an_epoch() {
        let (mut sim, t) = sim();
        city(&mut sim, &t, 1, 5, 5);
        building(&mut sim, 1, t.library, 12, 5);
        sim.lobby.difficulty = 3;
        sim.tech_tree.types[t.science1].cost[KNOWLEDGE] = 1_000_000;
        sim.ledgers[1].bucket[KNOWLEDGE] = 10;
        assert_eq!(value(&mut sim, 1, t.science1), None);
        sim.ledgers[1].bucket[KNOWLEDGE] = 10_000_000;
        assert!(value(&mut sim, 1, t.science1).is_some());
    }

    #[test]
    fn no_shortage_doubles_the_value() {
        let (mut sim, t) = sim();
        city(&mut sim, &t, 1, 5, 5);
        building(&mut sim, 1, t.library, 12, 5);
        sim.ai[1].shortages = 0;
        let (easy, _) = value(&mut sim, 1, t.science1).expect("valued");
        sim.ai[1].shortages = 1;
        let (tight, _) = value(&mut sim, 1, t.science1).expect("valued");
        assert_eq!(easy, mul(tight, 2));
    }

    // ---- produce_tech ----

    #[test]
    fn produce_tech_queues_at_the_research_building() {
        let (mut sim, t) = sim();
        city(&mut sim, &t, 1, 5, 5);
        let lib = building(&mut sim, 1, t.library, 12, 5);
        assert!(sim.produce_tech(1, t.science1, 1));
        assert_eq!(sim.buildings[lib].queue.items.len(), 1);
        // Already being researched: the original returns "queued" and
        // touches nothing.
        assert!(sim.produce_tech(1, t.science1, 1));
        assert_eq!(sim.buildings[lib].queue.items.len(), 1);
    }

    #[test]
    fn produce_tech_prefers_the_undamaged_and_the_shorter_queue() {
        let (mut sim, t) = sim();
        city(&mut sim, &t, 1, 5, 5);
        let a = building(&mut sim, 1, t.barracks, 12, 5);
        let b = building(&mut sim, 1, t.barracks, 12, 12);
        sim.ai[1].mil_trainers = vec![a, b];
        let bt_id = sim.build_types[t.barracks].tree.expect("the tree entry");
        sim.tech_tree.types[t.military1].where_ = Some(bt_id);
        sim.tech_tree.types[t.science1].where_ = Some(bt_id);
        // `Build+0x24` is `ObjectData::damage`: a damaged trainer scores
        // lower, so the second one wins even though it is second in the
        // list.
        sim.buildings[a].damage = 9;
        assert!(sim.produce_tech(1, t.military1, 1));
        assert_eq!(sim.buildings[b].queue.items.len(), 1);
        assert_eq!(sim.buildings[a].queue.items.len(), 0);
        // With the damage gone the queue length decides, and now `a` is
        // the empty one.
        sim.buildings[a].damage = 0;
        assert!(sim.produce_tech(1, t.science1, 1));
        assert_eq!(
            sim.buildings[a].queue.items.len(),
            1,
            "the empty queue wins"
        );
        assert_eq!(sim.buildings[b].queue.items.len(), 1);
    }

    #[test]
    fn produce_tech_refuses_when_no_building_stands() {
        let (mut sim, t) = sim();
        city(&mut sim, &t, 1, 5, 5);
        assert!(!sim.produce_tech(1, t.science1, 1));
    }

    #[test]
    fn a_military_trainer_is_searched_in_the_leaders_own_list() {
        let (mut sim, t) = sim();
        city(&mut sim, &t, 1, 5, 5);
        let bar = building(&mut sim, 1, t.barracks, 12, 5);
        let bt_id = sim.build_types[t.barracks].tree.expect("the tree entry");
        sim.tech_tree.types[t.military1].where_ = Some(bt_id);
        // Activating it filed it — `Wall::increment_stats`, `docs/AI.md` §29.
        assert_eq!(
            sim.ai[1].mil_trainers,
            vec![bar],
            "an active trainer joins the list when it activates"
        );
        sim.ai[1].mil_trainers.clear();
        assert!(
            !sim.produce_tech(1, t.military1, 1),
            "a trainer not in `mil_trainers` is invisible"
        );
        sim.ai[1].mil_trainers.push(bar);
        assert!(sim.produce_tech(1, t.military1, 1));
        assert_eq!(sim.buildings[bar].queue.items.len(), 1);
    }

    #[test]
    fn produce_tech_queues_a_unit_s_research_on_the_unit_queue() {
        // `produce_tech:262` ends in `Build::queue_up(t, escrow)`, the one
        // queue for every kind, so a unit type the leader does not own is
        // researched as the unit queue's research job (`docs/AI.md` §56).
        // `queue_tech` takes only a technology: this refused until item 545.
        let (mut sim, t) = sim();
        city(&mut sim, &t, 1, 5, 5);
        let bar = building(&mut sim, 1, t.barracks, 12, 5);
        let rec = sim.add_unit_type(crate::UnitType {
            tree: Some(t.phalanx),
            ..crate::UnitType::default()
        });
        assert_eq!(sim.type_avail(1, t.phalanx), tech::NOT_AVAILABLE);
        sim.gain_tech(1, t.classical);
        assert_eq!(sim.type_avail(1, t.phalanx), tech::RESEARCHABLE);
        assert!(sim.produce_tech(1, t.phalanx, 1));
        assert_eq!(sim.buildings[bar].queue.items.len(), 1);
        assert_eq!(sim.muster[1].queued_by_type[rec], 1);
        // Being researched: "queued", and nothing more.
        assert!(sim.produce_tech(1, t.phalanx, 1));
        assert_eq!(sim.buildings[bar].queue.items.len(), 1);
    }
}
