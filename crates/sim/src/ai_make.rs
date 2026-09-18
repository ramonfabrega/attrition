//! Spending from the make list — `Leader::make_stuff@006c8af0`,
//! `Leader::make_this@006c94f0`, the market (`Leader::use_market@006c91c0`,
//! `Leader::market_speculation@006c8110`) and the orphan check
//! (`Leader::check_orphaned_buildings@006c9f20`); `docs/AI.md` §2.6, §2.15,
//! §2.8.
//!
//! Four places where the decompile and `docs/AI.md` part company are noted
//! inline as `CORRECTION`; the seams the simulation cannot yet reach are
//! `SEAM`.

use crate::ai::{MAKE_SLOTS, MakeObject};
use crate::build::{self, BuildDomain, Ident, flags};
use crate::economy::{RESOURCES, Resource};
use crate::orders::{Body, Worker, index};
use crate::tech::{self, Kind, Line, TypeId};
use crate::{Player, Sim};

/// The two expiry walks' draw sites, under the original's own offsets —
/// step 4's over the head's type and step 6's over a bought slot's
/// (`docs/AI.md` §2.6). They share a body here and two addresses there, so
/// the mark is what tells them apart in the trace's sequence.
pub const SITE_EXPIRE_HEAD: &str = "Leader::make_stuff+0x221";
pub const SITE_EXPIRE_SLOT: &str = "Leader::make_stuff+0x63d";

/// `use_market`'s one draw — the offset the sell rotation starts from
/// (`docs/AI.md` §2.15, §40). It is the only `game_random` step in the whole
/// market, `use_market@006c91c0+0x1ed`, and the first one Great Lakes takes
/// is the frame item 348 was about.
pub const SITE_MARKET_SELL: &str = "Leader::use_market+0x1ed";

/// The Commerce level `has_preq(BUY_SELL)` asks for. `BUY_SELL` is
/// `TypeIndex` 685, the second entry of `rules.xml`'s `<TECHBONUSES>`
/// ("Can buy and sell resources at the Market"), and its `PREQ0` is
/// **Coinage** — the Commerce line's second library tech
/// (`techrules.xml`, `WHERE Library`, `GRID_X 1`, `GRID_Y 2`, `AGE 1`;
/// the line is Barter, Coinage, Trade, Mercantilism, Finance, Assembly
/// Line, Globalization). `has_preq`'s generic arm walks the type's own
/// prerequisites and asks `has_tech` of each, and `BUY_SELL` has the one.
const BUY_SELL_COMMERCE_LEVEL: i32 = 2;

/// Which good a gather building gathers — `make_stuff`'s switch on the build
/// type's own `TypeIndex` (`0x1a1` food, `0x1a2` wood, `0x1a3` metal, `0x1a4`
/// knowledge, `0x1a5`/`0x1a6` oil). An exact type test, not a lineage one, so
/// the identity is what answers it.
const fn gather_good(id: Ident) -> Option<usize> {
    match id {
        Ident::Farm => Some(Resource::Food as usize),
        Ident::Woodcutter => Some(Resource::Timber as usize),
        Ident::Mine => Some(Resource::Metal as usize),
        Ident::University => Some(Resource::Knowledge as usize),
        Ident::OilWell | Ident::OilPlatform => Some(Resource::Oil as usize),
        _ => None,
    }
}

impl Sim {
    // ------------------------------------------------------------------
    // The type-level predicates `make_stuff` and `make_this` ask for
    // ------------------------------------------------------------------

    /// `Type` vslot `+0x64` (`docs/AI.md` §2.10): `is_city` on a build type,
    /// an ICF-folded `return 0` on every other table — `Type::vftable +0x64`
    /// is `Window::get_button`, the fold's canonical name.
    fn slot64(&self, t: TypeId) -> bool {
        self.build_record(t)
            .is_some_and(|r| build::is_city(&self.build_types, r))
    }

    /// `TypeData::is(role, strict)` for one of the roles the tree names.
    fn type_is_role(&self, t: TypeId, role: Option<TypeId>, strict: bool) -> bool {
        role.is_some_and(|x| self.tech_tree.is(t, x, strict))
    }

    /// `TypeData::is_peasant`: exactly `PEASANTS` (`0x32`) or
    /// `PEASANTSKOREAN` (`0x33`) — which is what the loader calls
    /// [`Worker::Citizen`].
    fn is_peasant_type(&self, t: TypeId) -> bool {
        self.unit_record(t)
            .is_some_and(|r| self.unit_types[r].worker == Worker::Citizen)
    }

    fn is_wonder_type(&self, t: TypeId) -> bool {
        matches!(self.tech_tree.kind(t), Kind::Building { wonder: true, .. })
    }

    /// The expiry verdict for one type: `true` when every slot holding it is
    /// cleared outright, `false` when each is cleared on a one-in-three draw.
    ///
    /// The head (`head = true`) has two extra clauses the other slots do not.
    ///
    /// CORRECTION to `docs/AI.md` §2.6 step 4, which has the second pair
    /// inverted. `make_stuff@006c8af0:93–97` reads
    /// `if (!is(UNIVERSITY,1) && (!is_unit_type() || is_peasant())) goto
    /// <probabilistic>` — so the *unconditional* case is `is(UNIVERSITY, 1)`
    /// **or a unit type that is not a peasant**, i.e. exactly a military
    /// unit, not "not a military unit type".
    ///
    /// Public so the harness can replay a dumped `make_stuff` frame slot
    /// for slot (`rondata::diff`'s make-list window test).
    pub fn expire_all(&self, t: TypeId, head: bool) -> bool {
        let r = &self.tech_tree.roles;
        if self.type_is_role(t, r.tower, false)
            || self.type_is_role(t, r.fortx, false)
            || self.slot64(t)
            || self.is_wonder_type(t)
        {
            return true;
        }
        if !head {
            return false;
        }
        if self.type_is_role(t, r.university, true) {
            return true;
        }
        self.tech_tree.kind(t).is_unit() && !self.is_peasant_type(t)
    }

    /// `get_cost(t, g, who, o, city, 0, 1, −1)` over the six goods.
    ///
    /// SEAM: [`Sim::type_price`] has no object or city form, so the `o` and
    /// `city` arguments the original threads through are dropped. Nothing in
    /// the modelled price path reads them.
    fn make_cost(&self, who: Player, t: i32) -> [i32; RESOURCES] {
        usize::try_from(t)
            .ok()
            .filter(|&t| t < self.tech_tree.types.len())
            .and_then(|t| self.type_price(who, t))
            .unwrap_or([0; RESOURCES])
    }

    /// `Leader::can_pay(slot)@006c9b90`: `can_pay_cost(who, city, o, escrow)
    /// >= num`.
    fn can_pay_slot(&self, who: Player, m: &MakeObject) -> bool {
        let Ok(t) = usize::try_from(m.t) else {
            return false;
        };
        if t >= self.tech_tree.types.len() {
            return false;
        }
        m.num <= self.type_affordable(who, t, m.escrow != 0)
    }

    /// `type_avail(g, 1) != 0` on one of the six good types. The simulation
    /// carries a good's availability on [`crate::economy::Holdings`], which is
    /// what every other cost path reads.
    fn good_avail(&self, who: Player, g: usize) -> bool {
        self.holdings[who as usize].available[g]
    }

    // ------------------------------------------------------------------
    // `make_stuff`
    // ------------------------------------------------------------------

