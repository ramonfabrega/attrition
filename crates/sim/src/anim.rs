//! The animation clock — `docs/ANIM.md`.
//!
//! Every figure (`Guy`) of every unit plays an animation: `cur_time` frames
//! into one of `end_time` frames. `Objects::inc_time` steps every clock once
//! a frame (phase 7 of `Game::do_frame`, `docs/SYNC.md` §2), and a clock that
//! runs out restarts its animation through `Guy::set_anim@005da300` — which,
//! for an idle animation, **draws once from the sync stream** to pick the
//! next idle variant. That draw is why the clock is simulation state rather
//! than rendering: on run12's frame 0 forty of the 120 draws are the animals'
//! idle rolls, and on run13's sim-frame 100 twelve of the 18 are fish whose
//! idle animations ran out together.
//!
//! The lengths are art data — the animation packets' frame counts, which
//! `set_anim` reads through `AnimMgr::frames[]` — and the sim takes them as
//! an input ([`Art`]), read out of a `DUMP_ALL` dump's `GUY` blocks by
//! `rondata` until a BHA reader exists. So is the graphic piece each guy
//! plays (`GraphicPieces::get_unit_gpiece@0090c030`, by type, tribe, age,
//! gender and crew for a player's unit, and `(seed + o) % 3` for gaia's).
//!
//! Nothing here is a float: the one float in the original's path — the walk
//! variant's speed ratio — is a comparison of a quotient against `0.6f` and
//! `1.1f`, done here by cross-multiplication (§6 names the boundary).

use std::collections::{BTreeMap, BTreeSet};

use crate::orders::{MoveKind, QueuePos};
use crate::world::{Player, Pos, vector_dist};
use crate::{Sim, Unit};

// The `UnitAnim` enum (`rise.pdb`, type 0x46B1).
pub const DEFAULT: i8 = 0;
pub const IDLE1: i8 = 1;
pub const IDLE2: i8 = 2;
pub const IDLE3: i8 = 3;
pub const GROUP_IDLE1: i8 = 4;
pub const GROUP_IDLE3: i8 = 6;
pub const SLOG: i8 = 7;
pub const WALK: i8 = 8;
pub const JOG: i8 = 9;
pub const ATTACKWALK: i8 = 10;
pub const ATTACK1: i8 = 11;
pub const ATTACK2: i8 = 12;
pub const ATTACK3: i8 = 13;
pub const TURN_LEFT: i8 = 21;
pub const TURN_RIGHT: i8 = 22;
pub const CHOP_WOOD: i8 = 25;
pub const WALK_WITH_WOOD: i8 = 26;
pub const DUMP_WOOD: i8 = 27;
pub const WALK_TO_WOOD: i8 = 28;
pub const MINE_ORE: i8 = 29;
pub const WALK_WITH_ORE: i8 = 30;
pub const DUMP_ORE: i8 = 31;
pub const WALK_TO_ORE: i8 = 32;
pub const BUILD: i8 = 33;
pub const REPAIR: i8 = 34;
pub const SOW: i8 = 35;
pub const REAP: i8 = 36;
pub const FARM: i8 = 37;
pub const NUM_ANIMS: usize = 38;

/// The idle roll's draw sites, and the creation roll's.
///
/// The roll itself is one address — `Guy::set_anim+0x97a` — so a site alone
/// does not say *which* of the four things asked for it. The trace's `ebp`
/// chain does, and that is what these names carry: the roll under the
/// caller that made it. [`Sim::mark`] writes them into
/// [`Sim::phase_marks`], `rondata::trace` names the same four from the
/// chain, and the two sequences line up draw for draw (`docs/SYNC.md` §5).
///
/// The last two are what item 26 needs: the sim spends
/// [`SITE_STAND_GATHER`] in the unit loop for every gathering citizen,
/// and the original spends [`SITE_WRAP`] in phase 7 for the same figures
/// instead (`docs/SYNC.md` §4.2, §6).
pub const SITE_IDLE_ANIMAL: &str = "Guy::set_anim+0x97a < Animal::do_idle+0x19";
pub const SITE_IDLE_UNIT: &str = "Guy::set_anim+0x97a < Unit::do_idle+0x7d";
pub const SITE_WRAP: &str = "Guy::set_anim+0x97a < Guy::inc_time+0x271";
pub const SITE_STAND_GATHER: &str = "Guy::set_anim+0x97a < Unit::do_non_flat_gather+0x10f";
pub const SITE_STAND_TILE: &str = "Guy::set_anim+0x97a < Unit::do_non_flat_gather+0xfd4";
pub const SITE_STAND_RETURN: &str = "Guy::set_anim+0x97a < Unit::do_non_flat_gather+0xb99";

