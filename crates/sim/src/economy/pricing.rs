//! Pricing: `TypeData::get_cost`'s discount arms, a unit's price and the
//! upgrade bump (`docs/COSTS.md`; `docs/STRUCTURE.md` §3, `economy/`).
//!
//! Moved out of `lib.rs` unchanged by item 4014 (`docs/STRUCTURE.md` §5, move 6).

use crate::*;

impl Sim {
    /// What a player would be charged for one of a type, per resource.
    ///
    /// The discount tail is `docs/COSTS.md`'s forty predicates and arrives as
    /// [`cost::Modifiers`] when the nation, wonder and government layers exist
    /// to produce it. Until then it is the undiscounted price, which is
    /// exactly what a stock game with no bonuses charges.
    pub fn price_of(&self, who: Player, ty: usize) -> [i32; economy::RESOURCES] {
        let stable_rares = self.stable_rare_discounts(who, ty);
        let discount = self.nation_unit_discount(who, ty);
        let russian_spy = self.russian_spy_discount(who, ty);
        let british_aa = self.british_aa_discount(who, ty);
        let american_aircraft = self.american_aircraft_discounts(who, ty);
        let roman_legion = self.roman_legion_discount(who, ty);
        let uranium = self.uranium_discount(who, ty);
        let citizens = self.citizen_discounts(who, ty);
        let domain = self.domain_cost_discounts(who, ty);
        let liberty = self.liberty_discount(who, ty);
        let despotism = self.despotism_unit_discount(who, ty);
        let salt = self.salt_barracks_discount(who, ty);
        let indian_elephant = self.indian_elephant_discount(who, ty);
        let (rare_goods, gypsum) = self.rare_goods_discounts(who);
        let m = match self.research_modifiers(who, ty) {
            Some(r) => crate::economy::cost::Modifiers {
                research: Some(r),
                indian_elephant,
                american_aircraft,
                british_aa,
                russian_spy,
                roman_legion,
                discount,
                stable_rares,
                despotism,
                salt,
                liberty,
                uranium,
                citizens,
                domain,
                rare_goods,
                gypsum,
                ..crate::economy::cost::Modifiers::default()
            },
            None => crate::economy::cost::Modifiers {
                late_discount: self.military_unit_discount(who, ty),
                indian_elephant,
                monarchy: self.monarchy_cost_discount(who, ty),
                socialism: self.socialism_cost_discount(who, ty),
                salmon: self.salmon_cost_discount(who, ty),
                maize: self.has_rare(who, economy::MAIZE),
                american_aircraft,
                british_aa,
                russian_spy,
                roman_legion,
                discount,
                stable_rares,
                despotism,
                salt,
                liberty,
                uranium,
                citizens,
                domain,
                rare_goods,
                gypsum,
                ..crate::economy::cost::Modifiers::default()
            },
        };
        self.price_with(who, ty, &m)
    }

    /// `TypeData::get_cost@00664090`'s **Salt arm** (`:208`–`213`, the head
    /// of the unit path, ahead of the research fork): a unit whose trainer
    /// (`UnitTypeData +0x40`) is the Barracks, by equality, of a leader
    /// holding Salt ([`economy::SALT`]) is charged `scaled × 9 / 10`.
    /// run958d's Spanish Hoplites at `1/2019` on block 7783 are 48/31 and
    /// 52/35 where the list price with the ramp is 53/34 and 57/38 (item
    /// 3318, `docs/AI.md` §264).
    pub fn salt_barracks_discount(&self, who: Player, ty: usize) -> bool {
        self.trainer_where(ty) == Some(crate::cities::nations::ty::BARRACKS)
            && self.has_rare(who, economy::SALT)
    }

    /// `TypeData::get_cost@00664090`'s **Despotism arm** (`:382`–`404`, the
    /// shared pre-ramp tail, after the Citizen arms and before the Statue
    /// of Liberty): a unit whose trainer (`UnitTypeData +0x40`) is the
    /// Barracks, by equality, takes the highest Despotism tier the player
    /// holds — `has_preq(DESPOTISM_3)`, else `_2`, else `_1` — as
    /// `DESPOTISM_MILITARY_CHEAPER{,2,3}`; on the research arm too, so an
    /// Iroquois Despot's Javelineers upgrade is 68/68 where the list price
    /// is 80/80 (run937, block 7983; item 3242, `docs/AI.md` §255).
    pub fn despotism_unit_discount(&self, who: Player, ty: usize) -> i32 {
        if self.trainer_where(ty) != Some(crate::cities::nations::ty::BARRACKS) {
            return 0;
        }
        match self.government_bonus_level(who, &self.tables.tech_tree.roles.despotism_preq) {
            0 => 0,
            n => self.tables.tuning.despotism_military_cheaper[n - 1],
        }
    }

