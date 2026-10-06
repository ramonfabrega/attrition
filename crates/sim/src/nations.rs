//! The nation-power layer: a player's nation, and the flags it turns on.
//!
//! Every nation power in the original is one call —
//! `LeaderData::has_tribe_bonus(n)`, where `n` is the nation's index in the
//! roster `rules.xml`'s `TRIBES` block lists. [`TechTree::has_tribe_bonus`]
//! is that call, and it has been here since the tech tree was; what was
//! missing is the wire between it and the number the game actually carries.
//! `LeaderData::tribe` is dumped by `LEADERS=9` on every frame, and until
//! now nothing read it, so every traced game was played with **no nation
//! power on either side** — twenty-four of them inert at once.
//!
//! This module is that wire, and nothing else. [`Sim::set_tribe`] takes the
//! dump's own number; [`Sim::refresh_nation_powers`] recomputes the
//! per-nation booleans on [`city::Nation`] that the mechanics read. Those
//! booleans are a **cache of `has_tribe_bonus`, not a second source**: each
//! is that call's answer for one roster index, so the lobby's "No Nation
//! Powers" flag and the no-city gate apply here exactly as they do there.
//!
//! `docs/TECH.md` ("`has_preq`: are the prerequisites met") is where the
//! roster indices are established; `docs/ECONOMY.md` ("The commerce cap") is
//! the measurement that made the wire worth having.
//!
//! [`TechTree::has_tribe_bonus`]: crate::tech::TechTree::has_tribe_bonus
//! [`city::Nation`]: crate::city::Nation

use crate::tech::TypeId;
use crate::{Player, Sim};

/// The `TypeIndex` values `Build::activate`'s free-unit block names.
///
/// The unit values are each the *base* of a line: the block hands one to
/// `get_graft` and then — all but three arms — to `current_upgrade`, so what
/// is trained is the newest type the player owns in that line, not this one.
pub mod ty {
    use crate::tech::TypeId;
    /// `CARA`, the Nubians' free caravan from a Market.
    pub const CARA: TypeId = 0x3b;
    /// `SCOUT`, the Iroquois' from a Barracks.
    pub const SCOUT: TypeId = 0x45;
    /// `PEASANTS`, the Koreans' from a city.
    pub const PEASANTS: TypeId = 0x32;
    /// `SCHOLARS`, the Americans' from a University.
    pub const SCHOLARS: TypeId = 0x34;
    /// `GENERAL`, the French one from a fort.
    pub const GENERAL: TypeId = 0x36;
    /// `SUPPLYWAGON`, the French one from a Siege Factory.
    pub const SUPPLYWAGON: TypeId = 0x3f;
    /// `SLINGERS`, the Aztecs' from a Barracks.
    pub const SLINGERS: TypeId = 0x52;
    /// `HOPLITES`, the Roman legion.
    pub const HOPLITES: TypeId = 0x84;
    /// `BOWMEN`, the British archers.
    pub const BOWMEN: TypeId = 0xaa;
    /// `HORSEARCHERS`, the Mongols' from a Stable.
    pub const HORSEARCHERS: TypeId = 0xbb;
    /// `CATAPULT`, the Turks' from a Siege Factory.
    pub const CATAPULT: TypeId = 0x109;
    /// `BIPLANE`, the Germans' from an Airbase.
    pub const BIPLANE: TypeId = 0x11f;
    /// `BOMBER`, the Americans' from an Airbase, in the Modern age.
    pub const BOMBER: TypeId = 0x130;
    /// `MODERN_AGE`, the bomber's own gate.
    pub const MODERN_AGE: TypeId = 0x225;
    /// `FISHERMEN`, the British ones from a Dock.
    pub const FISHERMEN: TypeId = 0x13d;
    /// `BARK`, the Dutch light ship.
    pub const BARK: TypeId = 0x143;
    /// `TRIREME`, the Spanish one from a Dock.
    pub const TRIREME: TypeId = 0x154;

