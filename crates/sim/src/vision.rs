//! What a unit reveals as it moves — `docs/VISION.md`.
//!
//! Three things: the line of sight a unit has (`Unit::update_los`), the disc
//! of fog cells that lights up around it (`Object::update_seen`), and the two
//! moments it runs — a step that crosses a half-cell, and the hundredth-frame
//! resync.
//!
//! **Why it is load-bearing rather than cosmetic.** `seen2` is monotone and
//! three things read it: `PathFinder::calc_cost`'s fog branch
//! (`docs/PATHFINDER.md` §5), `Unit::think_scout`'s cell filter
//! (`docs/SCOUT.md` §7) and the AI's site census (`docs/AI.md` §15.8). Until
//! this module existed the grid was the `WORLD` dump's frame-0 snapshot and
//! nothing wrote it, so a path planned at frame 100 was planned against a map
//! the unit had walked off the edge of.
//!
//! Nothing here draws on the sync stream.

use crate::Sim;
use crate::ai::ai_load::uflags2;
use crate::ai::ai_place::circle;
use crate::movement::{cos_component, sin_component};
use crate::territory::attrition::Domain;
use crate::world::{Pos, vector_dist};
use std::sync::OnceLock;

/// A fog cell in position units: half a world cell.
pub use crate::ids::scale::UNITS_PER_FOG;
/// A tile in position units — `TCoord`'s scale, and what `<LOS>` is measured
/// in.
use crate::ids::scale::UNITS_PER_TILE;
/// `Object::update_seen`'s cap on the radius, in fog cells.
const MAX_RADIUS: i32 = 0x40;
/// `ring_init`'s last ring, and the ring branch's own clamp.
const RING_LAST: i32 = 0x20;
/// The distance a small land unit's vision is thrown forward, from the
/// `mov $0x180, %edx` at `651d00`.
const PROJECT_DIST: i32 = UNITS_PER_FOG;

/// `ring_x`/`ring_y`/`ring_radius` — `circle`'s thickened twin, the offsets
/// an *incremental* reveal walks (`docs/VISION.md` §4).
pub struct Ring {
    pub x: Vec<i32>,
    pub y: Vec<i32>,
    /// `ring_radius[r]`: how many entries lie within ring `r`. Only
    /// `0..=0x20` are filled; `ring_init` stops there.
    pub radius: [usize; RING_LAST as usize + 1],
}

/// The four orthogonal offsets `ring_init` patches a ring's gaps with, read
/// out of the PE at `.rdata` `00add254` (x) and `00add214` (y): north, east,
/// south, west.
const ORTHOG: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

/// `ring_init@00681920`, rebuilt.
pub fn ring() -> &'static Ring {
    static RING: OnceLock<Ring> = OnceLock::new();
    RING.get_or_init(|| {
        let c = circle();
        let mut r = Ring {
            x: vec![0],
            y: vec![0],
            radius: [1; RING_LAST as usize + 1],
        };
        for k in 1..=RING_LAST as usize {
            // The ring is the circle's own, copied out whole.
            let (cs, ce) = (c.radius[k - 1], c.radius[k]);
            if cs < ce {
                r.x.extend_from_slice(&c.x[cs..ce]);
                r.y.extend_from_slice(&c.y[cs..ce]);
            }
            let start = r.radius[k - 1];
            r.radius[k] = r.x.len();
            // Then patched: a point with fewer than two orthogonal
            // neighbours in the ring so far gains the ones that lie strictly
            // inside it. The count walks the **live** end, so a point added
            // for one `i` counts for the next.
            let end = r.x.len();
            for i in start..end {
                let (x, y) = (r.x[i], r.y[i]);
                let mut cnt = (start..r.x.len())
                    .filter(|&j| (y - r.y[j]).abs() + (x - r.x[j]).abs() == 1)
                    .count();
                if x == 0 || y == 0 {
                    cnt += 1;
                }
                if cnt >= 2 {
                    continue;
                }
                for (ox, oy) in ORTHOG {
                    if vector_dist(x + ox, y + oy) < k as i32 {
                        r.x.push(x + ox);
                        r.y.push(y + oy);
                    }
                }
                r.radius[k] = r.x.len();
            }
        }
        r
    })
}

/// Which table an `update_seen` pass walks, and the slice of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sweep {
    /// The fog cell the disc is centred on.
    pub centre: (i32, i32),
    /// The radius in fog cells, after every clamp `update_seen` applies.
    pub radius: i32,
    /// Whether the offsets come from [`ring`] rather than from `circle`.
    pub ring: bool,
    pub start: usize,
    pub end: usize,
}

/// The ceiling `Unit::update_los` puts on a **packed** unit's `mylos`:
/// four tiles, applied as `if (mylos > 3) mylos = 4` and so a clamp
/// rather than an assignment (`0060e638`).
pub const PACKED_LOS: i32 = 4;

/// `p >> 7` through `div_3_table` — the fog cell a position lies in,
/// `p / 0x180` floored. The same read `PathFinder::calc_cost` makes
/// (`docs/PATHFINDER.md` §5).
pub const fn fog_of(p: i32) -> i32 {
    p.div_euclid(UNITS_PER_FOG)
}

impl Sim {
    // ------------------------------------------------------------------
    // §2 — the line of sight
    // ------------------------------------------------------------------