/// `Guy::init_real@005db6b0`'s variant roll, one per guy created.
pub const SITE_INIT_REAL: &str = "Guy::init_real+0x52";

/// `UnitAnimCat` — the category of each animation, 38 dwords at
/// `.rdata+0x2f370` of the shipped executable (`docs/ANIM.md` §2). The
/// idle variants and the group idles are category 0 (`CHAR_DEFAULT`), the
/// three walks and the four carrying walks are 8 (`CHAR_WALK`), the three
/// attacks 12 (`CHAR_ATTACK2`), the six deaths 15, and everything else is
/// its own category.
pub const CAT: [i8; NUM_ANIMS] = [
    0, 0, 0, 0, 0, 0, 0, 8, 8, 8, 10, 12, 12, 12, 12, 15, 15, 15, 15, 15, 15, 21, 22, 23, 24, 25,
    8, 27, 8, 29, 8, 31, 8, 33, 34, 35, 36, 37,
];

/// The category of an animation index.
pub fn category(anim: i8) -> i8 {
    usize::try_from(anim)
        .ok()
        .and_then(|i| CAT.get(i).copied())
        .unwrap_or(0)
}

/// Whether a running-out animation restarts itself (`AnimMgr::loopings`,
/// `set_anim(same, 0, 1)`) or falls back to the idle (`set_anim(DEFAULT,
/// 0, 1)`). The flag is per animation *file*, listed by name under
/// `<LOOPING>` / `<NONLOOPING>` in the install's `anim_graphics.xml`
/// (`GraphicPieces::init_anims_pool@008fca40`); which file a piece's slot
/// names is packet data this sim does not read, so the rule here is by
/// slot, from the names: the attacks, deaths, turns, pack/unpack and the
/// two dumps are non-looping, every walk, idle and work animation loops.
/// For a category-0 animation the flag changes nothing — both restarts
/// go through the idle roll (§4) — so it matters only for the dumps and
/// the attacks (§6).
pub fn non_looping(anim: i8) -> bool {
    (ATTACKWALK..=24).contains(&anim) || anim == DUMP_WOOD || anim == DUMP_ORE
}

/// One figure's clock — `GuyData` `+0x74..+0x9d` (`docs/ANIM.md` §1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Guy {
    /// `cur_time`: frames into the animation.
    pub cur_time: u32,
    /// `end_time`: the animation's length; 0 right after `init_real`,
    /// [`UNKNOWN`] when the table has no entry for this piece and slot.
    pub end_time: u32,
    /// `last_time`: `cur_time` before the step; −1 after a `set_anim`.
    pub last_time: i32,
    /// `cur_anim`: the `UnitAnim` slot.
    pub anim: i8,
    /// `gpiece`: the graphic piece, −1 when the table could not name one.
    pub gpiece: i32,
    /// `stopped`: the body stood on its destination at the last follow.
    pub stopped: bool,
}

/// The length a guy takes when the table has none for its piece and slot:
/// the clock steps but never wraps, so a missing entry costs no draw
/// (§6). The original's own fallback — 3, for a slot the packet lacks —
/// is not taken because an unobserved length and a missing slot cannot be
/// told apart from a dump.
pub const UNKNOWN: u32 = u32::MAX;

impl Guy {
    /// `Guy::init_real@005db6b0`'s clock, before its draw: nothing played,
    /// so the first `inc_time` overflows at once.
    pub const fn fresh(gpiece: i32) -> Guy {
        Guy {
            cur_time: 0,
            end_time: 0,
            last_time: -1,
            anim: DEFAULT,
            gpiece,
            stopped: true,
        }
    }
}

/// The art the clock reads — an input, like the map.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Art {
    /// `(gpiece, anim) → frames`: `AnimMgr::frames[action_ids[anim]]`,
    /// what `set_anim` writes into `end_time`.
    pub lengths: BTreeMap<(i32, i8), u32>,
    /// `(owner, unit type, sub, guy_num) → gpiece`, the piece
    /// `get_unit_gpiece` picks. `sub` is the gender bit `o & 1` for a
    /// player's unit and `(seed + o) % 3` for a gaia type (§3).
    pub pieces: BTreeMap<(Player, usize, u8, u8), i32>,
    /// The unit types whose pieces come three to a type by `(seed + o) % 3`
    /// — `TypeIndex` `0x192..0x19e`, the birds and the herd animals.
    pub gaia_types: BTreeSet<usize>,
    /// Pieces whose packet has a `GROUP_IDLE2` animation, which makes a
    /// captain's idle re-roll skip a draw once every 16 frames (§4.2).
    /// Empty until a dump shows one.
    pub group_idle: BTreeSet<i32>,
}

