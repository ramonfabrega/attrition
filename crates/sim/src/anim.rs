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

/// `UnitTypeData +0x304 squad_size` — the figures of a unit that keep their
/// own clock, the rest of the stack mirroring guy 0 (§5).
///
/// It is a **constant of the executable, not a column**:
/// `UnitType::init@0061ab50:723` writes the literal 1 into every type before
/// it reads `UBER_SIZE` and `CREW_SIZE` beside it, and no other writer
/// exists. Every `UNITDATA` block of every capture prints `guy_mark 1`,
/// which is `Unit::init`'s copy of it.
pub const SQUAD_SIZE: usize = 1;

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

/// The **turning stand** — `Guy::set_anim+0x97a` under `Guy::do_turn+0x4a`,
/// which is `do_turn`'s own `set_anim(CHAR_TURN_LEFT/RIGHT, 0, 1)` falling
/// back to the idle because the piece has no turn animation to play
/// (`docs/ANIM.md` §4.8). Three chains, because `do_turn` is reached with
/// its override enabled from three call sites: `Unit::move_step`'s two
/// turn-in-place arms and `Guy::turn_towards`, which `Guy::move`'s
/// standing arm calls — for guy 0 and, in its own right, for a **tracked
/// crew figure** standing on its offset owed a turn ([`Sim::process_follower`]).
pub const SITE_TURN_NEAR: &str = "Guy::set_anim+0x97a < Guy::do_turn+0x4a < Unit::move_step+0x3b6";
pub const SITE_TURN_FAR: &str = "Guy::set_anim+0x97a < Guy::do_turn+0x4a < Unit::move_step+0x389";
pub const SITE_TURN_STAND: &str =
    "Guy::set_anim+0x97a < Guy::do_turn+0x4a < Guy::turn_towards+0x69";

/// `Guy::set_anim+0x97a` under `Unit::do_cast+0xc89` — the casting unit's
/// `set_anim(CHAR_DEFAULT, 0, 1)` on the first frame of a cast, **one draw
/// a figure** (`docs/TRANSPORT.md` §6).
pub const SITE_CAST: &str = "Guy::set_anim+0x97a < do_cast";

/// `Guy::set_anim+0x97a` under `Unit::do_trade+0x40` — the caravan's own
/// `set_anim(CHAR_DEFAULT, 0, 1)`, the **first instruction of the trade
/// order's step**, ahead of every one of its returns. It is silent while
/// the unit is walking (a walk category whose body has not arrived returns
/// without a roll), so the site only appears on the frames the caravan is
/// standing at one of its two cities — which is what makes it the arrival's
/// own mark (`docs/CARAVAN.md` §7.1).
pub const SITE_TRADE: &str = "Guy::set_anim+0x97a < do_trade";

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
///
/// **The two identities are different sets, and that is not a slip.**
/// `set_anim:221`'s early return names `0x192` alone; the walk arm's coin
/// at `set_anim:620` names `0x192`, `0x193` and `0x194`. So a gull asked
/// to walk while already walking does not take the `0x192` exemption —
/// but it never reaches that test either, because `set_anim:141`'s
/// gaia-walker return (`who >= 8`, `CHAR_WALK`, inside its length) catches
/// it first. Only a **wrap** re-throws its coin.
pub const BIRD_TYPE: i32 = crate::gaia::BIRD_TYPE_INDEX;

/// The last of the three gaia bird types — `GULLBIRD`, which a finished
/// dock spawns (`docs/TRANSPORT.md` §5.2). `FLOCKBIRD` (`0x193`) sits
/// between them and no capture has one.
pub const GULL_TYPE: i32 = crate::transport::ty::GULLBIRD as i32;

/// The three types `Guy::set_anim`'s walk arm throws the wing-beat coin
/// for (`set_anim:620`), which is also the set that flies: a bird is
/// carried by `Unit::do_air_physics` rather than by a ground body, so
/// `Guy::move`'s follow never runs for one.
pub const fn is_air_gaia(type_index: i32) -> bool {
    BIRD_TYPE <= type_index && type_index <= GULL_TYPE
}

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
/// slot, from the names: **`CHAR_ATTACKWALK`**, the attacks, deaths,
/// turns, pack/unpack and the two dumps are non-looping, every walk, idle
/// and work animation loops.
/// For a category-0 animation the flag changes nothing — both restarts
/// go through the idle roll (§4) — so it matters only for the dumps and
/// the attacks (§6).
///
/// **Slot 10 is in the range deliberately, and the install says so**
/// (2026-09-05, `docs/ANIM.md` §5). The range's lower bound reads like it
/// was chosen to start at the attacks and picked slot 10 up by
/// arithmetic; `anim_graphics.xml` settles it. Thirty-eight distinct
/// animations are named by a `CHAR_ATTACKWALK` row of the install's 795,
/// and **thirty-six sit under `<NONLOOPING>`** — every `… AttackWalk` and
/// `… AttackRun` file among them. The two that do not are
/// `Carbineer Idle1` and `TrebuchetWorkhorse Walk`, two pieces that point
/// slot 10 at art they already use elsewhere, and for those the slot rule
/// is wrong. That is §9's "should be a per-piece table beside
/// [`Art::piece_lengths`] rather than a rule", with a number on it:
/// `rondata::diff`'s `char_attackwalk_is_non_looping_but_for_two_pieces`
/// pins the pair so the day a per-file table lands it has its answer.
///
/// **This is only the third of `Guy::inc_time`'s three tests.** The
/// original reads `loopings[packet->ids[slot]]` behind `slot <
/// packet->count` and `ids[slot] >= 0`, and those two are
/// [`Sim::packet_has`]: a slot the piece's packet does not name has no
/// animation id and is non-looping whatever this rule says. The caller
/// asks both (§3.6).
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

