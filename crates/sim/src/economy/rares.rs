//! `Leader::calc_gather`'s **step 6** — every idle fisherman and merchant —
//! and the one thing owning a rare does to a unit.
//!
//! The specification is `docs/ECONOMY.md`, step 6 and "What an owned rare
//! does". Three pieces, and they are one mechanic because the middle one is
//! the only reason the first has a visible effect this early in a game:
//!
//! 1. **The walk.** `Leader::calc_gather@006ceee0` clears `rare_owned`, then
//!    walks the player's objects in `o` order and hands every `is(FISHERMEN)`
//!    or `UnitData::is_merchant` whose `order_type` is `NONE` to
//!    `Unit::do_gather@005fce20`. Each answers the good it stands on; the
//!    good's payout ([`economy::calc_rare`]) is divided by the crowd sharing
//!    it and added to the player's rate, and the good's bit goes into
//!    `rare_owned`.
//! 2. **The mask.** `Leader::gather@006ce280` then sets `rare =
//!    rare_owned | rare_conquest` and, when that differs from last frame's,
//!    raises `LeaderData`'s `0x4000000`. One bit of the forty-four is
//!    singled out there: when **Gems** ([`economy::GEMS`], bit 23) moves,
//!    and only then, the writer invalidates every region's border stamp —
//!    so [`Sim::tick`] re-runs `sync_territory` on that change alone
//!    (`docs/ATTRITION.md`, "Territory").
//! 3. **`Leader::calc_unit_stats@006cf970`**, which `Leader::process` runs on
//!    the same frame the flag goes up, calls `Unit::update_speed@006055c0` on
//!    every one of that player's units. Its one rare arm is **Whales**:
//!    a naval type's cached speed becomes `(WHALES_SHIPS_MOVE + 100) / 100`
//!    of itself.
//!
//! That third step is what makes this measurable rather than merely correct.
//! East Indies' AI puts a second Fisherman on a whale on frame 5551 and every
//! ship it owns goes from 38 to 45 in the same frame — the Fisherman walking
//! to its own deposit then arrives twenty-three frames earlier, and the word
//! is where that arrival is.
//!
//! # What is not modelled
//!
//! - **The Porcelain Tower's pass**, which sets `rare_owned` bits for every
//!   rare inside the player's own territory whether anyone stands on it or
//!   not. It is the one exception to "a rare pays only through a merchant"
//!   and it is the wonder layer's.
//! - **`rare_conquest`**, Conquer-the-World's own mask; always empty here.
//! - **Everything else `update_speed` does** — the transport bonus
//!   (`MILITARY_TRANSPORT_BONUS`, 0 as shipped), Bantu, French siege,
//!   Versailles, aluminium, the spy/general upgrade counts and the two
//!   Aztec types (`docs/MOVEMENT.md`, "The speed pipeline"). None of them is
//!   modelled anywhere in this crate, so recomputing a unit's cached speed
//!   from its type's `MOVES` loses nothing that was ever there.
//! - **`calc_unit_stats`' other three calls** — `calc_attrition`,
//!   `calc_anti_attrition` and `Unit::update_armor`.

use crate::economy::{self, RESOURCES};

/// The gunpowder foot line's `TypeIndex`es, the four `update_speed` tests
/// by identity (`enums/TypeIndex.txt`: 98, 100, 102, 104).
const ARQUEBUSIERS: i32 = 0x62;
const RIFLEMAN: i32 = 0x64;
const INFANTRY: i32 = 0x66;
const MECH_INFANTRY: i32 = 0x68;
/// The Bantu arm's identities (`enums/TypeIndex.txt`): the Barracks as a
/// trainer, and the two citizens.
const BARRACKS: TypeId = 0x1ab;
const PEASANTS: i32 = 0x32;
const PEASANTS_KOREAN: i32 = 0x33;
use crate::tech::TypeId;
use crate::world::Player;
use crate::{Sim, orders};

impl Sim {
    /// `LeaderData::get_fishermen@006d6e80` — the highest `FISHERMEN`
    /// bonus row whose prerequisite the player holds, 0 for none.
    pub(crate) fn fishermen_level(&self, who: Player) -> usize {
        self.bonus_level(who, &self.tables.tech_tree.roles.fishermen_preq)
    }

    /// `LeaderData::get_merchants_level@006d6dc0`.
    pub(crate) fn merchants_level(&self, who: Player) -> usize {
        self.bonus_level(who, &self.tables.tech_tree.roles.merchants_preq)
    }

    /// The shared shape of the two: the rows are tested most advanced
    /// first and the answer is that row's one-based position. A row the
    /// tree does not know is not held — the opposite of the *bonus* rule
    /// ([`crate::tech::Roles::colonize_preq`]), because here a missing
    /// name must not invent an upgrade the player never bought.
    pub(crate) fn bonus_level(&self, who: Player, rows: &[Option<TypeId>]) -> usize {
        top_held(rows.len(), |i| {
            rows[i].is_some_and(|t| {
                self.tables
                    .tech_tree
                    .has_tech(&self.tables.setup, &self.tech[who as usize], t)
            })
        })
    }