    /// `TypeData::get_cost@00664090`'s **Monarchy arm** (`:657`–`665`, the
    /// train arm, after `MILITARY_UNIT_DISCOUNT`): a unit whose trainer
    /// (`UnitTypeData +0x40`) is the Stable, by equality, of a player for
    /// whom `has_preq(MONARCHY_1)` or `has_preq(MONARCHY_2)` holds, takes
    /// `MONARCHY_UNIT_PROD` — on either tier; `MONARCHY_UNIT_PROD2` is
    /// `train_time`'s alone (item 2331, `docs/GOLDEN.md` §84).
    pub fn monarchy_cost_discount(&self, who: Player, ty: usize) -> i32 {
        let trainer = self.tables.unit_types[ty]
            .tree
            .and_then(|t| self.tables.tech_tree.types.get(t))
            .and_then(|d| d.where_);
        let preqs = &self.tables.tech_tree.roles.monarchy_preqs;
        let p = &self.tech[who as usize];
        let held = preqs.iter().flatten().any(|qs| {
            qs.iter()
                .all(|&q| self.tables.tech_tree.has_tech_p(&self.tables.setup, p, q))
        });
        if trainer == Some(crate::cities::nations::ty::STABLES[0]) && held {
            self.tables.tuning.monarchy_unit_prod[0]
        } else {
            0
        }
    }

    /// `TypeData::get_cost@00664090`'s **Socialism arm** (`:666`–`672`, the
    /// train arm, after Monarchy's): a unit whose trainer (`UnitTypeData
    /// +0x40`) is the Siege Factory, the Airbase or the Dock, each by
    /// equality, of a player for whom `has_preq(SOCIALISM_1)` holds, takes
    /// `SOCIALISM_UNIT_BONUS` — the same predicate as `train_time`'s
    /// Socialism arm in [`Sim::train_tail`] (item 2357, `docs/GOLDEN.md`
    /// §85).
    pub fn socialism_cost_discount(&self, who: Player, ty: usize) -> i32 {
        let trainer = self.tables.unit_types[ty]
            .tree
            .and_then(|t| self.tables.tech_tree.types.get(t))
            .and_then(|d| d.where_);
        if trainer.is_some_and(|w| {
            [
                crate::cities::nations::ty::SIEGE_FACTORIES[0],
                crate::cities::nations::ty::AIRBASE,
                crate::cities::nations::ty::DOCKS[0],
            ]
            .contains(&w)
        }) && self
            .bonus_tier_held(
                who,
                std::slice::from_ref(&self.tables.tech_tree.roles.socialism_preqs),
            )
            .is_some()
        {
            self.tables.tuning.socialism_unit_bonus
        } else {
            0
        }
    }

    /// `TypeData::get_cost@00664090`'s **domain arm** (`:356`–`375`, the
    /// pre-ramp tail after the Citizen arms and before Despotism, which the
    /// train and research arms share): on `UnitTypeData +0x218`, a sea unit
    /// (`1`) of a leader holding Angkor Wat (`has_wonder(0x217)`) takes
    /// `ANGKOR_SHIPS`; an air unit (`2`) takes `ALUMINUM_AIR_COST` when the
    /// leader holds Aluminium (`rare` byte 3 bit 0, or `rare_conquest`'s),
    /// then `SPACE_AIR_COST` with `has_wonder(0x21e)`, the Space Program.
    /// run1147b's block 4383: the Aztecs' Silo `1/2010` queues a Nuclear
    /// ICBM at 872 knowledge and 942 oil — `(750 × 50 % × 95 % + 525) ×
    /// 99 %` — where this crate, without the arm, charged 1224 and 1366
    /// and escrowed it (item 3514, `docs/AI.md` §301).
    pub fn domain_cost_discounts(&self, who: Player, ty: usize) -> [i32; 2] {
        let held = self.wonders_held(who);
        match self.tables.unit_types[ty].combat.domain {
            crate::territory::attrition::Domain::Sea
                if held & (1 << tech::wonder::ANGKOR_WAT) != 0 =>
            {
                [self.tables.tuning.angkor_ships, 0]
            }
            crate::territory::attrition::Domain::Air => [
                if self.has_rare(who, economy::ALUMINUM) {
                    self.tables.tuning.aluminum_air_cost
                } else {
                    0
                },
                if held & (1 << tech::wonder::SPACE_PROGRAM) != 0 {
                    self.tables.tuning.space_air_cost
                } else {
                    0
                },
            ],
            _ => [0, 0],
        }
    }

    /// `SALMON_NAVAL_COST` — `TypeData::get_cost@00664090:673`–`677`, the
    /// train arm after Socialism's: a sea unit (`UnitTypeData +0x218 == 1`)
    /// of a player holding Salmon (`+0x6da9 & 1`, else `+0x6dd1 & 1`,
    /// rare 46) (item 2389, `docs/GOLDEN.md` §86).
    pub fn salmon_cost_discount(&self, who: Player, ty: usize) -> i32 {
        if self.tables.unit_types[ty].combat.domain == crate::territory::attrition::Domain::Sea
            && self.has_rare(who, economy::SALMON)
        {
            self.tables.tuning.salmon_naval_cost
        } else {
            0
        }
    }

    /// The building type a unit type is trained at — `UnitTypeData +0x40`,
    /// this crate's `where_` — by its [`build::Ident`], exactly and not by
    /// lineage. `get_cost`'s Stable arm and `Unit::update_los`'s troops
    /// term both compare it to fixed `TypeIndex`es.
    pub fn trainer_ident(&self, ty: usize) -> Option<crate::cities::build::Ident> {
        self.tables.unit_types[ty]
            .tree
            .and_then(|t| self.tables.tech_tree.types.get(t))
            .and_then(|d| d.where_)
            .and_then(|w| self.tables.build_types.iter().find(|b| b.tree == Some(w)))
            .map(|b| b.ident)
    }