    /// `make_stuff`: buy what the list and the stockpile allow; `true` when
    /// the head was bought (the original's 1). Draws from the sync stream
    /// per expiring duplicate (§2.6 step 4).
    pub fn make_stuff(&mut self, who: Player) -> bool {
        let w = who as usize;
        let head = *self.ai[w].make_list.head();
        // The head's `t` is compared against `−1` exactly, and its `val`
        // against zero.
        if head.t == -1 || head.val == 0 {
            return false;
        }
        self.use_market(who);
        let paid = self.can_pay_slot(who, &head);
        let cost = self.make_cost(who, head.t);
        let head_t = usize::try_from(head.t).unwrap_or(usize::MAX);
        let valid_head = head_t < self.tech_tree.types.len();

        // `saving` is the original's `local_14 == 0` after the knowledge
        // override: a leader short of knowledge alone does not save up.
        let mut saving = !paid;
        let mut bought_head = false;
        if paid {
            bought_head = self.make_this(who, 0);
        } else if self.ledgers[w].bucket[Resource::Knowledge as usize]
            < cost[Resource::Knowledge as usize]
        {
            saving = false;
        }

        // `local_1c`, the "something in this pass was a city-kind" flag. The
        // two `+0x64` calls at :73 and :78 are one `paid || is_city(head)`:
        // the paid branch sets it when the predicate is false, the fall-
        // through when it is true.
        let mut any_city_kind = paid || (valid_head && self.slot64(head_t));

        // Step 4: the head and every duplicate of its type expire.
        let unconditional = valid_head && self.expire_all(head_t, true);
        self.expire(who, head.t, 0, unconditional);

        // Step 5: the mean shortfall over the goods the head actually costs.
        let mut need = 0;
        if saving {
            let mut n = 0;
            for (g, &c) in cost.iter().enumerate() {
                if self.good_avail(who, g) && c != 0 {
                    n += 1;
                    need += c - self.ledgers[w].bucket[g];
                }
            }
            if n != 0 {
                need /= n;
            }
            if need < 0 {
                need = 0;
            }
        }

        // Step 6: slots 1..10.
        for slot in 1..MAKE_SLOTS {
            let m = self.ai[w].make_list.list[slot];
            if m.t < 0 || m.val == 0 {
                continue;
            }
            let Ok(t) = usize::try_from(m.t) else {
                continue;
            };
            if t >= self.tech_tree.types.len() {
                continue;
            }
            // From slot 4 on, a slot that repeats a `(t, city)` already in
            // the ranked four is dropped.
            //
            // CORRECTION to `docs/AI.md` §2.6 step 6's "three duplicates are
            // tolerated": `make_stuff@006c8af0:147–167` tests all four of
            // slots 0..3, and the flag it keeps (`bVar3`) is cleared by any
            // of them. There is no tolerance — the four ranked slots simply
            // are not themselves subject to the test.
            if slot > 3
                && (0..4).any(|k| {
                    let h = self.ai[w].make_list.list[k];
                    h.t == m.t && h.city == m.city
                })
            {
                continue;
            }
            let slot_cost = self.make_cost(who, m.t);
            let mut buy = true;
            for g in 0..RESOURCES {
                // Both the head's cost and this slot's must be non-zero in
                // `g` before the good is examined at all.
                if !self.good_avail(who, g) || cost[g] == 0 || slot_cost[g] == 0 {
                    continue;
                }
                if self.ledgers[w].bucket[g] >= cost[g] + slot_cost[g] + need {
                    continue;
                }
                buy = false;
                if slot == 5 {
                    // The unit slot: buy anyway with no workers at all.
                    let c = &self.ai[w].census;
                    buy = c.free_peasants == 0 && c.gatherers == 0;
                } else if slot == 4 {
                    buy = self.gather_exception(who, t);
                }
                break;
            }
            if !buy {
                continue;
            }
            if !self.can_pay_slot(who, &m) {
                continue;
            }
            if self.slot64(t) {
                any_city_kind = true;
            }
            // The original ignores `make_this`'s answer here.
            self.make_this(who, slot);
            let unc = self.expire_all(t, false);
            // The slot loop's expiry starts at the slot itself, not at zero.
            self.expire(who, m.t, slot, unc);
        }

        if !any_city_kind {
            self.ai[w].site_mark = self.ai[w].site_mark.wrapping_add(1);
        }
        bought_head
    }

    /// The expiry walk: from `from` to the end of the list, one
    /// `Random::get(0, 0xffff)` per slot holding `t`, cleared on `% 3 == 0`
    /// or when `unconditional`. The draw is taken before the flag is
    /// consulted, so the flag never saves a draw.
    ///
    /// A cleared slot is `t = −1` and nothing else (`make_stuff@006c8af0:112`
    /// and `:254` write one word): its `val`, `city`, `cat` and the rest
    /// stay, and a `LEADERS=9` dump shows them standing — run19's dump-frame
    /// 8183 has slot 4 at `t −1 val 9999999`. `MakeList::clear` is the one
    /// that resets the whole record.
    pub fn expire(&mut self, who: Player, t: i32, from: usize, unconditional: bool) {
        let w = who as usize;
        let site = if from == 0 {
            SITE_EXPIRE_HEAD
        } else {
            SITE_EXPIRE_SLOT
        };
        for k in from..MAKE_SLOTS {
            if self.ai[w].make_list.list[k].t != t {
                continue;
            }
            self.mark(site);
            let r = self.rng.get(0, 0xffff);
            if r % 3 == 0 || unconditional {
                self.ai[w].make_list.list[k].t = -1;
            }
        }
    }

    /// Slot 4's exception: a gather building for a good whose `econ` says the
    /// rate is under `hi`.
    fn gather_exception(&self, who: Player, t: TypeId) -> bool {
        let Some(rec) = self.build_record(t) else {
            return false;
        };
        if !self.build_types[rec].has(flags::GATHER) {
            return false;
        }
        let Some(g) = gather_good(self.build_types[rec].ident) else {
            return false;
        };
        self.good_avail(who, g) && self.ai[who as usize].econ[g] & 4 != 0
    }

    // ------------------------------------------------------------------
    // `make_this`
    // ------------------------------------------------------------------

    /// `make_this(slot)`: dispatch one slot to its producer; `true` when it
    /// bought.
    pub fn make_this(&mut self, who: Player, slot: usize) -> bool {
        let w = who as usize;
        let m = self.ai[w].make_list.list[slot];
        let Ok(t) = usize::try_from(m.t) else {
            return false;
        };
        if t >= self.tech_tree.types.len() {
            return false;
        }
        let city = usize::try_from(m.city).ok();
        let kind = self.tech_tree.kind(t);
        let bought = if kind.is_building() {
            self.make_build_type(who, t, &m, city)
        } else if kind.is_tech() {
            self.produce_tech(who, t, m.escrow)
        } else if kind.is_unit() {
            if self.type_avail(who, t) == tech::AVAILABLE {
                self.produce_unit(who, t, city, m.num, m.escrow)
            } else {
                self.produce_tech(who, t, m.escrow)
            }
        } else {
            // SEAM: the spell band (`0x275..0x2ab`) has no `Kind` in the
            // tree, so `produce_spell` is unreachable. Everything else — a
            // good in the make list — is the original's bare `return 1`.
            false
        };
        if bought {
            self.ai[w].make_list.list[slot].val /= 100;
        }
        bought
    }

    /// `make_this`'s build-type arm: the tech that unlocks it, an upgrade, a
    /// city, or a building.
    fn make_build_type(
        &mut self,
        who: Player,
        t: TypeId,
        m: &MakeObject,
        city: Option<usize>,
    ) -> bool {
        let w = who as usize;
        if self.type_avail(who, t) != tech::AVAILABLE {
            // Not buildable yet: what gets bought is the research.
            return self.produce_tech(who, t, m.escrow);
        }
        if m.up != 0 {
            return self.produce_upgrade(who, t, city, m.escrow);
        }
        // CORRECTION to `docs/AI.md` §2.6, which has this arm as "`up != 0`
        // → `produce_upgrade`" and nothing else. There is a second, wholly
        // separate test at `make_this@006c94f0:88`, whose arguments Ghidra
        // drops: the listing at `0x6c957a` is `push 0; push 0x19f; call
        // [eax+0x60]` — `is(TOWN, 0)`. A Large City or a Major City is an
        // *upgrade* of the city it stands on, so it must be caught here;
        // without it both are `is_city` and would fall through to
        // `produce_city`, which would try to found a new one.
        if self.type_is_role(t, self.tech_tree.roles.town, false) {
            return self.produce_upgrade(who, t, city, m.escrow);
        }
        if self.slot64(t) {
            // A city site: `wx, wy` are world cells.
            let reg = self.world.region_of(crate::Cell::new(m.wx, m.wy));
            if !self.region_has_workers(who, reg) {
                return false;
            }
            return self.produce_city(who, t, m.wx, m.wy, m.escrow);
        }
        // An ordinary building or a wonder, placed near a city of mine.
        let Some(c) = city.filter(|&c| c < self.cities.len() && self.cities[c].alive) else {
            return false;
        };
        let centre = self.cities[c].building;
        let reg = self
            .buildings
            .get(centre)
            .and_then(|b| self.world.region_of(b.pos.cell()));
        if !self.region_has_workers(who, reg) {
            return false;
        }
        let Some(rec) = self.build_record(t) else {
            return false;
        };
        let bt = &self.build_types[rec];
        // `domain == 0` (land) and no `e` flag: the city must have room.
        //
        // CORRECTION to `docs/AI.md` §2.6, which reads `build_flags & 0x10`
        // as "not a gather building". `make_this@006c94f0:121` is
        // `(*(byte *)(iVar8 + 0x2c0) & 0x10) == 0` and `0x10` is flag `e`,
        // `NO_CITY` — `GATHER` is `0x40`, which is what `make_stuff`'s
        // slot-4 exception reads (`:194`). The pair is easy to swap because
        // both come off `build_flags` at `+0x2c0`.
        if bt.domain() == BuildDomain::Land && !bt.has(flags::NO_CITY) {
            let big = bt.x_size.max(bt.y_size);
            let ai = self.ai[w].city_ai.get(c);
            if let Some(ai) = ai
                && ai.land - ai.filled < 2
            {
                let n = (big - 2).clamp(0, 3) as usize;
                // `space` is `uchar[3]` at `CityData+0x69` and `ter` is
                // `uchar[6]` at `+0x6c`, so the clamp's fourth rung — a
                // footprint of five or more — reads `ter[0]`, the best food
                // amount on the city's occupied tiles. Reproduced, overread
                // and all.
                let room = if n == 3 { ai.ter[0] } else { ai.space[n] };
                if room < 2 {
                    return false;
                }
            }
        }
        self.produce_building(who, rec, centre, Some(c))
    }