    /// The tier of a **government's bonus ladder** held — `DESPOTISM_1..3` or
    /// `REPUBLIC_1..3`, tested most advanced first like [`Sim::bonus_level`]
    /// but with `has_preq`'s own gate on a tier above the first: its `preq0`,
    /// two governments taken (three for the third tier), and every tier under
    /// it held the same way (twenty-fourth pass, group 21; A1 rows 5 and 6).
    /// Every shipped tier carries one constant, so no price moves today.
    pub(crate) fn government_bonus_level(&self, who: Player, rows: &[Option<TypeId>]) -> usize {
        let p = &self.tech[who as usize];
        let held = |tier: usize| -> bool {
            (0..=tier).all(|k| {
                let Some(t) = rows.get(k).copied().flatten() else {
                    return false;
                };
                let below = k.checked_sub(1).and_then(|b| rows[b]);
                self.tables.tech_tree.has_tech(&self.tables.setup, p, t)
                    && self
                        .tables
                        .tech_tree
                        .bonus_tier_extra(&self.tables.setup, p, k, below)
            })
        };
        (0..rows.len())
            .rev()
            .find(|&i| held(i))
            .map_or(0, |i| i + 1)
    }

    /// Step 6 whole: the rates it adds and the `rare_owned` it rebuilds.
    ///
    /// Called from [`Sim::assemble_holdings`], which runs inside the same
    /// cadence gate the original's recompute does — so a rare appears on
    /// exactly the frame it appears there.
    pub(crate) fn gather_rares(&mut self, who: Player) -> ([i32; RESOURCES], u64) {
        let mut out = [0; RESOURCES];
        let mut owned = 0u64;
        let mut collected = vec![0; economy::RARES];
        let candidates: Vec<usize> = (0..self.units.len())
            .filter(|&u| {
                let unit = &self.units[u];
                unit.owner == who && unit.alive() && unit.on_map
            })
            .collect();
        for u in candidates {
            // `is(0x13d, 0) || is_merchant`, then `order_type() == NONE`.
            // A merchant walking somewhere earns nothing, which is also
            // why a boat only ever pays between two `think_fish` searches.
            if !(self.unit_line_is(u, crate::units::think::fish::FISHERMEN) || self.is_merchant(u))
            {
                continue;
            }
            if self.order_type(u) != orders::index::NONE {
                continue;
            }
            let Some((good, crowd)) = self.do_gather_rates(u) else {
                continue;
            };
            let Some(g) = self.economy.good_types.get(good).copied() else {
                continue;
            };
            // The ally test the good's own cell answers — `is_ally` in the
            // cached arm and the two-sided diplomacy compare in the
            // spiral's, which are the same predicate.
            let ally = self.gather_cell_friendly(u, who);
            let rate = economy::calc_rare(
                &self.tables.tuning,
                good,
                &g,
                self.fishermen_level(who),
                self.merchants_level(who),
                ally,
                self.tables.tech_tree.has_tribe_bonus(
                    &self.tables.setup,
                    &self.tech[who as usize],
                    crate::ai::tribe::NUBIANS,
                ),
                self.tables.tech_tree.has_tribe_bonus(
                    &self.tables.setup,
                    &self.tech[who as usize],
                    crate::cities::nations::power::JAPANESE,
                ),
            );
            let share = crowd + 1;
            for (i, r) in rate.iter().enumerate() {
                out[i] += r / share;
            }
            if good >= economy::BASE_RARE && good - economy::BASE_RARE < economy::RARES {
                owned |= 1 << (good - economy::BASE_RARE);
                collected[good - economy::BASE_RARE] += 1;
            }
        }
        // `rares_collected`, cleared and refilled by the same walk.
        if let Some(h) = self.economy.holdings.get_mut(who as usize) {
            h.rares_collected = collected;
        }
        (out, owned)
    }

    /// Whether the tile the unit found its good on is the finder's own
    /// ground or an ally's — `local_20` in `UnitData::calc_gather`.
    ///
    /// The original reads the **good's** cell rather than the unit's; both
    /// walks record the index they accepted, so the cell is the one
    /// `good_obj` names.
    fn gather_cell_friendly(&self, u: usize, who: Player) -> bool {
        let i = self.units[u].good_obj;
        let Ok(i) = usize::try_from(i) else {
            return false;
        };
        let circle = crate::ai::ai_place::circle();
        let (Some(&dx), Some(&dy)) = (circle.x.get(i), circle.y.get(i)) else {
            return false;
        };
        let t = self.units[u].pos.tile();
        let c = crate::world::Cell::new(
            (t.x + dx).div_euclid(crate::world::TILES_PER_CELL),
            (t.y + dy).div_euclid(crate::world::TILES_PER_CELL),
        );
        let Some(owner) = self.world.owner(c).player() else {
            return false;
        };
        owner == who
            || (self.allied_both_ways(who as usize, owner as usize)
                && self.allied_both_ways(owner as usize, who as usize))
    }

    fn allied_both_ways(&self, a: usize, b: usize) -> bool {
        self.leaders
            .allied
            .get(a)
            .and_then(|r| r.get(b))
            .copied()
            .unwrap_or(false)
    }

    /// `Unit::update_speed@006055c0`, as much of it as this crate models:
    /// the type's `MOVES`, the **Whales** arm, and the **gunpowder foot
    /// line's** hardcoded correction after it (`docs/MOVEMENT.md`, "The speed
    /// pipeline" §1).
    ///
    /// The foot line is four **lineage** tests, `is(t, 0)` through the type's
    /// vslot `0x60` — `ObjectTypeData::is@0065f7d0`, read off the PE at
    /// `0xb41d24` and `0xb41fd4` (twenty-fourth pass, group 22): the type
    /// itself, then its `is_list`, then the graft and `from` chain — and one
    /// scale each, in the order Mechanized Infantry, Infantry, Rifleman,
    /// Arquebusiers, every division truncating toward zero (the listing,
    /// `605700`..`6057d6`). So every national replacement of a line takes its
    /// scale (thirty-one shipped types, the adjudicator's count; an earlier
    /// reading here called it identity and moved four exact indices only):
    /// Mechanized Infantry `×36/32`, Infantry `×34/32`, Rifleman `×32/27`,
    /// Arquebusiers `×30/24`. run404 prints Infantry at **34** on a `MOVES`
    /// of 32 from its birth on 621 (item 1109).
    /// The supply-bit upgrade adds `level * speed / 4` after those terms
    /// (SUPPLY, "Upgrade speed", item 1441).
    pub(crate) fn type_speed(&self, who: Player, ty: usize) -> i32 {
        self.speed_of(who, ty, false)
    }

