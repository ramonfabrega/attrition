//! Orders: the per-unit order list, the three enqueue modes, one order stepped
//! per frame, and the lifecycle every other mechanic hangs off.
//! `docs/ORDERS.md`.
//!
//! The original keeps a circular ring of `UnitOrder`s per unit whose current
//! order is `head->prev`, the oldest; `QUEUE_LAST` appends, `QUEUE_FIRST`
//! rotates the new order to the front, `QUEUE_NEW` kills everything first.
//! Here that is a [`VecDeque`] whose **front is the current order**. The step
//! is `Unit::work` → `do_job`: one dispatch on the front order per unit per
//! frame, after attrition and before the body follows (`docs/ORDERS.md` §2).
//! An order a `do_*` inserts in front (`QUEUE_FIRST`) runs from the next
//! frame; an order applied from outside the step runs in the same frame.
//!
//! What is modelled and what is an input is §13 of the document. The
//! pathfinder is no longer a stub: [`Sim::find_path`] is the straight-line
//! verifier, and the three grid planners live in `path.rs`
//! (`docs/PATHFINDER.md`).

use crate::ai_load::{uflags, uflags2};
use crate::anim;
use crate::build::{Ident, flags as bflags};
use crate::combat::{self, Obj};
use crate::economy;
use crate::garrison::GarrisonRefused;
use crate::movement::{self, Angle, find_angle};
use crate::single::Single;
use crate::world::{tile, vector_dist};
use crate::{Player, Pos, Sim, Unit, farms};
/// `PackerStanceIndex::PACKER_AUTO` (the PDB): a packer that unpacks by
/// itself once idle.
const PACKER_AUTO: u8 = 0;

const SNAP: i32 = 0x30;
const SNAP_CENTRE: i32 = 0x18;
use crate::ids::scale::HALF_TILE;
const TILE: i32 = crate::ids::scale::UNITS_PER_TILE;
const CELL: i32 = crate::ids::scale::UNITS_PER_CELL;
/// `Game::init_data@0058dca0`: `num_def_builds = 200`, the per-player
/// building slot count, and the threshold `Objects::find_builds` compares
/// its circle against.
pub(crate) const NUM_DEF_BUILDS: usize = 200;
/// The 48-unit snap, `div_3_table[v >> 4] * 0x30 + 0x18`.
///
/// Two callers in the original and they are the same arithmetic: every move
/// order's destination (`Unit::add_move_facing_order`, §4.3) and **every
/// unit's starting position** — `Unit::init@00612100:69` snaps both
/// coordinates before it hands them to `Object::init` and to
/// `set_new_location`, so an object is never born off the grid however
/// unrounded the point its maker computed. `div_3_table` is `floor(i / 3)`
/// and the coordinates are non-negative, so `div_euclid` is the same table.
pub(crate) const fn snapped(p: Pos) -> Pos {
    Pos::new(
        p.x.div_euclid(SNAP) * SNAP + SNAP_CENTRE,
        p.y.div_euclid(SNAP) * SNAP + SNAP_CENTRE,
    )
}

/// The working-the-tile radius of a woodcutter or miner.
const AT_TILE: i32 = 0x140;
/// The oil-well stance offset.
const OILWELL_OFFSET: i32 = 0x120;
/// Adjacency: `attack_dist < 0x60`.
const ADJACENT: i32 = 0x60;
/// The herder's phase — `Unit::do_gather@005ef2a0:5efd8c`. A citizen
/// gathering at a **pasture** re-picks its spot on the frames where
/// `(o · 7 + frame + who) % 256` is zero, and does nothing else at all
/// (§6.5). Both numbers are the listing's.
const PASTURE_STRIDE: i32 = 7;

/// `unit_masks & 0x78000000` — the four carrying-walk bits
/// ([`crate::Unit::carry`]), named as `Guy::set_anim`'s walk arm tests
/// them and written only by `Unit::do_non_flat_gather` (§6.4).
pub(crate) const CARRY_WITH_WOOD: u32 = 0x0800_0000;
pub(crate) const CARRY_TO_WOOD: u32 = 0x1000_0000;
pub(crate) const CARRY_WITH_ORE: u32 = 0x2000_0000;
pub(crate) const CARRY_TO_ORE: u32 = 0x4000_0000;
/// The nibble `& 0x87ffffff` clears whole.
const CARRY_ANY: u32 = CARRY_WITH_WOOD | CARRY_TO_WOOD | CARRY_WITH_ORE | CARRY_TO_ORE;

/// The wood machine's **three** direct draw sites, under the original's own
/// offsets from `Unit::do_non_flat_gather@005f0170`, §6.4. [`Sim::mark`]
/// writes them into [`Sim::phase_marks`], which is what keeps them from
/// being read as the stand that precedes them (`docs/SYNC.md` §5).
///
/// The middle one is the trap. The two waits are **not** the same branch:
/// `+0xcc3` is the chopping guy's `% 100 + 300`, reached only through the
/// `cur_anim == CHAR_CHOP_WOOD` test that stands in front of the tile
/// arithmetic, and `+0xdad` is the arrival frame's `% 50 + 100`, reached
/// only when the guy is *not* yet chopping. A woodcutter takes the first
/// on every reroll after its first frame at the tile and the second
/// essentially never — so a merged branch that rolls `% 50 + 100` sends it
/// back to the camp two hundred frames early (`docs/JOURNAL.md`,
/// 2026-08-31).
pub const SITE_TILE_WAIT: &str = "Unit::do_non_flat_gather+0x54b";
pub const SITE_WORK_WAIT: &str = "Unit::do_non_flat_gather+0xcc3";
pub const SITE_ARRIVE_WAIT: &str = "Unit::do_non_flat_gather+0xdad";