    /// `TypeData::get_cost@00664090:416`–`420`: a type of the Nuclear Missile
    /// line (`is(0x13b, 0)`) takes `URANIUM_NUKE_COST` off the scaled base
    /// when the player holds Uranium — one more arm of the pre-ramp tail,
    /// on the train and the research arm alike. Great Sahara's Persians
    /// collect it between frames 1985 and 2182; their second queued ICBM
    /// (`1/2034` on 2183) is priced 927/1069 for it (`docs/AI.md` §167).
    pub fn uranium_discount(&self, who: Player, ty: usize) -> i32 {
        if self.is_nuke_line_type(ty) && self.has_rare(who, economy::URANIUM) {
            self.tables.tuning.uranium_nuke_cost
        } else {
            0
        }
    }

    /// `TypeData::get_cost@00664090`'s **Statue of Liberty** arm (listing
    /// `00664da7`..`00664e1e`, the pre-ramp tail after Despotism and before
    /// Uranium, on the train and the research arm alike): with
    /// `has_wonder(0x219)` held ([`Sim::wonders_held`]'s rule), a missile
    /// type (`is_missile`, the `obj_masks` bit) takes `LIBERTY_MISSILES` and
    /// any other type of the Bomber's line (`is(0x130, 0)`)
    /// `LIBERTY_BOMBERS`. Great Sahara's Persians finish the Statue
    /// before 3981 and queue their next Strategic Bomber at `1/2047` on
    /// 3982 for 136, not 173 (`docs/AI.md` §204).
    pub fn liberty_discount(&self, who: Player, ty: usize) -> i32 {
        let missile = self.tables.unit_types[ty].combat.obj_masks & combat::mask::MISSILE != 0;
        let pct = if missile {
            self.tables.tuning.liberty_missiles
        } else if self.tables.unit_types[ty].tree.is_some_and(|t| {
            t == air::BOMBER
                || (t < self.tables.tech_tree.types.len()
                    && self.tables.tech_tree.is(t, air::BOMBER, false))
        }) {
            self.tables.tuning.liberty_bombers
        } else {
            return 0;
        };
        if pct == 0 || self.wonders_held(who) & (1 << tech::wonder::STATUE_OF_LIBERTY) == 0 {
            return 0;
        }
        pct
    }

    /// `is(0x13b, 0)` on a unit record's type: the Nuclear Missile's line.
    pub(crate) fn is_nuke_line_type(&self, ty: usize) -> bool {
        self.tables.unit_types[ty].tree.is_some_and(|t| {
            t == crate::air::airbase::NUCLEARMISSILE
                || (t < self.tables.tech_tree.types.len()
                    && self
                        .tables
                        .tech_tree
                        .is(t, crate::air::airbase::NUCLEARMISSILE, false))
        })
    }

    /// What `get_cost`'s ramp counts beyond `num_units + num_queued`
    /// (`LeaderData::get_support_count@006da110` and `get_cost:546`): the
    /// leader's **`nukes_used`**, once for the two types the support count
    /// names by index (`0x13b`, `0x13c`) and once more for any type of the
    /// Nuclear Missile's line — so a spent nuke makes the next missile
    /// dearer forever, and an ICBM twice over (`docs/AI.md` §167). The
    /// matching `missiles_used` term (`0x139`, `0x13a`) is not modelled: no
    /// capture prices one after a launch.
    pub(crate) fn nuke_ramp_extra(&self, who: Player, ty: usize) -> i32 {
        let Some(t) = self.tables.unit_types[ty].tree else {
            return 0;
        };
        let used = self.combat.nukes.used_of(who);
        let mut extra = 0;
        if t == 0x13b || t == 0x13c {
            extra += used;
        }
        if self.is_nuke_line_type(ty) {
            extra += used;
        }
        extra
    }

    /// `TypeData::get_cost@00664090`'s Horses and Rubber arm, in the
    /// pre-ramp tail after the nations and Terra Cotta: a unit whose
    /// trainer (`UnitTypeData +0x40`, this crate's `where_`) is **exactly**
    /// the Stable (`0x1ac`) or the Auto Plant (`0x1ad`) takes
    /// `HORSES_STABLE_COST` off when the player holds Horses, then
    /// `RUBBER_AUTOPLANT_COST` when it holds Rubber — both rares on both
    /// kinds, as the listing tests them. [`Sim::has_rare`] is the union of
    /// `rare` and `rare_conquest` the arm reads. `docs/AI.md` §62.
    pub fn stable_rare_discounts(&self, who: Player, ty: usize) -> [i32; 3] {
        // **The Sulphur arm** (`get_cost:332`–`335`, right after Rubber): a
        // unit whose trainer is `0x1ae` or `0x1af`, the Siege Factory line,
        // takes `SULPHUR_SIEGE_COST` when the player holds Sulphur.
        let sulphur = if matches!(self.trainer_where(ty), Some(0x1ae | 0x1af))
            && self.has_rare(who, economy::SULPHUR)
        {
            self.tables.tuning.sulphur_siege_cost
        } else {
            0
        };
        let trainer = self.trainer_ident(ty);
        if !matches!(
            trainer,
            Some(crate::cities::build::Ident::Stable | crate::cities::build::Ident::AutoPlant)
        ) {
            return [0, 0, sulphur];
        }
        [
            if self.has_rare(who, economy::HORSES) {
                self.tables.tuning.horses_stable_cost
            } else {
                0
            },
            if self.has_rare(who, economy::RUBBER) {
                self.tables.tuning.rubber_autoplant_cost
            } else {
                0
            },
            sulphur,
        ]
    }

