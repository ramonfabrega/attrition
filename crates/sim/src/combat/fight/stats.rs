//! The combat stats: the table's percent, attack, armour and range with their nation arms, the side terms and the range tests (STRUCTURE §5, move 7).

use super::*;

impl Sim {
    // ------------------------------------------------------------------
    // Reading an object
    // ------------------------------------------------------------------

    /// The combat table's entry for an attacker and a target of either
    /// family — `return_modifier` over `TypeIndex`, here over unit ids and
    /// building ids. 100 where an object has no type.
    pub(super) fn table_pct(&self, attacker: Obj, target: Obj) -> i32 {
        let at = |o: Obj| match o {
            Obj::Unit(u) => self.units[u].ty.map(combat::TypeRef::Unit),
            Obj::Building(b) => self.buildings[b].ty.map(combat::TypeRef::Build),
        };
        match (at(attacker), at(target)) {
            (Some(a), Some(b)) => self.combat.table.pct_of(a, b),
            _ => 100,
        }
    }

    /// The combat profile of an object.
    pub fn profile(&self, o: Obj) -> Profile {
        match o {
            Obj::Unit(i) => self.units[i].ty.map_or_else(no_profile, |t| {
                let mut p = self.tables.unit_types[t].combat;
                if self.combat.second_range == Some(t) {
                    p.max_range = p.second_max_range;
                }
                p
            }),
            Obj::Building(b) => self.buildings[b].combat.unwrap_or_default(),
        }
    }

    /// **`f` with a melee-and-ranged type's second range in its
    /// `max_range`** — the bracket `Unit::cavarch_fight@005ff4b0` puts
    /// round its `fight` (`005ff4c5`, `005ff4f0`): for a type whose flags
    /// carry `0x400` the type's `+0x1fc` reads its `+0x2d0` for the call
    /// and 0 after it, which is the value [`uflags::MELEE_AND_RANGED`]
    /// leaves there at load ([`Sim::second_range`]).
    pub(crate) fn with_second_range<R>(&mut self, u: usize, f: impl FnOnce(&mut Self) -> R) -> R {
        let swap = self.units[u].ty.filter(|&t| {
            self.tables.unit_types[t]
                .cols
                .flag(uflags::MELEE_AND_RANGED)
        });
        let Some(t) = swap else {
            return f(self);
        };
        let held = self.combat.second_range.replace(t);
        let r = f(self);
        self.combat.second_range = held;
        r
    }

    pub(crate) fn owner_of(&self, o: Obj) -> Player {
        match o {
            Obj::Unit(i) => self.units[i].owner,
            Obj::Building(b) => self.buildings[b].owner,
        }
    }

    pub(crate) fn pos_of(&self, o: Obj) -> Pos {
        match o {
            Obj::Unit(i) => self.units[i].pos,
            Obj::Building(b) => self.buildings[b].pos,
        }
    }

    /// `flags & 1` and, for a unit, `is_on_map`.
    pub(crate) fn active(&self, o: Obj) -> bool {
        match o {
            Obj::Unit(i) => self.units.get(i).is_some_and(|u| u.alive() && u.on_map),
            Obj::Building(b) => self
                .buildings
                .get(b)
                .is_some_and(|b| b.alive && b.combat.is_some() && b.health > 0),
        }
    }

    /// **A thing a landing round or a blow can still strike**
    /// (`Ammo::hit_target@00678f90`, `Object::do_damage@0064a480`): the
    /// slot's `flags & 1` and, for a unit, `is_on_map` — and nothing about
    /// hit points. [`Sim::active`] also asks a building for health left,
    /// which is the target *search's* question; a city building at its
    /// ceiling (`health` 0) is still struck by the round already in the air
    /// (item 1528: run668's second siege shot on 15377, which charges the
    /// army a second time).
    pub(crate) fn struck(&self, o: Obj) -> bool {
        match o {
            Obj::Unit(_) => self.active(o),
            Obj::Building(b) => self
                .buildings
                .get(b)
                .is_some_and(|b| b.alive && b.combat.is_some()),
        }
    }