    /// `Unit::update_los@0060e4d0`'s value, as far as this simulation models
    /// it: the type's own `LOS`, the citizen terms, the science term, the
    /// two clamps and the troops term. It is the **derivation**; what the
    /// fog and the follower read is the cache [`Sim::update_los`] writes
    /// into [`crate::Unit::mylos`] at the original's call sites.
    /// `docs/VISION.md` §2 tabulates the terms that are read and not
    /// implemented (7–11 and term 6's archers' sub-arm); none fires in any
    /// capture on disk.
    ///
    /// Zero for a unit whose type has no `LOS`. The head's other exit —
    /// `leader_flags & 1`, an unused leader slot — has no counterpart here,
    /// where every player in [`Sim::players`] exists.
    pub fn unit_los(&self, u: usize) -> i32 {
        let unit = &self.units[u];
        let who = unit.owner as usize;
        let Some(rec) = unit.ty else { return 0 };
        let ty = &self.tables.unit_types[rec];
        let mut los = ty.los;
        if los == 0 {
            return 0;
        }
        // Term 3: the citizen upgrades and the nomad start. `TypeIndex`
        // `0x32`/`0x33` are the two citizens.
        // Each of the Militia line's bits the leader holds adds two
        // (`leader +0x6c20 & 4`, `& 8`), and PARTISAN's shares its two with
        // the nomad start (`0060e56a`..`0060e5b4`; run422's Citizens see 4
        // from the block after `tech who=0 militia on`, `docs/GOLDEN.md`
        // §48).
        if matches!(unit.type_index, 0x32 | 0x33) {
            let owner = unit.owner;
            if self.holds_unit_bit(owner, 0x42) {
                los += 2;
            }
            if self.holds_unit_bit(owner, 0x43) {
                los += 2;
            }
            if self.holds_unit_bit(owner, 0x44) || self.ai.lobby.starting_town == 0 {
                los += 2;
            }
        }
        // Term 4: the science line.
        let epoch = self
            .tech
            .get(who)
            .map_or(0, |t| t.epoch[crate::tech::Line::Science.index()]);
        los += epoch * ty.science_los;
        // Term 5: **a packed unit sees four tiles at most**
        // (`Unit::update_los@0060e4d0:0060e610`). The gate is the type's
        // `unit_flags2 & 4` *and* the state — `unit_masks & 0x80000`, or a
        // current order that is a pack cast (`UnitData::is_packing`) — and
        // a caster that fails it falls through to term 5b rather than
        // skipping both.
        //
        // Diff-backed on the AI Fisherman of run58, which carries
        // `mylos 4` from the Dock at 4461 through to the frame its `0x292`
        // casts and `mylos 6` — `LOS 4` plus one Science epoch's
        // `SCIENCE_LOS 2` — from 4989 on (`docs/ORDERS.md` §6.9).
        if ty.cols.flag2(uflags2::PACKS) && (unit.combat.packed || self.is_packing(u)) {
            los = los.min(PACKED_LOS);
        } else if matches!(unit.type_index, 0x3d | 0x3e | 0x190) {
            // Term 5b: the two merchants and the fur trapper see a fixed
            // radius that ignores everything above.
            los = epoch + 4;
        }
        // Term 6: **the troops upgrades** (`0060e64c`..`0060e70b`). A type
        // whose trainer — `UnitTypeData +0x40`, this crate's `where_` — is
        // exactly the Barracks, the Stable or the Auto Plant (`0x1ab`..
        // `0x1ad`) takes `TROOPS_UPGRADE_LOS` per `TROOPS_LOS_n` held,
        // added after term 5's clamp and 5b's replacement. Its archers'
        // sub-arm — `objmask 0x4000` or `0x400` and Obsidian — adds
        // `OBSIDIAN_ARCHERS_RANGE`, which ships as 0, and is not carried.
        // run529's who=1 takes Herbal Lore on 9782, and every Barracks and
        // Stable unit it owns sees two tiles more from block 9784
        // (`docs/VISION.md` §2).
        if matches!(
            self.trainer_ident(rec),
            Some(
                crate::cities::build::Ident::Barracks
                    | crate::cities::build::Ident::Stable
                    | crate::cities::build::Ident::AutoPlant
            )
        ) {
            los += self.troops_los_level(unit.owner) * self.tables.tuning.troops_upgrade_los;
        }
        // **The siege term** (`0060e7bc`..`0060e81f`, item 3380): a type
        // that `is_siege` (vslot `+0x10c`) of a leader with
        // `has_tribe_bonus(8)` sees `TURK_SIEGE_LOS` more — added after
        // term 5's packed clamp, so `GreatLakesTurkishTier4`'s packed
        // Trebuchets and Catapults print `mylos 7` in run1005, not 4. The
        // arm's Liberty (`0x219`) and Eiffel (`0x21c`) wonder terms are not
        // carried.
        if self.is_siege_unit(u) && self.cities.nation[who].turks {
            los += self.tables.tuning.turk_siege_los;
        }
        // A Spy sees `SPY_UPGRADE_LOS` more per upgrade held (`0060e87c`,
        // `docs/AI.md` §160).
        if self.is_spy_type(rec) {
            los += self.spy_upgrade_level(unit.owner) * self.tables.tuning.spy_upgrade_los;
        }
        // A hero sees `GENERAL_UPGRADE_LOS` more per General upgrade
        // (`0060e8a3`..`0060e8db`): the coverage pair's General `1/150`
        // prints 14 on its type's 8 (`docs/AI.md` §172).
        if ty.cols.flag2(uflags2::FLAG2_GENERAL) {
            los += self.general_upgrade_level(unit.owner) * self.tables.tuning.general_upgrade_los;
        }
        // The last word: a decoy (`unit_masks & 1`) sees one tile
        // (`0060e84a`; run422's decoys print `mylos 1`).
        if unit.decoy {
            return 1;
        }
        los
    }