    /// `Unit::update_speed` of the unit `u` as type `ty`: [`Sim::type_speed`]
    /// with the marines' bit read off the unit.
    pub(crate) fn own_speed(&self, u: usize, ty: usize) -> i32 {
        let marine = self.units[u].unit_masks2 & crate::combat::umask2::MARINES != 0;
        self.speed_of(self.units[u].owner, ty, marine)
    }

    fn speed_of(&self, who: Player, ty: usize, marine: bool) -> i32 {
        let mut speed = self.tables.unit_types[ty].moves;
        // **The American marines' arm** (item 3661, `update_speed@006055c0`,
        // before the `UNIT_MOVE_SPEED` scale and every percentage): a unit
        // carrying `unit_masks2 & 0x200` adds the Military library level
        // (`epoch[0]`) times `AMERICANS_MARINE_SPEED_BONUS`. The transport
        // arm beside it, `MILITARY_TRANSPORT_BONUS`, ships as 0. run1314b's
        // who=1 is at Military 3 and its Transport Galleon `1/105` steps 31
        // on a `MOVES` of 25 (`docs/AI.md` §341).
        if marine && let Some(p) = self.tech.get(who as usize) {
            speed += p.military_level() as i32 * self.tables.tuning.americans_marine_speed_bonus;
        }
        let naval = self.tables.unit_types[ty].combat.obj_masks & crate::combat::mask::NAVAL != 0;
        if naval && self.has_rare(who, economy::WHALES) {
            speed = (self.tables.tuning.whales_ships_move + 100) * speed / 100;
        }
        let lineage = |x: i32| {
            usize::try_from(self.tables.unit_types[ty].type_index)
                .is_ok_and(|t| self.tables.tech_tree.is(t, x as usize, false))
        };
        let (num, den) = if lineage(MECH_INFANTRY) {
            (36, 32)
        } else if lineage(INFANTRY) {
            (34, 32)
        } else if lineage(RIFLEMAN) {
            (32, 27)
        } else if lineage(ARQUEBUSIERS) {
            (30, 24)
        } else {
            (1, 1)
        };
        speed = speed * num / den;
        // `Unit::update_speed@006055c0`'s **Bantu arm** (`006057d8`..
        // `605858`), after the foot line and before the French siege arm: a
        // Bantu leader's unit (`has_tribe_bonus(3)`) moves
        // `BANTU_UNITS_MOVE` faster when its `WHERE` (`UnitTypeData +0x40`)
        // is the Barracks (`0x1ab`), its `TypeIndex` is `PEASANTS` or
        // `PEASANTSKOREAN` (`0x32`, `0x33`, by identity), or it is of the
        // Militia line (`is(0x42, 0)`). The Bantu word's citizens step 31
        // on a `MOVES` of 25 from frame 1 (run978, `docs/AI.md` §276).
        if self
            .cities
            .nation
            .get(who as usize)
            .is_some_and(|n| n.bantu)
        {
            let ti = self.tables.unit_types[ty].type_index;
            if self.trainer_where(ty) == Some(BARRACKS)
                || ti == PEASANTS
                || ti == PEASANTS_KOREAN
                || lineage(crate::spells::cast::MILITIA)
            {
                speed = (self.tables.tuning.bantu_units_move + 100) * speed / 100;
            }
        }
        // `Unit::update_speed@006055c0`'s trainer arm: a unit whose
        // `WHERE` (`UnitTypeData +0x40`) is `0x1ae` or `0x1af` — the Siege
        // Factory line — moves `FRENCH_SIEGE_MOVE` faster for a French
        // leader, then `VERSAILLES_UNITS_MOVE` under Versailles, each its
        // own `(x + 100) × speed / 100` (`docs/MOVEMENT.md`, "The French
        // siege move"). French East Indies' Supply Wagon `1/72` is born at
        // 30 there, 25 here before item 1455.
        if matches!(self.trainer_where(ty), Some(0x1ae | 0x1af)) {
            if self.cities.nation[who as usize].french {
                speed = (self.tables.tuning.french_siege_move + 100) * speed / 100;
            }
            if self.wonders_held(who) & (1 << crate::tech::wonder::VERSAILLES) != 0 {
                speed = (self.tables.tuning.versailles_units_move + 100) * speed / 100;
            }
        }
        // **The Aluminum arm** (`update_speed@006055c0`, after the French
        // siege arms and before the Spy's): an air type (`+0x218 == 2`)
        // whose leader holds Aluminum — `0x6da7 & 1`, byte 3 bit 0 of the
        // `rare` mask, or the same bit of `rare_conquest` at `0x6dcf`, the
        // Whales arm's byte — moves `ALUMINUM_AIR_SPEED` faster. The German
        // Modern word's Bombers sit at 75 on a `MOVES` of 60 (run1397's
        // stride from 3101), and the range a sortie asks of them is the
        // speed times the tank (`docs/AI.md` §366.3).
        if self.tables.unit_types[ty].kind.domain == crate::territory::attrition::Domain::Air
            && self.has_rare(who, economy::ALUMINUM)
        {
            speed = (self.tables.tuning.aluminum_air_speed + 100) * speed / 100;
        }
        // The Spy's upgrades (`update_speed@006055c0:131`–`134`): a quarter
        // of the speed per level, toward zero (`docs/AI.md` §160).
        if self.is_spy_type(ty) {
            speed += self.spy_upgrade_level(who) * speed / 4;
        }
        // A hero's (`is_hero`, `unit_flags2 & 0x20`) the same quarter per
        // General upgrade (`update_speed@006055c0:135`–`144`): the coverage
        // pair's General `1/150` is born at 73 on a `MOVES` of 42 with all
        // three held (`docs/AI.md` §172).
        if self.tables.unit_types[ty]
            .cols
            .flag2(crate::ai::ai_load::uflags2::FLAG2_GENERAL)
        {
            speed += self.general_upgrade_level(who) * speed / 4;
        }
        if self.tables.unit_types[ty]
            .cols
            .flag2(crate::ai::ai_load::uflags2::SUPPLY_OR_HERO)
        {
            speed += self.supply_upgrade_level(who) * speed / 4;
        }
        speed
    }