    /// `attack()` — the type's, plus the player's flat modifier (§4.1).
    ///
    /// The base is `ObjectData::attack@006469f0`'s
    /// ([`crate::combat::object_attack_view::object_attack_steps`], swept: item
    /// 3387). Its caravan arm replaces a caravan's attack with
    /// `CARAVAN_ATTACK_BONUS` once its owner holds that bonus's
    /// prerequisite, and its Dutch term (item 3329) is inside it: a Dutch
    /// kind ([`Sim::dutch_ages`]) adds `ages × DUTCH_ATTACK_BONUS × 10`
    /// **before** `UnitData::attack`'s zero test, so an armed merchant of
    /// an age past the first is a combatant on a type with no attack.
    pub fn attack_of(&self, o: Obj) -> i32 {
        let base = crate::combat::object_attack_view::object_attack_steps(&mut SimObjectAttack {
            sim: self,
            o,
        });
        if base == 0 {
            return 0;
        }
        let family = match o {
            Obj::Building(b) => self.build_attack_terms(b),
            Obj::Unit(_) => 0,
        };
        base + self.mods_of(self.owner_of(o) as usize).attack
            + self.citizen_attack(o)
            + self.monarch_cav_attack(o)
            + family
    }

    /// **`UnitData::attack@006103c0`'s Monarch arm** (item 3688, `docs/AI.md`
    /// §349), the override's last term (`610800`–`61083c`): a unit whose
    /// type's trainer is the Stable (`type +0x40 == 0x1ac`, by equality —
    /// [`Sim::trainer_ident`]) and which has one of its owner's Monarchs
    /// (`0x162`) in reach — `has_general(0, 0x162) >= 0`,
    /// [`Sim::has_general`] — adds `THEMONARCH_CAV_ATTACK × 10`. The gate
    /// ahead of both, the leader's short `+0x59c2`, is `+0x56fe`'s
    /// per-type hero count at `0x162` (`0x56fe + 2 × 0x162`), which a live
    /// Monarch the walk finds already implies. run1357's packet (logger
    /// frame 3439 of the Russian Gunpowder Great Lakes game) runs the
    /// original's `get_damage` on who=1's Heavy Knight `1/32` and the
    /// human's Citizen `0/5`: table 150, armour 0 and attack **190**, the
    /// base 170 and 20 off this arm (`has_general` answered 18 at
    /// `0x61082a`, the Monarch `1/18` the knight guards), so the blow is 29
    /// where this crate dealt 26.
    ///
    /// SEAM: the leader's per-type count is not read (a Monarch counted
    /// and off the map), as in [`Sim::near_monarch`].
    pub(super) fn monarch_cav_attack(&self, o: Obj) -> i32 {
        match o {
            Obj::Unit(u)
                if self.units[u].ty.is_some_and(|ty| {
                    self.trainer_ident(ty) == Some(crate::cities::build::Ident::Stable)
                }) && self.has_general(u, 0x162) =>
            {
                self.tables.tuning.themonarch_cav_attack * 10
            }
            _ => 0,
        }
    }

    /// **`UnitData::attack@006103c0`'s Citizen arm** (item 3677, `docs/AI.md`
    /// §346): a unit with one of its owner's Citizens (`0x164`, THECITIZEN)
    /// in reach — `has_general(0, 0x164) >= 0`, [`Sim::has_general`] — adds
    /// `THECITIZEN_ATTACK_BONUS × 10`. run1351c's packet (logger frame 3521
    /// of the German Industrial Great Lakes game) runs the original's
    /// `get_damage` on who=1's Armored Car `1/27` and the human's Citizen
    /// `0/13`: table 170, armour 0 and attack **220**, the base 200 and 20
    /// off this arm (`has_general` answered 17, the Citizen `1/17` 368
    /// units off), so the blow is 37 where this crate dealt 34.
    ///
    /// SEAM: the override's other terms (Terra Cotta, the Obsidian Archers,
    /// the Space Program's scale and the named generals') — the view
    /// [`crate::combat::unit_attack_view`] holds them all, swept, unwired.
    pub(super) fn citizen_attack(&self, o: Obj) -> i32 {
        match o {
            Obj::Unit(u) if self.has_general(u, 0x164) => {
                self.tables.tuning.thecitizen_attack_bonus * 10
            }
            _ => 0,
        }
    }

    /// `ages × DUTCH_ATTACK_BONUS` for a unit of a Dutch kind, else 0
    /// (`ObjectData::attack`'s and `::armor`'s shared arm; a building is
    /// never one).
    pub(super) fn dutch_term(&self, o: Obj) -> i32 {
        match o {
            Obj::Unit(u) => self.units[u].ty.map_or(0, |rec| {
                self.dutch_ages(self.units[u].owner, rec) * self.tables.tuning.dutch_attack_bonus
            }),
            Obj::Building(_) => 0,
        }
    }