    /// `Unit::update_los@0060e4d0` itself: [`Sim::unit_los`] written into
    /// the unit's [`crate::Unit::mylos`]. Called where the original calls
    /// vtable `+0x160` on a unit — `Unit::init`, `Unit::set_type`,
    /// `Leader::calc_unit_stats` and the unpack and decoy casts — and
    /// nowhere else, so a change to its inputs reaches the fog only at the
    /// next of those (`docs/VISION.md` §2).
    pub(crate) fn update_los(&mut self, u: usize) {
        // `Animal::update_los@005d7d50` overrides the slot: `mylos = 0`,
        // return 0 — for every gaia animal, whatever its type's sight
        // (`docs/CHECKSUM.md` §17, item 4109).
        self.units[u].mylos = if self.units[u].is_gaia() {
            0
        } else {
            self.unit_los(u)
        };
    }

    /// `LeaderData::get_troops_los_upgrade@006e1110`: how many of
    /// `TROOPS_LOS_1..3` the player holds. The listing's `BUY_SELL` arm —
    /// the Nubians' waiver of `has_preq` — compares a type the loop never
    /// reaches (`0x2ad` against `0x2e9..0x2eb`), so it is dead here.
    pub(crate) fn troops_los_level(&self, who: crate::Player) -> i32 {
        let p = &self.tech[who as usize];
        self.tables
            .tech_tree
            .roles
            .troops_los_preq
            .iter()
            .flatten()
            .filter(|&&t| self.tables.tech_tree.has_tech(&self.tables.setup, p, t))
            .count() as i32
    }

    /// `UnitData::is_packing@0060aa60` — the unit's **current** order is a
    /// cast whose spell `TypeData::is_pack` accepts
    /// ([`crate::orders::spell::is_pack`]). Not "is it packed": a unit
    /// halfway through the eighty frames of a `Pack` still has
    /// `unit_masks & 0x80000` clear, and this is what makes it see four
    /// tiles anyway.
    pub fn is_packing(&self, u: usize) -> bool {
        matches!(
            self.current_order(u).map(|o| o.body),
            Some(crate::orders::Body::Cast(c)) if crate::orders::spell::is_pack(c.spell)
        )
    }

    /// `UnitData::is_packing_or_unpacking@0060a410` — the current order is
    /// a cast whose spell is a pack (`TypeData` vslot `+0x50`) or an
    /// unpack (`+0x54`).
    pub fn is_packing_or_unpacking(&self, u: usize) -> bool {
        matches!(
            self.current_order(u).map(|o| o.body),
            Some(crate::orders::Body::Cast(c))
                if crate::orders::spell::is_pack(c.spell) || crate::orders::spell::is_unpack(c.spell)
        )
    }

    // ------------------------------------------------------------------
    // §3, §4 — the disc
    // ------------------------------------------------------------------

    /// **`UnitData::los@006100c0`**: the cache `update_los` wrote
    /// (`mylos`, `+0x3c`), and the CEO's term on top of it — a non-siege
    /// unit near its owner's `0x165` hero sees `THECEO_UNIT_LOS` (two
    /// tiles, one fog cell of radius) further ([`Sim::near_own_ceo`], the
    /// same block `UnitData::max_range` reaches with). The Russians' T34s
    /// beside their CEO light the East Indies' last fog row, (89, 119) and
    /// (90, 119), by frame 3920, where the bare cache left them dark
    /// (run1166's packet; `docs/AI.md` §304, item 3524).
    ///
    /// SEAM: the first term, Ptolemy's (`leader +0x59d2`, a type with
    /// `+0x2c8 & 0x420`, `has_general(0, 0x16a)`, `PTOLEMY_LOS_BONUS`), is
    /// not carried; no capture on disk fields him (scan: `grep -lac '^    type
    /// 362$' gamelog*.txt`, none of 748 dumps; the same pattern finds the CEO,
    /// type 357, 784 times in run1170).
    pub(crate) fn unit_data_los(&self, u: usize) -> i32 {
        let mut los = self.units[u].mylos;
        if self.near_own_ceo(u) {
            los += self.tables.tuning.theceo_unit_los;
        }
        los
    }

