//! `Unit::think_fish@005f4c60` — where an idle fishing boat goes next.
//!
//! The specification is `docs/ORDERS.md` §6.8. In one paragraph: an idle
//! Fisherman scores the 289 cells of the 17 × 17 around its own, keeps the
//! ones that are water in its own region and unblocked, and takes the best
//! of `1_000_000 / manhattan` — a million if the cell holds a good it may
//! gather, nothing at all if it does not — plus a jitter of `rnd(60)` and
//! **plus the ring index itself**. If the winner is the cell it is standing
//! on it unpacks; otherwise it walks there.
//!
//! Three arms, all three of them in run58's capture and all three needed:
//! frame 4462 has no good anywhere in range and the `+ i` bias alone
//! decides it; 4871 has one Fish five cells off and the million carries it;
//! 4948 is the boat standing on that Fish, which is the cast.

use crate::orders::{MoveKind, QueuePos};
use crate::tech::TypeId;
use crate::world::{Cell, Good, MOVE_289, OIL_GOOD, Pos, UNITS_PER_CELL};
use crate::{Player, Sim};

/// `TypeIndex::FISHERMEN` — the lineage `Unit::think@005f6e40:1421` tests
/// with `is(0x13d, 0)` before it calls this at all.
pub use crate::ids::ty::FISHERMEN;

/// The spell `Unit::add_cast_order` rewrites the generic **unpack**
/// (`0x28c`) into for a unit in the `FISHERMEN` lineage, and the value
/// run58's block 4949 prints as `spell 658`.
pub use crate::ids::spell::UNPACK_FISHERMEN;

/// The generic unpack, as `think_fish` asks for it.
pub use crate::ids::spell::UNPACK;

/// The numerator a cell that holds a gatherable good scores before the
/// distance divides it (`005f4e2e`: `0xf4240`).
const GOOD_SCORE: i32 = 1_000_000;

/// The jitter's span — `rnd(0x3c)`, added to every candidate's score.
const JITTER: i32 = 0x3c;

/// The one draw site, under the original's own offset from
/// `think_fish@005f4c60` — the `rnd(60)` each accepted cell spends. The
/// second draw in the function, at `+0x2c9`, is the fallback for
/// `best > 0x120`, and `best` starts at 0 and is only ever assigned a loop
/// index, so nothing can reach it (§6.8, "The unreachable half").
pub const SITE_JITTER: &str = "Unit::think_fish+0x27a";

/// `Unit::think_fish` up to `Unit::think_merchant`, the next function in
/// the export — what isolates this mechanic's own draws in a trace.
pub const CODE: std::ops::Range<u32> = 0x005f_4c60..0x005f_5170;

/// The head's cadence test (`005f4c8f`): whether `(index + frame)` is a
/// multiple of 1,024 — the original masks with `0x800003ff` and tests the
/// sign half separately, which comes to the low ten bits being clear.
pub(crate) fn recheck_due(index: i16, frame: i64) -> bool {
    (i64::from(index) + frame) & 0x3ff == 0
}

/// What one examined cell is worth before the distance divides it
/// (`005f4d8a`..`005f4e3e`), or `None` when the walk skips it: the coastal
/// half-land bit, a land type other than the water pair, a region that is
/// not the unit's own, or a good somebody else has already claimed.
///
/// `good` is the `find_good_at` answer (asked only for a cell that got that
/// far, as the original does) and `claim` the cell's `(down, who)` pair —
/// the claim holds when `down >= 0`, `who < 8` (the original compares the
/// **short** at `+0xa`) and `down` is not the asking unit's own index.
pub(crate) fn cell_base(
    flags: u16,
    land: i8,
    same_region: bool,
    good: impl FnOnce() -> bool,
    claim: impl FnOnce() -> Option<(i16, i16)>,
    me: i16,
) -> Option<i32> {
    if flags & 0x100 != 0 || !(land == 1 || land == 2) || !same_region {
        return None;
    }
    if !good() {
        return Some(0);
    }
    match claim() {
        Some((down, who)) if down >= 0 && who < 8 && down != me => None,
        _ => Some(GOOD_SCORE),
    }
}

/// The walk (`005f4d40`..`005f4f09`): `at(i, cell)` answers `None` for a
/// cell the walk skips, else its base score and the `Random::get(0, 0xffff)`
/// it spent; the score is `base / manhattan` (at least 1) `+ draw % 60 + i`,
/// the best strictly-greater score wins, and `0` — the unit's own cell — is
/// what stands when nothing was accepted.
pub(crate) fn best_cell(
    here: (i32, i32),
    moves: &[(i32, i32)],
    mut at: impl FnMut(usize, (i32, i32)) -> Option<(i32, i32)>,
) -> usize {
    let (mut best, mut best_score) = (0usize, 0i32);
    for (i, &(dx, dy)) in moves.iter().enumerate() {
        let c = (here.0.wrapping_add(dx), here.1.wrapping_add(dy));
        let Some((base, roll)) = at(i, c) else {
            continue;
        };
        let dist = here
            .0
            .wrapping_sub(c.0)
            .wrapping_abs()
            .wrapping_add(here.1.wrapping_sub(c.1).wrapping_abs())
            .max(1);
        let score = (base / dist)
            .wrapping_add(roll % JITTER)
            .wrapping_add(i32::try_from(i).expect("an index"));
        if best_score < score {
            best = i;
            best_score = score;
        }
    }
    best
}