    /// `reg_free_peasants[r] != 0 || reg_gatherers[r] != 0` — the region test
    /// both of `make_this`'s placement arms make before spending.
    fn region_has_workers(&self, who: Player, reg: Option<u16>) -> bool {
        let Some(r) = reg.map(usize::from) else {
            return false;
        };
        let c = &self.ai[who as usize].census;
        c.reg_free_peasants.get(r).copied().unwrap_or(0) != 0
            || c.reg_gatherers.get(r).copied().unwrap_or(0) != 0
    }

    // ------------------------------------------------------------------
    // The market
    // ------------------------------------------------------------------

    /// `LeaderData::has_market`: an active market building of mine.
    fn has_market(&self, who: Player) -> bool {
        self.buildings.iter().any(|b| {
            b.alive
                && b.active
                && b.owner == who
                && b.ty
                    .is_some_and(|r| build::is(&self.build_types, r, Ident::Market))
        })
    }

    /// The gate both market functions open with: the ability (tribe bonus 4
    /// or `has_preq(BUY_SELL)`), a market, and no nuclear embargo.
    ///
    /// **CORRECTION (item 348).** The tech half used to be read off the
    /// market building — "owning an active market is taken to imply it,
    /// which is true of the shipped tree". It is not: the Market's own
    /// prerequisite is *Barter*, Commerce 1, and `BUY_SELL`'s is **Coinage**,
    /// Commerce 2 ([`BUY_SELL_COMMERCE_LEVEL`]). The two are a whole library
    /// tech apart, and on Great Lakes that gap is 2,199 frames wide — the
    /// old gate opened on 6383 and the original's first market draw is on
    /// 8582. `docs/AI.md` §40.
    ///
    /// SEAM: `get_nuke_embargo` is not modelled and is taken as zero.
    fn market_gate(&self, who: Player) -> bool {
        let w = who as usize;
        let bonus = self
            .tech_tree
            .has_tribe_bonus(&self.setup, &self.tech[w], 4);
        (bonus || self.can_buy_sell(who)) && self.has_market(who)
    }

    /// `LeaderData::has_preq(BUY_SELL)` — the bonus type's own prerequisite,
    /// Coinage, asked as the Commerce line's level.
    fn can_buy_sell(&self, who: Player) -> bool {
        self.tech[who as usize].epoch[Line::Commerce.index()] >= BUY_SELL_COMMERCE_LEVEL
    }

    /// One good's place in `use_market`'s sell rotation, which is the whole
    /// of what the rotation decides without a price: not knowledge, not
    /// wealth, not oil, not the good being covered, available, a stock that
    /// clears its own need by a hundred, and either an income at half the
    /// commerce cap or more than 199 in the bucket.
    ///
    /// The filter matters beyond which good is sold: the original's
    /// once-flag is cleared **by a candidate passing this test**, not by a
    /// sale going through, so "nothing here is sellable" is exactly the case
    /// where the outer loop spends one draw and stops.
    fn market_sellable(
        &self,
        who: Player,
        g2: usize,
        covering: usize,
        need: &[i32; RESOURCES],
    ) -> bool {
        if g2 == Resource::Knowledge as usize
            || g2 == Resource::Wealth as usize
            || g2 == Resource::Oil as usize
            || g2 == covering
            || !self.good_avail(who, g2)
        {
            return false;
        }
        let l = &self.ledgers[who as usize];
        let stock = l.bucket[g2];
        need[g2] <= stock - 100 && (l.cap[g2] / 2 <= l.income[g2] || stock > 199)
    }

    /// `use_market@006c91c0`: cover every short good the market can, one
    /// good at a time — buy it outright when wealth can stand the price,
    /// otherwise **sell** a hundred of something else and come round again.
    ///
    /// The draw is the sell branch's `Random::get(game_random, 0, 0xffff)
    /// % 6`, which is where the six-good rotation starts, and it is spent
    /// once per pass of the outer loop. The once-flag (`uVar5`) is what
    /// bounds those passes: it clears **when a candidate passes
    /// [`Sim::market_sellable`]**, not when a sale goes through, so a pass
    /// that finds nothing sellable is the last one.
    ///
    /// **One draw is not "nothing was sellable"** — item 358. It is also
    /// what a *successful* sale looks like when the sale covers the need,
    /// because the outer `while (bucket[g] < need[g])` then exits before
    /// the once-flag is ever consulted. `docs/AI.md` §40 read the single
    /// draws on Great Lakes 8582/8585/8782/8982 the first way, and the
    /// second is what run97's own `BUILDDATA` says: the original queues
    /// two more scholars at 8985 than this crate could pay for, and a
    /// timber sale at 8982 is where the wealth comes from
    /// (`docs/ECONOMY.md` §12).
    pub fn use_market(&mut self, who: Player) {
        let w = who as usize;
        // `need[g]`: the first `max(1, epoch[Commerce])` slots' prices.
        let commerce = self.tech[w].epoch[Line::Commerce.index()].max(1);
        let mut need = [0i32; RESOURCES];
        for slot in 0..(commerce as usize).min(MAKE_SLOTS) {
            let m = self.ai[w].make_list.list[slot];
            // `val > 0 && t > 0`, both strict.
            if m.val <= 0 || m.t <= 0 {
                continue;
            }
            let cost = self.make_cost(who, m.t);
            for (n, c) in need.iter_mut().zip(cost) {
                *n += c;
            }
        }
        if !self.market_gate(who) {
            return;
        }
        let wealth = Resource::Wealth as usize;
        for g in 0..RESOURCES {
            if g == Resource::Knowledge as usize || !self.good_avail(who, g) {
                continue;
            }
            if self.ledgers[w].bucket[g] >= need[g] {
                continue;
            }
            // The original's `do { if (uVar5 != 0) break; uVar5 = 1; …
            // } while (bucket[g] < need[g])`, once-flag and all.
            let mut once = false;
            loop {
                if once {
                    break;
                }
                once = true;
                // `(g == WEALTH) || (bucket[WEALTH] − buy) < need[WEALTH]`.
                // C's `||` short-circuits, so wealth never asks a price —
                // it cannot be bought with itself.
                let sell = g == wealth || {
                    let (buy, _) = self.calc_market_prices(who, g);
                    self.ledgers[w].bucket[wealth] - buy < need[wealth]
                };
                if sell {
                    self.mark(SITE_MARKET_SELL);
                    let start = usize::try_from(self.rng.get(0, 0xffff) % 6).unwrap_or(0);
                    for k in 0..RESOURCES {
                        let g2 = (k + start) % RESOURCES;
                        if !self.market_sellable(who, g2, g, &need) {
                            continue;
                        }
                        // The inner gate is the outer one again (the
                        // embargo arm aside, which is zero here); the
                        // once-flag clears whether or not it holds.
                        if self.market_gate(who) {
                            self.do_sell(who, g2);
                        }
                        once = false;
                    }
                } else {
                    if self.market_gate(who) {
                        self.do_buy(who, g);
                    }
                    once = false;
                }
                if self.ledgers[w].bucket[g] >= need[g] {
                    break;
                }
            }
        }
    }

