//! The census — `Leader::plan_strategy@006b9620`'s sweep after the cadence
//! check, `docs/AI.md` §2.3 steps 1–16: the recount of everything a
//! computer leader owns into [`crate::ai::Census`] and the per-city
//! [`crate::ai::CityAi`] records. The tail (step 17) — the orphan check,
//! `compute_sites`, arming the step machine — is the driver's.
//!
//! Oracle: a `LEADERS=9` dump prints every field under the same name
//! (`tools/gamelog/leader.py FRAME WHO`); run8's frame 1 is the AI leader
//! after its frame-0 sweep.
//!
//! # Region numbering
//!
//! The original packs land regions into `0..0x3f` and sea regions into
//! `0x3f..0x7e`, stores the sea ones `% 0x3f` in its 63-entry arrays, and
//! tests "is this a land region" as `r < 0x40`. Ours are one flat list, one
//! slot per region either way, so every `r < 0x40` becomes
//! `world.terrain(r) == Land` and every `% 0x3f` disappears. That is the
//! only deliberate departure from the original's indexing in this module.
//!
//! # Seams
//!
//! The simulation does not carry every input the sweep reads. Each missing
//! one is a private fn below whose name is the original's and whose answer
//! is what an empty map would give; they are listed in one place at
//! [`seams`]. Nothing else here is approximate.

use crate::ai::{Census, CityAi};
use crate::attrition::Domain;
use crate::build;
use crate::city::radius_for_level;
use crate::combat::Obj;
use crate::economy::RESOURCES;
use crate::orders::{Body, Worker, index};
use crate::world::{Cell, Owner, Pos, Terrain, tile, vector_dist};
use crate::{Player, Sim};

/// The rate step 1 writes into every good's `escrow_rate` slot.
const ESCROW_RATE: i32 = 40;

/// `CityData::peasant_dist`'s "no citizen anywhere near" value.
const PEASANT_DIST_NONE: i32 = 100;

/// Position units per cell — the divisor `peasant_dist` is measured in.
const CELL: i32 = 0x300;

pub(crate) use crate::ids::ty;

/// `UnitTypeData::role & 0x10000` — a military unit.
const ROLE_MILITARY: u32 = 0x1_0000;
/// `role & 0x10` — the bit that, with `0x10000`, keeps a unit out of the
/// invader count (`role & 0x10010 == 0x10000`).
const ROLE_NO_INVADE: u32 = 0x10;
/// `ObjectTypeData::obj_masks & 0x8000000` — a missile rather than a plane.
const OBJ_MASK_MISSILE: u32 = 0x800_0000;

/// The seams, in one list, so a reader does not have to find them.
///
/// | Seam | Answers | Why |
/// | --- | --- | --- |
/// | `village_num` | 0 | `LeaderData::village_num` has no counterpart here. |
/// | `GoodData::ever_seen` | unwritten | step 9 sets the leader's bit on every rare it counts; only `GoodData::is_seen` reads it, and nothing here asks a good whether it is seen. |
/// | step 6's `ALLY_LOS` arm | never held | the meeting runs under `reveal_map` alone ([`Sim::census_meet`], item 3763); the `has_preq(ALLY_LOS)` half of its gate is not read. Outside the gate the live paths to the met bit are the fog — `check_ever_seen` → `Leader::meet` (`docs/VISION.md` §6.2) — and a strike, `Unit::set_attacking` ([`Sim::set_attacking`]). |
/// | `ScenarioData::ally_mask` | 0 | scenarios are cut from v1. |
/// | `unit_masks & 1` | clear | the flag that excludes an object from the census is unmodelled. |
/// | `unit_masks & 0x80000` | merchants only | a packed fisherman is idle (`idle_fishermen`, item 1666); every merchant still counts in `reg_unpack_merch`. |
/// | order kinds 8 / 0xe | never | build-and-repair-through-a-transport is unmodelled. |
///
/// Two seams closed 2026-08-25 with `docs/TRANSPORT.md`: `Region::num_coasts`
/// is computed from the cells (`World::rebuild_coasts`) and
/// `BuildTypeData::is_dock_tile` is `Sim::is_dock_tile`; `carry` and
/// `unit_flags2 & 4` come from the type's columns.
/// | region flags `& 8` | clear | step 15's two-landmass expand probe never fires. |
/// | `Armies::init_navy` | seeded by `create_units`' sea branch | step 16 is `Sim::census_seed_army` (`docs/ARMY.md`); the sea branch seeds the navy since item 579 (`docs/ORDERS.md` §25.3). |
/// | `check_explore`'s visibility | every region cell counts | no fog: the leader has seen the map. |
///
/// `home_reg`, `pop`, `control` and `scouts` are *not* seams: the sweep
/// reads them and never writes them, so they stay whatever the rest of the
/// simulation last put there.
pub mod seams {}

impl Sim {
    // ------------------------------------------------------------------
    // Seams
    // ------------------------------------------------------------------

    /// `treaties[i] & 1`, "I have met leader `i`" — the **whole** gate steps
    /// 14 and 15 carry, and until item 385 the one thing this crate had no
    /// way to derive.
    ///
    /// There is **no `human` test anywhere in either loop**:
    /// `plan_strategy@006b9620:1511,1557` gates on `leader_flags & 2`,
    /// `i != who` and this bit, and the human's `leader_flags` is 7. The
    /// original sets the bit on *first contact* and never clears it in a
    /// game, and the live path to it is the fog:
    /// [`Sim::check_ever_seen`](crate::Sim::check_ever_seen) →
    /// [`Sim::meet`] → [`Sim::treaty_on`]. `docs/VISION.md` §6.2.
    ///
    /// What this replaced, and why the replacement is the right way round:
    /// the human was skipped here as a stand-in for the missing bit, which
    /// on a one-human-one-AI capture is "never met" — 2,000 frames of Great
    /// Lakes' word closer than answering "yes" from frame 1, which is what
    /// removing the skip did (9182 → 7182, `docs/AI.md` §45). The dumps say
    /// what the right answer is: player 1's `treaties[0]` is **0** on blocks
    /// 6950 and 7514 and **1** on 8174 and 9170, with `diplos[0]` at 0 — at
    /// war — on all four, so the diplomacy never moves and only contact
    /// does, once, in (7616, 8174].
    fn met(&self, who: Player, other: usize) -> bool {
        other != who as usize && !self.defeated[other] && self.has_met(who, other)
    }

    /// `treaties[other] & 1` — the met bit, and the field the leader-record
    /// comparison prints. `defeated` is not part of the original's bit; it
    /// is [`Sim::met`]'s own `leader_flags & 2` folded in here so both
    /// readers agree.
    pub fn has_met(&self, who: Player, other: usize) -> bool {
        other == who as usize
            || (!self.defeated[other]
                && self
                    .treaties
                    .get(who as usize)
                    .and_then(|r| r.get(other))
                    .is_some_and(|t| t & 1 != 0))
    }

