//! Tech gain: the tree, the starting techs and goods, `gain_tech` and what it
//! sets off, and the availability it answers (`docs/TECH.md`;
//! `docs/STRUCTURE.md` §3, `tech/`).
//!
//! Moved out of `lib.rs` unchanged by item 4014 (`docs/STRUCTURE.md` §5, move 6).

use crate::*;

impl Sim {
    /// Installs a tech tree. Every player's tech state is reset to empty
    /// against it; call [`Sim::start_techs`] for each to lay down the starting
    /// position the way `Leader::init` does.
    pub fn set_tech_tree(&mut self, tree: tech::TechTree) {
        self.tables.tech_tree = tree;
        for t in &mut self.tech {
            let (tribe, power, team) = (t.tribe, t.power, t.team);
            *t = tech::PlayerTech::new(&self.tables.tech_tree);
            t.tribe = tribe;
            t.power = power;
            t.team = team;
        }
        self.sync_researched();
        // Availability is a fact about the tree, so it is wrong until there
        // is one: every player's two arrays are laid down here and then
        // maintained by [`Sim::apply_gained`].
        for who in 0..self.economy.holdings.len().min(self.tech.len()) {
            self.sync_goods_available(u8::try_from(who).expect("too many players"));
        }
    }

    /// `Leader::init`'s tech block for one player: the ages below the
    /// starting age, the building techs that need only ages, every building
    /// and unit type whose prerequisites hold, and the nation's free starting
    /// epoch — see `docs/TECH.md`, "The starting position".
    pub fn start_techs(&mut self, who: Player) -> Vec<tech::Gained> {
        let events = self.tables.tech_tree.start(
            &self.tables.setup,
            &self.tables.tuning,
            &mut self.tech[who as usize],
        );
        self.apply_gained(who);
        self.lay_starting_goods(who);
        events
    }

    /// `Leader::init@006e3930`'s last loop: the opening stockpile.
    ///
    /// The loop **zeroes all six buckets and then pays only the goods that
    /// are already available** — `type_avail(g, 1) != 0`, which for a good
    /// is `has_preq` alone (`docs/TECH.md`, "Two questions this answers").
    /// In the Ancient age that is food, timber and wealth; knowledge and
    /// metal wait for the Classical age and oil for the Industrial one, and
    /// each is paid the moment its age arrives — [`Sim::gain_tech`]'s half
    /// of the same rule. `docs/COSTS.md`, "The starting grant arrives with
    /// the good".
    ///
    /// Two nation terms ride in the same loop: the **Persians** scale the
    /// food grant by `(PERSIANS_BONUS_FOOD + 100) / 100`, and the
    /// **Greeks** — who hold knowledge from frame 0 through
    /// `GREEK_KNOWLEDGE_EARLY`, and so are paid here — have it taken
    /// straight back off them by `GREEK_DELAY_KNOWLEDGE` and re-granted at
    /// the Classical age. **Only in the Ancient age**: the original's
    /// branch also asks the leader's `ages` (`+0xdc`, key `0x62766`) for 0,
    /// so a Greek leader started at Classical or later keeps it (item 3566,
    /// `docs/AI.md` §310).
    ///
    /// It **assigns**, so re-laying it is what the original's single call
    /// does: a leader whose nation or lobby is only known after the first
    /// `start_techs` gets the opening bucket laid down again, exactly as
    /// the harness re-lays the opening tech set.
    pub fn lay_starting_goods(&mut self, who: Player) {
        let w = who as usize;
        let greek = self
            .tables
            .tech_tree
            .has_tribe_bonus(&self.tables.setup, &self.tech[w], 5)
            && self.tech[w].ages == 0
            && self.tables.tuning.greek_delay_knowledge != 0;
        let persian = self
            .tables
            .tech_tree
            .has_tribe_bonus(&self.tables.setup, &self.tech[w], 23);
        for g in 0..economy::RESOURCES {
            let amount = match self.good_tree_type(g) {
                // Without a tree there is no availability to test, and the
                // convention the rest of the simulation keeps
                // ([`economy::Holdings::new`]) is that everything is
                // available.
                None => self.starting_good(who, g),
                Some(t) if self.type_avail(who, t) != tech::NOT_AVAILABLE => {
                    self.starting_good(who, g)
                }
                Some(_) => 0,
            };
            let amount = match g {
                0 if persian => amount * (self.tables.tuning.persians_bonus_food + 100) / 100,
                3 if greek => 0,
                _ => amount,
            };
            self.economy.ledgers[w].bucket[g] = amount;
        }
        self.economy.ledgers[w].dirty = true;
    }

