//! The production queue: queueing, pricing a research, the train target and
//! tail, `do_queue`'s slots, and the tracking of what is queued
//! (`docs/PRODUCTION.md`; `docs/STRUCTURE.md` §3, `cities/`).
//!
//! Moved out of `lib.rs` unchanged by item 4014 (`docs/STRUCTURE.md` §5, move 6).

use crate::*;

/// What one call of `do_queue` on one slot did — the sign `Build::finished`
/// hands back, plus "not done yet".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Advanced {
    /// The counter moved and the entry is not done.
    Pending,
    /// A research entry completed: the bit is set, the entry is gone, no unit.
    Researched,
    /// A train entry completed and a unit was placed.
    Trained(Produced),
    /// Done, offered, refused — zero or negative from `finished`. The entry
    /// stays at full progress. At slot 0 this is what triggers the redirect.
    Blocked,
    /// Done, offered, refused with a **negative** answer — an aircraft or
    /// caravan limit (`Build::finished` < 0). The same as [`Self::Blocked`],
    /// and `do_queue`'s second fallback besides.
    Limited,
}

impl Sim {
    /// Where an order given at `at` actually lands. A library that is not the
    /// player's first forwards to the first — `queue_up`, `unqueue` and
    /// `do_queue` all perform this redirection, which is what puts every
    /// research job in the game on one queue.
    pub(crate) fn queue_home(&self, at: usize) -> usize {
        if !self.buildings[at].is_library {
            return at;
        }
        self.first_library(self.buildings[at].owner).unwrap_or(at)
    }

    /// [`Sim::queue_home`] for a cancel of `slot` at `at`:
    /// `Build::unqueue@006207c0` and `Build::action_unqueue@00620280` each
    /// forward to the first library **unless the slot they name, at the
    /// building itself, holds `DISBAND`** (0x29a, the type short at `+4` of
    /// the entry; `action_unqueue` reads slot `max(p, 0)`). A second
    /// library's own Disband — `Build::queue_up` never forwards one
    /// ([`Sim::queue_disband`]) — is cancelled where it stands (item 3989,
    /// `docs/GOLDEN.md` §182).
    pub(crate) fn unqueue_home(&self, at: usize, slot: usize) -> usize {
        if self.buildings[at]
            .queue
            .items
            .get(slot)
            .is_some_and(|i| i.disband)
        {
            return at;
        }
        self.queue_home(at)
    }

    /// Orders one unit of a type at a building — `Build::queue_up`.
    ///
    /// **The price is charged here**, and what was charged is written into the
    /// queue entry, which is what makes a later cancellation exact. Note what
    /// is *not* checked: the population cap. A player at the cap may queue
    /// freely, and finds out at the far end; see [`Sim::process_queues`].
    ///
    /// The two gates are the original's and in its order — affordability
    /// first, then room in the queue.
    pub fn queue_up(&mut self, at: usize, ty: usize) -> Result<usize, production::QueueFail> {
        self.queue_up_with(at, ty, false)
    }

    /// [`Sim::queue_up`] with the caller's escrow flag — `Build::queue_up(t,
    /// escrow)`. The affordability test counts the **whole** stockpile
    /// whatever the flag (`can_pay_cost(who, −1, o, 1)`, `0x6214b2`), and the
    /// flag goes to the charge (`pay_cost(…, escrow, …)`, `0x621949`): an
    /// escrowed purchase draws the reservation down, any other abandons it
    /// where it had to reach into it (`docs/AI.md` §98).
    pub(crate) fn queue_up_with(
        &mut self,
        at: usize,
        ty: usize,
        escrow: bool,
    ) -> Result<usize, production::QueueFail> {
        let at = self.queue_home(at);
        let who = self.buildings[at].owner;
        // `BuildData::can_make`: not active, or in an unassimilated city, makes
        // nothing.
        if !self.buildings[at].active
            || !self.buildings[at].alive
            || self.building_unassimilated(at)
        {
            return Err(production::QueueFail::CantTrain);
        }
        // `BuildData::can_make@0062db10:31`–`:36`: `BuildTypeData::queue_here`
        // stands ahead of the per-kind tail and binds a unit too, so a
        // building that does not make the type refuses it — what a press on
        // a Barracks and a Stable together leans on (item 3966, T71-14,
        // `docs/GOLDEN.md` §172). Only a building and a type the tree both
        // carry can be refused by it.
        if let Some(id) = self.unit_types[ty].tree
            && let Some(b) = self.buildings[at].ty.and_then(|b| self.build_types[b].tree)
            && !self.tech_tree.queue_here(b, id)
        {
            return Err(production::QueueFail::CantTrain);
        }
        // `BuildData::can_make` comes before the price: a type the tree says
        // is not available is refused before anything is charged. Only a type
        // that is in the tree can be refused by it.
        if let Some(id) = self.unit_types[ty].tree
            && self
                .tech_tree
                .type_avail(&self.setup, &self.tech[who as usize], id, true)
                == tech::NOT_AVAILABLE
        {
            return Err(production::QueueFail::CantTrain);
        }
        let charges = self.price_of(who, ty);
        let available = self.holdings[who as usize].available;
        if cost::affordable(&charges, &self.ledgers[who as usize], &available, true) < 1 {
            return Err(production::QueueFail::Cost);
        }
        if !self.unit_queue_has_room(at, ty) {
            return Err(production::QueueFail::Full);
        }
        cost::pay(
            &charges,
            &mut self.ledgers[who as usize],
            &available,
            escrow,
        );
        // `Build::queue_up@00620f40` stamps `tech_frame` for a type that is
        // not a technology too, when `type_avail` answers 2 (researchable):
        // a unit upgrade bought as research (item 1496, Great Lakes' 11582 —
        // run159's Phalanx at `1/2016` — holds it at 11582 on block 11583).
        if let Some(id) = self.unit_types[ty].tree
            && self
                .tech_tree
                .type_avail(&self.setup, &self.tech[who as usize], id, true)
                == tech::RESEARCHABLE
            && let Some(ai) = self.ai.get_mut(who as usize)
        {
            ai.tech_frame = self.frame;
        }
        let slot = self.buildings[at].queue.push(ty, &charges);
        self.muster[who as usize].queued_by_type[ty] += 1;
        self.track_tree_queued(who, ty, 1);
        // No `economy_changed` (item 1398, `docs/AI.md` §103): `Build::queue_up`
        // never raises `leader_flags & 0x2000000`.
        Ok(slot)
    }

    /// A technology's price for `who` — `TypeData::get_cost` over a tech
    /// record: `COST × TECH_COST_FACTOR`, the **science discount**, no ramp,
    /// Democracy's non-library discount, the British taxation discount,
    /// and the redirect for a good the player does not have
    /// (`docs/COSTS.md` §"The redirect"). The rest of the discount tail —
    /// being behind in ages, the lobby's tech-cost setting, the final-tech
    /// ramp, Incense — is [`cost::Modifiers`]'s
    /// and arrives as the undiscounted price until the layers that produce
    /// it exist, as [`Sim::price_of`] does for a unit.
    ///
    /// The British term is `get_cost`'s `TVar21 − TAXATION < 4 &&
    /// has_tribe_bonus(0xb)` → `(100 − BRITISH_TAXATION_DISCOUNT) × p /
    /// 100`, after the science discount and before the redirect. A tech has
    /// no ramp, so it rides [`cost::Modifiers::late_discount`], the same
    /// expression at the same place (`docs/AI.md` §74).
    pub fn tech_price(&self, who: Player, t: tech::TypeId) -> [i32; economy::RESOURCES] {
        // `get_cost:1048`–`1053`, right after `TECH_COST_FACTOR`: a
        // government (`TypeData::is_gov_type`, `0x26f..0x274`) costs an
        // American leader (`has_tribe_bonus(0x14)`) nothing, and every later
        // step is a percentage of that zero (item 3374: run999b's who=1
        // queues Republic and a Dock on 10982 and pays only the Dock's
        // timber).
        if matches!(self.tech_tree.types[t].kind, tech::Kind::Gov { .. })
            && self.tech_tree.has_tribe_bonus(
                &self.setup,
                &self.tech[who as usize],
                crate::ai::tribe::AMERICANS,
            )
        {
            return [0; economy::RESOURCES];
        }
        let holdings = &self.holdings[who as usize];
        let (rare_goods, gypsum) = self.rare_goods_discounts(who);
        let british_taxation = self.tech_tree.roles.taxation_line.contains(&t)
            && self
                .tech_tree
                .has_tribe_bonus(&self.setup, &self.tech[who as usize], 0xb);
        let price = cost::Price {
            kind: cost::Kind::Tech,
            base: self.tech_tree.types[t].cost,
            ..cost::Price::free()
        };
        cost::charges(
            &self.tuning,
            &price,
            cost::Counts::default(),
            &cost::Modifiers {
                science_ahead: self.science_ahead(who, t),
                democracy: self.democracy_tech_discount(who, t),
                industry: self.industry_tech_discount(who, t),
                german_industry: self.german_industry_tech_discount(who, t),
                greek_research: self.greek_research_discount(who, t),
                late_discount: if british_taxation {
                    self.tuning.british_taxation_discount
                } else {
                    0
                },
                line_discounts: self.line_discounts(who, t),
                silver: self.silver_age_discount(who, t),
                rare_goods,
                gypsum,
                ..cost::Modifiers::default()
            },
            &holdings.available,
            &holdings.discovered,
            &self.redirects,
        )
    }

    /// `get_cost`'s industry arm (`00666940`..`006669b0`): the Hanging
    /// Gardens' discount on the food, timber and metal gathering lines —
    /// `0x261..=0x263`, `0x264..=0x266` and `0x26c..=0x26e` (an unsigned
    /// `ja` on the last), not the knowledge line between them — when the
    /// leader holds the wonder (item 3332: who=1's Metal Alloys, 68 food
    /// and 68 timber where 200 left it unaffordable on Great Sahara's
    /// 15778). The German term on the same predicate is
    /// [`Sim::german_industry_tech_discount`].
    pub(crate) fn industry_tech_discount(&self, who: Player, t: tech::TypeId) -> i32 {
        let held = self.wonders_held(who) & (1 << tech::wonder::HANGING_GARDENS) != 0;
        if held && self.is_industry_tech(t) {
            self.tuning.hanging_gardens_industry_discount
        } else {
            0
        }
    }

    /// `get_cost`'s German term on the industry predicate (`006669b0`..,
    /// `:1277`–`1281`), right after the Hanging Gardens': a German leader
    /// (`has_tribe_bonus(0xc)`) takes `GERMAN_INDUSTRY_DISCOUNT` off a food,
    /// timber or metal gathering-line technology, its own truncation. Item
    /// 3378: run1003's block 13579, who=1's Carpentry (609) listed at `val`
    /// 120000 by the original and 30000 here — the price `research_techs`
    /// reads its affordability from was twice the original's.
    pub(crate) fn german_industry_tech_discount(&self, who: Player, t: tech::TypeId) -> i32 {
        if self.is_industry_tech(t)
            && self.tech_tree.has_tribe_bonus(
                &self.setup,
                &self.tech[who as usize],
                crate::nations::power::GERMANS,
            )
        {
            self.tuning.german_industry_discount
        } else {
            0
        }
    }