    // ------------------------------------------------------------------
    // Small readers the sweep leans on
    // ------------------------------------------------------------------

    /// Whether a region index names land — the original's `r < 0x40`.
    fn region_is_land(&self, r: u16) -> bool {
        (r as usize) < self.world.region_count() && self.world.terrain(r) == Terrain::Land
    }

    /// `diplos[a][b] == 0` — at war. The original tests either side.
    fn diplo_war(&self, a: usize, b: usize) -> bool {
        self.at_war[a][b]
    }

    /// `diplos[a][b] == 2` — allied. The original tests both sides.
    fn diplo_ally(&self, a: usize, b: usize) -> bool {
        self.allied[a][b]
    }

    /// `LeaderData::territory` (`+0x9d8`, the field `get_team_terr@006d62e0`
    /// sums): the **land** cells this leader owns, as the border pass last
    /// left them in [`crate::Holdings::territory`] (item 3516). A count of
    /// the live map took the sea too — run1162d's who=0 census read who=1's
    /// team as 462 on block 2601 where the original's field held 437.
    fn territory_of(&self, who: Player) -> i32 {
        self.holdings[who as usize].territory
    }

    /// `LeaderData::get_team_terr@006d62e0`: my territory plus every ally's.
    fn team_terr(&self, who: Player) -> i32 {
        let w = who as usize;
        (0..self.players.len())
            .filter(|&i| i == w || (!self.defeated[i] && self.diplo_ally(w, i)))
            .map(|i| self.territory_of(i as Player))
            .sum()
    }

    /// `reg_buildings[r][0] + [1] + [2] + [0x75]` — the four city types, as
    /// the building lifecycle keeps them: active city buildings of `who`
    /// whose cell is in `r`.
    /// [`Sim::reg_city_buildings`] for the one caller outside the sweep —
    /// `was_seen`'s territory shortcut, which asks about a leader whose
    /// census may never have run.
    pub(crate) fn reg_city_buildings_pub(&self, who: Player, r: u16) -> i32 {
        self.reg_city_buildings(who, r)
    }

    fn reg_city_buildings(&self, who: Player, r: u16) -> i32 {
        self.buildings
            .iter()
            .filter(|b| {
                b.alive
                    && b.active
                    && b.owner == who
                    && b.ty
                        .is_some_and(|t| build::city_level(&self.build_types, t) > 0)
                    && self.world.region_of(b.pos.cell()) == Some(r)
            })
            .count() as i32
    }

    /// `UnitTypeData::domain` on a unit's type.
    fn unit_domain(&self, u: usize) -> Domain {
        self.units[u]
            .ty
            .map_or(Domain::Land, |t| self.unit_types[t].combat.domain)
    }

    /// The unit's `TypeIndex`, when the tree knows it.
    pub(crate) fn unit_tree(&self, u: usize) -> Option<crate::tech::TypeId> {
        self.units[u].ty.and_then(|t| self.unit_types[t].tree)
    }

    /// `ObjectData::is(t, 0)` on a unit — the lineage test.
    pub(crate) fn unit_line_is(&self, u: usize, t: crate::tech::TypeId) -> bool {
        self.unit_tree(u)
            .is_some_and(|ut| self.tech_tree.is(ut, t, false))
    }

    /// `ObjectsData::find_city@0065ba90(x, y, SEARCH_FRIENDLY, who, 0x200)`
    /// — the nearest live city of `who` **in the query point's own region**
    /// (`0x200` is that flag, not a radius: the search has no distance
    /// limit, `find_dist` starts at 99,999,999). Returns the city and
    /// `objects.find_dist`, in position units. Ties go to the later city,
    /// because the original compares `<=`.
    fn census_find_city(&self, who: Player, at: Pos) -> Option<(usize, i32)> {
        let reg = self.world.region_of(at.cell());
        let mut best: Option<(usize, i32)> = None;
        for (i, c) in self.cities.iter().enumerate() {
            if !c.alive || c.owner != who || c.reg != reg {
                continue;
            }
            let d = vector_dist(c.pos.x - at.x, c.pos.y - at.y);
            if best.is_none_or(|(_, bd)| d <= bd) {
                best = Some((i, d));
            }
        }
        best
    }

    /// The region a unit counts in (`docs/AI.md` §2.3 step 10): its own
    /// tile's, with the coastal `region2` refinement; when garrisoned, the
    /// container's — except that a **sea-domain** unit inside a water-domain
    /// building takes the first ocean tile of the container's footprint, so
    /// a ship in a dock counts in the sea region. The result must be a sea
    /// region (`> 0x3e`) or the scan is discarded.
    ///
    /// **A container is a unit too** (item 592): `ObjectData::get_inside`
    /// walks `inside_up` to the outermost container, building or boat, and
    /// a land unit takes that container's tile, `region2` rule and all. A
    /// citizen riding a transport barge counts in the barge's region, not
    /// where it boarded (`docs/AI.md` §58). A sea-domain unit inside a boat
    /// asks the boat a vslot the export folds (`+0x108`); no such rider
    /// exists here, and it takes the boat's tile too.
    fn census_unit_region(&self, u: usize) -> Option<u16> {
        let unit = &self.units[u];
        if let Some(boat) = unit.inside_unit {
            return self.world.tregion_alt(self.units[boat].pos.tile());
        }
        let Some(b) = unit.inside else {
            return self.world.tregion_alt(unit.pos.tile());
        };
        let container = &self.buildings[b];
        let plain = self.world.tregion_alt(container.pos.tile());
        if self.unit_domain(u) != Domain::Sea {
            return plain;
        }
        let Some(bt) = container.ty.map(|t| &self.build_types[t]) else {
            return plain;
        };
        if bt.domain() == build::BuildDomain::Land {
            return plain;
        }
        let centre = container.pos.tile();
        let corner = Pos::new(centre.x - bt.x_size / 2, centre.y - bt.y_size / 2);
        for tx in corner.x..corner.x + bt.x_size {
            for ty in corner.y..corner.y + bt.y_size {
                let t = Pos::new(tx, ty);
                if self.world.tile_mask(t) & tile::SURFACE != tile::SURFACE_OCEAN {
                    continue;
                }
                if let Some(r) = self.world.tregion_alt(t)
                    && !self.region_is_land(r)
                {
                    return Some(r);
                }
                break;
            }
        }
        plain
    }

    /// `City::count_gather_slots@00737dc0(city, 0, 0)`'s return value: the
    /// gather slots of the city's own members, **knowledge excluded from
    /// the total** — the original skips good 3 when accumulating its return
    /// while still writing it into the per-good array it is not given here.
    ///
    /// `ai_units.rs` has the three-output form of the same function under
    /// the original's name; it is private to that module, so this is the
    /// total on its own until one of them is made `pub(crate)`.
    fn city_gather_slot_total(&self, c: usize) -> i32 {
        self.count_gather_slots(c).0
    }

    // ------------------------------------------------------------------
    // The sweep
    // ------------------------------------------------------------------

