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

use crate::movement::Angle;
use crate::orders::{MoveKind, QueuePos};
use crate::world::{Player, Pos, vector_dist};
use crate::{Sim, Unit};

// The `UnitAnim` enum (`rise.pdb`, type 0x46B1).
pub const DEFAULT: i8 = 0;
pub const IDLE1: i8 = 1;
pub const IDLE2: i8 = 2;
pub const IDLE3: i8 = 3;
pub const GROUP_IDLE1: i8 = 4;
pub const GROUP_IDLE2: i8 = 5;
pub const GROUP_IDLE3: i8 = 6;
pub const SLOG: i8 = 7;
pub const WALK: i8 = 8;
pub const JOG: i8 = 9;
pub const ATTACKWALK: i8 = 10;
pub const ATTACK1: i8 = 11;
pub const ATTACK2: i8 = 12;
pub const ATTACK3: i8 = 13;
pub const ATTACKSPECIAL: i8 = 14;
pub const DEATH_STAB1: i8 = 15;
pub const DEATH_STAB2: i8 = 16;
pub const DEATH_SHOT1: i8 = 17;
pub const DEATH_SHOT2: i8 = 18;
pub const DEATH_SPLODED1: i8 = 19;
pub const DEATH_SPLODED2: i8 = 20;
pub const TURN_LEFT: i8 = 21;
pub const TURN_RIGHT: i8 = 22;
pub const PACK: i8 = 23;
pub const UNPACK: i8 = 24;
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

/// `Unit::move_step@005faf30+0x823` — the stand a blocked unit plays. The
/// call is `Unit::set_anim`'s, so the chain runs through `+0x56`; the
/// disambiguator is the frame above it (`docs/COLLISION.md` §5).
pub const SITE_BLOCKED: &str = "Guy::set_anim+0x97a < Unit::move_step+0x823";

/// `Guy::move@005d9240+0x19f` — the arrival stand. A guy whose body has
/// caught up with its destination and whose angle is settled, still on a
/// walk it has been told to stop (`field_0x9c == 8 && field_0x9d`), is
/// asked for the idle; the call is `Guy::move`'s own, not `Unit::set_anim`'s,
/// so the chain is one frame shorter than the four above
/// (`docs/ANIM.md` §4, `docs/SYNC.md` §3.10).
pub const SITE_ARRIVE: &str = "Guy::set_anim+0x97a < Guy::move+0x19f";

/// `Guy::init_real@005db6b0`'s variant roll, one per guy created.
pub const SITE_INIT_REAL: &str = "Guy::init_real+0x52";

/// `Guy::set_anim+0x104b` — the coin a gaia bird throws for its wing beat
/// every time a walk-category animation is resolved for it: `rnd % 100 >
/// 0x31` takes `CHAR_JOG` (*Bird Flap*), anything else `CHAR_WALK`
/// (*Bird Soar*). It is its own address, not the `+0x97a` the other four
/// share, so the trace names it without a chain (`docs/SYNC.md` §3.9).
pub const SITE_BIRD_COIN: &str = "Guy::set_anim+0x104b";

/// `TypeIndex::BIRD` = `BASE_GAIATYPES`, the wild bird — the one type
/// `set_anim` names by identity twice: in the walk coin's guard and in the
/// same-category early return it is exempt from.
pub const BIRD_TYPE: i32 = crate::gaia::BIRD_TYPE_INDEX;

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
    /// A **crew** guy's own body, or `None` for one that has none.
    ///
    /// Guy 0's body is the unit's — `Movement::body` and
    /// `Movement::facing`, which the unit step's turn rate reads — so it
    /// is never held here. A crew guy whose piece names a track offset
    /// (`Art::tracks`) has a second body that walks its own destination,
    /// and this is it; one without stands on its leader and is turned by
    /// it, so it has none either (`Guy::do_turn@005d97a0:37` recurses only
    /// into the trackless crew).
    pub follow: Option<Follow>,
}

/// A crew guy's own body — `GuyData`'s second half, for the guys
/// `Guy::move`'s third branch walks.
///
/// `docs/MOVEMENT.md`, "The follower's destination". Everything here is a
/// field of the same `GuyData` the clock above is; they are split only
/// because guy 0's live on [`crate::Movement`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Follow {
    /// `x` / `y` (`+0xc` / `+0x10`), with `last_speed` and `avg_speed`
    /// (`+0x80` / `+0x84`) — this guy's own [`crate::movement::Body`].
    pub body: crate::movement::Body,
    /// `des_x` / `des_y` (`+0x5c` / `+0x60`): where it is trying to be,
    /// written by its leader's `Guy::set_new_location` on every frame the
    /// leader moves and left alone on every frame it does not.
    pub des: Pos,
    /// `angle` (`+0x18`): its own facing, turned by its own `Guy::move`.
    pub facing: Angle,
    /// `des_angle` (`+0x64`): its leader's facing, as of the last frame
    /// the leader moved.
    pub des_angle: Angle,
    /// `track_dx` / `track_dy` (`+0x54` / `+0x58`), from [`Art::tracks`].
    pub track: (i32, i32),
}