    /// **The `resource` cheat** (`ConsoleWin::run_cmd@007d6a70`'s case,
    /// the listing `7dd7d8`..`7dd8b2`; `docs/GOLDEN.md` §44): `bucket_set(g,
    /// max(0, bucket_get(g) + amount))`, the bucket written as it stands,
    /// with no escrow, income or cap between. One good (`Some(g)`) is
    /// written whatever its availability; `all` (`None`) walks the six
    /// basic goods and writes only those `type_avail` answers non-zero for
    /// (`7dd825`), the rule [`Sim::lay_starting_goods`] pays the opening
    /// stockpile by. An amount of 0 is refused before either arm (`je` at
    /// `7dd7e8`) and prints the totals.
    pub fn cheat_resource(&mut self, who: Player, good: Option<usize>, amount: i32) {
        let w = who as usize;
        for g in 0..economy::RESOURCES {
            let hit = match good {
                Some(one) => one == g,
                None => self
                    .good_tree_type(g)
                    .is_none_or(|t| self.type_avail(who, t) != tech::NOT_AVAILABLE),
            };
            if hit {
                let b = &mut self.economy.ledgers[w].bucket[g];
                *b = (*b + amount).max(0);
            }
        }
        self.economy_changed(who);
    }

    /// `game->starting[g]` as one leader is paid it — the amount both
    /// `Leader::init@006e3930` and `Leader::gain_tech@006dcb60` hand to
    /// `bucket_add`, under the same three-armed scale.
    ///
    /// `Game::init_starting_resources@0058a500` fills the array before the
    /// first frame (`docs/ORDERS.md` §9.4): the base is `STARTING_GOODS[g]`,
    /// or `STARTING_GOODS[0]` for every good but knowledge when the lobby is
    /// row 7; the lobby's `startingresources` row gives a `lo`/`hi` pair;
    /// `lo == 0` halves the base, else the grant is `lo x base` plus, for a
    /// row with a spread, `rand % ((hi - lo) x base)` from the sync stream.
    /// **Row 1 and Deathmatch (row 7, `100`/`100`) are measured**, run40's
    /// forty frames and run651's block 1 (`docs/AI.md` §129): neither has a
    /// spread, so neither draws. **A spread row (9..11) is not modelled**:
    /// its draws are spent before the first dumped frame and the amount
    /// they chose is in no record, so it pays `lo x base`. Row 8, the
    /// unlimited lobby, is the one other arm that is read, and both
    /// callers *assign* 99,999 after the add.
    ///
    /// Conquer the World's `ctw_nomad_starting_res_x` is the third arm and
    /// is cut from v1.
    pub(crate) fn starting_good(&self, who: Player, g: usize) -> i32 {
        if self.ai.lobby.resources_unlimited() {
            return economy::UNLIMITED_GOODS;
        }
        let mut base = self.tables.tuning.starting_goods[g];
        if self.ai.lobby.starting_resources == 7 && g != 3 {
            base = self.tables.tuning.starting_goods[0];
        }
        let (lo, _hi) = self.ai.lobby.starting_resources_row;
        let amount = if lo == 0 { base / 2 } else { lo * base };
        // Barbarians at the Gates pays the defending team the *other* row's
        // index as a multiplier, which is what both callers write.
        if self.ai.lobby.game_rules == 8 && self.tech[who as usize].team == 0 {
            return (self.ai.lobby.starting_resources2 + 1) * amount;
        }
        amount
    }

    /// The tree entry of basic good `g`, if the tree has one: the goods come
    /// out of `resourcerules.xml` in order and the first six are
    /// [`economy::Resource`]'s own.
    pub(crate) fn good_tree_type(&self, g: usize) -> Option<tech::TypeId> {
        self.tables
            .tech_tree
            .types
            .iter()
            .enumerate()
            .filter(|(_, d)| matches!(d.kind, tech::Kind::Good))
            .map(|(i, _)| i)
            .nth(g)
            .filter(|_| g < economy::RESOURCES)
    }