    /// The sweep, steps 1–16. Writes `self.ai[who].census` and
    /// `self.ai[who].city_ai`, and `invaders[who]` on every other leader.
    pub fn census(&mut self, who: Player) {
        let w = who as usize;
        let regions = self.world.region_count();
        let players = self.players.len();
        self.ai[w].census.resize(regions, players);
        let n = self.cities.len();
        self.ai[w].city_ai.resize(n, CityAi::default());

        self.census_escrow_rate(who);
        self.census_reset_cities(who);
        census_zero_scalars(&mut self.ai[w].census);
        self.census_territory(who);
        self.census_ally_mask(who);
        self.census_meet(who);
        self.census_clear_invaders(who);
        self.census_zero_regions(who);
        self.census_rares(who);
        self.census_units(who);
        self.census_buildings(who);
        self.census_maxima(who);
        self.census_city_sites(who);
        self.census_wars(who);
        self.census_strategy(who);
        // Step 16, the army seeding (`docs/ARMY.md` §15): not for a human
        // leader without computer assist, `(leader_flags & 0xc) != 4`
        // (`plan_strategy@006b9620:1642`).
        if !self.nation[w].human {
            self.census_seed_army(who);
        }
    }

    /// Step 1. Once the leader holds more than two cities and villages,
    /// every good escrows at 40 — and never goes back.
    fn census_escrow_rate(&mut self, who: Player) {
        if self.city_num(who) + self.village_num(who) > 2 {
            self.ai[who as usize].census.escrow_rate = [ESCROW_RATE; RESOURCES];
        }
    }

    /// Step 2, and the `in_port` half that the original defers until after
    /// the rares.
    fn census_reset_cities(&mut self, who: Player) {
        let w = who as usize;
        for c in self.cities_of(who) {
            let r = &mut self.ai[w].city_ai[c];
            r.free = 0;
            r.busy = 0;
            r.gatherers = 0;
            r.peasant_dist = PEASANT_DIST_NONE;
            r.in_port = 0;
        }
    }

    /// Step 4. My team's territory, and the widest and narrowest of every
    /// other computer leader's team that is not allied with me both ways.
    fn census_territory(&mut self, who: Player) {
        let w = who as usize;
        let mine = self.team_terr(who);
        let mut other = 0;
        let mut min_other = 0;
        for i in 0..self.players.len() {
            // **Every other leader the roster holds, the human included**
            // (item 368). `plan_strategy@006b9620:159-166`'s gate is
            // `leader_flags & 2`, `i != who` and `i >= 0` — and bit 1 is
            // set for *every* leader every capture prints, the human's
            // `leader_flags 7` among them. Skipping humans here read
            // `other_team_terr` as nought on a one-human-one-AI game,
            // which is every capture this crate is diffed against.
            if i == w {
                continue;
            }
            if self.diplo_ally(i, w) && self.diplo_ally(w, i) {
                continue;
            }
            let t = self.team_terr(i as Player);
            other = other.max(t);
            min_other = if min_other == 0 { t } else { min_other.min(t) };
        }
        let cs = &mut self.ai[w].census;
        cs.my_team_terr = mine;
        cs.other_team_terr = other;
        cs.min_other_team_terr = min_other;
    }

    /// Step 5, and step 6's one surviving effect: the mask is my own bit
    /// plus every leader allied with me both ways. The meeting pass itself
    /// is a seam.
    fn census_ally_mask(&mut self, who: Player) {
        let w = who as usize;
        let mut mask = 1u32 << (w & 0x1f);
        for i in 0..self.players.len() {
            if i == w || self.defeated[i] || self.nation[i].human {
                continue;
            }
            if self.diplo_ally(w, i) && self.diplo_ally(i, w) {
                mask |= 1 << (i & 0x1f);
            }
        }
        let cs = &mut self.ai[w].census;
        cs.filled_gather_slots = [0; RESOURCES];
        cs.ally_mask = mask;
    }

    /// **Step 6, the meeting** (`plan_strategy@006b9620:205`–`265`, item
    /// 3763, `docs/GOLDEN.md` §149.4): under `has_preq(ALLY_LOS) ||
    /// reveal_map != 0`, every other active leader (`leader_flags & 2`) not
    /// yet met (`treaties[i] & 1 == 0`) has its **buildings** asked
    /// `is_seen(who, 0)` (`BuildData::is_seen`, vslot `+0x48` of the data
    /// `+0xb0` hands back) — the two object lists from `obj_base[1]` (2000)
    /// and `obj_base[2]`, never the units' `obj_base[0]` — and the first that answers
    /// sets the met bit on both sides — inline, not through
    /// `Leader::meet`. The war loop of step 15 reads the bit.
    ///
    /// **No walk holds this step** (item 3763's mutation): a building is
    /// seen here only through `ever_seen`, and a foreign bit arriving there
    /// meets in `check_ever_seen` first. The melee's meeting is a strike's
    /// ([`Sim::set_attacking`]).
    ///
    /// SEAM: the `ALLY_LOS` preq (`0x2b0`) is not read; the arm runs under
    /// `reveal_map` alone.
    fn census_meet(&mut self, who: Player) {
        if self.lobby.reveal_map == 0 {
            return;
        }
        let w = who as usize;
        for i in 0..self.players.len().min(8) {
            if i == w || self.defeated[i] || self.has_met(who, i) {
                continue;
            }
            let o = i as Player;
            let seen = (0..self.buildings.len()).any(|b| {
                self.buildings[b].owner == o
                    && self.buildings[b].alive
                    && self.target_seen_by(who, Obj::Building(b))
            });
            if seen {
                self.meet(who, o);
            }
        }
    }

    /// Step 7. `invaders[who]` is cleared on **every** active leader,
    /// mine included; step 10 refills it.
    fn census_clear_invaders(&mut self, who: Player) {
        let w = who as usize;
        let players = self.players.len();
        for i in 0..players {
            if self.defeated[i] {
                continue;
            }
            let inv = &mut self.ai[i].census.invaders;
            inv.resize(players, 0);
            inv[w] = 0;
        }
    }

    /// Step 8. Every per-region array back to zero, and `reg_cities`
    /// recounted from the four city types. `reg_pop`, `reg_forts`,
    /// `reg_docks`, `reg_terr` and `reg_buildings` are *not* zeroed — they
    /// belong to the building lifecycle.
    fn census_zero_regions(&mut self, who: Player) {
        let w = who as usize;
        let cities: Vec<i32> = (0..self.world.region_count())
            .map(|r| self.reg_city_buildings(who, r as u16))
            .collect();
        let cs = &mut self.ai[w].census;
        for v in [
            &mut cs.reg_active,
            &mut cs.reg_combat,
            &mut cs.reg_attack,
            &mut cs.reg_naval,
            &mut cs.reg_transports,
            &mut cs.reg_defense,
            &mut cs.reg_attacked,
            &mut cs.reg_land,
            &mut cs.reg_peasants,
            &mut cs.reg_free_peasants,
            &mut cs.reg_xport_peasants,
            &mut cs.reg_gatherers,
            &mut cs.reg_gather_slots,
            &mut cs.reg_known_rares,
            &mut cs.reg_unpack_merch,
        ] {
            v.iter_mut().for_each(|s| *s = 0);
        }
        cs.reg_cities.copy_from_slice(&cities);
        // `pop` and `reg_pop` belong to the city lifecycle and not to this
        // sweep — [`Sim::sync_leader_pop`] keeps them, and every writer the
        // original has is one of its call sites. It is repeated here for
        // one reason: the per-region arrays are `resize`d by the sweep, so
        // a `sync_pop_cities` that ran before the first one (`build_sim`
        // standing a dump up) wrote `reg_pop` into an empty vector.
        self.sync_leader_pop();
    }