impl Art {
    /// The length of `anim` on `gpiece`, if the table has it.
    pub fn length(&self, gpiece: i32, anim: i8) -> Option<u32> {
        self.lengths.get(&(gpiece, anim)).copied()
    }
}

/// The idle variant `set_anim` picks from `p = rand % 100` when the guy was
/// already idle (`docs/ANIM.md` §4.1): 0 up to 69, `IDLE1` to 82, `IDLE2` to
/// 95, `IDLE3` above — with two collapses: a guy with `guy_flags & 0x20`
/// takes `IDLE1` for anything above 69, and a peasant standing on a tile
/// whose mask has `& 3` takes `IDLE1` for anything above 82.
pub fn idle_variant(p: i32, flag_0x20: bool, peasant_on_masked: bool) -> i8 {
    if p <= 69 {
        DEFAULT
    } else if flag_0x20 || p <= 82 || peasant_on_masked {
        IDLE1
    } else if p <= 95 {
        IDLE2
    } else {
        IDLE3
    }
}

/// `Guy::init_real`'s variant from its own `rand % 100`: 0 up to 69, then
/// `IDLE1`, `IDLE2`, `IDLE3` by tens.
pub fn init_variant(p: i32) -> i8 {
    if p < 70 {
        DEFAULT
    } else if p < 80 {
        IDLE1
    } else if p < 90 {
        IDLE2
    } else {
        IDLE3
    }
}

/// The herd centre a wandering animal measures from — `Animal::do_idle@
/// 005d7460`'s `((wx + 2·cx) · 0x300 + 0x480) / 3` on each axis: a third of
/// the way from the home cell's centre to the wander centre's, in position
/// units.
pub fn herd_centre(cx: i32, cy: i32, wx: i32, wy: i32) -> Pos {
    Pos::new(
        ((wx + 2 * cx) * 0x300 + 0x480) / 3,
        ((wy + 2 * cy) * 0x300 + 0x480) / 3,
    )
}

/// The animals' wander radius around the herd centre.
pub const WANDER_NEAR: i32 = 0x181;

impl Sim {
    /// The piece a guy of `ty` owned by `who` plays — [`Art::pieces`] by
    /// the gender bit or the gaia variant. `None` when the table has no
    /// entry; the guy then carries −1 and every length lookup fails.
    pub fn piece_of(&self, who: Player, ty: usize, o: i16, guy_num: u8) -> Option<i32> {
        let sub = if self.art.gaia_types.contains(&ty) {
            (self.game_seed.wrapping_add(i32::from(o))).rem_euclid(3) as u8
        } else {
            (o & 1) as u8
        };
        self.art.pieces.get(&(who, ty, sub, guy_num)).copied()
    }

    /// `Unit::init`'s guys: one [`Guy::fresh`] per member, each with
    /// `Guy::init_real`'s one draw (`% 100`, [`init_variant`]). The count
    /// is the type's — one, until the loader carries `num_guys`.
    pub fn init_guys(&mut self, u: usize, ty: Option<usize>) {
        let unit = &self.units[u];
        let (who, o) = (unit.owner, unit.index);
        let ty = ty.or(unit.ty);
        let count = 1usize;
        let mut guys = Vec::with_capacity(count);
        for n in 0..count {
            let piece = ty
                .and_then(|t| self.piece_of(who, t, o, n as u8))
                .unwrap_or(-1);
            let mut g = Guy::fresh(piece);
            self.mark(SITE_INIT_REAL);
            let p = self.rng.roll() % 100;
            g.anim = init_variant(p);
            // A variant the packet lacks falls back to the default: the
            // table stands in for the packet here.
            if g.anim != DEFAULT && piece >= 0 && self.art.length(piece, g.anim).is_none() {
                g.anim = DEFAULT;
            }
            guys.push(g);
        }
        self.units[u].guys = guys;
        self.units[u].born = self.frame;
    }

    /// `Unit::set_anim(anim, force, p3)`: every guy's `Guy::set_anim`.
    /// `force` is the original's second argument (a non-zero one skips the
    /// "already playing" early returns), `p3` the third (an idle request
    /// with it set re-rolls the variant).
    pub(crate) fn set_anim(&mut self, u: usize, anim: i8, force: bool, p3: bool) {
        for g in 0..self.units[u].guys.len() {
            self.guy_set_anim(u, g, anim, force, p3);
        }
    }

    /// `set_anim(CHAR_DEFAULT, 0, 1)` — the call `do_idle`, the gather
    /// stands and the arrival make.
    pub(crate) fn set_default_anim(&mut self, u: usize) {
        self.set_anim(u, DEFAULT, false, true);
    }