    /// `Leader::gain_tech`, as far as the simulation acts on it: the tree's
    /// bits and counters with their cascades, then the researched bit of every
    /// unit type the gain reached, and the population cap when a Military
    /// epoch arrived. The events come back so a caller can see what cascaded;
    /// `docs/TECH.md`, "`gain_tech`: owning it".
    pub fn gain_tech(&mut self, who: Player, t: tech::TypeId) -> Vec<tech::Gained> {
        let frame = self.frame;
        // `Leader::gain_tech@006dcb60` line 189, *before* the epoch counter
        // it is about to raise: a Science level re-prices every technology
        // waiting in the player's first library.
        self.reprice_library(who, t);
        // And `has_preq` on the six goods, read into a stack array at line
        // 297 — *before* the bit is set, which is what makes the grant
        // below fire once and only for a good the leader did not already
        // hold the prerequisite for.
        let had_preq: [bool; economy::RESOURCES] = std::array::from_fn(|g| {
            self.good_tree_type(g).is_some_and(|x| {
                self.tables
                    .tech_tree
                    .has_preq(&self.tables.setup, &self.tech[who as usize], x)
            })
        });
        // `LeaderData::has_wonder` is asked live by the wonder-gated blocks
        // at the end of `gain_tech` (`docs/NATIONS.md` §7, item 4): feed the
        // player's set off the buildings first.
        let held = self.wonders_held(who);
        self.tech[who as usize].wonders = self
            .tables
            .tech_tree
            .roles
            .wonder_line
            .iter()
            .enumerate()
            .filter(|&(k, _)| held & (1 << k) != 0)
            .map(|(_, &w)| w)
            .collect();
        let events = self.tables.tech_tree.gain_tech(
            &self.tables.setup,
            &mut self.tech[who as usize],
            t,
            frame,
        );
        self.pay_arriving_goods(who, &had_preq, &events);
        // Step 9's government arm (`006de29e`–`006de2e4`): with `gov`
        // written, a gained `is(0x274, 0)` — Capitalism — after frame 0
        // takes `CAPITALISM_OIL_GIFT` into the oil bucket (`bucket_add`, a
        // plain add). `0x274` is `gov_bonuses`' second row's government.
        if frame != 0
            && matches!(self.tables.tech_tree.kind(t), tech::Kind::Gov { .. })
            && self.tables.tech_tree.roles.gov_bonuses.get(1).map(|g| g.1) == Some(t)
        {
            self.economy.ledgers[who as usize].bucket[economy::Resource::Oil.index()] +=
                self.tables.tuning.capitalism_oil_gift;
        }
        // `*this |= 0x2000000` at line 319, on every gain: the economy is
        // reassembled at the next dirty-grid frame, which is what reads a
        // Commerce level into the caps.
        self.economy_changed(who);
        // `Object::update_hits`'s Citizen arm reads the Militia line's bits
        // (`docs/GOLDEN.md` §48), and **it is not called here**:
        // `gain_tech@006dcb60:661` raises `0x4000000` for a unit type, and
        // the Citizens' pools move at the next `Leader::process`'s
        // `calc_unit_stats` ([`Sim::apply_gained`]'s flag). run912c: who=1's
        // Tower `1/2014` finishes the Militia on block 12562 (`job_counter`
        // 18000, the slot empty on 12563) and the thirty Citizens are at 40
        // on 12563 and at 50 on 12564 (`docs/AI.md` §244).
        // Step 7's **object** half, in the order the cascade set the bits:
        // every standing unit of the line converts in place.
        for e in &events {
            match *e {
                tech::Gained::UnitUpgrade { to } => {
                    self.upgrade_units_to(who, to);
                    self.retarget_queued_to(who, to);
                }
                tech::Gained::BuildingUpgrade { to } => self.upgrade_buildings_to(who, to),
                _ => {}
            }
        }
        let wall_stats_were = self.cities.wall_stats_dirty[who as usize];
        self.apply_gained(who);
        // **A unit type takes no wall stats** (item 3965): the unit arm
        // (`gain_tech@006dcb60:658`, the type's vslot `+0xc`) raises
        // `0x4000000` alone and leaves by `goto LAB_006e05d9` (`:883`), past
        // the tail's `|= 0xc000000` (`:2313`). run99's Tower `1/2014` gains
        // the Militia on 10163 and keeps its cached `mylos` 11 to 10334's
        // tech, where this crate's refresh read 13 on 10165.
        if self.tables.tech_tree.kind(t).is_unit() {
            self.cities.wall_stats_dirty[who as usize] = wall_stats_were;
        }
        // Step 10's second half: a Civic level re-masks every city and
        // sweeps the buildings its new reach covers into it.
        if matches!(
            self.tables.tech_tree.kind(t),
            tech::Kind::Epoch {
                line: tech::Line::Civic,
                ..
            }
        ) {
            self.civic_epoch_sweep(who);
        }
        // Step 11's city arm: a gain that is a prerequisite of the city line
        // (Medieval for the Large City, Industrial for the Major) runs
        // `City::check_upgrade` on every one of the leader's cities
        // (`0x6dec2f`), so a city that already holds its kinds levels up on
        // the frame the age lands rather than at its next building's
        // completion (`docs/AI.md` §98).
        for _ in events
            .iter()
            .filter(|e| matches!(e, tech::Gained::CityCheck))
        {
            for c in self.cities_of(who) {
                if self.cities.cities[c].alive {
                    self.check_upgrade(c);
                }
            }
        }
        // **An age re-places every one of the leader's units where it
        // already stands** — `Leader::gain_tech@006dcb60:2366`, gated on
        // `TypeData::is_age_type` (the type vtable's `+0x34`, `is_epoch_type`
        // is `+0x38` and does not qualify). The loop is
        // `Unit::update_gpiece` then `Unit::set_new_location(u, u.x, u.y,
        // 1, 1)`, and it is the **snap** flags that matter here rather than
        // the graphic: `param_4` runs `Guy::set_angle(guy 0, unit->angle,
        // 1)`, which writes the figure's own facing outright, and `param_3`
        // puts guy 0 and every tracked crew figure on their points.
        //
        // For a unit standing still, or one already facing the way it is
        // going, this changes nothing — which is why a once-per-age event
        // is invisible almost everywhere. For a unit **mid-turn** it is a
        // free turn: the frame's step has already been taken along the old
        // facing, and the next one starts from the heading.
        //
        // [`Sim::tick`] runs the buildings after the unit loop, so the snap
        // lands after the frame's own step, which is what run86 shows.
        if matches!(self.tables.tech_tree.kind(t), tech::Kind::Age(_)) {
            self.age_snap_units(who);
        }
        // `Leader::gain_tech`'s tail: `check_transport` (`docs/TRANSPORT.md`
        // §4).
        self.check_transport(who);
        events
    }