/// The length a guy takes when the table has none for its piece and slot:
/// the clock steps but never wraps, so a missing entry costs no draw
/// (§6). The original's own fallback is [`MISSING`], and it is not taken
/// here because an unobserved length and a missing slot cannot be told
/// apart *from a dump* — only from the install's own tables, which is
/// what [`Art::gaia_lengths`] carries for the types it covers.
pub const UNKNOWN: u32 = u32::MAX;

/// `AnimationPacket::get_game_frames@00918cc0`'s own `return 3`: the
/// length of a slot the packet does not name. It is reachable only where
/// the slot list is known — the gaia types the install describes — and it
/// is what a bird's `CHAR_DEFAULT` is worth, since `WILDBIRD` names two
/// animations and neither is an idle.
pub const MISSING: u32 = 3;

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
            follow: None,
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
    /// `gpiece → (slot → frames)` for every graphic piece the install's own
    /// `<UNIT>` entries name — the same table as [`Art::lengths`], read
    /// from the whole of `unit_graphics.xml` rather than from what a dump
    /// happened to play, and **it wins where it speaks**.
    ///
    /// The difference is not cosmetic. A dump shows a length only where
    /// something played it, and the fallback for a length nothing knew was
    /// `CHAR_DEFAULT` — so a scout that rolled `IDLE1` played its default
    /// idle instead, and its clock ran to the wrong number
    /// (`docs/ANIM.md` §3.2). A piece here has its **whole** slot list, so
    /// a slot it omits is the packet's own missing slot: [`MISSING`]
    /// frames, and a variant the roll must fall back from.
    pub piece_lengths: BTreeMap<i32, BTreeMap<i8, u32>>,
    /// `(TypeIndex, variant, slot) → frames` for the gaia types, read from
    /// the install's own `unit_graphics.xml` rather than from a dump
    /// (`rondata::artdata`, §3.1). It is the stronger of the two sources
    /// and takes precedence where it speaks: a `(type, variant)` it
    /// mentions at all is one whose **whole slot list** is known, so a
    /// slot missing from it is the packet's own missing slot and takes
    /// [`MISSING`] rather than [`UNKNOWN`].
    ///
    /// This is what gives gaia's bird a length. No dump prints owner 9,
    /// so its piece is unknown and every `lengths` lookup fails; the
    /// install says `WILDBIRD` plays *Bird Soar* for `CHAR_WALK` and
    /// *Bird Flap* for `CHAR_JOG`, 31 frames and 23 (`docs/SYNC.md` §3.9).
    pub gaia_lengths: BTreeMap<(i32, u8, i8), u32>,
    /// `gpiece → (track_dx, track_dy)`: the offset a **crew** guy is held
    /// at behind and beside the guy it follows —
    /// `GuyData::track_dx / track_dy` (`+0x54` / `+0x58`), which
    /// `Guy::update_gpiece@005d8530` reads out of the piece's own art
    /// (`rondata::artdata::piece_tracks`).
    ///
    /// A piece absent here has no track, which is the answer for every
    /// guy 0 and for a crew guy that stands on its leader: both branches
    /// of `Guy::move` that read the pair test it against zero. A guy with
    /// a track walks its own body toward its own destination and makes its
    /// own `set_anim` decisions — `docs/MOVEMENT.md`, "The follower's
    /// destination".
    pub tracks: BTreeMap<i32, (i32, i32)>,
}

impl Art {
    /// The length of `anim` on `gpiece`, if the table has it.
    pub fn length(&self, gpiece: i32, anim: i8) -> Option<u32> {
        self.lengths.get(&(gpiece, anim)).copied()
    }