    /// Step 9, the rares — `plan_strategy@006b9620:314–361`, the only
    /// writer `reg_known_rares` (`LeaderData +0x4d4`) has. Every good in
    /// the leader's `new_rares` list is counted into its cell's region
    /// when two things hold:
    ///
    /// * the leader or an ally has **explored** the half-cell the good
    ///   stands in — `seen2[div_3_table[y >> 7]][div_3_table[x >> 7]] &
    ///   ally_mask`, which is [`Sim::was_really_seen_fog`] exactly, exits
    ///   and all (`who >= 8`, `reveal_map == 3`, and the two leader flags
    ///   it keeps as seams);
    /// * the cell under it is **unowned, mine, or a mutual ally's** — the
    ///   `WData` owner byte is negative, or `who`'s, or `diplos` reads 2
    ///   both ways.
    ///
    /// The list only grows (`Leader::new_rare@006d9e70`), so a rare once
    /// seen is counted on every sweep until someone else's border covers
    /// it. `Leader::calc_gather@006ceee0` sums the array into
    /// `known_rares` under its own cadence ([`Sim::assemble_holdings`]),
    /// and `create_units`' merchant arm reads that sum — `docs/AI.md` §55.
    fn census_rares(&mut self, who: Player) {
        let w = who as usize;
        let list = self.ai[w].new_rares.clone();
        for gi in list {
            let Some(pos) = self.world.goods().get(gi).map(|g| g.pos) else {
                continue;
            };
            // `div_3_table[c >> 7]`: the half-cell, `c / 0x180`.
            let half = |c: i32| c.div_euclid(crate::world::UNITS_PER_CELL / 2);
            if !self.was_really_seen_fog(half(pos.x), half(pos.y), who) {
                continue;
            }
            let cell = pos.cell();
            if let Owner::Player(o) = self.world.owner(cell)
                && o != who
                && !(self.diplo_ally(w, o as usize) && self.diplo_ally(o as usize, w))
            {
                continue;
            }
            let Some(r) = self.world.region_of(cell) else {
                continue;
            };
            if let Some(slot) = self.ai[w].census.reg_known_rares.get_mut(usize::from(r)) {
                *slot += 1;
            }
        }
    }

    /// Step 10, the unit census.
    fn census_units(&mut self, who: Player) {
        let w = who as usize;
        for u in 0..self.units.len() {
            let unit = &self.units[u];
            if unit.owner != who || !unit.alive() {
                continue;
            }
            // **Captains only** — `Group::get_num_cap@007145c0`'s `is_captain`
            // through vslot `+0xe8`, the sweep's own iteration (`docs/AI.md`
            // §2.3 step 10). A follower of a squad is an object of mine and
            // is not one of my units for any of these counters: run91's
            // leader 1 stands nine Longbowmen in three squads and three
            // Hoplites in one, and the original's `active` is **32** — the
            // roster less those eight followers, to the unit. Item 295,
            // `docs/AI.md` §35.
            if !self.is_captain(u) {
                continue;
            }
            // A decoy is no unit of mine (`testb $0x1, 0x68(%esi)` at
            // `6b9f57`, beside the captain test): run346's six decoy squads
            // of 11637 stay out of `non_siege`, and `create_units`' military
            // gate reads the count without them (item 1302).
            if unit.decoy {
                continue;
            }
            // `type->control_cost != 0`: a unit that costs no population is
            // not counted at all.
            let control_cost = unit.ty.map_or(0, |t| self.unit_types[t].price.pop);
            if control_cost == 0 {
                continue;
            }
            let Some(reg) = self.census_unit_region(u) else {
                continue;
            };
            let land_reg = self.region_is_land(reg);
            let domain = self.unit_domain(u);
            let profile = unit.ty.map(|t| &self.unit_types[t].combat);
            // **`UnitTypeData::role`, not [`crate::combat::Profile::roles`]**
            // — item 295. The two words share no bit assignment: this
            // crate's own profile bitfield puts `CARAVAN` at `1 << 16` and
            // `V2ROCKET` at `1 << 4`, exactly where the original's role word
            // keeps `combat_role` and the scout bit, so the sweep read a
            // caravan as the leader's only soldier and every Hoplite and
            // Longbowman as a civilian. `docs/AI.md` §35.
            let roles = unit.ty.map_or(0, |rec| self.role_word_of_rec(rec));
            let obj_masks = profile.map_or(0, |p| p.obj_masks);
            let siege = profile.is_some_and(|p| p.siege);
            let military = roles & ROLE_MILITARY != 0;
            // `UnitTypeData::carry` and `unit_flags2 & 4` — a transport and
            // a fisherman.
            let carry = unit.ty.is_some_and(|t| self.unit_types[t].cols.carry != 0);
            let fisherman = unit
                .ty
                .is_some_and(|t| self.unit_types[t].cols.unit_flags2 & 4 != 0);
            // `attack() / 10`, or a flat 10 off the ground.
            let strength = if domain == Domain::Land {
                self.attack_of(Obj::Unit(u)) / 10
            } else {
                10
            };

            self.ai[w].census.active += 1;
            if military {
                let cs = &mut self.ai[w].census;
                cs.attack += strength;
                cs.combat += 1;
                if siege {
                    cs.siege += 1;
                } else {
                    cs.non_siege += 1;
                }
            }
            match domain {
                Domain::Sea => {
                    let cs = &mut self.ai[w].census;
                    cs.naval += strength;
                    if strength != 0 {
                        cs.sea_combat += 1;
                    }
                    if carry {
                        cs.transports += 1;
                    }
                    if fisherman {
                        cs.fishermen += 1;
                        // `unit_masks & 0x80000` (packed): a fisherman that
                        // has not unpacked to fish is idle (`docs/AI.md` §179).
                        if self.units[u].combat.packed {
                            cs.idle_fishermen += 1;
                        }
                    }
                }
                Domain::Air => {
                    let cs = &mut self.ai[w].census;
                    if obj_masks & OBJ_MASK_MISSILE != 0 {
                        cs.missile += 1;
                    } else {
                        cs.air += 1;
                    }
                    if obj_masks & OBJ_MASK_MISSILE != 0 {
                        if self.unit_line_is(u, ty::CRUISE) {
                            self.ai[w].census.cruise += 1;
                        } else if self.unit_line_is(u, ty::NUKE) {
                            self.ai[w].census.nuke += 1;
                        }
                    } else if self.unit_line_is(u, ty::BIPLANE) {
                        self.ai[w].census.fighters += 1;
                    } else if self.unit_line_is(u, ty::BOMBER) {
                        self.ai[w].census.bombers += 1;
                    }
                }
                Domain::Land => {}
            }

            // The invader count: a military unit without `role & 0x10`,
            // standing on a cell another active leader owns.
            if roles & (ROLE_MILITARY | ROLE_NO_INVADE) == ROLE_MILITARY
                && let Some(o) = self.world.owner(self.units[u].pos.cell()).player()
                && o as usize != w
                && !self.defeated[o as usize]
            {
                let players = self.players.len();
                let inv = &mut self.ai[o as usize].census.invaders;
                inv.resize(players, 0);
                inv[w] += 1;
            }

            let mut reg = reg;
            let mut land_reg = land_reg;
            match self.worker_of(u) {
                Worker::Scholar => self.census_scholar(who, u),
                Worker::Citizen => {
                    // **The region is one local, and the rider's arm writes
                    // it** (item 592): `plan_strategy`'s `xport` arm keeps
                    // the move point's region in the variable the unit's
                    // own region lives in, and the `reg_active` family
                    // below reads it after. run143's 10576 prints the rider
                    // `1/31` in `reg_active[5]`, the barge's destination,
                    // and in no region of the barge's own tile
                    // (`docs/AI.md` §58).
                    if let Some(r) = self.census_citizen(who, u, reg, land_reg) {
                        reg = r;
                        land_reg = self.region_is_land(r);
                    }
                }
                Worker::None => self.census_trader(who, u, reg),
            }

            // `reg_active`/`reg_combat`/`reg_attack` count only where the
            // unit's domain matches the region's kind, or the unit flies.
            let matches_region = match domain {
                Domain::Air => true,
                Domain::Land => land_reg,
                Domain::Sea => !land_reg,
            };
            if !matches_region {
                continue;
            }
            let r = reg as usize;
            let cs = &mut self.ai[w].census;
            cs.reg_active[r] += 1;
            if military {
                cs.reg_attack[r] += strength;
                cs.reg_combat[r] += 1;
            }
            if domain == Domain::Sea && !land_reg {
                cs.reg_naval[r] += strength;
                if carry {
                    cs.reg_transports[r] += 1;
                }
            }
        }
    }