    /// The age's re-placement loop, for one leader — `Leader::gain_tech`'s
    /// `is_age_type` arm (`docs/TECH.md`, "An age snaps every figure").
    ///
    /// `Unit::set_new_location(u, u.x, u.y, 1, 1)` on a unit that is already
    /// there reduces to three writes: the figure's facing takes the unit's
    /// own `+0x50` (this crate's [`crate::Movement::heading`]), guy 0's body
    /// goes onto the unit, and each tracked crew figure is **put** on its
    /// offset rather than told to walk to it — [`Sim::crew_des`]'s `snap`.
    /// `Guy::set_angle(guy 0, +0x50, 1)` writes the figure and nothing of
    /// the unit's, so the order's `dest_angle` (`+0x58`,
    /// [`crate::Movement::des_angle`]) stands, which is why this is not
    /// [`crate::Movement::set_facing`] (item 1281).
    ///
    /// Before it, `Unit::update_gpiece`: the age coordinate of
    /// `get_unit_gpiece` is `ages < 5 ? ages / 3 : 2`, so the third and the
    /// fifth ages move every figure one bracket up, and the new piece's
    /// lengths are what its next `set_anim` reads. The flags stay the ones
    /// `init_real` took ([`crate::units::anim::Guy::flag_piece`]); `docs/TECH.md`,
    /// "The piece moves with the age".
    pub(crate) fn age_snap_units(&mut self, who: Player) {
        for u in 0..self.units.len() {
            if !self.units[u].alive() || self.units[u].owner != who {
                continue;
            }
            self.update_gpiece(u);
            let pos = self.units[u].pos;
            let angle = self.units[u].movement.heading;
            self.units[u].movement.facing = angle;
            self.units[u].movement.body.pos = pos;
            self.coll_follow(u);
            self.crew_des(u, pos, angle, true);
        }
    }

