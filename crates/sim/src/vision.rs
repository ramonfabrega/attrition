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
use crate::ai_load::uflags2;
use crate::ai_place::circle;
use crate::attrition::Domain;
use crate::movement::{cos_component, sin_component};
use crate::world::{Pos, vector_dist};
use std::sync::OnceLock;

/// A tile in position units — `TCoord`'s scale, and what `<LOS>` is measured
/// in.
const UNITS_PER_TILE: i32 = 0xc0;
/// A fog cell in position units: half a world cell.
pub const UNITS_PER_FOG: i32 = 0x180;
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
        let ty = &self.unit_types[rec];
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
            if self.holds_unit_bit(owner, 0x44) || self.lobby.starting_town == 0 {
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
                crate::build::Ident::Barracks
                    | crate::build::Ident::Stable
                    | crate::build::Ident::AutoPlant
            )
        ) {
            los += self.troops_los_level(unit.owner) * self.tuning.troops_upgrade_los;
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
        self.units[u].mylos = self.unit_los(u);
    }

    /// `LeaderData::get_troops_los_upgrade@006e1110`: how many of
    /// `TROOPS_LOS_1..3` the player holds. The listing's `BUY_SELL` arm —
    /// the Nubians' waiver of `has_preq` — compares a type the loop never
    /// reaches (`0x2ad` against `0x2e9..0x2eb`), so it is dead here.
    pub(crate) fn troops_los_level(&self, who: crate::Player) -> i32 {
        let p = &self.tech[who as usize];
        self.tech_tree
            .roles
            .troops_los_preq
            .iter()
            .flatten()
            .filter(|&&t| self.tech_tree.has_tech(&self.setup, p, t))
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

    // ------------------------------------------------------------------
    // §3, §4 — the disc
    // ------------------------------------------------------------------

    /// The centre, radius and index range `Object::update_seen` would walk
    /// for this unit, or `None` when it reveals nothing.
    ///
    /// Split out from [`Sim::update_seen`] so the arithmetic can be tested
    /// without a fog grid.
    pub fn seen_sweep(&self, u: usize, ring_pass: bool) -> Option<Sweep> {
        // `UnitData::los@006100c0` reads the cache `update_los` wrote.
        let los = self.units[u].mylos;
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
            // its own nose.
            let projects = unit.ty.is_some_and(|rec| {
                self.unit_types[rec].kind.domain == Domain::Land
                    && !self.unit_types[rec].cols.flag2(uflags2::PACKS)
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
    /// SEAM: the fort/tower/colosseum and furs terms are read and not
    /// carried; none can fire in any capture on disk.
    pub fn build_los(&self, b: usize) -> i32 {
        let bd = &self.buildings[b];
        if !bd.started || !bd.alive {
            return 0;
        }
        let Some(rec) = bd.ty else { return 0 };
        let ty = &self.build_types[rec];
        let mut los = if bd.active {
            let epoch = self
                .tech
                .get(bd.owner as usize)
                .map_or(0, |t| t.epoch[crate::tech::Line::Science.index()]);
            ty.los + epoch * ty.science_los
        } else {
            1
        };
        los += ty.x_size / 2;
        los
    }

    /// The disc `Object::update_seen(0)` walks for a building: its own
    /// half-cell, radius `los / 2`, the whole circle. A building is not
    /// `is_unit()`, so §3's projected centre never applies to it.
    pub fn build_sweep(&self, b: usize) -> Option<Sweep> {
        let los = self.build_los(b);
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
        self.write_sweep(&sw, who)
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
        self.write_sweep(&sw, who)
    }

    /// §5's loop, shared by the unit's disc and the building's.
    fn write_sweep(&mut self, sw: &Sweep, who: crate::Player) -> usize {
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
            if (whole || (x >= 0 && y >= 0 && x < fw && y < fh)) && self.world.set_seen(x, y, mask)
            {
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
        for u in 0..self.units.len() {
            self.update_seen(u, false);
        }
        for b in 0..self.buildings.len() {
            let bd = &self.buildings[b];
            if bd.alive && bd.active {
                self.update_seen_build(b);
            } else if bd.alive && bd.started && bd.ty.is_some_and(|t| self.build_types[t].wonder) {
                self.update_local_seen_build(b);
            }
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
        for i in 0..self.players.len().min(8) {
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
        let all = ((1u16 << self.players.len().min(8)) - 1) as u8;
        let want = if started { all } else { mine };
        if self.buildings[b].ever_seen & want != want
            || self.buildings[b].ever_seen_completed & want != want
        {
            let corner = self.tile_corner(ty, self.buildings[b].pos);
            let (xs, ys) = (self.build_types[ty].x_size, self.build_types[ty].y_size);
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
        for w in 0..self.players.len().min(8) {
            let o = w as crate::Player;
            if o == who || self.defeated.get(w).copied().unwrap_or(false) {
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
            .map_or(0, |t| self.unit_types[t].combat.circle_radius);
        let c = crate::ai_place::circle();
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
    /// `0xff`, every player at once. The city flag and the fort test cannot
    /// hold for a wonder, so `BuildData::is_wonder` alone decides. Because
    /// it reaches `seen`, a wonder lit this way is taken into its own and
    /// its neighbours' `ever_seen` at their owner's next
    /// [`Sim::check_ever_seen`] — which is how a wonder the enemy has never
    /// looked at becomes first contact (`docs/VISION.md` §6.4).
    /// Returns how many half-cells changed, which nothing but the tests
    /// reads.
    pub(crate) fn update_local_seen_build(&mut self, b: usize) -> usize {
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
        let wonder = self.build_types[ty].wonder;
        let mask = if wonder && self.buildings[b].started {
            0xff
        } else {
            self.buildings[b].ever_seen | self.buildings[b].visible | (1u8 << who)
        };
        let corner = self.tile_corner(ty, self.buildings[b].pos);
        let (xs, ys) = (self.build_types[ty].x_size, self.build_types[ty].y_size);
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
mod tests {
    use super::*;
    use crate::Tuning;
    use crate::ai_place::circle;
    use crate::movement::Angle;
    use crate::world::{Cell, Terrain, World};

    /// A 40 × 40 land world under an all-dark fog grid, one player, one
    /// unit of a type whose `LOS` the caller picks.
    fn fog_sim(los: i32, science_los: i32) -> (crate::Sim, usize) {
        let mut world = World::new(40, 40);
        world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
        assert!(world.set_fog(vec![0; 80 * 80]));
        let mut s = crate::Sim::new(Tuning::RON, world, 2);
        let t = s.add_unit_type(crate::UnitType {
            hits: 20,
            moves: 40,
            los,
            science_los,
            ..crate::UnitType::default()
        });
        let at = Pos::new(20 * 0x300 + 0x180, 20 * 0x300 + 0x180);
        let mut u = crate::Unit::new(0, 0, at, 20);
        u.ty = Some(t);
        let u = s.add_unit(u);
        (s, u)
    }

    /// How many fog cells this player has ever seen.
    fn seen(s: &crate::Sim, who: u8) -> usize {
        let (fw, fh) = (s.world.fog_xs(), s.world.fog_ys());
        (0..fh)
            .flat_map(|y| (0..fw).map(move |x| (x, y)))
            .filter(|&(x, y)| s.world.seen2(x, y).is_some_and(|v| v & (1 << who) != 0))
            .count()
    }

    /// The same world with one **building** of the caller's `LOS`,
    /// `SCIENCE_LOS` and footprint, finished and on a cell centre.
    fn fog_build(los: i32, science_los: i32, x_size: i32) -> (crate::Sim, usize) {
        let mut world = World::new(40, 40);
        world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
        assert!(world.set_fog(vec![0; 80 * 80]));
        let mut s = crate::Sim::new(Tuning::RON, world, 2);
        let t = s.add_build_type(crate::build::BuildType {
            los,
            science_los,
            x_size,
            y_size: x_size,
            ..crate::build::BuildType::default()
        });
        let at = Pos::new(20 * 0x300 + 0x180, 20 * 0x300 + 0x180);
        let b = s.add_building(0, at, 0);
        s.buildings[b].ty = Some(t);
        (s, b)
    }

    /// §2.1: a building's `mylos` is its type's `LOS`, plus the science
    /// term, plus **half its footprint** — the tail
    /// `Wall::update_los@0063eeb0` adds last and the unit's own
    /// `update_los` has no counterpart to.
    ///
    /// The three run39 opens with are the oracle: a Small City
    /// (`LOS 12`, `X_SIZE 7`) at `mylos 15`, a Woodcutter's Camp
    /// (`6`, `2`) at `7`, a Farm (`6`, `4`) at `8` — and the city at
    /// **17** once its Science epoch is 1, `SCIENCE_LOS` being 2.
    #[test]
    fn a_buildings_los_adds_half_its_footprint_and_the_science_term() {
        for (los, x, want) in [(12, 7, 15), (6, 2, 7), (6, 4, 8), (8, 4, 10)] {
            let (s, b) = fog_build(los, 0, x);
            assert_eq!(s.build_los(b), want, "LOS {los}, X_SIZE {x}");
        }
        let (mut s, b) = fog_build(12, 2, 7);
        assert_eq!(s.build_los(b), 15);
        s.tech[0].epoch[crate::tech::Line::Science.index()] = 1;
        assert_eq!(s.build_los(b), 17, "the science term");
        assert_eq!(s.build_sweep(b).map(|w| w.radius), Some(8));
    }

    /// §6.2: **first contact is a building of one leader entering the
    /// other's current line of sight**, and nothing else here reaches it.
    ///
    /// The met bit is `treaties[other] & 1`, [`Sim::treaty_on`] writes it
    /// on both sides, and `Wall::check_ever_seen@0063ce70`'s tail is the
    /// only live caller of `Leader::meet`. So: a player-0 building
    /// standing in the dark, a player-1 unit that has not looked at it,
    /// and the bit stays clear however often the building checks — then
    /// the unit's disc lands on the footprint and the **next** check sets
    /// it, on both leaders at once.
    ///
    /// The order matters and is asserted: the bit follows `ever_seen`
    /// rather than the fog. A leader whose unit is standing on the
    /// building's own ground has still not met its owner until the
    /// building's own eighth-frame check runs.
    #[test]
    fn a_building_seen_by_the_enemy_is_first_contact() {
        let (mut s, b) = fog_build(6, 0, 4);
        let t = s.add_unit_type(crate::UnitType {
            hits: 20,
            moves: 40,
            los: 16,
            ..crate::UnitType::default()
        });
        // Nothing has looked at anything yet — asserted **before** the
        // scout exists, because a unit born on the map lights its own disc
        // (`Object::add_to_world`, item 447).
        s.check_ever_seen(b, false);
        assert_eq!(
            s.buildings[b].ever_seen, 0,
            "nobody's line of sight is on the footprint yet — not even the \
             owner's, since the building has not lit itself"
        );
        assert!(
            !s.has_met(0, 1) && !s.has_met(1, 0),
            "nobody has met anybody"
        );
        assert_eq!(s.treaties[0][1], 0);

        // Player 1's scout, three cells east of player 0's building. Its
        // disc lands as it is born, and the fog now carries player 1's bit
        // over the footprint — which alone is **not** contact.
        let at = Pos::new(23 * 0x300 + 0x180, 20 * 0x300 + 0x180);
        let mut u = crate::Unit::new(1, 0, at, 20);
        u.ty = Some(t);
        let u = s.add_unit(u);
        assert_eq!(
            s.update_seen(u, false),
            0,
            "the scout's disc landed at its birth, so nothing is new"
        );
        assert!(
            !s.has_met(1, 0),
            "the fog is lit but the building has not checked"
        );

        // The building checks, and the two leaders have met — both ways,
        // which is `treaty_on`'s own double write.
        s.check_ever_seen(b, false);
        assert_eq!(
            s.buildings[b].ever_seen, 0b10,
            "player 1 has looked at it, and only player 1"
        );
        assert!(s.has_met(0, 1) && s.has_met(1, 0), "first contact");
        assert_eq!((s.treaties[0][1], s.treaties[1][0]), (1, 1));

        // And it is never cleared: `Leader::treaty_off` has no caller in a
        // game. A second check changes nothing.
        s.check_ever_seen(b, false);
        assert_eq!((s.treaties[0][1], s.treaties[1][0]), (1, 1));
    }

    /// A 4 × 4 site of player 1's, placed and not started, in a two-player
    /// fogged world; `wonder` picks `BuildData::is_wonder`.
    fn site(wonder: bool) -> (crate::Sim, usize) {
        let (mut s, b) = fog_build(6, 0, 4);
        let t = s.buildings[b].ty.unwrap();
        s.build_types[t].wonder = wonder;
        let bd = &mut s.buildings[b];
        bd.owner = 1;
        (bd.started, bd.active) = (false, false);
        (s, b)
    }

    /// How many fog cells carry every bit in the **current** plane.
    fn lit_for_all(s: &crate::Sim) -> usize {
        let (fw, fh) = (s.world.fog_xs(), s.world.fog_ys());
        (0..fh)
            .flat_map(|y| (0..fw).map(move |x| (x, y)))
            .filter(|&(x, y)| s.world.seen(x, y) == Some(0xff))
            .count()
    }

    /// **§6.4: a wonder's start is first contact** (item 1446).
    /// `Build::start@006273a0` ends with vslot `+0x164` for a wonder, whose
    /// write is `0xff` into `seen` as well as `seen2`; `Wall::start`'s own
    /// `check_ever_seen` has already run, so the bits are taken at the
    /// owner's next check — and with them player 0, never having looked,
    /// has met player 1. run613 dates it in the original: the wonder starts
    /// on 7941 and both met bits are set on block 7946.
    ///
    /// Made to fail on purpose (item 1446): with the `+0x164` call removed
    /// from `start_building`, or with the wonder's write sent to `seen2`
    /// only, the current plane is dark over the footprint (0 of 16).
    #[test]
    fn a_wonder_s_start_is_first_contact() {
        let (mut s, b) = site(true);
        s.start_building(b);
        // The grown rectangle, 6 × 6 tiles from an even corner, is 4 × 4
        // half-cells: tiles c − 1 ..= c + 4 map to c/2 − 1 ..= c/2 + 2.
        assert_eq!(lit_for_all(&s), 16, "the grown footprint, every player");
        assert_eq!(
            s.buildings[b].ever_seen, 0,
            "Wall::start's own check ran before the reveal"
        );
        assert!(!s.has_met(0, 1));
        s.check_ever_seen(b, false);
        assert_eq!(s.buildings[b].ever_seen, 0xff, "every bit, as run613 reads");
        assert!(s.has_met(0, 1) && s.has_met(1, 0), "first contact");

        // An ordinary site starts dark for everyone else.
        let (mut s, b) = site(false);
        s.start_building(b);
        assert_eq!(lit_for_all(&s), 0);
        s.check_ever_seen(b, false);
        assert!(!s.has_met(0, 1), "no contact from an ordinary start");
    }

    /// **§6.4: the resync relights a wonder under construction** and lights
    /// no disc for an unfinished ordinary site (`GameDaemon::update_all_seen
    /// @00732840`'s not-active arm; `Wall::update_los` is 0 for such a site
    /// in any case). Made to fail on purpose (item 1446): the old
    /// every-building disc fails the wonder's relight (0 of 16), and a disc
    /// for an unfinished ordinary site fails the second half.
    #[test]
    fn the_resync_relights_a_started_wonder_and_no_unfinished_site() {
        let (mut s, b) = site(true);
        s.buildings[b].started = true;
        s.update_all_seen();
        assert_eq!(
            lit_for_all(&s),
            16,
            "the wonder is lit again after the clear"
        );

        let (mut s, b) = site(false);
        s.buildings[b].started = true;
        s.update_all_seen();
        let (fw, fh) = (s.world.fog_xs(), s.world.fog_ys());
        let any = (0..fh)
            .flat_map(|y| (0..fw).map(move |x| (x, y)))
            .any(|(x, y)| s.world.seen(x, y).unwrap_or(0) != 0);
        assert!(!any, "an unfinished ordinary site lights nothing");
    }

    /// §6.2's other half: `meet` fires on the **new** bit only, so a
    /// leader already met does not meet again — and a building nobody but
    /// its owner has seen never fires it at all.
    ///
    /// Made to fail on purpose by dropping the `old & m == 0` test, which
    /// makes every later check a fresh contact; the assertion that catches
    /// it is the census's, not the bit's, so what is checked here is the
    /// thing a bit cannot show: `ever_seen`'s own monotonicity, and that
    /// an unstarted building filters the mask down to its owner's allies
    /// and therefore cannot be the thing that introduces two enemies.
    #[test]
    fn an_unstarted_building_cannot_introduce_two_enemies() {
        let (mut s, b) = fog_build(6, 0, 4);
        s.buildings[b].started = false;
        s.buildings[b].active = false;
        let t = s.add_unit_type(crate::UnitType {
            hits: 20,
            moves: 40,
            los: 16,
            ..crate::UnitType::default()
        });
        let at = Pos::new(23 * 0x300 + 0x180, 20 * 0x300 + 0x180);
        let mut u = crate::Unit::new(1, 0, at, 20);
        u.ty = Some(t);
        // The disc lands at birth (item 447), so the look has happened by
        // the time the building checks.
        let u = s.add_unit(u);
        assert!(seen(&s, 1) > 0, "the scout's disc is on the grid");
        assert_eq!(s.update_seen(u, false), 0, "and nothing is new");
        s.check_ever_seen(b, false);
        assert_eq!(
            s.buildings[b].ever_seen, 0,
            "an unstarted building takes the owner's ally mask only, and the \
             enemy's bit is not in it"
        );
        assert!(!s.has_met(1, 0), "a foundation is not a sighting");
        // Started, the same look lands.
        s.buildings[b].started = true;
        s.check_ever_seen(b, false);
        assert!(s.has_met(1, 0), "a started building is");
    }

    /// §2.1's head: an unstarted building sees nothing, and a started but
    /// unfinished one sees `1 + x_size / 2` — never its type's `LOS`.
    #[test]
    fn an_unfinished_building_sees_one_plus_half_its_footprint() {
        let (mut s, b) = fog_build(12, 0, 7);
        s.buildings[b].active = false;
        assert_eq!(s.build_los(b), 4);
        s.buildings[b].started = false;
        assert_eq!(s.build_los(b), 0);
        assert!(s.build_sweep(b).is_none());
    }

    /// §6: the disc a finished building throws is the **whole** circle at
    /// its own half-cell — never §3's projected centre, which is an
    /// `is_unit()` case — and `update_all_seen` throws it again.
    #[test]
    fn a_finished_building_lights_its_whole_disc_from_its_own_half_cell() {
        let (mut s, b) = fog_build(6, 0, 4);
        let own = (fog_of(s.buildings[b].pos.x), fog_of(s.buildings[b].pos.y));
        let sw = s.build_sweep(b).expect("a finished building sees");
        assert_eq!(
            (sw.centre, sw.radius, sw.ring, sw.start),
            (own, 4, false, 0)
        );
        assert_eq!(sw.end, circle().radius[4]);
        assert_eq!(s.update_seen_build(b), sw.end, "every point is new");
        assert_eq!(seen(&s, 0), sw.end);
        // The half-cell four east is inside the disc and the one six south
        // of that is not — the octagonal radius, not a square.
        assert!(s.world.seen2(own.0 + 4, own.1).is_some_and(|v| v & 1 != 0));
        assert!(
            s.world
                .seen2(own.0 + 4, own.1 + 6)
                .is_some_and(|v| v & 1 == 0)
        );
        // Idempotent, and the hundred-frame pass runs it for buildings too.
        s.update_all_seen();
        assert_eq!(seen(&s, 0), sw.end);
    }

    /// §3: `LOS` is in tiles and the fog radius is half of it, truncating.
    /// The five the Ancient age actually fields, and the cap.
    #[test]
    fn the_radius_is_half_the_los_in_tiles() {
        for (los, want) in [
            (0, None),
            (2, Some(1)),
            (4, Some(2)),
            (6, Some(3)),
            (8, Some(4)),
            (11, Some(5)),
            (200, Some(0x40)),
        ] {
            let (s, u) = fog_sim(los, 0);
            assert_eq!(
                s.seen_sweep(u, false).map(|w| w.radius),
                want,
                "LOS {los} tiles"
            );
        }
    }

    /// §2 term 4: the science line multiplies `SCIENCE_LOS`, and it is
    /// `epoch[3]` — a Scout's `4 + 1 × 2 = 6`, which run10's frame 202
    /// confirms against the original's own `mylos`.
    #[test]
    fn a_science_level_adds_science_los_and_the_radius_follows() {
        let (mut s, u) = fog_sim(4, 2);
        assert_eq!(s.unit_los(u), 4);
        assert_eq!(s.seen_sweep(u, false).map(|w| w.radius), Some(2));
        s.tech[0].epoch[crate::tech::Line::Science.index()] = 1;
        assert_eq!(s.unit_los(u), 6);
        // §2: `mylos` is a cache. The sweep reads what `update_los` last
        // wrote, so the level reaches the fog only at the next refresh.
        assert_eq!(s.seen_sweep(u, false).map(|w| w.radius), Some(2));
        s.update_los(u);
        assert_eq!(s.seen_sweep(u, false).map(|w| w.radius), Some(3));
    }

    /// Term 6's fixture: a Barracks and a Market, a unit trained at each
    /// (`LOS 6`), and Herbal Lore as `TROOPS_LOS_1`. Player 0 is the AI.
    fn troops_sim() -> (crate::Sim, usize, usize, crate::tech::TypeId) {
        use crate::build::{BuildType, Ident};
        use crate::tech::{TechTree, TypeDef};
        let mut world = World::new(40, 40);
        world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
        assert!(world.set_fog(vec![0; 80 * 80]));
        let mut s = crate::Sim::new(Tuning::RON, world, 2);
        let barracks = s.add_build_type(BuildType {
            ident: Ident::Barracks,
            ..BuildType::default()
        });
        let market = s.add_build_type(BuildType {
            ident: Ident::Market,
            ..BuildType::default()
        });
        let mut tree = TechTree::new().with_tuning(&Tuning::RON);
        let barracks_id = tree.add(TypeDef::building("Barracks"));
        let market_id = tree.add(TypeDef::building("Market"));
        let herbal = tree.add(TypeDef::plain("Herbal Lore", 0));
        let explorer_id =
            tree.add(TypeDef::unit("Explorer", crate::tech::UnitTraits::default()).at(barracks_id));
        let merchant_id =
            tree.add(TypeDef::unit("Merchant", crate::tech::UnitTraits::default()).at(market_id));
        tree.roles.troops_los_preq = [Some(herbal), None, None];
        s.set_tech_tree(tree);
        s.build_types[barracks].tree = Some(barracks_id);
        s.build_types[market].tree = Some(market_id);
        let mut add = |tree_id| {
            let t = s.add_unit_type(crate::UnitType {
                hits: 20,
                moves: 40,
                los: 6,
                tree: Some(tree_id),
                ..crate::UnitType::default()
            });
            let at = Pos::new(20 * 0x300 + 0x180, 20 * 0x300 + 0x180);
            let mut u = crate::Unit::new(0, 0, at, 20);
            u.ty = Some(t);
            s.add_unit(u)
        };
        let (explorer, merchant) = (add(explorer_id), add(merchant_id));
        (s, explorer, merchant, herbal)
    }

    /// §2 term 6: `TROOPS_UPGRADE_LOS` per `TROOPS_LOS_n` held, for a unit
    /// whose trainer is the Barracks, the Stable or the Auto Plant — and
    /// for nothing else. run529's who=1 Explorer is 12 and then 14, and
    /// its Market-trained Merchant stays where it was.
    #[test]
    fn a_troops_los_level_adds_two_tiles_to_a_barracks_unit_only() {
        let (mut s, explorer, merchant, herbal) = troops_sim();
        assert_eq!((s.unit_los(explorer), s.unit_los(merchant)), (6, 6));
        s.tech[0].tech[herbal] = true;
        assert_eq!(s.troops_los_level(0), 1);
        assert_eq!(
            (s.unit_los(explorer), s.unit_los(merchant)),
            (6 + s.tuning.troops_upgrade_los, 6)
        );
        assert_eq!(s.tuning.troops_upgrade_los, 2);
    }

    /// §2: the tech reaches a unit's `mylos` through `gain_tech`'s tail —
    /// `leader_flags |= 0xc000000` — and the next `Leader::process`'s
    /// `calc_unit_stats`, not on the frame it is gained. run529: Herbal
    /// Lore on 9782, every Barracks and Stable unit two tiles more from
    /// block 9784.
    #[test]
    fn a_gained_tech_reaches_the_cache_at_the_next_leader_pass() {
        let (mut s, explorer, _, herbal) = troops_sim();
        assert_eq!(s.units[explorer].mylos, 6);
        s.gain_tech(0, herbal);
        assert_eq!(s.unit_los(explorer), 8, "the derivation sees it at once");
        assert_eq!(s.units[explorer].mylos, 6, "the cache does not");
        assert!(s.unit_stats_dirty[0], "the tail raised 0x4000000");
        s.tick();
        assert_eq!(s.units[explorer].mylos, 8, "the next leader pass wrote it");
        assert!(!s.unit_stats_dirty[0]);
    }

    /// §3: a small land unit sees from a half-cell in front of its nose,
    /// not from where it stands. Heading east and heading west put the
    /// centre on opposite sides of the unit's own fog cell.
    #[test]
    fn a_small_land_unit_sees_from_a_half_cell_ahead_of_its_heading() {
        let (mut s, u) = fog_sim(4, 0);
        let own = (fog_of(s.units[u].pos.x), fog_of(s.units[u].pos.y));
        // `Angle` runs clockwise from north through the full `u32`; a
        // quarter turn is east, three quarters west (`docs/MOVEMENT.md`).
        s.units[u].movement.set_facing(Angle(0x4000_0000));
        let east = s.seen_sweep(u, false).unwrap().centre;
        s.units[u].movement.set_facing(Angle(-0x4000_0000));
        let west = s.seen_sweep(u, false).unwrap().centre;
        assert_eq!(east, (own.0 + 1, own.1), "heading east, one fog cell east");
        assert_eq!(west, (own.0 - 1, own.1), "heading west, one fog cell west");
    }

    /// §3, and the distinction item 79 turned on: the angle the disc is
    /// thrown along is **`UnitData::angle`** — [`Movement::heading`] — not
    /// the guy's eased [`Movement::facing`]. `set_facing` writes all three
    /// together, so a test that only ever uses it cannot tell them apart;
    /// this one sets them a half-turn apart, which is what a unit
    /// mid-turn looks like.
    #[test]
    fn the_projection_follows_the_unit_s_heading_not_its_body_s_facing() {
        let (mut s, u) = fog_sim(4, 0);
        let own = (fog_of(s.units[u].pos.x), fog_of(s.units[u].pos.y));
        s.units[u].movement.facing = Angle(-0x4000_0000);
        s.units[u].movement.heading = Angle(0x4000_0000);
        assert_eq!(
            s.seen_sweep(u, false).unwrap().centre,
            (own.0 + 1, own.1),
            "heading east while the body still points west"
        );
    }

    /// §3: and a unit with radius four or more does not project — its
    /// centre is its own fog cell whichever way it faces.
    #[test]
    fn a_unit_with_radius_four_or_more_sees_from_where_it_stands() {
        let (mut s, u) = fog_sim(8, 0);
        let own = (fog_of(s.units[u].pos.x), fog_of(s.units[u].pos.y));
        s.units[u].movement.set_facing(Angle(0x4000_0000));
        assert_eq!(s.seen_sweep(u, false).unwrap().centre, own);
        s.units[u].movement.set_facing(Angle(-0x4000_0000));
        assert_eq!(s.seen_sweep(u, false).unwrap().centre, own);
    }

    /// §4: the moving pass walks the **ring** table only from radius four
    /// up. Below it — every unit an Ancient game starts with — it is the
    /// whole disc bar its centre, which is why a wrong ring table would not
    /// be caught by any capture on disk.
    #[test]
    fn the_ring_table_is_reached_only_from_radius_four() {
        for (los, want_ring) in [(2, false), (4, false), (6, false), (8, true), (11, true)] {
            let (s, u) = fog_sim(los, 0);
            let sw = s.seen_sweep(u, true).unwrap();
            assert_eq!(sw.ring, want_ring, "LOS {los}");
            if want_ring {
                // The thickened annulus, out of the ring table's own
                // cumulative counts — not the circle's, which are smaller.
                let g = ring();
                let r = sw.radius as usize;
                assert_eq!((sw.start, sw.end), (g.radius[r - 1], g.radius[r]));
                assert_ne!(
                    (sw.start, sw.end),
                    (circle().radius[r - 1], circle().radius[r]),
                    "LOS {los}: the ring table's slice is the circle's"
                );
            } else {
                assert_eq!(sw.start, 1, "LOS {los}: the disc bar its centre");
                assert_eq!(sw.end, circle().radius[sw.radius as usize]);
            }
        }
        // And the standing pass is the whole disc, centre included.
        let (s, u) = fog_sim(8, 0);
        let sw = s.seen_sweep(u, false).unwrap();
        assert!(!sw.ring);
        assert_eq!((sw.start, sw.end), (0, circle().radius[4]));
    }

    /// §5: the disc is written into `seen2`, and `update_seen` answers with
    /// the count of cells newly revealed — the original's `reveal_fog`
    /// calls. A second pass from the same spot reveals nothing.
    ///
    /// **And the first pass is the unit's birth.** `Object::add_to_world`
    /// reaches `update_seen(0)` and [`Sim::add_unit`] makes that call since
    /// item 447, so `fog_sim` hands back a grid with the disc already on
    /// it; before then a unit born on the map lit nothing at all until it
    /// crossed a half-cell (`docs/COMBAT.md` §31.4). That is asserted here
    /// rather than worked around: the count is the same disc either way,
    /// and the difference is whose call made it.
    #[test]
    fn the_disc_lands_in_the_fog_and_only_new_cells_count() {
        let (mut s, u) = fog_sim(4, 0);
        let born = seen(&s, 0);
        assert_eq!(
            born,
            circle().radius[2],
            "the whole disc of radius two, lit at the unit's birth"
        );
        assert_eq!(s.update_seen(u, false), 0, "nothing new the second time");
        assert_eq!(seen(&s, 0), born);
    }

    /// §6: the trigger is a **half-cell** crossing, not any movement. A
    /// step inside the unit's own fog cell writes nothing.
    #[test]
    fn only_a_step_that_crosses_a_half_cell_reveals() {
        let (mut s, u) = fog_sim(4, 0);
        s.update_seen(u, false);
        let base = seen(&s, 0);
        let from = s.units[u].pos;
        // Inside the same fog cell: it is `0x180` wide and the unit stands
        // at its middle.
        s.units[u].pos = Pos::new(from.x + 0x20, from.y);
        assert_eq!(
            s.moved_to(u, from, true),
            None,
            "a step inside the half-cell must not reach `update_seen` at all"
        );
        assert_eq!(seen(&s, 0), base);
        let from = s.units[u].pos;
        s.units[u].pos = Pos::new(from.x + 0x180, from.y);
        assert!(
            s.moved_to(u, from, true).is_some_and(|n| n > 0),
            "a step across a half-cell reveals"
        );
        assert!(seen(&s, 0) > base);
    }

    /// §6: and the hundredth-frame resync, which is what covers a unit that
    /// never crosses one. Frame 33, not frame 0 and not frame 100.
    #[test]
    fn the_resync_runs_on_frame_thirty_three_of_each_hundred() {
        for (frame, reveals) in [
            (0, false),
            (32, false),
            (33, true),
            (100, false),
            (133, true),
        ] {
            let (mut s, u) = fog_sim(4, 0);
            // The birth disc is already on the grid (item 447), so what is
            // measured is the **delta**: the unit is teleported to ground
            // nobody has looked at — assigning `pos` reaches no reveal of
            // its own — and only the resync can light it.
            let base = seen(&s, 0);
            assert!(base > 0, "the birth disc");
            s.units[u].pos = Pos::new(6 * 0x300 + 0x180, 6 * 0x300 + 0x180);
            s.frame = frame;
            s.tick();
            assert_eq!(
                seen(&s, 0) > base,
                reveals,
                "frame {frame}: the resync {}",
                if reveals { "should run" } else { "should not" }
            );
        }
    }

    /// §10: **the resync forgets what nothing sees any more.**
    /// `update_all_seen` opens with `World::clear_seen@006b2250`, so the
    /// current plane under ground the unit has walked off goes dark for
    /// its owner, while `seen2`, the ever-seen plane, keeps it.
    ///
    /// **Made to fail on purpose**: without the clear, the birth cell is
    /// still in `seen` after the resync.
    #[test]
    fn the_resync_forgets_what_nothing_sees_any_more() {
        let (mut s, u) = fog_sim(4, 0);
        let born = (fog_of(s.units[u].pos.x), fog_of(s.units[u].pos.y));
        assert_eq!(s.world.seen(born.0, born.1), Some(1), "the birth disc");
        s.units[u].pos = Pos::new(6 * 0x300 + 0x180, 6 * 0x300 + 0x180);
        s.frame = 133;
        s.tick();
        assert_eq!(
            s.world.seen(born.0, born.1),
            Some(0),
            "nothing stands there now"
        );
        assert_eq!(s.world.seen2(born.0, born.1), Some(1), "ever seen");
        let now = (fog_of(s.units[u].pos.x), fog_of(s.units[u].pos.y));
        assert_eq!(s.world.seen(now.0, now.1), Some(1));
    }

    /// §10: **and it relights an attacker for its victims.** The whole-disc
    /// `update_seen(0)` calls `Unit::update_local_seen` first when the
    /// unit's `visible` byte is not 0 (`00651b80`), so a resync keeps the
    /// attacker's own cell lit for the players it has attacked, and only
    /// while the byte stands. Chapter eleven's guard is the diff: lit for
    /// who=1 by 1033's resync, dark from 1133's (`docs/VISION.md` §10).
    ///
    /// **Made to fail on purpose**: without the call in `update_seen`, the
    /// victim's bit is gone after the resync whatever `visible` holds.
    #[test]
    fn the_resync_relights_an_attacker_for_its_victims() {
        for (visible, lit) in [(1u8 << 1, Some(0b11)), (0, Some(0b01))] {
            let (mut s, u) = fog_sim(4, 0);
            s.units[u].visible = visible;
            let at = (fog_of(s.units[u].pos.x), fog_of(s.units[u].pos.y));
            s.frame = 133;
            s.tick();
            assert_eq!(
                s.world.seen(at.0, at.1),
                lit,
                "visible {visible:#04b}: who=1's bit on the attacker's cell"
            );
        }
    }

    /// §10.4: **a building target is seen through its `ever_seen` byte,
    /// not the fog plane** (`BuildData::is_seen@0062e1a0` →
    /// `WallData::is_seen@00642bd0`). A started building who=1 has once
    /// laid eyes on stays a legal target of who=1's after the resync has
    /// forgotten its cell; one it never saw is refused though the cell is
    /// lit; and its owner always sees it. Great Lakes is the diff: without
    /// this arm, the resync's clear dropped who=1's army's target on 8233
    /// and the word fell 14982 → 9401.
    ///
    /// **Made to fail on purpose**: through the fog plane instead, the
    /// first and second rows answer the other way.
    #[test]
    fn a_building_target_is_seen_through_its_ever_seen_byte() {
        let (mut s, b) = fog_build(4, 0, 2);
        s.buildings[b].started = true;
        let (fx, fy) = (
            s.buildings[b].pos.x / UNITS_PER_FOG,
            s.buildings[b].pos.y / UNITS_PER_FOG,
        );
        for (ever, lit, seen) in [(0b10, 0, true), (0, 0b10, false), (0b10, 0b10, true)] {
            s.buildings[b].ever_seen = ever;
            s.world.clear_seen();
            s.world.set_seen(fx, fy, lit);
            assert_eq!(
                s.build_is_seen(b, 1),
                seen,
                "ever_seen {ever:#04b}, who=1's fog bit {lit:#04b}"
            );
        }
        s.buildings[b].ever_seen = 0;
        assert!(s.build_is_seen(b, 0), "the owner");
        s.buildings[b].started = false;
        s.buildings[b].ever_seen = 0b10;
        assert!(!s.build_is_seen(b, 1), "unstarted, and not an ally's");
    }

    /// **The unpack lights the whole disc** (`SpellType::cast_unpack
    /// @006709c0`, `docs/COMBAT.md` §56): the packed bit clears, and the
    /// next call is `update_seen(0)` at the new line of sight. A siege
    /// engine born packed lights a four-tile disc; unpacked where it
    /// stands, it crosses no half-cell and would keep that disc to the
    /// next resync. run146's catapult is the diff: its search on the
    /// frame after the unpack takes a hoplite seven tiles off.
    ///
    /// **Made to fail on purpose**: without the call in `cast_unpack`, the
    /// count after the unpack is the packed disc's.
    #[test]
    fn the_unpack_lights_the_whole_disc_at_the_new_line_of_sight() {
        let mut world = World::new(40, 40);
        world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
        assert!(world.set_fog(vec![0; 80 * 80]));
        let mut s = crate::Sim::new(Tuning::RON, world, 2);
        let mut ty = crate::UnitType {
            hits: 80,
            los: 10,
            ..crate::UnitType::default()
        };
        ty.combat.packs = true;
        ty.cols.unit_flags2 |= crate::ai_load::uflags2::PACKS;
        let t = s.add_unit_type(ty);
        let at = Pos::new(20 * 0x300 + 0x180, 20 * 0x300 + 0x180);
        let mut u = crate::Unit::new(0, 0, at, 80);
        u.ty = Some(t);
        let u = s.add_unit(u);
        assert!(s.units[u].combat.packed, "a type that packs is born packed");
        assert_eq!(s.unit_los(u), PACKED_LOS);
        assert_eq!(
            seen(&s, 0),
            circle().radius[2],
            "the packed disc: four tiles, radius two"
        );
        s.cast_unpack(u);
        assert_eq!(s.unit_los(u), 10);
        assert_eq!(
            seen(&s, 0),
            circle().radius[5],
            "the unpack lights the ten-tile disc, radius five, where it stands"
        );
    }

    /// A type with no `LOS` reveals nothing at all — the `mylos == 0` head,
    /// which is what keeps a wall or a projectile out of the fog.
    #[test]
    fn a_type_with_no_los_reveals_nothing() {
        let (mut s, u) = fog_sim(0, 0);
        assert_eq!(s.seen_sweep(u, false), None);
        assert_eq!(s.update_seen(u, false), 0);
        assert_eq!(seen(&s, 0), 0);
    }

    /// §4: `ring_init`'s first rings against `circle_init`'s. Ring `r` is at
    /// least the circle's own ring — it is copied out of it — and the patch
    /// only ever adds.
    #[test]
    fn the_ring_table_contains_the_circles_own_ring_and_thickens_it() {
        let c = circle();
        let g = ring();
        assert_eq!((g.x[0], g.y[0]), (0, 0), "the centre is entry zero");
        assert_eq!(g.radius[0], 1);
        for r in 1..=RING_LAST as usize {
            let circle_ring = c.radius[r] - c.radius[r - 1];
            let ring_ring = g.radius[r] - g.radius[r - 1];
            assert!(
                ring_ring >= circle_ring,
                "ring {r}: {ring_ring} entries against the circle's {circle_ring}"
            );
            // Every entry of the circle's ring is at the front of the ring's.
            for k in 0..circle_ring {
                assert_eq!(
                    (g.x[g.radius[r - 1] + k], g.y[g.radius[r - 1] + k]),
                    (c.x[c.radius[r - 1] + k], c.y[c.radius[r - 1] + k]),
                    "ring {r} entry {k} is the circle's"
                );
            }
            // And the patch lands strictly inside ring `r`.
            for k in circle_ring..ring_ring {
                let i = g.radius[r - 1] + k;
                assert!(
                    vector_dist(g.x[i], g.y[i]) < r as i32,
                    "ring {r}'s patch at {:?} is not inside it",
                    (g.x[i], g.y[i])
                );
            }
        }
    }

    /// The thickening is not decorative: ring 1 is the eight neighbours and
    /// gains nothing, and the first ring that gains is the first with a
    /// diagonal gap.
    #[test]
    fn ring_one_is_the_eight_neighbours_and_the_thickening_starts_later() {
        let c = circle();
        let g = ring();
        assert_eq!(g.radius[1] - g.radius[0], 8, "ring 1 is the neighbourhood");
        assert_eq!(c.radius[1] - c.radius[0], 8);
        let grown: Vec<usize> = (1..=RING_LAST as usize)
            .filter(|&r| (g.radius[r] - g.radius[r - 1]) > (c.radius[r] - c.radius[r - 1]))
            .collect();
        assert!(
            !grown.is_empty(),
            "no ring is thickened, so `ring_init`'s patch never fires"
        );
    }
}