    /// The damage formula's modifiers of an owner: the player's, or none
    /// for a gaia owner (animals, goods), which the leaders' list holds as
    /// zeroes and `mods` — one a player — does not (a nuke's ring strikes
    /// the animals around a city, `docs/AI.md` §158).
    ///
    /// **The Japanese level is read live** (item 3590): `get_damage@
    /// 00644130:243`–`262` asks `has_tribe_bonus(0xf)` of the attacker's
    /// leader on every blow and takes `n = min(ages, epoch[0])` off its
    /// `LeaderData` there, so nothing caches it; before this nothing wrote
    /// `japanese_level` at all (parked 3510a), and run1208d's Arquebusiers
    /// `1/20` struck the Citizen `0/12` for 12 where the original's +15 %
    /// (ages 3, `epoch[0]` 3) struck 13 + 10/16 (`docs/AI.md` §316).
    pub(super) fn mods_of(&self, owner: usize) -> combat::Modifiers {
        let mut m = self.combat.mods.get(owner).copied().unwrap_or_default();
        if let Some(p) = self.tech.get(owner)
            && self
                .tables
                .tech_tree
                .has_tribe_bonus(&self.tables.setup, p, 0xf)
        {
            m.japanese_level = Some(p.ages.min(p.epoch[crate::tech::Line::Military.index()]));
        }
        m
    }

    /// `armor()` (§4.2). **A building that is not finished holds half its
    /// armour** (item 1350): a building's `armor()` is vslot `+0x124`,
    /// `WallData::armor@0063fa60` (read off the PE at `Build::vftable +
    /// 0x124`), and its tail at `63fb25`–`63fb41` tests `is_active`
    /// (`flags & 4`) and, when it is clear, halves with `cltd; sub; sar`
    /// — toward zero, as `/ 2` does here. run514's Keep site, armour 4,
    /// took a Hoplite's blow at 2: 13 whole hits, 69 sixteenths a figure.
    ///
    /// **The Senator's arm above the halving** (`63fab0`–`63fb1f`, item
    /// 3986, `docs/GOLDEN.md` §179): with a leader whose `+0x59c0` is set
    /// (`+0x56fe`'s per-type count at `0x161`), `HeroesData::find_hero@
    /// 0073a1b0` for one of the owner's `THESENATOR`s (`0x161`) whose
    /// radius covers the building less its footprint's `(type +0x234 +
    /// +0x238) × 96` adds `THESENATOR_BUILD_ARMOR` — before the halving,
    /// so an unfinished building under a Senator holds half the sum
    /// ([`Sim::hero_near_building`]).
    /// **`ObjectData::armor@00647db0`** alone — the profile's armour, the
    /// Dutch term and the owner's modifiers, without a unit's rally or a
    /// building's Senator and halving: what `Unit::update_armor@006054c0`
    /// caches into `UnitData::myarmor` (`+0x9c`). Read by the units line
    /// (`docs/CHECKSUM.md` §17); the cattle citizens' term is not added.
    pub(crate) fn object_armor(&self, o: Obj) -> i32 {
        self.profile(o).armor + self.dutch_term(o) + self.mods_of(self.owner_of(o) as usize).armor
    }

    pub fn armor_of(&self, o: Obj) -> i32 {
        // `ObjectData::armor@00647db0`'s Dutch term (item 3329): `ages ×
        // DUTCH_ATTACK_BONUS` on a Dutch kind.
        let armor = self.profile(o).armor
            + self.dutch_term(o)
            + self.mods_of(self.owner_of(o) as usize).armor;
        match o {
            Obj::Unit(u) => armor + self.rally_armor(u),
            Obj::Building(b) => {
                let armor = armor + self.senator_build_armor(b);
                if self.buildings[b].active {
                    armor
                } else {
                    armor / 2
                }
            }
        }
    }

    /// `WallData::armor@0063fa60`'s Senator term: `THESENATOR_BUILD_ARMOR`
    /// when one of the owner's Senators stands within reach of `b`. The
    /// arm's first branch — the object itself `is(0x161)` behind vslot
    /// `+0x18` — is a unit's, never a building's.
    pub(super) fn senator_build_armor(&self, b: usize) -> i32 {
        const THESENATOR: i32 = 0x161;
        if self.hero_near_building(b, THESENATOR) {
            self.tables.tuning.thesenator_build_armor
        } else {
            0
        }
    }