    /// The centre, radius and index range `Object::update_seen` would walk
    /// for this unit, or `None` when it reveals nothing.
    ///
    /// Split out from [`Sim::update_seen`] so the arithmetic can be tested
    /// without a fog grid.
    pub fn seen_sweep(&self, u: usize, ring_pass: bool) -> Option<Sweep> {
        let los = self.unit_data_los(u);
        if los == 0 {
            return None;
        }
        let mut r = (los * UNITS_PER_TILE) / UNITS_PER_FOG;
        if r > MAX_RADIUS {
            r = MAX_RADIUS;
        }
        let c = circle();
        let unit = &self.units[u];
        let pos = unit.pos;
        if r < 4 {
            // A small, ordinary, land unit sees from a half-cell in front of
            // its own nose. **A decoy does not** (`00651cdc`, the test's
            // fourth term, `unit_masks & 1`): it takes the own-cell arm, so
            // a step's ring pass at radius 0 lights the cell it stands in
            // where the projected arm's lights nothing. The battery's French
            // decoy Hussars `1/100` lights the Citizen `0/1`'s half-cell
            // (11, 73) on 14596 there, and holds it as a valid target on
            // 14604 (item 3347, `docs/VISION.md` §3).
            let projects = !unit.decoy
                && unit.ty.is_some_and(|rec| {
                    self.tables.unit_types[rec].kind.domain == Domain::Land
                        && !self.tables.unit_types[rec].cols.flag2(uflags2::PACKS)
                });
            let (centre, start) = if projects {
                // **The unit's own angle, not its body's.** The `project` at
                // `00651d05` is handed `UnitData +0x50` in `ecx`
                // (`movl 0x50(%ecx), %ecx` at `00651cf1`, `ecx` the
                // `units.list[who][o]` the two tests above it read) — the
                // heading `Unit::set_angle` writes toward the next waypoint,
                // which the dump prints as `UNITDATA angle`. It is
                // [`Movement::heading`], not the guy's eased
                // [`Movement::facing`]: while a unit turns, the two differ by
                // as much as thirty degrees, and the half-cell the disc is
                // thrown to differs with them (`docs/VISION.md` §3).
                let heading = unit.movement.heading;
                let p = Pos::new(
                    pos.x + sin_component(heading, PROJECT_DIST),
                    pos.y - cos_component(heading, PROJECT_DIST),
                );
                // `r - 5 < 1` for every `r < 4`, so this is always
                // `circle_radius[0]` — the whole disc bar its centre.
                let start = if ring_pass { c.radius[0] } else { 0 };
                ((fog_of(p.x), fog_of(p.y)), start)
            } else {
                // The original's `iVar5 = r - 1; if (iVar5 < 1) …` — a
                // radius of one takes the whole disc rather than a ring of
                // eight, because there is no ring below it.
                let start = if ring_pass && r > 1 {
                    c.radius[(r - 1) as usize]
                } else {
                    0
                };
                ((fog_of(pos.x), fog_of(pos.y)), start)
            };
            return Some(Sweep {
                centre,
                radius: r,
                ring: false,
                start,
                end: c.radius[r as usize],
            });
        }
        let centre = (fog_of(pos.x), fog_of(pos.y));
        if !ring_pass {
            return Some(Sweep {
                centre,
                radius: r,
                ring: false,
                start: 0,
                end: c.radius[r as usize],
            });
        }
        if r > RING_LAST {
            r = RING_LAST;
        }
        let g = ring();
        let inner = if r - 1 < 1 { 0 } else { r - 1 } as usize;
        Some(Sweep {
            centre,
            radius: r,
            ring: true,
            start: g.radius[inner],
            end: g.radius[r as usize],
        })
    }

    /// `Wall::update_los@0063eeb0` — the **building's** line of sight,
    /// which sits in the same vtable slot (`+0x160`) as the unit's and is a
    /// different function (`docs/VISION.md` §2.1).
    ///
    /// ```text
    /// if !is_started              { mylos = 0; return 0 }
    /// if !is_active && !vfunc0x2c { mylos = 0; return 0 }
    /// if !is_active              { mylos = 1; goto tail }
    /// mylos = type->LOS + epoch[Science] * type->SCIENCE_LOS
    /// <fort, tower, colosseum and furs terms>
    /// tail: mylos += type->x_size / 2
    /// ```
    ///
    /// The tail is what the dump's figures are read off: run39's AI opens
    /// with a Small City at `mylos 15` (`LOS 12`, `X_SIZE 7`), a
    /// Woodcutter's Camp at `7` (`6`, `2`) and four Farms at `8`
    /// (`6`, `4`) — and the city goes to **17** on the frame its Science
    /// epoch reaches 1, which is the science term with `SCIENCE_LOS 2`.
    ///
    /// **The tower term** (item 3543): a finished Tower-line building that
    /// is not a fort adds `TOWER_FORT_LOS[get_tower_fort_los − 1]`. The
    /// Turks' age-4 tower `1/2012` (TypeIndex 441, `LOS 14`, `X_SIZE 2`)
    /// prints `mylos 19` in run1186, its owner holding the `MINUTEMAN` bit
    /// and not `PARTISAN`'s: 14 + 4 + 1.
    /// **The fort term** (item 3631): a finished Fort-line building adds
    /// `FORT_UPGRADE_LOS[get_fort_los − 1]` in place of the tower's. The
    /// Romans' Fort `1/2013` (TypeIndex 444, `LOS 12`, `X_SIZE 5`) prints
    /// `mylos 16` in run1277: 12 + 2 + 2, its owner at fort level 2.
    /// SEAM: the colosseum and furs terms (both 0 in the shipped rules,
    /// parked 3631a) are read and not carried.
    pub fn build_los(&self, b: usize) -> i32 {
        let bd = &self.buildings[b];
        if !bd.started || !bd.alive {
            return 0;
        }
        let Some(rec) = bd.ty else { return 0 };
        let ty = &self.tables.build_types[rec];
        let mut los = if bd.active {
            let epoch = self
                .tech
                .get(bd.owner as usize)
                .map_or(0, |t| t.epoch[crate::tech::Line::Science.index()]);
            let mut los = ty.los + epoch * ty.science_los;
            if crate::cities::build::is_fort(&self.tables.build_types, rec) {
                los += self.tables.tuning.fort_upgrade_los[self.fort_los(bd.owner) - 1];
            } else if crate::cities::build::is_tower(&self.tables.build_types, rec) {
                los +=
                    self.tables.tuning.tower_fort_los[(self.tower_fort_los(bd.owner) - 1) as usize];
            }
            los
        } else if ty.wonder {
            // Vslot `+0x2c`, `BuildData::is_wonder`: the one unfinished
            // site `Wall::update_los` gives sight (item 3391's sweep).
            1
        } else {
            return 0;
        };
        los += ty.x_size / 2;
        los
    }

    /// The building's vslot `+0x160` call: [`Self::build_los`] into the
    /// cache, [`crate::Building::mylos`] (item 3965). The original makes it
    /// beside every `+0x15c` (`update_hits`) on a building — `Wall::init`,
    /// `Leader::calc_wall_stats`, `Build::activate`, `Cities::capture_city`
    /// and `Wall::swap_team` — and nowhere else, so a wonder site started
    /// after its `Wall::init` keeps `0` until its owner's next
    /// `calc_wall_stats`, and a Science epoch reaches a finished building
    /// on the leader pass after the gain, as it reaches a unit (§2).
    pub(crate) fn update_build_los(&mut self, b: usize) {
        self.buildings[b].mylos = self.build_los(b);
    }