impl Sim {
    /// `ObjectsData::find_good_at@0065bec0(x, y, who, 0, 0)` — the fast
    /// path, which is the only one `think_fish` uses.
    ///
    /// The original walks the cell's object chain and asks whether it ends
    /// in a good; here the terminator is a field, because
    /// `Objects::init_good` is the only thing that ever writes one
    /// ([`crate::world::World::good_at`]). Then: the good must be live, it
    /// must not be `OIL_GOOD`, and — since `who` is a real player here —
    /// `LeaderData::type_avail(t, 1)` must be non-zero.
    ///
    /// Answers the good's **`TypeIndex`**, as the original does; every
    /// caller here only asks whether there is one.
    pub(crate) fn find_good_at(&self, c: Cell, who: Player) -> Option<TypeId> {
        let (_, g): (usize, Good) = self.world.good_at(c)?;
        if !g.alive || g.ty == OIL_GOOD {
            return None;
        }
        (self.type_avail(who, g.ty) != crate::tech::NOT_AVAILABLE).then_some(g.ty)
    }

    /// `Unit::think_fish@005f4c60` (`docs/ORDERS.md` §6.8). `true` when it
    /// did something — a move order or a cast — which ends the caller's
    /// `think`.
    pub(crate) fn think_fish(&mut self, u: usize, frame: i64) -> bool {
        let packs = self.units[u]
            .ty
            .is_some_and(|t| self.tables.unit_types[t].combat.packs);
        let packed = self.units[u].combat.packed;
        let who = self.units[u].owner;

        // The head, and it is only for a boat that has already deployed:
        // once in 1,024 frames it asks `UnitData::calc_gather` whether it
        // can still gather where it stands, records the answer in
        // `unit_masks & 0x20`, and gives up for this frame if it can. A
        // **packed** fisherman skips the whole thing and searches every
        // time it is called.
        //
        // `calc_gather` is `docs/ORDERS.md` §6.10 and
        // [`crate::calc_gather`]; with `param_7` set it never writes the
        // `0x20` bit, so the head's only visible effect on a fishing boat
        // is `rare`, `good_obj` and this early return. run58's `1/14`
        // reaches it once, on frame **5106**, where it is standing on its
        // fish — and the whole of East Indies' long word sat on this
        // return.
        if packs && !packed {
            if !recheck_due(self.units[u].index, frame) {
                return false;
            }
            if self.calc_gather(u) {
                return false;
            }
        }

        let here = self.units[u].pos.cell();
        let Some(region) = self.world.tregion_alt(self.units[u].pos.tile()) else {
            return false;
        };

        // The walk. `best` starts at **0** — the unit's own cell — and the
        // score to beat at 0, so a fisherman that accepts nothing anywhere
        // ends up pointing at itself.
        let me = self.units[u].index;
        let best = best_cell((here.x, here.y), &MOVE_289, |_, (x, y)| {
            let c = Cell::new(x, y);
            if !self.world.contains(c) {
                return None;
            }
            let d = self.world.cell_data(c);
            // `WData.region`, not `get_tregion`: the candidate is compared
            // raw against the *unit's* `get_tregion`. A good another
            // fisherman has already claimed — the cell's chain head is
            // somebody else's object — is skipped entirely, and skipped
            // **before** the draw.
            let base = cell_base(
                d.flags,
                d.land,
                self.world.region_of(c) == Some(region),
                || self.find_good_at(c, who).is_some(),
                || self.claim_of(c).map(|(down, w)| (down, i16::from(w))),
                me,
            )?;
            self.mark(SITE_JITTER);
            Some((base, self.rng.roll()))
        });

        let (dx, dy) = MOVE_289[best];
        if dx == 0 && dy == 0 {
            // Standing on the winner. A packed boat deploys; an unpacked
            // one has nothing left to do.
            if packs && packed {
                self.add_cast_order(u, UNPACK);
                return true;
            }
            return false;
        }
        let to = Pos::new(
            (here.x + dx) * UNITS_PER_CELL + UNITS_PER_CELL / 2,
            (here.y + dy) * UNITS_PER_CELL + UNITS_PER_CELL / 2,
        );
        self.add_move_order(u, to, MoveKind::MoveTo, QueuePos::New, false);
        true
    }

    /// `WData.down` / `down_who` as `think_fish` reads them: the object at
    /// the head of the cell's chain, when there is one.
    ///
    /// **The live chain first, and the loaded snapshot only behind it.**
    /// `Object::add_to_world` pushes every object onto the head of its
    /// cell's list, so once a boat has deployed onto its fish it *is* that
    /// cell's `down` — and the next boat's search must refuse the cell.
    /// This crate chains units and nothing else (`docs/COLLISION.md` §3,
    /// item 48), so the buildings and goodies the original threads through
    /// the same list can only come from the snapshot a start dump loaded,
    /// which is what the fallback is for. run54's frame **5285** is the
    /// diff: with the snapshot alone the AI's second Fisherman accepted
    /// 178 cells where the original accepted 177, and the extra one was
    /// the fish the first boat is sitting on.
    pub(crate) fn claim_of(&self, c: Cell) -> Option<(i16, i8)> {
        if c.x >= 0
            && c.y >= 0
            && c.x < self.world.width()
            && c.y < self.world.height()
            && let Some(o) =
                self.chain_heads[(c.y as usize) * (self.world.width() as usize) + (c.x as usize)]
        {
            return Some((self.units[o].index, self.units[o].owner as i8));
        }
        let d = self.world.cell_data(c);
        (d.down >= 0).then_some((d.down, d.down_who))
    }
}

#[cfg(test)]
mod tests;