    /// `Leader::gain_tech@006dcb60`'s goods loop: **the starting grant
    /// arrives with the good, not at `Leader::init`.**
    ///
    /// The original walks the six basic goods on every gain and pays
    /// `bucket_add(g, game->starting[g])` for one that passes both halves
    /// of a two-part gate:
    ///
    /// - the leader did **not** hold `has_preq(g)` before this call — the
    ///   stack array read at line 297, ahead of the bit — which is what
    ///   stops a re-gain of a tech already owned from paying twice; and
    /// - `goodtypes[g] + 0x30` — `TypeData::preq[0]`, the good's **first**
    ///   prerequisite, read raw off the type record rather than through
    ///   `get_preq`'s substitutions — **is the tech just gained**.
    ///
    /// So knowledge and metal are paid as the Classical age lands and oil
    /// as the Industrial one does, and a nation that holds a good early
    /// through a `has_preq` waiver — the Germans' metal, the Greeks'
    /// knowledge — fails the first half here and was paid at
    /// [`Sim::lay_starting_goods`] instead. The Greeks are the one case
    /// where both halves miss, because `GREEK_DELAY_KNOWLEDGE` takes the
    /// init grant back off them; the original's own branch at the foot of
    /// the loop re-grants it on the Classical age, and that is the
    /// `greek` arm below.
    ///
    /// `docs/COSTS.md`, "The starting grant arrives with the good".
    pub(crate) fn pay_arriving_goods(
        &mut self,
        who: Player,
        had_preq: &[bool; economy::RESOURCES],
        events: &[tech::Gained],
    ) {
        let w = who as usize;
        let greek = self
            .tables
            .tech_tree
            .has_tribe_bonus(&self.tables.setup, &self.tech[w], 5)
            && self.tables.tuning.greek_delay_knowledge != 0;
        for e in events {
            let tech::Gained::Type(x) = *e else { continue };
            for (g, &held) in had_preq.iter().enumerate() {
                if held {
                    continue;
                }
                let Some(id) = self.good_tree_type(g) else {
                    continue;
                };
                if self.tables.tech_tree.types[id].preq[0] != tech::Preq::Of(x) {
                    continue;
                }
                let amount = self.starting_good(who, g);
                self.economy.ledgers[w].bucket[g] += amount;
                self.economy.ledgers[w].dirty = true;
            }
            // The Greeks' delayed knowledge, re-granted on the first age.
            if greek && matches!(self.tables.tech_tree.kind(x), tech::Kind::Age(0)) {
                let amount = self.starting_good(who, 3);
                self.economy.ledgers[w].bucket[3] += amount;
                self.economy.ledgers[w].dirty = true;
            }
        }
    }

    /// `Leader::gain_tech`'s refund pass: gaining a **Science** library tech
    /// re-prices every technology queued in the player's first library and
    /// hands back the difference — `Build::refund_cost@00620490`,
    /// `docs/COSTS.md`, "The discounts".
    ///
    /// The gained tech is skipped, and so is anything queued that is not a
    /// technology. Nothing else in the executable calls `refund_cost`.
    pub(crate) fn reprice_library(&mut self, who: Player, t: tech::TypeId) {
        let tech::Kind::Epoch { line, .. } = self.tables.tech_tree.kind(t) else {
            return;
        };
        if line != tech::Line::Science {
            return;
        }
        let Some(lib) = self.first_library(who) else {
            return;
        };
        let level = self.tech[who as usize].epoch[tech::Line::Science.index()];
        for slot in 0..self.buildings[lib].queue.items.len() {
            let item = &self.buildings[lib].queue.items[slot];
            let Some(id) = item.tech else { continue };
            if id == t || !self.tables.tech_tree.kind(id).is_tech() {
                continue;
            }
            let age = self.tables.tech_tree.types[id].age;
            let mut back = [0i32; economy::RESOURCES];
            let item = &mut self.buildings[lib].queue.items[slot];
            for pair in 0..crate::cities::production::PAIRS {
                let good = item.good[pair];
                if good < 0 {
                    continue;
                }
                let paid = i32::from(item.cost[pair]);
                let now = crate::economy::cost::reprice(&self.tables.tuning, paid, level, age);
                back[good as usize] += paid - now;
                item.cost[pair] = now as i16;
            }
            let ledger = &mut self.economy.ledgers[who as usize];
            for (g, amount) in back.iter().enumerate() {
                ledger.bucket[g] += amount;
            }
        }
    }