    /// A scholar: counted, and its university slot filled when it is inside
    /// one.
    fn census_scholar(&mut self, who: Player, u: usize) {
        let w = who as usize;
        self.ai[w].census.scholars += 1;
        if let Some(b) = self.units[u].inside
            && self.building_ident(b) == build::Ident::University
        {
            self.ai[w].census.filled_gather_slots[3] += 1;
        }
    }

    /// A merchant or a caravan — the sweep's `else` arm, which every unit
    /// that is neither a scholar nor a citizen falls into.
    fn census_trader(&mut self, who: Player, u: usize, reg: u16) {
        let w = who as usize;
        let base = self.unit_tree(u);
        if base.is_some_and(|t| ty::MERCHANTS.contains(&t)) {
            self.ai[w].census.merchants += 1;
            // seam: `unit_masks & 0x80000` (packed) is never set.
            self.ai[w].census.reg_unpack_merch[reg as usize] += 1;
        } else if self.unit_line_is(u, ty::CARAVAN) {
            self.ai[w].census.caras += 1;
        }
    }

    /// A citizen: the nearest friendly city, the per-region peasant counts,
    /// and then the split by the *action*'s kind.
    ///
    /// Returns the region the `xport` arm left in the sweep's region local,
    /// when it ran (see [`Sim::census_citizen_inside`]).
    fn census_citizen(&mut self, who: Player, u: usize, reg: u16, land_reg: bool) -> Option<u16> {
        let w = who as usize;
        let found = self.census_find_city(who, self.units[u].pos);
        self.ai[w].census.peasants += 1;

        // `is_on_map`: the sign of `inside_up`, so a rider on a boat is
        // off the map exactly as a garrison is (item 592).
        if self.units[u].inside.is_some() || self.units[u].inside_unit.is_some() {
            return self.census_citizen_inside(who, u);
        }

        let r = reg as usize;
        let kind = self
            .action_of(u)
            .map_or(index::NONE, |i| self.units[u].orders[i].index());

        if land_reg {
            self.ai[w].census.reg_peasants[r] += 1;
            if self.ai[w].census.reg_cities[r] == 0 && kind != index::BUILD_AT {
                self.ai[w].census.reg_xport_peasants[r] += 1;
                self.ai[w].census.xport_peasants += 1;
            }
        }

        if kind == index::NONE || kind == index::EXPLORE_TO {
            self.ai[w].census.free_peasants += 1;
            if land_reg {
                self.ai[w].census.reg_free_peasants[r] += 1;
            }
            if let Some((c, dist)) = found {
                let rec = &mut self.ai[w].city_ai[c];
                rec.free = rec.free.wrapping_add(1);
                rec.peasant_dist = rec.peasant_dist.min(dist / CELL);
            }
            return None;
        }

        if let Some((c, _)) = found {
            self.ai[w].city_ai[c].busy += 1;
        }
        if kind != index::GATHER {
            // Kinds 8 and 0xe — build and repair from inside a transport —
            // are a seam; nothing else in the original's table counts here.
            return None;
        }

        let target = self
            .action_of(u)
            .and_then(|i| match self.units[u].orders[i].body {
                Body::Gather(g) => Some(g.building),
                _ => None,
            });
        // Knowledge is the scholar's, not a gatherer's: the original's
        // switch has no `0x1a4` arm here.
        if let Some(b) = target
            && let Some(good) = crate::ai_place::gather_good(self.building_ident(b))
            && good != 3
        {
            self.ai[w].census.filled_gather_slots[good] += 1;
        }
        if let Some((c, _)) = found {
            self.ai[w].city_ai[c].busy -= 1;
        }
        // **A gatherer counts in its building's city, not the nearest one**
        // (item 752, `docs/AI.md` §73). The listing keeps the found city in
        // `edi` and overwrites it with the target building's `+0x72` city
        // at `6babfb` (`movswl 0x72(%eax), %edi`), once the building is
        // the leader's own and its city is not negative; `+0x5c`'s
        // increment at `6bac3c` and the `peasant_dist` minimum after it
        // index `edi`. The decompiler prints the same reuse as `iVar25`.
        // The distance stays the found city's, `objects+0x1fc`.
        let mut counted = found.map(|(c, _)| c);
        if let Some(b) = target
            && self.buildings[b].owner == who
            && let Some(tc) = self.buildings[b].city
        {
            self.ai[w].city_ai[tc].busy += 1;
            counted = Some(tc);
        }
        self.ai[w].census.gatherers += 1;
        if let Some(c) = counted {
            let rec = &mut self.ai[w].city_ai[c];
            rec.gatherers = rec.gatherers.wrapping_add(1);
            // SEAM: a gatherer with no friendly city in its region but a
            // target in one reads the search's untouched `find_dist`
            // (99,999,999) truncated to a short; no capture has one, so
            // its minimum is skipped.
            if let Some((_, dist)) = found {
                rec.peasant_dist = rec.peasant_dist.min(dist / CELL);
            }
        }
        if land_reg {
            self.ai[w].census.reg_gatherers[r] += 1;
        }
        None
    }