    /// Whether [`Art::gaia_lengths`] describes this type and variant — a
    /// packet whose slot list is known, so a lookup that misses is a
    /// missing slot rather than an unobserved one.
    pub fn knows_gaia(&self, ty: i32, variant: u8) -> bool {
        self.gaia_lengths
            .range((ty, variant, i8::MIN)..=(ty, variant, i8::MAX))
            .next()
            .is_some()
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

    /// The gaia variant of a unit — `(seed + o) % 3`, the index into the
    /// three `<UNIT name="X-TYPE<v>">` entries the install names for a
    /// gaia type and the `sub` [`Sim::piece_of`] keys by.
    fn gaia_variant(&self, u: usize) -> u8 {
        let o = self.units[u].index;
        (self.game_seed.wrapping_add(i32::from(o))).rem_euclid(3) as u8
    }

    /// `end_time` for one guy's slot: the install's own table for a gaia
    /// type it describes, and the dump-derived `(gpiece, slot)` one for
    /// everybody else.
    ///
    /// The two are the same table read from two ends, and the install's
    /// is the complete one — which is the whole difference for a bird,
    /// whose piece no dump has ever named. Where the install describes a
    /// `(type, variant)`, a slot it omits is the packet's missing slot and
    /// gets [`MISSING`]; where it does not, an omission is only an
    /// unobserved length and gets [`UNKNOWN`].
    pub(crate) fn slot_length(&self, u: usize, gpiece: i32, anim: i8) -> Option<u32> {
        let ty = self.units[u].type_index;
        let v = self.gaia_variant(u);
        if self.art.knows_gaia(ty, v) {
            return Some(
                self.art
                    .gaia_lengths
                    .get(&(ty, v, anim))
                    .copied()
                    .unwrap_or(MISSING),
            );
        }
        if let Some(slots) = self.art.piece_lengths.get(&gpiece) {
            return Some(slots.get(&anim).copied().unwrap_or(MISSING));
        }
        self.art.length(gpiece, anim)
    }

    /// Whether the guy's **packet** names this slot at all —
    /// `AnimationPacket::get_animobj(packet, slot) != NULL`, which is the
    /// test `set_anim` makes before falling a variant back to
    /// `CHAR_DEFAULT` (`:546`) and a walk back to `CHAR_WALK` (`:596`).
    ///
    /// It is not the same question as [`Sim::slot_length`]: a slot the
    /// packet lacks still gets a length, the three frames
    /// `AnimationPacket::get_game_frames` returns. Where neither the
    /// install nor a dump describes the piece the answer is the dump's —
    /// a length nothing has seen reads as a slot nothing has, which is
    /// what this crate did everywhere before the install was read.
    pub(crate) fn packet_has(&self, u: usize, gpiece: i32, anim: i8) -> bool {
        let ty = self.units[u].type_index;
        let v = self.gaia_variant(u);
        if self.art.knows_gaia(ty, v) {
            return self.art.gaia_lengths.contains_key(&(ty, v, anim));
        }
        match self.art.piece_lengths.get(&gpiece) {
            Some(slots) => slots.contains_key(&anim),
            None => self.art.length(gpiece, anim).is_some(),
        }
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
            if g.anim != DEFAULT && piece >= 0 && !self.packet_has(u, piece, g.anim) {
                g.anim = DEFAULT;
            }
            guys.push(g);
        }
        self.units[u].guys = guys;
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

    /// Whether **this guy's** body stands on its destination —
    /// `Guy::set_anim@005da300:111` and `:163`, which is what turns an idle
    /// request on a walking guy into the arrival draw.
    ///
    /// The test the original writes is `des_x != x - off_x || des_y != y -
    /// off_y`, and `GuyData::off_x / off_y` (`+0x92` / `+0x94`) are
    /// **always zero**: `Guy::clear@005db590:49` writes the pair once at
    /// construction and nothing in the executable writes them again — the
    /// only other mentions are the two readers, here and in `Guy::move`.
    /// So it is `des == pos`, and the subtraction is dropped.
    ///
    /// Guy 0's destination is the unit's own position; a crew guy with a
    /// track offset has its own ([`Follow::des`]).
    fn body_at_des(&self, u: usize, g: usize) -> bool {
        let unit = &self.units[u];
        match unit.guys.get(g).and_then(|g| g.follow) {
            Some(f) => f.body.pos == f.des,
            None => unit.movement.body.pos == unit.pos,
        }
    }

    /// `Guy::set_anim@005da300`, the paths a unit on open ground reaches
    /// (`docs/ANIM.md` §4). Returns whether the stream was drawn from.
    pub(crate) fn guy_set_anim(&mut self, u: usize, g: usize, anim: i8, force: bool, p3: bool) {
        let frame = self.frame;
        let guy = self.units[u].guys[g];
        let cur_cat = category(guy.anim);
        let end_at_entry = guy.end_time;
        let who = self.units[u].owner;
        let at_des = self.body_at_des(u, g);

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
        // `set_anim:219` — the same category, already inside its length,
        // is left alone. The walk category is the exception, because a
        // walk re-resolves its slot from the body's speed every time it is
        // asked — **unless the type is `BIRD`**, whose slot is a coin and
        // is not re-thrown while the wing beat is still running.
        if !force
            && cur_cat == target_cat
            && (cur_cat != 8 || self.units[u].type_index == BIRD_TYPE)
            && guy.cur_time < guy.end_time
        {
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
                && self.packet_has(u, guy.gpiece, GROUP_IDLE2);
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
            if v != DEFAULT && guy.gpiece >= 0 && !self.packet_has(u, guy.gpiece, v) {
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
        let (piece, slot) = (guy.gpiece, guy.anim);
        self.units[u].guys[g].end_time = self.slot_length(u, piece, slot).unwrap_or(UNKNOWN);
    }

    /// The walk slot for a walk-category request (`set_anim:613–657`).
    ///
    /// **The slot asked for does not enter it.** `set_anim` dispatches on
    /// `UnitAnimCat[anim]`, and the walk arm opens with that category —
    /// `CHAR_WALK` — as the answer, so every walk request is re-resolved
    /// from scratch: gaia's bird by a coin, everyone else by the body's
    /// average speed against the type's base (below six tenths slogs,
    /// above eleven tenths jogs, §4.3), and then the carrying walks by the
    /// **gather mask** rather than by what the caller named. A wrap's
    /// `set_anim(CHAR_JOG)` therefore re-throws the bird's coin and can
    /// hand it back `CHAR_WALK`, which is what makes run14's bird alternate
    /// its two wing beats (`docs/SYNC.md` §3.9).
    fn walk_variant(&mut self, u: usize, _anim: i8) -> i8 {
        let unit = &self.units[u];
        let mut v = WALK;
        if unit.owner == 9 {
            if (BIRD_TYPE..=0x194).contains(&unit.type_index) {
                self.mark(SITE_BIRD_COIN);
                if self.rng.roll() % 100 > 0x31 {
                    v = JOG;
                }
            }
        } else {
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
        // The gather mask's override — `unit_masks & 0x78000000`, the same
        // read `Guy::move` makes, and it wins over the speed.
        if let Some(w) = self.gather_walk(u) {
            v = w;
        }
        // `set_anim:596` — a slot the packet lacks falls back to
        // `CHAR_WALK`, the category's own.
        let guy = self.units[u].guys.first().copied();
        if let Some(g) = guy
            && g.gpiece >= 0
            && v != WALK
            && !self.packet_has(u, g.gpiece, v)
        {
            v = WALK;
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
    /// slots).
    ///
    /// **A unit created this frame is *not* skipped**, whoever owns it. A
    /// bird is created on open ground in the middle of
    /// `Objects::process_all`, so the same frame's `inc_time` reaches it,
    /// and with `end_time` still `Guy::init_real`'s zero it wraps at once
    /// — run14's frame 96, the sampling frame, carries that wrap
    /// (`docs/SYNC.md` §3.9). So is a **trained** unit, for the same
    /// reason and by the same clock: `Build::do_queue` runs in
    /// `Objects::process_all`'s *second* loop, after every unit and before
    /// `Objects::inc_time`. Run33's frame 99 is the record — the citizen's
    /// two draws are `Guy::init_real+0x52` and
    /// `Guy::set_anim+0x97a < Guy::inc_time+0x271`, not the
    /// `Unit::do_idle+0x7d` this crate rolled until 2026-08-30 — and
    /// run13's `1/6` ends that frame at `0/232, last −1`, which is the
    /// state the wrap's `set_anim` leaves and not the state
    /// `Guy::init_real` does (`docs/SYNC.md` §3.16).
    pub(crate) fn guys_inc_time(&mut self) {
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
            if !unit.alive() || inside_and_not_scholar || unit.guys.is_empty() {
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
        // The original loops until the clock is inside its animation, and
        // the loop is real: a bird's coin can hand a walk-to-walk change
        // an old `cur_time` that overruns the new slot too, and run14's
        // frame 243 spends **nine** coins in one wrap before the slot
        // repeats itself. A table with a zero length would spin, so the
        // loop is bounded well above what any capture has needed.
        for _ in 0..64 {
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
    /// `Guy::move@005d9240:52–90`, for every guy whose body **is** guy 0's.
    /// Called with the body as it stood before this frame's follow.
    ///
    /// A crew guy with a track offset has a body of its own and is left
    /// out here: [`Sim::process_follower`] runs the same arm on its own
    /// `des` and its own angles.
    pub(crate) fn guys_follow(&mut self, u: usize, was_at_des: bool) {
        if self.units[u].guys.is_empty() {
            return;
        }
        // A bird has no ground body to follow: `Unit::do_air_physics`
        // moves it with `set_new_location` and asks for `CHAR_WALK`
        // itself, and `Guy::move`'s arrival half never runs for it. This
        // crate parks the bird on its hatch cell (`orders.rs`), so without
        // this the standing body would ask it to idle every frame and
        // spend a draw the original never spends.
        if self.units[u].type_index == BIRD_TYPE {
            return;
        }
        let unit = &self.units[u];
        let settled = unit.movement.facing == unit.movement.heading;
        for g in 0..self.units[u].guys.len() {
            if self.units[u].guys[g].follow.is_some() {
                continue;
            }
            self.guy_follow_anim(u, g, was_at_des, settled);
            if !was_at_des {
                self.units[u].guys[g].stopped = false;
            }
        }
    }

    /// `Guy::move`'s animation half for **one** guy, on that guy's own
    /// `des` and its own pair of angles.
    ///
    /// `at_des` is `des == pos` for this guy and `settled` is
    /// `des_angle == angle` for it; for every guy but a tracked crew one
    /// both are guy 0's, because every other guy shares guy 0's body.
    ///
    /// The moving arm does **not** write `stopped`: `Guy::move` writes it
    /// at the foot of the function, past the tracked branch's turn-gate
    /// `return`, so a crew guy that spends its frame turning keeps the
    /// flag it had. The caller writes it where the original reaches it.
    pub(crate) fn guy_follow_anim(&mut self, u: usize, g: usize, at_des: bool, settled: bool) {
        let anim = self.units[u].guys[g].anim;
        if at_des {
            if settled {
                if anim == WALK && self.units[u].guys[g].stopped {
                    self.mark(SITE_ARRIVE);
                    self.guy_set_anim(u, g, DEFAULT, false, true);
                }
                self.units[u].guys[g].stopped = true;
                return;
            }
            // **Standing but still owed a turn**: `Guy::move:73–89` puts the
            // body back on `CHAR_WALK` and marks it unstopped, so the next
            // frame's `do_idle` sees the walk category again and rolls
            // again. That second roll is the arrival's second draw
            // (`docs/SYNC.md` §3.11, the pair) and it is the plain walk,
            // not [`Sim::walk_for`]'s carrying one — the carry nibble is
            // read in the *moving* half of `Guy::move` and not here.
            //
            // The arm's guard is a **sea** unit (`type+0x218 == 1`) or a
            // `SPECIAL_ANIM` order, either of which only marks the body
            // stopped. `SPECIAL_ANIM` is not modelled in this crate at all
            // (`docs/ORDERS.md` §3), so only the first is tested.
            if self.units[u].kind.domain == crate::attrition::Domain::Sea {
                self.units[u].guys[g].stopped = true;
                return;
            }
            if anim != TURN_LEFT && anim != TURN_RIGHT && anim != ATTACKWALK {
                self.guy_set_anim(u, g, WALK, false, true);
            }
            self.units[u].guys[g].stopped = false;
            return;
        }
        if anim != TURN_LEFT && anim != TURN_RIGHT && anim != ATTACKWALK {
            let walk = self.walk_for(u);
            self.guy_set_anim(u, g, walk, false, true);
        }
    }

    /// The walk a unit plays — `CHAR_WALK`, or a carrying walk when the
    /// gather machine has it heading to or from a tile (`unit_masks &
    /// 0x78000000`, `Guy::move:118–137`).
    pub(crate) fn walk_for(&self, u: usize) -> i8 {
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
        self.mark(crate::gaia::SITE_WANDER_ROLL);
        if self.rng.roll() % 10 >= 3 {
            return;
        }
        let Some(h) = self.gaia.herds.get(herd) else {
            return;
        };
        let centre = herd_centre(h.cx, h.cy, h.wx, h.wy);
        let here = self.units[u].pos;
        if vector_dist(centre.x - here.x, centre.y - here.y) < WANDER_NEAR {
            self.mark(crate::gaia::SITE_WANDER_DIR);
            let d = (self.rng.roll() & 7) as usize;
            self.mark(crate::gaia::SITE_WANDER_X);
            let kx = self.rng.roll() & 3;
            let x = (kx + 1) * crate::ai_place::MOVE_X[d + 1] * 0x30 + here.x;
            self.mark(crate::gaia::SITE_WANDER_Y);
            let ky = self.rng.roll() & 3;
            let y = (ky + 1) * crate::ai_place::MOVE_Y[d + 1] * 0x30 + here.y;
            let dest = Pos::new(x, y);
            // `WorldData::is_valid`, then **`detect_unit_collision(dest,
            // quick 1, boats 1, 0, 0, 0)`** — a wander onto an occupied
            // cell is not ordered at all. The sheep of this lobby's one
            // herd stand shoulder to shoulder, so this is the gate that
            // keeps them still: without it `8/1` walks off on frame 108
            // where the original's has not moved a unit in 120 frames.
            // `boats` is asked for, but a herd animal is neither
            // sea-domain nor a hero nor supply, so the arm never fires.
            if self.world.accepts(dest) && !self.detect_quick(u, dest) {
                self.add_move_order(u, dest, MoveKind::MoveTo, QueuePos::New, false);
            }
        } else {
            // **The bearing the sweep starts from is a literal, not the
            // animal's facing.** `Animal::do_idle@005d7460` passes
            // `0x55555555` — [`crate::movement::Angle::INITIAL`], the same
            // 120° `Unit::init` writes into a unit that has never turned —
            // as `find_nearby_spot`'s ninth argument, where every other
            // call site passes a real bearing. So a herd's far wanderers
            // all sweep from the same direction whatever way they happen
            // to be looking (`docs/SYNC.md` §3.19, `docs/ANIM.md` §7).
            let spot = self.find_nearby_spot(u, centre, 0xc0, -1, 0, Angle::INITIAL, None);
            if let Some(spot) = spot {
                self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::New, false);
            }
        }
    }

    /// Installs a guy's **clock** from outside — the harness, from a dump.
    ///
    /// Only the clock: a crew guy's own body ([`Guy::follow`]) is derived
    /// rather than read, so it survives a reinstall. The harness re-seats
    /// clocks on every traced frame, and a body cleared there would put
    /// the crew back on its leader every time the dump spoke.
    pub fn set_guy(&mut self, u: usize, g: usize, guy: Guy) {
        let guys = &mut self.units[u].guys;
        while guys.len() <= g {
            guys.push(Guy::fresh(-1));
        }
        let follow = guys[g].follow;
        guys[g] = Guy { follow, ..guy };
    }

    /// Seats every crew guy on its offset — `Unit::set_new_location`'s
    /// placement path, whose `param_3` reaches `Guy::set_new_location(guy
    /// 0, pos, 1)` and puts the crew *on* its destination rather than
    /// letting it walk there.
    ///
    /// This is also where a crew guy's track offset is read: it is a
    /// property of the graphic piece `get_unit_gpiece` handed it, which
    /// `Guy::update_gpiece@005d8530` turns into `track_dx / track_dy`
    /// ([`Art::tracks`]). Guy 0 never has one — the function's first test
    /// is `guy_num != 0` — and a crew guy whose piece names none keeps
    /// [`Guy::follow`] `None` and goes on sharing guy 0's body, which is
    /// exactly what the original's trackless crew does.
    ///
    /// Idempotent, and the harness calls it once the dump's guys and their
    /// pieces are in.
    pub fn seat_guys(&mut self, u: usize) {
        let unit = &self.units[u];
        let (pos, facing) = (unit.pos, unit.movement.facing);
        let bound = Pos::new(
            self.world.width() * crate::world::UNITS_PER_CELL,
            self.world.height() * crate::world::UNITS_PER_CELL,
        );
        for g in 1..self.units[u].guys.len() {
            let piece = self.units[u].guys[g].gpiece;
            let track = self.art.tracks.get(&piece).copied();
            let follow = track.map(|track| {
                let des = crate::movement::follower_des(pos, facing, track, bound);
                Follow {
                    body: crate::movement::Body::at(des),
                    des,
                    facing,
                    des_angle: facing,
                    track,
                }
            });
            self.units[u].guys[g].follow = follow;
        }
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
            follow: None,
        }];
        s.add_unit(u)
    }

    /// **A piece the install describes has its whole slot list**, and a
    /// slot it leaves out is the packet's own — three frames from
    /// `AnimationPacket::get_game_frames`, and a variant the idle roll has
    /// to fall back from. That is not the same question as "does anything
    /// know a length", and reading it as one is what made the roll's
    /// fallback swallow the human scout's `CHAR_IDLE1` (`docs/ANIM.md`
    /// §3.2).
    #[test]
    fn a_known_piece_s_missing_slot_is_three_frames_and_not_a_variant() {
        let mut s = sim_at(1);
        let u = animal(&mut s, 0, 0, 7, DEFAULT, 0, 61);
        // Nothing knows the piece: every slot is unknown, and an unknown
        // length reads as a slot the packet lacks — which is what this
        // crate did everywhere before the install was read.
        assert_eq!(s.slot_length(u, 7, IDLE1), None);
        assert!(!s.packet_has(u, 7, IDLE1));
        // The install names it: `CHAR_DEFAULT` and `CHAR_IDLE1` only.
        s.art
            .piece_lengths
            .insert(7, [(DEFAULT, 61u32), (IDLE1, 76)].into_iter().collect());
        assert_eq!(s.slot_length(u, 7, IDLE1), Some(76));
        assert!(s.packet_has(u, 7, IDLE1));
        assert_eq!(s.slot_length(u, 7, IDLE2), Some(MISSING));
        assert!(!s.packet_has(u, 7, IDLE2));
        // And it wins over a dump row for the same pair.
        s.art.lengths.insert((7, IDLE1), 61);
        assert_eq!(s.slot_length(u, 7, IDLE1), Some(76));
    }

    /// **The scout's fifteen frames.** Run33's human scout wraps its idle
    /// on frame 284 and the roll takes `CHAR_IDLE1`; the original's clock
    /// then runs 76 frames and this crate's ran 61, because the variant
    /// fell back to `CHAR_DEFAULT` for want of a length. The draw is the
    /// same either way — what changed is the slot it lands on and the
    /// frame the clock next wraps (`docs/ANIM.md` §3.2).
    #[test]
    fn the_scout_s_idle1_runs_seventy_six_frames_not_the_default_s_sixty_one() {
        // A word whose `% 100` is `IDLE1`'s band.
        let seed = (1u32..)
            .take(10_000)
            .find(|&sd| (70..=82).contains(&(Rng::new(sd).roll() % 100)))
            .expect("a seed in the band");
        for (known, wrap) in [(false, 345u32), (true, 360)] {
            let mut s = sim_at(seed);
            let u = animal(&mut s, 0, 0, 371, DEFAULT, 61, 61);
            s.art.lengths.insert((371, DEFAULT), 61);
            if known {
                s.art.piece_lengths.insert(
                    371,
                    [(DEFAULT, 61u32), (IDLE1, 76), (IDLE2, 41), (IDLE3, 190)]
                        .into_iter()
                        .collect(),
                );
            }
            s.guy_set_anim(u, 0, DEFAULT, false, true);
            let g = s.units[u].guys[0];
            assert_eq!(g.anim, if known { IDLE1 } else { DEFAULT });
            assert_eq!(g.end_time, if known { 76 } else { 61 });
            assert_eq!(284 + g.end_time, wrap, "the frame the clock next wraps");
        }
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
        s.guys_inc_time();
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

    /// **The standing body still owed a turn walks in place** —
    /// `Guy::move@005d9240:73–89`, the arm the arrival's *second* draw
    /// comes from (`docs/SYNC.md` §3.11, "the pair").
    ///
    /// The body is on its unit, so the moving half never runs; but
    /// `des_angle != angle`, so instead of idling it is put back on
    /// `CHAR_WALK` and marked unstopped. The frame after, `do_idle` sees
    /// the walk category and a stopped body again and rolls a second
    /// time — which is why a farm animal arriving spends two
    /// `Animal::do_idle` draws on consecutive frames, run39's 19 and 20.
    ///
    /// The arm's only guard is a **sea** unit or a `SPECIAL_ANIM` order,
    /// and the walk it asks for is the plain one: the carrying walk is
    /// chosen in the moving half, which this is not.
    ///
    /// Made to fail by returning early when the facing has not settled —
    /// the body then idles on the arrival frame and draws once, not twice.
    #[test]
    fn a_standing_body_still_owed_a_turn_walks_in_place_and_draws_again() {
        let mut s = sim_at(7);
        s.art.lengths.insert((60063, WALK), 20);
        s.art.lengths.insert((60063, DEFAULT), 90);
        let u = animal(&mut s, 8, 0, 60063, WALK, 11, 20);
        // On its unit — the body has arrived — but a turn is still owed.
        s.units[u].movement.body.pos = s.units[u].pos;
        s.units[u].movement.facing = crate::movement::Angle::NORTH;
        s.units[u].movement.heading = crate::movement::Angle::EAST;
        s.guys_follow(u, true);
        assert_eq!(s.units[u].guys[0].anim, WALK, "back on the walk");
        assert!(!s.units[u].guys[0].stopped, "and unstopped");

        // With the facing settled it is an arrival instead: one draw, and
        // the body stops.
        let mut s = sim_at(7);
        s.art.lengths.insert((60063, WALK), 20);
        s.art.lengths.insert((60063, DEFAULT), 90);
        let u = animal(&mut s, 8, 0, 60063, WALK, 11, 20);
        s.units[u].movement.body.pos = s.units[u].pos;
        let before = s.rng.seed;
        s.guys_follow(u, true);
        assert_eq!(s.rng.seed, stepped(before, 1), "the arrival's own draw");
        assert_eq!(s.units[u].guys[0].anim, DEFAULT);
        assert!(s.units[u].guys[0].stopped);

        // A ship owed a turn takes the guard: stopped, and no walk.
        let mut s = sim_at(7);
        s.art.lengths.insert((60063, WALK), 20);
        let u = animal(&mut s, 8, 0, 60063, DEFAULT, 11, 90);
        s.units[u].kind.domain = crate::attrition::Domain::Sea;
        s.units[u].movement.body.pos = s.units[u].pos;
        s.units[u].movement.facing = crate::movement::Angle::NORTH;
        s.units[u].movement.heading = crate::movement::Angle::EAST;
        s.units[u].guys[0].stopped = false;
        s.guys_follow(u, true);
        assert_eq!(s.units[u].guys[0].anim, DEFAULT, "a ship keeps its slot");
        assert!(s.units[u].guys[0].stopped);
    }

    /// The step: one a frame, `last_time` trailing; a work animation that
    /// runs out restarts silently, keeping the overrun.
    #[test]
    fn the_step_and_a_silent_restart() {
        let mut s = sim_at(7);
        s.art.lengths.insert((6688, SOW), 47);
        let u = animal(&mut s, 0, 3, 6688, SOW, 45, 47);
        let before = s.rng.seed;
        s.guys_inc_time();
        assert_eq!(
            (s.units[u].guys[0].cur_time, s.units[u].guys[0].last_time),
            (46, 45)
        );
        s.guys_inc_time();
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
            follow: None,
        });
        s.frame = 101;
        s.guys_inc_time();
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

    /// **`Animal::do_idle`'s own collision test.** A wander destination
    /// that is occupied is not ordered at all — `is_valid`, then
    /// `detect_unit_collision(dest, quick 1, …)`. The four draws are spent
    /// either way; only the order differs. This is what keeps a herd of
    /// sheep standing shoulder to shoulder still, and without it run14's
    /// `8/1` walks off on frame 108 where the original's has not moved in
    /// 120 frames.
    #[test]
    fn a_wander_onto_an_occupied_cell_is_not_ordered() {
        use crate::world::{Cell, Terrain};
        // Both sims are the same game to the draw; only the second has a
        // unit standing where the wander wants to go.
        let build = |seed: u32, blocker: Option<Pos>| -> (Sim, usize, u32) {
            let mut world = World::new(40, 40);
            world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
            let mut s = Sim::new(Tuning::RON, world, 2);
            s.rng = Rng::new(seed);
            let ty = s.add_unit_type(crate::UnitType {
                hits: 10,
                moves: 11,
                combat: crate::combat::Profile {
                    block_radius: 48,
                    big_radius: 48,
                    uber_size: 1,
                    ..crate::combat::Profile::default()
                },
                ..crate::UnitType::default()
            });
            s.gaia.herds.push(crate::gaia::Herd {
                cx: 10,
                cy: 10,
                wx: 10,
                wy: 10,
                kind: 408,
                alive: true,
            });
            for slot in 0..4 {
                s.art.lengths.insert((60063, slot), 90);
            }
            let at = herd_centre(10, 10, 10, 10);
            let mut u = Unit::new(8, 1, at, 10);
            u.ty = Some(ty);
            u.herd = Some(0);
            u.guys = vec![Guy {
                cur_time: 89,
                end_time: 90,
                last_time: -1,
                anim: DEFAULT,
                gpiece: 60063,
                stopped: true,
                follow: None,
            }];
            let a = s.add_unit(u);
            if let Some(p) = blocker {
                let mut b = Unit::new(8, 2, p, 10);
                b.ty = Some(ty);
                s.add_unit(b);
            }
            let before = s.rng.seed;
            s.animal_idle(a);
            (s, a, before)
        };

        // The first word that carries the roll through — three in ten,
        // and then a destination the world accepts.
        let (seed, open, a, before) = (1u32..200)
            .find_map(|n| {
                let (s, a, before) = build(n, None);
                (!s.units[a].orders.is_empty()).then_some((n, s, a, before))
            })
            .expect("a seed whose wander lands");
        let dest = open.units[a]
            .orders
            .front()
            .and_then(crate::orders::Order::move_dest)
            .expect("the open ground gets a wander order");
        let spent = open.rng.seed;
        assert_ne!(spent, before, "the wander's own draws");

        let (blocked, b, _) = build(seed, Some(dest));
        assert!(
            blocked.units[b].orders.is_empty(),
            "the same wander onto an occupied cell is refused"
        );
        assert_eq!(
            blocked.rng.seed, spent,
            "and the refusal costs no draw — the gate is after all four"
        );
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
