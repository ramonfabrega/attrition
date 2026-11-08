//! **The armed building family's overrides** (item 3970, `docs/GOLDEN.md`
//! §176; parked T71-8): what `BuildData` adds to `ObjectData`'s attack, reach
//! and arrows for a Tower-line, Fort-line or city-centre building.
//!
//! - `BuildData::attack@0062e610` — [`crate::build_attack_view`]'s swept
//!   walk, now wired: a General inside (`GENERAL_BUILDING_ATTACK × 10`), an
//!   Obsidian holder (`OBSIDIAN_ARCHERS_ATTACK × 10`) and Antipater inside
//!   (`ANTIPATER_GARRISON_ATTACK_BONUS × 10`).
//! - `BuildTypeData::get_building_range@00639990` and
//!   `BuildData::max_range@0062e470` — the tier arrays
//!   (`TOWER_FORT_RANGE`, `FORT_UPGRADE_RANGE`), the Colosseum, British and
//!   Roman terms, the General's `GENERAL_BUILDING_RANGE` and a templed city
//!   centre's `TEMPLE_UPGRADE_RANGE` (`LeaderData::get_temple_range@
//!   006db4a0`, Tikal's scaling on it).
//! - `BuildData::get_base_garrison_arrows@0062e340` — the Maya's
//!   `MAYA_GARRISON_ARROWS` on every building but a city centre whose `race`
//!   is not its owner.
//!
//! The garrison class every one of them asks is the building's `is(0x1b7, 0)`
//! (the Tower line, [`build::is_tower`]), the type's slot `+0xfc`
//! (`is_fort`, [`build::is_fort`]), or — `attack` and `max_range`'s second
//! block only — the city flag (`flags & 0x20`, [`build::is_city`]) with a
//! city.
//!
//! SEAM: `tribe_can_type(General)` is taken as yes, so a nation whose
//! General the tribe mask refuses counts the General line and not the type
//! its slot `+0x68` names.

use crate::build;
use crate::build_attack_view::{BuildAttackView, build_attack_steps};
use crate::combat::Obj;
use crate::{Player, Sim};

/// `TypeIndex::GENERAL`, the line `count_inside(COUNT_TYPE)` asks.
const GENERAL: crate::tech::TypeId = 0x36;
/// `TypeIndex::ANTIPATER`, asked strictly.
const ANTIPATER: i32 = 0x169;

struct SimBuildAttack<'a> {
    sim: &'a Sim,
    b: usize,
}

impl BuildAttackView for SimBuildAttack<'_> {
    fn base_attack(&mut self) -> i32 {
        0
    }
    fn is_base_is(&mut self) -> bool {
        true
    }
    fn type_slot_60(&mut self) -> i32 {
        i32::from(self.sim.is_tower_building(self.b))
    }
    fn is_override(&mut self) -> i32 {
        i32::from(self.sim.is_tower_building(self.b))
    }
    fn type_slot_fc(&mut self) -> i32 {
        i32::from(self.sim.is_fort_building(self.b))
    }
    fn flag_20(&mut self) -> bool {
        self.sim.is_city_building(self.b)
    }
    fn city(&mut self) -> i16 {
        self.sim.buildings[self.b].city.map_or(-1, |c| c as i16)
    }
    fn tribe_can_type(&mut self) -> bool {
        true
    }
    fn type_slot_68_who(&mut self) -> i32 {
        GENERAL as i32
    }
    fn count_inside(&mut self, kind: i32, ty: i32) -> i32 {
        self.sim.count_inside_type(self.b, kind, ty)
    }
    fn obsidian(&mut self) -> bool {
        self.sim
            .has_rare(self.sim.buildings[self.b].owner, crate::economy::OBSIDIAN)
    }
    fn antipater(&mut self) -> bool {
        self.sim
            .muster_count(self.sim.buildings[self.b].owner, ANTIPATER)
            != 0
    }
    fn general_building_attack(&mut self) -> i32 {
        self.sim.tuning.general_building_attack
    }
    fn obsidian_archers_attack(&mut self) -> i32 {
        self.sim.tuning.obsidian_archers_attack
    }
    fn antipater_garrison_attack_bonus(&mut self) -> i32 {
        self.sim.tuning.antipater_garrison_attack_bonus
    }
}

impl Sim {
    fn is_tower_building(&self, b: usize) -> bool {
        self.buildings[b]
            .ty
            .is_some_and(|t| build::is_tower(&self.build_types, t))
    }

    fn is_fort_building(&self, b: usize) -> bool {
        self.buildings[b]
            .ty
            .is_some_and(|t| build::is_fort(&self.build_types, t))
    }

    fn is_city_building(&self, b: usize) -> bool {
        self.buildings[b]
            .ty
            .is_some_and(|t| build::is_city(&self.build_types, t))
    }

    /// The leader's per-type count (`+0x56fe + 2 × t`) of the unit type
    /// whose `TypeIndex` is `t`.
    fn muster_count(&self, who: Player, t: i32) -> i32 {
        self.unit_types
            .iter()
            .position(|u| u.type_index == t)
            .and_then(|r| self.muster.get(who as usize).and_then(|m| m.by_type.get(r)))
            .copied()
            .unwrap_or(0)
    }