    /// Whether the body stands on the unit — `Guy::set_anim`'s `des == pos`
    /// test on guy 0, which is what turns an idle request on a walking guy
    /// into the arrival draw.
    fn body_at_des(&self, u: usize) -> bool {
        let unit = &self.units[u];
        unit.movement.body.pos == unit.pos
    }

    /// `Guy::set_anim@005da300`, the paths a unit on open ground reaches
    /// (`docs/ANIM.md` §4). Returns whether the stream was drawn from.
    pub(crate) fn guy_set_anim(&mut self, u: usize, g: usize, anim: i8, force: bool, p3: bool) {
        let frame = self.frame;
        let guy = self.units[u].guys[g];
        let cur_cat = category(guy.anim);
        let end_at_entry = guy.end_time;
        let who = self.units[u].owner;
        let at_des = self.body_at_des(u);

        // The early returns (`set_anim:155–224`).
        if anim == DEFAULT && !force {
            if cur_cat == 0 {
                if guy.cur_time < guy.end_time {
                    return;
                }
            } else if cur_cat == 8 && !at_des {
                // A walking guy asked to idle before its body has arrived:
                // nothing, or a rewind once the walk cycle has run out.
                if guy.cur_time >= guy.end_time {
                    self.units[u].guys[g].cur_time = 0;
                }
                return;
            }
        } else if who >= 8 && anim == WALK && cur_cat == 8 && guy.cur_time < guy.end_time {
            // Gaia's walkers: a walk already playing is left alone.
            return;
        }
        let target_cat = category(anim);
        if !force && cur_cat == target_cat && cur_cat != 8 && guy.cur_time < guy.end_time {
            return;
        }

        let target: i8 = if target_cat == 0 {
            // The idle roll. A captain whose piece has a group idle skips it
            // one frame in sixteen, phased by `o` (§4.2).
            let unit = &self.units[u];
            let gate = (frame + 0x2e + i64::from(unit.index)) & 15 == 0
                && unit.captain
                && !(GROUP_IDLE1..=GROUP_IDLE3).contains(&guy.anim)
                && g == 0
                && self.art.group_idle.contains(&guy.gpiece);
            let mut v = DEFAULT;
            if !gate {
                // `openlist == 0`: a unit with a suspended search does not
                // draw. The sim keeps no suspended search (`path.rs`).
                let p = self.rng.roll() % 100;
                if cur_cat == 0 && p3 {
                    let peasant_on_masked = self.is_peasant(u) && self.on_masked_tile(u);
                    v = idle_variant(p, self.units[u].guy_flag_0x20, peasant_on_masked);
                }
            }
            if v != DEFAULT && guy.gpiece >= 0 && self.art.length(guy.gpiece, v).is_none() {
                v = DEFAULT;
            }
            v
        } else if target_cat == 12 {
            // An attack: `ATTACK1` / `ATTACK2` / `ATTACK3` by one draw when
            // asked with `p3` (30 / 40 / 30 percent); §6.
            if p3 {
                let p = self.rng.roll() % 100;
                if p < 30 {
                    ATTACK1
                } else if p > 70 {
                    ATTACK3
                } else {
                    ATTACK2
                }
            } else {
                anim
            }
        } else if target_cat == 8 {
            self.walk_variant(u, anim)
        } else {
            anim
        };

        // The apply (`set_anim:559–573`, `707–717`): a new animation starts
        // at 0; the same one keeps what ran past its end.
        let guy = &mut self.units[u].guys[g];
        if target_cat == 8 && cur_cat == 8 && guy.anim != target {
            // A walk-to-walk slot change keeps `cur_time` as it is: the
            // original's rescale (`set_anim:691`, `cur_time · len_new /
            // len_old`) passes the *old* slot to both `get_anim_time` calls
            // — the listing at `0x5db43c`/`0x5db450`, the second reading's
            // finding (`docs/audit/2026-08-24-anim.md`) — so it is
            // `cur_time · t / t`, and the time may stand past the new
            // slot's end until the next step wraps it.
            guy.anim = target;
        } else if guy.anim != target {
            guy.anim = target;
            guy.cur_time = 0;
        } else {
            guy.cur_time -= guy.cur_time.min(end_at_entry);
        }
        guy.last_time = -1;
        guy.end_time = self.art.length(guy.gpiece, guy.anim).unwrap_or(UNKNOWN);
    }