    /// `get_cost`'s industry predicate: the food, timber and metal
    /// gathering lines — `0x261..=0x263`, `0x264..=0x266` and
    /// `0x26c..=0x26e` — and not the knowledge line between them.
    pub(crate) fn is_industry_tech(&self, t: tech::TypeId) -> bool {
        const INDUSTRY: [&str; 9] = [
            "Carpentry",
            "Logging Industry",
            "Papermill",
            "Agriculture",
            "Crop Rotation",
            "Food Industry",
            "Metal Alloys",
            "Cold Casting",
            "Steel",
        ];
        INDUSTRY.contains(&self.tech_tree.types[t].name.as_str())
    }

    /// The highest held Democracy bonus, for non-library technologies
    /// only (`docs/AI.md` §109). `where` is tested directly, not by lineage.
    pub(crate) fn democracy_tech_discount(&self, who: Player, t: tech::TypeId) -> i32 {
        if matches!(
            self.tech_tree.kind(t),
            tech::Kind::Age(_) | tech::Kind::Epoch { .. }
        ) || self.tech_tree.types[t]
            .where_
            .and_then(|at| self.build_record(at))
            .is_some_and(|at| self.build_types[at].ident == build::Ident::Library)
        {
            return 0;
        }
        self.bonus_tier_held(who, &self.tech_tree.roles.democracy_preqs)
            .map_or(0, |tier| self.tuning.democracy_tech_bonus[tier])
    }

    /// The highest tier of a government bonus ladder the player holds —
    /// `LeaderData::has_preq` asked of each tier from the top down, `None`
    /// when not even the first is held.
    pub(crate) fn bonus_tier_held(
        &self,
        who: Player,
        rows: &[Option<[tech::Preq; 3]>],
    ) -> Option<usize> {
        (0..rows.len()).rev().find(|&tier| {
            // A tier above the first also holds the one under it, two
            // governments taken, and — Monarchy or Democracy being the tier
            // below's `preq0` — Socialism or Capitalism (group 21; A1 row 5).
            let p = &self.tech[who as usize];
            (0..=tier).all(|k| {
                rows[k].is_some_and(|qs| {
                    qs.iter()
                        .all(|&q| self.tech_tree.has_tech_p(&self.setup, p, q))
                }) && self.tech_tree.bonus_tier_extra(
                    &self.setup,
                    p,
                    k,
                    k.checked_sub(1).and_then(|b| match rows[b] {
                        Some([tech::Preq::Of(t), ..]) => Some(t),
                        _ => None,
                    }),
                )
            })
        })
    }

    /// `TypeData::get_cost`'s **Greek research arm** (`LAB_006667c4`): an
    /// age (`0x220..0x226`) or a library epoch (`0x227..0x242`) priced for
    /// a Greek leader (`has_tribe_bonus(5)`) takes `GREEK_RESEARCH_COST`
    /// off every good but knowledge ([`cost::Modifiers::greek_research`]).
    /// run961c's block 37: who=1's City State at the Library `1/2005` is
    /// queued at food 108 in the original against 120 here, and who=1's food
    /// bucket stands twelve apart from frame 36 (item 3326, `docs/AI.md`
    /// §266).
    pub(crate) fn greek_research_discount(&self, who: Player, t: tech::TypeId) -> i32 {
        if matches!(
            self.tech_tree.kind(t),
            tech::Kind::Age(_) | tech::Kind::Epoch { .. }
        ) && self.tech_tree.has_tribe_bonus(
            &self.setup,
            &self.tech[who as usize],
            crate::nations::power::GREEKS,
        ) {
            self.tuning.greek_research_cost
        } else {
            0
        }
    }

    /// `TypeData::get_cost`'s **Silver arm** (`00666af0`..`00666b51`): an
    /// age (`0x220..0x226`, the `is_age_type` slot) priced for a leader whose
    /// `rare` or `rare_conquest` carries bit 12, `SILVER`, takes
    /// `SILVER_AGE_COST` off — `docs/COSTS.md`, "The discounts". Anything
    /// that is not an age takes nothing.
    /// `get_cost`'s rares' tail ([`cost::Modifiers::rare_goods`]): every
    /// price — unit, building and technology — reaches it.
    pub(crate) fn rare_goods_discounts(&self, who: Player) -> ([i32; economy::RESOURCES], i32) {
        let pct = |rare, x| if self.has_rare(who, rare) { x } else { 0 };
        let t = &self.tuning;
        let mut by = [0; economy::RESOURCES];
        by[economy::Resource::Timber.index()] = pct(economy::COAL, t.coal_timber_cost);
        by[economy::Resource::Food.index()] = pct(economy::SUGAR, t.sugar_food_cost);
        by[economy::Resource::Wealth.index()] = pct(economy::GOLD, t.gold_wealth_cost);
        by[economy::Resource::Metal.index()] = pct(economy::IRON, t.iron_metal_cost);
        (by, pct(economy::GYPSUM, t.gypsum_cost))
    }

    pub(crate) fn silver_age_discount(&self, who: Player, t: tech::TypeId) -> i32 {
        if matches!(self.tech_tree.kind(t), tech::Kind::Age(_))
            && self.has_rare(who, economy::SILVER)
        {
            self.tuning.silver_age_cost
        } else {
            0
        }
    }

    /// `TypeData::get_cost`'s **library-line tail** (`00666b7d`, the jump
    /// table at `006673a0`): a library epoch's price takes up to three more
    /// percentages, chosen by its line, after every other tech discount —
    /// `docs/AI.md` §99.8.
    ///
    /// | line | first | second | third |
    /// |---|---|---|---|
    /// | military | Furs | the highest `DESPOTISM_n` held | the Turks |
    /// | civic | Dye | the Persians | — |
    /// | commerce | Silk | the Dutch | — |
    /// | science | Papyrus | the Chinese | the Americans |
    ///
    /// Anything that is not a library epoch — an age, a plain tech, a
    /// unit — takes none of them.
    pub(crate) fn line_discounts(&self, who: Player, t: tech::TypeId) -> [i32; 3] {
        let tech::Kind::Epoch { line, .. } = self.tech_tree.kind(t) else {
            return [0; 3];
        };
        let w = who as usize;
        let tu = &self.tuning;
        let rare = |good: usize, x: i32| if self.has_rare(who, good) { x } else { 0 };
        let tribe = |n: usize, x: i32| {
            if self
                .tech_tree
                .has_tribe_bonus(&self.setup, &self.tech[w], n)
            {
                x
            } else {
                0
            }
        };
        match line {
            tech::Line::Military => {
                let despot =
                    match self.government_bonus_level(who, &self.tech_tree.roles.despotism_preq) {
                        0 => 0,
                        n => tu.despotism_military_cheaper[n - 1],
                    };
                [
                    rare(0x15, tu.furs_military),
                    despot,
                    tribe(8, tu.turk_military_cheap),
                ]
            }
            tech::Line::Civic => [
                rare(10, tu.dye_civic_commerce),
                tribe(0x17, tu.persians_civic_discount),
                0,
            ],
            tech::Line::Commerce => [
                rare(11, tu.silk_commerce),
                tribe(0x16, tu.dutch_commerce_discount),
                0,
            ],
            tech::Line::Science => [
                rare(9, tu.papyrus_science_military),
                tribe(0xe, tu.chinese_science_discount),
                tribe(0x14, tu.americans_science_discount),
            ],
        }
    }

    /// How many Science levels `who` is ahead of technology `t` —
    /// `LeaderData::calc_science_discount`'s level term, on its **price**
    /// side.
    ///
    /// `epoch[3]` less the tech's own `AGE`, and the plus-one is the whole
    /// subtlety: the original adds one to `AGE` unless the type is an age
    /// (`0x220..0x226`) or a library epoch (`0x227..0x242`), which is
    /// [`tech::Kind::is_plain_tech`]. A non-tech gets zero, because the
    /// original guards the function on `is_tech_type` and returns the price
    /// untouched.
    ///
    /// The answer is signed. `docs/COSTS.md` §"The discounts": a player whose
    /// Science line is behind the tech's age pays *more*, out of the same
    /// expression.
    pub(crate) fn science_ahead(&self, who: Player, t: tech::TypeId) -> i32 {
        let kind = self.tech_tree.kind(t);
        if !kind.is_tech() {
            return 0;
        }
        let level = self.tech_tree.types[t].age + i32::from(kind.is_plain_tech());
        self.tech[who as usize].epoch[tech::Line::Science.index()] - level
    }

    /// `TypeData::research_time` for a tech, in hundredths of a frame:
    /// `JOB_TIME × 100 × RESEARCH_TICK_PREMIUM >> 8`, then the one step of
    /// `train_time`'s research tail this simulation has — the science
    /// speedup — then the floor at one. The unit-only
    /// `RESEARCH_PREMIUM_TIME` does not apply (`docs/PRODUCTION.md` §"The
    /// base, and the research step").
    ///
    /// The speedup reads the tech's `AGE` **raw**, not the price side's
    /// `AGE + 1`: see [`production::science_speedup`]. Because the target is
    /// recomputed every frame rather than stored, a Science epoch landing
    /// mid-research shortens the job already in progress — which is what
    /// puts the AI's second library entry on the original's frame rather
    /// than twenty later.
    ///
    /// **A held Supercollider zeroes it** (`train_time@006508c0`'s tech
    /// block: `has_wonder(0x21d)` sets `local_8 = 0` for every type but
    /// `DISBAND` and `RAZE`, neither of which is a tech here), so the floor
    /// answers one and the entry finishes on its next tick whatever its
    /// counter. A unit's research job is not in that block and keeps its
    /// time. run838's 4671: who=1's Supercollider `1/2034` completes, and
    /// the Senate `1/2019`'s Monarchy (at 28900 of 42000) finishes the same
    /// frame (`docs/AI.md` §216).
    pub fn tech_time(&self, who: Player, t: tech::TypeId) -> i32 {
        if self.wonders_held(who) & (1 << tech::wonder::SUPERCOLLIDER) != 0 {
            return production::MIN_TIME;
        }
        let time = self.tech_tree.types[t].job_time * production::TIME_SCALE;
        let time = production::research_tick(time, self.tuning.research_tick_premium);
        let science = self.tech[who as usize].epoch[tech::Line::Science.index()];
        let mut tail = self.held_elsewhere(t);
        tail.extend(self.age_catch_up(who, t));
        tail.extend(self.epoch_catch_up(who, t));
        tail.extend(self.greek_research(who, t));
        tail.extend(self.research_rares(who));
        tail.extend(production::science_speedup(
            &self.tuning,
            science,
            self.tech_tree.types[t].age,
        ));
        production::adjusted(time, tail.as_slice()).max(production::MIN_TIME)
    }