    /// **`UnitData::armor@00610160`'s rally arm** (item 3385): a unit that
    /// is itself a hero, or that `HeroesData::find_hero@0073a1b0` places
    /// inside one of its owner's heroes' radii, adds `GENERAL_RALLY_ARMOR ×
    /// (general_upgrade + 1)`. `find_hero` is asked with no mask, no type
    /// and no reach (vslot `+0x20` is clear for a unit): any active hero
    /// on the map whose `get_radius × 0xc0` covers the unit's position by
    /// `vector_dist`. run1011e's packet: the Horse Archer `1/72`'s round on
    /// the Inti Maceman `1/61` on 13297, `get_damage` 13 at armour 4 (the
    /// cached `myarmor` 2 and the General's 2) where this crate dealt 15.
    ///
    /// **The patriots' terms inside it** (item 3986, `docs/GOLDEN.md`
    /// §179), each its own `has_general` walk: `has_general(0, 0x164)`
    /// (`THECITIZEN`, shipped as The Comrade) adds
    /// `THECITIZEN_ARMOR_BONUS`, and a Stable-trained unit (`type +0x40 ==
    /// 0x1ac`, [`Sim::trainer_ident`]) with `has_general(0, 0x162)` (The
    /// Monarch) adds `THEMONARCH_CAV_ARMOR`. Either hero in reach opens
    /// the gate itself, so the rally term is always beside them.
    ///
    /// SEAM: the arm's named generals between the two — Ptolemy, Darius,
    /// Wellington, Blücher, Memnon — are not carried; each is gated on a
    /// leader's counter of its own a skirmish leaves at 0 (territory
    /// adjudication 4, row 61).
    pub(super) fn rally_armor(&self, u: usize) -> i32 {
        let me = &self.units[u];
        let rallied = (me.alive() && self.is_hero_unit(u))
            || (0..self.units.len()).any(|h| {
                let x = &self.units[h];
                x.owner == me.owner
                    && x.alive()
                    && x.on_map
                    && self.is_hero_unit(h)
                    && crate::world::vector_dist(
                        (x.pos.x - me.pos.x).abs(),
                        (x.pos.y - me.pos.y).abs(),
                    ) <= self.hero_radius(h) * 0xc0
            });
        if !rallied {
            return 0;
        }
        let citizen = if self.has_general(u, 0x164) {
            self.tables.tuning.thecitizen_armor_bonus
        } else {
            0
        };
        let monarch = if me
            .ty
            .is_some_and(|ty| self.trainer_ident(ty) == Some(crate::cities::build::Ident::Stable))
            && self.has_general(u, 0x162)
        {
            self.tables.tuning.themonarch_cav_armor
        } else {
            0
        };
        self.tables.tuning.general_rally_armor * (self.general_upgrade_level(me.owner) + 1)
            + citizen
            + monarch
    }

    /// `max_range()` (§4.4): zero stays zero.
    pub fn max_range_of(&self, o: Obj) -> i32 {
        let p = self.profile(o);
        if p.max_range == 0 {
            return 0;
        }
        p.max_range
            + self.mods_of(self.owner_of(o) as usize).range
            + match o {
                Obj::Building(b) => self.building_range_terms(b),
                Obj::Unit(_) => 0,
            }
            + self.ceo_unit_range(o)
            + self.siege_unit_range(o)
    }

    /// **`UnitData::max_range@0060fe90`'s siege arm** (item 3380): a unit
    /// whose type is `is_siege` (the type's vslot `+0x10c`, the same test
    /// [`Self::ceo_unit_range`] refuses on) and whose leader
    /// `has_tribe_bonus(8)` adds `TURK_SIEGE_RANGE`, three tiles.
    /// `GreatLakesTurkishTier4`'s Trebuchet `1/70` walks its bombard ring
    /// round Napata `0/2000` on 14991 at a stand-off of 19 tiles' 3552, not
    /// 16's 2976, and stops at (7368, 29544) in run1005.
    ///
    /// SEAM: the same arm's Liberty (`0x219`, `LIBERTY_SIEGE_RANGE`) and
    /// Eiffel (`0x21c`, `EIFFEL_SIEGE_RANGE`) wonders are not carried; no
    /// capture holds either wonder (scan: `grep -h '^    wonder [0-9]'
    /// gamelog*.txt`, no wonder held in the 66 of 573 dumps that print
    /// `WONDERS`).
    pub(super) fn siege_unit_range(&self, o: Obj) -> i32 {
        let Obj::Unit(u) = o else {
            return 0;
        };
        if !self.is_siege_unit(u) {
            return 0;
        }
        let who = self.units[u].owner as usize;
        if self.cities.nation[who].turks {
            self.tables.tuning.turk_siege_range
        } else {
            0
        }
    }