    /// `LeaderData::type_avail(t, 1)` for a tree entry: 0, 2 or 4. A
    /// wonder type any player has built answers 0 ([`Sim::wonders_built`]).
    pub fn type_avail(&self, who: Player, t: tech::TypeId) -> i32 {
        let line = &self.tables.tech_tree.roles.wonder_line;
        let built = line
            .iter()
            .position(|&w| w == t)
            .is_some_and(|k| self.cities.wonders_built & (1 << k) != 0);
        let a = self.tables.tech_tree.type_avail_built(
            &self.tables.setup,
            &self.tech[who as usize],
            t,
            true,
            built,
        );
        if a == tech::AVAILABLE && !self.senate_avail(who, t) {
            return tech::NOT_AVAILABLE;
        }
        a
    }

    /// **The Senate arm** of `LeaderData::type_avail@006e33a0` (item 3597,
    /// `docs/AI.md` §320.1), the last test before its `return 4`, on exact
    /// identity (`TVar3 == SENATE`, 438): `false` — the original's 0 — when
    ///
    /// 1. `num_queued[SENATE] != 0`: a Senate site of mine is placed and not
    ///    yet active (`Wall::increment_stats` files an inactive site,
    ///    `Wall::activate` takes it off; [`Sim::num_sites_of`]);
    /// 2. `find_capital(−1, −1)` answers no city;
    /// 3. the capital it answers is another leader's (a city that was mine,
    ///    `was_capital_flags`) and a second `find_capital` skipping it does
    ///    not answer a city of mine that holds no active Senate and is the
    ///    founding capital (`0x4000`);
    /// 4. `count_buildings(SENATE, 0, 1)` — active members whose type `is`
    ///    the Senate — is non-zero on my city at the capital's index;
    /// 5. a second capital skipping the first is mine and holds an active
    ///    Senate, or is another leader's while my city at the first index is
    ///    the founding capital.
    ///
    /// The capital indices are per leader: the position in
    /// [`Sim::cities_of`], as `target_gate`'s `find_capital` reads it.
    ///
    /// SEAM: the original's per-leader city array keeps a dead city's slot
    /// (`city_mark`); [`Sim::cities_of`] drops it, so an index past a lost
    /// city reads its neighbour (parked 3597a).
    pub(crate) fn senate_avail(&self, who: Player, t: tech::TypeId) -> bool {
        let Some(rec) = self.build_record(t) else {
            return true;
        };
        if self.tables.build_types[rec].ident != crate::cities::build::Ident::Senate {
            return true;
        }
        if self.num_sites_of(who, rec) != 0 {
            return false;
        }
        let own = self.cities_of(who);
        let active_senates =
            |c: usize| self.count_buildings(c, crate::cities::build::Ident::Senate, true) != 0;
        let Some((c1, o1)) = self.leader_find_capital(who, None) else {
            return false;
        };
        if o1 != who {
            let Some((c2, o2)) = self.leader_find_capital(who, Some((c1, o1))) else {
                return false;
            };
            if o2 != who {
                return false;
            }
            let Some(&g2) = own.get(c2) else {
                return false;
            };
            if active_senates(g2) || !self.cities.cities[g2].founding_capital {
                return false;
            }
        }
        if own.get(c1).is_some_and(|&g| active_senates(g)) {
            return false;
        }
        if let Some((c2, o2)) = self.leader_find_capital(who, Some((c1, o1))) {
            if o2 == who {
                if own.get(c2).is_some_and(|&g| active_senates(g)) {
                    return false;
                }
            } else if own
                .get(c1)
                .is_some_and(|&g| self.cities.cities[g].founding_capital)
            {
                return false;
            }
        }
        true
    }

    /// `LeaderData::find_capital@006eb930(&c, &owner, skip_c, skip_owner)`:
    /// my first live capital (`city_flags & 1` and `& 0x10`) in slot order,
    /// else the first live city of another leader whose `was_capital_flags`
    /// carries my bit — each skipping `(skip_c, skip_owner)`. The index is
    /// the owner's own (a position in [`Sim::cities_of`]); `None` is the
    /// original's `c = −1`.
    ///
    /// SEAM: the second arm's gate is `leaders[who] & 3 == 3` read on *my*
    /// row for every other leader, and is taken as true for a leader in
    /// play.
    pub(crate) fn leader_find_capital(
        &self,
        who: Player,
        skip: Option<(usize, Player)>,
    ) -> Option<(usize, Player)> {
        let mine = self
            .cities_of(who)
            .into_iter()
            .enumerate()
            .find(|&(i, c)| self.cities.cities[c].capital && skip != Some((i, who)));
        if let Some((i, _)) = mine {
            return Some((i, who));
        }
        (0..self.territory.players.len() as Player)
            .filter(|&o| o != who)
            .find_map(|o| {
                self.cities_of(o)
                    .into_iter()
                    .enumerate()
                    .find(|&(i, c)| {
                        self.cities.cities[c].was_capital & (1u64 << (who as u32)) != 0
                            && skip != Some((i, o))
                    })
                    .map(|(i, _)| (i, o))
            })
    }