    /// **The held-elsewhere term** of `ObjectData::train_time@006508c0`'s
    /// research block (`LAB_006513c0`, the decompile's lines 414–419): a
    /// tech that is neither an age nor an epoch — a final, a building tech
    /// or a government ([`tech::Kind::is_plain_tech`]) — asks its type's
    /// vslot `+0x74`, `TypeData::count_discovered@00663f80`, how many leader
    /// slots in use (`leader_flags & 1`, a defeated seat too) hold its bit,
    /// and with `n` of them takes `t × (num_players − n + 1)` and divides by
    /// `num_players + 1` at `LAB_006513e8`, the Greeks' divide, which the
    /// term reaches past the Greeks' test: the two are never on one tech.
    /// Like the rest of the block it is read live, so the frame another
    /// leader gains the tech shortens a job already running.
    ///
    /// The first melee's Despotism (item 3783, `docs/GOLDEN.md` §149.9):
    /// who=2's Senate finishes it on 957, and who=1's, at 37,700 of 40,000
    /// on 958, is then at 30,000 of `num_players` 3 — `3 × 40000 / 4` — and
    /// finishes on 958 as theirs does, re-making The Monarch `1/28` as The
    /// Despot; run1413b's coverage enters `count_discovered` first on 557,
    /// the frame who=2's job starts.
    pub(crate) fn held_elsewhere(&self, t: tech::TypeId) -> Vec<production::Adjust> {
        if !self.tech_tree.kind(t).is_plain_tech() {
            return Vec::new();
        }
        let held = self
            .tech
            .iter()
            .take(self.players.len().min(8))
            .filter(|p| p.tech[t])
            .count() as i32;
        if held == 0 {
            return Vec::new();
        }
        let players = self.players.len() as i32;
        vec![production::Adjust::Ratio(players - held + 1, players + 1)]
    }

    /// **The epoch's catch-up term** of `ObjectData::train_time@006508c0`'s
    /// research block (`LAB_00651510`, the decompile's lines 381–408): an
    /// epoch (`0x227..0x242`) counts, over the eight leader slots in use
    /// (`leader_flags & 1`), the leaders whose `ages + epochs` exceed this
    /// leader's by more than three — 1 for an ally (`LeaderData::is_team`,
    /// which in play is `is_ally`), 2 for anyone else — and with `n` of
    /// them takes `t × (2N − n + 2) / (2N + 2)`, `N` the game's
    /// `num_nations`. It runs ahead of the Greeks' divide, the
    /// held-elsewhere term's place: the two are never on one tech. Read
    /// live, like the rest of the block, so the frame another leader gains
    /// its fourth epoch ahead shortens a job already running.
    ///
    /// The second melee (item 3790, `docs/GOLDEN.md` §150.4): who=1 holds
    /// eight epochs from 1004 against who=2's four, so who=2's Military
    /// epoch runs at 6/8 and its Civic epoch, City State, lands before the
    /// Stable `2/2012` finishes — the Civic sweep pulls the Stable into
    /// Thebes, and its activation on 1279 flags the city's roads.
    ///
    /// SEAM: `is_team`'s team-style-7 arm (a seat on team 8 is nobody's
    /// team) is not carried; the alliance test answers for it.
    pub(crate) fn epoch_catch_up(&self, who: Player, t: tech::TypeId) -> Vec<production::Adjust> {
        if !matches!(self.tech_tree.kind(t), tech::Kind::Epoch { .. }) {
            return Vec::new();
        }
        let me = &self.tech[who as usize];
        let mine = me.ages + me.epochs;
        let n: i32 = (0..self.players.len().min(8))
            .filter(|&p| {
                let o = &self.tech[p];
                mine + 3 < o.ages + o.epochs
            })
            .map(|p| if self.is_ally(p as Player, who) { 1 } else { 2 })
            .sum();
        if n == 0 {
            return Vec::new();
        }
        let nations = 2 * self.players.len() as i32;
        vec![production::Adjust::Ratio(nations - n + 2, nations + 2)]
    }

    /// **The age's catch-up term** of `ObjectData::train_time@006508c0`'s
    /// research block (`LAB_00651454`, the decompile's lines 424–449): an
    /// age (`0x220..0x226`) counts, over the eight leader slots in use, the
    /// leaders whose `ages + ages_queued` exceed this leader's by more than
    /// one — 1 for an ally, 2 for anyone else, as the epoch's — and with `n`
    /// of them takes `t × (2N − n + 2) / (2N + 2)`, then falls to the
    /// Greeks' arm. The epoch's term reads `ages + epochs` and no queued
    /// count; this one reads the queued ages on both sides of the test, so
    /// another leader's age queued shortens a job already running, and its
    /// cancel lengthens it again (item 3982, `docs/AI.md` §414).
    ///
    /// SEAM: `is_team`'s team-style-7 arm, as the epoch's.
    pub(crate) fn age_catch_up(&self, who: Player, t: tech::TypeId) -> Vec<production::Adjust> {
        if !matches!(self.tech_tree.kind(t), tech::Kind::Age(_)) {
            return Vec::new();
        }
        let me = &self.tech[who as usize];
        let mine = me.ages + i32::from(me.ages_queued);
        let n: i32 = (0..self.players.len().min(8))
            .filter(|&p| {
                let o = &self.tech[p];
                mine + 1 < o.ages + i32::from(o.ages_queued)
            })
            .map(|p| if self.is_ally(p as Player, who) { 1 } else { 2 })
            .sum();
        if n == 0 {
            return Vec::new();
        }
        let nations = 2 * self.players.len() as i32;
        vec![production::Adjust::Ratio(nations - n + 2, nations + 2)]
    }

    /// **The Greeks' arm** of `ObjectData::train_time@006508c0`'s research
    /// block (`LAB_006515d7`, `:453`–`458`): for a leader with
    /// `has_tribe_bonus(5)`, `t × 100 / (GREEK_RESEARCH_SPEED + 100)`. The
    /// label is reached from the age branch (`0x220..0x226`, after its
    /// `ages_queued` catch-up) and the epoch branch (`0x227..0x242`, after
    /// its `ages + epochs` catch-up) and from nowhere else: a final, a
    /// building tech or a government takes `LAB_006513c0`'s held-elsewhere
    /// term and divides at `LAB_006513e8`, past the arm, and a unit's
    /// research never enters the block. It stands ahead of Relics. run957b's
    /// Greeks hold City State (a Civic epoch) on 176 and their script places
    /// its second city five times there; without the arm ours is still
    /// researching it (item 3326, `docs/AI.md` §266).
    ///
    /// The age's catch-up term ahead of it is built (item 3982), as the
    /// epoch's is (item 3790).
    pub(crate) fn greek_research(&self, who: Player, t: tech::TypeId) -> Vec<production::Adjust> {
        let age_or_epoch = matches!(
            self.tech_tree.kind(t),
            tech::Kind::Age(_) | tech::Kind::Epoch { .. }
        );
        if age_or_epoch
            && self.tech_tree.has_tribe_bonus(
                &self.setup,
                &self.tech[who as usize],
                crate::nations::power::GREEKS,
            )
        {
            vec![production::Adjust::Faster(self.tuning.greek_research_speed)]
        } else {
            Vec::new()
        }
    }

    /// The steps of `ObjectData::train_time@006508c0`'s research block
    /// that stand ahead of the science speedup and that this crate
    /// carries: **Relics** (`rare.ptr[0] & 2`, or `rare_conquest`'s;
    /// `:521`–`524`), `(100 − RELICS_RESEARCH_SPEED) × t / 100`, read live
    /// every frame as the target is. run896b's Spanish Chemistry, queued
    /// at the Library `1/2005` on 14035, finishes before run958e's block
    /// 14376 in the original once who=1 holds Relics, where 35000 without
    /// the term runs past 14382 (item 3318, `docs/AI.md` §264).
    ///
    /// SEAM: the block's other steps — `has_preq(RESEARCH_FASTER)`'s three
    /// quarters, Angkor Wat's `ANGKOR_RESEARCH` (0% shipped), Conquer the
    /// World's cards, the handicap and the lobby's `tech_cost` — are not
    /// carried (the age's and the epoch's catch-up terms are built, items
    /// 3982 and 3790).
    pub(crate) fn research_rares(&self, who: Player) -> Vec<production::Adjust> {
        if self.has_rare(who, economy::RELICS) {
            vec![production::Adjust::Scale(
                100 - self.tuning.relics_research_speed,
            )]
        } else {
            Vec::new()
        }
    }

    /// `LeaderData::researching(t)` as the scripts' `researching_tech` reads
    /// it: queued anywhere by this player.
    pub fn researching(&self, who: Player, t: tech::TypeId) -> bool {
        self.tech[who as usize].queued[t] > 0
    }

    /// Orders a technology at a building — `Build::queue_up` for a tech type.
    ///
    /// The same gates as [`Sim::queue_up`], in the same order: the building
    /// must make it (`can_make` → `queue_here` on the building's tree entry,
    /// and the tree must call it researchable), then the price, then room.
    /// `BuildQueue` holds units and techs in one list, and the library's
    /// fan-out and the stuck-head redirect treat a tech entry as the research
    /// job it is.
    pub fn queue_tech(
        &mut self,
        at: usize,
        t: tech::TypeId,
    ) -> Result<usize, production::QueueFail> {
        self.queue_tech_with(at, t, false)
    }

    /// [`Sim::queue_tech`] with the caller's escrow flag, as
    /// [`Sim::queue_up_with`]: the same `Build::queue_up`.
    pub(crate) fn queue_tech_with(
        &mut self,
        at: usize,
        t: tech::TypeId,
        escrow: bool,
    ) -> Result<usize, production::QueueFail> {
        let at = self.queue_home(at);
        let who = self.buildings[at].owner;
        if !self.buildings[at].active
            || !self.buildings[at].alive
            || self.building_unassimilated(at)
        {
            return Err(production::QueueFail::CantTrain);
        }
        let here = self.buildings[at]
            .ty
            .and_then(|b| self.build_types[b].tree)
            .is_some_and(|b| self.tech_tree.queue_here(b, t));
        // `BuildData::can_make@0062db10`'s tech arm, after `type_avail`: a
        // technology already held (`has_tech`, the bit) or `researching`
        // anywhere is refused (`docs/GOLDEN.md` §35).
        if !here
            || self
                .tech_tree
                .type_avail(&self.setup, &self.tech[who as usize], t, true)
                != tech::AVAILABLE
            || self.tech[who as usize].tech[t]
            || self.researching(who, t)
        {
            return Err(production::QueueFail::CantTrain);
        }
        let charges = self.tech_price(who, t);
        let available = self.holdings[who as usize].available;
        if cost::affordable(&charges, &self.ledgers[who as usize], &available, true) < 1 {
            return Err(production::QueueFail::Cost);
        }
        if !self.buildings[at].queue.has_room() {
            return Err(production::QueueFail::Full);
        }
        cost::pay(
            &charges,
            &mut self.ledgers[who as usize],
            &available,
            escrow,
        );
        let slot = self.buildings[at].queue.push_tech(t, &charges);
        self.tech[who as usize].queued[t] += 1;
        self.track_age_epoch_queued(who, t, 1);
        // `Build::queue_up@00620f40` stamps the leader's `tech_frame` (`+0x7c4`)
        // and its category's `tech_cat_frame[cat]` (`+0x7c8 + 4 cat`) with the
        // frame, for every technology queued (item 1496: run656's block 5
        // holds both at 4, the frame the AI bought Missile Shield).
        let cat = self.tech_cat(t);
        if let Some(ai) = self.ai.get_mut(who as usize) {
            ai.tech_frame = self.frame;
            if let Some(f) = ai.tech_cat_frame.get_mut(cat) {
                *f = self.frame;
            }
        }
        Ok(slot)
    }

