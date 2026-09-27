//! The gather point, the player's rally point on a building
//! (`docs/PRODUCTION.md`, "The gather point"; `docs/GOLDEN.md` §39).
//!
//! `BuildData::gather` (`+0xb8`, a `PtrLinkListAbstract<GatherPoint>`, the
//! count at `+0xc8`) holds the points in order; `Build::add_gather_point@
//! 00622e70` and `Build::clear_gather@00623180` are its only writers.

use crate::world::Pos;

/// `GatherPoint`: `x`, `y` and `action`, the record the dump prints as
/// `GATHERPOINT`. `action` 0 is a point on the ground, 1 a friendly
/// object's point, 2 an enemy's, 3 an object named by `(o, who)` in
/// `(x, y)`; `(−1, −1)` with an action other than 3 is "inside"
/// (`GatherPoint::is_inside@00730400`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GatherPoint {
    pub pos: Pos,
    pub action: u8,
}