    /// `TypeData::get_cost@00664090`'s **French arm** of the pre-ramp
    /// nation tail (`get_cost:224`–`235`), right after `UNIT_COST_FACTOR`
    /// and before the stable rares: a French leader's unit whose trainer
    /// (`UnitTypeData +0x40`) is `0x1ae` or `0x1af`, the Siege Factory
    /// line, takes `FRENCH_SIEGE_COST` off, on the train arm and the
    /// research arm alike (`docs/COSTS.md`, "A French siege unit costs
    /// less"). French East Indies' Trebuchet research at `1/2034` is 79/79
    /// there, 94/94 here before item 1460.
    ///
    /// The arm's other branch, `FRENCH_SPECIAL_COST` on a type of class
    /// `0x36`, ships as 0 and is not modelled.
    ///
    /// **The German arm** (`get_cost:258`–`266`, after the Japanese): a
    /// German leader's (`has_tribe_bonus(0xc)`) unit of the Fire Raft line
    /// — `UnitTypeData::is(0x14e, 0)`, by lineage, so the Fire Raft through
    /// the Submarine — takes `GERMAN_SUBMARINE_COST` off, 25% as shipped.
    /// The battery's German who=1 priced the Fire Raft 20/41 here and 15/31
    /// there on 6580, and with a wealth bucket of 39 offered it at a
    /// quarter (`docs/AI.md` §242). The nations' arms are exclusive by
    /// tribe, so one answer per type is the tail's product. The Mongol,
    /// American, British, Nubian and Roman arms of the same tail are not
    /// modelled.
    ///
    /// **The Japanese arm** (`get_cost:236`–`257`, after the French and
    /// before the German): a Japanese leader's (`has_tribe_bonus(0xf)`)
    /// unit whose trainer is, by equality, the Barracks (`0x1ab`) takes
    /// `JAPANESE_BARRACKS_COST` — negative, so per tier, `(100 + tier × k) ×
    /// cost / 100` with the tier `min(ages, epoch[0])` — and one trained at
    /// a Dock, Anchorage or Shipyard (`0x1b0`..`0x1b2`) takes
    /// `JAPANESE_SHIP_COST` off. The Japanese Industrial pair's Barracks
    /// `1/2013` prices its Rifleman 68/60 there on 1345, 80/70 here before
    /// item 3510 (`docs/AI.md` §303).
    pub fn nation_unit_discount(&self, who: Player, ty: usize) -> i32 {
        let french = self.cities.nation[who as usize].french;
        if french && matches!(self.trainer_where(ty), Some(0x1ae | 0x1af)) {
            return self.tables.tuning.french_siege_cost;
        }
        if self.japanese(who) {
            let trainer = self.trainer_where(ty);
            if trainer == Some(crate::cities::nations::ty::BARRACKS) {
                return self.japanese_per_tier(who, self.tables.tuning.japanese_barracks_cost);
            }
            if trainer.is_some_and(|t| crate::cities::nations::ty::DOCKS.contains(&t)) {
                return self.tables.tuning.japanese_ship_cost;
            }
        }
        if self.cities.nation[who as usize].germans
            && self.tables.unit_types[ty].tree.is_some_and(|t| {
                self.tables
                    .tech_tree
                    .types
                    .get(crate::cities::nations::ty::FIRERAFT)
                    .is_some()
                    && self
                        .tables
                        .tech_tree
                        .is(t, crate::cities::nations::ty::FIRERAFT, false)
            })
        {
            return self.tables.tuning.german_submarine_cost;
        }
        // The tail's last arm (`get_cost:336`–`343`): a Spy or a General
        // under `SPIES_GENERALS_CHEAPER`, a half in the shipped rules
        // (Great Sahara's Spy at `1/2030` is 25/25 on 1983, 50/50 before
        // item 1583). The Russian spy arm before it is [`Sim::russian_spy_discount`].
        if self.is_spy_or_general(ty)
            && self
                .tables
                .tech_tree
                .roles
                .spy_general_cheaper_preq
                .is_some_and(|t| {
                    self.tables
                        .tech_tree
                        .has_tech(&self.tables.setup, &self.tech[who as usize], t)
                })
        {
            return self.tables.tuning.spy_general_cost;
        }
        0
    }

    /// `has_tribe_bonus(0xf)`: the Japanese, read off the tree as
    /// [`Self::refresh_nation_powers`] leaves them.
    pub(crate) fn japanese(&self, who: Player) -> bool {
        self.tables.tech_tree.has_tribe_bonus(
            &self.tables.setup,
            &self.tech[who as usize],
            crate::cities::nations::power::JAPANESE,
        )
    }