    /// Cancels a queued order — `Build::unqueue` with a refund.
    ///
    /// The slot removed is not necessarily the one named: a cancel walks
    /// forward past entries of the same type, so cancelling one of a run
    /// removes the last of it and the one in progress keeps its progress.
    pub fn cancel(&mut self, at: usize, slot: usize) -> Option<production::Item> {
        let at = self.unqueue_home(at, slot);
        let who = self.buildings[at].owner;
        let item = {
            let ledger = &mut self.ledgers[who as usize];
            self.buildings[at].queue.unqueue(slot, true, ledger)?
        };
        if let Some(t) = item.tech {
            self.tech[who as usize].queued[t] -= 1;
            self.track_age_epoch_queued(who, t, -1);
        } else if !item.disband {
            // A `DISBAND` entry's tally is `num_queued[0x29a]`, a craft's,
            // which this crate never raised ([`Sim::queue_disband`]); its
            // `ty` 0 is no unit's (item 3989).
            self.untrack_queued(who, item.ty);
        }
        Some(item)
    }

    /// How long the item in `slot` at building `at` takes, right now.
    ///
    /// Recomputed every frame rather than stored, because the original
    /// recomputes it every frame: a unit of the same type completing
    /// elsewhere lengthens this one mid-build.
    pub fn queue_target(&self, at: usize, slot: usize) -> i32 {
        let b = &self.buildings[at];
        let item = &b.queue.items[slot];
        let who = b.owner;
        if let Some(t) = item.tech {
            return self.tech_time(who, t);
        }
        if item.disband {
            return self.disband_time(at);
        }
        self.unit_train_target(who, item.ty, self.president_near_building(at))
    }

    /// `ObjectData::train_time` of unit type `ty` for `who`, the trainer
    /// reduced to the one thing a unit's clock reads of it: whether The
    /// President stands in reach (`president`). [`Sim::queue_target`]'s
    /// unit arm, and a carrier's plane's clock (item 3981).
    pub(crate) fn unit_train_target(&self, who: Player, ty: usize, president: bool) -> i32 {
        let muster = &self.muster[who as usize];
        let researched = muster.researched[ty];
        // **The Kremlin's Spy** (`train_time@006508c0:344`–`354`), the
        // train block's last step: a type of the Spy line (`is(0x3a, 1)`)
        // whose leader holds the Kremlin (`has_wonder(0x21a)`) returns
        // `ACCEL_TRAIN × 5 − 1` from the whole function — no floor, nothing
        // after it — so the job completes on its sixth call. The Russian
        // pair's Redoubt `1/2034` trains its Spy in 499 on 5646..5652 with
        // who=1's Kremlin `1/2021` standing (item 3527, `docs/AI.md` §304).
        if researched && self.kremlin_trains_fast(who, ty) {
            return self.tuning.accel_train * 5 - 1;
        }
        // **The tail is partitioned and the two halves never mix**
        // (`docs/PRODUCTION.md`, "The tail"): a train job takes the national
        // block and jumps over the science speedup, a research job the
        // reverse.
        //
        // A unit type whose availability bit is *clear* is a research job, and
        // it reaches `train_time`'s research block — so it takes the science
        // speedup and a trained unit does not. Its level is its first
        // prerequisite's `AGE` (`TypeData +0x30`, then `TechTypeData +0x1c8`);
        // a prerequisite that is not a technology has no `AGE` column, and
        // reads as zero here.
        let tail = if researched {
            // **The President arm** (`train_time@006508c0:103`–`107`), after
            // the ramp and the lobby handicap and before the national block:
            // `has_general(0, THEPRESIDENT) >= 0` asked of the trainer, the
            // reach `construct_time`'s clause reads ([`Self::
            // president_near_building`]). The peace pair's Airbase `1/2005`
            // trains its Strategic Bomber in 16500 on 4859, `33000 × 100 /
            // 200`, with who=1's President in reach (run865's packet,
            // `docs/AI.md` §225).
            let mut tail = Vec::new();
            if president {
                tail.push(production::Adjust::Faster(
                    self.tuning.thepresident_unit_build_speed,
                ));
            }
            tail.extend(self.train_tail(who, ty));
            tail
        } else {
            let science = self.tech[who as usize].epoch[tech::Line::Science.index()];
            let level = match self.unit_types[ty]
                .tree
                .map(|id| self.tech_tree.types[id].preq[0])
            {
                Some(tech::Preq::Of(p)) if self.tech_tree.kind(p).is_tech() => {
                    self.tech_tree.types[p].age
                }
                _ => 0,
            };
            let mut tail = self.research_rares(who);
            tail.extend(production::science_speedup(&self.tuning, science, level));
            tail
        };
        // `train_time@006508c0:79`–`85`, before the ramp: a type of the
        // Nuclear Missile line (`is(0x13b, 0)`) of a player holding Uranium
        // divides its base by `(URANIUM_NUKE_SPEED + 100) / 100`
        // (`docs/AI.md` §184).
        let base_faster = if self.is_nuke_line_type(ty) && self.has_rare(who, economy::URANIUM) {
            self.tuning.uranium_nuke_speed
        } else {
            0
        };
        production::train_time_from(
            &self.tuning,
            &self.unit_types[ty].times,
            researched,
            muster.by_type[ty],
            base_faster,
            &tail,
        )
    }