    /// `LeaderData::has_rare@006e0770` — a good's bit in the player's
    /// `rare` mask, by **type index** rather than by bit. A good below
    /// [`economy::BASE_RARE`] is not a rare and is never in it, which is
    /// what the original's own `param_1 + -6 >> 3` would index off the
    /// front of.
    ///
    /// The original reads two masks and takes their union,
    /// `rare | rare_conquest`; this crate keeps the union itself in
    /// [`crate::economy::Ledger::rare`], which `Sim::tick` rebuilds every
    /// frame as `Leader::gather` does, so the one field answers both.
    pub fn has_rare(&self, who: Player, good: usize) -> bool {
        let Some(l) = self.economy.ledgers.get(who as usize) else {
            return false;
        };
        rare_bit(l.rare, good)
    }

    /// `World::reveal_fog@006b3d30`'s two goods arms — the rare arm and the
    /// oil arm (`docs/ECONOMY.md`, "The rares a leader has seen"; the oil
    /// arm is `docs/AI.md` §133).
    ///
    /// `(fx, fy)` is a **fog** cell, and the call happens exactly where
    /// `World::set_seen` answered that `seen2` changed, so a cell is offered
    /// once per player for the life of a game. The rare arm's own gate is a
    /// tile-mask read at `(2fx + 1, 2fy + 1)` — [`crate::world::tile`]'s
    /// `0x200`, which `Objects::init_good` sets over a good's footprint —
    /// and only then the cell's `find_good_at`. **The oil arm is
    /// independent of it**: it tests the world cell's own flag `0x800`
    /// ([`crate::world::cell::OIL`]) and then walks the goods list, from the
    /// front, for the first live good standing on the cell — whatever its
    /// type — and adds it to the revealing leader's `oil_patches` unless
    /// it is already there.
    pub(crate) fn reveal_fog(&mut self, fx: i32, fy: i32, who: Player) {
        // This crate has no unit index to hand (`param_4`, the revealing
        // unit's slot): the goody-box arm is the seam `goody.rs` names. The
        // goody arm is the item's own `ever_seen` (item 3335).
        reveal_fog_steps(
            &mut SimRevealFog {
                sim: self,
                goody: true,
            },
            fx,
            fy,
            i32::from(who),
            -1,
        );
    }

    /// [`Sim::reveal_fog`]'s rare and oil arms alone — what the harness's
    /// replay of the dump's `seen2` ([`Sim::seed_new_rares_from_fog`])
    /// stands in for. Its goody arm is not replayed: the dump prints each
    /// box's `ever_seen` itself, and a `seen2` bit a building's light set
    /// never passed through `reveal_fog` (item 3335).
    pub(crate) fn reveal_fog_goods(&mut self, fx: i32, fy: i32, who: Player) {
        reveal_fog_steps(
            &mut SimRevealFog {
                sim: self,
                goody: false,
            },
            fx,
            fy,
            i32::from(who),
            -1,
        );
    }

    /// The first good in the goods list, front to back, that is alive and
    /// stands on `c` — the scan both `reveal_fog`'s oil arm and
    /// `compute_reg_territory`'s per-cell arm make (`div_3_table[(pos ^
    /// 0x63637) >> 8]` on each axis against the cell).
    pub(crate) fn first_live_good_at(&self, c: crate::world::Cell) -> Option<usize> {
        self.world
            .goods()
            .iter()
            .position(|g| g.alive && g.pos.cell() == c)
    }

    /// `ArrayBase<int>::add` guarded by the linear scan both writers make
    /// first: the patch joins the leader's list once.
    pub(crate) fn add_oil_patch(&mut self, who: Player, gi: usize) {
        let list = &mut self.ai.ai[who as usize].oil_patches;
        if !list.contains(&gi) {
            list.push(gi);
        }
    }