/// `GraphicPieces::init_piece_ranges@008f70e0`'s four nested strides —
/// literals in the executable, and the coordinates
/// `GraphicPieces::get_unit_gpiece` sums (`docs/ANIM.md` §3.2, §3.4).
///
/// `first_unit_piece` is zero, `num_unit_pieces` is one art style's worth
/// of unit records, and `total_num_unit_pieces` is `0xc606` — four crews
/// of [`PIECES_PER_CREW`] plus the six "over time" merchant pieces.
pub const PIECES_PER_STYLE: i32 = 0x160;
/// Six art styles: one step of the **age bracket** coordinate.
pub const PIECES_PER_AGE: i32 = 0x840;
/// Three age brackets: one step of the **gender** coordinate, which is
/// the same slot the `-PACKED` art sits in.
pub const PIECES_PER_GENDER: i32 = 0x18c0;
/// Two genders: one step of the **crew** coordinate, `guy_num`.
pub const PIECES_PER_CREW: i32 = 0x3180;
/// `GraphicPieces::first_unit_piece` — zero, and the answer
/// `get_unit_gpiece` falls back to when its four walks find nothing.
pub const FIRST_UNIT_PIECE: i32 = 0;

impl Sim {
    /// The two coordinates a **leader** contributes to
    /// `get_unit_gpiece`'s sum: the nation's art style
    /// (`Tribe::unit_continent`) and the age bracket
    /// `age < 5 ? age / 3 : 2` off `LeaderDataEncrypt::ages` (`+0xdc`,
    /// which the function reads through its `^ 0x62766`).
    ///
    /// A `who` this simulation keeps no leader for is the function's own
    /// `param_2 == −1`: style 0 and bracket 0, and no gender walk.
    fn leader_art(&self, who: Player) -> (i32, i32) {
        let Some(p) = self.tech.get(who as usize) else {
            return (0, 0);
        };
        let style = self
            .tech_tree
            .tribes
            .get(p.tribe)
            .map_or(0, |t| t.unit_continent);
        let age = p.ages;
        (style, if age < 5 { age / 3 } else { 2 })
    }

    /// `GraphicPieces::get_unit_gpiece@0090c030`'s **unit** arm: the piece
    /// a guy of `ty` owned by `who` plays, derived rather than looked up
    /// (`docs/ANIM.md` §3.4).
    ///
    /// The sum is `(TypeIndex − 0x32)` plus the four strides above times
    /// the four coordinates — style, age bracket, gender and crew — and
    /// the function then *walks down* from it: for each of four
    /// combinations, the age bracket down to 0, taking the first piece the
    /// art actually has. The four are (style, gender), (no style, gender),
    /// (style, no gender) and (no style, no gender), and the first two are
    /// skipped unless the gender coordinate applies at all. Nothing found
    /// is [`FIRST_UNIT_PIECE`].
    ///
    /// The gender coordinate is the `-FEMALE` art, and the `-PACKED` art
    /// is the **same slot**: `packed` takes it whatever the object number
    /// says, and a type that packs (`unit_flags2 & 4`) is refused it
    /// otherwise — which is why a fishing boat's `CHAR_UNPACK` is on one
    /// piece and its `CHAR_PACK` on the other.
    ///
    /// The existence test is [`Art::piece_lengths`], the install's own
    /// `<UNIT>` entries: the original asks `data_pieces[piece] != 0` after
    /// a `verify_load`, and a piece the graphics file names at all is one
    /// that loads.
    ///
    /// SEAM: the merchant family's "over time" pieces — `TypeIndex`
    /// `0x3d`, `0x3e` and `0x190` reach `total_num_unit_pieces − 6 … − 1`
    /// by the nation's `build_continent` before any of this, and those six
    /// are the `-NEUROPE-`/`-KOREAN-`/`-IROQUOIS-`/`-COLONIAL-`/
    /// `-EINDIAN-` entries whose names the piece arithmetic cannot build.
    /// No capture holds a merchant.
    pub fn unit_gpiece(
        &self,
        who: Player,
        ty: usize,
        o: i16,
        guy_num: u8,
        packed: bool,
    ) -> Option<i32> {
        let t = self.unit_types.get(ty)?.type_index;
        if t < 0x32 {
            return None;
        }
        let (style, bracket) = self.leader_art(who);
        let base = t - 0x32 + FIRST_UNIT_PIECE + PIECES_PER_CREW * i32::from(guy_num);
        // `LAB_0090c2ff`: the gender bit is the object number's low bit,
        // and a type the packet exempts (`unit_flags2 & 4`) or an object
        // number of −1 never takes it. `packed` overrides all three.
        let gender = packed || (!self.unit_types[ty].combat.packs && o >= 0 && o & 1 == 1);
        let walk = |style: i32, gender: bool| {
            (0..=bracket)
                .rev()
                .map(|age| {
                    base + PIECES_PER_AGE * age
                        + PIECES_PER_STYLE * style
                        + if gender { PIECES_PER_GENDER } else { 0 }
                })
                .find(|p| self.art.piece_lengths.contains_key(p))
        };
        let found = gender
            .then(|| walk(style, true).or_else(|| walk(0, true)))
            .flatten()
            .or_else(|| walk(style, false))
            .or_else(|| walk(0, false));
        Some(found.unwrap_or(FIRST_UNIT_PIECE))
    }