    /// **`UnitData::max_range@0060fe90`'s first block** (`docs/AI.md`
    /// §236): a unit whose type is not `is_siege` (the type's vslot
    /// `+0x10c`), whose leader counts a `0x165` (`+0x56fe + 0x165 × 2`,
    /// the per-type count at `+0x59c8`), and which `has_general(0, 0x165)`
    /// — it is the `0x165` hero itself, or `HeroesData::find_hero@0073a1b0`
    /// finds one of its owner's, active and on the map, within the hero's
    /// `get_radius × 0xc0` (`vector_dist(|dx|, |dy|)`, less a building's
    /// footprint, 0 for a unit) — adds `THECEO_UNIT_RANGE`, two tiles.
    /// run881's Strategic Bomber `1/137` bombs `0/2000` on 6452 at
    /// `attack_dist` 918, past three tiles' 582 and inside five's 966, with
    /// who=1's `1/161` (type 357) 2371 off.
    ///
    /// SEAM: of the siege arm below it only `TURK_SIEGE_RANGE` is carried
    /// (item 3380); the Liberty and Eiffel wonders and Napoleon are not —
    /// nor Ptolemy's general, the rum naval range
    /// and the obsidian archers are not carried; `is(0x165)` is taken as
    /// the type itself, as the patriot tests of the radius are.
    pub(super) fn ceo_unit_range(&self, o: Obj) -> i32 {
        let Obj::Unit(u) = o else {
            return 0;
        };
        if self.near_own_ceo(u) {
            self.tables.tuning.theceo_unit_range
        } else {
            0
        }
    }

    /// The CEO block `UnitData::max_range@0060fe90` and
    /// `UnitData::los@006100c0` share: the unit is not `is_siege` (the
    /// type's vslot `+0x10c`), its leader counts a `0x165` (`+0x59c8`), and
    /// `has_general(0, 0x165)` — it is the `0x165` hero itself, or one of its
    /// owner's stands active and on the map within the hero's
    /// `get_radius × 0xc0` (`vector_dist`).
    pub(crate) fn near_own_ceo(&self, u: usize) -> bool {
        const CEO: i32 = 0x165;
        if self.is_siege_unit(u) {
            return false;
        }
        let who = self.units[u].owner;
        let counted = self
            .tables
            .unit_types
            .iter()
            .position(|t| t.type_index == CEO)
            .and_then(|r| {
                self.roster
                    .muster
                    .get(who as usize)
                    .and_then(|m| m.by_type.get(r))
            })
            .is_some_and(|&n| n != 0);
        if !counted {
            return false;
        }
        let at = self.units[u].pos;
        self.units[u].type_index == CEO && self.is_hero_unit(u)
            || (0..self.units.len()).any(|h| {
                let x = &self.units[h];
                x.owner == who
                    && x.type_index == CEO
                    && x.alive()
                    && x.on_map
                    && self.is_hero_unit(h)
                    && crate::world::vector_dist((x.pos.x - at.x).abs(), (x.pos.y - at.y).abs())
                        <= self.hero_radius(h) * 0xc0
            })
    }

    pub(super) fn hits_left(&self, o: Obj) -> i32 {
        match o {
            Obj::Unit(i) => self.units[i].health.max(0),
            Obj::Building(b) => self.buildings[b].health.max(0),
        }
    }

    /// `WallData::in_unfriendly_territory@0063eca0`: the cell under the
    /// building is owned (`>= 0`) by a player who is not its owner and not
    /// allied with it both ways (`diplos == 2` each way). `get_damage` reads
    /// it through the target's vslot `+0x184` (step 18, `0063eca0` off the
    /// PE at `Build::vftable`, `BuildData::vftable` and `WallData::vftable`
    /// alike), and there it zeroes the armour (item 1375: run551's Tower
    /// site on who=0's claim took the Elite Pikemen's blow at 133
    /// sixteenths, where its halved armour 2 dealt 122). The same predicate
    /// as `Wall::process`'s enemy-land hit (`crate::cities::city`).
    pub(crate) fn in_unfriendly_territory(&self, b: usize) -> bool {
        let who = self.buildings[b].owner;
        matches!(self.world.owner_at(self.buildings[b].pos),
            crate::Owner::Player(t) if !crate::cities::place::friendly_ground(
                i32::from(t), i32::from(who), self.allied_with(who, t), self.allied_with(t, who)))
    }