    /// `LeaderData::calc_market_prices@006dc2a0` — one good's `(buy, sell)`.
    ///
    /// The sell price is the market's own price plus its flux; the buy
    /// price is **twice** the price plus the same flux. One bonus moves
    /// them apart in the seller's favour — the Nubians' if the nation has
    /// it, otherwise Amber — and it is added to the sell price and taken
    /// off the buy price, never both. Then three floors: a sell price of
    /// at least 1, a buy price of at least `MARKET_BASEMENT × 2`, and a
    /// buy price at least ten over the sell price.
    ///
    /// Two arms are **stated rather than modelled**, and neither is
    /// reachable by any capture on disk:
    ///
    /// - the Conquer-the-World per-leader market bonus, behind
    ///   `semaphore[2] & 2` — CtW is cut from v1 (`CLAUDE.md`);
    /// - the **Supercollider**'s clamp (`has_wonder(0x21d)`, building type
    ///   index 541), which pins the buy price under `SUPER_BUY` and the
    ///   sell price over `SUPER_SELL`. [`crate::tech::PlayerTech::wonders`]
    ///   has no writer in this crate at all, so the predicate could only be
    ///   written as a constant `false`; it is a seam instead.
    ///
    /// `RUSSIAN_COMMUNISM` ships as **0**, which switches its own arm off
    /// in the original too, so the flat 100/100 it would impose is dead
    /// data rather than a seam.
    pub fn calc_market_prices(&self, who: Player, g: usize) -> (i32, i32) {
        let w = who as usize;
        let price = self.market.price[g];
        let flux = self.market.flux[g];
        let mut sell = price + flux;
        let mut buy = price * 2 + flux;
        let nubian = self.tuning.nubian_market_prices;
        let bonus = if nubian != 0
            && self
                .tech_tree
                .has_tribe_bonus(&self.setup, &self.tech[w], 4)
        {
            nubian
        } else if self.has_rare(who, crate::economy::AMBER) {
            self.tuning.amber_market
        } else {
            0
        };
        sell += bonus;
        buy -= bonus;
        sell = sell.max(1);
        buy = buy.max(self.tuning.market_basement * 2);
        buy = buy.max(sell + 10);
        (buy, sell)
    }

    /// `Leader::do_sell@006cfc60` — a hundred of `g` for the sell price.
    ///
    /// `true` when the sale went through, which is the original's **0**;
    /// its `1` is the one refusal, a stock under a hundred. The sale
    /// releases a hundred of that good's escrow (floored at zero), pays
    /// the wealth in, and walks the market's own price **down** by
    /// `MARKET_SUPPLY_DEMAND`, floored at zero — the supply half of the
    /// price feedback `GameDaemon::calc_market`'s flux rides on top of.
    pub fn do_sell(&mut self, who: Player, g: usize) -> bool {
        let w = who as usize;
        let (_, sell) = self.calc_market_prices(who, g);
        if self.ledgers[w].bucket[g] < 100 {
            return false;
        }
        self.ledgers[w].bucket[g] -= 100;
        self.ledgers[w].escrow[g] = (self.ledgers[w].escrow[g] - 100).max(0);
        self.ledgers[w].bucket[Resource::Wealth as usize] += sell;
        self.market.price[g] = (self.market.price[g] - self.tuning.market_supply_demand).max(0);
        true
    }

    /// `Leader::do_buy@006cfbd0` — a hundred of `g` for the buy price.
    ///
    /// `true` when the purchase went through; the refusal is wealth under
    /// the price. The escrow released is **wealth's**, not the bought
    /// good's, and the price walks **up**, with no ceiling of its own.
    pub fn do_buy(&mut self, who: Player, g: usize) -> bool {
        let w = who as usize;
        let wealth = Resource::Wealth as usize;
        let (buy, _) = self.calc_market_prices(who, g);
        if self.ledgers[w].bucket[wealth] < buy {
            return false;
        }
        self.ledgers[w].bucket[wealth] -= buy;
        self.ledgers[w].escrow[wealth] = (self.ledgers[w].escrow[wealth] - 100).max(0);
        self.ledgers[w].bucket[g] += 100;
        self.market.price[g] += self.tuning.market_supply_demand;
        true
    }

    /// `market_speculation`: `production_ai_setup`'s last act.
    ///
    /// The escrow clamp and the tier are the whole of what survives without
    /// prices; the sell and buy passes are the same SEAM as [`Sim::use_market`]
    /// and the original takes no draw in either, so nothing is lost from the
    /// sync stream here.
    pub fn market_speculation(&mut self, who: Player) {
        let w = who as usize;
        if self.lobby.resources_unlimited() || !self.market_gate(who) {
            return;
        }
        let mut tier = 0;
        for g in 0..RESOURCES {
            if !self.good_avail(who, g) {
                continue;
            }
            if self.ledgers[w].escrow[g] > 4000 {
                self.ledgers[w].escrow[g] = 2000;
            }
            let stock = self.ledgers[w].bucket[g];
            if stock < 100 {
                tier = tier.max(2);
            } else if stock < 200 && tier == 0 {
                tier = 1;
            }
        }
        // SEAM: the sell pass (`bucket − escrow >= 2000 >> tier` and sell
        // price `>= 75 / (tier + 1)`) and the buy pass (§2.15) both need
        // `calc_market_prices`.
        let _ = tier;
    }

    // ------------------------------------------------------------------
    // The orphan check
    // ------------------------------------------------------------------

    /// `check_orphaned_buildings`: disband marked buildings and stranded
    /// sites, or send a builder.
    pub fn check_orphaned_buildings(&mut self, who: Player) {
        let w = who as usize;
        if self.nation[w].human {
            return;
        }
        // SEAM: the first arm — a building whose `build_masks & 1` (the
        // "sell me" mark) is set gets it cleared and, unless it is a city, is
        // disbanded when it is a tower / FORTX / AIRBASE, has an `attack`, or
        // is one of several of its type. `Building` carries no `build_masks`
        // and no writer of bit 0 was found, so the arm is unreachable and is
        // not modelled.
        for b in 0..self.buildings.len() {
            let bd = &self.buildings[b];
            if !bd.alive || bd.owner != who || bd.active {
                continue;
            }
            let site_reg = self.world.region_of(bd.pos.cell());
            // Is the site a land building? A water one may be built from
            // another region.
            let land = bd
                .ty
                .is_none_or(|r| self.build_types[r].domain() == BuildDomain::Land);
            if self.site_has_builder(who, b, site_reg, land) {
                continue;
            }
            // CORRECTION to `docs/AI.md` §2.8, which calls this "my territory
            // at the site". `check_orphaned_buildings@006c9f20:196–199` and
            // `:228–232` read `world+0x13c`, which the PDB names
            // `WorldData::danger[8]` (`int *[8]`), indexed `(y >> 9)/3 *
            // reg_xs + (x >> 9)/3` — the same grid `calc_cost` reads for its
            // danger term, though **not** the same shift: the world grid
            // eighths it and the AI takes it whole (`docs/DANGER.md` §5).
            // It is the danger map, not territory.
            let danger = self.world.danger_at(who, self.buildings[b].pos);
            if self.buildings[b].job_counter == 0 && danger > 0 {
                self.disband_building(b, false);
                continue;
            }
            if danger < 1 && self.recruit_builder(who, b, site_reg) {
                continue;
            }
            // SEAM: `build_masks |= 0x2000` — "no builder could be found" —
            // has nowhere to live. Note that a *started* site inside my own
            // borders also lands here: `danger >= 1` skips the recruit scan
            // entirely, so the original never re-sends a builder to it.
        }
    }

    /// The builder scan: `true` when this site should be left alone — either
    /// a builder is on it in the right region (or it is a water building), or
    /// one in the wrong region had its orders cleared.
    fn site_has_builder(
        &mut self,
        who: Player,
        b: usize,
        site_reg: Option<u16>,
        land: bool,
    ) -> bool {
        let mut cleared = false;
        for u in 0..self.units.len() {
            if !self.is_own_citizen(who, u) {
                continue;
            }
            let Some(i) = self.action_of(u) else { continue };
            let Some(order) = self.units[u].orders.get(i) else {
                continue;
            };
            let Body::Build(target) = order.body else {
                continue;
            };
            if target != b {
                continue;
            }
            let ureg = self.world.region_of(self.units[u].pos.cell());
            if ureg == site_reg || !land {
                return true;
            }
            self.clear_orders(u);
            cleared = true;
        }
        cleared
    }