/// The farmer's cell re-pick — `Unit::do_gather@005ef2a0`'s two
/// `GameAccess::rnd(4)` calls, the pair that follows the cell-state switch
/// when the state and the animation disagree (§6.5).
///
/// Both draws are the **same** address: `GameAccess::rnd` is a frameless
/// helper whose `Random::get` call returns to `+0x20` whoever asked, so the
/// trace names the pair once and the `ebp` walk skips straight past both it
/// and `do_gather` to `Unit::do_job+0x67`. One mark therefore carries the
/// two draws (`docs/SYNC.md` §3.10).
pub const SITE_FARM_CELL: &str = "GameAccess::rnd+0x20 < Unit::do_job+0x67";

/// `Unit::do_move@005f7b30:599`'s grid draw — the call is at `005f89af`,
/// so the site is `+0xe84` (§4.4, `docs/SYNC.md` §6's frame-3 item). The
/// sim reaches it where the original does not, and naming it is what turns
/// that from an unattributed unit-loop draw into a row.
///
/// It is also Great Lakes' parting frame, and there in the other
/// direction: on 6994 the original spends two of these under
/// `Unit::do_attack_to` and this crate none, because the units that spend
/// them are a squad the original has just given an order to and this crate
/// leaves standing — `docs/ARMY.md` §16.7, not a pathfinder question.
pub const SITE_MOVE_GRID: &str = "Unit::do_move+0xe84";

/// `Unit::do_guard@005e5c70+0x8fb` — the `retry` roll after a reposition
/// whose leg ended on the frame it was issued (`docs/ORDERS.md` §24).
pub const SITE_GUARD_RETRY: &str = "Unit::do_guard+0x8fb";

/// `do_guard`'s three stands, each `set_anim(CHAR_DEFAULT, 0, 1)` through
/// `Unit::set_anim`, one draw a figure, named by their return addresses in
/// the listing (item 569): `5e6464` the **idle stand on the post**,
/// `5e6550` the stand before the `retry` roll when a reposition leg ended
/// the frame it was issued, and `5e6596` the stand before a dead target's
/// order is killed. Item 567 built all three unmarked, and golden chapter
/// four's escort spends the first from 1464.
pub const SITE_GUARD_IDLE: &str = "Guy::set_anim+0x97a < Unit::do_guard+0x7f4";
pub const SITE_GUARD_STAND: &str = "Guy::set_anim+0x97a < Unit::do_guard+0x8e0";
pub const SITE_GUARD_DEAD: &str = "Guy::set_anim+0x97a < Unit::do_guard+0x926";

/// `Unit::do_follow@005e65d0`'s stand within the standoff,
/// `set_anim(CHAR_DEFAULT, 0, 1)` at `5e68f5`, named by its return
/// address (`docs/ORDERS.md` §28).
pub const SITE_FOLLOW_STAND: &str = "Guy::set_anim+0x97a < Unit::do_follow+0x32a";

/// The 31 bearings of one ring of `find_nearby_spot`, as multiples of a
/// sixteenth of a turn from the base angle; `|k| >= 8` adds a thirty-second.
/// The tile bits a warship's spot may not carry
/// (`find_nearby_spot@0061de70`, `uVar7 & 0x2400`): [`tile::BAD_PATH`]
/// and the `0x400` this crate has no name for. `UnitData::invalid_loc`
/// answers 3 on the same pair (`docs/ORDERS.md` §10).
pub const WARSHIP_REFUSES: u16 = tile::BAD_PATH | 0x400;

impl Unit {
    /// Discard waypoints through the current segment's final waypoint.
    ///
    /// This is the path-only operation used by order teardown (ORDERS §3.2).
    /// It can also be replayed on a unit without constructing a world or sim.
    pub fn discard_current_path_segment(&mut self) {
        while let Some(p) = self.path.pop() {
            if p.flags & path_flag::FINAL != 0 {
                break;
            }
        }
    }
}

#[cfg(test)]
mod chase_tests;

#[cfg(test)]
mod resolve_block_tests;

#[cfg(test)]
mod loose_tests;

#[cfg(test)]
mod all_gathering_tests;

#[cfg(test)]
mod unreachable_goal_tests;

#[cfg(test)]
mod snap_refusal_tests;

pub mod build_spot_view;
pub mod check_build_view;
pub mod do_attack;
pub mod group_move;
#[cfg(test)]
mod helicopter_garrison_tests;
pub mod topp;

// Item 4015 cut this file six ways (STRUCTURE §5, move 7). Each piece is a
// child, so an item private here is `pub(super)` there and reaches what it
// reached; the globs keep every `crate::orders::x` path. `think` holds only
// `impl Sim` methods and two `pub(crate)` steps only the sweeps name, so
// its glob is the tests'.
mod attack;
mod jobs;
mod moving;
mod order;
mod order_list;
mod think;
pub use self::attack::*;
pub use self::jobs::*;
pub use self::moving::*;
pub use self::order::*;
pub use self::order_list::*;
#[cfg(test)]
pub(crate) use self::think::*;