    /// The national block of `ObjectData::train_time`'s tail, in the
    /// original's own order — as far as it is built — and the
    /// speed-upgrade step after it (item 1365).
    ///
    /// **Of the national arms only the British, the German (item 2974), the
    /// Chinese citizen (item 3092), the Japanese (item 3510) and the Mongol
    /// are here**, and that is a scope claim rather
    /// than an oversight: the block's nine other arms are inert in every
    /// capture on disk, so each would be a predicate no diff could check,
    /// and the audit's standing lesson is that predicates are exactly where
    /// a reading goes wrong. The British arm is different — player 1 is
    /// British on both East Indies captures, and `BRITISH_SHIP_SPEED` is
    /// what the AI's Dock clock is measured against.
    ///
    /// The arms that come *before* it in the original — the lobby handicap
    /// and The President — are absent from the captures: the handicap is zero on
    /// both players and the rest are other nations' powers, so starting
    /// the tail here is exact rather than approximate. **The Mongol stable
    /// arm is built** (item 1744), first of the block. Of the ones
    /// after it, the speed-upgrade step is built; the rest are the queue's
    /// own item.
    pub(crate) fn train_tail(&self, who: Player, ty: usize) -> Vec<production::Adjust> {
        let mut tail = Vec::new();
        // **The Mongol stable arm** (`train_time@006508c0:109`–`117`), the
        // first of the national block: a Mongol leader's unit with an attack
        // (`type +0x1e8`) whose trainer is of the Stable line (`is(0x1ac)`,
        // the Auto Plant included) takes `MONGOL_STABLE_SPEED` percent off.
        if self.nation[who as usize].mongols
            && self.unit_types[ty].combat.attack != 0
            && self.unit_types[ty]
                .tree
                .and_then(|t| self.tech_tree.types.get(t))
                .and_then(|d| d.where_)
                .is_some_and(|w| self.tech_tree.is(w, nations::ty::STABLES[0], false))
        {
            tail.push(production::Adjust::Faster(self.tuning.mongol_stable_speed));
        }
        // **The Japanese arms** (`train_time@006508c0:118`–`139`, after the
        // Mongol and before the Chinese): a Japanese leader's unit whose
        // trainer is the Barracks (`0x1ab`) by equality takes
        // `JAPANESE_BARRACKS_SPEED` per tier, `t × 100 / (−k × tier +
        // 100)`; a unit of the Aircraft Carrier's lineage (`is(0x15f, 0)`)
        // `JAPANESE_CARRIER_SPEED`. The Japanese Industrial pair's
        // Riflemen leave Barracks `1/2013` on 1351 there (item 3510,
        // `docs/AI.md` §303).
        if self.japanese(who) {
            if self.trainer_where(ty) == Some(nations::ty::BARRACKS) {
                tail.push(production::Adjust::Faster(
                    self.japanese_per_tier(who, self.tuning.japanese_barracks_speed),
                ));
            }
            if self.unit_types[ty].tree.is_some_and(|t| {
                self.tech_tree.types.get(0x15f).is_some() && self.tech_tree.is(t, 0x15f, false)
            }) {
                tail.push(production::Adjust::Faster(
                    self.tuning.japanese_carrier_speed,
                ));
            }
        }
        // **The Chinese citizen arm** (`train_time@006508c0:140`–`159`,
        // after the Japanese arms and before the British): a Chinese
        // leader's unit that is a `BASE_UNITTYPES`/`PEASANTSKOREAN` citizen
        // (types 0 and 1), a merchant by id, or a caravan (`unit_flags2 &
        // 8`) takes `CHINESE_CITIZEN_CREATION` percent off, and under
        // `CHINESE_CITIZEN_INSTANT` the function returns 1 there — nothing
        // after it, the whole remaining tail included, is applied. The
        // result is clamped at `MIN_TIME` (1), so a lone scale to zero is
        // the return.
        if self.nation[who as usize].chinese {
            let t = &self.unit_types[ty];
            let spared = matches!(ty, 0 | 1)
                || t.tree.is_some_and(|tr| [0x3d, 0x3e, 0x190].contains(&tr))
                || t.cols.flag2(ai_load::uflags2::FLAG2_CARAVAN);
            if spared {
                if self.tuning.chinese_citizen_instant != 0 {
                    return vec![production::Adjust::Scale(0)];
                }
                tail.push(production::Adjust::Faster(
                    self.tuning.chinese_citizen_creation,
                ));
            }
        }
        // `has_tribe_bonus(0xb)`, cached — the lobby's "No Nation Powers"
        // and the no-city gate apply through it (`crate::nations`).
        if self.nation[who as usize].british {
            let t = &self.unit_types[ty];
            if t.combat.domain == attrition::Domain::Sea {
                tail.push(production::Adjust::Faster(self.tuning.british_ship_speed));
            }
            if t.archer {
                tail.push(production::Adjust::Faster(self.tuning.british_archer_speed));
            }
            if t.anti_air {
                tail.push(production::Adjust::Faster(self.tuning.british_aa_speed));
            }
        }
        // **The French arm** (`train_time@006508c0:179`–`192`, after the
        // British and before the German): a French leader's unit whose
        // trainer (`TypeData +0x40`) is, by equality, the Siege Factory or
        // the Factory (0x1ae, 0x1af) takes `FRENCH_SIEGE_SPEED`, `t × 100 /
        // (k + 100)`. The battery's French Siege Factory `1/2026` finishes
        // its type 266 on 13422 there: 30000 × 100 / 125 = 24000 (item
        // 3347, `docs/AI.md` §278). The arm's other branch, a type of class
        // `0x36` taking `FRENCH_SPECIAL_SPEED`, ships as 0 and is not
        // modelled.
        if self.nation[who as usize].french && matches!(self.trainer_where(ty), Some(0x1ae | 0x1af))
        {
            tail.push(production::Adjust::Faster(self.tuning.french_siege_speed));
        }
        // **The German arm** (`train_time@006508c0:193`–`205`, after the
        // French): a German leader's aircraft (`+0x218 == 2`) takes
        // `GERMAN_AIR_SPEED`, and a unit of the Fire Raft line (`is(0x14e,
        // 0)`, through the Submarine) `GERMAN_SUBMARINE_SPEED`, each `t ×
        // 100 / (k + 100)`. The battery's German Dock `1/2010` finishes its
        // Fire Raft on 6718 there (`docs/AI.md` §242).
        if self.nation[who as usize].germans {
            if self.unit_types[ty].combat.domain == attrition::Domain::Air {
                tail.push(production::Adjust::Faster(self.tuning.german_air_speed));
            }
            if self.unit_types[ty].tree.is_some_and(|t| {
                self.tech_tree.types.get(nations::ty::FIRERAFT).is_some()
                    && self.tech_tree.is(t, nations::ty::FIRERAFT, false)
            }) {
                tail.push(production::Adjust::Faster(
                    self.tuning.german_submarine_speed,
                ));
            }
        }
        // **The Roman arm** (`train_time@006508c0:208`–`213`, after the
        // German arms and before `TROOPS_FASTER`): a Legion-line unit of a
        // Roman leader takes `ROMAN_LEGION_FAST`, `t × 100 / (k + 100)`.
        if self.nation[who as usize].romans && self.is_legion_line(ty) {
            tail.push(production::Adjust::Faster(self.tuning.roman_legion_fast));
        }
        // `TROOPS_FASTER` (`role & 0x10000`, three-quarters) follows it:
        // the bonus's prerequisite is `Disable` in the shipped file, so no
        // player holds it.
        //
        // **The speed-upgrade step** (`0065102d`): `t = (10 − n) × t / 10`,
        // `n` the ships' count for a sea unit (`+0x218 == 1`), else the
        // troops' for `obj_masks` `F` or `M`, else the vehicles' for `V` —
        // the first arm that holds and no other (`docs/PRODUCTION.md`, "The
        // tail's first caller").
        let t = &self.unit_types[ty];
        let roles = &self.tech_tree.roles;
        let ladder = if t.combat.domain == attrition::Domain::Sea {
            Some(&roles.ships_speed_preq)
        } else if t.combat.obj_masks & (combat::mask::FOOT | combat::mask::MOUNTED) != 0 {
            Some(&roles.troops_speed_preq)
        } else if t.combat.obj_masks & combat::mask::VEHICLE != 0 {
            Some(&roles.vehicles_speed_preq)
        } else {
            None
        };
        if let Some(ladder) = ladder {
            let n = self.speed_upgrade_level(who, ladder);
            tail.push(production::Adjust::Ratio(10 - n, 10));
        }
        // **The Cotton arm** (`train_time@006508c0:268`–`273`, after the
        // speed-upgrade step and before the spy/general arm): a unit whose
        // trainer (`TypeData +0x40`) is, by equality, the Barracks, the
        // Stable or the Dock (0x1ab, 0x1ac, 0x1b0) takes
        // `COTTON_CREATION_SPEED` when its player holds Cotton (`0x6da6 &
        // 1` or `0x6dce & 1`). The battery's Iroquois Barracks `1/2016`
        // finishes its Javelineers on 8415, the frame who=1 gains Cotton:
        // 18750 × 100 / 125 = 15000 against a clock of 16200 (run940,
        // `docs/AI.md` §256).
        if self.unit_types[ty]
            .tree
            .and_then(|t| self.tech_tree.types.get(t))
            .and_then(|d| d.where_)
            .is_some_and(|w| {
                [
                    nations::ty::BARRACKS,
                    nations::ty::STABLES[0],
                    nations::ty::DOCKS[0],
                ]
                .contains(&w)
            })
            && self.has_rare(who, economy::COTTON)
        {
            tail.push(production::Adjust::Faster(
                self.tuning.cotton_creation_speed,
            ));
        }
        // `SPIES_GENERALS_CREATED_FASTER`: a Spy or a General under the
        // bonus takes half (`train_time@006508c0:271`–`283`, after the
        // cotton arm and before the wool one; item 1584).
        if self.is_spy_or_general(ty)
            && self
                .tech_tree
                .roles
                .spy_general_faster_preq
                .is_some_and(|t| {
                    self.tech_tree
                        .has_tech(&self.setup, &self.tech[who as usize], t)
                })
        {
            tail.push(production::Adjust::Ratio(1, 2));
        }
        // The wool arm, after the spy/general one: a Citizen (`TypeIndex`
        // 0x32 or 0x33, unit types 0 and 1) of a player holding Wool takes
        // `WOOL_CITIZENS_SPEED` percent off (`train_time@006508c0`,
        // `0x6da7 & 8` or `0x6dcf & 8`, the same mask `has_rare` unions;
        // `docs/AI.md` §184).
        if matches!(ty, 0 | 1) && self.has_rare(who, economy::WOOL) {
            tail.push(production::Adjust::Faster(self.tuning.wool_citizens_speed));
        }
        // **The two government arms** (`train_time@006508c0:315`–`339`,
        // after the wool arm; `docs/AI.md` §217). Both test the unit's
        // trainer (`TypeData +0x40`) by equality, never by lineage: a
        // Stable's unit under `MONARCHY_2` takes `MONARCHY_UNIT_PROD2`,
        // else under `MONARCHY_1` `MONARCHY_UNIT_PROD`; a Siege Factory's,
        // an Airbase's or a Dock's under `SOCIALISM_1` takes
        // `SOCIALISM_UNIT_BONUS`.
        let trainer = self.unit_types[ty]
            .tree
            .and_then(|t| self.tech_tree.types.get(t))
            .and_then(|d| d.where_);
        if trainer == Some(nations::ty::STABLES[0])
            && let Some(tier) = self.bonus_tier_held(who, &self.tech_tree.roles.monarchy_preqs)
        {
            tail.push(production::Adjust::Faster(
                self.tuning.monarchy_unit_prod[tier],
            ));
        }
        if trainer.is_some_and(|w| {
            [
                nations::ty::SIEGE_FACTORIES[0],
                nations::ty::AIRBASE,
                nations::ty::DOCKS[0],
            ]
            .contains(&w)
        }) && self
            .bonus_tier_held(
                who,
                std::slice::from_ref(&self.tech_tree.roles.socialism_preqs),
            )
            .is_some()
        {
            tail.push(production::Adjust::Faster(self.tuning.socialism_unit_bonus));
        }
        // **The unit wonders' arm** (`train_time@006508c0:316`–`340`), after
        // the government arms: by the type's domain (`+0x218`), the Statue
        // of Liberty on a land unit, the Porcelain Tower on a ship and the
        // Space Program on an aircraft, each `t × 100 / (k + 100)`. The
        // peace pair's who=1 finishes its Space Program `1/2042` on 5036 and
        // its three air trainers finish that frame: the ICBM at `1/2030` at
        // 38000, the Strategic Bombers at `1/2032` and `1/2037` at 33500
        // and 32700, against a whole bomber's 33750 (`docs/AI.md` §226).
        let held = self.wonders_held(who);
        let wonder = match self.unit_types[ty].combat.domain {
            attrition::Domain::Land => (tech::wonder::STATUE_OF_LIBERTY, self.tuning.liberty_units),
            attrition::Domain::Sea => (tech::wonder::PORCELAIN_TOWER, self.tuning.porcelain_ships),
            attrition::Domain::Air => (tech::wonder::SPACE_PROGRAM, self.tuning.space_air_speed),
        };
        if held & (1 << wonder.0) != 0 {
            tail.push(production::Adjust::Faster(wonder.1));
        }
        // The Kremlin's `ACCEL_TRAIN * 5 - 1` return after it is
        // [`Self::kremlin_trains_fast`], taken in [`Self::queue_target`]. SEAM: its
        // other arm, `has_preq(INSTANT_UNIT_BONUS)` for any unit, is read in
        // `docs/PRODUCTION.md` and not built (`docs/AI.md` §217.4).
        tail
    }

    /// `train_time@006508c0:344`–`349`'s first arm: `is(0x3a, 1)` of the
    /// type — the Spy, by equality first — and `has_wonder(0x21a)`, the
    /// Kremlin, of its leader.
    pub(crate) fn kremlin_trains_fast(&self, who: Player, ty: usize) -> bool {
        const SPY: tech::TypeId = 0x3a;
        self.unit_types[ty]
            .tree
            .is_some_and(|t| self.tech_tree.is(t, SPY, true))
            && self.wonders_held(who) & (1 << tech::wonder::KREMLIN) != 0
    }

    /// `LeaderData::get_ships_speed_upgrade@006da800` and its two
    /// siblings: how many of a ladder's three bonuses the player holds,
    /// every one counted. The listing's `BUY_SELL` arm compares `0x2ad`
    /// against types the loops never reach, so it is dead in all three.
    pub(crate) fn speed_upgrade_level(
        &self,
        who: Player,
        ladder: &[Option<tech::TypeId>; 3],
    ) -> i32 {
        let p = &self.tech[who as usize];
        ladder
            .iter()
            .flatten()
            .filter(|&&t| self.tech_tree.has_tech(&self.setup, p, t))
            .count() as i32
    }

    /// Advances every building's queue by one frame — `Build::do_queue`.
    ///
    /// Three things about the cadence are worth stating because they would be
    /// easy to get plausibly wrong, and one of them was. It runs on **every**
    /// frame, with no phase offset by building index — unlike the per-unit
    /// upkeep in `docs/ATTRITION.md`, which is phased. **Only the first
    /// library fans out**: its queue advances `Muster::library_cities` slots at
    /// once (`LeaderData::get_building_cities`, inside `do_queue`'s library
    /// branch), a non-first library's queue never advances at all, and every
    /// other building advances slot 0 and slot 0 only. (An earlier draft
    /// applied the fan-out to every building.) And **a stuck head is not a
    /// stuck queue**: when slot 0 is done and refused, the first research
    /// entry behind it advances in its place — see [`Sim::advance_slot`].
    #[cfg(test)]
    pub(crate) fn process_queues(&mut self) -> Vec<Produced> {
        let mut out = Vec::new();
        for at in 0..self.buildings.len() {
            self.process_queue(at, &mut out);
        }
        out
    }