    /// A Japanese constant as both its readers take it: `k` as written when
    /// it is not negative, and `−k × min(ages, epoch[0])` when it is ("per
    /// age" in the rules) — `get_cost:241`–`251` and `train_time:123`–`131`.
    pub(crate) fn japanese_per_tier(&self, who: Player, k: i32) -> i32 {
        if k < 0 {
            let p = &self.tech[who as usize];
            -k * p.epoch[0].min(p.ages)
        } else {
            k
        }
    }

    /// `get_cost:283`–`289`: a British leader (`has_tribe_bonus(0xb)`)'s
    /// unit of the Anti-Aircraft Gun's lineage (`is(0x119, 0)`, the loose
    /// test) takes `BRITISH_AA_COST` (25) off, a step of its own after the
    /// American aircraft arm. Great Sahara's British at Industrial price the
    /// AA Gun at 58 food and 58 metal in theirs' queues and 79 and 79 here
    /// without it, and `create_units`' income factor read 206 against
    /// theirs' 225 on 2184 (item 3678, `docs/AI.md` §347).
    pub fn british_aa_discount(&self, who: Player, ty: usize) -> i32 {
        if self.cities.nation[who as usize].british
            && self.tables.unit_types[ty].tree.is_some_and(|t| {
                self.tables.tech_tree.types.get(0x119).is_some()
                    && self.tables.tech_tree.is(t, 0x119, false)
            })
        {
            self.tables.tuning.british_aa_cost
        } else {
            0
        }
    }

    /// `get_cost:268`–`282`: an American leader (`has_tribe_bonus(0x14)`)
    /// takes `AMERICANS_AIRCRAFT` (20) off a unit whose trainer
    /// (`UnitTypeData +0x40`) is the Airbase (`0x1bf`), by equality; then
    /// again off one that `is(0x15f, 0)`, the Aircraft Carrier's lineage;
    /// then again off one that `is(0x134, 0)`, the Fighter Bomber's — three
    /// tests, each its own truncating step, so they stack. The Great Lakes
    /// American Information pair's Airbase `1/2021` prices its Jet Fighters
    /// 71, 73, 75, 77 there on 2295 and 89, 91, 93, 95 here before item
    /// 3755 (`docs/AI.md` §372.1).
    pub fn american_aircraft_discounts(&self, who: Player, ty: usize) -> [i32; 3] {
        let american = self.tech.get(who as usize).is_some_and(|p| {
            self.tables.tech_tree.has_tribe_bonus(
                &self.tables.setup,
                p,
                crate::ai::tribe::AMERICANS,
            )
        });
        if !american {
            return [0; 3];
        }
        let k = self.tables.tuning.americans_aircraft;
        let line = |root: usize| {
            self.tables.unit_types[ty].tree.is_some_and(|t| {
                self.tables.tech_tree.types.get(root).is_some()
                    && self.tables.tech_tree.is(t, root, false)
            })
        };
        [
            if self.trainer_where(ty) == Some(0x1bf) {
                k
            } else {
                0
            },
            if line(0x15f) { k } else { 0 },
            if line(0x134) { k } else { 0 },
        ]
    }

    /// `get_cost:299`–`305`: a Russian leader's unit of the Spy's lineage
    /// (`is(0x3a, 0)`, the loose test) takes `RUSSIAN_SPY_COST` off, a
    /// step of its own before the French/German/Spy-General tail.
    pub fn russian_spy_discount(&self, who: Player, ty: usize) -> i32 {
        if self.cities.nation[who as usize].russians
            && self.tables.unit_types[ty].tree.is_some_and(|t| {
                self.tables.tech_tree.types.get(0x3a).is_some()
                    && self.tables.tech_tree.is(t, 0x3a, false)
            })
        {
            self.tables.tuning.russian_spy_cost
        } else {
            0
        }
    }

    /// Whether `ty` is of the Legion's lineage (`is(0x84, 0)`, the loose
    /// test) — the test both Roman legion arms make.
    pub(crate) fn is_legion_line(&self, ty: usize) -> bool {
        self.tables.unit_types[ty].tree.is_some_and(|t| {
            self.tables.tech_tree.types.get(0x84).is_some()
                && self.tables.tech_tree.is(t, 0x84, false)
        })
    }

    /// `get_cost`'s Citizen arms (`TypeIndex` `0x32`/`0x33`, the Citizen
    /// and the Korean one): `WOOL_CITIZENS_COST` for a player holding Wool,
    /// then `TURK_CITIZEN` for a Turkish leader (`has_tribe_bonus(8)`), two
    /// truncating steps after the Spy/General arm and before the Statue of
    /// Liberty, on the train and research arms alike.
    pub fn citizen_discounts(&self, who: Player, ty: usize) -> [i32; 2] {
        if !matches!(ty, 0 | 1) {
            return [0, 0];
        }
        [
            if self.has_rare(who, economy::WOOL) {
                self.tables.tuning.wool_citizens_cost
            } else {
                0
            },
            if self.cities.nation[who as usize].turks {
                self.tables.tuning.turk_citizen
            } else {
                0
            },
        ]
    }