    /// `LeaderData::get_fort_los@006daf30`: `1` plus the highest of
    /// `FORTLOS2..4` held ([`crate::tech::Roles::fort_los_preq`]).
    pub(crate) fn fort_los(&self, who: crate::Player) -> usize {
        1 + self.bonus_level(who, &self.tables.tech_tree.roles.fort_los_preq)
    }

    /// `LeaderData::get_tower_fort_los@006daf80`: 4 with the third of
    /// [`crate::tech::Roles::tower_fort_los_units`]' bits, 3 with the
    /// second, 2 with the first, else 1. The raw bits, not `has_tech`.
    pub(crate) fn tower_fort_los(&self, who: crate::Player) -> i32 {
        let held = |k: usize| {
            self.tables.tech_tree.roles.tower_fort_los_units[k].is_some_and(|t| {
                self.tech
                    .get(who as usize)
                    .is_some_and(|p| p.tech.get(t).copied().unwrap_or(false))
            })
        };
        if held(2) {
            4
        } else if held(1) {
            3
        } else {
            i32::from(held(0)) + 1
        }
    }

    /// The disc `Object::update_seen(0)` walks for a building: its own
    /// half-cell, radius `los / 2`, the whole circle. A building is not
    /// `is_unit()`, so §3's projected centre never applies to it.
    ///
    /// **The radius is the cache's** (item 3970, parked 3965a): the sweep
    /// reads the `mylos` byte `Wall::update_los` last left
    /// ([`crate::Building::mylos`]), never the live [`Sim::build_los`]. A
    /// Tower gaining the Militia bit keeps lighting its old disc until its
    /// owner's next `calc_wall_stats`: chapter 126's `1/2007` lights 11
    /// tiles, so who=1 first sees `0/6` on 764 and its Tower's 777 scan
    /// takes it, where the live 13 lit the chariot's half-cell by 745.
    pub fn build_sweep(&self, b: usize) -> Option<Sweep> {
        let los = self.buildings[b].mylos;
        if los == 0 {
            return None;
        }
        let mut r = (los * UNITS_PER_TILE) / UNITS_PER_FOG;
        if r > MAX_RADIUS {
            r = MAX_RADIUS;
        }
        let pos = self.buildings[b].pos;
        Some(Sweep {
            centre: (fog_of(pos.x), fog_of(pos.y)),
            radius: r,
            ring: false,
            start: 0,
            end: circle().radius[r as usize],
        })
    }

    /// `Object::update_seen@00651b80` for a building — the call
    /// `Build::activate@00623e20` makes last, at vtable `+0x174` with
    /// `ring = 0`.
    ///
    /// **This is what a building erected during a game reveals**, and until
    /// 2026-08-31 nothing here did it: the fog grew only where units walked,
    /// so run39's AI farm at cell `(54, 51)` — finished on frame 219 — left
    /// the three cells east of it dark, and nineteen frames later its
    /// scout's `EXPLORE_TO` path ran through them as cheap unexplored ground
    /// where the original, which could see them, went round
    /// (`docs/PATHFINDER.md` §5, `docs/SCOUT.md`).
    pub fn update_seen_build(&mut self, b: usize) -> usize {
        if !self.world.has_fog() || !self.buildings[b].alive {
            return 0;
        }
        let who = self.buildings[b].owner;
        if who >= 8 {
            return 0;
        }
        let Some(sw) = self.build_sweep(b) else {
            return 0;
        };
        // **The whole-disc call makes its own `+0x164` first**
        // (`Object::update_seen@00651b80`: `param_1 == 0` and `visible != 0`,
        // or a started visible wonder; twenty-fourth pass, group 19; A8 row
        // 10), so a finished wonder is relit `0xff` after every clear, as the
        // unit twin does.
        if self.buildings[b].visible != 0
            || (self.buildings[b].started && self.is_visible_wonder(b))
        {
            self.update_local_seen_build(b);
        }
        self.write_sweep(&sw, who, false)
    }

    /// `ObjectData::is_visible_wonder@00471130`: a wonder without the city
    /// flag and not a fort type (group 5).
    fn is_visible_wonder(&self, b: usize) -> bool {
        self.buildings[b].ty.is_some_and(|ty| {
            self.tables.build_types[ty].wonder
                && !crate::cities::build::is_city(&self.tables.build_types, ty)
                && !crate::cities::build::is_fort(&self.tables.build_types, ty)
        })
    }

    // ------------------------------------------------------------------
    // §5 — the write
    // ------------------------------------------------------------------