    /// `Wall::process@00640450`'s enemy-land owner, `:209`–`:219`: the
    /// cell under the building, or for a dock (`is_dock`, the type's vslot
    /// `+0x108`) [`Sim::check_enemy_adjacent`]'s ring in its place; then
    /// the same three refusals — below 0, the owner's own, an ally both
    /// ways (item 3992, `docs/GOLDEN.md` §185). `get_damage`'s armour read
    /// ([`Self::in_unfriendly_territory`], `WallData::in_unfriendly_territory`)
    /// asks the own cell for a dock too.
    pub(crate) fn bleeds_in_enemy_land(&self, b: usize) -> bool {
        let dock = self.buildings[b]
            .ty
            .is_some_and(|t| crate::cities::build::is_dock(&self.tables.build_types, t));
        if !dock {
            return self.in_unfriendly_territory(b);
        }
        let who = self.buildings[b].owner;
        let o = self.check_enemy_adjacent(b);
        u8::try_from(o).is_ok_and(|p| {
            !crate::cities::place::friendly_ground(
                o,
                i32::from(who),
                self.allied_with(who, p),
                self.allied_with(p, who),
            )
        })
    }

    /// The side of the damage formula an object presents as a **target**.
    pub(super) fn target_side(&self, o: Obj, attacker: Obj) -> Side {
        match o {
            Obj::Unit(i) => {
                let u = &self.units[i];
                Side {
                    unit: true,
                    packed: u.combat.packed,
                    moving: self.is_moving(i),
                    attacks: self.attack_of(o) != 0,
                    entrenched: u.combat.entrenched,
                    facing: u.movement.heading,
                    trench_facing: u.combat.trench_angle,
                    // The object's own `z` (`+0xc`), which `Unit::update_z`
                    // writes as `find_tcoord_z` at its tile, clamped at zero
                    // (§46.2, item 1131).
                    z: self.world.object_z(u.pos.tile()),
                    damage_frame: u.combat.damage_frame,
                    damage_o: u.combat.damage_o,
                    captain: u.combat.captain,
                    // `unit_masks & 1`, step 17's arm: anything kills a
                    // decoy (item 3385: run1011e's packet, `get_damage`
                    // 19688 on the General's decoy Light Horse `1/82`,
                    // `take_damage` 0x19a2 after the uber divide).
                    decoy: u.decoy,
                    rocky: false,
                    tile_owned_by_attacker: self
                        .world
                        .owner_at(u.pos)
                        .player()
                        .is_some_and(|p| p == self.owner_of(attacker)),
                    ..Side::default()
                }
            }
            Obj::Building(b) => {
                let bd = &self.buildings[b];
                Side {
                    unit: false,
                    building: true,
                    build_proper: true,
                    under_construction: !bd.active,
                    attacks: self.attack_of(o) != 0,
                    in_unfriendly_territory: self.in_unfriendly_territory(b),
                    z: self.world.object_z(bd.pos.tile()),
                    tile_owned_by_attacker: self
                        .world
                        .owner_at(bd.pos)
                        .player()
                        .is_some_and(|p| p == self.owner_of(attacker)),
                    ..Side::default()
                }
            }
        }
    }

    /// The side an object presents as an **attacker**.
    pub(super) fn attacker_side(&self, o: Obj) -> Side {
        match o {
            Obj::Unit(i) => Side {
                unit: true,
                captain: self.units[i].combat.captain,
                z: self.world.object_z(self.units[i].pos.tile()),
                ..Side::default()
            },
            // `ObjectData::get_captain@00472400` is the object's own `o`,
            // which `do_damage` step 2 writes as the target's `damage_o`
            // (item 1131: run404's `0/7` reads 2007 from the Radar's hit
            // on 828).
            Obj::Building(b) => Side {
                building: true,
                build_proper: true,
                captain: crate::core::flag_reads::object_get_captain(self.buildings[b].index),
                z: self.world.object_z(self.buildings[b].pos.tile()),
                ..Side::default()
            },
        }
    }

    // ------------------------------------------------------------------
    // Distance and validity
    // ------------------------------------------------------------------