    /// `World::compute_reg_territory@006b0bb0`'s per-cell goods scan, for
    /// one cell the pass has just given to `who`: when the first live good
    /// standing on it is an oil patch, it goes into that leader's
    /// `oil_patches`; any other good is met as a rare once the cell is seen
    /// (below).
    pub(crate) fn claim_cell_goods(&mut self, c: crate::world::Cell, who: Player) {
        let Some(gi) = self.first_live_good_at(c) else {
            return;
        };
        if self.world.goods()[gi].ty == crate::world::OIL_GOOD {
            self.add_oil_patch(who, gi);
            return;
        }
        // **The rare arm** (item 1555, `docs/AI.md` §152): a good the pass
        // finds on a cell it gives `who` goes to `Leader::new_rare` when
        // `who` or an ally has seen the cell's second half-cell each way
        // (`seen2[2cy + 1][2cx + 1] & ally_mask`, [`Sim::was_really_seen`]
        // exactly) — `reveal_fog`'s tile-mark gate does not stand in front
        // of it, so a good whose footprint misses the revealed half-cell
        // is met here — and the good's `ever_seen` takes the owner's bit.
        if !self.was_really_seen(c, who) {
            return;
        }
        self.new_rare(who, gi);
        self.mark_good_seen(gi, who);
    }

    /// `GoodData::ever_seen |= 1 << who` (`+0x20`), as `reveal_fog`'s rare
    /// arm and `compute_reg_territory`'s goods scan both write it.
    pub(crate) fn mark_good_seen(&mut self, gi: usize, who: Player) {
        if self.territory.good_seen_bits.len() <= gi {
            self.territory.good_seen_bits.resize(gi + 1, 0);
        }
        if who < 8 {
            self.territory.good_seen_bits[gi] |= 1 << who;
        }
    }

    /// [`Sim::claim_cell_goods`] over every cell at once — the wholesale
    /// recompute of setup and of [`Sim::settle_borders`], and the harness's
    /// stand-in for the passes a dump's installed owner grid skipped. Oil
    /// patches in the goods list's order, which is the order the original's
    /// per-cell scan reaches them in only where it reaches them by cell.
    pub fn claim_oil_from_owners(&mut self) {
        // **A land patch joins in the per-cell scan's own order** (item
        // 1538, `docs/AI.md` §147): `compute_reg_territory` walks the land
        // regions in order and each region's cells row-major
        // ([`Sim::advance_border_pass`]), so two patches tied on score reach
        // `pick_oil_patch` in that order and the original's tie goes to the
        // other one than the goods list's order picks.
        let mut land: Vec<((u16, usize), usize)> = Vec::new();
        let mut rest = Vec::new();
        for gi in 0..self.world.goods().len() {
            let g = self.world.goods()[gi];
            if !g.alive || g.ty != crate::world::OIL_GOOD {
                continue;
            }
            let c = g.pos.cell();
            if self.first_live_good_at(c) != Some(gi) {
                continue;
            }
            match self.world.region_of(c) {
                Some(r)
                    if self
                        .world
                        .regions()
                        .any(|(i, t)| i == r && t == crate::world::Terrain::Land) =>
                {
                    land.push(((r, (c.y * self.world.width() + c.x) as usize), gi));
                }
                _ => rest.push(gi),
            }
        }
        land.sort_unstable();
        for gi in land.into_iter().map(|(_, gi)| gi).chain(rest) {
            let c = self.world.goods()[gi].pos.cell();
            if let Some(p) = self.world.owner(c).player() {
                self.add_oil_patch(p, gi);
            }
        }
    }

    /// **Replay the start-of-game rare reveals over an installed fog
    /// grid** — the reconstruction a harness owes a simulation it starts
    /// from a snapshot rather than from `Game::init`.
    ///
    /// [`Sim::reveal_fog`] is reached from one place, `World::set_seen`
    /// answering that `seen2` **changed**, so a good is offered to a
    /// leader once for the life of a game. A harness that *installs* the
    /// dump's `seen2` instead of walking the sweeps that produced it
    /// therefore skips every offer the original had already made when the
    /// block was written, and `new_rares` starts empty where the
    /// original's already holds the rares the leader had seen. Nothing
    /// downstream can recover them: the cells are seen, so `set_seen` will
    /// never answer true for them again.
    ///
    /// That is what sent Great Lakes' AI Merchant to the wrong rare.
    /// `1/24` is born on 5753 and `think_merchant` scores `new_rares`; the
    /// original's list holds the `SILK` at `(40320, 14208)` — seen at
    /// game start, so first in the list and worth the full 200 — and this
    /// crate's held only the two goods a unit walked past *during* the
    /// run, so it picked the one at `(31800, 21816)` and set off
    /// south-west across the map (`docs/MERCHANT.md` §2.2, item 207).
    ///
    /// **The order is the grid's, not the sweep's**, and that is the one
    /// thing this cannot reconstruct: `new_rare` appends, and the score's
    /// base falls ten a slot, so two goods first seen in the same
    /// start-of-game reveal would be ranked by fog-cell order here and by
    /// the reveal's own order there. Captures so far carry at most one
    /// such good a leader, so the difference has never been observable;
    /// `docs/MERCHANT.md` §7 keeps it as an open question rather than a
    /// claim.
    pub fn seed_new_rares_from_fog(&mut self) {
        if !self.world.has_fog() {
            return;
        }
        let (fw, fh) = (self.world.fog_xs(), self.world.fog_ys());
        for fy in 0..fh {
            for fx in 0..fw {
                let Some(bits) = self.world.seen2(fx, fy) else {
                    continue;
                };
                if bits == 0 {
                    continue;
                }
                for who in 0..self.territory.players.len().min(8) {
                    if bits >> who & 1 != 0 {
                        self.reveal_fog_goods(fx, fy, who as Player);
                    }
                }
            }
        }
    }