    /// A garrisoned citizen: the oil slot when it is inside an oil
    /// platform. The transport half — a citizen riding a ship with a
    /// building-targeting order — is a seam.
    ///
    /// Two arms, on the outermost container (`plan_strategy@006b9620`, the
    /// citizen branch's `!is_on_map` half): an oil platform fills a slot of
    /// oil; and **a sea-domain unit whose current order is a move** — vslot
    /// `+0x14`, `is_move`, which every `MoveOrder` class answers — makes the
    /// rider an `xport` peasant in the region of the move's point, the cell
    /// under `MoveOrder::x/y` (`reg_xport_peasants[r]++`, `xport_peasants++`,
    /// both only when that cell has a region). Neither arm counts it free,
    /// busy or in `reg_peasants`: a rider is not on the map (`docs/AI.md`
    /// §58).
    fn census_citizen_inside(&mut self, who: Player, u: usize) -> Option<u16> {
        let w = who as usize;
        if let Some(b) = self.units[u].inside {
            if self.buildings[b]
                .ty
                .is_some_and(|t| build::is(&self.build_types, t, build::Ident::OilPlatform))
            {
                self.ai[w].census.filled_gather_slots[5] += 1;
            }
            return None;
        }
        let boat = self.units[u].inside_unit?;
        if self.unit_domain(boat) != Domain::Sea {
            return None;
        }
        let Some(crate::orders::Order {
            body: crate::orders::Body::Move(m),
            ..
        }) = self.units[boat].orders.front()
        else {
            return None;
        };
        let r = self.world.region_of(m.dest.cell())?;
        let cs = &mut self.ai[w].census;
        if let Some(x) = cs.reg_xport_peasants.get_mut(usize::from(r)) {
            *x += 1;
        }
        cs.xport_peasants += 1;
        Some(r)
    }

    /// Step 11, the building census: the gather slots and the defensive
    /// score. The original walks the build list and the wall list, which is
    /// one array here.
    fn census_buildings(&mut self, who: Player) {
        let w = who as usize;
        let mut slots = [0i32; RESOURCES];
        let mut defense = 0;
        let mut reg_defense: Vec<(usize, i32)> = Vec::new();
        for b in 0..self.buildings.len() {
            let bd = &self.buildings[b];
            if !bd.alive || bd.owner != who {
                continue;
            }
            let Some(rec) = bd.ty else {
                continue;
            };
            let bt = &self.build_types[rec];
            let reg = self.world.region_of(bd.pos.cell());
            // **The active test gates the gather slots alone**: its `je`
            // at `006bb09c` lands on `006bb15a`, the defensive count's own
            // guard, so an unfinished Keep counts (item 1377).
            if bd.active
                && bt.has(build::flags::GATHER)
                && let Some(good) = crate::ai_place::gather_good(bt.ident)
            {
                slots[good] += bd.gather_max.unwrap_or(0);
            }
            // A dock is exempt: the guard is the *type's* domain, not the
            // region's.
            if bt.domain() == build::BuildDomain::Water || bt.attack == 0 {
                continue;
            }
            let Some(r) = reg.filter(|r| self.region_is_land(*r)) else {
                continue;
            };
            let n = if build::is_tower(&self.build_types, rec) {
                1
            } else if build::is_fort(&self.build_types, rec) {
                2
            } else {
                0
            };
            defense += n;
            reg_defense.push((r as usize, n));
        }
        // **The census writes the ledger's own array** (item 3332):
        // `plan_strategy@006b9620:1162`–`1191` stores `local_58` into
        // `LeaderData +0x8a4`, the one field `Build::activate` and
        // `Build::close` also move. This crate once kept the census's sum in
        // a second copy, so a Farm finishing between two censuses reached
        // the ledger and never the AI's gather gate. `gather_slots[2]` —
        // wealth — is the one slot the census never writes: no building
        // gathers it, and the store keeps what the commerce slots made it.
        let ledger = &mut self.ledgers[w];
        for g in [0, 1, 3, 4, 5] {
            ledger.gather_slots[g] = slots[g];
        }
        let cs = &mut self.ai[w].census;
        cs.defense += defense;
        for (r, n) in reg_defense {
            cs.reg_defense[r] += n;
        }
    }

    /// Step 12, the maxima and the rare count.
    fn census_maxima(&mut self, who: Player) {
        let w = who as usize;
        let cities = self.city_num(who);
        let villages = self.village_num(who);
        // `+0x8d4`, the ledger's high-water mark, raised to the census's
        // count (`plan_strategy:1163`–`1167`) — the same field
        // `Build::activate`'s bonus reads.
        let ledger = &mut self.ledgers[w];
        for g in [0, 1, 3, 4, 5] {
            ledger.gather_slots_high[g] = ledger.gather_slots_high[g].max(ledger.gather_slots[g]);
        }
        let cs = &mut self.ai[w].census;
        cs.peasant_high = cs.peasant_high.max(cs.peasants);
        cs.scholar_high = cs.scholar_high.max(cs.scholars);
        cs.caravan_high = cs.caravan_high.max(cs.caras);
        cs.merchant_high = cs.merchant_high.max(cs.merchants);
        cs.army_high = cs.army_high.max(cs.combat);
        cs.city_high = cs.city_high.max(cities);
        cs.village_high = cs.village_high.max(villages);
        cs.population_high = cs.population_high.max(cs.pop);
        // `resources_controlled = bit_count(rare)`; the rares are a seam,
        // so the mask is empty and the count is zero.
        cs.resources_controlled = 0;
    }