    /// The walk slot for a walk-category request: the carrying walks by
    /// the gather state, else `SLOG` / `WALK` / `JOG` by the body's average
    /// speed against the type's base — below six tenths slogs, above
    /// eleven tenths jogs (§4.3). A bird's is a coin.
    fn walk_variant(&mut self, u: usize, anim: i8) -> i8 {
        let unit = &self.units[u];
        let mut v = anim;
        if unit.owner == 9 {
            if (0x192..=0x194).contains(&unit.type_index) && self.rng.roll() % 100 > 0x31 {
                v = JOG;
            }
        } else if anim == WALK {
            let base = unit.ty.map_or(0, |t| self.unit_types[t].moves);
            let avg = unit.movement.body.avg_speed;
            if base > 0 {
                if avg * 10 < base * 6 {
                    v = SLOG;
                } else if avg * 10 > base * 11 {
                    v = JOG;
                }
            }
        }
        let guy = unit.guys.first().copied();
        if let Some(g) = guy
            && g.gpiece >= 0
            && v != anim
            && self.art.length(g.gpiece, v).is_none()
        {
            v = anim;
        }
        v
    }

    /// `ObjectData::is_peasant`: a worker type.
    fn is_peasant(&self, u: usize) -> bool {
        self.worker_of(u) != crate::orders::Worker::None
    }

    /// The tile under the unit has `mask & 3` — the two low mask bits.
    fn on_masked_tile(&self, u: usize) -> bool {
        let t = self.units[u].pos.tile();
        self.world.tile_mask(t) & 3 != 0
    }

    /// The tail of `Objects::inc_time@0065db70` that steps the guys: every
    /// leader 0–9 in order, no rotation, each of its live units in object
    /// order — `Unit::inc_time@00610b40` → `Guy::inc_time@005d9e10` for
    /// every guy, then the buildings (no clock here). A garrisoned unit's
    /// clock stops (`inside_up ≥ 0`) unless it is a scholar (`TypeIndex`
    /// 52/53 — a scholar inside a university plays its teach/student
    /// slots), and so does a unit's on the frame it was created, which the
    /// original spends inside its building.
    pub(crate) fn guys_inc_time(&mut self, frame: i64) {
        let mut visit: Vec<usize> = Vec::with_capacity(self.units.len());
        for who in 0..10u8 {
            let mut mine: Vec<usize> = (0..self.units.len())
                .filter(|&i| self.units[i].owner == who)
                .collect();
            mine.sort_by_key(|&i| self.units[i].index);
            visit.extend(mine);
        }
        for u in visit {
            let unit = &self.units[u];
            let inside_and_not_scholar =
                unit.inside.is_some() && self.worker_of(u) != crate::orders::Worker::Scholar;
            if !unit.alive() || inside_and_not_scholar || unit.born == frame || unit.guys.is_empty()
            {
                continue;
            }
            for g in 0..self.units[u].guys.len() {
                self.guy_inc_time(u, g);
            }
        }
    }

    /// `Guy::inc_time`: the step, and the restart when the animation runs
    /// out — a looping one restarts itself, any other falls back to the
    /// idle (a run to a walk). A member past the squad's size that is not
    /// walking mirrors guy 0 instead of stepping (§5).
    fn guy_inc_time(&mut self, u: usize, g: usize) {
        let guy = self.units[u].guys[g];
        let squad = 1usize;
        if g >= squad && category(guy.anim) != 8 {
            let lead = self.units[u].guys[0];
            let mine = &mut self.units[u].guys[g];
            mine.anim = lead.anim;
            mine.cur_time = lead.cur_time;
            return;
        }
        // `ATTACK2` under `guy_flags & 4` steps by two; no unit here fights
        // through its animation.
        let step = 1u32;
        {
            let guy = &mut self.units[u].guys[g];
            guy.last_time = i32::try_from(guy.cur_time).unwrap_or(i32::MAX);
            guy.cur_time = guy.cur_time.saturating_add(step);
        }
        // The original loops until the clock is inside its animation; a
        // table with a zero length would spin, so the loop is bounded.
        for _ in 0..4 {
            let guy = self.units[u].guys[g];
            if guy.cur_time < guy.end_time {
                break;
            }
            let cat = category(guy.anim);
            self.mark(SITE_WRAP);
            if !non_looping(guy.anim) {
                self.guy_set_anim(u, g, guy.anim, false, true);
            } else if cat != 12 {
                let next = if guy.anim == ATTACKWALK {
                    WALK
                } else {
                    DEFAULT
                };
                self.guy_set_anim(u, g, next, false, true);
            } else {
                self.guy_set_anim(u, g, DEFAULT, false, false);
            }
        }
    }