    /// `Leader::new_rare@006d9e70`: record a seen good on this leader and
    /// on every ally that can still be told about it.
    ///
    /// The four gates, in the original's order:
    ///
    /// * the **caller** is not a plain human — `leader_flags & 0xc != 4`,
    ///   so a human's own reveals record nothing unless the AI is driving
    ///   it (`& 8`, never set here);
    /// * the recipient is in the game, is this leader or a mutual ally, and
    ///   is not a human (`leader_flags & 4`) unless the caller is that
    ///   AI-driven human;
    /// * the good is not already in the list — a linear scan, which is why
    ///   the list stays short and ordered;
    /// * `type_avail(good, strict)` is non-zero, and the good is neither
    ///   `FISH` nor `WHALES`. Those two are the **fisherman's** rares
    ///   (`LeaderData::calc_rare` pays them to a boat, not to a merchant),
    ///   so a coast full of fish never buys a merchant.
    pub(crate) fn new_rare(&mut self, who: Player, gi: usize) {
        if self.cities.nation[who as usize].human {
            return;
        }
        let Some(ty) = self.world.goods().get(gi).map(|g| g.ty) else {
            return;
        };
        if self.tables.tech_tree.is(ty, economy::FISH, false)
            || self.tables.tech_tree.is(ty, economy::WHALES, false)
        {
            return;
        }
        for w in 0..self.territory.players.len() {
            if self.leaders.defeated[w] || self.cities.nation[w].human {
                continue;
            }
            if w != who as usize
                && !(self.leaders.allied[who as usize][w] && self.leaders.allied[w][who as usize])
            {
                continue;
            }
            if self.ai.ai[w].new_rares.contains(&gi) {
                continue;
            }
            if self.type_avail(w as Player, ty) == crate::tech::NOT_AVAILABLE {
                continue;
            }
            self.ai.ai[w].new_rares.push(gi);
        }
    }

    /// `Leader::calc_unit_stats@006cf970`, its line of sight and speed: every
    /// one of the player's live units runs `update_los` (vtable `+0x160`,
    /// before the listing's `+0xe8` test) and takes its cached speed again.
    pub(crate) fn calc_unit_stats(&mut self, who: Player) {
        for u in 0..self.units.len() {
            let unit = &self.units[u];
            if unit.owner != who || !unit.alive() {
                continue;
            }
            let Some(ty) = unit.ty else { continue };
            self.update_los(u);
            // The listing's `+0xe8` test is `is_captain`, and behind it come
            // `update_hits(0)` (`+0x15c`), `update_speed` and `update_armor`:
            // a captain keeps its damage and takes the new pool — The
            // Despot's 659 is 109 once `age` has rescaled the upgrade
            // prerequisites (item 2448, `docs/GOLDEN.md` §89, run859).
            if self.is_captain(u) {
                let hits = self.own_hits(u, ty);
                let damage = self.units[u].max_health - self.units[u].health;
                self.units[u].max_health = hits;
                self.units[u].health = hits - damage;
            }
            self.units[u].movement.speed = self.own_speed(u, ty);
        }
    }
}

/// A ladder of `n` rows asked from the top (`held(n − 1)` first) — `LeaderData::get_granary@006db340` and its
/// siblings, `CityData::lumber_level@00736820` — answering `i + 1` at the first row held and 0 past the last;
/// no row under the first held is asked (item 2090's sweep holds the asks).
pub fn top_held(n: usize, mut held: impl FnMut(usize) -> bool) -> usize {
    (0..n).rev().find(|&i| held(i)).map_or(0, |i| i + 1)
}

/// A good's bit in a `rare` mask by **type index** — `has_rare`'s test on
/// the mask alone, so a caller that reads the leader's two words (`rare`
/// and `rare_conquest`, `+0x6da4` and `+0x6dcc`) asks the same bit
/// (`Caravan::trade_value@0073d9d0`'s sweep, item 2368).
pub(crate) fn rare_bit(rare: u64, good: usize) -> bool {
    good >= economy::BASE_RARE
        && good - economy::BASE_RARE < economy::RARES
        && rare >> (good - economy::BASE_RARE) & 1 != 0
}