    /// `UNIVERSITY`.
    pub const UNIVERSITY: TypeId = 0x1a4;
    /// `BARRACKS`.
    pub const BARRACKS: TypeId = 0x1ab;
    /// `STABLE` and `AUTOPLANT`, the Mongol arm's pair.
    pub const STABLES: [TypeId; 2] = [0x1ac, 0x1ad];
    /// `SIEGEFACTORY` and `FACTORY`, the Turkish and French arm's pair.
    pub const SIEGE: [TypeId; 2] = [0x1ae, 0x1af];
    /// `DOCK`, `ANCHORAGE`, `SHIPYARD` — the naval arm tests all three by
    /// name rather than through the lineage.
    pub const DOCKS: [TypeId; 3] = [0x1b0, 0x1b1, 0x1b2];
    /// `MARKET`.
    pub const MARKET: TypeId = 0x1b4;
    /// `AIRBASE`.
    pub const AIRBASE: TypeId = 0x1bf;
}

/// The roster indices `Build::activate`'s free-unit block passes
/// `has_tribe_bonus`, in the order the block tests them.
pub(crate) mod power {
    pub const AZTECS: usize = 0;
    pub const NUBIANS: usize = 4;
    pub const ROMANS: usize = 6;
    pub const TURKS: usize = 8;
    pub const SPANISH: usize = 9;
    pub const FRENCH: usize = 10;
    pub const BRITISH: usize = 11;
    pub const GERMANS: usize = 12;
    pub const KOREANS: usize = 16;
    pub const MONGOLS: usize = 17;
    pub const IROQUOIS: usize = 18;
    pub const AMERICANS: usize = 20;
    pub const DUTCH: usize = 22;
    /// Not `Build::activate`'s: the Persians' caravan is `Build::process`'s
    /// (`Sim::persian_market_caravan`).
    pub const PERSIANS: usize = 23;
}

/// The nation roster, in the order `rules.xml`'s `TRIBES` block lists it.
///
/// The index is the identity everywhere: it is the `n` of
/// `has_tribe_bonus(n)`, the number `LEADERDATA` prints as `tribe`, and the
/// bit position `TRIBE_MASK` sets (`crate::tech::TypeDef::tribes`). The
/// names are each nation file's own `<TRIBE name="…">`, which is what
/// `ScenarioFuncSet::find_nation` matches — so a name here is a claim about
/// the user's install, and `cargo run -p rondata -- <install>` re-derives
/// the whole list from `rules.xml` and `tribes/` and fails if it has
/// drifted.
pub const ROSTER: [&str; 24] = [
    "Aztecs",
    "Maya",
    "Inca",
    "Bantu",
    "Nubians",
    "Greeks",
    "Romans",
    "Egyptians",
    "Turks",
    "Spanish",
    "French",
    "British",
    "Germans",
    "Russians",
    "Chinese",
    "Japanese",
    "Koreans",
    "Mongols",
    "Iroquois",
    "Lakota",
    "Americans",
    "Indians",
    "Dutch",
    "Persians",
];

/// The roster index of a nation, by the name [`ROSTER`] holds.
pub fn power_of(name: &str) -> Option<usize> {
    ROSTER.iter().position(|n| *n == name)
}

impl Sim {
    /// Sets a player's nation from the dump's own `LeaderData::tribe`, and
    /// refreshes the flags that follow from it.
    ///
    /// The original's `tribe` is a signed index and **−1 is a real value** —
    /// a leader with no nation, which `has_tribe_bonus` answers no to for
    /// every power. That is what `power: None` means here.
    /// [`crate::tech::PlayerTech::tribe`] is a different thing: it indexes
    /// [`crate::tech::TechTree::tribes`] for the unit-graft and unique-unit
    /// tables, so it stays inside the roster and a −1 leaves it at zero,
    /// whose graft table is identity.
    ///
    /// An index past the end of a *loaded* roster still counts as a power —
    /// the tree built by a unit test has one placeholder tribe in it, and a
    /// power is not a table lookup.
    pub fn set_tribe(&mut self, who: Player, tribe: i64) {
        let w = who as usize;
        let power = usize::try_from(tribe).ok();
        self.tech[w].power = power;
        self.tech[w].tribe = power
            .filter(|&i| i < self.tech_tree.tribes.len())
            .unwrap_or(0);
        self.refresh_nation_powers(who);
    }