    /// One building's `Build::do_queue` (vslot `+0x1b4`), which
    /// `Build::process` calls for that building after its own
    /// `Wall::process` and before the next building's — see the tick.
    pub(crate) fn process_queue(&mut self, at: usize, out: &mut Vec<Produced>) {
        {
            let who = self.buildings[at].owner;
            let queued = self.buildings[at].queue.items.len();
            if queued == 0 {
                return;
            }
            if self.buildings[at].is_library {
                if self.first_library(who) != Some(at) && !self.buildings[at].queue.items[0].disband
                {
                    // `do_queue`'s first gate: a non-first library returns,
                    // unless the slot holds `DISBAND` (`do_queue@0061e410`,
                    // `TVar21 != DISBAND`): a second library's own Disband
                    // runs at 100 a frame (item 3989, run1547's `0/2008`
                    // from 632).
                    return;
                }
                // The library branch recurses into `i + 1` before handling
                // its own slot, so deeper slots complete first. Walking the
                // slots in reverse is that order, and it also keeps the lower
                // indices stable when a deeper entry is removed.
                // `LeaderData::get_building_cities@006e06f0` counts the
                // live cities that hold an active library. Nothing wrote
                // `Muster::library_cities` before item 1388, so every
                // research queue ran one slot at a time; a test that sets
                // the field keeps its value.
                let cities = self
                    .cities_of(who)
                    .into_iter()
                    .filter(|&c| self.count_buildings(c, build::Ident::Library, true) != 0)
                    .count();
                let slots = production::parallel_slots(
                    self.muster[who as usize].library_cities.max(cities),
                    queued,
                )
                .max(1);
                // **But each slot's clock is read before the recursion**
                // (`do_queue:100`, `:283`–`290`, the recursion at `:344`):
                // its `train_time`, its done test and its new counter are
                // computed first, so a deeper entry that completes on the
                // frame — a Science epoch, say — does not shorten a
                // shallower one's target until the next. run963c's Library
                // `1/2005`: Electricity (slot 1) completes on 22935 and the
                // Industrial Age (slot 0) still advances 95300 → 95400
                // under the Science-4 target (item 3329, `docs/AI.md` §267).
                let plans: Vec<_> = (0..slots).map(|slot| self.slot_plan(at, slot)).collect();
                for (slot, plan) in plans.into_iter().enumerate().rev() {
                    if let Advanced::Trained(p) = self.advance_slot_with(at, slot, plan) {
                        out.push(p);
                    }
                }
                return;
            }
            match self.advance_slot(at, 0) {
                Advanced::Trained(p) => out.push(p),
                blocked @ (Advanced::Blocked | Advanced::Limited) => {
                    // The head is done and refused. `do_queue` then asks
                    // `get_next_non_unit` for the first research entry
                    // behind it and advances that one this frame instead. A
                    // train entry behind the head stays put: the cap blocks
                    // every train job, and only train jobs.
                    //
                    //
                    // **The second fallback** (`do_queue@0061e410:442`–`447`,
                    // `docs/PRODUCTION.md`): on a *negative* answer only —
                    // the aircraft limit, a full Airbase's pad — with two or
                    // more entries and no research behind the head, an
                    // Airbase runs `get_next_helicopter`'s slot, the first
                    // behind the head whose type `is(HELICOPTER)`
                    // (`0x136`), so a Transport Helicopter queued behind a
                    // Bomber the pad refuses still trains (run790's `1/2011`
                    // on 3767). Every other building runs
                    // `BuildQueueData::get_next_non_caravan@00630c20`'s
                    // slot, the first behind the head whose type is not
                    // `CARA` (`0x3b`) itself — run834's Market `1/2019`
                    // trains the Merchant behind a Caravan the limit holds,
                    // from 2996.
                    let researched = &self.muster[who as usize].researched;
                    let mut next = self.buildings[at].queue.next_research(|ty| !researched[ty]);
                    if next.is_none()
                        && matches!(blocked, Advanced::Limited)
                        && self.buildings[at].queue.items.len() >= 2
                    {
                        let airbase = self.building_ident(at) == build::Ident::Airbase;
                        next = (1..self.buildings[at].queue.items.len()).find(|&i| {
                            let it = &self.buildings[at].queue.items[i];
                            if airbase {
                                it.tech.is_none()
                                    && self.unit_types[it.ty].cols.unit_flags
                                        & ai_load::uflags::FLAG_HELICOPTER
                                        != 0
                            } else {
                                it.tech.is_some() || self.unit_types[it.ty].type_index != 0x3b
                            }
                        });
                    }
                    if let Some(slot) = next
                        && let Advanced::Trained(p) = self.advance_slot(at, slot)
                    {
                        out.push(p);
                    }
                }
                Advanced::Pending | Advanced::Researched => {}
            }
        }
    }

    /// One queue slot, for one frame.
    ///
    /// A slot that is done is offered to [`production::finished`]. A research
    /// entry — a unit type whose availability bit is clear — completes through
    /// `Leader::gain_tech`: the bit is set, the entry is removed with no
    /// refund, and **no unit is placed**; the player queues again for the
    /// first trained one. (An earlier draft spawned a unit here too.) A train
    /// entry that is refused keeps its full progress and is retried next
    /// frame, which is where a population cap actually bites.
    pub(crate) fn advance_slot(&mut self, at: usize, slot: usize) -> Advanced {
        let plan = self.slot_plan(at, slot);
        self.advance_slot_with(at, slot, plan)
    }

    /// One slot's clock for this frame, `(target, accel)`, read before the
    /// slot is advanced — `do_queue`'s `train_time` and accelerator at its
    /// head — or `None` for a silo that holds it.
    pub(crate) fn slot_plan(&self, at: usize, slot: usize) -> Option<(i32, i32)> {
        let who = self.buildings[at].owner;
        let item = self.buildings[at].queue.items.get(slot)?;
        if let Some(t) = item.tech {
            let accel = production::accel(&self.tuning, production::Job::Research, 1);
            return Some((self.tech_time(who, t), accel));
        }
        // A `DISBAND` entry is not a unit type: `do_queue` takes the
        // research accelerator, 100 a frame on run1512's `0/2007`.
        if item.disband {
            let accel = production::accel(&self.tuning, production::Job::Research, 1);
            return Some((self.disband_time(at), accel));
        }
        let ty = item.ty;
        let researched = self.muster[who as usize].researched[ty];
        // **A silo with a missile inside holds its clock**
        // (`Build::do_queue@0061e410`'s second early return, listing
        // `61e4b4`..`61e510`; `docs/PRODUCTION.md`, "The two early
        // returns"): `is(0x208, 1)`, `inside_down >= 0`, the entry's
        // availability bit set and the entry not `DISBAND` (which this
        // crate never queues) return before the accelerator is chosen. The
        // peace pair's `1/2018` holds its second ICBM at `job_counter` 0
        // from 2582 while the first, `1/50`, stands inside it (run853,
        // `docs/AI.md` §222).
        if researched
            && self.building_ident(at) == build::Ident::MissileSilo
            && !self.buildings[at].garrison.is_empty()
        {
            return None;
        }
        let accel = production::accel(&self.tuning, production::Job::for_unit(researched), 1);
        Some((self.queue_target(at, slot), accel))
    }

    /// [`Sim::advance_slot`] on a clock already read.
    pub(crate) fn advance_slot_with(
        &mut self,
        at: usize,
        slot: usize,
        plan: Option<(i32, i32)>,
    ) -> Advanced {
        let who = self.buildings[at].owner;
        let Some((target, accel)) = plan else {
            return Advanced::Pending;
        };
        if let Some(t) = self.buildings[at].queue.items[slot].tech {
            // A technology entry: research pace, and on completion
            // `Leader::gain_tech` and nothing else — no unit, no refund, no
            // population test.
            let done =
                production::advance(&mut self.buildings[at].queue.items[slot], target, accel);
            if !done {
                return Advanced::Pending;
            }
            let mut ledger = economy::Ledger::default();
            self.buildings[at].queue.unqueue(slot, false, &mut ledger);
            self.tech[who as usize].queued[t] -= 1;
            self.track_age_epoch_queued(who, t, -1);
            self.gain_tech(who, t);
            self.economy_changed(who);
            self.senate_gov_hero(at);
            return Advanced::Researched;
        }
        if self.buildings[at].queue.items[slot].disband {
            // `Build::finished`'s spell arm, then `do_queue`'s unqueue of
            // the entry — which the disband's close has already emptied.
            let done =
                production::advance(&mut self.buildings[at].queue.items[slot], target, accel);
            if !done {
                return Advanced::Pending;
            }
            self.finish_disband(at);
            if self.buildings[at]
                .queue
                .items
                .get(slot)
                .is_some_and(|i| i.disband)
            {
                let mut ledger = economy::Ledger::default();
                self.buildings[at].queue.unqueue(slot, false, &mut ledger);
            }
            return Advanced::Researched;
        }
        let ty = self.buildings[at].queue.items[slot].ty;
        let researched = self.muster[who as usize].researched[ty];
        let done = production::advance(&mut self.buildings[at].queue.items[slot], target, accel);
        if !done {
            return Advanced::Pending;
        }

        // `Build::finished@00628490`'s two negative answers, in its order
        // and both after the population test: a caravan (`unit_flags2 & 8`, `+0x2b8`)
        // at `get_units(type, 0) >= get_caravan_limit(0)`, then the
        // Airbase's pad. Run834's Market `1/2019` holds a done Caravan at
        // 17304 from 2995 to the game's end with `caras` 8 at the limit 8.
        let caravan_full = researched
            && self.unit_types[ty]
                .cols
                .flag2(ai_load::uflags2::FLAG2_CARAVAN)
            && self.unit_types[ty]
                .tree
                .is_some_and(|t| self.line_count(who, t, false) >= self.caravan_cap(who));
        let pad_full = researched && (caravan_full || self.airbase_pad_full(at, ty));
        let muster = &self.muster[who as usize];
        let pop = self.unit_types[ty].price.pop;
        match production::finished(researched, muster.cap, muster.control, pop, pad_full) {
            production::Handover::Population => Advanced::Blocked,
            production::Handover::Limit => Advanced::Limited,
            production::Handover::Researched => {
                // `gain_tech`: the bit, and nothing else. No refund, no
                // skip-forward, no unit, no population.
                //
                // **The gain before the unqueue** (`docs/TECH.md`, "The
                // queue loop"): `Build::do_queue@0061e410` calls vslot
                // `+0x1b0` (`Build::finished`, and so `gain_tech`) at
                // `61ec12` and vslot `+0x1c8` (`Build::unqueue`) only after
                // it returns. So the re-target's `track_queued(t, −1)`
                // finds the research entry's own count still standing, and
                // the unqueue takes it off after. run462's block 10178:
                // the Pikemen research at `1/2020` finished on 9143 with a
                // Hoplites entry behind it (a `jump` match), and who=1's
                // `num_queued[84]` stands at 0 with the re-targeted entry
                // in the queue.
                self.muster[who as usize].researched[ty] = true;
                if let Some(id) = self.unit_types[ty].tree {
                    self.gain_tech(who, id);
                }
                debug_assert_eq!(self.buildings[at].queue.items[slot].ty, ty);
                let mut ledger = economy::Ledger::default();
                self.buildings[at].queue.unqueue(slot, false, &mut ledger);
                self.untrack_queued(who, ty);
                self.economy_changed(who);
                self.senate_gov_hero(at);
                Advanced::Researched
            }
            production::Handover::Trained => {
                // No refund on completion, and no skip-forward: the original
                // passes false for both, and they are the same argument.
                let mut ledger = economy::Ledger::default();
                // `61ec24`: the infinite queue is read before the unqueue.
                let infinite = self.buildings[at].queue.infinite;
                self.buildings[at].queue.unqueue(slot, false, &mut ledger);
                self.untrack_queued(who, ty);
                let trained = self.build_train(at, ty);
                self.requeue_infinite(at, ty, infinite);
                Advanced::Trained(trained)
            }
        }
    }