    /// `ObjectData::attack_dist(o, who, x, y)` from `at` (§13.1).
    ///
    /// **The centre-to-centre arm is the attacker's `is_plane`** (vslot
    /// `+0xc0`, `006488f0`; `UnitData::is_plane@0046ce40`, `domain == 2 &&
    /// !(unit_flags & 0x20)`) **and not a missile** (type `+0x1e4 &
    /// 0x8000000`): a helicopter measures edge to edge, as a ground unit
    /// does. This crate read the domain alone, so an Attack Helicopter's
    /// look at a city measured from the city's centre and passed it over
    /// (`docs/AI.md` §227: run866's `1/145` on 5135).
    pub fn attack_dist_from(&self, attacker: Obj, at: Pos, target: Obj) -> i32 {
        let ap = self.profile(attacker);
        let tp = self.profile(target);
        let plane = matches!(attacker, Obj::Unit(u) if self.is_plane(u)) && !ap.has(mask::MISSILE);
        combat::attack_dist(
            at,
            self.pos_of(target),
            combat::extent(&ap, matches!(attacker, Obj::Building(_))),
            combat::extent(&tp, matches!(target, Obj::Building(_))),
            plane,
        )
    }

    pub fn attack_dist(&self, attacker: Obj, target: Obj) -> i32 {
        self.attack_dist_from(attacker, self.pos_of(attacker), target)
    }

    /// `ObjectData::is_in_range(o, who, x, y, …)` (§13.2). The unseen-tile
    /// test is absent: the simulation has no fog.
    pub fn is_in_range(&self, attacker: Obj, target: Obj) -> bool {
        self.is_in_range_at(attacker, self.pos_of(attacker), target)
    }

    /// The same question asked from a **given** point rather than the
    /// attacker's own — the eight-argument overload, which
    /// `Group::action_attack` uses to ask whether a member's current move
    /// order would carry it into range (`docs/GROUPS.md` §10).
    ///
    /// This overload passes `006486b0`'s sixth argument **zero**, which
    /// every call site in the executable but one does; the one is
    /// [`Self::is_in_range_at_margin`]'s. Item 470 measured the margin
    /// and did not land it, because on its own it moved `0/10` from one
    /// frame early to one frame late; item 472 landed it beside
    /// `Unit::check_target_path`, which is what actually ends `0/10`'s
    /// chase (`docs/COMBAT.md` §36).
    pub fn is_in_range_at(&self, attacker: Obj, at: Pos, target: Obj) -> bool {
        self.is_in_range_at_margin(attacker, at, target, false)
    }

    /// The same with `006486b0`'s **sixth argument** — the `0x90` the test
    /// adds to the measured distance before the max-range bound
    /// (`docs/COMBAT.md` §35.3). `do_move@005f7b30:216` is the only call
    /// site in the executable that passes it non-zero, and what it passes
    /// is `attack->mandatory == 0`: an ordered attack is dropped at the
    /// edge of reach, an opportunistic one three quarters of a tile
    /// inside it.
    pub fn is_in_range_at_margin(&self, attacker: Obj, at: Pos, target: Obj, margin: bool) -> bool {
        self.in_range_from(attacker, at, target, margin, true)
    }

    /// `is_in_range@00648d70` as `006486b0`'s head has it: **the slot,
    /// never the health** — `flags & 1`, and for a unit `is_on_map`
    /// ([`Self::struck`]). A city building at its ceiling (health 0) that
    /// still stands is in range: run809's `0/2000` on 3647, which
    /// `think`'s `near_o` arm hands the raiding `1/113` (`docs/AI.md`
    /// §205). SEAM: [`Self::is_in_range_at_margin`] still asks
    /// [`Self::active`] for every other caller — made `struck` there, the
    /// coverage pair parts at 3142 (`1/59`), so some reader leans on the
    /// health term (parked 2051).
    pub(crate) fn is_in_range_struck(&self, attacker: Obj, target: Obj) -> bool {
        self.in_range_from(attacker, self.pos_of(attacker), target, false, false)
    }

    /// `006486b0` whole, through [`in_range_steps`]: the slot test is
    /// [`Self::active`]'s when `health` (a building with no health left is
    /// out), [`Self::struck`]'s when not — split into the slot's own
    /// condition and, for a unit, `is_on_map`.
    pub(super) fn in_range_from(
        &self,
        attacker: Obj,
        at: Pos,
        target: Obj,
        margin: bool,
        health: bool,
    ) -> bool {
        let slot = match target {
            Obj::Unit(u) => self.units.get(u).is_some_and(|u| u.alive()),
            Obj::Building(b) => self
                .buildings
                .get(b)
                .is_some_and(|b| b.alive && b.combat.is_some() && (!health || b.health > 0)),
        };
        let ap = self.profile(attacker);
        in_range_steps(
            slot,
            || matches!(target, Obj::Unit(_)),
            || match target {
                Obj::Unit(u) => self.units[u].on_map,
                Obj::Building(_) => true,
            },
            at,
            // **The world-cell test** (`6486b0`..`64875b`, item 1200): the
            // tile under the point asked from — `div_3_table[x >> 6]`, the
            // tile, of the eight-argument overload's own `x/y` — answers no
            // when its surface is forest (`TData.mask & 0x30 == 0x30`). It
            // asks no domain, so a plane over a wood holds its fire:
            // chapter forty-one's Biplane on 839, at `0x7138`
            // (`docs/GOLDEN.md` §50).
            |at| {
                self.world.tile_mask(at.tile()) & crate::world::tile::SURFACE
                    == crate::world::tile::SURFACE_FOREST
            },
            |centre| self.attack_dist_from(attacker, centre, target),
            None,
            ap.max_range != 0,
            || self.reaches_like_a_hoplite(attacker),
            || ap.min_range,
            |of_target| {
                let o = if of_target { target } else { attacker };
                (matches!(o, Obj::Unit(_)), self.profile(o).big_radius)
            },
            || self.max_range_of(attacker),
            margin,
        )
    }