    /// Step 13, the per-city site picture: the circle walk over the city's
    /// own radius.
    fn census_city_sites(&mut self, who: Player) {
        let w = who as usize;
        let world_cells = self.world.cell_count();
        let circle = crate::ai_place::circle();
        for c in self.cities_of(who) {
            let level = self.city_level_of(c);
            let indians = self.nation[w].indians;
            let radius = radius_for_level(&self.tuning, level, indians);
            let k = ((radius + 2) / 4) as usize;
            let city_reg = self.cities[c].reg;
            let centre = self.cities[c].pos.cell();

            {
                let pop = self.cities[c].pop;
                let cs = &mut self.ai[w].census;
                cs.city_pop_high = cs.city_pop_high.max(pop);
            }
            {
                let rec = &mut self.ai[w].city_ai[c];
                rec.ocean = 0;
                rec.land = 0;
                rec.filled = 0;
                rec.dock_tile = 0;
                rec.space = [0; 3];
                rec.ter = [0; RESOURCES];
            }
            let slots = self.city_gather_slot_total(c);
            if let Some(r) = city_reg {
                self.ai[w].census.reg_gather_slots[r as usize] += slots;
            }
            if self.cities[c].no_heal {
                self.ai[w].census.attacked += 1;
                if let Some(r) = city_reg.filter(|r| self.region_is_land(*r)) {
                    self.ai[w].census.reg_attacked[r as usize] += 1;
                }
            }

            let outer = circle.radius.get(k + 1).copied().unwrap_or(0);
            let inner = circle.radius.get(k).copied().unwrap_or(0);
            if outer > 1 {
                // **The walk starts at entry 1** (`docs/AI.md` §122): the
                // loop at `plan_strategy@006b9620:0x6bb6f7` reads the offset
                // tables at `circle_x + 1` and `circle_y + 1` from `i = 0`
                // and stops on `i + 1 < circle_radius[k + 1]`, so the
                // centre's own `(0, 0)` is never visited and the last entry
                // before `outer` is — every other reader of the tables
                // starts at `+0`.
                for i in 1..outer {
                    let cell = Cell::new(centre.x + circle.x[i], centre.y + circle.y[i]);
                    if !self.world.contains(cell) {
                        continue;
                    }
                    self.census_site_cell(who, c, cell, i, inner, city_reg, world_cells);
                }
            }

            let rec = self.ai[w].city_ai[c];
            let open = rec.land - rec.filled;
            if open < 2 {
                self.ai[w].census.full_cities += 1;
            }
            if let Some(r) = city_reg {
                self.ai[w].census.reg_land[r as usize] += open;
            }
        }
    }

    /// One cell of step 13's circle.
    #[allow(clippy::too_many_arguments)]
    fn census_site_cell(
        &mut self,
        who: Player,
        c: usize,
        cell: Cell,
        i: usize,
        inner: usize,
        city_reg: Option<u16>,
        world_cells: i32,
    ) {
        let w = who as usize;
        let d = self.world.cell_data(cell);
        // Mine or nobody's.
        match self.world.owner(cell) {
            Owner::Player(p) if p != who => return,
            _ => {}
        }
        // A water cell — `land` 1 or 2, and not flagged coastal.
        if d.flags & 0x100 == 0 && (d.land == 1 || d.land == 2) {
            self.ai[w].city_ai[c].ocean += 1;
            if let Some(r) = self.world.region_of(cell) {
                let big =
                    self.world.num_coasts(r) > 1 || self.world.region_size(r) >= world_cells / 10;
                if big && self.ai[w].city_ai[c].dock_tile == 0 && self.is_dock_tile(cell) {
                    self.ai[w].city_ai[c].dock_tile += 1;
                }
            }
            return;
        }
        // Land, inside the inner ring, in the city's own region. `i` is the
        // table entry (`i + 1` in the original's own counter, tested `<
        // circle_radius[k]`).
        if i >= inner || self.world.region_of(cell) != city_reg {
            return;
        }
        if d.flags & 0x70 != 0 {
            self.census_site_gather(who, c, cell);
            return;
        }
        self.ai[w].city_ai[c].land += 1;
        let space = self.check_building_wcoord(who, cell, 0, 0, 1, true);
        if space > 1 {
            for n in 2..=space.min(4) {
                self.ai[w].city_ai[c].space[(n - 2) as usize] += 1;
            }
        }
        if space >= 4 {
            // A wide-open cell is *not* counted filled; its gather value is
            // recorded instead, exactly as an occupied one's is.
            self.census_site_gather(who, c, cell);
            return;
        }
        self.ai[w].city_ai[c].filled += 1;
    }

    /// `World::gather_at` into `CityAi::ter`, per good, as the maximum
    /// over the circle's cells. Step 13 is the `centre_only` caller
    /// (`docs/AI.md` §24), so each cell answers out of its own land class
    /// alone.
    fn census_site_gather(&mut self, who: Player, c: usize, cell: Cell) {
        let amounts = self.world.gather_at(cell, true);
        let rec = &mut self.ai[who as usize].city_ai[c];
        for (t, a) in rec.ter.iter_mut().zip(amounts) {
            *t = (*t).max(a);
        }
    }

    /// Step 14: wars and alliances with every other met computer leader.
    fn census_wars(&mut self, who: Player) {
        let w = who as usize;
        let mut wars = 0;
        let mut allies = 0;
        for i in 0..self.players.len() {
            if i == w || !self.met(who, i) {
                continue;
            }
            if self.diplo_war(w, i) || self.diplo_war(i, w) {
                wars += 1;
            } else if self.diplo_ally(w, i) && self.diplo_ally(i, w) {
                allies += 1;
            }
        }
        let cs = &mut self.ai[w].census;
        cs.wars = wars;
        cs.allies = allies;
        cs.active_wars = 0;
        cs.active_wars_with = 0;
    }

    /// Step 15, the per-region strategy word.
    ///
    /// Bit 1 is set when the region is *thin* — every leader's cities there
    /// times fifty do not fill it. When it is not thin and I have a city
    /// there, the word starts at **8** rather than 0; the original's `else
    /// if` arm is the only place that value comes from, and `docs/AI.md`
    /// §2.3 step 15 does not mention it.
    fn census_strategy(&mut self, who: Player) {
        let w = who as usize;
        let players = self.players.len();
        // `world+0x34`, the style's sea class (`World::sea_map`).
        let sea_map = self.world.sea_map();
        let difficulty = self.ai_difficulty();
        let start_res_eight = self.lobby.starting_resources == 8;
        for r in 0..self.world.region_count() {
            if !self.region_is_land(r as u16) {
                continue;
            }
            {
                let cs = &mut self.ai[w].census;
                cs.reg_wars[r] = 0;
                cs.reg_allies[r] = 0;
                cs.reg_neutrals[r] = 0;
                cs.strategy[r] = 0;
            }
            let total: i32 = (0..players)
                .filter(|&i| !self.defeated[i])
                .map(|i| self.reg_city_buildings(i as Player, r as u16))
                .sum();
            let thin = total * 50 < self.world.region_size(r as u16);
            let mine = self.ai[w].census.reg_cities[r];
            if thin {
                self.ai[w].census.strategy[r] = 1;
            } else if mine != 0 {
                self.ai[w].census.strategy[r] = 8;
            } else {
                continue;
            }
            if mine == 0 {
                continue;
            }

            let my_attack = self.ai[w].census.attack;
            let home = self.ai[w].census.home_reg;
            for i in 0..players {
                if i == w || self.defeated[i] || !self.met(who, i) {
                    continue;
                }
                let theirs = self.reg_city_buildings(i as Player, r as u16);
                if theirs == 0 {
                    continue;
                }
                if self.diplo_war(w, i) || self.diplo_war(i, w) {
                    {
                        let cs = &mut self.ai[w].census;
                        cs.active_wars += 1;
                        cs.active_wars_with |= 1 << (i & 0x1f);
                        cs.reg_wars[r] += 1;
                    }
                    // Weaker: my attack is under theirs, and either this is
                    // my home region or they hold fewer cities here than I
                    // do. (The decompile's operands are that way round;
                    // §2.3 has them the other way — see the report.)
                    let weaker =
                        my_attack < self.ai[i].census.attack && (home == r as i32 || theirs < mine);
                    let cs = &mut self.ai[w].census;
                    cs.strategy[r] = if weaker {
                        (cs.strategy[r] & 0xfff5) | 4
                    } else {
                        (cs.strategy[r] & 0xfffb) | 2
                    };
                    cs.strategy[r] = if difficulty < 2 {
                        (cs.strategy[r] & 0xfffd) | 4
                    } else if start_res_eight {
                        (cs.strategy[r] & 0xfffb) | 2
                    } else {
                        cs.strategy[r]
                    };
                } else if self.diplo_ally(w, i) && self.diplo_ally(i, w) {
                    self.ai[w].census.reg_allies[r] += 1;
                } else {
                    self.ai[w].census.reg_neutrals[r] += 1;
                }
            }

            let cs = &mut self.ai[w].census;
            if sea_map < 1 {
                cs.strategy[r] &= 0xfff7;
            } else if sea_map == 2 {
                // The class-2 probe wants a region flagged `8` with
                // nobody's city in it — a seam, so it never fires.
            } else if sea_map >= 3 && cs.strategy[r] & 6 == 0 {
                cs.strategy[r] |= 8;
            }
        }
    }