    /// `city_num != 0` — the half of `has_tribe_bonus`'s city gate that
    /// moves (`LeaderData::has_tribe_bonus@006e1370`, group 22) — written
    /// where a city is founded or closed, and the cached flags redone when
    /// it flips.
    pub(crate) fn sync_has_city(&mut self, who: Player) {
        let has = self.city_num(who) != 0;
        let w = who as usize;
        if self.tech[w].has_city != has {
            self.tech[w].has_city = has;
            self.refresh_nation_powers(who);
        }
    }

    /// The lobby's two bits `has_tribe_bonus` reads, as the tech tree's
    /// [`crate::tech::Setup`] holds them — "No Nation Powers" and the
    /// starting town — and every player's city half brought in line.
    pub fn sync_setup_from_lobby(&mut self) {
        self.setup.no_nation_powers = self.lobby.no_nation_powers;
        self.setup.starting_town = self.lobby.starting_town != 0;
        // The age the game opens in, which `Leader::init`'s starting
        // position reads (`docs/TECH.md`, "The starting position"): every
        // lobby on file before item 1466 was Ancient, to the Information
        // age, so the default `Setup::STANDARD` was right until a lobby
        // said otherwise (run650..652).
        self.setup.starting_age = self.lobby.starting_technology;
        self.setup.starting_age2 = self.lobby.starting_technology2;
        self.setup.ending = self.lobby.ending_technology;
        for who in 0..self.tech.len() {
            self.sync_has_city(who as Player);
        }
    }

    /// Recomputes one player's per-nation flags from their power.
    ///
    /// Call it after anything `has_tribe_bonus` reads changes — the power
    /// itself, `has_city`, or the lobby's "No Nation Powers". The seven
    /// nations with no flag on [`crate::city::Nation`] — the Greeks, Spanish,
    /// Japanese, Mongols, Iroquois, Americans and Persians — are not missing:
    /// their powers are read straight off `has_tribe_bonus` at the rules that
    /// use them, and this cache exists only for the mechanics that took a
    /// boolean before the tree did.
    pub fn refresh_nation_powers(&mut self, who: Player) {
        let w = who as usize;
        let on: [bool; ROSTER.len()] = std::array::from_fn(|n| {
            self.tech_tree
                .has_tribe_bonus(&self.setup, &self.tech[w], n)
        });
        let n = &mut self.nation[w];
        n.aztecs = on[0];
        n.maya = on[1];
        n.inca = on[2];
        n.bantu = on[3];
        n.nubians = on[4];
        n.romans = on[6];
        n.egyptians = on[7];
        n.turks = on[8];
        n.french = on[10];
        n.british = on[11];
        n.germans = on[12];
        n.russians = on[13];
        n.chinese = on[14];
        n.koreans = on[16];
        n.lakota = on[19];
        n.indians = on[21];
        n.dutch = on[22];
    }
}

impl Sim {
    /// **`Build::activate@00623e20:603–1150` — the high-water mark, and the
    /// nation's free units.**
    ///
    /// It is a **mark**, not a first-one flag: the British get archers with
    /// every Barracks, and what the mark buys is that a *replacement* for
    /// one that died gets none.
    ///
    /// `n = num_buildings[type] + get_buildings(to)` — the finished
    /// buildings of this record and of every record it upgrades to; when
    /// that beats `high_buildings[base_type]` the mark rises, and a
    /// building that is *counted* and did not arrive by capture pays the
    /// owner's nation its free units. The mark is per **lineage root**
    /// (`TypeData::get_base_type`, the type vtable's `+0xe4` — the slot
    /// `Unit::think_scout` hands `FILTER_BASE_TYPE`), so a Keep does not
    /// re-earn what its Tower already claimed.
    ///
    /// `docs/CITIES.md` §4.3.
    pub(crate) fn claim_building_high(&mut self, b: usize, captured: bool, counted: bool) {
        let Some(rec) = self.buildings[b].ty else {
            return;
        };
        let who = self.buildings[b].owner;
        let n = self.buildings_of_line(who, rec);
        let root = self.base_build_type(rec);
        let w = who as usize;
        // Gaia owns buildings in some fixtures and has no leader record.
        if w >= self.building_high.len() {
            return;
        }
        if self.building_high[w].len() <= root {
            self.building_high[w].resize(root + 1, 0);
        }
        if self.building_high[w][root] >= n || captured {
            return;
        }
        self.building_high[w][root] = n;
        if counted {
            self.free_units_for(b, rec);
        }
    }