    /// `get_cost:306`–`311`: a Roman leader's Legion-line unit takes
    /// `ROMAN_LEGION_CHEAP` off, a truncating step of its own after the
    /// Russian spy's and before the Terra Cotta Army's.
    pub fn roman_legion_discount(&self, who: Player, ty: usize) -> i32 {
        if self.cities.nation[who as usize].romans && self.is_legion_line(ty) {
            self.tables.tuning.roman_legion_cheap
        } else {
            0
        }
    }

    /// `TypeData::get_cost@00664090`'s **Indian elephant arm** (`:1556`–
    /// `1576`, after the captured-building doubling and the Supercollider
    /// surcharge, on the one return both the train and the research arm
    /// reach): an Indian leader (`has_tribe_bonus(0x15)`) and a unit that
    /// `is(t, 0)` for one of `0x105..=0x108` — the War Elephant and the
    /// three Mahouts — take `INDIANS_ELEPHANT_COST` (15) off. run970d's
    /// block 9589: the War Elephant queued at the Stable `1/2019` costs
    /// 81 food and 47 wealth in theirs, 96 and 56 without the arm (item
    /// 3335, `docs/AI.md` §271).
    pub fn indian_elephant_discount(&self, who: Player, ty: usize) -> i32 {
        let Some(t) = self.tables.unit_types[ty].tree else {
            return 0;
        };
        let elephant = (0x105..=0x108).any(|x: tech::TypeId| {
            self.tables.tech_tree.types.get(x).is_some() && self.tables.tech_tree.is(t, x, false)
        });
        if elephant
            && self.tables.tech_tree.has_tribe_bonus(
                &self.tables.setup,
                &self.tech[who as usize],
                0x15,
            )
        {
            self.tables.tuning.indians_elephant_cost
        } else {
            0
        }
    }

    /// `is(0x3a, 1) || is(0x36, 1)`: a Spy or a General, the strict test.
    pub(crate) fn is_spy_or_general(&self, ty: usize) -> bool {
        let strict = |x: tech::TypeId| {
            self.tables.unit_types[ty].tree.is_some_and(|t| {
                self.tables.tech_tree.types.get(x).is_some() && self.tables.tech_tree.is(t, x, true)
            })
        };
        strict(0x3a) || strict(0x36)
    }

    /// A unit type's trainer as the raw `UnitTypeData +0x40`.
    pub(crate) fn trainer_where(&self, ty: usize) -> Option<tech::TypeId> {
        self.tables.unit_types[ty]
            .tree
            .and_then(|t| self.tables.tech_tree.types.get(t))
            .and_then(|d| d.where_)
    }

