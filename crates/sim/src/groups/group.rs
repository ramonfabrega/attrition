//! Groups — the order layer between an army and its units
//! (`docs/GROUPS.md`).
//!
//! Nothing in the original gives an order to a unit directly: a click, an
//! AI army's tick and a unit's own `go_to` all build a `Group` and call a
//! `Group::action_*` on it, and that walks the members and calls
//! `Unit::add_*_order` on each. This module is that layer — the five
//! actions `docs/ARMY.md` cites, the membership queries they lean on, and
//! `Groups::push_group`'s one live rule.
//!
//! The simulation collapses an army to a single group (`docs/ARMY.md`
//! §3.2) and has no player selection, so a [`Group`] here is a **value**:
//! the member list plus the two fields an action reads, built from an army
//! ([`Sim::army_group`]) or on the stack (§9's siege sub-group). The
//! persistent half of the record lives on the army as [`GroupState`].
//!
//! What is modelled and what stands in is `docs/GROUPS.md` §12; the
//! largest seam is `Form::compute`'s slot table (§6.4), so every member of
//! a group is given the **same** destination. That seam stands on **cost
//! alone**: the second reading (`docs/audit/2026-08-25-groups.md`) killed
//! both of the reasons the first gave for it — there is no float barrier,
//! and `GroupData::log_data` dumps the table's whole output every frame.
//!
//! Two substitutions worth naming here rather than only in the document:
//!
//! - §6.6 step 1 exempts exactly `TypeIndex` `0x32..=0x35` from the
//!   per-member `+0xaa` write. This module tests `role::ROLE_CITIZEN`, and for
//!   types loaded from the install the two are the **same set**:
//!   `UnitType::determine_roles@0061c320` opens by setting `role = 0x200`
//!   for those four ids and nothing else ever sets that bit
//!   (`ai_load::RoleFacts::citizen_id`, by identity). For a hand-built test
//!   type they are whatever the test says.
//! - The simulation keeps an attack's target on the unit
//!   (`combat.target`/`combat.mandatory`) rather than on the order, so
//!   §10's `ox`/`whom`/`+0x1c` comparison is made against those.

use crate::ai::ai_load::{uflags, uflags2};
use crate::combat::{Obj, Stance};
use crate::movement::{Angle, find_angle};
use crate::orders::{Body, MoveKind, Order, QueuePos, flag, index};
use crate::territory::attrition::Domain;
use crate::world::{Pos, vector_dist};

/// `GroupData::get_loc`'s two gates, `70e128` and `70e1b7`. The compare is
/// `cmpl $0x180` followed by `jg`, so the bound is **inclusive** and the
/// decompiler's `< 0x181` names the same set.
const GROUP_LOC_NEAR: i32 = 0x180;
use crate::{Player, Sim};

/// Where this module knowingly stands in for the original, in one list.
///
/// | seam | stands in for | what it costs |
/// | --- | --- | --- |
/// | `Form::compute`'s slot table | §6.4, where in the formation each member stands | every member takes the group's own destination; the group arrives as a heap. Diffable: `GROUPDATA` logs `off_x`/`off_y`/`curr_x`/`curr_y`/`angles`/`form_num` per member |
/// | the group pool | §3, 64 slots a leader and `get_open_slot`'s recycling | numbered since item 518 (§19) and reset by [`Sim::groups_process`]; `get_open_slot`'s fallbacks are not modelled (`equals_group`'s normalize is, §36.2) |
/// | `GroupMoveOrder` | §6.6's per-frame formation | every member gets a plain `Move` — `docs/ORDERS.md` §8.4's verdict |
/// | `action_guard`'s building arm, human sweep and `QUEUE_FIRST` | §9's escort half, `docs/ORDERS.md` §24.7 | no building is ever guarded; the unit arm is built (item 567) |
/// | the order-time path plan | §6.7 | the sim plans on the first step, in `do_move`; with a zero slot offset the plan is the same one |
/// | `find_nearby_spot`'s collision, `invalid_loc` on a slot | §6.6 step 4's same-region arm | no slot is ever invalid in the same region, so no member is re-slotted there; the different-region arm is built (§37.1) |
/// | `QUEUE_FIRST`'s insert dance (`set_up_insert` / `action_halt` / recurse / `finish_insert`) | §6.2, §10 | `charge`'s `QUEUE_FIRST` is a plain `push_front` on each member |
/// | the scenario `ignore_orders` filter | §5 | never set outside a scenario |
/// | `is_modern_infantry`, the strafe | §6.6, §10 | no modern infantry, no planes (`is_packing` is built: [`Sim::clear_keeping_pack`], item 3336) |
/// | the area-id table (`world +0x134`) | §6.7, §9's anchor test | the sim compares tile regions instead |
pub mod seams {}

// Item 4015 cut this file four ways (STRUCTURE §5, move 7). Each piece is a
// child, so an item private here is `pub(super)` there and reaches what it
// reached; the globs keep every `crate::groups::group::x` path. `pool` and
// `actions` hold `impl Sim` methods and nothing named through this path, so
// they take no glob.
mod actions;
mod pool;
mod query;
mod state;
#[cfg(test)]
mod tests;
pub(crate) use self::query::*;
pub use self::state::*;