/// What `World::reveal_fog@006b3d30` reads and does, one method a read or a
/// call, so [`reveal_fog_steps`] is the original's body and a view is the state
/// it runs over — [`SimRevealFog`] here, the emulated original's windows in
/// the sweep (`sweep/world_reveal_fog.rs`, item 3337).
pub trait RevealFogView {
    /// The tile mask at `(2fx + 1, 2fy + 1)` carries `0x200`
    /// ([`crate::world::tile::AS_BUILDING`]) — the gate the original reads
    /// twice, through `GameAccess::world` and `GameAccessConst::worldc`, which
    /// are one world.
    fn tile_marked(&mut self, fx: i32, fy: i32) -> bool;
    /// `ObjectsData::find_good_at(qx, qy, who, 1, 0)`, the index form: a
    /// goods-list index, or `< 0` for none.
    fn find_good_at(&mut self, qx: i32, qy: i32, who: i32) -> i32;
    /// `goods[gi]`'s `flags & 1`.
    fn good_alive(&mut self, gi: i32) -> bool;
    /// `Leader::new_rare@006d9e70` on `who`'s leader.
    fn new_rare(&mut self, who: i32, gi: i32);
    /// `MiscAccess::console->who`.
    fn console_who(&mut self) -> i32;
    /// `GameAccess::game->frame != 0`.
    fn game_started(&mut self) -> bool;
    /// `goods[gi]`'s `ever_seen` (`+0x20`).
    fn good_ever_seen(&mut self, gi: i32) -> u8;
    /// The leader's byte at `+0x6929`, the mask `ever_seen` is tested against.
    fn leader_seen_mask(&mut self, who: i32) -> u8;
    /// `goods[gi]`'s type index (`*(*(good + 0x18) + 4)`).
    fn good_type(&mut self, gi: i32) -> i32;
    /// The game's `reveal_map` setting (`game + 0x30`, a byte).
    fn reveal_map(&mut self) -> u8;
    /// The leader's `leader_flags` (`+0x0`).
    fn leader_flags(&mut self, who: i32) -> u32;
    /// The leader's short at `+0x59e4`.
    fn leader_w59e4(&mut self, who: i32) -> u16;
    /// The world cell's owner byte (`+0xf`, signed) at world cell `(cx, cy)`.
    fn cell_owner(&mut self, cx: i32, cy: i32) -> i32;
    /// `LeaderData::type_avail(ty, 1) != 0` on `who`'s leader.
    fn type_avail(&mut self, who: i32, ty: i32) -> bool;
    /// The "found a rare resource" message: the string, the sound, the
    /// leader's neon colour and `MessageWin::add_message` at the fog cell's
    /// world position `(wx, wy)`.
    fn found_rare_message(&mut self, who: i32, ty: i32, wx: i32, wy: i32);
    /// `goods[gi].ever_seen |= 1 << who` (a byte).
    fn mark_good_seen(&mut self, gi: i32, who: i32);
    /// The world cell's `flags` short (`+0x0`).
    fn cell_flags(&mut self, cx: i32, cy: i32) -> u16;
    /// The world cell's signed byte at `+0x2`.
    fn cell_kind(&mut self, cx: i32, cy: i32) -> i32;
    /// `objects.count` — the goods list's length.
    fn goods_len(&mut self) -> i32;
    /// Whether good `i` is alive, and the world cell it stands on, each axis
    /// through `div_3_table`.
    fn good_cell(&mut self, i: i32) -> (i32, i32);
    /// `who`'s leader's `oil_patches` (`ArrayBase<int>`) count.
    fn oil_count(&mut self, who: i32) -> i32;
    /// The `k`-th of them.
    fn oil_at(&mut self, who: i32, k: i32) -> i32;
    /// `ArrayBase<int>::add` on `who`'s `oil_patches`.
    fn oil_add(&mut self, who: i32, gi: i32);
    /// `ObjectsData::find_goody_at` on world cell `(cx, cy)`: an items-list
    /// index, or `< 0`.
    fn find_goody_at(&mut self, cx: i32, cy: i32) -> i32;
    /// `items[gi].ever_seen |= 1 << who` (a byte).
    fn mark_item_seen(&mut self, gi: i32, who: i32);
    /// `who`'s `objects[unit]` virtual slot `+0x18` answered non-zero.
    fn unit_is_unit(&mut self, who: i32, unit: i32) -> bool;
    /// `who`'s `units[unit]`'s `unit_masks` (`+0x68`).
    fn unit_masks(&mut self, who: i32, unit: i32) -> u32;
    /// `Unit::get_goody_box@005f7690` on that unit, handed world cell `(cx, cy)`.
    fn get_goody_box(&mut self, who: i32, unit: i32, cx: i32, cy: i32);
}

/// `World::reveal_fog@006b3d30(fx, fy, who, unit)`: a **fog** cell is revealed
/// to `who`, and `unit` is the revealing unit's slot or `< 0`.
///
/// Four arms in the original's order: the rare (tile mark, then the cell's
/// good — met by `new_rare`, announced to the console's human, marked seen),
/// the oil patch (the cell's `0x800` flag, then the goods list's first live
/// good standing on the cell, added to the leader's `oil_patches` once), the
/// goody (a cell whose `flags` sign is set: `find_goody_at`'s item marked
/// seen), and the revealing unit's goody box (`Unit::get_goody_box` for a
/// unit with `unit_masks & 0x100`). The last three read the world cell
/// `(fx >> 1, fy >> 1)`; **a cell whose `flags` sign is clear returns before
/// the goody arm and the box arm**, so the box arm is reached only on a goody
/// cell.
pub fn reveal_fog_steps<V: RevealFogView>(v: &mut V, fx: i32, fy: i32, who: i32, unit: i32) {
    let (cx, cy) = (fx >> 1, fy >> 1);
    if v.tile_marked(fx, fy) {
        let gi = v.find_good_at((2 * fx + 1) >> 2, (2 * fy + 1) >> 2, who);
        if gi >= 0 && v.good_alive(gi) {
            v.new_rare(who, gi);
            // The message is the console player's alone, once the game runs,
            // for a good its leader has not yet seen, that is not the base
            // rare (type 6), under `reveal_map < 3`, to a leader without
            // `flags & 0x800` or the `+0x59e4` short, not standing on its own
            // ground under the territory-reveal bonus (`flags & 0x2000`), and
            // that could build with the good (`type_avail`).
            if who == v.console_who()
                && v.game_started()
                && v.good_ever_seen(gi) & v.leader_seen_mask(who) == 0
            {
                let ty = v.good_type(gi);
                if ty != 6 && v.reveal_map() < 3 {
                    let flags = v.leader_flags(who);
                    if flags & 0x800 == 0
                        && v.leader_w59e4(who) == 0
                        && (flags & 0x2000 == 0 || v.cell_owner(cx, cy) != who)
                        && ty >= 0
                        && v.type_avail(who, ty)
                    {
                        v.found_rare_message(who, ty, fx * 0x180 + 0xc0, fy * 0x180 + 0xc0);
                    }
                }
            }
            v.mark_good_seen(gi, who);
        }
    }
    if v.cell_flags(cx, cy) & 0x800 != 0 {
        let n = v.goods_len();
        let mut i = 0;
        while i < n {
            if v.good_alive(i) && v.good_cell(i) == (cx, cy) {
                let count = v.oil_count(who);
                let mut k = 0;
                while k < count {
                    if v.oil_at(who, k) == i {
                        break;
                    }
                    k += 1;
                }
                if k >= count {
                    v.oil_add(who, i);
                }
                break;
            }
            i += 1;
        }
    }
    let flags = v.cell_flags(cx, cy);
    if flags & 0x8000 == 0 {
        return;
    }
    if flags & 0x100 != 0 || !matches!(v.cell_kind(cx, cy), 1 | 2) {
        let gi = v.find_goody_at(cx, cy);
        if gi >= 0 {
            v.mark_item_seen(gi, who);
        }
    }
    if unit >= 0 && v.unit_is_unit(who, unit) && v.unit_masks(who, unit) & 0x100 != 0 {
        v.get_goody_box(who, unit, cx, cy);
    }
}