    /// `TypeData::get_cost@00664090`'s fork on the `leader + 0x6c18` bit —
    /// this crate's `PlayerTech::tech` — for a unit type in the tree: clear,
    /// and what is priced is the type's **research** (`docs/AI.md` §56).
    /// `None` when the bit is set, or the type is outside the tree, which is
    /// the train arm.
    ///
    /// The refit surcharge (`get_cost:438`–`505`) walks every unit type `p`
    /// below the gaia band and charges for it when `p` is **this type's own
    /// `FROM`**, or when this type lies on `p`'s `JUMP` chain — each link
    /// through `get_graft`, the nation's variant. Those are the units the
    /// research upgrades. Per resource, `d = base − p.base`, halved when `p`
    /// costs nothing in it, capped at `UNIT_REFIT_MAX_COST`, and charged
    /// `UNIT_COST_FACTOR × (num_queued + num_units) × d`. `TypeIndex`
    /// `0x42`–`0x44` (the Militia line) skip the walk.
    ///
    /// `MILITARY_UPGRADE_DISCOUNT` is the train arm's rule with the upgrade
    /// constant and **no floor on the type's level** (`get_cost:507`–`525`
    /// has none of `:639`'s `max(level, 1)` or its `max(pct, 1)`).
    pub fn research_modifiers(
        &self,
        who: Player,
        ty: usize,
    ) -> Option<crate::economy::cost::Research> {
        let unit = &self.tables.unit_types[ty];
        let t = unit.tree?;
        let w = who as usize;
        if self.tech[w].tech.get(t).copied().unwrap_or(true)
            || unit
                .cols
                .flag(crate::ai::ai_load::uflags::NO_RESEARCH_PRICE)
        {
            return None;
        }
        let tree = &self.tables.tech_tree;
        let resolve =
            |x: Option<tech::TypeId>| tree.get_graft(&self.tables.setup, &self.tech[w], x);
        let factor = self.tables.tuning.unit_cost_factor;
        let mut refit = [0; economy::RESOURCES];
        if !(0x42..=0x44).contains(&t) {
            let from = resolve(tree.types[t].from);
            for (prec, p) in self.tables.unit_types.iter().enumerate() {
                let Some(pt) = p.tree.filter(|_| !p.gaia) else {
                    continue;
                };
                let mut hit = from == Some(pt);
                let mut j = resolve(tree.types[pt].jump);
                let mut guard = 0;
                while !hit && let Some(x) = j {
                    hit = x == t;
                    j = resolve(tree.types[x].jump);
                    guard += 1;
                    if guard > 64 {
                        break;
                    }
                }
                if !hit {
                    continue;
                }
                let n = self.roster.muster[w].queued_by_type[prec]
                    + self.roster.muster[w].by_type[prec];
                if n == 0 {
                    continue;
                }
                for (r, out) in refit.iter_mut().enumerate() {
                    let theirs = p.price.base[r];
                    let mut d = unit.price.base[r] - theirs;
                    if theirs == 0 {
                        d /= 2;
                    }
                    if d > 0 {
                        *out += factor * n * d.min(self.tables.tuning.unit_refit_max_cost);
                    }
                }
            }
        }
        let mut discount = 0;
        let mut dutch_ship = 0;
        let mut american_land = 0;
        if self.role_word_of_rec(ty) & crate::ai::ai_load::role::ROLE_MILITARY != 0 {
            let ahead = self.tech[w].epoch[tech::Line::Military.index()]
                - self
                    .tables
                    .tech_tree
                    .military_level_in(&self.tables.setup, t);
            if ahead > 0 {
                discount = self.tables.tuning.military_upgrade_discount * ahead;
                let span = self.tables.setup.ending - self.tables.setup.starting_age + 1;
                if span < 8 {
                    discount = (span * discount + 7) / 8;
                }
            }
            // `get_cost:532`–`535` is after the Military block's close, so
            // a Dutch ship's research pays it whether or not the line is
            // ahead (item 3329: run963b's Dromon at the dock, 54 against
            // 60, on block 5783). The Americans' land arm is the step before
            // it (`get_cost:528`–`531`, `has_tribe_bonus(0x14)` and `+0x218
            // == 0`; item 3374: run999's who=1 held 20 food and 20 metal more
            // than ours on block 9376, a quarter of an 80-and-80 upgrade).
            if self.tables.unit_types[ty].combat.domain == crate::territory::attrition::Domain::Land
                && self.tables.tech_tree.has_tribe_bonus(
                    &self.tables.setup,
                    &self.tech[w],
                    crate::ai::tribe::AMERICANS,
                )
            {
                american_land = self.tables.tuning.americans_upgrade_discount;
            }
            if self.cities.nation[w].dutch
                && self.tables.unit_types[ty].combat.domain
                    == crate::territory::attrition::Domain::Sea
            {
                dutch_ship = self.tables.tuning.dutch_ship_discount;
            }
        }
        Some(crate::economy::cost::Research {
            american_land,
            dutch_ship,
            wine: if self.has_rare(who, economy::WINE) {
                self.tables.tuning.wine_unit_upgrades
            } else {
                0
            },
            premium_cost: unit.cols.research_premium_cost,
            refit,
            discount,
        })
    }

    /// `MILITARY_UNIT_DISCOUNT` — the percentage `TypeData::get_cost@00664090`
    /// takes off a military unit's train price, **after** the ramp and before
    /// Monarchy, Socialism and Salmon (`docs/COSTS.md`, "The discounts";
    /// `docs/AI.md` §56).
    ///
    /// For a type whose `role & 0x10000` is set: `level` is the type's own
    /// Military level ([`tech::TechTree::military_level_of`]) floored at 1,
    /// and for every level of `epoch[0]` above it the price falls
    /// `MILITARY_UNIT_DISCOUNT` percent. A scenario spanning fewer than
    /// eight ages scales the percentage by `span / 8`, rounded up through
    /// the `+ 7` and floored at 1. Zero for anything else. It is the only
    /// late discount this crate applies, so the fold into
    /// [`cost::Modifiers::late_discount`] is exact.
    pub fn military_unit_discount(&self, who: Player, ty: usize) -> i32 {
        if self.role_word_of_rec(ty) & crate::ai::ai_load::role::ROLE_MILITARY == 0 {
            return 0;
        }
        let level = self.tables.unit_types[ty]
            .tree
            .map_or(0, |t| {
                self.tables
                    .tech_tree
                    .military_level_in(&self.tables.setup, t)
            })
            .max(1);
        let ahead = self.tech[who as usize].epoch[tech::Line::Military.index()] - level;
        if ahead <= 0 {
            return 0;
        }
        let mut d = self.tables.tuning.military_unit_discount * ahead;
        let span = self.tables.setup.ending - self.tables.setup.starting_age + 1;
        if span < 8 {
            d = ((span * d + 7) / 8).max(1);
        }
        d
    }