    /// `TypeData::get_base_type` on a building record: the root of its
    /// `FROM` chain.
    fn base_build_type(&self, rec: usize) -> usize {
        let mut cur = rec;
        let mut guard = 0;
        while let Some(x) = self.build_types[cur].from {
            cur = x;
            guard += 1;
            if guard > 16 {
                break;
            }
        }
        cur
    }

    /// The else-if chain over the finished building's own `TypeIndex` —
    /// which is how the original spells it, so a Keep is not a Tower here
    /// and an Anchorage is named beside its Dock. Only the British arm has a
    /// diff behind it (`docs/CITIES.md` §4.3, Coverage).
    fn free_units_for(&mut self, b: usize, rec: usize) {
        let who = self.buildings[b].owner;
        let t = self.build_types[rec].tree;
        if t == Some(ty::MARKET) {
            // The Nubian caravan. `NUBIAN_FREE_CARAVAN` ships as 0, so the
            // arm is live and inert at once.
            let n = i32::from(self.tuning.nubian_free_caravan != 0);
            self.free_train(b, power::NUBIANS, ty::CARA, n, true);
        } else if t == Some(ty::BARRACKS) {
            self.barracks_free_units(b, who);
        } else if t.is_some_and(|x| ty::SIEGE.contains(&x)) {
            let siege = self.tuning.turk_free_siege;
            self.free_train(b, power::TURKS, ty::CATAPULT, siege, true);
            // The French wagon and general are the two arms that skip
            // `current_upgrade`: the graft's answer is trained as it stands.
            let wagons = i32::from(self.tuning.french_free_supply != 0);
            self.free_train(b, power::FRENCH, ty::SUPPLYWAGON, wagons, false);
        } else if t.is_some_and(|x| ty::DOCKS.contains(&x)) {
            // The Spanish trireme is gated on the age as well as the count:
            // `ages < 5`, so it stops at Gunpowder.
            let ships = if self.tech[who as usize].ages < 5 {
                self.tuning.spanish_free_trireme
            } else {
                0
            };
            self.free_train(b, power::SPANISH, ty::TRIREME, ships, true);
            let barks = self.tuning.dutch_free_light_ship;
            self.free_train(b, power::DUTCH, ty::BARK, barks, true);
            let fishers = self.tuning.british_free_fishermen;
            self.free_train(b, power::BRITISH, ty::FISHERMEN, fishers, true);
        } else if t == Some(ty::UNIVERSITY) {
            // The third arm with no `current_upgrade`.
            let scholars = self.tuning.americans_free_scholar;
            self.free_train(b, power::AMERICANS, ty::SCHOLARS, scholars, false);
        } else if crate::build::is_fort(&self.build_types, rec) {
            let generals = i32::from(self.tuning.french_free_general != 0);
            self.free_train(b, power::FRENCH, ty::GENERAL, generals, false);
        } else if t == Some(ty::AIRBASE) {
            let fighters = self.tuning.german_free_fighter;
            self.free_train(b, power::GERMANS, ty::BIPLANE, fighters, true);
            // The American bomber wants the Modern age as well as the base.
            let modern =
                self.tech_tree
                    .has_tech(&self.setup, &self.tech[who as usize], ty::MODERN_AGE);
            let bombers = if modern {
                self.tuning.americans_free_bomber
            } else {
                0
            };
            self.free_train(b, power::AMERICANS, ty::BOMBER, bombers, true);
        } else if !self.building_is_city(b) {
            if t.is_some_and(|x| ty::STABLES.contains(&x)) {
                let epoch = self.tech[who as usize].epoch[0];
                let g = &self.tuning;
                let mut horse = if epoch < 2 {
                    g.mongol_start_cavalry
                } else {
                    g.mongol_free_cavalry
                };
                if epoch > 2 && horse < g.mongol_three_mil_cavalry {
                    horse = g.mongol_three_mil_cavalry;
                }
                self.free_train(b, power::MONGOLS, ty::HORSEARCHERS, horse, true);
            }
        } else {
            // `KOREAN_CITIZENS[city_num − 1]`, clamped to the array — one
            // for the first city, three for the second, five after. The
            // count already includes the city being founded, which
            // `Build::activate` raised a few lines above this block.
            let i = (self.city_num(who) - 1).clamp(0, 7) as usize;
            let n = self.tuning.korean_citizens[i];
            self.free_train(b, power::KOREANS, ty::PEASANTS, n, false);
        }
    }