    /// `Object::update_seen@00651b80` for a unit: light its disc into the
    /// fog. `ring_pass` is the original's `param_1` — set by a walking
    /// unit, clear by one arriving on the map.
    ///
    /// Returns how many fog cells this player saw for the first time, which
    /// is the count of `reveal_fog` calls the original would make; nothing
    /// reads it but the tests.
    pub fn update_seen(&mut self, u: usize, ring_pass: bool) -> usize {
        if !self.world.has_fog() || !self.units[u].on_map || !self.units[u].alive() {
            return 0;
        }
        let who = self.units[u].owner;
        if who >= 8 {
            return 0;
        }
        let Some(sw) = self.seen_sweep(u, ring_pass) else {
            return 0;
        };
        // The whole-disc call relights what the unit has made visible to
        // others first (`00651b80`: `param_1 == 0` and `visible != 0` →
        // vtable `+0x164`, `Unit::update_local_seen`), so a rebuild of
        // `seen` keeps an attacker lit for its victims while its byte
        // stands. `docs/VISION.md` §10.
        if !ring_pass && self.units[u].visible != 0 {
            self.update_local_seen_unit(u);
        }
        // `Object::init@00647750` sets `ObjectData` flag `0x40` for a type
        // with the `DETECT` objmask (`Z`); `update_seen` then lights `seen3`
        // over the whole range it writes (the cutoff is the range's end
        // below radius 32, `docs/VISION.md` §4).
        let detect = self.units[u].ty.is_some_and(|t| {
            self.tables.unit_types[t].combat.obj_masks & crate::combat::mask::DETECT != 0
        });
        // **A detector's ring pass at radius 32 lights nothing** (`651d91`–`651dac`): the
        // `seen3` cutoff test clamps `r` to `0x1f` *in place* (`cmpl $0x20; cmovge`, stored back
        // to the local the range's end is read from), so the ring's last slice `[ring_radius[31],
        // ring_radius[32])` becomes `[ring_radius[31], ring_radius[31])`. Reached only by a
        // `DETECT` unit of `LOS` 64 tiles or more (item 3268's sweep; none in the data).
        if detect && sw.ring && sw.radius >= RING_LAST {
            return 0;
        }
        self.write_sweep(&sw, who, detect)
    }

    /// §5's loop, shared by the unit's disc and the building's. `detect`
    /// is `World::set_seen`'s `param_4` for the whole range.
    fn write_sweep(&mut self, sw: &Sweep, who: crate::Player, detect: bool) -> usize {
        let mask = 1u8 << who;
        let (cx, cy) = sw.centre;
        let (fw, fh) = (self.world.fog_xs(), self.world.fog_ys());
        // The hoisted test: the whole disc on the grid, so the per-point
        // bounds check can be skipped.
        let whole = cx - sw.radius >= 0
            && cy - sw.radius >= 0
            && cx + sw.radius < fw
            && cy + sw.radius < fh;
        let (xs, ys): (&[i32], &[i32]) = if sw.ring {
            let g = ring();
            (&g.x, &g.y)
        } else {
            let c = circle();
            (&c.x, &c.y)
        };
        let mut revealed = 0;
        for i in sw.start..sw.end {
            let (x, y) = (xs[i] + cx, ys[i] + cy);
            if !(whole || (x >= 0 && y >= 0 && x < fw && y < fh)) {
                continue;
            }
            if detect {
                self.world.set_seen3(x, y, mask);
            }
            if self.world.set_seen(x, y, mask) {
                revealed += 1;
                // `World::set_seen`'s **return value is the gate**: the
                // original calls `reveal_fog` on exactly the cells this
                // answered true for, which is how a good is offered to a
                // leader once and only once ([`Sim::reveal_fog`]).
                self.reveal_fog(x, y, who);
            }
        }
        revealed
    }

    // ------------------------------------------------------------------
    // §6 — the cadence
    // ------------------------------------------------------------------

    /// The half-cell test `Unit::set_new_location@005f8d20` puts in front of
    /// `update_seen`, and the call. `from` is where the unit was before the
    /// step; the unit has already been moved.
    /// Answers `None` when the step stayed inside one fog cell and the
    /// reveal was therefore skipped, and `Some(newly revealed)` when it ran
    /// — the distinction a fog-cell count cannot make, since a second
    /// sweep from the same centre reveals nothing either way.
    pub(crate) fn moved_to(&mut self, u: usize, from: Pos, ring_pass: bool) -> Option<usize> {
        let to = self.units[u].pos;
        if fog_of(from.x) == fog_of(to.x) && fog_of(from.y) == fog_of(to.y) {
            return None;
        }
        Some(self.update_seen(u, ring_pass))
    }

    /// `GameDaemon::update_all_seen@00732840`, on the planes this
    /// simulation keeps: `seen` cleared, then every unit's whole disc and
    /// every building's, `frame % 100 == 0x21`.
    ///
    /// **The clear is the pass's point for `seen`** (`docs/VISION.md` §10):
    /// between resyncs the current plane only grows, and this is the one
    /// place it forgets. `valid_target`'s fog test reads it
    /// ([`Sim::world_sees`]), so a unit whose target's cell nobody sees
    /// after the resync drops the target; chapter eleven's `1/6` does on
    /// 1133. `seen2` is monotone and the pass cannot remove a bit from it;
    /// what it adds there is the objects the incremental path misses — a
    /// unit that never crosses a half-cell, and every building.
    ///
    /// Each unit's whole disc relights its `visible` cells first
    /// ([`Sim::update_seen`]). The original's unit arm relights only what
    /// passes vslots `+0x8` and `+0xbc`. `seen3` and the cell twin `+0x168`
    /// are not kept.
    ///
    /// **The buildings' arm has two branches** (item 1446, `docs/VISION.md`
    /// §6.4): a live, finished building lights its disc (vslot `+0x174`);
    /// any other — `!(flags & 1) || !is_active()` — that is live, a wonder
    /// and started relights its grown footprint through `+0x164`
    /// ([`Sim::update_local_seen_build`], `0xff` into both planes), and the
    /// rest light nothing. So a wonder under construction is lit for every
    /// player after each clear, and an unfinished ordinary site — whose
    /// `Wall::update_los` is 0 in any case — contributes no disc.
    pub(crate) fn update_all_seen(&mut self) {
        if !self.world.has_fog() {
            return;
        }
        self.world.clear_seen();
        // **The buildings come before the units** (`GameDaemon::update_all_seen
        // @00732840`: the building loop closes before the unit loop;
        // twenty-fourth pass, group 18; A8 row 12). Every write is an OR, so
        // only `reveal_fog`'s order of discovery can show it.
        for b in 0..self.buildings.len() {
            let bd = &self.buildings[b];
            if bd.alive && bd.active {
                self.update_seen_build(b);
            } else if bd.alive
                && bd.started
                && bd.ty.is_some_and(|t| self.tables.build_types[t].wonder)
            {
                self.update_local_seen_build(b);
            }
        }
        for u in 0..self.units.len() {
            self.update_seen(u, false);
        }
    }