    /// The walk's start and the arrival, as the body follow sees them —
    /// `Guy::move@005d9240:52–90` for guy 0. Called with the body as it
    /// stood before this frame's follow.
    pub(crate) fn guys_follow(&mut self, u: usize, was_at_des: bool) {
        if self.units[u].guys.is_empty() {
            return;
        }
        let unit = &self.units[u];
        let facing_settled = unit.movement.facing == unit.movement.des_angle;
        let anim = unit.guys[0].anim;
        if was_at_des {
            if facing_settled {
                if anim == WALK && unit.guys[0].stopped {
                    self.set_default_anim(u);
                }
                for g in &mut self.units[u].guys {
                    g.stopped = true;
                }
            }
            return;
        }
        if anim != TURN_LEFT && anim != TURN_RIGHT && anim != ATTACKWALK {
            let walk = self.walk_for(u);
            self.set_anim(u, walk, false, true);
        }
        for g in &mut self.units[u].guys {
            g.stopped = false;
        }
    }

    /// The walk a unit plays — `CHAR_WALK`, or a carrying walk when the
    /// gather machine has it heading to or from a tile (`unit_masks &
    /// 0x78000000`, `Guy::move:118–137`).
    fn walk_for(&self, u: usize) -> i8 {
        match self.gather_walk(u) {
            Some(w) => w,
            None => WALK,
        }
    }

    /// `Animal::do_idle@005d7460`: the idle request every frame, then a
    /// herd member whose idle animation is on its last frame rolls to
    /// wander — three in ten — near the herd centre by three more draws,
    /// or by a spot search further out (§7).
    pub(crate) fn animal_idle(&mut self, u: usize) {
        self.mark(SITE_IDLE_ANIMAL);
        self.set_default_anim(u);
        let unit = &self.units[u];
        if unit.kind.domain != crate::attrition::Domain::Land {
            return;
        }
        let Some(herd) = unit.herd else {
            // A herdless animal is a pasture's (`UnitData+0x86 < 0`), and
            // `do_idle` hands it straight to `think_farm_animal` with no
            // clock gate of its own — `docs/SYNC.md` §3.6.
            self.think_farm_animal(u);
            return;
        };
        let Some(g) = unit.guys.first().copied() else {
            return;
        };
        if g.end_time == 0 || g.cur_time != g.end_time - 1 {
            return;
        }
        if self.rng.roll() % 10 >= 3 {
            return;
        }
        let Some(h) = self.gaia.herds.get(herd) else {
            return;
        };
        let centre = herd_centre(h.cx, h.cy, h.wx, h.wy);
        let here = self.units[u].pos;
        if vector_dist(centre.x - here.x, centre.y - here.y) < WANDER_NEAR {
            let d = (self.rng.roll() & 7) as usize;
            let kx = self.rng.roll() & 3;
            let x = (kx + 1) * crate::ai_place::MOVE_X[d + 1] * 0x30 + here.x;
            let ky = self.rng.roll() & 3;
            let y = (ky + 1) * crate::ai_place::MOVE_Y[d + 1] * 0x30 + here.y;
            let dest = Pos::new(x, y);
            // `WorldData::is_valid`, then `detect_unit_collision` — the
            // collision test is a seam (`docs/ORDERS.md`).
            if self.world.accepts(dest) {
                self.add_move_order(u, dest, MoveKind::MoveTo, QueuePos::New, false);
            }
        } else {
            let angle = self.units[u].movement.facing;
            if let Some(spot) = self.find_nearby_spot(u, centre, 0xc0, -1, 0, angle, None) {
                self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::New, false);
            }
        }
    }

    /// Installs a guy's clock from outside — the harness, from a dump.
    pub fn set_guy(&mut self, u: usize, g: usize, guy: Guy) {
        let guys = &mut self.units[u].guys;
        while guys.len() <= g {
            guys.push(Guy::fresh(-1));
        }
        guys[g] = guy;
    }
}