    /// The Barracks' four arms, in the order the original tests them.
    fn barracks_free_units(&mut self, b: usize, who: Player) {
        let scouts = self.tuning.iroquois_free_scout;
        self.free_train(b, power::IROQUOIS, ty::SCOUT, scouts, true);

        let age = self.nation_age(who);
        let g = &self.tuning;
        let legions = free_scaled(
            g.roman_barracks_legion,
            g.roman_max_legion,
            free_tier(
                age,
                g.roman_age_for_1_legion,
                g.roman_age_for_2_legions,
                g.roman_age_for_3_legions,
            ),
        );
        self.free_train(b, power::ROMANS, ty::HOPLITES, legions, true);

        // The British ladder has **no multiplier and no cap**: the tier is
        // the count, where the Roman and Aztec arms scale and clamp theirs.
        let g = &self.tuning;
        let archers = free_tier(
            age,
            g.british_age_for_1_archer,
            g.british_age_for_2_archers,
            g.british_age_for_3_archers,
        );
        self.free_train(b, power::BRITISH, ty::BOWMEN, archers, true);

        // The Aztec ladder is not the other two's: it is 1 below the first
        // age, then 2, then 3 past the third. Only the patch-4 arm is
        // modelled — the older one reads `ages` where this reads the min.
        let g = &self.tuning;
        let tier = if age == 0 { 1 } else { 2 + i32::from(age > 2) };
        let light = free_scaled(g.aztec_barracks_light, g.aztec_max_light, tier);
        self.free_train(b, power::AZTECS, ty::SLINGERS, light, true);
    }

    /// **`Build::process@0061edf0:362–381` — the Persians' Market trains a
    /// caravan** (`docs/AI.md` §146). Every fifteen frames of the Market's
    /// own phase, `(o + frame) % 15 == 0`, a Market (`0x1b4`) whose owner
    /// `has_tribe_bonus(0x17)` trains a `CARA` — straight through
    /// `Build::train`, no queue and no price — while the leader's `caras`
    /// (`+0x980`) is under `get_caravan_limit(1)` and
    /// `check_population(CARA)` says it fits. It is the stretch after
    /// `do_queue` and the gather tiles, behind the Kremlin's spy and the
    /// government hero and before the Terra Cotta's soldier, so the tick
    /// calls it after the gather step and only for an active building.
    ///
    /// Not the graft and not the upgrade: the call names `CARA` itself.
    /// The coverage pair's Persians: the Market `1/2018` completes on 661
    /// and trains `1/15` on 667, its first slot as an active building.
    pub(crate) fn persian_market_caravan(&mut self, b: usize, frame: i64) {
        // The type itself, `== 0x1b4`, not its lineage: the record's own
        // `Ident`.
        let market = self.buildings[b]
            .ty
            .is_some_and(|r| self.build_types[r].ident == crate::build::Ident::Market);
        if !market || self.buildings[b].phase(frame) % 15 != 0 {
            return;
        }
        let who = self.buildings[b].owner;
        let w = who as usize;
        if w >= self.tech.len()
            || !self
                .tech_tree
                .has_tribe_bonus(&self.setup, &self.tech[w], power::PERSIANS)
        {
            return;
        }
        let caras = self.ai.get(w).map_or(0, |l| l.census.caras);
        if caras >= self.caravan_limit(who) {
            return;
        }
        let Some(rec) = self.unit_record(ty::CARA) else {
            return;
        };
        let m = &self.muster[w];
        if crate::cost::exceeds_population(m.cap, m.control, self.unit_types[rec].price.pop) {
            return;
        }
        self.build_train(b, rec);
    }