    /// `Build::finished@00628490`'s tail, after its `gain_tech`: **a
    /// Senate that finishes a research job trains the government's
    /// patriot** when none has been born (`gov_hero_frame < 0`) —
    /// `LeaderData::get_gov_hero`, then `Build::train`, with no queue and
    /// no population test. Great Lakes' Despot `1/79` is this, on tick
    /// 14982: who=1's Senate finishes Despotism (`docs/TECH.md` §"The
    /// government patriot").
    ///
    /// **The other arm** (item 1416): with a patriot already born, the
    /// original asks `ObjectsData::find_unit(Senate x/y, SEARCH_FRIENDLY,
    /// who, −1, 0x20000, FILTER_GOV_HERO)` — the **global** scan, since
    /// `0x20000` is a flag of the sixth argument and the range is −1 — and
    /// `set_type`s the nearest of its own patriots (a later slot on a tie:
    /// the scan keeps `dist <= find_dist`) to the new government's: Great
    /// Sahara at Toughest's Senator `1/80` becomes The President on frame
    /// 11882, Republic (624) giving way to Democracy (626).
    pub(crate) fn senate_gov_hero(&mut self, at: usize) {
        if !self.building_is(at, build::Ident::Senate) {
            return;
        }
        let who = self.buildings[at].owner;
        let p = &self.tech[who as usize];
        let Some(hero) = self.tech_tree.get_gov_hero(&self.setup, p) else {
            return;
        };
        let Some(rec) = self.unit_record(hero) else {
            return;
        };
        if p.gov_hero_frame >= 0 {
            let site = self.buildings[at].pos;
            let mut best: Option<(i32, usize)> = None;
            for u in 0..self.units.len() {
                if !self.units[u].alive() || self.units[u].owner != who {
                    continue;
                }
                let is_patriot = self
                    .unit_tree(u)
                    .is_some_and(|t| self.tech_tree.kind(t).unit().is_some_and(|x| x.patriot));
                if !is_patriot {
                    continue;
                }
                let pos = self.units[u].pos;
                let d = world::vector_dist(pos.x - site.x, pos.y - site.y);
                if best.is_none_or(|(b, _)| d <= b) {
                    best = Some((d, u));
                }
            }
            if let Some((_, u)) = best {
                self.unit_set_type(u, rec);
            }
            return;
        }
        self.build_train(at, rec);
    }

    /// `Build::train@0062f9b0` — a unit born at a building.
    ///
    /// The queue is not in it: `do_queue` has already taken the entry off
    /// and dropped `num_queued` before it calls this, and the two callers
    /// that are *not* a queue — `Build::activate`'s free units
    /// (`crate::nations`) and `Build::finished`'s government hero — pay no
    /// queue at all.
    ///
    /// **The trained unit is born inside its trainer and walks out.**
    /// `Build::train` creates it at the building's own position, calls
    /// `Unit::go_inside`, and then `Unit::come_out` — which is what puts it
    /// on the exit ring rather than on the building's centre tile. The arms
    /// `train` takes before that last call — a gather-inside building that
    /// keeps its worker, a dock's boat count, the player's own text bubble
    /// — are not modelled; every trainer here lets its unit straight out.
    pub(crate) fn build_train(&mut self, at: usize, ty: usize) -> Produced {
        let who = self.buildings[at].owner;
        let unit = self.init_unit(who, ty, self.buildings[at].pos);
        // `Build::train@0062f9b0:86–101`: when the trainer's stance kind is
        // the type's, the building's own byte overrides the one
        // `Unit::init` was born with — for the whole squad, which is one
        // object as far as a stance is concerned. The write is
        // `Unit::set_stance@00605310`, the byte `+0xb1` alone — and that
        // byte *is* the combat stance `get_combat_stance` reads, which this
        // crate keeps again as `combat.stance`, so a combat unit takes both
        // halves (item 3367: a computer's Bark out of a Dock is byte 0 in
        // the original, and this crate left it RAID in `combat.stance`).
        if let Some(stance) = self.trained_stance(ty, at) {
            let combat = (self.type_stance_type(ty) == crate::stance::StanceType::Combat)
                .then(|| crate::group::combat_stance_of(stance))
                .flatten();
            for f in self.squad_members(unit) {
                self.units[f].stance = stance;
                if let Some(c) = combat {
                    self.units[f].combat.stance = c;
                }
            }
        }
        self.go_inside(unit, at);
        // **A scholar trained at a university stays in it** — the `else`
        // arm of `Build::train@0062f9b0`'s exit block, reached when the
        // *trainer* answers `ObjectData::is(0x1a4)`. It re-reads the
        // **trained type's** `is_scholar` (`UnitTypeData +0x4` in
        // `0x34`/`0x35`, `ObjectData::is_scholar@0046d330`), and only then
        // compares `BuildData::gather_max` (`+0x80`, a `char`, settled by
        // the type record) against `ObjectData::num_inside(1)` — the count
        // *after* the unit is in. Over the limit it ejects like anyone
        // else; at or under it calls `check_gatherers` and the scholar
        // never reaches the exit ring.
        //
        // Great Lakes 8285 is what this is: `1/44`, the game's first
        // scholar, was let out on 8272 by this crate and walked back in
        // thirteen frames later, spending
        // [`SITE_GO_INSIDE`](crate::anim::SITE_GO_INSIDE) a second time
        // where the original spends it once (`docs/CITIES.md` §6.5.2).
        // `num_inside(1)` is [`Sim::squads_inside`] here: a scholar's
        // `uber_size` is 1, so the chain count and the captain count are
        // the same number on every path this arm can take.
        //
        // **An aircraft trained at an Airbase stays in it** (item 915,
        // `docs/PRODUCTION.md` "The trained aircraft"): `Build::train`
        // tests the *trainer's* type for `CARRY_AIR` (`+0x1e4 & 0x200`,
        // `62fac0`) before any exit, and that arm calls no `come_out` — the
        // Airbase and the Missile Silo alone carry it ([`Sim::is_hangar`]).
        // With no gather point (`+0xcc` 0, every building in every capture)
        // the unit is left inside with no order, in the base's chain, for
        // `do_launch` to pass over. run308's Biplane `0/9`, inside `0/2007`
        // from 1746.
        //
        // A gather point's arm is below (item 947). **A helicopter trained
        // with no point comes out at once** (item 1019, `docs/PRODUCTION.md`
        // "The Helicopter and the missile under a point"; `62ff59`..
        // `62ff8f` → `62feea`): the list empty and the unit `is(0x136)`,
        // `come_out(0)` with no limit test, which is the invalid point's
        // alone. SEAM: `come_out`'s refusal, on which the unit dies.
        let hangar = self.buildings[at].ty.is_some_and(|t| self.is_hangar(t));
        let inside = self.squads_inside(at);
        let stays = self.building_ident(at) == crate::build::Ident::University
            && self.worker_of(unit) == crate::orders::Worker::Scholar
            && self.buildings[at].gather_max.is_none_or(|m| inside <= m);
        // **A point on the trainer itself keeps the unit in** (item 928,
        // `docs/PRODUCTION.md` "The gather point"; run312's `0/14`, inside
        // `0/2008` from 953): off the University's arm, the trainer's
        // `BuildData::gather_inside` with `num_inside(0)` — the new unit
        // counted — at or under its garrison limit (10 where the limit is
        // 0), and a unit that is not an Aircraft Carrier (`is(0x15f)`), is
        // left inside with no `come_out`. SEAM: the player's text bubble
        // (`MessageWin::add_event`), which is the interface's.
        let university = self.building_ident(at) == crate::build::Ident::University;
        let kept = !university && !hangar && self.gather_inside(at) && {
            let limit = self.garrison_limit(at);
            let limit = if limit == 0 { 10 } else { limit };
            self.num_inside(at) <= limit && self.unit_types[ty].type_index != 0x15f
        };
        if stays {
            self.check_gatherers(at);
        } else if (!hangar && !kept)
            || (hangar
                && self.buildings[at].gather.is_empty()
                && self.air_line_is(unit, crate::airbase::HELICOPTER))
        {
            self.come_out(unit);
        }
        // **Under a gather point the aircraft takes a patrol over the list**
        // (item 947, `docs/PRODUCTION.md` "The gather point"; `62fadf`..
        // `62fbfb`): for a type that is neither a missile (`+0x1e4 &
        // 0x8000000`) nor a helicopter (`+0x2b4 & 0x20`), each point in list order that
        // is on the world gives an order-less unit `add_air_patrol_order(
        // point, this, 1)` — the action bit — and is appended to its patrol
        // after that. The point's `action` is not read here. It stays
        // inside for `do_launch`.
        //
        // The other arm (`62fc47`..), a missile's or a helicopter's, reads
        // the first point alone: [`Sim::train_first_point`] (item 1019).
        if hangar
            && !self.buildings[at].gather.is_empty()
            && !self
                .profile(crate::combat::Obj::Unit(unit))
                .has(crate::combat::mask::MISSILE)
            && !self.is_helicopter(unit)
        {
            let list = self.buildings[at].gather.clone();
            for q in list {
                if !self.in_world(q.pos) {
                    continue;
                }
                if self.units[unit].orders.is_empty() {
                    self.add_air_patrol_order(unit, q.pos, Some(at), true);
                } else if let Some(orders::Body::AirPatrol(ap)) =
                    self.units[unit].orders.front_mut().map(|o| &mut o.body)
                {
                    ap.push(q.pos);
                }
            }
        }
        if hangar
            && let Some(q) = self.first_gather(at)
            && (self
                .profile(crate::combat::Obj::Unit(unit))
                .has(crate::combat::mask::MISSILE)
                || self.is_helicopter(unit))
        {
            self.train_first_point(unit, at, q);
        }
        // **The economy is dirty only where the original says so** (item
        // 1398, `docs/AI.md` §103): `Unit::go_inside@0061a2e0:81` for a
        // Scholar (`0x34`, `0x35`), and `Unit::come_out@00617c10:6180ea` for
        // an exit from a University or an Oil Platform (`is(0x1a4)` or
        // `is(0x1a6)` on the container). Every other trained unit is born
        // clean; `Build::queue_up` raises nothing either.
        // SEAM: `Unit::init`'s Americans' Barracks arm is not modelled
        // (its Lakota arm is built, `docs/AI.md` §219).
        let container = matches!(
            self.building_ident(at),
            crate::build::Ident::University | crate::build::Ident::OilPlatform
        );
        if matches!(self.unit_types[ty].type_index, 0x34 | 0x35) || container {
            self.economy_changed(who);
        }
        Produced { unit, ty, at }
    }

    /// Step 8's object half (`docs/TECH.md`, `Leader::gain_tech@006dcb60`,
    /// the loop at `6dde64`–`6ddf2b`): every in-use building of the leader
    /// whose type's `upgrade` is `t` becomes `t` through `Wall::set_type(t,
    /// 0)`. A Tower stands as a Keep from the frame its leader gains the
    /// Keep, and every count keyed on its type moves with it — the
    /// original's `num_buildings`, which `create_buildings` reads (item
    /// 1264). The test is the in-use flag alone: a site converts too.
    pub(crate) fn upgrade_buildings_to(&mut self, who: Player, t: tech::TypeId) {
        let Some(rec) = self.build_record(t) else {
            return;
        };
        for b in 0..self.buildings.len() {
            let bd = &self.buildings[b];
            if !bd.alive || bd.owner != who {
                continue;
            }
            let upgrades = bd
                .ty
                .and_then(|r| self.build_types[r].tree)
                .is_some_and(|x| self.tech_tree.types[x].upgrade == Some(t));
            if upgrades {
                self.set_type(b, rec);
            }
        }
    }

