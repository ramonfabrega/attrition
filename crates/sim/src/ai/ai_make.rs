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
use crate::cities::build::{self, BuildDomain, Ident, flags};
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
            .is_some_and(|r| build::is_city(&self.tables.build_types, r))
    }

    /// `TypeData::is(role, strict)` for one of the roles the tree names.
    fn type_is_role(&self, t: TypeId, role: Option<TypeId>, strict: bool) -> bool {
        role.is_some_and(|x| self.tables.tech_tree.is(t, x, strict))
    }

    /// `TypeData::is_peasant`: exactly `PEASANTS` (`0x32`) or
    /// `PEASANTSKOREAN` (`0x33`) — which is what the loader calls
    /// [`Worker::Citizen`].
    fn is_peasant_type(&self, t: TypeId) -> bool {
        self.unit_record(t)
            .is_some_and(|r| self.tables.unit_types[r].worker == Worker::Citizen)
    } // twin of TypeData::is_peasant

    fn is_wonder_type(&self, t: TypeId) -> bool {
        matches!(
            self.tables.tech_tree.kind(t),
            Kind::Building { wonder: true, .. }
        )
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
        let r = &self.tables.tech_tree.roles;
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
        self.tables.tech_tree.kind(t).is_unit() && !self.is_peasant_type(t)
    }

    /// `get_cost(t, g, who, o, city, 0, 1, −1)` over the six goods.
    ///
    /// SEAM: [`Sim::type_price`] has no object or city form, so the `o` and
    /// `city` arguments the original threads through are dropped. Nothing in
    /// the modelled price path reads them.
    fn make_cost(&self, who: Player, t: i32) -> [i32; RESOURCES] {
        usize::try_from(t)
            .ok()
            .filter(|&t| t < self.tables.tech_tree.types.len())
            .and_then(|t| self.type_price(who, t))
            .unwrap_or([0; RESOURCES])
    }

    /// `RON_DEBUG_MAKE=all|<frame>|<lo>-<hi>`: the head's `can_pay` inputs
    /// on the frames named — its type, `num`, `escrow`, the cost, the
    /// bucket, the affordable count and the verdict — the probe every AI
    /// item re-added by hand (Loop 2955, the thirty-ninth pass; it
    /// answered the battery word's 576 in one run).
    fn make_debug(
        &self,
        who: Player,
        head: &MakeObject,
        cost: &[i32; RESOURCES],
        paid: bool,
    ) -> Option<String> {
        let spec = std::env::var("RON_DEBUG_MAKE").ok()?;
        if !debug_make_wanted(&spec, self.frame) {
            return None;
        }
        let affordable = usize::try_from(head.t)
            .ok()
            .filter(|&t| t < self.tables.tech_tree.types.len())
            .map_or(-1, |t| self.type_affordable(who, t, head.escrow != 0));
        Some(make_debug_line(
            self.frame,
            who,
            head,
            cost,
            &self.economy.ledgers[who as usize].bucket,
            affordable,
            paid,
        ))
    }

    /// `Leader::can_pay(slot)@006c9b90`: `can_pay_cost(who, city, o, escrow)
    /// >= num`.
    pub(crate) fn can_pay_slot(&self, who: Player, m: &MakeObject) -> bool {
        let Ok(t) = usize::try_from(m.t) else {
            return false;
        };
        if t >= self.tables.tech_tree.types.len() {
            return false;
        }
        m.num <= self.type_affordable(who, t, m.escrow != 0)
    }

    /// `type_avail(g, 1) != 0` on one of the six good types. The simulation
    /// carries a good's availability on [`crate::economy::Holdings`], which is
    /// what every other cost path reads.
    fn good_avail(&self, who: Player, g: usize) -> bool {
        self.economy.holdings[who as usize].available[g]
    }

    // ------------------------------------------------------------------
    // `make_stuff`
    // ------------------------------------------------------------------

    /// `make_stuff`: buy what the list and the stockpile allow; `true` when
    /// the head was bought (the original's 1). Draws from the sync stream
    /// per expiring duplicate (§2.6 step 4).
    pub fn make_stuff(&mut self, who: Player) -> bool {
        let w = who as usize;
        let head = *self.ai.ai[w].make_list.head();
        // The head's `t` is compared against `−1` exactly, and its `val`
        // against zero.
        if head.t == -1 || head.val == 0 {
            return false;
        }
        self.use_market(who);
        let paid = self.can_pay_slot(who, &head);
        let cost = self.make_cost(who, head.t);
        let head_t = usize::try_from(head.t).unwrap_or(usize::MAX);
        let valid_head = head_t < self.tables.tech_tree.types.len();
        if let Some(line) = self.make_debug(who, &head, &cost, paid) {
            eprintln!("{line}");
        }

        // `saving` is the original's `local_14 == 0` after the knowledge
        // override: a leader short of knowledge alone does not save up.
        let mut saving = !paid;
        let mut bought_head = false;
        if paid {
            bought_head = self.make_this(who, 0);
        } else if self.economy.ledgers[w].bucket[Resource::Knowledge as usize]
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
                    need += c - self.economy.ledgers[w].bucket[g];
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
            let m = self.ai.ai[w].make_list.list[slot];
            if m.t < 0 || m.val == 0 {
                continue;
            }
            let Ok(t) = usize::try_from(m.t) else {
                continue;
            };
            if t >= self.tables.tech_tree.types.len() {
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
                    let h = self.ai.ai[w].make_list.list[k];
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
                if self.economy.ledgers[w].bucket[g] >= cost[g] + slot_cost[g] + need {
                    continue;
                }
                buy = false;
                if slot == 5 {
                    // The unit slot: buy anyway with no workers at all.
                    let c = &self.ai.ai[w].census;
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
            self.ai.ai[w].site_mark = self.ai.ai[w].site_mark.wrapping_add(1);
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
            if self.ai.ai[w].make_list.list[k].t != t {
                continue;
            }
            self.mark(site);
            let r = self.rng.get(0, 0xffff);
            if r % 3 == 0 || unconditional {
                self.ai.ai[w].make_list.list[k].t = -1;
            }
        }
    }

    /// Slot 4's exception: a gather building for a good whose `econ` says the
    /// rate is under `hi`.
    fn gather_exception(&self, who: Player, t: TypeId) -> bool {
        let Some(rec) = self.build_record(t) else {
            return false;
        };
        if !self.tables.build_types[rec].has(flags::GATHERER) {
            return false;
        }
        let Some(g) = gather_good(self.tables.build_types[rec].ident) else {
            return false;
        };
        self.good_avail(who, g) && self.ai.ai[who as usize].econ[g] & 4 != 0
    }

    // ------------------------------------------------------------------
    // `make_this`
    // ------------------------------------------------------------------

    /// `make_this(slot)`: dispatch one slot to its producer; `true` when it
    /// bought.
    pub fn make_this(&mut self, who: Player, slot: usize) -> bool {
        let w = who as usize;
        let m = self.ai.ai[w].make_list.list[slot];
        let Ok(t) = usize::try_from(m.t) else {
            return false;
        };
        if t >= self.tables.tech_tree.types.len() {
            return false;
        }
        let city = usize::try_from(m.city).ok();
        let kind = self.tables.tech_tree.kind(t);
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
            self.ai.ai[w].make_list.list[slot].val /= 100;
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
        if self.type_is_role(t, self.tables.tech_tree.roles.town, false) {
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
        let Some(c) = city.filter(|&c| c < self.cities.cities.len() && self.cities.cities[c].alive)
        else {
            return false;
        };
        let centre = self.cities.cities[c].building;
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
        let bt = &self.tables.build_types[rec];
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
            let ai = self.ai.ai[w].city_ai.get(c);
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
        self.produce_building(who, rec, centre, Some(c), m.escrow != 0)
    }

    /// `reg_free_peasants[r] != 0 || reg_gatherers[r] != 0` — the region test
    /// both of `make_this`'s placement arms make before spending.
    fn region_has_workers(&self, who: Player, reg: Option<u16>) -> bool {
        let Some(r) = reg.map(usize::from) else {
            return false;
        };
        let c = &self.ai.ai[who as usize].census;
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
                    .is_some_and(|r| build::is(&self.tables.build_types, r, Ident::Market))
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
    /// And **no nuke embargo** ([`Sim::nuke_embargo`]) — `get_nuke_embargo`
    /// is the gate's last clause in `use_market` and `market_speculation`
    /// both (item 1870: Great Sahara's leader 1 holds one from its first
    /// nuke at 2101 to frame 4001, and sold no timber on 3176).
    fn market_gate(&self, who: Player) -> bool {
        let w = who as usize;
        let bonus = self
            .tables
            .tech_tree
            .has_tribe_bonus(&self.tables.setup, &self.tech[w], 4);
        (bonus || self.can_buy_sell(who)) && self.has_market(who) && self.nuke_embargo(who) == 0
    }

    /// `LeaderData::get_my_nuke_embargo@006d5350` — the frames a nation's own
    /// nukes keep it out of the market: `0` unless `nuke_stamp` is set or the
    /// nation holds the Space Program, else `NUKE_EMBARGO_WORLD ×
    /// armageddon + NUKE_EMBARGO_NATION × nukes_used − frame +
    /// NUKE_EMBARGO_BASE + nuke_stamp`, floored at zero.
    ///
    /// SEAM: `Game::armageddon` (`+0x6e0`) is not carried; the shipped
    /// `NUKE_EMBARGO_WORLD` is `0 frames`, which makes the term vanish.
    fn my_nuke_embargo(&self, who: Player) -> i32 {
        let stamp = self.combat.nukes.stamp_of(who);
        if stamp == 0 || self.wonders_held(who) & (1 << crate::tech::wonder::SPACE_PROGRAM) != 0 {
            return 0;
        }
        let t = &self.tables.tuning;
        let v = i64::from(t.nuke_embargo_nation) * i64::from(self.combat.nukes.used_of(who))
            - self.frame
            + i64::from(t.nuke_embargo_base)
            + stamp;
        v.max(0) as i32
    }

    /// `LeaderData::get_nuke_embargo@006d52c0` — the longest embargo among
    /// the nation and its allies, unless the nation holds the Space Program.
    pub fn nuke_embargo(&self, who: Player) -> i32 {
        if self.wonders_held(who) & (1 << crate::tech::wonder::SPACE_PROGRAM) != 0 {
            return 0;
        }
        (0..self.territory.players.len())
            .map(|p| p as Player)
            .filter(|&p| !self.leaders.defeated[p as usize] && (p == who || self.is_ally(who, p)))
            .map(|p| self.my_nuke_embargo(p))
            .max()
            .unwrap_or(0)
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
        let l = &self.economy.ledgers[who as usize];
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
            let m = self.ai.ai[w].make_list.list[slot];
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
            if self.economy.ledgers[w].bucket[g] >= need[g] {
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
                    self.economy.ledgers[w].bucket[wealth] - buy < need[wealth]
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
                if self.economy.ledgers[w].bucket[g] >= need[g] {
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
        let price = self.economy.market.price[g];
        let flux = self.economy.market.flux[g];
        let mut sell = price + flux;
        let mut buy = price * 2 + flux;
        let nubian = self.tables.tuning.nubian_market_prices;
        let bonus = if nubian != 0
            && self
                .tables
                .tech_tree
                .has_tribe_bonus(&self.tables.setup, &self.tech[w], 4)
        {
            nubian
        } else if self.has_rare(who, crate::economy::AMBER) {
            self.tables.tuning.amber_market
        } else {
            0
        };
        sell += bonus;
        buy -= bonus;
        sell = sell.max(1);
        buy = buy.max(self.tables.tuning.market_basement * 2);
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
        if self.economy.ledgers[w].bucket[g] < 100 {
            return false;
        }
        self.economy.ledgers[w].bucket[g] -= 100;
        self.economy.ledgers[w].escrow[g] = (self.economy.ledgers[w].escrow[g] - 100).max(0);
        self.economy.ledgers[w].bucket[Resource::Wealth as usize] += sell;
        self.economy.market.price[g] =
            (self.economy.market.price[g] - self.tables.tuning.market_supply_demand).max(0);
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
        if self.economy.ledgers[w].bucket[wealth] < buy {
            return false;
        }
        self.economy.ledgers[w].bucket[wealth] -= buy;
        self.economy.ledgers[w].escrow[wealth] =
            (self.economy.ledgers[w].escrow[wealth] - 100).max(0);
        self.economy.ledgers[w].bucket[g] += 100;
        self.economy.market.price[g] += self.tables.tuning.market_supply_demand;
        true
    }

    /// `market_speculation@006c8110`: `production_ai_setup`'s last act, and
    /// the only place the AI trades **off** a shortfall — three passes over
    /// the six goods behind one gate.
    ///
    /// 1. **The tier**, and the escrow clamp beside it: an escrow over 4000
    ///    is pulled back to 2000, and the smallest stockpile decides how
    ///    hungry the leader is — `2` when any available good is under 100,
    ///    `1` when one is under 200 and none under 100, `0` otherwise. The
    ///    tier is taken over **every** available good, wealth and knowledge
    ///    included.
    /// 2. **The sell pass**, over the four tradeable goods: sell a hundred
    ///    when the unescrowed pile clears `2000 >> tier` **and** the sell
    ///    price clears `75 / (tier + 1)`. A hungry leader will part with a
    ///    smaller pile and accept a worse price.
    /// 3. **The buy pass**, over the same four, and it does *not* read the
    ///    tier. A hundred is bought when the buy price is within the
    ///    unescrowed wealth, the stock is under 2000, the price is under
    ///    201, and a ladder that gets stricter as the pile grows: at 500 or
    ///    more the price must be under 26, at 200 under 51, at 100 under
    ///    101. Each purchase spends wealth and the next good re-reads it,
    ///    so a leader can clear its purse on the first good it wants.
    ///
    /// **No draw in any of it**, which is why the buy pass could be missing
    /// for two hundred items without the draw stream saying so — and why
    /// Great Lakes' word sat five frames downstream of it. ~~The sell and
    /// buy passes are the same SEAM as [`Sim::use_market`]~~ — both were
    /// written when [`Sim::calc_market_prices`] did not exist, and the
    /// comment outlived it by a month: on run117's block 10577 the original
    /// buys a hundred food for **128** wealth and this crate buys nothing,
    /// so it enters the rotation's `make_stuff` five frames later holding
    /// `94 87 125` where the original holds `194 87 0`
    /// (`docs/ECONOMY.md` §13).
    pub fn market_speculation(&mut self, who: Player) {
        let w = who as usize;
        if self.ai.lobby.resources_unlimited() || !self.market_gate(who) {
            return;
        }
        let mut tier = 0i32;
        for g in 0..RESOURCES {
            if !self.good_avail(who, g) {
                continue;
            }
            if self.economy.ledgers[w].escrow[g] > 4000 {
                self.economy.ledgers[w].escrow[g] = 2000;
            }
            let stock = self.economy.ledgers[w].bucket[g];
            if stock < 100 {
                tier = tier.max(2);
            } else if stock < 200 && tier == 0 {
                tier = 1;
            }
        }
        let wealth = Resource::Wealth as usize;
        let knowledge = Resource::Knowledge as usize;
        // The sell pass. `2000 / (1 << tier)` in the original, which is the
        // shift for every tier it can produce.
        for g in 0..RESOURCES {
            if !self.good_avail(who, g) || g == wealth || g == knowledge {
                continue;
            }
            if self.economy.ledgers[w].bucket[g] - self.economy.ledgers[w].escrow[g] < 2000 >> tier
            {
                continue;
            }
            let (_, sell) = self.calc_market_prices(who, g);
            if sell < 75 / (tier + 1) {
                continue;
            }
            // The inner gate is the outer one again (the embargo arm aside,
            // which is zero here), exactly as in [`Sim::use_market`].
            if self.market_gate(who) {
                self.do_sell(who, g);
            }
        }
        // The buy pass, which reads no tier at all. The purse and the stock
        // are re-read per good because the good before may have spent them.
        for g in 0..RESOURCES {
            if !self.good_avail(who, g) || g == wealth || g == knowledge {
                continue;
            }
            let (buy, _) = self.calc_market_prices(who, g);
            let purse =
                self.economy.ledgers[w].bucket[wealth] - self.economy.ledgers[w].escrow[wealth];
            let stock = self.economy.ledgers[w].bucket[g];
            if buy > purse || stock >= 2000 || buy >= 201 {
                continue;
            }
            if (stock >= 500 && buy >= 26)
                || (stock >= 200 && buy >= 51)
                || (stock >= 100 && buy >= 101)
            {
                continue;
            }
            if self.market_gate(who) {
                self.do_buy(who, g);
            }
        }
    }

    // ------------------------------------------------------------------
    // The orphan check
    // ------------------------------------------------------------------

    /// `check_orphaned_buildings`: disband marked buildings and stranded
    /// sites, or send a builder.
    pub fn check_orphaned_buildings(&mut self, who: Player) {
        let w = who as usize;
        if self.cities.nation[w].human {
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
                .is_none_or(|r| self.tables.build_types[r].domain() == BuildDomain::Land);
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
        // By object number, as the recruit scan below (`check_orphaned_buildings`'s loop
        // over `objects +0x15c[who]`).
        for u in self.units_by_o(who) {
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
    ///
    /// **First by object number, not by this crate's slot**
    /// (`check_orphaned_buildings@006c9f20`: `iVar1` from 0 to `objects
    /// +0x15c[who]`, the list indexed by `o`). A reused number puts the two
    /// apart: the Bantu Modern pair's 3376 (`docs/AI.md` §370.1), where
    /// `1/94` (uid 162, born into a freed number) is the original's first
    /// gatherer for the site `1/2046` and the slot walk reached `1/95` (uid
    /// 137) and then `1/105` (uid 149) first. The same lesson as
    /// `produce_building`'s (§210) and `produce_city`'s (§358.4).
    fn recruit_builder(&mut self, who: Player, b: usize, site_reg: Option<u16>) -> bool {
        let mut pick = None;
        for u in self.units_by_o(who) {
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

/// Whether `RON_DEBUG_MAKE`'s spec names `frame`: `all`, one frame, or an
/// inclusive `lo-hi`; anything else names nothing (Loop 2955).
pub(crate) fn debug_make_wanted(spec: &str, frame: i64) -> bool {
    let spec = spec.trim();
    if spec == "all" {
        return true;
    }
    if let Some((lo, hi)) = spec.split_once('-') {
        return match (lo.trim().parse::<i64>(), hi.trim().parse::<i64>()) {
            (Ok(lo), Ok(hi)) => (lo..=hi).contains(&frame),
            _ => false,
        };
    }
    spec.parse::<i64>().is_ok_and(|f| f == frame)
}

/// One line, the head's `can_pay` inputs (Loop 2955): `MAKE f<frame> who=W
/// head t=T num=N escrow=E cost=[..] bucket=[..] affordable=A paid=yes|no`.
pub(crate) fn make_debug_line(
    frame: i64,
    who: Player,
    head: &MakeObject,
    cost: &[i32; RESOURCES],
    bucket: &[i32; RESOURCES],
    affordable: i32,
    paid: bool,
) -> String {
    format!(
        "MAKE f{frame} who={who} head t={} num={} escrow={} cost={cost:?} bucket={bucket:?} affordable={affordable} paid={}",
        head.t,
        head.num,
        head.escrow,
        if paid { "yes" } else { "no" }
    )
}

#[cfg(test)]
mod tests;