impl Unit {
    /// Whether this unit is gaia's — an animal or a bird (owner 8 or 9).
    ///
    /// Which is also the bound every object search and every valid-target
    /// test stops at; see [`crate::world::PLAYER_SLOTS`].
    pub fn is_gaia(&self) -> bool {
        self.owner >= crate::world::PLAYER_SLOTS
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::Rng;
    use crate::tuning::Tuning;
    use crate::world::World;

    fn stepped(seed: u32, n: usize) -> u32 {
        let mut r = Rng::new(seed);
        for _ in 0..n {
            r.roll();
        }
        r.seed
    }

    fn sim_at(seed: u32) -> Sim {
        let mut s = Sim::new(Tuning::RON, World::new(60, 60), 2);
        s.rng = Rng::new(seed);
        s
    }

    fn animal(s: &mut Sim, who: Player, o: i16, piece: i32, anim: i8, cur: u32, end: u32) -> usize {
        let mut u = Unit::new(who, o, Pos::new(3000, 3000), 10);
        u.guys = vec![Guy {
            cur_time: cur,
            end_time: end,
            last_time: -1,
            anim,
            gpiece: piece,
            stopped: true,
        }];
        s.add_unit(u)
    }

    /// The categories as the executable holds them: idles 0, walks 8, the
    /// carrying walks 8, the sow its own.
    #[test]
    fn the_categories() {
        assert_eq!(category(IDLE3), 0);
        assert_eq!(category(GROUP_IDLE3), 0);
        assert_eq!(category(SLOG), 8);
        assert_eq!(category(JOG), 8);
        assert_eq!(category(WALK_TO_WOOD), 8);
        assert_eq!(category(WALK_WITH_ORE), 8);
        assert_eq!(category(ATTACK1), 12);
        assert_eq!(category(ATTACK3), 12);
        assert_eq!(category(CHOP_WOOD), 25);
        assert_eq!(category(SOW), 35);
        assert_eq!(category(FARM), 37);
        assert_eq!(category(40), 0);
    }

    /// Run13, sim-frame 100: the twelve `HERDFISH` on piece 60073 (length
    /// 101) reach 101 together in `o` order and each rolls one `% 100`
    /// from `0x60032f25`, the word at the end of 99 — the replay gives the
    /// variants `2,0,0,1,0,0,0,0,0,0,0,0`, which are the dump's new
    /// `cur_anim`s (`docs/SYNC.md` §4.1).
    #[test]
    fn run13_s_fish_wrap_together() {
        let mut s = sim_at(0x6003_2f25);
        for (p, a) in [(60073, 0), (60074, 1), (60072, 2)] {
            s.art.lengths.insert((p, a), 0);
        }
        for a in 0..4 {
            s.art.lengths.insert((60073, a), 101);
            s.art.lengths.insert((60074, a), 116);
            s.art.lengths.insert((60072, a), 170);
        }
        let mut fish = Vec::new();
        for o in 4..=37 {
            let (piece, end) = match o % 3 {
                1 => (60073, 101),
                2 => (60074, 116),
                _ => (60072, 170),
            };
            let cur = 100;
            fish.push((o, animal(&mut s, 8, o, piece, 0, cur, end)));
        }
        s.frame = 100;
        s.guys_inc_time(100);
        let got: Vec<i8> = fish
            .iter()
            .filter(|(o, _)| o % 3 == 1)
            .map(|&(_, u)| s.units[u].guys[0].anim)
            .collect();
        assert_eq!(got, vec![2, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
        for &(o, u) in &fish {
            let g = s.units[u].guys[0];
            if o % 3 == 1 {
                assert_eq!((g.cur_time, g.end_time, g.last_time), (0, 101, -1), "o {o}");
            } else {
                assert_eq!(g.cur_time, 101, "o {o} keeps counting");
            }
        }
        assert_eq!(s.rng.seed, stepped(0x6003_2f25, 12));
    }

    /// Run12, frame 0, draws 48–87: forty animals fresh from `init_real`
    /// (`end_time 0`) each roll their first idle from `set_anim` on their
    /// first `do_idle`, in `o` order — the variants the dump's end-of-frame
    /// state shows at offset 48 and nowhere else. The four sheep are
    /// pieces 60063/4/5 by `(seed + o) % 3` with seed 7236, the fish
    /// 60072/3/4; the lengths are the dump's.
    #[test]
    fn run12_s_forty_first_idles() {
        let mut s = sim_at(stepped(0x3bd3_9ae9, 48));
        s.game_seed = 7236;
        for a in 0..4 {
            s.art.lengths.insert((60063, a), 90);
            s.art.lengths.insert((60064, a), 109);
            s.art.lengths.insert((60065, a), 250);
            s.art.lengths.insert((60073, a), 101);
            s.art.lengths.insert((60074, a), 116);
            s.art.lengths.insert((60072, a), 170);
        }
        let mut units = Vec::new();
        for o in 0..40i16 {
            let sheep = o < 4;
            let base = if sheep { 60063 } else { 60072 };
            let piece = base + (7236 + i32::from(o)) % 3;
            let end = 0;
            let u = animal(&mut s, 8, o, piece, 0, 0, end);
            units.push(u);
        }
        for &u in &units {
            s.set_default_anim(u);
        }
        assert_eq!(s.rng.seed, stepped(0x3bd3_9ae9, 88), "forty draws");
        // The dump's end-of-frame-0 `cur_anim` for `o` 0–39, read from
        // run12 with `tools/gamelog/anims.py` — and the LCG's rolls 48–87
        // give exactly these under the idle thresholds.
        let got: Vec<i8> = units.iter().map(|&u| s.units[u].guys[0].anim).collect();
        assert_eq!(
            got,
            vec![
                0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 3, 0, 2, 1, 0, 2, 0, 0,
                0, 0, 0, 1, 1, 3, 0, 3, 2, 1, 0, 0
            ]
        );
        assert_eq!(s.units[units[0]].guys[0].end_time, 90);
        assert_eq!(s.units[units[1]].guys[0].end_time, 109);
        assert_eq!(s.units[units[2]].guys[0].end_time, 250);
        assert_eq!(s.units[units[5]].guys[0].end_time, 116);
        assert_eq!(s.units[units[6]].guys[0].end_time, 170);
    }

    /// A walking guy asked to idle draws nothing until the body arrives,
    /// then one draw starts the idle at 0 — the sheep at run13's 101.
    #[test]
    fn the_arrival_draws_once() {
        let mut s = sim_at(7);
        s.art.lengths.insert((60063, SLOG), 16);
        s.art.lengths.insert((60063, DEFAULT), 90);
        let u = animal(&mut s, 8, 0, 60063, SLOG, 11, 16);
        s.units[u].movement.body.pos = Pos::new(2990, 3000);
        let before = s.rng.seed;
        s.set_default_anim(u);
        assert_eq!(s.rng.seed, before, "still walking: no draw");
        assert_eq!(s.units[u].guys[0].anim, SLOG);
        s.units[u].movement.body.pos = Pos::new(3000, 3000);
        s.set_default_anim(u);
        assert_eq!(s.rng.seed, stepped(before, 1));
        let g = s.units[u].guys[0];
        assert_eq!((g.anim, g.cur_time, g.end_time), (DEFAULT, 0, 90));
    }

    /// The step: one a frame, `last_time` trailing; a work animation that
    /// runs out restarts silently, keeping the overrun.
    #[test]
    fn the_step_and_a_silent_restart() {
        let mut s = sim_at(7);
        s.art.lengths.insert((6688, SOW), 47);
        let u = animal(&mut s, 0, 3, 6688, SOW, 45, 47);
        let before = s.rng.seed;
        s.guys_inc_time(0);
        assert_eq!(
            (s.units[u].guys[0].cur_time, s.units[u].guys[0].last_time),
            (46, 45)
        );
        s.guys_inc_time(1);
        let g = s.units[u].guys[0];
        assert_eq!((g.anim, g.cur_time, g.last_time), (SOW, 0, -1));
        assert_eq!(s.rng.seed, before, "the sow loops without a draw");
    }

    /// A second guy past the squad's size mirrors guy 0 and never draws —
    /// the human scout's two guys wrap on one draw (run13's 101).
    #[test]
    fn the_mirror() {
        let mut s = sim_at(0xa45f_ecaf);
        s.art.lengths.insert((371, DEFAULT), 61);
        s.art.lengths.insert((13043, DEFAULT), 61);
        let u = animal(&mut s, 0, 0, 371, DEFAULT, 60, 61);
        s.units[u].guys.push(Guy {
            cur_time: 60,
            end_time: 61,
            last_time: -1,
            anim: DEFAULT,
            gpiece: 13043,
            stopped: true,
        });
        s.frame = 101;
        s.guys_inc_time(101);
        assert_eq!(s.rng.seed, stepped(0xa45f_ecaf, 1));
        let (a, b) = (s.units[u].guys[0], s.units[u].guys[1]);
        assert_eq!((a.cur_time, a.anim), (0, b.anim));
        assert_eq!(b.cur_time, 0);
        assert_eq!(b.last_time, -1);
    }

    /// The idle roll's thresholds, and `init_real`'s.
    #[test]
    fn the_variants() {
        assert_eq!(idle_variant(69, false, false), DEFAULT);
        assert_eq!(idle_variant(70, false, false), IDLE1);
        assert_eq!(idle_variant(82, false, false), IDLE1);
        assert_eq!(idle_variant(83, false, false), IDLE2);
        assert_eq!(idle_variant(95, false, false), IDLE2);
        assert_eq!(idle_variant(96, false, false), IDLE3);
        assert_eq!(idle_variant(96, true, false), IDLE1);
        assert_eq!(idle_variant(96, false, true), IDLE1);
        assert_eq!(init_variant(69), DEFAULT);
        assert_eq!(init_variant(79), IDLE1);
        assert_eq!(init_variant(89), IDLE2);
        assert_eq!(init_variant(99), IDLE3);
    }

    /// Run12's frame 0: the herd centre of herd 0 at home `(22, 34)`,
    /// wander `(22, 34)`, is the cell's centre.
    #[test]
    fn the_herd_centre() {
        assert_eq!(
            herd_centre(22, 34, 22, 34),
            Pos::new(22 * 768 + 384, 34 * 768 + 384)
        );
    }
}