    /// `is_in_range@006486b0`'s melee arm asks the **attacker** `is(0x84,
    /// 0)` and, for the lineage it names, takes `0xf6` where everything
    /// else takes `0x66` — a reach of two thirds of a tile rather than a
    /// half (`docs/COMBAT.md` §13.2). The constant had been in the
    /// document since the second reading and in [`combat::in_range`]'s
    /// signature since it was written; nothing ever passed it, so every
    /// melee unit in this simulation fought at `0x66`.
    ///
    /// It is the whole of the golden record's engagement frame
    /// (`docs/COMBAT.md` §12.5): six range verdicts on frame 615, and the
    /// `0x66` reading gets all six wrong in the same direction.
    pub(super) fn reaches_like_a_hoplite(&self, attacker: Obj) -> bool {
        match attacker {
            Obj::Unit(u) => self.unit_line_is(u, HOPLITES),
            Obj::Building(_) => false,
        }
    }
}

/// **`ObjectData::is_in_range@006486b0`'s steps** (item 2314,
/// `docs/COMBAT.md` §13.2), in its order: the target's slot (`flags & 1`)
/// and, for a unit (vslot `+8`), `is_on_map` (vslot `+0xbc`); the world
/// cell under `at`, refused when forest; `attack_dist` at the quarter-tile
/// centre of `at` (`div_3_table[v >> 4] * 0x30 + 0x18`), written to the
/// out-word. A type with no range (`+0x1fc` zero) reaches `0x66`, or
/// `0xf6` for the HOPLITES lineage (`is(0x84, 0)`). Otherwise inside
/// `min_range × 0xc0 − 6` (vslot `+0x12c`) the attacker's and the target's
/// `big_radius` (`type +0x244`, each only for a unit, vslot `+0x18`) are
/// added before the inner bound is asked again, and the outer bound is
/// `max_range × 0xc0 + 6` (vslot `+0x130`) with `0x90` added for the
/// margin. The sums are the original's 32-bit ones, wrapping.
#[allow(clippy::too_many_arguments)]
pub(crate) fn in_range_steps(
    slot: bool,
    target_unit: impl FnOnce() -> bool,
    on_map: impl FnOnce() -> bool,
    at: Pos,
    forest: impl FnOnce(Pos) -> bool,
    dist: impl FnOnce(Pos) -> i32,
    out: Option<&mut i32>,
    ranged: bool,
    hoplite: impl FnOnce() -> bool,
    min_range: impl FnOnce() -> i32,
    mut big: impl FnMut(bool) -> (bool, i32),
    max_range: impl FnOnce() -> i32,
    margin: bool,
) -> bool {
    if !slot || (target_unit() && !on_map()) || forest(at) {
        return false;
    }
    let centre = Pos::new(
        at.x.div_euclid(48) * 48 + 0x18,
        at.y.div_euclid(48) * 48 + 0x18,
    );
    let d = dist(centre);
    if let Some(o) = out {
        *o = d;
    }
    if !ranged {
        return d <= if hoplite() { 0xf6 } else { 0x66 };
    }
    let inner = min_range().wrapping_mul(0xc0).wrapping_sub(6);
    if d < inner {
        let radius = |(unit, r): (bool, i32)| if unit { r } else { 0 };
        let mine = radius(big(false));
        let theirs = radius(big(true));
        if d.wrapping_add(theirs).wrapping_add(mine) < inner {
            return false;
        }
    }
    let reach = max_range().wrapping_mul(0xc0).wrapping_add(6);
    d.wrapping_add(if margin { 0x90 } else { 0 }) <= reach
}