    /// **`get_cost`'s bump loop** (`TypeData::get_cost@00664090:158`–`205`,
    /// listing `0066446d`..`006646cb`): a unit trained while its upgrade is
    /// queued is charged the upgrade's base. Only for a type whose
    /// availability bit (`leader + 0x6c18`) is set — the train arm; a clear
    /// bit skips the loop whatever [`Sim::research_modifiers`] decides.
    ///
    /// Every unit type `p` below `0x192` whose `num_queued[p]` (`+0x5a22`,
    /// a research of `p` included) is not zero is a hit when `p`'s `FROM`,
    /// through the nation's graft, is this type, or when `p` lies on this
    /// type's `JUMP` chain, each link through the graft — the research
    /// refit's walk with the two ends swapped. Per resource, a hit adds
    /// `max(0, p.base − base)` to the base (`006646b9`–`006646c9`), and
    /// `UNIT_COST_FACTOR` multiplies the sum after the loop (`006645ae`),
    /// so the ramp's ceiling is measured against the bumped base too.
    /// `docs/AI.md` §56.5; run523's block 13383, where the Bombard's
    /// research at `1/2024` makes `1/2028`'s Trebuchet 80 × 95/100.
    pub fn upgrade_bump(&self, who: Player, ty: usize) -> [i32; economy::RESOURCES] {
        let mut bump = [0; economy::RESOURCES];
        let unit = &self.tables.unit_types[ty];
        let Some(t) = unit.tree else {
            return bump;
        };
        let w = who as usize;
        if !self.tech[w].tech.get(t).copied().unwrap_or(false) {
            return bump;
        }
        let tree = &self.tables.tech_tree;
        let resolve =
            |x: Option<tech::TypeId>| tree.get_graft(&self.tables.setup, &self.tech[w], x);
        for (prec, p) in self.tables.unit_types.iter().enumerate() {
            let Some(pt) = p.tree.filter(|_| !p.gaia) else {
                continue;
            };
            if self.roster.muster[w].queued_by_type[prec] == 0 {
                continue;
            }
            let mut hit = resolve(tree.types[pt].from) == Some(t);
            let mut j = resolve(tree.types[t].jump);
            let mut guard = 0;
            while !hit && let Some(x) = j {
                hit = x == pt;
                j = resolve(tree.types[x].jump);
                guard += 1;
                if guard > 64 {
                    break;
                }
            }
            if !hit {
                continue;
            }
            for (out, (theirs, ours)) in bump
                .iter_mut()
                .zip(p.price.base.iter().zip(unit.price.base))
            {
                *out += (theirs - ours).max(0);
            }
        }
        bump
    }

    /// [`Sim::price_of`], with the discount tail supplied.
    pub fn price_with(
        &self,
        who: Player,
        ty: usize,
        m: &crate::economy::cost::Modifiers,
    ) -> [i32; economy::RESOURCES] {
        let muster = &self.roster.muster[who as usize];
        let holdings = &self.economy.holdings[who as usize];
        let unit = &self.tables.unit_types[ty];
        // Built *and* ordered. `LeaderData::get_support_count` sums
        // `num_units` and `num_queued`, so five hoplites ordered at once each
        // pay a ramp step for the ones ahead of them in the queue. The time
        // ramp reads only the first of the two arrays; see
        // `docs/PRODUCTION.md`.
        let counts = crate::economy::cost::Counts {
            of_type: muster.by_type[ty]
                + muster.queued_by_type[ty]
                + self.worker_support(who, ty)
                + self.nuke_ramp_extra(who, ty),
            of_group: unit
                .group
                .map_or(0, |g| muster.by_group[g] + muster.queued_by_group[g]),
        };
        let bump = self.upgrade_bump(who, ty);
        let bumped;
        let price = if bump.iter().all(|&b| b == 0) {
            &unit.price
        } else {
            let mut p = unit.price.clone();
            for (base, b) in p.base.iter_mut().zip(bump) {
                *base += b;
            }
            bumped = p;
            &bumped
        };
        crate::economy::cost::charges(
            &self.tables.tuning,
            price,
            counts,
            m,
            &holdings.available,
            &holdings.discovered,
            &self.economy.redirects,
        )
    }

    /// The worker lines' extra term in `LeaderData::get_support_count`'s
    /// caller, `TypeData::get_cost@00664090`'s unit ramp: a **Citizen**
    /// (`0x32`, or the Korean `0x33`) is ramped by the Militia line too,
    /// `support(0x42) + support(0x43) − scholar_militia + support(0x44)`,
    /// and a **Scholar** (`0x34`, `0x35`) by `scholar_militia`
    /// (`docs/COSTS.md`, "A Citizen is ramped by the Militia too").
    /// `support(t)` is `num_units[t] + num_queued[t]`.
    ///
    /// `scholar_militia` (`LeaderData +0x9f0`) is
    /// `Leader::track_unit_type@006e0dd0`'s count of militia whose former
    /// type (`UnitData +0x54`, [`Unit::rare`]) is a scholar; it is derived
    /// here from the live militia rather than kept.
    pub fn worker_support(&self, who: Player, ty: usize) -> i32 {
        let ti = self.tables.unit_types[ty].type_index;
        if !(0x32..=0x35).contains(&ti) {
            return 0;
        }
        let scholar_militia = self
            .units
            .iter()
            .filter(|u| {
                u.alive()
                    && u.owner == who
                    && u.captain
                    && !u.decoy
                    && (0x42..=0x44).contains(&u.type_index)
                    && matches!(u.rare, 0x34 | 0x35)
            })
            .count() as i32;
        if ti >= 0x34 {
            return scholar_militia;
        }
        let m = &self.roster.muster[who as usize];
        let support = |t: usize| {
            self.tables
                .unit_types
                .iter()
                .position(|u| u.type_index == t as i32)
                .map_or(0, |r| m.by_type[r] + m.queued_by_type[r])
        };
        support(0x42) + support(0x43) - scholar_militia + support(0x44)
    }
}