    /// One arm: `n` copies of `base`'s line, each under `check_population`.
    ///
    /// `upgrade` is the arm's own shape — every arm but the American
    /// scholar, the French pair and the Korean citizen runs the graft's
    /// answer through `current_upgrade` first.
    fn free_train(&mut self, b: usize, power: usize, base: TypeId, n: i32, upgrade: bool) {
        if n <= 0 {
            return;
        }
        let who = self.buildings[b].owner;
        let w = who as usize;
        if !self
            .tech_tree
            .has_tribe_bonus(&self.setup, &self.tech[w], power)
        {
            return;
        }
        let Some(rec) = self.free_unit_record(who, base, upgrade) else {
            return;
        };
        let pop = self.unit_types[rec].price.pop;
        for _ in 0..n {
            let m = &self.muster[w];
            if crate::cost::exceeds_population(m.cap, m.control, pop) {
                continue;
            }
            self.build_train(b, rec);
        }
    }

    /// `get_graft` then, for all but three arms, `current_upgrade` — as an
    /// index into [`Sim::unit_types`].
    fn free_unit_record(&self, who: Player, base: TypeId, upgrade: bool) -> Option<usize> {
        let p = self.tech.get(who as usize)?;
        if base >= self.tech_tree.types.len() {
            return None;
        }
        let t = self.tech_tree.get_graft(&self.setup, p, Some(base))?;
        let t = if upgrade {
            self.tech_tree.current_upgrade(&self.setup, p, t)
        } else {
            t
        };
        self.unit_record(t)
    }

    /// `min(epoch[0], ages)` — the number the Roman, British and Aztec
    /// ladders are read against. The Military level and the age are
    /// **different fields** (`LeaderDataEncrypt +0xe8` and `+0xdc`), and the
    /// smaller of the two is the tier.
    fn nation_age(&self, who: Player) -> i32 {
        let p = &self.tech[who as usize];
        p.epoch[0].min(p.ages)
    }
}

/// The 1/2/3 ladder the Roman and British arms share: the highest threshold
/// `age` clears.
const fn free_tier(age: i32, one: i32, two: i32, three: i32) -> i32 {
    if age >= three {
        3
    } else if age >= two {
        2
    } else if age >= one {
        1
    } else {
        0
    }
}