    /// `Leader::gain_tech@006dcb60`'s unit-conversion loop, the **object**
    /// half of `docs/TECH.md` §7 (`6dd9bd`–`6ddbd1`).
    ///
    /// Gaining a unit type converts what you already have. The loop walks
    /// the player's object slots in order and takes every live unit whose
    /// own type is `get_graft(t.from)`, or whose `jump` chain reaches `t`,
    /// and calls `Unit::set_type(t, 0)` on it. That is how run53's three
    /// Bowmen objects become Archers on frame 6736, the frame the AI's
    /// Classical Age hands its British owner a free archer upgrade
    /// (§13's `BRITISH_ARCHER_UPGRADES` block).
    ///
    /// **The squad-size shrink** (`6ddaf0`): when the new type's
    /// `uber_size` is smaller than the old one's, a new size of **1** kills
    /// every member that is not the captain (`Unit::die`, vslot `+0x158`)
    /// and gives the captain the squad's `total_damage`; any other
    /// shrink is an error box in the original — "No support for decreasing
    /// number of guys in a squad to anything other than 2" — so it cannot
    /// be reproduced and is left alone here.
    ///
    /// SEAM: the carrier arm above it (`6dd9ec`), which moves a carrier's
    /// own `num_queued` from the old type to the new instead of converting
    /// it — `types[t].is(0x134, 0) && u.is(0x15f, 0)` and two
    /// `track_queued` calls; no capture has a carrier. The **buildings'**
    /// queues are the loop after this one, [`Sim::retarget_queued_to`].
    pub(crate) fn upgrade_units_to(&mut self, who: Player, t: tech::TypeId) {
        let Some(rec) = self.unit_record(t) else {
            return;
        };
        let from = self.tech_tree.get_graft(
            &self.setup,
            &self.tech[who as usize],
            self.tech_tree.types[t].from,
        );
        let mut list: Vec<usize> = (0..self.units.len())
            .filter(|&u| self.units[u].alive() && self.units[u].owner == who)
            .collect();
        list.sort_by_key(|&u| self.units[u].index);
        let new_uber = self.unit_types[rec].combat.uber_size.max(1);
        for u in list {
            let Some(ty) = self.unit_tree(u) else {
                continue;
            };
            if ty == t {
                continue;
            }
            let hit = Some(ty) == from
                || self
                    .tech_tree
                    .jumps_to(&self.setup, &self.tech[who as usize], ty, t);
            if !hit {
                continue;
            }
            let old_uber = self.units[u]
                .ty
                .map_or(1, |o| self.unit_types[o].combat.uber_size.max(1));
            if new_uber < old_uber {
                if new_uber != 1 {
                    // The original's own error box: not reproducible.
                    continue;
                }
                if !self.units[u].captain {
                    self.units[u].health = 0;
                    // `Unit::die`: `Unit::close`'s orders, and
                    // `Object::close`'s hold (§59.3).
                    self.close_dead_orders(u);
                    self.hold_dead_slot(u);
                    continue;
                }
                let damage: i32 = self
                    .squad_members(u)
                    .iter()
                    .map(|&f| self.units[f].max_health - self.units[f].health)
                    .sum();
                self.units[u].health = (self.units[u].max_health - damage).max(1);
            }
            self.unit_set_type(u, rec);
        }
    }

    /// `Leader::gain_tech@006dcb60`'s **queue** loop (`6ddbed`..`6ddd8e`),
    /// after the object loop and under the same `upgrade_units` flag: every
    /// live building of the player, in object order, and every entry of its
    /// queue that is a unit type `q` with `q == get_graft(t.from)`, or whose
    /// grafted `jump` chain reaches `t`, is re-targeted in place —
    /// `BuildQueue::set_queue(i, t, NULL, 1)`, which writes the type and
    /// keeps the entry's progress and its recorded price — between
    /// `track_queued(·, −1)` and `track_queued(t, +1)`. Nothing is paid or
    /// refunded (`docs/GOLDEN.md` §36, the emulator's rows b to f; run300's
    /// 922, the Slingers entry `[83 at 0]` with its 46/46 kept).
    ///
    /// **The decrement names the walker, not the entry**: on a match by the
    /// `jump` chain the register holding `q` has been overwritten by the walk,
    /// so `track_queued(t, −1)` runs where `(q, −1)` was meant (`6ddcf4`'s
    /// `push esi`; the emulator's row f). A match by `from` decrements `q`.
    /// `track_queued` never takes a count below zero.
    pub(crate) fn retarget_queued_to(&mut self, who: Player, t: tech::TypeId) {
        let Some(rec) = self.unit_record(t) else {
            return;
        };
        let w = who as usize;
        let from =
            self.tech_tree
                .get_graft(&self.setup, &self.tech[w], self.tech_tree.types[t].from);
        let mut list: Vec<usize> = (0..self.buildings.len())
            .filter(|&b| self.buildings[b].alive && self.buildings[b].owner == who)
            .collect();
        list.sort_by_key(|&b| self.buildings[b].index);
        for b in list {
            for i in 0..self.buildings[b].queue.items.len() {
                let item = &self.buildings[b].queue.items[i];
                if item.tech.is_some() {
                    continue;
                }
                let q_rec = item.ty;
                let Some(q) = self.unit_types[q_rec].tree else {
                    continue;
                };
                let dec = if Some(q) == from {
                    q_rec
                } else if self.tech_tree.jumps_to(&self.setup, &self.tech[w], q, t) {
                    rec
                } else {
                    continue;
                };
                self.untrack_queued(who, dec);
                self.buildings[b].queue.items[i].ty = rec;
                self.muster[w].queued_by_type[rec] += 1;
                self.track_tree_queued(who, rec, 1);
            }
        }
    }

    /// [`Sim::track_tree_queued`], reachable from a test — the ramp's
    /// group counter has six writers and this is how one of them is
    /// exercised without standing a whole city up.
    #[cfg(test)]
    pub(crate) fn track_queued_for_test(&mut self, who: Player, ty: usize, delta: i32) {
        self.track_tree_queued(who, ty, delta);
    }

    /// The tree's own queued count for the government-pairing rule, and the
    /// ramp's group count beside it.
    pub(crate) fn track_tree_queued(&mut self, who: Player, ty: usize, delta: i32) {
        if let Some(id) = self.unit_types[ty].tree {
            self.tech[who as usize].queued[id] += delta;
        }
        // The ramp's group half, kept on the same statement as the type
        // half it sits beside ([`Muster::queued_by_group`]). Every site
        // that moves `queued_by_type` calls this, which is what makes the
        // two counts agree by construction rather than by inspection.
        if let Some(g) = self.unit_types[ty].group {
            self.muster[who as usize].queued_by_group[g] += delta;
        }
    }

    /// A queue entry's own count taken down, as `Build::clean_queue
    /// @00620b60:75`–`:77` does by the entry's type: a technology's
    /// `num_queued` is its own (`PlayerTech::queued`), a unit's goes through
    /// [`Sim::untrack_queued`], and a Disband's is not one this crate keeps.
    /// The close loops took a technology off the Citizen's count instead
    /// (item 3969; parked T71-7).
    pub(crate) fn untrack_item(&mut self, who: Player, item: &production::Item) {
        if item.disband {
            return;
        }
        if let Some(t) = item.tech {
            let q = &mut self.tech[who as usize].queued[t];
            if *q != 0 {
                *q -= 1;
            }
            self.track_age_epoch_queued(who, t, -1);
        } else {
            self.untrack_queued(who, item.ty);
        }
    }

    /// **`ages_queued` and `epochs_queued`** (`LeaderData +0x67f4`,
    /// `+0x67f5`; item 3982, `docs/AI.md` §414): the byte for an age
    /// (`0x220..0x226`) or an epoch (`0x227..0x242`) moved by `delta` beside
    /// the technology's own `num_queued` — raised by `Build::queue_up
    /// @00620f40:518`–`:541`, taken down by `Build::unqueue@006207c0:139`–
    /// `:164` (a cancel, and a finished research) and `Build::clean_queue
    /// @00620b60:132`–`:158`. The decrement is a bare byte `− 1`, no zero
    /// test, where `num_queued`'s beside it has one; a library's hand-off
    /// moves the entries and neither count.
    pub(crate) fn track_age_epoch_queued(&mut self, who: Player, t: tech::TypeId, delta: i8) {
        let p = &mut self.tech[who as usize];
        let c = match self.tech_tree.types.get(t).map(|d| d.kind) {
            Some(tech::Kind::Age(_)) => &mut p.ages_queued,
            Some(tech::Kind::Epoch { .. }) => &mut p.epochs_queued,
            _ => return,
        };
        *c = c.wrapping_add_signed(delta);
    }

    /// **One off the queued count, never below zero** — the decrement
    /// `Build::unqueue@006207c0` and `Build::clean_queue@00620b60` inline
    /// and `Leader::track_queued@006e0f30`'s `−1`: `num_queued[t]`
    /// (`+0x5a22`, the absolute `0xe3fdb2` plus the stride) comes off only
    /// when it is not zero, and each of the per-building tallies beside it
    /// (`+0xa10`..`+0xa24`) on the same test. Every site that took a queued
    /// entry off unguarded now comes here (`docs/TECH.md`, "The queue
    /// loop").
    pub(crate) fn untrack_queued(&mut self, who: Player, ty: usize) {
        let w = who as usize;
        if self.muster[w].queued_by_type[ty] != 0 {
            self.muster[w].queued_by_type[ty] -= 1;
        }
        if let Some(id) = self.unit_types[ty].tree
            && self.tech[w].queued[id] != 0
        {
            self.tech[w].queued[id] -= 1;
        }
        if let Some(g) = self.unit_types[ty].group
            && self.muster[w].queued_by_group[g] != 0
        {
            self.muster[w].queued_by_group[g] -= 1;
        }
    }

    /// Builds one unit of a type, if the player can pay for it and has room.
    ///
    /// This is the *instant* path — the price and the unit in one call, with
    /// no queue and no time. [`Sim::queue_up`] is the ordinary one.
    ///
    /// The order is the original's and it is observable: **the population is
    /// checked before the price**, so a player at the cap is refused without
    /// being charged. Everything after that is bookkeeping the ramp depends
    /// on — a count that is not incremented leaves the next one just as cheap.
    pub fn produce(&mut self, who: Player, ty: usize, pos: Pos) -> Result<usize, Refused> {
        let charges = self.price_of(who, ty);
        let pop = self.unit_types[ty].price.pop;
        let muster = &self.muster[who as usize];
        if cost::exceeds_population(muster.cap, muster.control, pop) {
            return Err(Refused::Population);
        }

        let available = self.holdings[who as usize].available;
        if !cost::can_pay(&charges, &self.ledgers[who as usize], &available, 1) {
            return Err(Refused::Cost);
        }
        cost::pay(&charges, &mut self.ledgers[who as usize], &available, false);

        self.track_unit_type(who, ty, 1);

        let index = self
            .find_free(who, UNIT_BASE, BUILD_BASE)
            .unwrap_or(i16::MAX);
        let pos = Pos::new(gaia::init_snap(pos.x), gaia::init_snap(pos.y));
        let mut unit = Unit::new(who, index, pos, self.unit_types[ty].hits);
        unit.kind = self.unit_types[ty].kind;
        unit.ty = Some(ty);
        let at = self.add_unit(unit);
        self.init_guys(at, Some(ty));
        self.economy_changed(who);
        Ok(at)
    }
}