    /// The recruit arm: the first idle or gathering citizen of mine in the
    /// site's region is swarmed onto it with `QUEUE_NEW`.
    fn recruit_builder(&mut self, who: Player, b: usize, site_reg: Option<u16>) -> bool {
        let mut pick = None;
        for u in 0..self.units.len() {
            if !self.is_own_citizen(who, u) {
                continue;
            }
            let kind = self
                .action_of(u)
                .and_then(|i| self.units[u].orders.get(i))
                .map_or(index::NONE, |o| o.index());
            if kind != index::NONE && kind != index::GATHER {
                continue;
            }
            if self.world.region_of(self.units[u].pos.cell()) != site_reg {
                continue;
            }
            pick = Some(u);
            break;
        }
        let Some(u) = pick else { return false };
        self.clear_orders(u);
        self.swarm_around(u, b, Body::Build(b), true);
        true
    }

    /// A live, on-map citizen of mine — the two scans' common filter.
    fn is_own_citizen(&self, who: Player, u: usize) -> bool {
        let unit = &self.units[u];
        unit.alive() && unit.on_map && unit.owner == who && self.worker_of(u) == Worker::Citizen
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::CityAi;
    use crate::build::BuildType;
    use crate::economy::Resource;
    use crate::tech::{TypeDef, UnitTraits};
    use crate::world::{Cell, Terrain, UNITS_PER_TILE};
    use crate::{Pos, Tuning, UnitType, World, cost};

    fn tile_pos(tx: i32, ty: i32) -> Pos {
        Pos::new(
            tx * UNITS_PER_TILE + UNITS_PER_TILE / 2,
            ty * UNITS_PER_TILE + UNITS_PER_TILE / 2,
        )
    }

    const COMBAT: UnitTraits = UnitTraits {
        free: false,
        jumpable: false,
        unique: false,
        hero: false,
        patriot: false,
        combat: false,
    };

    /// Everything the make list can hold, wired tree entry to record.
    struct Fx {
        sim: Sim,
        village: TypeId,
        tower: TypeId,
        wonder: TypeId,
        farm: TypeId,
        barracks: TypeId,
        soldier: TypeId,
        citizen: TypeId,
        tech: TypeId,
        town: TypeId,
        farm_rec: usize,
        village_rec: usize,
    }

    fn bt(ident: Ident, tree: TypeId, x: i32, y: i32, food: i32) -> BuildType {
        BuildType {
            ident,
            x_size: x,
            y_size: y,
            job_time: 100,
            hits: 100,
            tree: Some(tree),
            price: cost::Price::free().with_base(Resource::Food, food),
            ..BuildType::default()
        }
    }

    fn fx() -> Fx {
        let mut w = World::new(16, 16);
        w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(15, 15));
        let mut sim = Sim::new(Tuning::RON, w, 2);
        sim.nation[1].human = false;
        let t = &mut sim.tech_tree;
        for n in ["Food", "Timber", "Wealth", "Knowledge", "Metal", "Oil"] {
            t.add(TypeDef::good(n));
        }
        let village = t.add(TypeDef::building("Small City"));
        let town = t.add(TypeDef::building("Large City").from(village));
        let tower = t.add(TypeDef::building("Tower"));
        let wonder = t.add(TypeDef::new(
            "Colossus",
            Kind::Building {
                auto: false,
                wonder: true,
            },
        ));
        let farm = t.add(TypeDef::building("Farm"));
        let barracks = t.add(TypeDef::building("Barracks"));
        let university = t.add(TypeDef::building("University"));
        let soldier = t.add(TypeDef::unit("Hoplite", COMBAT));
        let citizen = t.add(TypeDef::unit("Citizen", COMBAT));
        let tech = t.add(TypeDef::plain("Fortification", -1));
        t.roles.tower = Some(tower);
        t.roles.university = Some(university);
        t.roles.town = Some(town);
        sim.tech = (0..2)
            .map(|_| crate::tech::PlayerTech::new(&sim.tech_tree))
            .collect();

        let village_rec = sim.add_build_type(bt(Ident::Village, village, 7, 7, 0));
        let mut town_bt = bt(Ident::Town, town, 7, 7, 0);
        town_bt.from = Some(village_rec);
        sim.add_build_type(town_bt);
        sim.add_build_type(bt(Ident::Tower, tower, 2, 2, 30));
        sim.add_build_type(bt(Ident::Wonder, wonder, 5, 5, 0));
        let mut f = bt(Ident::Farm, farm, 4, 4, 60);
        // A farm is a gather type *and* a flat one — `init_final_flags`
        // makes the FARM, OILWELL and OILPLATFORM lineages flat, and the
        // flat half is what keeps `blocked_location`'s gather tail off it
        // (`docs/CITIES.md` §2.6.7). Without it a farm on this bare test
        // world is refused for having nothing to gather.
        f.flags |= flags::GATHER | flags::FLAT;
        let farm_rec = sim.add_build_type(f);
        // The barracks costs two goods, so that `need`'s mean has something
        // to average.
        let mut b = bt(Ident::Barracks, barracks, 4, 4, 100);
        b.price = b.price.with_base(Resource::Timber, 100);
        sim.add_build_type(b);
        sim.add_build_type(bt(Ident::University, university, 5, 5, 0));
        sim.add_unit_type(UnitType {
            tree: Some(soldier),
            price: cost::Price::free().with_base(Resource::Food, 50),
            ..UnitType::default()
        });
        sim.add_unit_type(UnitType {
            tree: Some(citizen),
            worker: Worker::Citizen,
            price: cost::Price::free().with_base(Resource::Food, 20),
            ..UnitType::default()
        });
        Fx {
            sim,
            village,
            tower,
            wonder,
            farm,
            barracks,
            soldier,
            citizen,
            tech,
            town,
            farm_rec,
            village_rec,
        }
    }

    fn slot(t: TypeId, val: i32) -> MakeObject {
        MakeObject {
            t: t as i32,
            val,
            ..MakeObject::EMPTY
        }
    }

    /// How many `Random::get(0, 0xffff)` calls separate two seeds.
    fn draws(before: u32, after: u32) -> usize {
        let mut r = crate::combat::Rng::new(before);
        for n in 0..64 {
            if r.seed == after {
                return n;
            }
            r.get(0, 0xffff);
        }
        panic!("not reached within 64 draws");
    }

    /// Step 4 draws once for every slot holding the head's type, on every
    /// call, whether or not the head could be paid for.
    #[test]
    fn the_head_s_expiry_draws_once_per_matching_slot() {
        // Three slots hold the head's type. The two behind it carry `val =
        // 0`, so the slot loop passes over them — but the expiry walk keys
        // on the type alone and draws for all three.
        let mut f = fx();
        let s = &mut f.sim;
        s.ai[1].make_list.list[0] = slot(f.tech, 100);
        s.ai[1].make_list.list[3] = slot(f.tech, 0);
        s.ai[1].make_list.list[7] = slot(f.tech, 0);
        s.ledgers[1].bucket = [0; RESOURCES];
        let before = s.rng.seed;
        s.make_stuff(1);
        assert_eq!(draws(before, s.rng.seed), 3);

        // A tower head clears every one of them outright, and still takes
        // the three draws: the `Random::get` is evaluated before the flag.
        let mut f = fx();
        let s = &mut f.sim;
        s.ai[1].make_list.list[0] = slot(f.tower, 100);
        s.ai[1].make_list.list[3] = slot(f.tower, 0);
        s.ai[1].make_list.list[7] = slot(f.tower, 0);
        let before = s.rng.seed;
        s.make_stuff(1);
        assert_eq!(draws(before, s.rng.seed), 3);
        for k in [0, 3, 7] {
            assert_eq!(s.ai[1].make_list.list[k].t, -1, "slot {k} cleared");
        }
    }

    /// A plain tech head expires one slot in three; a tower, a city type, a
    /// wonder, a university and a military unit expire every duplicate.
    #[test]
    fn the_head_s_unconditional_expiry_is_the_decompile_s_five_cases() {
        let f = fx();
        let s = &f.sim;
        assert!(!s.expire_all(f.tech, true), "a plain tech is probabilistic");
        assert!(
            !s.expire_all(f.citizen, true),
            "a peasant is probabilistic — it is not a military unit"
        );
        assert!(s.expire_all(f.tower, true));
        assert!(s.expire_all(f.village, true), "a city type, through +0x64");
        assert!(s.expire_all(f.wonder, true));
        assert!(
            s.expire_all(f.soldier, true),
            "a unit type that is not a peasant — the clause docs/AI.md inverts"
        );
        // The non-head slots lose the last two clauses.
        assert!(!s.expire_all(f.soldier, false));
        assert!(s.expire_all(f.tower, false));
        assert!(s.expire_all(f.wonder, false));
    }