/// `per × tier`, held to `max`. **A zero `max` skips the ladder entirely**
/// and pays `per` — the whole scaling block sits inside `if (max != 0)` —
/// and a negative either side pays nothing.
const fn free_scaled(per: i32, max: i32, tier: i32) -> i32 {
    if max == 0 {
        return per;
    }
    let n = per * tier;
    if n < 0 || max < 0 {
        0
    } else if n > max {
        max
    } else {
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::{self, Resource};

    fn sim() -> Sim {
        Sim::new(crate::Tuning::RON, crate::World::new(16, 16), 2)
    }

    /// The ladder the Roman and British Barracks arms share, on the
    /// shipped numbers. The British one is the *count* — no multiplier, no
    /// cap — which is why run53's AI is paid **one** archer at Ancient and
    /// not three.
    #[test]
    fn the_age_ladder_is_the_british_arm_s_whole_count() {
        let t = crate::Tuning::RON;
        let british = |age| {
            free_tier(
                age,
                t.british_age_for_1_archer,
                t.british_age_for_2_archers,
                t.british_age_for_3_archers,
            )
        };
        assert_eq!(british(0), 1, "BRITISH_AGE_FOR_1_ARCHER is zero");
        assert_eq!(british(1), 1);
        assert_eq!(british(2), 2);
        assert_eq!(british(3), 3);
        assert_eq!(british(7), 3, "and it stops at three");

        // The Roman one has both a multiplier and a cap, and starts an age
        // later: nothing at Ancient.
        let roman = |age| {
            free_scaled(
                t.roman_barracks_legion,
                t.roman_max_legion,
                free_tier(
                    age,
                    t.roman_age_for_1_legion,
                    t.roman_age_for_2_legions,
                    t.roman_age_for_3_legions,
                ),
            )
        };
        assert_eq!(roman(0), 0, "ROMAN_AGE_FOR_1_LEGION is one");
        assert_eq!(roman(1), 1);
        assert_eq!(roman(3), 2);
        assert_eq!(roman(5), 3);
        assert_eq!(roman(7), 3);
    }

    /// `free_scaled`'s two edges, both read off the decompile rather than
    /// guessed: a zero cap skips the ladder entirely and pays the plain
    /// per-building figure, and a negative either side pays nothing.
    #[test]
    fn a_zero_cap_skips_the_ladder_and_a_negative_pays_nothing() {
        assert_eq!(
            free_scaled(2, 0, 3),
            2,
            "the whole block sits inside max != 0"
        );
        assert_eq!(free_scaled(2, 5, 3), 5);
        assert_eq!(free_scaled(2, 5, 2), 4);
        assert_eq!(free_scaled(-1, 5, 2), 0);
        assert_eq!(free_scaled(2, -1, 2), 0);
    }

    #[test]
    fn the_roster_index_is_the_power_the_rules_name() {
        // The indices `docs/TECH.md` pins at `has_preq`, each of which is a
        // `has_tribe_bonus(n)` this crate already hardcodes.
        assert_eq!(power_of("Nubians"), Some(4));
        assert_eq!(power_of("Greeks"), Some(5));
        assert_eq!(power_of("Romans"), Some(6));
        assert_eq!(power_of("Egyptians"), Some(7));
        assert_eq!(power_of("French"), Some(10));
        assert_eq!(power_of("British"), Some(11));
        assert_eq!(power_of("Germans"), Some(12));
        assert_eq!(power_of("Russians"), Some(13));
        assert_eq!(power_of("Koreans"), Some(16));
        assert_eq!(power_of("Iroquois"), Some(18));
        assert_eq!(power_of("Lakota"), Some(0x13));
        assert_eq!(power_of("Dutch"), Some(0x16));
        assert_eq!(power_of("Persians"), Some(23));
        assert_eq!(power_of("Prussians"), None);
    }

    #[test]
    fn the_dump_s_tribe_turns_one_flag_on_and_the_rest_off() {
        let mut s = sim();
        s.set_tribe(0, 11);
        assert_eq!(s.tech[0].power, Some(11));
        assert!(s.nation[0].british);
        assert!(!s.nation[0].french);
        assert!(!s.nation[0].nubians);

        // Nubians on the other side, from the same capture.
        s.set_tribe(1, 4);
        assert!(s.nation[1].nubians);
        assert!(!s.nation[1].british);
        // And the first player is untouched by the second's.
        assert!(s.nation[0].british);
    }

    #[test]
    fn a_tribe_of_minus_one_is_a_leader_with_no_nation() {
        let mut s = sim();
        s.set_tribe(0, 11);
        assert!(s.nation[0].british);
        s.set_tribe(0, -1);
        assert_eq!(s.tech[0].power, None);
        assert_eq!(s.tech[0].tribe, 0, "the graft table stays in range");
        assert!(!s.nation[0].british);
        assert!(!s.nation[0].aztecs, "and −1 is not roster index 0");
    }

    #[test]
    fn the_lobby_s_no_nation_powers_puts_every_flag_out() {
        let mut s = sim();
        s.setup.no_nation_powers = true;
        s.set_tribe(0, 11);
        assert_eq!(s.tech[0].power, Some(11), "the nation is still the nation");
        assert!(!s.nation[0].british, "but the power is off");

        // And a leader with no city takes none of it either — in a nomad
        // lobby. With a starting town the gate is `starting_town || city_num`
        // and the first half is true (group 22).
        let mut t = sim();
        t.tech[0].has_city = false;
        t.set_tribe(0, 11);
        assert!(t.nation[0].british, "a starting town opens the gate");
        let mut n = sim();
        n.setup.starting_town = false;
        n.tech[0].has_city = false;
        n.set_tribe(0, 11);
        assert!(!n.nation[0].british, "nomads have no power without a city");
        n.tech[0].has_city = true;
        n.refresh_nation_powers(0);
        assert!(n.nation[0].british, "and the first city turns it on");
    }

    /// The measurement the wire was worth: run40's AI is the British and
    /// carries a commerce cap of 1392 where the human carries 1120.
    #[test]
    fn run40_s_british_cap_follows_from_the_dump_s_tribe() {
        let mut s = sim();
        s.set_tribe(0, 4); // the human, Nubians
        s.set_tribe(1, 11); // the AI, British
        s.assemble_holdings(0);
        s.assemble_holdings(1);
        let t = &s.tuning;
        let cap = |w: usize| economy::commerce_cap(t, &s.holdings[w], Resource::Food);
        assert_eq!(cap(0), 1120, "70 x 16");
        assert_eq!(cap(1), 1392, "70 x 125 / 100 = 87, then x 16");
    }
}