    /// After the tree changed: the researched bits follow it, and so does the
    /// Military level the population cap is indexed by — and the wall stats
    /// are stale, since a tech can change a site's clock and a building's
    /// hits.
    pub(crate) fn apply_gained(&mut self, who: Player) {
        // `Leader::gain_tech@006dcb60`'s tail: `leader_flags |= 0xc000000`,
        // both stats flags, on every tech that reaches it — so the units'
        // cached line of sight is refreshed by the next `Leader::process`
        // (`docs/VISION.md` §2).
        self.cities.wall_stats_dirty[who as usize] = true;
        self.roster.unit_stats_dirty[who as usize] = true;
        self.calc_attrition(who);
        self.sync_researched();
        self.sync_goods_available(who);
        let level = self.tech[who as usize].military_level();
        if self.roster.muster[who as usize].military_level != level {
            self.roster.muster[who as usize].military_level = level;
            self.recompute_pop_caps();
        }
        // The Military epoch's sibling: a **Civic** epoch widens every one of
        // this leader's cities, because `compute_reg_territory` reads the
        // level rather than a cached bonus. Guarded on the table actually
        // changing so an ordinary tech does not repaint 3,600 cells.
        if self.territory.borders[who as usize] != self.player_borders(who) {
            self.sync_territory();
        }
    }

    /// Which of the six basic resources this player may spend, and which they
    /// hold the prerequisite for — the two arrays `crates/sim/src/economy/cost.rs`
    /// steers its redirect tables by.
    ///
    /// `docs/COSTS.md`, "Three of the six resources are not available from
    /// the start": `resourcerules.xml` gives Knowledge and Metal the
    /// Classical Age as their prerequisite and Oil the Industrial Age, so a
    /// price written in one of those is charged somewhere else until the age
    /// arrives — Knowledge in food at three halves, Metal in timber at five
    /// quarters, Oil in metal at three halves and then on into timber. The
    /// test is `type_avail`, which `docs/TECH.md` ("Two questions this
    /// answers") settles for a good: `has_preq` — slot 0 is the unlocking
    /// age, slot 1 nothing, `obs` never — and a `type_eligible` that is
    /// always 4. So `available` and `discovered` agree for a good, which is
    /// why the obsolete table never fires in a stock game.
    ///
    /// The goods come out of the tree in `resourcerules.xml` order, and the
    /// first six of them are [`economy::Resource`]'s own.
    pub(crate) fn sync_goods_available(&mut self, who: Player) {
        let w = who as usize;
        let goods: Vec<tech::TypeId> = self
            .tables
            .tech_tree
            .types
            .iter()
            .enumerate()
            .filter(|(_, d)| matches!(d.kind, tech::Kind::Good))
            .map(|(i, _)| i)
            .take(economy::RESOURCES)
            .collect();
        for (g, t) in goods.into_iter().enumerate() {
            let p = &self.tech[w];
            let discovered = self.tables.tech_tree.has_preq(&self.tables.setup, p, t);
            let available = self
                .tables
                .tech_tree
                .type_avail(&self.tables.setup, p, t, true)
                == tech::AVAILABLE;
            self.economy.holdings[w].discovered[g] = discovered;
            self.economy.holdings[w].available[g] = available;
        }
    }

    /// The tree owns the bit for every unit type that is in it; `Muster`
    /// keeps a copy because production reads it per slot per frame.
    pub(crate) fn sync_researched(&mut self) {
        for (who, tech) in self.tech.iter().enumerate() {
            for (ty, ut) in self.tables.unit_types.iter().enumerate() {
                if let Some(id) = ut.tree {
                    self.roster.muster[who].researched[ty] = tech.tech[id];
                }
            }
        }
    }
}