    /// **Run18b's five expiry draws, against the original's own seeds**
    /// (`docs/RUNS.md`, "The producers' run"; `docs/AI.md` §15). The trace
    /// records the seed before every `Random::get`, and the dump records
    /// which slots survived — so the two together settle the arm that
    /// `docs/AI.md` §2.6's prose gets backwards. An ordinary building (the
    /// original's Temple; a Barracks here) takes the **probabilistic** arm,
    /// and all five rolls decide the observed slot by `% 3 == 0`. Were the
    /// arm unconditional, four of these five slots would be empty.
    ///
    /// The seeds chain: each draw's successor is the next record's seed, and
    /// the last is the frame's following draw at another site.
    #[test]
    fn run18_s_expiry_rolls_decide_exactly_the_slots_the_dump_kept() {
        let mut f = fx();
        let s = &mut f.sim;
        let temple = f.barracks; // an ordinary building — the Temple's class
        assert!(
            !s.expire_all(temple, true),
            "an ordinary building is probabilistic, head or not"
        );

        // sim-frame 6383: slots 0 (the head) and 8 hold it. Rolls 61545 and
        // 25792 — the head clears, slot 8 keeps.
        s.rng = crate::combat::Rng::new(0xc593_8177);
        s.ai[1].make_list.list[0] = slot(temple, 2_499_999);
        s.ai[1].make_list.list[8] = slot(temple, 2_499_999);
        s.expire(1, temple as i32, 0, false);
        assert_eq!(s.ai[1].make_list.list[0].t, -1, "61545 % 3 == 0: cleared");
        assert_eq!(
            s.ai[1].make_list.list[8].t, temple as i32,
            "25792 % 3 == 1: kept"
        );
        assert_eq!(s.rng.seed, 0xbb3f_64c1, "two draws, the trace's next seed");

        // sim-frame 6582: the same two slots, the other way round — 17105
        // keeps the head, 1032 clears slot 8. Then the slot loop's own walk
        // (`make_stuff+0x63d`) from slot 5, over the citizen it just bought:
        // 48595 keeps it, and the demoted `val` stays in the list.
        s.rng = crate::combat::Rng::new(0x833a_ab7f);
        s.ai[1].make_list.list[0] = slot(temple, 2_499_999);
        s.ai[1].make_list.list[8] = slot(temple, 2_499_999);
        s.ai[1].make_list.list[5] = slot(f.citizen, 714);
        s.expire(1, temple as i32, 0, false);
        assert_eq!(
            s.ai[1].make_list.list[0].t, temple as i32,
            "17105 % 3 == 2: the head survived its own expiry"
        );
        assert_eq!(s.ai[1].make_list.list[8].t, -1, "1032 % 3 == 0: cleared");
        assert_eq!(s.rng.seed, 0xeb75_0409, "two draws");

        assert!(
            !s.expire_all(f.citizen, false),
            "a peasant slot is probabilistic"
        );
        s.expire(1, f.citizen as i32, 5, false);
        assert_eq!(
            s.ai[1].make_list.list[5].t, f.citizen as i32,
            "48595 % 3 == 1: the bought slot is demoted, not cleared"
        );
        assert_eq!(
            s.rng.seed, 0x35dc_bdd4,
            "one draw, and the frame's next record's seed"
        );
    }

    /// **Run19's four, which separate the head clause from the rest**
    /// (`docs/AI.md` §15.6). Run18b only ever had an ordinary building at
    /// the head, so it could confirm the probabilistic arm and no more. Run19
    /// puts a **scholar** — a unit type that is not a peasant, `0x34` — at
    /// slot 0 on one frame and at slot 1 on another, and the same residue
    /// decides opposite outcomes:
    ///
    /// - sim-frame 8182, slot 1, *not* the head: roll 22883, `% 3 == 2`,
    ///   **kept** — the non-head walk is probabilistic for every type.
    /// - sim-frame 8185, slot 0, the head: roll 45911, `% 3 == 2`, and the
    ///   dump shows it **gone** — the roll is taken and then ignored, which
    ///   is the unconditional arm and the exact clause `docs/AI.md` §2.6
    ///   inverts.
    ///
    /// A same-type, same-residue, opposite-outcome pair is the strongest
    /// evidence either run produced, and it fails under either inversion.
    #[test]
    fn run19_s_head_clause_clears_a_scholar_a_non_head_slot_would_keep() {
        let mut f = fx();
        let s = &mut f.sim;
        // `f.soldier` stands for the scholar: a unit type, not a peasant.
        let scholar = f.soldier;
        assert!(s.expire_all(scholar, true), "at the head: unconditional");
        assert!(!s.expire_all(scholar, false), "elsewhere: probabilistic");

        // 8182: the head is a tech (Coinage — an ordinary, probabilistic
        // type), bought and demoted; its duplicate at slot 4 clears; then the
        // slot loop's own walk keeps the scholar it just bought at slot 1.
        let coinage = f.tech;
        s.rng = crate::combat::Rng::new(0xa6d1_84cf);
        s.ai[1].make_list.list[0] = slot(coinage, 9_999_999);
        s.ai[1].make_list.list[1] = slot(scholar, 9_999_999);
        s.ai[1].make_list.list[4] = slot(coinage, 9_999_999);
        s.expire(1, coinage as i32, 0, s.expire_all(coinage, true));
        assert_eq!(
            s.ai[1].make_list.list[0].t, coinage as i32,
            "11233 % 3 == 1: the bought head is demoted, not cleared"
        );
        assert_eq!(s.ai[1].make_list.list[4].t, -1, "14808 % 3 == 0: cleared");
        s.expire(1, scholar as i32, 1, s.expire_all(scholar, false));
        assert_eq!(
            s.ai[1].make_list.list[1].t, scholar as i32,
            "22883 % 3 == 2: a scholar at a non-head slot survives"
        );
        assert_eq!(s.rng.seed, 0x78f6_5964, "three draws");

        // 8185: the same scholar type, now the head, on a residue that would
        // have kept it anywhere else.
        s.rng = crate::combat::Rng::new(0x3f5a_529d);
        s.ai[1].make_list.list = [MakeObject::EMPTY; MAKE_SLOTS];
        s.ai[1].make_list.list[0] = slot(scholar, 9_999_999);
        s.expire(1, scholar as i32, 0, s.expire_all(scholar, true));
        assert_eq!(
            s.ai[1].make_list.list[0].t, -1,
            "45911 % 3 == 2 and it goes anyway: the head clause is unconditional"
        );
        assert_eq!(s.rng.seed, 0x8244_b358, "the roll is still taken");
    }

    /// `need` is the mean shortfall over the goods the head costs, floored
    /// at zero, and only when the leader is saving up.
    ///
    /// The barracks head costs 1000 food and 1000 timber; the farm in slot 8
    /// costs 600 food. The food purse sits a hundred over `head + slot`, so
    /// the slot's fate turns on `need` alone — and `need` is the mean over
    /// **both** the head's goods, so how short the *timber* purse is decides
    /// whether a food purchase goes through.
    #[test]
    fn the_mean_shortfall_gates_the_rest_of_the_list() {
        fn case(timber_purse: i32) -> usize {
            let mut f = fx();
            let s = &mut f.sim;
            let pb = s.type_price(1, f.barracks).expect("a price");
            let pf = s.type_price(1, f.farm).expect("a price");
            let (fo, ti) = (Resource::Food as usize, Resource::Timber as usize);
            assert_eq!((pb[fo], pb[ti], pf[fo], pf[ti]), (1000, 1000, 600, 0));
            let mut head = slot(f.barracks, 100);
            head.num = 1; // so `can_pay` can answer no
            s.ai[1].make_list.list[0] = head;
            s.ai[1].make_list.list[8] = slot(f.farm, 50);
            s.ledgers[1].bucket[fo] = pb[fo] + pf[fo] + 100;
            s.ledgers[1].bucket[ti] = timber_purse;
            let before = s.rng.seed;
            s.make_stuff(1);
            draws(before, s.rng.seed)
        }
        // No timber: need = ((1000 − 1700) + (1000 − 0)) / 2 = 150, and
        // 1700 < 1000 + 600 + 150. Only the head expires.
        assert_eq!(case(0), 1);
        // 900 timber: need = (−700 + 100) / 2 → floored to 0, and 1700 ≥
        // 1600. The slot runs and takes an expiry of its own.
        assert_eq!(case(900), 2);
    }