    // ------------------------------------------------------------------
    // §6.1 — the second reveal: a building the enemy has laid eyes on
    // ------------------------------------------------------------------

    /// `LeaderData +0x6929` — the byte `check_ever_seen` masks the line of
    /// sight with when the building is unstarted, and the byte its meet
    /// loop tests each other leader by: the leader's own bit plus every
    /// leader allied with it.
    pub(crate) fn seen_ally_mask(&self, who: crate::Player) -> u8 {
        let mut m = 1u8 << (who & 7);
        for i in 0..self.territory.players.len().min(8) {
            let o = i as crate::Player;
            if o != who && self.is_ally(who, o) {
                m |= 1 << i;
            }
        }
        m
    }

    /// `Wall::check_ever_seen@0063ce70` — who has ever *looked* at this
    /// building.
    ///
    /// The scan is the footprint, cell by cell, of the **current** line of
    /// sight (`World +0x15c`, [`World::seen`](crate::world::World::seen)),
    /// ored into `ever_seen`; while the building is unstarted the owner's
    /// own ally mask filters it, and once started every bit is taken and
    /// `ever_seen_completed` takes them too for a finished one. When a
    /// leader that is not the owner appears in `ever_seen` for the first
    /// time the building answers by lighting itself — the call at vtable
    /// `+0x164`, [`Sim::update_local_seen_build`].
    ///
    /// **This is what a scout walking past a city does to the map.** Until
    /// 2026-09-17 nothing here made that write, and Great Lakes' AI scout
    /// therefore walked *through* the human capital's footprint on block
    /// 8002 rather than round it: fourteen half-cells of the original's fog
    /// plane carry player 1's bit over player 0's ground and this crate's
    /// carried none of them (`docs/PATHFINDER.md` §20.2,
    /// `run95_s_block_8002_is_where_the_fog_parts_and_the_price_with_it`).
    ///
    /// `force` is the original's `param_1`, set by the two `Wall::process`
    /// call sites that have already put a bit in by hand.
    pub(crate) fn check_ever_seen(&mut self, b: usize, force: bool) {
        if !self.world.has_fog() || !self.buildings[b].alive {
            return;
        }
        let who = self.buildings[b].owner;
        let Some(ty) = self.buildings[b].ty else {
            return;
        };
        if who >= 8 {
            return;
        }
        let old = self.buildings[b].ever_seen;
        let started = self.buildings[b].started;
        let mine = self.seen_ally_mask(who);
        // `game->everyone_mask` — the bits of the leaders in this game, so
        // the scan stops once every one of them has seen it.
        let all = ((1u16 << self.territory.players.len().min(8)) - 1) as u8;
        let want = if started { all } else { mine };
        if self.buildings[b].ever_seen & want != want
            || self.buildings[b].ever_seen_completed & want != want
        {
            let corner = self.tile_corner(ty, self.buildings[b].pos);
            let (xs, ys) = (
                self.tables.build_types[ty].x_size,
                self.tables.build_types[ty].y_size,
            );
            let active = self.buildings[b].active;
            let (mut es, mut ec) = (
                self.buildings[b].ever_seen,
                self.buildings[b].ever_seen_completed,
            );
            for v in 0..ys {
                for u in 0..xs {
                    let s = self
                        .world
                        .seen((corner.x + u) >> 1, (corner.y + v) >> 1)
                        .unwrap_or(0);
                    if started {
                        es |= s;
                        if active {
                            ec |= s;
                        }
                    } else {
                        es |= s & mine;
                    }
                }
            }
            self.buildings[b].ever_seen = es;
            self.buildings[b].ever_seen_completed = ec;
        }
        let now = self.buildings[b].ever_seen;
        if old == now && !force {
            return;
        }
        // **The meet loop — `docs/VISION.md` §6.2.** For every other
        // leader whose ally mask has *newly* appeared in `ever_seen`,
        // `Leader::meet` fires unless the two have met already, and the
        // building then lights itself once for any of them.
        //
        // Two gates, both the original's: `leader_flags & 1`
        // (`LEADER_VALID`, a slot in use — not the `& 2` the census loops
        // carry) and `o != owner`. The `met` flag that drives the reveal
        // is set **before** the already-met test, so a second sighting
        // still relights the footprint.
        let mut met = false;
        let mut greet = Vec::new();
        for w in 0..self.territory.players.len().min(8) {
            let o = w as crate::Player;
            // `leaders[o].leader_flags & 1` is a slot in use, and a defeat
            // clears only bit 1 (`Leader::defeat@006ecb00:63`): a defeated
            // leader is still met (twenty-fourth pass, group 6; A8 row 15).
            if o == who {
                continue;
            }
            let m = self.seen_ally_mask(o);
            if now & m != 0 && old & m == 0 {
                met = true;
                if !self.has_met(who, w) {
                    greet.push(o);
                }
            }
        }
        for o in greet {
            self.meet(who, o);
        }
        if met {
            self.update_local_seen_build(b);
        }
    }