    /// `Leader::check_explore@006bc860`: the explored recount
    /// (`docs/AI.md` §2.1). Its cadence lives in the driver
    /// ([`crate::ai::cadence::explore_due`]), so this is the body only.
    ///
    /// With `EXPLORE_MAP_BONUS` the original answers `reg_size`
    /// (`world+0x2c`) outright; otherwise it walks the **region grid**
    /// (`reg_xs × reg_ys`, `world+0x24`/`+0x28`) and counts the cells whose
    /// sample byte in `seen2` (`world+0x160`, stride `fog_xs`) carries the
    /// leader's bit. That grid is two world cells across each way: `seen2`
    /// is indexed `div_3_table[pos >> 7]`, half a world cell, so `fog_xs`
    /// is twice `xs`, and check_explore's `fog_xs * (4·ry + 3) + 4·rx + 3`
    /// steps four fog cells per region cell.
    ///
    /// There is no visibility here, so the seam answers what
    /// `EXPLORE_MAP_BONUS` would: the whole region grid. The rounding when
    /// the world has an odd extent is not established.
    pub fn check_explore(&mut self, who: Player) {
        let w = who as usize;
        self.ai[w].census.explored = (self.world.width() / 2) * (self.world.height() / 2);
    }
}

impl Sim {
    /// `LeaderData::wonder_mark`'s writers, on every leader: the entry
    /// `Wonders::close_wonder@0073c7e0` clears when a wonder closes, and
    /// the one `Wonders::init_wonder@0073c860` writes when one activates
    /// (`Build::activate`'s wonder arm, after `remove_unbuilt_wonder`).
    ///
    /// - **A close** clears its entry and walks the mark down while the
    ///   entry under it is clear, so the mark is one past the highest
    ///   entry in use.
    /// - **An activation** takes the first clear entry under the mark, or
    ///   the one at it, and raises the mark past it.
    ///
    /// The original writes both inside the objects' pass, so this runs at
    /// that pass's end: a wonder that activates on sim-frame N is on the
    /// dump's block N + 1 and is read by the next frame's
    /// `create_buildings` (`docs/AI.md` §75). It is derived from the
    /// buildings rather than called from the activation, in building
    /// order. A wonder closes when it is no longer an alive, activated
    /// wonder of the leader's; a capture is not read.
    pub fn note_wonders(&mut self) {
        for w in 0..self.ai.len() {
            let who = w as Player;
            let standing: Vec<usize> = (0..self.buildings.len())
                .filter(|&b| {
                    let bd = &self.buildings[b];
                    bd.alive
                        && bd.active
                        && bd.owner == who
                        && bd.ty.is_some_and(|t| self.build_types[t].wonder)
                })
                .collect();
            let cs = &mut self.ai[w].census;
            // `close_wonder`: clear, then walk the mark down.
            for e in &mut cs.wonder_slots {
                if e.is_some_and(|b| !standing.contains(&b)) {
                    *e = None;
                }
            }
            while cs.wonder_mark > 0 && cs.wonder_slots[cs.wonder_mark as usize - 1].is_none() {
                cs.wonder_mark -= 1;
            }
            // `init_wonder`: the first clear entry under the mark, else the
            // mark's own.
            for b in standing {
                if cs.wonder_slots.contains(&Some(b)) {
                    continue;
                }
                let mark = cs.wonder_mark as usize;
                let i = cs.wonder_slots[..mark]
                    .iter()
                    .position(Option::is_none)
                    .unwrap_or(mark);
                if cs.wonder_slots.len() <= i {
                    cs.wonder_slots.resize(i + 1, None);
                }
                cs.wonder_slots[i] = Some(b);
                cs.wonder_mark = cs.wonder_mark.max(i as i32 + 1);
                // `Game::wonders[t − 0x20e] |= 1`, in the same call.
                let tree = self.buildings[b].ty.and_then(|t| self.build_types[t].tree);
                let line = &self.tech_tree.roles.wonder_line;
                if let Some(k) = tree.and_then(|tr| line.iter().position(|&w| w == tr)) {
                    self.wonders_built |= 1 << k;
                }
            }
        }
    }
}

/// Step 3: the scalars the sweep zeroes. `control`, `pop`, `scouts`, the
/// maxima and the `reg_*` arrays the building lifecycle keeps are all
/// deliberately absent.
fn census_zero_scalars(cs: &mut Census) {
    cs.active = 0;
    cs.combat = 0;
    cs.non_siege = 0;
    cs.sea_combat = 0;
    cs.siege = 0;
    cs.defense = 0;
    cs.attack = 0;
    cs.naval = 0;
    cs.air = 0;
    cs.missile = 0;
    cs.transports = 0;
    cs.fishermen = 0;
    cs.idle_fishermen = 0;
    cs.peasants = 0;
    cs.scholars = 0;
    cs.caras = 0;
    cs.merchants = 0;
    cs.fighters = 0;
    cs.bombers = 0;
    cs.cruise = 0;
    cs.nuke = 0;
    cs.free_peasants = 0;
    cs.xport_peasants = 0;
    cs.gatherers = 0;
    cs.attacked = 0;
    cs.full_cities = 0;
}

#[cfg(test)]
mod tests;