    /// `ObjectData::count_inside@0064dda0`'s `COUNT_TYPE` (`is(ty, 0)`, the
    /// line) and `COUNT_STRICT_TYPE` (`is(ty, 1)`, the type) arms: the
    /// living units inside a building on the map. Any other kind counts 0.
    pub(crate) fn count_inside_type(&self, b: usize, kind: i32, ty: i32) -> i32 {
        use crate::build_attack_view::{COUNT_STRICT_TYPE, COUNT_TYPE};
        let bd = &self.buildings[b];
        if !bd.alive {
            return 0;
        }
        let n = bd
            .garrison
            .iter()
            .filter(|&&u| {
                let x = &self.units[u];
                x.inside == Some(b)
                    && x.alive()
                    && match kind {
                        COUNT_TYPE => self.unit_line_is(u, ty as crate::tech::TypeId),
                        COUNT_STRICT_TYPE => x.type_index == ty,
                        _ => false,
                    }
            })
            .count();
        n as i32
    }

    /// `BuildData::attack@0062e610` past `ObjectData::attack`: the three
    /// garrison terms, 0 for a building outside the garrison class.
    pub(crate) fn build_attack_terms(&self, b: usize) -> i32 {
        build_attack_steps(&mut SimBuildAttack { sim: self, b })
    }

    /// `count_inside(COUNT_TYPE, GENERAL) != 0` — `max_range`'s question,
    /// asked of both its blocks.
    fn holds_general(&self, b: usize) -> bool {
        self.count_inside_type(b, crate::build_attack_view::COUNT_TYPE, GENERAL as i32) != 0
    }

    /// `BuildTypeData::get_building_range@00639990` and
    /// `BuildData::max_range@0062e470` past the type's own range: what a
    /// building's reach gains, in tiles.
    ///
    /// 1. A Tower- or Fort-line building adds `COLOSSEUM_FORT_RANGE` for a
    ///    Colosseum holder and `BRITISH_TOWER_RANGE` for a British owner
    ///    (`docs/GOLDEN.md` §59).
    /// 2. A Fort-line building then adds `ROMAN_FORT_RANGE` for a Roman and
    ///    `FORT_UPGRADE_RANGE` at the highest `FORTRANGE2..4` held; a
    ///    Tower-line one that is not a fort adds `ROMAN_FORT_RANGE` and
    ///    `TOWER_FORT_RANGE` at [`Sim::tower_fort_los`] − 1 — the same three
    ///    bits of the tech mask's byte 8.
    /// 3. A Tower- or Fort-line building holding a General adds
    ///    `GENERAL_BUILDING_RANGE`.
    /// 4. A city centre with a city adds, when that city holds a Temple,
    ///    `TEMPLE_UPGRADE_RANGE` at the highest `TEMPLERANGE2..4` held,
    ///    raised by `TIKAL_TEMPLE_HP` percent rounded up for a Tikal holder;
    ///    and `GENERAL_BUILDING_RANGE` again for a General inside.
    pub(crate) fn building_range_terms(&self, b: usize) -> i32 {
        let bd = &self.buildings[b];
        let who = bd.owner;
        let n = &self.nation[who as usize];
        let held = self.wonders_held(who);
        let wonder = |w: usize| held & (1 << w) != 0;
        let (tower, fort) = (self.is_tower_building(b), self.is_fort_building(b));
        let mut r = 0;
        if tower || fort {
            if wonder(crate::tech::wonder::COLOSSEUM) {
                r += self.tuning.colosseum_fort_range;
            }
            if n.british {
                r += self.tuning.british_tower_range;
            }
        }
        if fort {
            if n.romans {
                r += self.tuning.roman_fort_range;
            }
            r += self.tuning.fort_upgrade_range
                [self.bonus_level(who, &self.tech_tree.roles.fort_range_preq)];
        } else if tower {
            if n.romans {
                r += self.tuning.roman_fort_range;
            }
            r += self.tuning.tower_fort_range[(self.tower_fort_los(who) - 1) as usize];
        }
        if (tower || fort) && self.holds_general(b) {
            r += self.tuning.general_building_range;
        }
        if std::env::var_os("RON_PROBE_DOCK").is_some() {
            let up = if fort {
                self.tuning.fort_upgrade_range
                    [self.bonus_level(who, &self.tech_tree.roles.fort_range_preq)]
            } else {
                0
            };
            let held_gen = (tower || fort) && self.holds_general(b);
            if up != 0 || held_gen {
                eprintln!("FORTPROBE b={b} who={who} fort={fort} up={up} general={held_gen} r={r}");
            }
        }
        if let Some(c) = bd.city
            && self.is_city_building(b)
        {
            if self.cities[c].alive && self.city_has(c, build::Ident::Temple) {
                let mut t = self.tuning.temple_upgrade_range
                    [self.bonus_level(who, &self.tech_tree.roles.temple_range_preq)];
                if wonder(crate::tech::wonder::TIKAL) {
                    t = ((self.tuning.tikal_temple_hp + 100) * t + 99) / 100;
                }
                r += t;
            }
            if self.holds_general(b) {
                r += self.tuning.general_building_range;
            }
        }
        r
    }

    /// `BuildData::get_base_garrison_arrows@0062e340`: the type's
    /// `BASE_ARROWS`, plus `MAYA_GARRISON_ARROWS` for a Mayan owner
    /// (`has_tribe_bonus(1)`) unless the building is a finished city centre
    /// whose city's `race` is not its owner (`+0x5f != +0x5e`).
    pub(crate) fn base_garrison_arrows(&self, b: usize) -> i32 {
        let bd = &self.buildings[b];
        let base = self.profile(Obj::Building(b)).base_arrows;
        if !self.nation[bd.owner as usize].maya {
            return base;
        }
        if self.is_city_building(b)
            && bd.active
            && let Some(c) = bd.city
            && self.cities[c].race != Some(bd.owner)
        {
            return base;
        }
        base + self.tuning.maya_garrison_arrows
    }
}