    /// Slot 5 buys through a shortfall when the leader has no workers at
    /// all; slot 4 buys a gather building for a good `econ` calls short.
    #[test]
    fn slots_four_and_five_have_their_exceptions() {
        // Every case: a barracks head (1000 food, 1000 timber) over a purse
        // of 600 food — the farm's own price and nothing else — so the head
        // cannot be paid, the slot under test is unaffordable-while-saving,
        // and only its exception can buy it; `can_pay` (`num 1` against the
        // affordable count) then lets the one farm through. One draw is the
        // head alone; two means the slot ran.
        fn case(slot_no: usize, t: TypeId, set: impl FnOnce(&mut Sim)) -> usize {
            let mut f = fx();
            let s = &mut f.sim;
            let head = slot(f.barracks, 100);
            s.ai[1].make_list.list[0] = head;
            s.ai[1].make_list.list[slot_no] = slot(t, 50);
            s.ledgers[1].bucket[Resource::Food as usize] = 600;
            set(s);
            let before = s.rng.seed;
            s.make_stuff(1);
            draws(before, s.rng.seed)
        }
        let f = fx();
        let (farm, tower) = (f.farm, f.tower);
        // Slot 5 with a free peasant: skipped. With none: bought anyway.
        assert_eq!(case(5, farm, |s| s.ai[1].census.free_peasants = 1), 1);
        assert_eq!(case(5, farm, |s| s.ai[1].census.gatherers = 1), 1);
        assert_eq!(case(5, farm, |_| {}), 2);
        // Slot 4, a farm, with food's `econ` bit 4 clear: skipped; set: bought.
        assert_eq!(case(4, farm, |_| {}), 1);
        assert_eq!(
            case(4, farm, |s| s.ai[1].econ[Resource::Food as usize] = 4),
            2
        );
        // The exception is the *type's own* good and only a gather building
        // gets it: a tower in slot 4 is skipped however short food is.
        assert_eq!(
            case(4, tower, |s| s.ai[1].econ[Resource::Food as usize] = 4),
            1
        );
        // And neither exception reaches any other slot.
        assert_eq!(case(6, farm, |_| {}), 1);
    }

    /// From slot 4 on, a `(t, city)` already in **any** of the ranked four is
    /// dropped — slot 0 included — and the city has to match too.
    #[test]
    fn a_slot_that_repeats_the_ranked_four_is_dropped() {
        /// A seed whose next `n` draws all miss the one-in-three, so an
        /// expiry walk over `n` slots clears nothing and the dup rule is
        /// what decides whether slot 9 runs.
        fn quiet_seed(n: usize) -> u32 {
            for seed in 1u32..10_000 {
                let mut r = crate::combat::Rng::new(seed);
                if (0..n).all(|_| r.get(0, 0xffff) % 3 != 0) {
                    return seed;
                }
            }
            panic!("no quiet seed");
        }
        // `k` is the ranked slot slot 9 duplicates; `city` is slot 9's.
        fn case(k: usize, city: i32) -> usize {
            let mut f = fx();
            let s = &mut f.sim;
            s.rng.seed = quiet_seed(4);
            // The head is a farm at city 0, so `k == 0` is a real case; the
            // farm's expiry is probabilistic and the quiet seed keeps every
            // slot alive through it.
            let mut head = slot(f.farm, 100);
            head.city = 0;
            s.ai[1].make_list.list[0] = head;
            if k != 0 {
                let mut m = slot(f.farm, 0); // `val == 0`: the slot loop skips it
                m.city = 0;
                s.ai[1].make_list.list[k] = m;
            }
            let mut dup = slot(f.farm, 10);
            dup.city = city;
            s.ai[1].make_list.list[9] = dup;
            s.ledgers[1].bucket = [10_000; RESOURCES];
            let before = s.rng.seed;
            s.make_stuff(1);
            draws(before, s.rng.seed)
        }
        // The head's expiry walk always draws for slot 0, the ranked slot and
        // slot 9 — every one holds the farm. A slot 9 that survives the dup
        // test adds one more, its own expiry walk.
        for k in 0..4 {
            let n = if k == 0 { 2 } else { 3 };
            assert_eq!(case(k, 0), n, "slot {k}: the duplicate was dropped");
            assert_eq!(case(k, 1), n + 1, "slot {k}: a different city runs");
        }
    }

    /// `make_this` routes by class, and a producer that bought demotes the
    /// slot's value a hundredfold rather than clearing it.
    #[test]
    fn make_this_dispatches_by_class_and_demotes_on_a_buy() {
        let mut f = fx();
        let s = &mut f.sim;
        // Every producer is a stub returning "could not", so nothing is
        // demoted and nothing panics on any of the four classes.
        for (i, t) in [f.tech, f.soldier, f.farm, f.village]
            .into_iter()
            .enumerate()
        {
            s.ai[1].make_list.list[i] = slot(t, 500);
            assert!(!s.make_this(1, i));
            assert_eq!(s.ai[1].make_list.list[i].val, 500);
        }
        // A good in the list is the original's bare `return 1`.
        s.ai[1].make_list.list[0] = slot(0, 500);
        assert!(!s.make_this(1, 0));
        // The `is(TOWN, 0)` arm the listing settled: a Large City is a city
        // type *and* an upgrade, and the upgrade test comes first. Both
        // producers are stubs, so the routing is not yet observable through
        // a return value; what is pinned here is the predicate that picks it.
        assert!(s.type_is_role(f.town, s.tech_tree.roles.town, false));
        assert!(s.slot64(f.town), "and it is also is_city, hence the order");
        assert!(!s.type_is_role(f.village, s.tech_tree.roles.town, false));
        s.ai[1].make_list.list[0] = slot(f.town, 500);
        assert!(!s.make_this(1, 0));

        // The demotion itself, driven through the one producer that is not a
        // stub: a farm placed next to a city.
        let mut f = fx();
        let s = &mut f.sim;
        let at = tile_pos(32, 32);
        let b = s.init_build(1, f.village_rec, at, false);
        s.activate(b, false, true);
        let c = s.buildings[b].city.expect("a finished city has a record");
        s.ai[1].city_ai = vec![CityAi::default(); s.cities.len()];
        s.ai[1].city_ai[c].land = 40;
        s.ai[1].city_ai[c].space = [40, 40, 40];
        s.ai[1].census.resize(256, 2);
        let reg = s.world.region_of(at.cell()).expect("a region");
        s.ai[1].census.reg_free_peasants[usize::from(reg)] = 1;
        s.ledgers[1].bucket = [10_000; RESOURCES];
        let mut m = slot(f.farm, 500);
        m.city = c as i32;
        s.ai[1].make_list.list[0] = m;
        assert!(s.make_this(1, 0), "produce_building placed the farm");
        assert_eq!(s.ai[1].make_list.list[0].val, 5, "val /= 100");
        assert_eq!(s.build_types[f.farm_rec].ident, Ident::Farm);
    }

    /// An active market of my own, which is half the gate.
    fn give_a_market(s: &mut Sim) {
        let market = s.add_build_type(BuildType {
            ident: Ident::Market,
            x_size: 4,
            y_size: 4,
            job_time: 100,
            hits: 100,
            ..BuildType::default()
        });
        let b = s.init_build(1, market, tile_pos(32, 32), false);
        s.activate(b, false, true);
    }

    /// `market_speculation` clamps a runaway escrow — and only behind the
    /// market gate.
    #[test]
    fn market_speculation_clamps_escrow_behind_the_gate() {
        let mut f = fx();
        let s = &mut f.sim;
        s.ledgers[1].escrow[Resource::Food as usize] = 9000;
        s.market_speculation(1);
        assert_eq!(
            s.ledgers[1].escrow[Resource::Food as usize],
            9000,
            "no market: the whole function is skipped"
        );
        // Give the leader an active market — and, since item 348, the
        // **Coinage** that `BUY_SELL` actually asks for. A market alone is
        // Barter, one library tech short of the ability.
        give_a_market(s);
        s.market_speculation(1);
        assert_eq!(
            s.ledgers[1].escrow[Resource::Food as usize],
            9000,
            "a market without Coinage is not the ability"
        );
        s.tech[1].epoch[Line::Commerce.index()] = BUY_SELL_COMMERCE_LEVEL;
        s.market_speculation(1);
        assert_eq!(s.ledgers[1].escrow[Resource::Food as usize], 2000);
        // Under 4000 it stands.
        s.ledgers[1].escrow[Resource::Timber as usize] = 3999;
        s.market_speculation(1);
        assert_eq!(s.ledgers[1].escrow[Resource::Timber as usize], 3999);
    }