    /// **`Unit::update_local_seen@0060e410`** — the unit writes itself into
    /// the fog of everyone it has attacked (`docs/VISION.md` §7).
    ///
    /// The disc is `circle_radius[type->x_size]` points of the
    /// `circle_x`/`circle_y` spiral around the unit's own **half-cell**,
    /// and the mask is `ObjectData::visible` itself — so the write lands
    /// in the victim's plane and not the owner's, which is exactly the
    /// shape `docs/VISION.md` §7's Great Lakes elimination could not
    /// account for.
    ///
    /// Unlike the building's second reveal this is
    /// `World::set_seen2(…, param_4 = 0)`: **both** planes, the current
    /// line of sight as well as the permanent one
    /// ([`World::set_seen`](crate::world::World::set_seen)). A shooter in
    /// the dark is not merely remembered, it is *lit* — which is what lets
    /// the return fire find it on the next frame rather than only
    /// through the `visible` fallback.
    ///
    /// The gate is `visible != 0`, so a unit that has never attacked
    /// anyone writes nothing. Returns how many half-cells changed, which
    /// nothing but the tests reads.
    ///
    /// SEAM: the original's third and fourth writes — `World +0x168` and
    /// the cell's `WData +0x14` — have no reader here, the same two
    /// [`World::set_seen`](crate::world::World::set_seen) already skips.
    pub(crate) fn update_local_seen_unit(&mut self, u: usize) -> usize {
        if !self.world.has_fog() {
            return 0;
        }
        let mask = self.units[u].visible;
        if mask == 0 {
            return 0;
        }
        let r = self.units[u]
            .ty
            .map_or(0, |t| self.tables.unit_types[t].combat.circle_radius);
        let c = crate::ai::ai_place::circle();
        let n = c.radius[r.clamp(0, 0x40) as usize];
        let (fx, fy) = (
            self.units[u].pos.x.div_euclid(UNITS_PER_FOG),
            self.units[u].pos.y.div_euclid(UNITS_PER_FOG),
        );
        let mut lit = 0;
        for i in 0..n {
            if self.world.set_seen(fx + c.x[i], fy + c.y[i], mask) {
                lit += 1;
            }
        }
        lit
    }

    /// `Wall::update_local_seen@0063ed50` — the building writes itself into
    /// the fog of everyone who has seen it.
    ///
    /// The rectangle is the footprint **grown by one tile on every side**,
    /// each tile mapped to its half-cell, and the mask is
    /// `ever_seen | visible | (1 << owner)`. `docs/VISION.md` §6.1 has the
    /// two branches.
    ///
    /// For an ordinary building the write is `seen2` only
    /// ([`World::set_seen2_only`]): the current line of sight is
    /// deliberately untouched, which is what stops one building's reveal
    /// from convincing its neighbour it has been spotted.
    ///
    /// **A wonder writes both planes** (item 1446): the function's first
    /// test is `is_wonder() && !(flags & 0x20) && !type->is_fort()`, and
    /// on it `set_seen2`'s `param_4` is 0 — `seen` as well as `seen2` —
    /// whether or not the wonder is started; a started one's mask is
    /// `0xff`, every player at once. **The city flag and the fort test can
    /// hold for a wonder in the range** — the Forbidden City is `FROM Small
    /// City` — and then it is written as an ordinary building is
    /// (twenty-fourth pass, group 5). Because
    /// it reaches `seen`, a wonder lit this way is taken into its own and
    /// its neighbours' `ever_seen` at their owner's next
    /// [`Sim::check_ever_seen`] — which is how a wonder the enemy has never
    /// looked at becomes first contact (`docs/VISION.md` §6.4).
    /// Returns how many half-cells changed, which nothing but the tests
    /// reads.
    pub fn update_local_seen_build(&mut self, b: usize) -> usize {
        if !self.world.has_fog() || !self.buildings[b].alive {
            return 0;
        }
        let who = self.buildings[b].owner;
        let Some(ty) = self.buildings[b].ty else {
            return 0;
        };
        if who >= 8 {
            return 0;
        }
        // `ObjectData::visible` (`+0x40`) is the third term of the mask;
        // `Build::do_attack` writes it after a round (`docs/GOLDEN.md` §59).
        // `is_visible_wonder` (`ObjectData::is_visible_wonder@00471130`): a
        // wonder **without** the city flag and not a fort type. The Forbidden
        // City is in the wonder range and is `FROM Small City`, so `is(VILLAGE)`
        // holds for it and it takes the ordinary branch (twenty-fourth pass,
        // group 5; A8 row 4).
        let wonder = self.tables.build_types[ty].wonder
            && !crate::cities::build::is_city(&self.tables.build_types, ty)
            && !crate::cities::build::is_fort(&self.tables.build_types, ty);
        let mask = if wonder && self.buildings[b].started {
            0xff
        } else {
            self.buildings[b].ever_seen | self.buildings[b].visible | (1u8 << who)
        };
        let corner = self.tile_corner(ty, self.buildings[b].pos);
        let (xs, ys) = (
            self.tables.build_types[ty].x_size,
            self.tables.build_types[ty].y_size,
        );
        let mut lit = 0;
        for u in -1..=xs {
            for v in -1..=ys {
                let (fx, fy) = ((corner.x + u) >> 1, (corner.y + v) >> 1);
                let changed = if wonder {
                    self.world.set_seen(fx, fy, mask)
                } else {
                    self.world.set_seen2_only(fx, fy, mask)
                };
                if changed {
                    lit += 1;
                }
            }
        }
        lit
    }
}

#[cfg(test)]
mod tests;

// The files of `vision/` (item 4011, `docs/STRUCTURE.md` §3).
mod seen_ally;
pub mod wall_los_view;