/// [`RevealFogView`] over a [`Sim`]. What this crate does not carry answers
/// "nothing there": no message window (`found_rare_message`) and no unit
/// slots (`unit_is_unit`: nothing sets `unit_masks & 0x100`, `goody.rs`).
/// **The console answers `-1`**, so the message gate fails before it asks
/// anything else. The items chain is the goody boxes'
/// ([`Sim::goody_items`], item 3335) when `goody` is set; the harness's
/// `seen2` replay clears it and the goody arm finds nothing.
struct SimRevealFog<'a> {
    sim: &'a mut Sim,
    goody: bool,
}

impl RevealFogView for SimRevealFog<'_> {
    fn tile_marked(&mut self, fx: i32, fy: i32) -> bool {
        let t = crate::Pos::new(2 * fx + 1, 2 * fy + 1);
        self.sim.world.tile_mask(t) & crate::world::tile::AS_BUILDING != 0
    }
    fn find_good_at(&mut self, qx: i32, qy: i32, _who: i32) -> i32 {
        // The index form answers before `type_avail`; the non-`OIL` test is
        // the one [`Sim::find_good_at`] makes.
        match self.sim.world.good_at(crate::world::Cell::new(qx, qy)) {
            Some((gi, g)) if g.ty != crate::world::OIL_GOOD => gi as i32,
            _ => -1,
        }
    }
    fn good_alive(&mut self, gi: i32) -> bool {
        self.sim.world.goods()[gi as usize].alive
    }
    fn new_rare(&mut self, who: i32, gi: i32) {
        self.sim.new_rare(who as Player, gi as usize);
    }
    fn console_who(&mut self) -> i32 {
        -1
    }
    fn game_started(&mut self) -> bool {
        false
    }
    fn good_ever_seen(&mut self, _gi: i32) -> u8 {
        0
    }
    fn leader_seen_mask(&mut self, _who: i32) -> u8 {
        0
    }
    fn good_type(&mut self, _gi: i32) -> i32 {
        -1
    }
    fn reveal_map(&mut self) -> u8 {
        0
    }
    fn leader_flags(&mut self, _who: i32) -> u32 {
        0
    }
    fn leader_w59e4(&mut self, _who: i32) -> u16 {
        0
    }
    fn cell_owner(&mut self, _cx: i32, _cy: i32) -> i32 {
        -1
    }
    fn type_avail(&mut self, _who: i32, _ty: i32) -> bool {
        false
    }
    fn found_rare_message(&mut self, _who: i32, _ty: i32, _wx: i32, _wy: i32) {}
    fn mark_good_seen(&mut self, gi: i32, who: i32) {
        self.sim.mark_good_seen(gi as usize, who as Player);
    }
    fn cell_flags(&mut self, cx: i32, cy: i32) -> u16 {
        self.sim
            .world
            .cell_data(crate::world::Cell::new(cx, cy))
            .flags
    }
    fn cell_kind(&mut self, cx: i32, cy: i32) -> i32 {
        i32::from(
            self.sim
                .world
                .cell_data(crate::world::Cell::new(cx, cy))
                .land,
        )
    }
    fn goods_len(&mut self) -> i32 {
        self.sim.world.goods().len() as i32
    }
    fn good_cell(&mut self, i: i32) -> (i32, i32) {
        let c = self.sim.world.goods()[i as usize].pos.cell();
        (c.x, c.y)
    }
    fn oil_count(&mut self, who: i32) -> i32 {
        self.sim.ai.ai[who as usize].oil_patches.len() as i32
    }
    fn oil_at(&mut self, who: i32, k: i32) -> i32 {
        self.sim.ai.ai[who as usize].oil_patches[k as usize] as i32
    }
    fn oil_add(&mut self, who: i32, gi: i32) {
        self.sim.ai.ai[who as usize].oil_patches.push(gi as usize);
    }
    fn find_goody_at(&mut self, cx: i32, cy: i32) -> i32 {
        let c = crate::world::Cell::new(cx, cy);
        if !self.goody || !self.sim.world.contains(c) {
            return -1;
        }
        self.sim.goody_item_at(c).map_or(-1, |i| i as i32)
    }
    fn mark_item_seen(&mut self, gi: i32, who: i32) {
        if (0..8).contains(&who) {
            self.sim.gaia.goody_items[gi as usize].ever_seen |= 1 << who;
        }
    }
    fn unit_is_unit(&mut self, _who: i32, _unit: i32) -> bool {
        false
    }
    fn unit_masks(&mut self, _who: i32, _unit: i32) -> u32 {
        0
    }
    fn get_goody_box(&mut self, _who: i32, _unit: i32, _cx: i32, _cy: i32) {}
}

#[cfg(test)]
mod tests;