    /// **`use_market` spends one draw on a wealth shortfall, and none on
    /// anything else** — item 348, `docs/AI.md` §40.
    ///
    /// The gate is Coinage *and* a market; the draw is the sell rotation's
    /// offset, which wealth reaches without a price because
    /// `TVar3 == WEALTH` short-circuits `calc_market_prices` away.
    #[test]
    fn the_market_draws_once_for_wealth_behind_coinage() {
        let spent = |s: &mut Sim| {
            let before = s.rng.seed;
            s.use_market(1);
            draws(before, s.rng.seed)
        };
        let f_tower = fx().tower;
        let mut f = fx();
        let s = &mut f.sim;
        // A make-list head that costs 64 wealth and nothing else, which is
        // the shape of Great Lakes 8582 — and that map's own bucket.
        let rec = s
            .build_types
            .iter()
            .position(|b| b.ident == Ident::Tower)
            .expect("the fixture's tower");
        // The fixture's prices come out of `type_price` ten times their
        // base, so 6 is the 60 a make-list slot sees.
        s.build_types[rec].price = cost::Price::free().with_base(Resource::Wealth, 6);
        s.ai[1].make_list.list[0] = slot(f_tower, 500);
        s.holdings[1].available = [true, true, true, true, true, false];
        s.ledgers[1].bucket = [58, 55, 43, 68, 54, 0];
        assert_eq!(spent(s), 0, "no market, no ability: nothing is spent");
        give_a_market(s);
        assert_eq!(spent(s), 0, "a market is Barter, not Coinage");
        s.tech[1].epoch[Line::Commerce.index()] = BUY_SELL_COMMERCE_LEVEL;
        assert_eq!(spent(s), 1, "wealth is 43 against a need of 60");
        assert_eq!(
            s.ledgers[1].bucket,
            [58, 55, 43, 68, 54, 0],
            "nothing here clears `need <= bucket - 100`, so nothing is sold"
        );
        // **Raise food over the bar and the sale goes through** (item
        // 358). It is still *one* draw — not because nothing was
        // sellable, but because the sale covers the need and the outer
        // `while` exits before the once-flag is consulted. A hundred food
        // leaves for the default market's `price + flux`, 65.
        s.ledgers[1].bucket[Resource::Food as usize] = 300;
        assert_eq!(spent(s), 1, "a covering sale is also one draw");
        assert_eq!(
            s.ledgers[1].bucket,
            [200, 55, 108, 68, 54, 0],
            "a hundred food for 65 wealth"
        );
        assert_eq!(
            s.market.price[Resource::Food as usize],
            47,
            "supply walks it down"
        );
        // A bucket that covers the need takes nothing at all.
        s.ledgers[1].bucket[Resource::Wealth as usize] = 60;
        assert_eq!(spent(s), 0, "wealth is no longer short");
        // And a shortfall in anything but wealth is **bought**, not sold
        // for, when wealth can stand the buy price — and a buy spends no
        // draw at all.
        s.build_types[rec].price = cost::Price::free().with_base(Resource::Food, 6);
        s.ledgers[1].bucket = [0, 0, 10_000, 0, 0, 0];
        s.market.price = [50; crate::market::GOODS];
        assert_eq!(
            spent(s),
            0,
            "food's branch is a buy, and a buy is draw-free"
        );
        assert_eq!(
            s.ledgers[1].bucket,
            [100, 0, 9_885, 0, 0, 0],
            "a hundred food for the buy price, 2 x 50 + 15"
        );
    }

    /// **The market's three prices and its two trades** — item 358,
    /// `docs/ECONOMY.md` §12. Capture-free, and every clamp on its own
    /// line.
    #[test]
    fn the_market_prices_a_trade_and_moves_its_own_price() {
        let mut f = fx();
        let s = &mut f.sim;
        s.holdings[1].available = [true; RESOURCES];
        let g = Resource::Timber as usize;
        // The default market: price 50, flux 15. Sell is `price + flux`,
        // buy is `2 x price + flux`.
        assert_eq!(s.calc_market_prices(1, g), (115, 65));
        // The sell floor of 1, and the buy floor of `MARKET_BASEMENT x 2`
        // — which a price of zero reaches before the `sell + 10` floor.
        s.market.price[g] = 0;
        s.market.flux[g] = 0;
        assert_eq!(s.calc_market_prices(1, g), (20, 1));
        // And the spread floor: a buy is never under ten over a sell.
        s.market.flux[g] = 40;
        assert_eq!(s.calc_market_prices(1, g), (50, 40));
        // Amber widens the spread by `AMBER_MARKET` in the holder's
        // favour — added to the sell, taken off the buy, never both.
        s.market.price[g] = 50;
        s.market.flux[g] = 15;
        s.ledgers[1].rare = 1 << (crate::economy::AMBER - crate::economy::BASE_RARE);
        assert_eq!(s.calc_market_prices(1, g), (105, 75));
        s.ledgers[1].rare = 0;
        // `do_sell`: a hundred out, the sell price in, the price down by
        // `MARKET_SUPPLY_DEMAND`, and the good's escrow released.
        s.ledgers[1].bucket = [0, 250, 7, 0, 0, 0];
        s.ledgers[1].escrow[g] = 40;
        assert!(s.do_sell(1, g));
        assert_eq!(s.ledgers[1].bucket, [0, 150, 72, 0, 0, 0]);
        assert_eq!((s.market.price[g], s.ledgers[1].escrow[g]), (47, 0));
        // Under a hundred in the bucket is the one refusal, and it moves
        // nothing.
        s.ledgers[1].bucket[g] = 99;
        assert!(!s.do_sell(1, g));
        assert_eq!((s.ledgers[1].bucket[g], s.market.price[g]), (99, 47));
        // `do_buy`: the buy price out of **wealth**, a hundred of the
        // good in, the price up, and it is *wealth's* escrow that is
        // released.
        s.market.price[g] = 50;
        s.ledgers[1].bucket = [0, 0, 200, 0, 0, 0];
        s.ledgers[1].escrow = [0, 0, 130, 0, 0, 0];
        assert!(s.do_buy(1, g));
        assert_eq!(s.ledgers[1].bucket, [0, 100, 85, 0, 0, 0]);
        assert_eq!(
            (
                s.market.price[g],
                s.ledgers[1].escrow[Resource::Wealth as usize]
            ),
            (53, 30)
        );
        // Wealth under the price refuses, and moves nothing.
        s.ledgers[1].bucket[Resource::Wealth as usize] = 10;
        assert!(!s.do_buy(1, g));
        assert_eq!((s.ledgers[1].bucket[g], s.market.price[g]), (100, 53));
    }

    /// A never-started site with no builder is disbanded where the danger
    /// map is positive, and gets a citizen where it is not.
    #[test]
    fn the_orphan_check_disbands_or_recruits() {
        let at = tile_pos(32, 32);
        let mut f = fx();
        let s = &mut f.sim;
        let b = s.init_build(1, f.farm_rec, at, false);
        assert!(!s.buildings[b].active && s.buildings[b].job_counter == 0);
        s.world.set_danger_at(1, at.cell(), 5);
        s.check_orphaned_buildings(1);
        assert!(!s.buildings[b].alive, "disbanded: no builder, danger > 0");

        // With no danger the site keeps, and an idle citizen in the region is
        // sent to build it.
        let mut f = fx();
        let s = &mut f.sim;
        let b = s.init_build(1, f.farm_rec, at, false);
        let mut u = crate::Unit::new(1, 0, tile_pos(36, 32), 40);
        u.ty = Some(1); // the citizen record
        s.units.push(u);
        s.check_orphaned_buildings(1);
        assert!(s.buildings[b].alive);
        assert!(
            s.units[0]
                .orders
                .iter()
                .any(|o| matches!(o.body, Body::Build(x) if x == b)),
            "the citizen was swarmed onto the site"
        );

        // A builder already on it in the same region is left alone.
        let before = s.units[0].orders.len();
        s.check_orphaned_buildings(1);
        assert_eq!(s.units[0].orders.len(), before);
    }

    /// A human leader is never swept.
    #[test]
    fn a_human_leader_is_skipped() {
        let at = tile_pos(32, 32);
        let mut f = fx();
        let s = &mut f.sim;
        let b = s.init_build(0, f.farm_rec, at, false);
        s.world.set_danger_at(0, at.cell(), 5);
        s.nation[0].human = true;
        s.check_orphaned_buildings(0);
        assert!(s.buildings[b].alive);
    }
}