    /// The piece a guy of `ty` owned by `who` plays.
    ///
    /// A gaia type's comes off `first_bird_piece`, a runtime pointer no
    /// file states, so it stays [`Art::pieces`]' — the start dump's own
    /// answer, keyed by the variant `(seed + o) % 3`. A player's unit is
    /// **derived** by [`Sim::unit_gpiece`] wherever the install's piece
    /// table was read, and only falls back to the dump's where it was not
    /// (a fixture with no install behind it). That is the difference
    /// between a unit the opening dump happened to hold and one trained
    /// mid-game: the table knows only the first.
    pub fn piece_of(
        &self,
        who: Player,
        ty: usize,
        o: i16,
        guy_num: u8,
        packed: bool,
    ) -> Option<i32> {
        if self.art.gaia_types.contains(&ty) {
            let sub = (self.game_seed.wrapping_add(i32::from(o))).rem_euclid(3) as u8;
            return self.art.pieces.get(&(who, ty, sub, guy_num)).copied();
        }
        if !self.art.piece_lengths.is_empty() {
            return self.unit_gpiece(who, ty, o, guy_num, packed);
        }
        self.art
            .pieces
            .get(&(who, ty, (o & 1) as u8, guy_num))
            .copied()
    }

    /// `Unit::update_gpiece@005e2920` — every guy's piece recomputed from
    /// the unit's *current* state, which is what a pack or an unpack is
    /// for: `SpellType::cast_unpack` clears `unit_masks & 0x80000` and
    /// then calls this, and the `-PACKED` art is swapped for the plain
    /// piece (`docs/ORDERS.md` §6.9).
    ///
    /// The clock is not touched. `Guy::update_gpiece` writes `gpiece` and
    /// nothing else about the animation, so a guy mid-`CHAR_UNPACK` keeps
    /// its `cur_time` and its `end_time` — the length it was given when
    /// the animation was set — and the next `set_anim` picks up the new
    /// piece's lengths.
    pub fn update_gpiece(&mut self, u: usize) {
        let (who, o, ty) = {
            let unit = &self.units[u];
            (unit.owner, unit.index, unit.ty)
        };
        let packed = self.units[u].combat.packed;
        let Some(ty) = ty else { return };
        for n in 0..self.units[u].guys.len() {
            let piece = self.piece_of(who, ty, o, n as u8, packed).unwrap_or(-1);
            self.units[u].guys[n].gpiece = piece;
        }
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
    /// `Guy::init_real`'s one draw (`% 100`, [`init_variant`]).
    ///
    /// The count is `crew_size + squad_size` (SQUAD_SIZE, §3.5):
    /// `Unit::init@00612100:471`-`508` grows the stack to that sum, fills
    /// every slot from a `Recycler<Guy>::pop`, then walks `0..len` giving
    /// each one `Guy::init_real`. `squad_size` is the literal 1 for every
    /// type, so the count is the `CREW_SIZE` column plus one: 1 for a
    /// Citizen, 3 for a Caravan, 4 for a Trebuchet.
    pub fn init_guys(&mut self, u: usize, ty: Option<usize>) {
        let unit = &self.units[u];
        let (who, o) = (unit.owner, unit.index);
        // `Unit::init` sets `unit_masks |= 0x80000` for a type that packs
        // at `:376` and only makes its guys at `:540`, so a boat born
        // packed asks for the `-PACKED` art on its very first frame.
        let packed = unit.combat.packed;
        let ty = ty.or(unit.ty);
        let count = ty.map_or(SQUAD_SIZE, |t| {
            usize::try_from(self.unit_types[t].combat.crew_size + SQUAD_SIZE as i32).unwrap_or(0)
        });
        let mut guys = Vec::with_capacity(count);
        for n in 0..count {
            let piece = ty
                .and_then(|t| self.piece_of(who, t, o, n as u8, packed))
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
        // `Unit::init@00612100:548`-`549`, which follow the guy loop:
        // `update_gpiece`, then `set_new_location(x, y, 1, 1)`. The
        // `param_3 = 1` is what reaches `Guy::set_new_location(crew, des,
        // 1)` and puts a **tracked** crew figure straight onto its offset
        // instead of leaving it to walk there from the leader's own
        // point. This crate seated only the guys a dump handed it and the
        // ones a transport put ashore, so every unit it *trained* kept a
        // trackless crew — and a trackless crew is one `Guy::do_turn`
        // recurses into (§4.8), so East Indies' Merchant turned twice a
        // frame where the original turns once.
        self.seat_guys(u);
    }

    /// `Unit::set_type@00612fa0`'s guy loops (`6133ac`, `613441`) — the
    /// figures a **converted** unit keeps, and the draw each one pays.
    ///
    /// `set_type` does not rebuild the stack the way `Unit::init` does. It
    /// clamps `guy_mark` to the new type's `squad_size` — written 1 for
    /// every type (§3.5) — kills and recycles every slot past
    /// `crew_size + squad_size`, pops fresh ones for any new slot, and
    /// then walks the whole stack giving each guy the new type and a fresh
    /// `Guy::init_real(guy, 1)`. So a **kept** guy holds its body and its
    /// place and loses only its clock, and every guy costs one
    /// [`SITE_INIT_REAL`] draw whether it is kept or new.
    ///
    /// That reset is what the frame after a conversion shows: `init_real`
    /// leaves `end_time` at zero, so the guy wraps on its very next
    /// `Unit::inc_time` and rolls again (§4.9). Run53's frame 6736 is
    /// three of each — three archer objects re-typed, three `init_real`
    /// and three extra `Guy::inc_time` rolls.
    pub(crate) fn reinit_guys(&mut self, u: usize, ty: usize) {
        let unit = &self.units[u];
        let (who, o) = (unit.owner, unit.index);
        let packed = unit.combat.packed;
        let count =
            usize::try_from(self.unit_types[ty].combat.crew_size + SQUAD_SIZE as i32).unwrap_or(0);
        self.units[u].guys.truncate(count);
        for n in 0..count {
            let piece = self.piece_of(who, ty, o, n as u8, packed).unwrap_or(-1);
            self.mark(SITE_INIT_REAL);
            let p = self.rng.roll() % 100;
            let mut anim = init_variant(p);
            if anim != DEFAULT && piece >= 0 && !self.packet_has(u, piece, anim) {
                anim = DEFAULT;
            }
            match self.units[u].guys.get_mut(n) {
                Some(g) => {
                    g.gpiece = piece;
                    g.anim = anim;
                    g.cur_time = 0;
                    g.end_time = 0;
                    g.last_time = -1;
                }
                None => {
                    let mut g = Guy::fresh(piece);
                    g.anim = anim;
                    self.units[u].guys.push(g);
                }
            }
        }
    }

    /// `guy_flags & 8` — whether `Guy::do_turn` asks this guy for a turn
    /// animation at all.
    ///
    /// `Guy::init_real@005db6b0` has **two** writers of the bit and they
    /// are independent:
    ///
    /// - `:179`, the piece's own packet naming `CHAR_TURN_RIGHT` — slot
    ///   22, `action_ids[0x16]`, and only that one — with a file that
    ///   loads;
    /// - `:215`, the `else` of `(type+0x2b8 & 4) == 0`, which sets the bit
    ///   for **every type that packs** whatever its art says.
    ///
    /// So the bit does not mean "has a turn animation", and the second
    /// writer is what makes it visible in a game with no siege in it: a
    /// fishing boat packs, so it asks for a turn it cannot play and pays
    /// the idle roll instead (`docs/ANIM.md` §4.8). Run26's dump is the
    /// evidence for both halves — `FISHERMEN` and `MERCHANT`, which pack
    /// and name no turn, carry `guy_flags 8`; `PIKEMEN`, which names the
    /// turn and does not pack, carries `24`; `TRIREME`, neither, carries
    /// `0`.
    ///
    /// Derived rather than stored: neither the type nor the piece changes
    /// under a guy, so the answer is the one `init_real` would have
    /// written.
    pub(crate) fn guy_turns(&self, u: usize, g: usize) -> bool {
        let unit = &self.units[u];
        if unit.ty.is_some_and(|t| self.unit_types[t].combat.packs) {
            return true;
        }
        let Some(guy) = unit.guys.get(g).copied() else {
            return false;
        };
        self.packet_has(u, guy.gpiece, TURN_RIGHT)
    }

    /// `Guy::do_turn@005d97a0`'s animation half — the override a caller
    /// asks for with a non-zero fifth argument.
    ///
    /// `to` is the angle the turn lands on this frame and `heading` the
    /// bearing it is turning towards; the guard is `to != angle`, read
    /// **before** the turn is applied, and the direction is the sign of
    /// `heading − angle` as an unsigned wrap: at or below half a turn it
    /// is `CHAR_TURN_RIGHT`, above it `CHAR_TURN_LEFT` (`005d97cd`–
    /// `005d97e5`).
    ///
    /// The set is guy 0 and the trackless crew — `do_turn` recurses into
    /// exactly the guys that share guy 0's body, the same set
    /// [`Sim::guys_follow`] walks.
    pub(crate) fn do_turn_anim(&mut self, u: usize, was: Angle, to: Angle, heading: Angle) {
        for g in 0..self.units[u].guys.len() {
            if self.units[u].guys[g].follow.is_some() {
                continue;
            }
            self.guy_do_turn_anim(u, g, was, to, heading);
        }
    }

    /// The same override for **one** guy, which is what a tracked crew
    /// figure needs: `Guy::do_turn` recurses into the trackless crew only
    /// (`+0x304` upward), but every guy runs its own `Guy::process ->
    /// Guy::move`, and a tracked crew figure standing on its offset with
    /// `des_angle != angle` reaches `Guy::move:109`'s own `turn_towards ->
    /// do_turn(..., 1)` exactly as guy 0 does ([`Sim::process_follower`]).
    /// Its `guy_flags & 8` is the type's, so a merchant's driver asks for a
    /// turn animation the art has not got and pays the idle roll for it.
    pub(crate) fn guy_do_turn_anim(
        &mut self,
        u: usize,
        g: usize,
        was: Angle,
        to: Angle,
        heading: Angle,
    ) {
        if to == was || !self.guy_turns(u, g) {
            return;
        }
        let anim = if heading.0.wrapping_sub(was.0) as u32 <= 0x8000_0000 {
            TURN_RIGHT
        } else {
            TURN_LEFT
        };
        self.guy_set_anim(u, g, anim, false, true);
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

    /// Whether **this guy's** angle has reached the one it is turning to —
    /// `des_angle == angle` (`+0x64` against `+0x18`), the test
    /// `Guy::set_anim`'s turn arm makes and the one [`Sim::guys_follow`]
    /// makes for the standing walk.
    ///
    /// Guy 0's pair is the unit's own — `Movement::facing` against
    /// `Movement::heading` — and a tracked crew figure has its own. An
    /// **untracked** crew figure is never owed a turn: `Guy::do_turn` writes
    /// guy 0's new angle into every carried figure as *both* its angle and
    /// its `des_angle`, so the two are equal on every frame
    /// ([`Sim::guys_follow`]'s `g >= SQUAD_SIZE`).
    fn body_settled(&self, u: usize, g: usize) -> bool {
        let unit = &self.units[u];
        match unit.guys.get(g).and_then(|g| g.follow) {
            Some(f) => f.facing == f.des_angle,
            None => g >= SQUAD_SIZE || unit.movement.facing == unit.movement.heading,
        }
    }

    /// `guy_flags & 0x80` — the scholar bit.
    ///
    /// `Guy::init_real@005db6b0:233` sets it when the unit's `TypeIndex` is
    /// `0x34` or `0x35`, which is exactly [`crate::orders::Worker::Scholar`],
    /// so it is derived here rather than stored — the same choice
    /// [`Sim::guy_turns`] makes, and for the same reason: a type does not
    /// change under a guy. Run25's `GUYS=4` window is the record — its
    /// thirteen scholars carry `guy_flags 144` (`0x80 | 0x10`) and nothing
    /// else in the corpus carries the bit at all.
    fn guy_is_scholar(&self, u: usize) -> bool {
        self.worker_of(u) == crate::orders::Worker::Scholar
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

        // **The head arm, ahead of every early return** (`set_anim:88–103`,
        // `docs/ANIM.md` §4.9). A `CHAR_ATTACKWALK` request on a guy not
        // already on that slot zeroes its unit's `recharging`
        // (`UnitData+0xae`) — or, once the counter has reached six, becomes
        // a plain `CHAR_WALK` instead. Nothing in this crate passes
        // `CHAR_ATTACKWALK` yet (`Guy::move:110`'s mounted arm is
        // unmodelled), so the arm is written for the day something does.
        let anim = if anim == ATTACKWALK {
            if guy.anim == ATTACKWALK {
                anim
            } else if self.units[u].combat.recharging < 6 {
                self.units[u].combat.recharging = 0;
                anim
            } else {
                WALK
            }
        } else {
            anim
        };

        // The early returns (`set_anim:155–224`).
        if anim == DEFAULT && !force {
            if cur_cat == 0 {
                if guy.cur_time < guy.end_time {
                    return;
                }
            } else if cur_cat == 8 {
                if !at_des {
                    // A walking guy asked to idle before its body has
                    // arrived: nothing, or a rewind once the walk cycle has
                    // run out.
                    if guy.cur_time >= guy.end_time {
                        self.units[u].guys[g].cur_time = 0;
                    }
                    return;
                }
            } else if (guy.anim == TURN_LEFT || guy.anim == TURN_RIGHT) && !self.body_settled(u, g)
            {
                // **A guy actually playing a turn has the same protection**
                // (`set_anim:184–202`, `docs/ANIM.md` §4 step 1). It is the
                // walking guy's arm one branch above with "still owed a
                // turn" in place of "still on its way": inside its length it
                // returns, past its length it rewinds `cur_time` to zero —
                // and neither spends a draw. `Guy::move`'s slot exclusions
                // keep such a guy on its turn animation (§4.7); this is what
                // keeps the idle roll off it, so the turn costs no draw at
                // all until the angle settles.
                if guy.cur_time >= guy.end_time {
                    self.units[u].guys[g].cur_time = 0;
                }
                return;
            }
        } else if who >= 8 && anim == WALK && cur_cat == 8 && guy.cur_time < guy.end_time {
            // Gaia's walkers: a walk already playing is left alone.
            return;
        }
        // `set_anim:225–252` — **a turn the packet cannot play becomes the
        // idle.** `Guy::do_turn` asks for `CHAR_TURN_LEFT`/`CHAR_TURN_RIGHT`
        // on the strength of `guy_flags & 8` alone, and that bit is set for
        // a type that *packs* as well as for a piece that names the turn
        // ([`Sim::guy_turns`]), so a fishing boat asks for an animation it
        // does not have. The request is rewritten to `CHAR_DEFAULT` here —
        // past the `param_1 == CHAR_DEFAULT` early returns above, which is
        // why a boat on the walk category then rolls (`docs/ANIM.md` §4.8).
        let anim =
            if (anim == TURN_LEFT || anim == TURN_RIGHT) && !self.packet_has(u, guy.gpiece, anim) {
                DEFAULT
            } else {
                anim
            };
        // **A scholar collapses every slot above `CHAR_UNPACK` into the
        // idle**, on both sides of the test that follows (`set_anim:205–221`,
        // `docs/ANIM.md` §4.11). `guy_flags & 0x80` is
        // `Guy::init_real@005db6b0:233`'s bit — the unit's `TypeIndex` is
        // `0x34` or `0x35` — and it rewrites `UnitAnimCat[cur_anim]` for a
        // guy on slot 25 or above, and `UnitAnimCat[requested]` for a
        // request of slot 25 or above when the call does not force. The
        // remapped pair is what the same-category return and the idle
        // roll's variant gate read; the **apply** reads the raw categories,
        // which is why only these two are remapped here.
        let scholar = self.guy_is_scholar(u);
        let cur_cat_test = if scholar && guy.anim > UNPACK {
            DEFAULT
        } else {
            cur_cat
        };
        let target_cat = if scholar && anim > UNPACK && !force {
            DEFAULT
        } else {
            category(anim)
        };
        // `set_anim:219` — the same category, already inside its length,
        // is left alone. The walk category is the exception, because a
        // walk re-resolves its slot from the body's speed every time it is
        // asked — **unless the type is `BIRD`**, whose slot is a coin and
        // is not re-thrown while the wing beat is still running.
        if !force
            && cur_cat_test == target_cat
            && (cur_cat_test != 8 || self.units[u].type_index == BIRD_TYPE)
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
                if cur_cat_test == 0 && p3 {
                    let peasant_on_masked = self.is_peasant(u) && self.on_masked_tile(u);
                    v = idle_variant(p, self.units[u].guy_flag_0x20, peasant_on_masked);
                }
            }
            // The variant fallback. `set_anim`'s `LAB_005daba0` is one
            // disjunction with **six** terms and this is the first three of
            // them — the packet is null, the slot is past `action_ids`'
            // count, or its id is negative. The fourth and fifth are the
            // animation file failing to load, which no data this crate has
            // can answer; the sixth is not about the packet at all.
            //
            // SEAM: `unit_masks & 0x2000000` also forces `CHAR_DEFAULT`
            // (`set_anim:553`, `docs/ANIM.md` §4 step 2). `unit_masks` is
            // not a field of this crate, and the bit is set on **no unit of
            // any capture on disk** — 191 distinct values across the whole
            // corpus, whose bitwise or is `0x58dd17ce`, guarded by
            // `rondata::diff`'s `no_capture_carries_the_no_animations_bit`.
            if v != DEFAULT && guy.gpiece >= 0 && !self.packet_has(u, guy.gpiece, v) {
                v = DEFAULT;
            }
            v
        } else if target_cat == 12 {
            // An attack: `ATTACK1` / `ATTACK2` / `ATTACK3` by one draw when
            // asked with `p3` (30 / 40 / 30 percent); §6.
            let a = if p3 {
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
            };
            // **And the attack arm falls back to its own category's slot**,
            // `set_anim:588–591` — the same rule the idle's `:546` and the
            // walk's `:596` have, against **this guy's own** packet, and it
            // applies whether or not the roll was thrown. A piece that names
            // `CHAR_ATTACK2` and neither of its neighbours plays `ATTACK2`
            // for all three (`docs/ANIM.md` §4 step 3).
            if guy.gpiece >= 0 && !self.packet_has(u, guy.gpiece, a) {
                ATTACK2
            } else {
                a
            }
        } else if target_cat == 8 {
            self.walk_variant(u, g)
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
        } else if target_cat == 8 {
            // **The walk category's same-slot arm is not the others'.**
            // `set_anim:667` is `if (cur_time < len) goto <past the
            // write>` — a walk still inside its animation keeps the clock
            // it has, and only one that has run past takes the length off.
            // Everything else takes `cur − min(cur, len)`, which is the
            // same thing for an overrun and **zero** for a clock still
            // running; applying that to a walk froze every walking guy at
            // `cur_time == 1`, because `Guy::move` asks for the walk again
            // on every frame and `Guy::inc_time` stepped it straight back.
            // No guy walking for longer than its cycle could then wrap,
            // and a wrap is a draw: run54's frame 6169 is two of them,
            // the caravan's crew figures (`docs/CARAVAN.md` §4.1).
            if guy.cur_time >= end_at_entry {
                guy.cur_time -= end_at_entry;
            }
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
    fn walk_variant(&mut self, u: usize, g: usize) -> i8 {
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
            // **The asked guy's own average**, `this->field_0x84` at
            // `005db438` — not guy 0's. A tracked crew figure keeps its
            // own `avg_speed`, and `Guy::move`'s tracked branch pays it
            // `(get_speed · 11) / 8`, so a figure that keeps station
            // behind a leader travelling at its base speed averages
            // eleven eighths of it and **jogs where its leader walks**.
            // run67's merchant crew is `cur_anim 9` on every frame of the
            // window against its driver's 8, and the slot is what
            // `Guy::move`'s arrival arm tests for (`== CHAR_WALK`, the
            // slot and not the category), so reading guy 0's cost a draw
            // on every arrival a crew figure made.
            let avg = self.units[u].guys[g]
                .follow
                .map_or(unit.movement.body.avg_speed, |f| f.body.avg_speed);
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
        // `CHAR_WALK`, the category's own. **It is the asked guy's own
        // packet**, not guy 0's: a crew figure whose `<UNIT>` entry names
        // no animation at all has neither `CHAR_SLOG` nor `CHAR_JOG`, so a
        // caravan whose driver slogs has two crew figures walking beside
        // it (run64's frames 6168 and 6170, `docs/ANIM.md` §3.6). Reading
        // guy 0's was invisible while every crew figure shared its piece.
        let guy = self.units[u].guys.get(g).copied();
        if let Some(guy) = guy
            && guy.gpiece >= 0
            && v != WALK
            && !self.packet_has(u, guy.gpiece, v)
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
            // The site fold's unit attribution (`rondata::diff`'s
            // `attributed_sites`) is the unit loop's mark, and this phase
            // runs past it — so a wrap's draw was nobody's until the mark
            // was made here too. It costs nothing: the mark is taken
            // before any draw of this unit's, so the label carries zero
            // draws of its own.
            self.mark(&format!(
                "unit {}/{}",
                self.units[u].owner, self.units[u].index
            ));
            let unit = &self.units[u];
            // `inside_up >= 0` is one field, and it points at a boat as
            // readily as at a building (`docs/TRANSPORT.md` §6): a
            // passenger's clock stops the same way a garrison's does.
            let inside_and_not_scholar = (unit.inside.is_some() || unit.inside_unit.is_some())
                && self.worker_of(u) != crate::orders::Worker::Scholar;
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
        if g >= SQUAD_SIZE && category(guy.anim) != 8 {
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
            // `Guy::inc_time`'s own test is `slot < packet->count &&
            // packet->ids[slot] >= 0 && loopings[id]`: a slot the packet
            // does not name has **no animation id**, so the flag is false
            // whatever the slot's name would say. That is what makes a
            // crew figure's walk fall to the idle and roll (§3.6); for a
            // category-0 slot both arms make the same call, which is why
            // it was invisible until a piece with an empty packet walked.
            if self.packet_has(u, guy.gpiece, guy.anim) && !non_looping(guy.anim) {
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
        // crate parks the bird on its hatch cell — and the dock's gull on
        // the tile it was born on (`orders.rs`) — so without this the
        // standing body would ask it to idle every frame and spend a draw
        // the original never spends.
        if is_air_gaia(self.units[u].type_index) {
            return;
        }
        let unit = &self.units[u];
        let settled = unit.movement.facing == unit.movement.heading;
        for g in 0..self.units[u].guys.len() {
            if self.units[u].guys[g].follow.is_some() {
                continue;
            }
            // **A carried crew figure is never owed a turn.**
            // `Guy::do_turn@005d97a0` writes the frame's new angle into
            // guy 0's `angle`, calls `Guy::set_angle` with it — which
            // hands *that* angle to every crew figure as its `des_angle`
            // — and then puts guy 0's own `des_angle` back from the local
            // it saved. Only guy 0 keeps the heading; an untracked crew
            // figure is recursed into with the same pair and ends every
            // turn with `angle == des_angle`. So it takes `Guy::move`'s
            // standing arm while guy 0 takes the turning one, which is
            // the frame a caravan starts moving: the driver is put back
            // on the walk and the crew is left alone to be mirrored
            // (run64's frame 6167, `docs/ANIM.md` §4.7).
            let settled = settled || g >= SQUAD_SIZE;
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
            if self.world.accepts(dest) && !self.detect_quick(u, dest, false) {
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

    /// **A guy actually playing a turn is not idled out of it** — the
    /// third early return of `Guy::set_anim` (`:184–202`,
    /// `docs/ANIM.md` §4 step 1, item 234 / audit row R4).
    ///
    /// The shape is the walking guy's arm one branch above it with "still
    /// owed a turn" in place of "still on its way": while `des_angle !=
    /// angle` an idle request returns, and once the turn animation has run
    /// past its length it rewinds `cur_time` to zero — neither spending a
    /// draw. The moment the angle settles the same request rolls.
    ///
    /// Made to fail by deleting the arm: the first `set_default_anim` then
    /// spends a draw and puts the guy on `CHAR_DEFAULT`, which is the
    /// `Guy::set_anim+0x97a` the original does not spend.
    #[test]
    fn a_turning_guy_is_not_idled_out_of_its_turn_animation() {
        let mut s = sim_at(7);
        s.art.lengths.insert((436, TURN_LEFT), 15);
        s.art.lengths.insert((436, DEFAULT), 32);
        let u = animal(&mut s, 0, 12, 436, TURN_LEFT, 4, 15);
        s.units[u].movement.body.pos = s.units[u].pos;
        s.units[u].movement.facing = crate::movement::Angle::NORTH;
        s.units[u].movement.heading = crate::movement::Angle::EAST;

        // Inside its length and still owed the turn: nothing at all.
        let before = s.rng.seed;
        s.set_default_anim(u);
        assert_eq!(s.rng.seed, before, "no draw while the turn is owed");
        let g = s.units[u].guys[0];
        assert_eq!((g.anim, g.cur_time), (TURN_LEFT, 4), "and the clock stands");

        // Past its length, still owed: the no-draw rewind.
        s.units[u].guys[0].cur_time = 15;
        s.set_default_anim(u);
        assert_eq!(s.rng.seed, before, "the rewind spends nothing either");
        let g = s.units[u].guys[0];
        assert_eq!((g.anim, g.cur_time), (TURN_LEFT, 0), "rewound to zero");

        // The angle settles and the same request rolls.
        s.units[u].movement.heading = crate::movement::Angle::NORTH;
        s.set_default_anim(u);
        assert_eq!(s.rng.seed, stepped(before, 1), "one draw once settled");
        let g = s.units[u].guys[0];
        assert_eq!((g.anim, g.cur_time, g.end_time), (DEFAULT, 0, 32));
    }

    /// **And the wrap cannot take it off either.** `Guy::inc_time`'s
    /// non-looping arm calls `set_anim(CHAR_DEFAULT, 0, 1)`, which is the
    /// same request — so a turn animation that runs out while the angle is
    /// unsettled cycles its clock and spends no wrap draw. run44's
    /// catapults hold 27 and 19 consecutive frames of one without ever
    /// reaching `end_time`; this is the frame past that.
    #[test]
    fn a_turn_animation_that_runs_out_while_owed_a_turn_costs_no_wrap_draw() {
        let mut s = sim_at(7);
        s.art.lengths.insert((6903, TURN_LEFT), 30);
        s.art.lengths.insert((6903, DEFAULT), 50);
        let u = animal(&mut s, 0, 15, 6903, TURN_LEFT, 29, 30);
        s.units[u].movement.body.pos = s.units[u].pos;
        s.units[u].movement.facing = crate::movement::Angle::NORTH;
        s.units[u].movement.heading = crate::movement::Angle::EAST;
        let before = s.rng.seed;
        s.guys_inc_time();
        assert_eq!(s.rng.seed, before, "the wrap's own `set_anim` returns");
        let g = s.units[u].guys[0];
        assert_eq!((g.anim, g.cur_time), (TURN_LEFT, 0), "rewound, not rolled");
    }

    /// **The attack roll falls back to its own category's slot** —
    /// `set_anim:588–591` (`docs/ANIM.md` §4 step 3, audit row R5).
    ///
    /// A piece whose packet names `CHAR_ATTACK2` and neither of its
    /// neighbours plays `ATTACK2` whatever the roll says, rather than the
    /// missing slot and its three frames. The draw is spent either way.
    #[test]
    fn an_attack_the_packet_lacks_falls_back_to_attack2() {
        let mut s = sim_at(7);
        // A piece the install describes: only `CHAR_ATTACK2` is named, so
        // `ATTACK1` and `ATTACK3` are the packet's own missing slots.
        s.art
            .piece_lengths
            .insert(436, [(ATTACK2, 57i32 as u32)].into_iter().collect());
        let u = animal(&mut s, 0, 12, 436, DEFAULT, 0, 32);
        let before = s.rng.seed;
        s.set_anim(u, ATTACK2, false, true);
        assert_eq!(s.rng.seed, stepped(before, 1), "the roll is spent");
        let g = s.units[u].guys[0];
        assert_eq!((g.anim, g.end_time), (ATTACK2, 57), "whatever it rolled");

        // And a piece that has all three keeps what the roll gave it.
        let mut s = sim_at(7);
        s.art.piece_lengths.insert(
            436,
            [(ATTACK1, 40u32), (ATTACK2, 57), (ATTACK3, 60)]
                .into_iter()
                .collect(),
        );
        let u = animal(&mut s, 0, 12, 436, DEFAULT, 0, 32);
        s.set_anim(u, ATTACK2, false, true);
        let rolled = s.units[u].guys[0].anim;
        assert!(
            (ATTACK1..=ATTACK3).contains(&rolled),
            "one of the three: {rolled}"
        );
    }

    /// **A scholar's same-category test collapses every slot above
    /// `CHAR_UNPACK`** — `set_anim:205–221` (`docs/ANIM.md` §4.11, audit
    /// row R8). Both sides are remapped, so a scholar asked for one work
    /// slot while playing another keeps the clock it has instead of
    /// restarting; a non-scholar on the same pair restarts.
    #[test]
    fn a_scholar_keeps_its_clock_between_two_work_slots() {
        let mut s = sim_at(7);
        s.art.lengths.insert((6338, DUMP_ORE), 80);
        s.art.lengths.insert((6338, MINE_ORE), 30);
        let scholar = s.add_unit_type(crate::UnitType {
            worker: crate::orders::Worker::Scholar,
            ..Default::default()
        });
        let u = animal(&mut s, 1, 33, 6338, DUMP_ORE, 62, 80);
        s.units[u].ty = Some(scholar);
        let before = s.rng.seed;
        s.set_anim(u, MINE_ORE, false, true);
        assert_eq!(s.rng.seed, before, "the same-category return draws nothing");
        let g = s.units[u].guys[0];
        assert_eq!(
            (g.anim, g.cur_time, g.end_time),
            (DUMP_ORE, 62, 80),
            "the scholar keeps slot and clock"
        );

        // The control: the same pair on a unit that is not a scholar.
        let mut s = sim_at(7);
        s.art.lengths.insert((6338, DUMP_ORE), 80);
        s.art.lengths.insert((6338, MINE_ORE), 30);
        let u = animal(&mut s, 1, 33, 6338, DUMP_ORE, 62, 80);
        s.set_anim(u, MINE_ORE, false, true);
        let g = s.units[u].guys[0];
        assert_eq!((g.anim, g.cur_time, g.end_time), (MINE_ORE, 0, 30));
    }

    /// **`CHAR_ATTACKWALK`'s head arm zeroes `UnitData::recharging`** —
    /// `set_anim:88–103` (`docs/ANIM.md` §4.9, audit row R9). Ahead of
    /// every early return: a guy not already on slot 10 asked for it
    /// clears the counter, unless the counter has reached six, in which
    /// case the request becomes a plain `CHAR_WALK` instead.
    #[test]
    fn an_attackwalk_request_clears_the_recharge_or_becomes_a_walk() {
        let mut s = sim_at(7);
        s.art.lengths.insert((436, ATTACKWALK), 20);
        s.art.lengths.insert((436, WALK), 15);
        let u = animal(&mut s, 0, 12, 436, DEFAULT, 0, 32);
        s.units[u].combat.recharging = 5;
        s.set_anim(u, ATTACKWALK, false, true);
        assert_eq!(s.units[u].combat.recharging, 0, "under six: cleared");
        assert_eq!(s.units[u].guys[0].anim, ATTACKWALK);

        // At six the request is a walk and the counter stands.
        let mut s = sim_at(7);
        s.art.lengths.insert((436, ATTACKWALK), 20);
        s.art.lengths.insert((436, WALK), 15);
        let u = animal(&mut s, 0, 12, 436, DEFAULT, 0, 32);
        s.units[u].combat.recharging = 6;
        s.set_anim(u, ATTACKWALK, false, true);
        assert_eq!(s.units[u].combat.recharging, 6, "at six: untouched");
        assert_eq!(s.units[u].guys[0].anim, WALK, "and the request is a walk");

        // A guy already on slot 10 skips the arm entirely.
        let mut s = sim_at(7);
        s.art.lengths.insert((436, ATTACKWALK), 20);
        let u = animal(&mut s, 0, 12, 436, ATTACKWALK, 3, 20);
        s.units[u].combat.recharging = 6;
        s.set_anim(u, ATTACKWALK, false, true);
        assert_eq!(s.units[u].combat.recharging, 6);
        assert_eq!(s.units[u].guys[0].anim, ATTACKWALK);
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
