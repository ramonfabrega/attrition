//! The order list and the path stack — a unit's `ORDERS` record.

use super::*;

/// What two order lists, or two path stacks, disagree about.
///
/// The order list is the unit's *intent*, and it diverges long before a
/// position does — or, worse, never shows in a position at all, which is the
/// whole reason this comparison exists (`check_start_orders`' doc comment
/// says it for the starting orders; this says it for every frame).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrderMismatch {
    /// The lists are different lengths.
    Length { ours: usize, theirs: usize },
    /// `get_type()` — the `OrderIndex` of the order in this slot, in the
    /// **original's** space on both sides.
    ///
    /// It was not, until item 237. This crate spelled a grouped move
    /// `MOVE_TO`/`ATTACK_TO` with a [`sim::orders::MoveOrder::group`]
    /// beside it where the original has a `GroupMoveOrder`/
    /// `GroupAttackToOrder` of its own and writes `19`/`21`, so the row
    /// fired on **every frame of every formation** whatever this crate
    /// did — 630 of them across run76's Archer march alone — and,
    /// because a kind disagreement skips the rest of the slot, took the
    /// whole `MOVEORDER` and `GROUPORDER` row with it. Now
    /// [`sim::orders::Order::index`] answers `get_type()` and the row
    /// can fail: a squad that stays in formation past its ungroup, or
    /// leaves one early, disagrees here on the frame it happens.
    Kind { ours: i64, theirs: i64 },
    /// The dump holds an `OrderIndex` this crate has **no spelling for**
    /// — `sim::orders::index::is_modelled` is false, so no state of this
    /// simulation could ever produce it (`docs/ORDERS.md` §1.2's second
    /// table). A structural gap rather than a state divergence, and
    /// separated from [`Self::Kind`] for that reason; it still scores,
    /// because the alternative is the silent pass this item was written
    /// to remove.
    Unspellable { theirs: i64 },
    /// The dump's own two statements of the kind disagree: the block's
    /// **name** implies one `OrderIndex` and the `type` line beside it
    /// says another ([`crate::gamelog::OrderDump::named_index`]). Not a
    /// divergence between the two simulations at all — it means the
    /// positional `type`/body pairing has slid, which a block name the
    /// walk drops does, and it has happened once already. Scores,
    /// loudly, because every field of every later slot is then reading
    /// the wrong record.
    Header { named: i64, theirs: i64 },
    /// One field of a `GROUPORDER` row, or `GroupMoveOrder`'s own
    /// `in_group`, named as the log writes it (`docs/ORDERS.md` §11.1,
    /// `docs/GROUPS.md` §6.6). The whole record the dump prints for a
    /// formation's order, which nothing compared while the kind row above
    /// it could not agree: `oxx`/`whose` name the leader
    /// `GroupData::find_leader` chose, `id` the group-move id, `form_id`
    /// the member's slot and `group_angle` the bearing its slot was laid
    /// out on.
    Group {
        field: &'static str,
        ours: i64,
        theirs: i64,
    },
    /// The action bit, `UnitOrder::flags & 4` (§1.3): an intent rather than
    /// a transit leg. Settled, so it scores.
    Action { ours: bool, theirs: bool },
    /// `TargetOrder`'s `whom`/`ox`: whose object, and which.
    Target {
        ours: Option<(i64, i64)>,
        theirs: Option<(i64, i64)>,
    },
    /// The whole `flags` byte. **Does not score**: `0x8` and `0x10` have no
    /// established reader (`docs/ORDERS.md` §14) and `0x1` (`PATHED`) follows
    /// the path stack, which the pathfinder stub does not reproduce. Reported
    /// because a surprise here is worth seeing.
    Flags { ours: u8, theirs: i64 },
    /// One field of a `GATHERORDER`'s own row, named as the log writes it
    /// (`docs/ORDERS.md` §6.4). The tile is the one that matters: the order
    /// list can agree on kind, target and flags for a hundred frames while
    /// the two sides send the worker to different trees, and the first sign
    /// is a `Length` two frames later when one of them queues a walk the
    /// other does not.
    Gather {
        field: &'static str,
        ours: i64,
        theirs: i64,
    },
    /// `MoveOrder::coll_x`/`coll_y` — the point the last collision refused
    /// (`docs/COLLISION.md` §4.3).
    Coll {
        ours: Option<(i32, i32)>,
        theirs: (i64, i64),
    },
    /// One field of a `MOVEORDER`'s own row, named as the log writes it
    /// (`docs/ORDERS.md` §4.1). The destination is the one that matters:
    /// two sides can hold the same kind, the same target and the same
    /// flags while walking to different points, and the first thing any
    /// other check sees is a `Length` a few frames later when one of them
    /// arrives and the other does not.
    Move {
        field: &'static str,
        ours: i64,
        theirs: i64,
    },
    /// The path stack's depth.
    PathLength { ours: usize, theirs: usize },
    /// A path segment's goal, bottom-first.
    PathTo {
        slot: usize,
        ours: (i32, i32),
        theirs: (i64, i64),
    },
    /// The **rest** of a `PATHDATA` row — `tolerance` and `flags`, the two
    /// fields the dump prints beside the point and which nothing compared
    /// for as long as the path stack has been an oracle
    /// (`docs/PATHFINDER.md` §10). The working agreement's "diff the whole
    /// record": a stack whose points are the original's can still carry a
    /// waypoint the original marked final and this crate did not, and that
    /// difference is what says *who built the stack*, not merely where it
    /// goes.
    PathField {
        slot: usize,
        field: &'static str,
        ours: i64,
        theirs: i64,
    },
}

impl OrderMismatch {
    /// Whether this is about the path stack rather than the order list.
    pub const fn is_path(&self) -> bool {
        matches!(
            self,
            Self::PathLength { .. } | Self::PathTo { .. } | Self::PathField { .. }
        )
    }

    /// Whether it counts against the order score. Everything does except
    /// [`Self::Flags`], for the reason on that variant, and two fields of
    /// [`Self::Move`]:
    ///
    /// - **`dest`** is "I have a current waypoint", and the original clears
    ///   it on arrival, on a `go_around_building` failure, after
    ///   `resolve_unit_collision` and on a collision at `coll_x/coll_y`
    ///   (`docs/ORDERS.md` §4.1) — three of the four are the pathfinder
    ///   seam's own timing, which this crate does not reproduce frame for
    ///   frame. `dest_x/dest_y`, the waypoint itself, scores on the frames
    ///   the flag says it is live, and the path stack scores outright.
    /// - **`last_x/last_y`** is where the last straight-line plan was made,
    ///   and only a *successful detour* writes it — `go_around_building`,
    ///   the same seam.
    /// - **`facing`** is the formation mirror, and `docs/QUEUE.md` item 23
    ///   is the open question of its sign: the mirrors the orders carry and
    ///   the flags the pool prints are known to be two different sequences.
    ///
    /// Both are reported, because a surprise in either is worth seeing.
    pub fn scores(&self) -> bool {
        match self {
            Self::Flags { .. } => false,
            // **`id`** is `GroupData +0x4` carried onto the order, and
            // this crate has no group pool to allocate one from: an
            // army group's *slot* stands in for it
            // (`crate::sim::group_id`), which is unique per owner and is
            // all `group_move_id` needs, but it is not the original's
            // number — run76's Archers carry the original's 64 where the
            // army slot is 1. A declared stand-in is not a drift, so it
            // does not score; it is still reported, because the other
            // five fields of the same row do score and a surprise in
            // this one would say the stand-in had stopped being unique.
            Self::Group { field: "id", .. } => false,
            Self::Move { field, .. } => !matches!(*field, "dest" | "facing" | "last_x" | "last_y"),
            _ => true,
        }
    }

    /// The row's **field name**, for a widening keyed by field — `order:coll`,
    /// `order:move.dest_x`, `path[2].to` — rather than the variant's name
    /// or, worse, the bare word `order`.
    ///
    /// **Why a key is a field and never a unit** (parked 452, item 448):
    /// a widening that keys its rows on `(who, o, "order")` files every
    /// order field after the first as "this unit's order already parted",
    /// and `1/38` carried a `GROUPORDER` `id` residue from block 9340 that
    /// swallowed `coll_x`/`coll_y` on the AI word's own frame — the two
    /// fields that said what the collision sweep decided. Item 448 keyed
    /// by field inside its own test; this is that label made the only
    /// one, so `unit::rows` and every reader of it see every field.
    pub fn label(&self) -> String {
        match self {
            Self::Length { .. } => "order:length".into(),
            Self::Kind { .. } => "order:kind".into(),
            Self::Unspellable { .. } => "order:unspellable".into(),
            Self::Header { .. } => "order:header".into(),
            Self::Group { field, .. } => format!("order:group.{field}"),
            Self::Action { .. } => "order:action".into(),
            Self::Target { .. } => "order:target".into(),
            Self::Flags { .. } => "order:flags".into(),
            Self::Gather { field, .. } => format!("order:gather.{field}"),
            Self::Coll { .. } => "order:coll".into(),
            Self::Move { field, .. } => format!("order:move.{field}"),
            Self::PathLength { .. } => "path:length".into(),
            Self::PathTo { slot, .. } => format!("path[{slot}].to"),
            Self::PathField { slot, field, .. } => format!("path[{slot}].{field}"),
        }
    }

    /// The variant's name, for a tally.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Length { .. } => "length",
            Self::Kind { .. } => "kind",
            Self::Unspellable { .. } => "unspellable",
            Self::Header { .. } => "header",
            Self::Group { .. } => "group",
            Self::Action { .. } => "action",
            Self::Target { .. } => "target",
            Self::Flags { .. } => "flags",
            Self::Gather { .. } => "gather",
            Self::Coll { .. } => "coll",
            Self::Move { .. } => "move",
            Self::PathLength { .. } => "path-length",
            Self::PathTo { .. } => "path-to",
            Self::PathField { .. } => "path-field",
        }
    }
}

/// One unit's order list or path stack disagreeing on one frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrderDivergence {
    pub frame: i64,
    pub who: i64,
    pub o: i64,
    /// Position in the list, **front first** — slot 0 is the order being
    /// executed. The log writes the list the other way round
    /// ([`UnitDump::orders_front_first`]).
    pub slot: usize,
    pub what: OrderMismatch,
}

/// Compares one unit's order list and path stack against the logged ones
/// (`docs/ORDERS.md` §11.1).
///
/// Both sides are walked **front first** — the order being executed is slot
/// 0 — which for the log means reversing what it wrote. The path stack needs
/// no reversing: the log writes it bottom first and `Vec<PathData>` is pushed
/// and popped at the end, so both start at the goal.
///
/// A target is only compared when both sides name one: an order kind that
/// carries no target has none, and a simulation building the start-of-game
/// rule did not create has no logged id to be compared against. That keeps
/// the check from manufacturing disagreements out of what it cannot see.
pub(crate) fn compare_orders(
    built: &Built,
    link: &UnitLink,
    them: &UnitDump,
    frame: i64,
) -> Vec<OrderDivergence> {
    let mut out = Vec::new();
    let unit = &built.sim.units[link.unit];
    let mut at = |slot: usize, what: OrderMismatch| {
        out.push(OrderDivergence {
            frame,
            who: link.who,
            o: link.o,
            slot,
            what,
        });
    };

    if unit.orders.len() != them.orders.len() {
        at(
            0,
            OrderMismatch::Length {
                ours: unit.orders.len(),
                theirs: them.orders.len(),
            },
        );
    }
    for (slot, (ours, theirs)) in unit
        .orders
        .iter()
        .zip(them.orders_front_first())
        .enumerate()
    {
        // **The record against itself, first.** The block's name and the
        // `type` beside it are two independent statements of the kind; a
        // disagreement means the positional pairing has slid and every
        // field read out of this slot belongs to another order.
        if let Some(named) = theirs.named_index()
            && named != theirs.index
        {
            at(
                slot,
                OrderMismatch::Header {
                    named,
                    theirs: theirs.index,
                },
            );
            continue;
        }
        let kind = i64::from(ours.index());
        if kind != theirs.index {
            at(
                slot,
                // An `OrderIndex` no state of this simulation can produce
                // is a hole in the crate, not a disagreement about the
                // state; both are reported and both score, and the two
                // are told apart so a tally can say which it is.
                match u8::try_from(theirs.index) {
                    Ok(k) if !sim::orders::index::is_modelled(k) => OrderMismatch::Unspellable {
                        theirs: theirs.index,
                    },
                    _ => OrderMismatch::Kind {
                        ours: kind,
                        theirs: theirs.index,
                    },
                },
            );
            // The kinds disagree, so the fields under them are not
            // comparable; the rest of this slot would be noise.
            continue;
        }
        let action = ours.has(sim::orders::flag::ACTION);
        if action != theirs.is_action() {
            at(
                slot,
                OrderMismatch::Action {
                    ours: action,
                    theirs: theirs.is_action(),
                },
            );
        }
        if i64::from(ours.flags) != theirs.flags {
            at(
                slot,
                OrderMismatch::Flags {
                    ours: ours.flags,
                    theirs: theirs.flags,
                },
            );
        }
        let mine = built.target_ids(link.unit, ours);
        let logged = theirs
            .whom
            .zip(theirs.ox)
            .filter(|&(who, o)| who >= 0 && o >= 0);
        // **A target this crate cannot name is a divergence on an attack
        // order and a silence everywhere else** (item 496). `target_ids`
        // answers `None` for two different reasons and until now both
        // were skipped: for a `Move`, `Cast`, `Trade` or `Think` order,
        // and for a building the simulation made itself, it means *this
        // crate does not model a target here* — noise if it were
        // reported. For an **attack** order it means the opposite: the
        // order is the wrapper and the target lives in `combat::State`
        // (`docs/ORDERS.md` §13), so `None` is this crate saying the
        // unit is attacking nothing while the dump names what it is
        // attacking. That is a state disagreement, and it was quiet.
        //
        // It was quiet over chapter two's whole second fight.
        // `Sim::forget` drops a dead object from every attacker's target
        // slot; the original does not — run112's three bowmen carry
        // `ox 8 whom 1` on their `ATTACKORDER` from the frame `1/8` dies
        // (683) to the frame their reload opens (695), and this crate
        // carried nothing there. Twelve frames, three units, thirty-six
        // unit-frames of divergence that the comparison read as
        // agreement. `docs/COMBAT.md` §43.
        let attack = matches!(ours.body, sim::orders::Body::Attack(_));
        if logged.is_some() && mine != logged && (mine.is_some() || attack) {
            at(
                slot,
                OrderMismatch::Target {
                    ours: mine,
                    theirs: logged,
                },
            );
        }
        // **`coll_x`/`coll_y`**, the point the last collision refused. The
        // original leaves the pair at its `(0, 0)` start until a collision
        // writes it, and never clears it (`docs/COLLISION.md` §4.3).
        if let Some(theirs) = theirs.coll_x.zip(theirs.coll_y)
            && theirs != (0, 0)
            && let sim::orders::Body::Move(m) = ours.body
        {
            let mine = m.coll.map(|p| (p.x, p.y));
            if mine.map(|(x, y)| (i64::from(x), i64::from(y))) != Some(theirs) {
                at(slot, OrderMismatch::Coll { ours: mine, theirs });
            }
        }
        // **The move order's own row**, field for field (§4.1) — every
        // field of the record this crate models. `tolerance`, `retry`,
        // `attempts` and `orig_x/orig_y` are left out because nothing here
        // writes them and `run29_s_move_orders_match_the_field_table_row_
        // for_row` already pins them against the original; `off_x/off_y`
        // are `x mod 0x300` and so are the destination said twice, which
        // is worth having as a check on this crate's own snapping.
        //
        // Until 2026-08-31 the whole row went uncompared and only
        // `coll_x/coll_y` was read, which is how East Indies' `1/4` came
        // to be booked as an order-list *length* at frame 168: the two
        // sides had picked different camp spots on **167**, and nothing
        // looked at the field that said so.
        if let sim::orders::Body::Move(m) = ours.body {
            // `dest_x/dest_y` is the **current waypoint**, and it is live
            // only while `dest` is 1: on arrival the original clears the
            // flag and leaves the pair holding the waypoint it just
            // reached (§4.1), which is state this crate does not carry.
            // So the pair is compared on the frames the flag says it means
            // something, and `dest` itself on every frame.
            let live = m.has_waypoint && theirs.dest == Some(1);
            let last = m.last.unwrap_or(sim::Pos::new(-1, -1));
            for (field, mine, logged) in [
                ("x", i64::from(m.dest.x), theirs.x),
                ("y", i64::from(m.dest.y), theirs.y),
                ("angle", i64::from(m.angle.0), theirs.angle),
                ("dest", i64::from(m.has_waypoint), theirs.dest),
                (
                    "dest_x",
                    i64::from(m.waypoint.x),
                    live.then_some(theirs.dest_x).flatten(),
                ),
                (
                    "dest_y",
                    i64::from(m.waypoint.y),
                    live.then_some(theirs.dest_y).flatten(),
                ),
                ("last_x", i64::from(last.x), theirs.last_x),
                ("last_y", i64::from(last.y), theirs.last_y),
                ("pause", i64::from(m.pause), theirs.pause),
                ("timer", i64::from(m.timer), theirs.timer),
                ("facing", m.facing.map_or(-1, i64::from), theirs.facing),
                ("off_x", i64::from(m.dest.x).rem_euclid(0x300), theirs.off_x),
                ("off_y", i64::from(m.dest.y).rem_euclid(0x300), theirs.off_y),
            ] {
                if let Some(theirs) = logged
                    && theirs != mine
                {
                    at(
                        slot,
                        OrderMismatch::Move {
                            field,
                            ours: mine,
                            theirs,
                        },
                    );
                }
            }
        }
        // **The `GROUPORDER` row**, field for field — the record a
        // formation's order carries and which nothing compared for as
        // long as the kind above it could not agree (item 237). The kind
        // matching already says both sides call this a group order, so
        // both carry the block; `oxx`/`whose` are the leader
        // `GroupData::find_leader` chose, in the log's ids.
        if let sim::orders::Body::Move(m) = ours.body
            && let Some(gm) = m.group
        {
            let leader = built.unit_ids(gm.leader);
            for (field, mine, logged) in [
                ("oxx", leader.map(|(_, o)| o), theirs.oxx),
                ("whose", leader.map(|(w, _)| w), theirs.whose),
                ("id", Some(gm.id), theirs.group_id),
                ("form_id", i64::try_from(gm.form_id).ok(), theirs.form_id),
                (
                    "group_angle",
                    Some(i64::from(gm.group_angle.0)),
                    theirs.group_angle,
                ),
                ("in_group", Some(i64::from(gm.in_group)), theirs.in_group),
            ] {
                // both sides: `logged` None is a detail level that does not
                // print the field; `mine` None is `form_id` past i64 or a
                // leader `unit_ids` cannot name — the second is a quiet
                // disagreement this row does not report, and the target row
                // above compares dead or alive (item 502) for that reason.
                if let (Some(mine), Some(theirs)) = (mine, logged)
                    && mine != theirs
                {
                    at(
                        slot,
                        OrderMismatch::Group {
                            field,
                            ours: mine,
                            theirs,
                        },
                    );
                }
            }
        }
        // **The gather order's own row**, field for field (§6.4). The kind
        // and the target agreeing says only that both sides are working the
        // same camp; the tile, the phase and the countdown are what say they
        // are doing the same thing at it — and the tile is the field whose
        // absence from this comparison hid the tile choice's missing access
        // filter for as long as it existed.
        if let sim::orders::Body::Gather(g) = ours.body {
            let t = g.tile.unwrap_or(sim::Pos::new(-1, -1));
            for (field, mine, logged) in [
                ("tx", i64::from(t.x), theirs.tx),
                ("ty", i64::from(t.y), theirs.ty),
                ("wait", i64::from(g.wait), theirs.wait),
                ("goto_build", i64::from(g.goto_build), theirs.goto_build),
                ("been_there", i64::from(g.been_there), theirs.been_there),
                ("dist_mod", i64::from(g.dist_mod), theirs.dist_mod),
            ] {
                if let Some(theirs) = logged
                    && theirs != mine
                {
                    at(
                        slot,
                        OrderMismatch::Gather {
                            field,
                            ours: mine,
                            theirs,
                        },
                    );
                }
            }
        }
    }

    if unit.path.len() != them.path.len() {
        at(
            0,
            OrderMismatch::PathLength {
                ours: unit.path.len(),
                theirs: them.path.len(),
            },
        );
    }
    for (slot, (ours, theirs)) in unit.path.iter().zip(them.path.iter()).enumerate() {
        let mine = (ours.to.x, ours.to.y);
        if i64::from(mine.0) != theirs.to.0 || i64::from(mine.1) != theirs.to.1 {
            at(
                slot,
                OrderMismatch::PathTo {
                    slot,
                    ours: mine,
                    theirs: theirs.to,
                },
            );
        }
        // The rest of the row. `PATHDATA` prints four numbers and this
        // compared two of them for as long as the stack has been an
        // oracle.
        for (field, mine, logged) in [
            ("tolerance", i64::from(ours.tolerance), theirs.tolerance),
            ("flags", i64::from(ours.flags), theirs.flags),
        ] {
            if mine != logged {
                at(
                    slot,
                    OrderMismatch::PathField {
                        slot,
                        field,
                        ours: mine,
                        theirs: logged,
                    },
                );
            }
        }
    }
    out
}

/// One starting citizen's derived order against the one the original issued.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrderCheck {
    pub who: i64,
    pub o: i64,
    /// The building object number the simulation's derivation chose.
    pub ours: Option<i64>,
    /// The `ox` of the logged `GATHERORDER`, if the unit is holding one.
    pub theirs: Option<i64>,
    /// The logged current order's `OrderIndex` — `-1` for an empty list.
    pub their_kind: i64,
}

impl OrderCheck {
    pub const fn agrees(&self) -> bool {
        match (self.ours, self.theirs) {
            (Some(a), Some(b)) => a == b,
            (None, None) => true,
            _ => false,
        }
    }
}

/// **Derive, then read, then compare** — the check the position diff cannot
/// make.
///
/// [`build_sim`] derives every starting citizen's gather order from
/// `docs/ORDERS.md` §9.3's rule *without looking at the log*. This reads what
/// the original actually issued, out of the first logged frame's `UNITS=3`
/// order blocks, and lines the two up by `(who, o)`.
///
/// It is the honest test of that rule, because **positions cannot show it**: a
/// farm's citizen is placed inside the footprint and has already arrived, so
/// it stands still in both simulations for the whole of a short dump, and a
/// woodcutter's cannot walk at all until `gather_from` arrives (which needs
/// `BUILDS=7`, not the `BUILDS=6` the first reading claimed). Two simulations
/// can agree on every position for 47 frames and still have given every
/// citizen the wrong job.
pub fn check_start_orders(built: &Built, log: &Log<'_>) -> Vec<OrderCheck> {
    let states = log.frame_states();
    let Some(first) = states.first() else {
        return Vec::new();
    };
    let logged = &first.units;
    let o_of = |handle: usize| -> Option<i64> {
        built
            .builds
            .iter()
            .find(|(h, _)| *h == handle)
            .map(|(_, o)| *o)
    };
    built
        .units
        .iter()
        .filter_map(|link| {
            let them = logged.iter().find(|u| u.who == link.who && u.o == link.o)?;
            let ours = built.sim.units[link.unit]
                .orders
                .iter()
                .find_map(|o| match o.body {
                    sim::orders::Body::Gather(g) => Some(g.building),
                    _ => None,
                })
                .and_then(o_of);
            let cur = them.current_order();
            Some(OrderCheck {
                who: link.who,
                o: link.o,
                ours,
                theirs: cur.and_then(|c| {
                    (c.index == i64::from(sim::orders::index::GATHER))
                        .then_some(c.ox)
                        .flatten()
                }),
                their_kind: cur.map_or(-1, |c| c.index),
            })
        })
        .collect()
}

/// A freshly built simulation at frame 0, for [`check_start_orders`] — the
/// same construction [`run`] does, before any frame is stepped.
pub fn build_for_check(loaded: &Loaded, log: &Log<'_>, tuning: Tuning) -> Option<Built> {
    let init = log.initial()?;
    Some(build_sim(loaded, &init, tuning))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::diff::testkit::*;
    use crate::gamelog::{OrderDump, UnitDump};

    use crate::testenv::{dump, install};

    /// **`toff`, against the original's own world chain** (item 30).
    ///
    /// Run20's unit `1/0` walks a `find_wpath` chain the original logs at
    /// `(42744, 37368)`, `(41976, 38136)`, `(41976, 38904)`, … — every one
    /// of them `cell*0x300 + 504` on both axes, where the simulation used
    /// to emit the cell **centre** `+0x180`. `504` is the unit's own move
    /// order's `off_x` (`MoveOrder +0x4c` = `dest % 0x300`), and
    /// `astar_path`'s prologue reads it through `is_move` / `update_move_
    /// order` — the current order's own, with no target involved
    /// (`docs/PATHFINDER.md` §2, §7).
    ///
    /// So the assertion is on the **lattice**: every waypoint the sim puts
    /// on the world grid sits at `+504` and is one the original's stack
    /// also carries. With the old `toff = 0` seam every one of them sits at
    /// `+0x180` instead, no sim entry is on the lattice at all, and the
    /// count below is zero — which is how this was made to fail.
    ///
    /// ~~What it does **not** yet assert is the two ends.~~ The goal end is
    /// closed by item 31 (`docs/GROUPS.md` §6.7): the bottom entry is the
    /// leader's raw slot and the sim now writes the original's own number.
    /// What is left is the **middle** of the route, which is the
    /// pathfinder's and not this mechanic's — see
    /// `run20_s_group_member_is_pathed_at_order_time_off_the_leaders_slot`.
    #[test]
    fn run20_s_world_chain_sits_on_the_move_orders_own_offset() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run20-islands-dumpall.txt") else {
            eprintln!("skipping: no gamelog-run20-islands-dumpall.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial().unwrap();
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.tick();
        built.tick();

        let theirs = log
            .frame_states()
            .into_iter()
            .find(|f| f.n == 2)
            .expect("frame 2")
            .units
            .into_iter()
            .find(|ud| ud.who == 1 && ud.o == 0)
            .expect("1/0 at frame 2");
        // The oracle, pinned: the goal, then eight world nodes, every one
        // of them on the `+504` lattice with the world grid's tolerance.
        let theirs_path: Vec<(i64, i64, i64, i64)> = theirs
            .path
            .iter()
            .map(|p| (p.to.0, p.to.1, p.tolerance, p.flags))
            .collect();
        assert_eq!(
            theirs_path,
            vec![
                (41952, 36576, 0, 1),
                (42744, 37368, 384, 0),
                (41976, 38136, 384, 0),
                (41976, 38904, 384, 0),
                (41208, 39672, 384, 0),
                (40440, 38904, 384, 0),
                (39672, 38904, 384, 0),
                (38904, 38904, 384, 0),
                (38136, 39672, 384, 0),
            ],
            "the original's 1/0 at frame 2"
        );
        // And the offset is the order's own, read straight off the dump.
        let mo = theirs.orders.first().expect("the current order");
        assert_eq!(
            (mo.off_x, mo.off_y),
            (Some(504), Some(504)),
            "run20's `off_x`/`off_y`"
        );
        assert_eq!(
            (mo.x, mo.y),
            (Some(41976), Some(36600)),
            "and they are `x mod 0x300`"
        );
        for e in &theirs_path[1..] {
            assert_eq!((e.0 % 0x300, e.1 % 0x300), (504, 504), "{e:?}");
        }

        let v = built
            .sim
            .units
            .iter()
            .position(|x| x.alive() && x.owner == 1 && x.index == 0)
            .expect("the AI's unit 0");
        let ours: Vec<(i64, i64)> = built.sim.units[v]
            .path
            .iter()
            .map(|p| (i64::from(p.to.x), i64::from(p.to.y)))
            .collect();
        // Item 31, closed: the goal at the bottom is the leader's raw slot
        // — `Group::action_move_near` pushes `form +0x514/+0x714` un-snapped
        // — and not the order's `dest`, which `add_move_facing_order` has
        // put on the `u*0x30 + 0x18` grid `0x18` further out on both axes.
        assert_eq!(ours[0], (41952, 36576), "ours: the leader's raw slot");
        assert_eq!((theirs_path[0].0, theirs_path[0].1), ours[0]);
        let on_lattice: Vec<(i64, i64)> = ours[1..]
            .iter()
            .copied()
            .filter(|(x, y)| x % 0x300 == 504 && y % 0x300 == 504)
            .collect();
        // Not one of them on the cell centre any more — that is the whole
        // of the old seam, and it is what pinning `toff` back to zero
        // restores.
        let on_centre = ours[1..]
            .iter()
            .filter(|(x, y)| x % 0x300 == 0x180 && y % 0x300 == 0x180)
            .count();
        assert_eq!(on_centre, 0, "cell centres are back: {ours:?}");
        assert!(
            on_lattice.len() >= 5,
            "the sim's chain is off the order's lattice: {ours:?}"
        );
        // And they are the original's own entries, exactly — every one of
        // them, since item 32. This check keeps its own floor rather than
        // pinning the chain: what it is *for* is the lattice, and the
        // chain is `run20_s_group_member_is_pathed_at_order_time_off_the_
        // leaders_slot`'s to pin.
        let shared = on_lattice
            .iter()
            .filter(|e| theirs_path.iter().any(|t| (t.0, t.1) == **e))
            .count();
        assert!(
            shared >= 3,
            "shared with the original: {shared} of {ours:?}"
        );
    }

    /// **The path stack's other two columns** (2026-08-31) — `tolerance`
    /// and `flags`, which every `PATHDATA` row prints beside the point and
    /// which nothing compared for the eight days the stack has been an
    /// oracle (`docs/PATHFINDER.md` §10). The working agreement's "diff the
    /// whole record", applied to the record it had been applied to least.
    ///
    /// Two halves. The first is the **census of the original's own
    /// shapes** over run39's whole capture — 1,724 multi-entry stacks —
    /// which is an oracle in its own right and cost one pass: the bottom
    /// is the order's goal every time, and above it there are exactly
    /// three shapes, one of which this crate never writes (see the counts
    /// below). The second is the harness's own disagreement count on the
    /// same capture, as a ceiling.
    ///
    /// `UnitData::tolerance` is read straight off the top entry
    /// (`docs/ORDERS.md` §4.4, the waypoint block), so a wrong tolerance is
    /// not cosmetic: it is the radius at which a unit calls a waypoint
    /// reached.
    #[test]
    fn a_path_stack_s_rows_are_compared_whole() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run39.log"),
        ) else {
            eprintln!("skipping: no East Indies capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);

        // Half one: the original's own shape, read straight off the dump —
        // `(tolerance, flags)` of every multi-entry stack's top and bottom,
        // tallied rather than asserted one by one, because the tally is the
        // finding.
        let mut tops: std::collections::BTreeMap<(i64, i64), usize> = Default::default();
        let mut bottoms: std::collections::BTreeMap<(i64, i64), usize> = Default::default();
        let mut middles: std::collections::BTreeMap<(i64, i64), usize> = Default::default();
        for f in log.frame_states() {
            for u in &f.units {
                if u.path.len() < 2 {
                    continue;
                }
                *bottoms
                    .entry((u.path[0].tolerance, u.path[0].flags))
                    .or_default() += 1;
                let top = &u.path[u.path.len() - 1];
                *tops.entry((top.tolerance, top.flags)).or_default() += 1;
                for e in &u.path[1..u.path.len() - 1] {
                    *middles.entry((e.tolerance, e.flags)).or_default() += 1;
                }
            }
        }
        eprintln!("run39 stacks: tops {tops:?} bottoms {bottoms:?} middles {middles:?}");
        // The bottom is the order's own goal, always — `Unit::do_move`
        // pushes `{mo->x, mo->y, 0, 1}` before it plans (`docs/ORDERS.md`
        // §4.4) and `Group::action_move_near` the leader's raw slot with
        // the same pair (`docs/GROUPS.md` §6.7). 1,724 stacks, no
        // exception.
        assert_eq!(
            bottoms,
            [((0, 1), 1_724)].into_iter().collect(),
            "a stack bottom that is not the order's goal"
        );
        // Above it, three shapes and only three, and the counts are the
        // dump's own so they are exact rather than a ceiling:
        //
        // - `(384, 0)` is `astar_path`'s world reconstruction
        //   (`docs/PATHFINDER.md` §7), and it is what this crate writes;
        // - `(0, 2)` is the **unit grid**'s, `SIDESTEP` and no tolerance —
        //   `find_upath`'s 48-cell plan round a blocker;
        // - `(0, 0)` is a top the collision arm has rewritten: §4.4 pops
        //   the waypoint and pushes it back with `tolerance =
        //   collider.big_radius × 3`, which is **zero** for every
        //   `BLOCK_RADIUS 1` type in the corpus, and leaves the flags —
        //   so a world node the unit has been blocked at reads `(0, 0)`
        //   rather than `(384, 0)`. That rewrite is not modelled here
        //   (`docs/ORDERS.md` §4.4), and these 243 tops are its record.
        assert_eq!(
            tops,
            [((0, 0), 243), ((0, 2), 73), ((384, 0), 1_408)]
                .into_iter()
                .collect(),
            "the stack-top shapes moved"
        );
        assert_eq!(
            middles,
            [((0, 2), 54), ((384, 0), 2_325)].into_iter().collect(),
            "the stack-middle shapes moved"
        );

        // Half two: what this crate writes instead, as a ceiling. Every one
        // of these is a row the old comparison could not see.
        let sib_text = crate::capture::read(&sib);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        let rows: Vec<&OrderDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.order_diverged.iter())
            .filter(|d| matches!(d.what, OrderMismatch::PathField { .. }))
            .collect();
        let first = rows.first().copied().copied();
        eprintln!("run39 path-field rows: {}, first {first:?}", rows.len());
        // 24 rows, the first at frame **1518** — past this capture's score
        // (ticks 1477, orders 1476), and all of them the AI scout `1/0`'s,
        // whose stack has been its own since the order at 1477. Nothing the
        // widening found lies before a parting, which is why no floor moved
        // when it landed.
        assert!(
            rows.len() <= 24,
            "path-field disagreements grew: {} — first {first:?}",
            rows.len()
        );
    }

    /// **§6.7 — the group plans, at order time, off the leader's raw slot**
    /// (item 31, `docs/GROUPS.md` §6.7).
    ///
    /// Run20's `1/0` is a one-member group on auto-explore
    /// (`Sim::scout_issue` → `group_action_move_to`), which makes it the
    /// cheapest possible fixture for this section: with one member the slot
    /// translation is the identity, so what is left is exactly the two
    /// halves the simulation did not have — **when** the path is planned
    /// and **what goal** it is planned to.
    ///
    /// The original's frame **1** already carries `flags 1` (`PATHED`) and
    /// a nine-entry stack whose bottom is `(41952, 36576)` with
    /// `tolerance 0` and `flags 1`; before this landed the simulation's
    /// frame 1 had `flags 0` and an **empty** stack, because every member
    /// got a bare `MoveOrder` and waited for its own `do_move` a frame
    /// later. Both halves are asserted here, and both were made to fail
    /// first — by handing `add_move_facing_order` `pathed = false`, and by
    /// pushing the order's snapped `dest` in place of `form.to[idx]`.
    ///
    /// It asserts the **whole chain**, and it did not always. When this
    /// landed, four of the sim's seven entries were the original's entry
    /// for entry and the middle parted — the original ran along cell row
    /// 50 where the sim ran along row 51. That residue was
    /// `astar_path`'s own, and item 32 closed it the same day by pricing
    /// the two things `calc_cost` was not reading: whether the step's
    /// half-cell is **seen**, and what the cell it lands in **costs**
    /// (`docs/PATHFINDER.md` §5). All nine entries agree now, in order.
    #[test]
    fn run20_s_group_member_is_pathed_at_order_time_off_the_leaders_slot() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run20-islands-dumpall.txt") else {
            eprintln!("skipping: no gamelog-run20-islands-dumpall.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial().unwrap();
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.tick();

        let theirs = log
            .frame_states()
            .into_iter()
            .find(|f| f.n == 1)
            .expect("frame 1")
            .units
            .into_iter()
            .find(|ud| ud.who == 1 && ud.o == 0)
            .expect("1/0 at frame 1");
        let theirs_path: Vec<(i64, i64, i64, i64)> = theirs
            .path
            .iter()
            .map(|p| (p.to.0, p.to.1, p.tolerance, p.flags))
            .collect();
        // The oracle: pathed on the frame the order was issued, with the
        // whole chain already on the stack.
        assert_eq!(
            theirs.orders.first().map(|o| o.flags),
            Some(1),
            "the original's order is PATHED at frame 1"
        );
        assert_eq!(theirs_path.len(), 9, "and its stack is nine deep");
        assert_eq!(
            theirs_path[0],
            (41952, 36576, 0, 1),
            "whose bottom is the leader's raw slot, tolerance and flag"
        );

        let v = built
            .sim
            .units
            .iter()
            .position(|x| x.alive() && x.owner == 1 && x.index == 0)
            .expect("the AI's unit 0");
        // The same position at plan time, so that the two searches are
        // comparable at all: if this drifts, the rest is measuring
        // something else.
        assert_eq!(
            (
                i64::from(built.sim.units[v].pos.x),
                i64::from(built.sim.units[v].pos.y)
            ),
            (theirs.pos.x, theirs.pos.y),
            "the plan frame's position"
        );
        let front = built.sim.units[v].orders.front().expect("the move order");
        assert_eq!(
            front.flags & sim::orders::flag::PATHED,
            sim::orders::flag::PATHED,
            "ours is PATHED at frame 1 too"
        );
        let ours: Vec<(i64, i64, i64, i64)> = built.sim.units[v]
            .path
            .iter()
            .map(|p| {
                (
                    i64::from(p.to.x),
                    i64::from(p.to.y),
                    i64::from(p.tolerance),
                    i64::from(p.flags),
                )
            })
            .collect();
        assert!(!ours.is_empty(), "the group planned nothing");
        assert_eq!(
            ours[0], theirs_path[0],
            "the goal is the leader's raw slot, whole"
        );
        // **No residue left.** Item 32 closed the middle of the chain on
        // 2026-08-26: `calc_cost`'s fog read and its terrain cost are both
        // live (`docs/PATHFINDER.md` §5, §12), and with them the sim plans
        // the original's nine entries — position, tolerance and flag —
        // **in order**. The whole chain is the assertion now; anything
        // that moves one waypoint fails here.
        assert_eq!(ours, theirs_path, "the sim's chain: {ours:?}");
    }

    /// One of the kept recordings, if this machine has it.
    ///
    /// `$RON_RECGAME_DIR`, or the profile's own `Recorded Games` directory —
    /// `PlayerProfile::get_record_game_directory` builds it under
    /// `CSIDL_PERSONAL`, which the Wine prefix maps to the Mac's `~/Documents`.
    fn recording(name: &str) -> Option<String> {
        if let Ok(dir) = std::env::var("RON_RECGAME_DIR") {
            let path = format!("{dir}/{name}");
            return std::path::Path::new(&path).is_file().then_some(path);
        }
        // **The kept corpus first, the game's own output directory second.**
        // A recording is a capture like a gamelog or a trace, so it belongs
        // with them — `dump`'s directory, which is `$RON_GAMELOG_DIR` or
        // `~/ron-data`'s `Logs\`. The original writes new ones to
        // `PlayerProfile::get_record_game_directory`, which the prefix maps
        // to the Mac's `~/Documents`, and that path is **gated by macOS
        // consent**: the first `open` under it blocks until a human at the
        // machine clicks Allow, which over SSH is nobody. A background run
        // then looks hung at 0 % CPU for as long as it is left to. So the
        // fallback stays — a fresh capture is found where the game put it —
        // but it is the fallback.
        dump(name).or_else(|| {
            let home = std::env::var("HOME").unwrap_or_default();
            let path = format!("{home}/Documents/My Games/Rise of Nations/Recorded Games/{name}");
            std::path::Path::new(&path).is_file().then_some(path)
        })
    }

    /// **The paired run** (2026-08-24): one game described by both ground
    /// truths at once — the gamelog's per-frame state and the recording's
    /// command stream — which is what `docs/RECGAME.md` §5 left open and
    /// what `docs/DATALAYER.md` called the diff's missing input.
    ///
    /// This is written to fail if the wiring breaks in either direction: if
    /// the pairing is wrong the frame counts stop matching, and if the
    /// stream stops reaching the simulation the scout goes back to holding
    /// no order at all on the frame the original moved it.
    #[test]
    fn the_recorded_order_stream_drives_the_units_it_names() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run7-ancient-nubian-orders.txt") else {
            eprintln!(
                "skipping: no gamelog-run7-ancient-nubian-orders.txt \
                 (set RON_GAMELOG_DIR; docs/ORACLE.md says how to capture one)"
            );
            return;
        };
        let Some(rc) = recording("Playback - 2026.08.24 10'15'53 (Mon).rcx") else {
            eprintln!(
                "skipping: no run7 recording (set RON_RECGAME_DIR; \
                 docs/RECGAME.md says where the game writes them)"
            );
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let data = crate::recgame::decompress(&rc).unwrap();
        let rec = crate::recgame::parse(&data, &rc).unwrap();

        // The pairing itself. A recording holds one package a frame, so a
        // recording and a dump of the *same* run have equal counts; this is
        // the cheapest possible guard against diffing two different games.
        assert_eq!(
            rec.packages.len(),
            log.frame_states().len(),
            "the recording and the dump are not the same run"
        );
        assert_eq!(rec.packages.len(), 1732, "run7's frame count");
        assert_eq!(rec.seed, 12345, "run7's fixed seed");

        // The input, and what of it the harness can act on today. Both are
        // ceilings in opposite directions: the stream never carries fewer
        // commands, and the number it cannot act on only ever falls as
        // mechanics land.
        let mut stream = crate::input::Stream::new(&rec);
        assert_eq!(stream.len(), 46, "run7's input commands");
        let report = run_with(&loaded, &log, Tuning::RON, None, Some(&mut stream)).unwrap();
        assert_eq!(
            report.applied.orders, 9,
            "the nine move orders the stream names"
        );
        assert!(
            report.applied.skipped_total() <= 22,
            "commands the harness ignores grew to {}: this only ever shrinks",
            report.applied.skipped_total()
        );

        // **The check with teeth.** Unit 0/0 is the scout, and the original
        // moves it on frame 477. Un-fed, the harness has no order there at
        // all and the disagreement is `Length { ours: 0, theirs: 1 }`; fed,
        // the order exists and matches in kind, so whatever remains is
        // path-level. If the stream stops arriving, this reverts.
        let scout: Vec<_> = report
            .frames
            .iter()
            .flat_map(|f| f.order_diverged.iter())
            .filter(|d| d.who == 0 && d.o == 0)
            .collect();
        assert!(
            !scout
                .iter()
                .any(|d| matches!(d.what, OrderMismatch::Length { ours: 0, .. })),
            "the scout holds no order where the original moved it: the \
             stream is not reaching the simulation"
        );
    }

    /// A unit holding a build order on the building the log calls `2001`,
    /// with a transit move in front of it — the shape §11.1's worked block
    /// has, and the one that catches a list read the wrong way round.
    fn ordered_citizen(loaded: &crate::load::Loaded) -> (Built, usize, Vec<OrderDump>) {
        let init = initial();
        let mut built = build_sim(loaded, &init, Tuning::RON);
        let u = built.units[1].unit;
        let b = built.sim.add_building(0, Pos::new(4248, 28680), 8);
        built.builds.push((b, 2001));
        built
            .sim
            .add_build_order(u, b, sim::orders::QueuePos::New, true);
        built.sim.add_move_order(
            u,
            Pos::new(4300, 28700),
            sim::orders::MoveKind::ExploreTo,
            sim::orders::QueuePos::First,
            false,
        );
        // Newest first, as `OrderList::log_data` writes it: the build order
        // was given first and prints first, the transit move it is walking
        // now prints last.
        let logged = vec![
            OrderDump {
                index: i64::from(sim::orders::index::BUILD_AT),
                kind: "BUILDORDER".into(),
                flags: 4,
                ox: Some(2001),
                whom: Some(0),
                ..OrderDump::default()
            },
            OrderDump {
                index: i64::from(sim::orders::index::EXPLORE_TO),
                kind: "EXPLORETOORDER".into(),
                flags: built.sim.units[u].orders[0].flags.into(),
                ..OrderDump::default()
            },
        ];
        (built, u, logged)
    }

    #[test]
    fn two_order_lists_that_agree_are_walked_front_first_and_report_nothing() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        let (built, _, logged) = ordered_citizen(&loaded);
        let them = UnitDump {
            who: 0,
            o: 1,
            orders: logged,
            ..UnitDump::default()
        };
        let d = compare_orders(&built, &built.units[1].clone(), &them, 1);
        assert!(d.is_empty(), "{d:?}");
        // Read the other way round it is two kind disagreements — which is
        // the whole risk in a list the log writes newest first.
        let mut backwards = them.clone();
        backwards.orders.reverse();
        let d = compare_orders(&built, &built.units[1].clone(), &backwards, 1);
        assert_eq!(d.len(), 2);
        assert!(
            d.iter()
                .all(|d| matches!(d.what, OrderMismatch::Kind { .. }))
        );
    }

    #[test]
    fn each_field_of_an_order_is_reported_on_its_own() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        let (built, _, logged) = ordered_citizen(&loaded);
        let link = built.units[1];
        let one = |orders: Vec<OrderDump>| {
            compare_orders(
                &built,
                &link,
                &UnitDump {
                    who: 0,
                    o: 1,
                    orders,
                    ..UnitDump::default()
                },
                1,
            )
        };

        // A shorter list: the length, and then the slots that do line up.
        let short = one(logged[..1].to_vec());
        assert!(matches!(
            short[0].what,
            OrderMismatch::Length { ours: 2, theirs: 1 }
        ));

        // A different target on the build order — slot 1, front first.
        let mut other = logged.clone();
        other[0].ox = Some(2002);
        let d = one(other);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].slot, 1);
        assert!(matches!(
            d[0].what,
            OrderMismatch::Target {
                ours: Some((0, 2001)),
                theirs: Some((0, 2002))
            }
        ));

        // The action bit alone, which is what says "intent" rather than
        // "transit leg" — and it takes the flags byte with it.
        let mut unset = logged.clone();
        unset[0].flags = 0;
        let d = one(unset);
        assert_eq!(d.len(), 2);
        assert!(matches!(
            d[0].what,
            OrderMismatch::Action {
                ours: true,
                theirs: false
            }
        ));
        assert!(matches!(d[1].what, OrderMismatch::Flags { .. }));
        assert!(!d[1].what.scores(), "a flags byte does not score");
    }

    #[test]
    fn the_path_stack_is_compared_bottom_first() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        let (mut built, u, logged) = ordered_citizen(&loaded);
        built.sim.units[u].path = vec![
            sim::orders::PathData {
                to: Pos::new(45024, 19680),
                tolerance: 0,
                flags: 1,
            },
            sim::orders::PathData {
                to: Pos::new(45048, 18168),
                tolerance: 384,
                flags: 0,
            },
        ];
        let link = built.units[1];
        let them = |path: Vec<crate::gamelog::PathDump>| UnitDump {
            who: 0,
            o: 1,
            orders: logged.clone(),
            path,
            ..UnitDump::default()
        };
        // The log writes the goal first and so does the `Vec`, so equal
        // stacks agree without either being reversed.
        let same = vec![
            crate::gamelog::PathDump {
                to: (45024, 19680),
                tolerance: 0,
                flags: 1,
            },
            crate::gamelog::PathDump {
                to: (45048, 18168),
                tolerance: 384,
                flags: 0,
            },
        ];
        assert!(compare_orders(&built, &link, &them(same.clone()), 1).is_empty());
        // The stub's shape: we keep only the goal, the original had waypoints.
        let d = compare_orders(&built, &link, &them(same[..1].to_vec()), 1);
        assert_eq!(d.len(), 1);
        assert!(d[0].what.is_path());
        assert!(matches!(
            d[0].what,
            OrderMismatch::PathLength { ours: 2, theirs: 1 }
        ));
    }

    /// **The animal's hurry, re-derived from the original's own record.**
    ///
    /// `AnimalData::get_speed@005d8380` is `Animal`'s and `AnimalData`'s
    /// slot `+0x17c` — the virtual `do_move` and `find_path` both take the
    /// step length from — and it does not call `UnitData::get_speed` at
    /// all. It takes `UnitData::speed`, and then, on a land or sea animal
    /// whose current order `is_move` (slot `+0x14`, named from the PDB's
    /// `LF_ONEMETHOD` list, not from the map's COMDAT-folded stub),
    /// measures `vector_dist` from the animal to that order's **goal** —
    /// `get_move_order` (slot `+0xb8`), then `MoveOrder +0x4/+0x8`, the
    /// ordered point rather than `dest_x/dest_y`, the current waypoint —
    /// and multiplies the speed by **3/2** when it is more than `0x180`.
    /// Then a floor of 3. An air animal returns before both.
    ///
    /// This is checked against the record rather than the listing.
    /// run39's gaia `8/2` walks nine frames from `(28776, 24360)` toward
    /// `(28968, 23976)`; the dump prints `myspeed 19` on every one of
    /// them, so the cached base never moves, and the steps it prints are
    /// **`(12, −25)` twice and then `(8, −17)`, `(8, −16)` ×6** — a step
    /// of 28 while `vector_dist` is 432 and 404, and of 19 once it is 376.
    /// `19 * 3 / 2 = 28`, truncated, and there is no free parameter in
    /// that.
    ///
    /// So the assertion is the whole prediction: for every gaia
    /// unit-frame in the capture on which the animal moved, the step
    /// `Unit::move_step` would take at that speed, and nothing else. The
    /// same walk without the 3/2 is checked to *fail*, because a rule
    /// nothing can break is not a rule.
    #[test]
    fn an_animal_more_than_0x180_from_its_order_hurries_by_three_halves() {
        use sim::movement::{cos_component, find_angle, sin_component};
        use sim::world::vector_dist;
        // **Both maps.** The rule was found on East Indies and is checked
        // on Great Lakes too, where the herd's `myspeed` is 11 rather than
        // 19 — so the `3/2` is exercised against two different bases and
        // cannot be a coincidence of one animal's arithmetic.
        let files = [
            "gamelog-run39-islands-longtrace.txt",
            "gamelog-run33-longtrace.txt",
        ];
        let Some(paths) = files
            .iter()
            .map(|f| dump(f))
            .collect::<Option<Vec<String>>>()
        else {
            eprintln!("skipping: no long-trace captures (set RON_GAMELOG_DIR)");
            return;
        };
        let texts: Vec<String> = paths.iter().map(crate::capture::read).collect();
        let captures: Vec<Vec<Frame>> =
            texts.iter().map(|t| Log::parse(t).frame_states()).collect();

        // One dumped step: where it was, where its order sends it, the
        // cached speed `myspeed`, and where it ended up.
        struct Step {
            capture: &'static str,
            frame: i64,
            who: i64,
            o: i64,
            from: (i64, i64),
            goal: (i64, i64),
            base: i64,
            to: (i64, i64),
        }
        let mut steps: Vec<Step> = Vec::new();
        for (c, frames) in captures.iter().enumerate() {
            for w in frames.windows(2) {
                for u in w[0].units.iter().filter(|u| u.who >= 8) {
                    let Some(next) = w[1].units.iter().find(|n| n.who == u.who && n.o == u.o)
                    else {
                        continue;
                    };
                    if (next.pos.x, next.pos.y) == (u.pos.x, u.pos.y) {
                        continue;
                    }
                    let (Some(gx), Some(gy), Some(base)) = (u.orders_x, u.orders_y, u.myspeed)
                    else {
                        continue;
                    };
                    steps.push(Step {
                        capture: files[c],
                        frame: w[0].n,
                        who: u.who,
                        o: u.o,
                        from: (u.pos.x, u.pos.y),
                        goal: (gx, gy),
                        base,
                        to: (next.pos.x, next.pos.y),
                    });
                }
            }
        }
        assert!(
            steps.len() >= 9,
            "the gaia walks: {} steps, wanted at least the nine `8/2` takes",
            steps.len()
        );

        // The prediction, with the hurry and without it.
        let walk = |s: &Step, boost: bool| -> (i64, i64) {
            let (dx, dy) = (s.goal.0 - s.from.0, s.goal.1 - s.from.1);
            let far = vector_dist(dx as i32, dy as i32) > 0x180;
            let mut speed = s.base as i32;
            if boost && far {
                speed = speed * 3 / 2;
            }
            let speed = speed.max(3);
            if dx.abs() + dy.abs() <= i64::from(speed) {
                return s.goal;
            }
            let a = find_angle(dx as i32, dy as i32);
            (
                s.from.0 + i64::from(sin_component(a, speed)),
                s.from.1 - i64::from(cos_component(a, speed)),
            )
        };

        let wrong: Vec<String> = steps
            .iter()
            .filter(|s| walk(s, true) != s.to)
            .map(|s| {
                format!(
                    "{} frame {} {}/{}: from {:?} goal {:?} base {} — ours {:?} theirs {:?}",
                    s.capture,
                    s.frame,
                    s.who,
                    s.o,
                    s.from,
                    s.goal,
                    s.base,
                    walk(s, true),
                    s.to
                )
            })
            .collect();
        assert!(
            wrong.is_empty(),
            "{} of {} gaia steps are not the rule's:\n{}",
            wrong.len(),
            steps.len(),
            wrong.join("\n")
        );

        let far = steps
            .iter()
            .filter(|s| {
                vector_dist((s.goal.0 - s.from.0) as i32, (s.goal.1 - s.from.1) as i32) > 0x180
            })
            .count();
        eprintln!(
            "the two long traces: {} gaia steps, {far} of them beyond 0x180",
            steps.len()
        );
        // And the same walk with the 3/2 taken out, which must break: the
        // frames on which an animal is beyond `0x180` are the ones the
        // hurry is for.
        let unboosted = steps.iter().filter(|s| walk(s, false) != s.to).count();
        assert_eq!(
            (steps.len(), far, unboosted),
            (264, 39, 39),
            "the two captures' gaia steps, the ones beyond `0x180`, and the \
             ones the rule's absence would get wrong"
        );
    }

    /// **A blocked animal drops its walk where it stands.**
    ///
    /// `Unit::resolve_unit_collision@005f9d30`'s first statement is a
    /// virtual on slot `+0x30` — the PDB's `LF_ONEMETHOD` list names it
    /// `SubObjectData::is_animal` at vftable offset 48, which the map
    /// cannot, because both overrides are COMDAT-folded onto trivial
    /// stubs (`Buffer::is_pending_load`, `return 1`, in `Animal`'s
    /// vtable; `Window::get_button`, `return 0`, in `Unit`'s). When it
    /// answers, the body is the `QUEUE_NEW` clear and nothing else —
    /// `unit_masks &= ~0x4000000`, `path.length = 0`, `close_orders`,
    /// `clear_partial_path`, `update_action` — and **none of
    /// `docs/COLLISION.md` §6's six steps runs**. No sidestep, no wait,
    /// no repath, and above all no cell-centre snap.
    ///
    /// The snap is what the record can see. §6 step 6 moves every other
    /// unit onto the middle of its 48-cell before it re-plans, so a
    /// simulation that lets an animal through to step 6 moves it on the
    /// frame the collision lands; the original does not move it at all.
    ///
    /// So the assertion is over every animal walk in both long captures
    /// that **ends short of its goal**: `orders_x/orders_y` stop naming a
    /// point and start naming the animal's own position, while the animal
    /// is not standing on the goal. Sixteen of those — twelve on East
    /// Indies, four on Great Lakes — and on every one of them the
    /// position is **unchanged** across the frame the order dies. Before
    /// this rule was in, `crates/sim` moved the animal on all sixteen.
    ///
    /// The other half of the same window is the arrival, which is not a
    /// collision: seventeen walks end *on* their goal, and they are
    /// counted here so that a parse which stopped seeing orders would
    /// fail rather than pass with nothing to check.
    #[test]
    fn a_blocked_animal_drops_its_walk_where_it_stands() {
        let files = [
            "gamelog-run39-islands-longtrace.txt",
            "gamelog-run33-longtrace.txt",
        ];
        let Some(paths) = files
            .iter()
            .map(|f| dump(f))
            .collect::<Option<Vec<String>>>()
        else {
            eprintln!("skipping: no long-trace captures (set RON_GAMELOG_DIR)");
            return;
        };
        let texts: Vec<String> = paths.iter().map(crate::capture::read).collect();
        let captures: Vec<Vec<Frame>> =
            texts.iter().map(|t| Log::parse(t).frame_states()).collect();

        let mut reached = 0usize;
        let mut abandoned = 0usize;
        let mut moved: Vec<String> = Vec::new();
        for (c, frames) in captures.iter().enumerate() {
            for w in frames.windows(2) {
                for u in w[0].units.iter().filter(|u| u.who >= 8) {
                    let (Some(gx), Some(gy)) = (u.orders_x, u.orders_y) else {
                        continue;
                    };
                    // A live goal: the order names somewhere else.
                    if (gx, gy) == (u.pos.x, u.pos.y) {
                        continue;
                    }
                    let Some(next) = w[1].units.iter().find(|n| n.who == u.who && n.o == u.o)
                    else {
                        continue;
                    };
                    let (Some(nx), Some(ny)) = (next.orders_x, next.orders_y) else {
                        continue;
                    };
                    // The goal is gone: the order list is empty and
                    // `orders_x/y` name the animal itself again.
                    if (nx, ny) != (next.pos.x, next.pos.y) {
                        continue;
                    }
                    if (next.pos.x, next.pos.y) == (gx, gy) {
                        reached += 1;
                        continue;
                    }
                    abandoned += 1;
                    if (next.pos.x, next.pos.y) != (u.pos.x, u.pos.y) {
                        moved.push(format!(
                            "{} frame {} {}/{}: {:?} → {:?}, goal {:?}",
                            files[c],
                            w[0].n,
                            u.who,
                            u.o,
                            (u.pos.x, u.pos.y),
                            (next.pos.x, next.pos.y),
                            (gx, gy),
                        ));
                    }
                }
            }
        }
        eprintln!(
            "the two long traces: {reached} animal walks reached their goal, {abandoned} were dropped short of it"
        );
        assert!(
            moved.is_empty(),
            "{} of {abandoned} dropped animal walks moved the animal — §6 step 6's \
             cell-centre snap, which an animal never takes:\n{}",
            moved.len(),
            moved.join("\n")
        );
        assert_eq!(
            (reached, abandoned),
            (17, 16),
            "the two captures' animal walks that arrived and that were dropped short"
        );
    }

    /// **The chopping guy's own wait, and the two sites that look like one
    /// branch** (2026-08-31, item 101) — the mechanic behind East Indies'
    /// word 645 → 742.
    ///
    /// `Unit::do_non_flat_gather` reads guy 0's `cur_anim` **before** it
    /// reads the tile (`005f0d0f`), and the branch it takes there is where
    /// a woodcutter spends the rest of its life. `CHAR_CHOP_WOOD`
    /// decrements the wait and on zero rerolls it `% 100 + 300` at
    /// `+0xcc3`; the *arrival* frame — the one that sets `CHAR_CHOP_WOOD`
    /// — rerolls `% 50 + 100` at `+0xdad`; and `CHAR_MINE_ORE` returns
    /// without doing anything at all. `docs/ORDERS.md` §6.4 has carried
    /// all three since August and the implementation had one merged
    /// branch, rolling the arrival's formula at the chop site.
    ///
    /// **A count could not see it.** Both sites draw exactly once, so the
    /// word stayed matched for six hundred frames while the AI's
    /// woodcutter ran its clock at a third of the original's and walked
    /// home two hundred frames early. What sees it is the record.
    ///
    /// Two halves, each of which fails on its own:
    ///
    /// - **The original's own.** Every rise in a dumped `GATHERORDER`'s
    ///   `wait` over run39's 1,850 frames is matched against the draw the
    ///   trace took on that frame, and the value the LCG returned must
    ///   produce it under *that site's* formula. This half needs no
    ///   simulation and is what names the sites; with the two formulas
    ///   swapped it fails on the first reroll.
    /// - **Ours.** Every non-flat `GATHERORDER` the dump prints — tile,
    ///   wait, phase, `been_there`, `dist_mod` — against the simulation's
    ///   own, on every frame to the floor.
    #[test]
    fn run39_s_woodcutters_reroll_on_the_chop_branch_not_the_arrival_s() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            dump("rontrace-run39.log").map(|p| {
                crate::trace::Trace::read(std::path::Path::new(&p))
                    .expect("invalid finalized trace")
                    .expect("missing RONT header")
            }),
        ) else {
            eprintln!("skipping: no East Indies capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        borrow_pasture(&mut init, &tr);
        let frames = log.frame_states();

        // The three sites, taken **through the naming table** rather than
        // written down again, so a label put on the wrong address fails
        // here and not silently three mechanics later.
        let site_of = |label: &str| -> u32 {
            crate::trace::SITES
                .iter()
                .find(|(_, _, l)| *l == label)
                .map(|(a, _, _)| *a)
                .expect("the wood machine's sites are named")
        };
        let tile = site_of(sim::orders::SITE_TILE_WAIT); // `+0x54b`
        let chop = site_of(sim::orders::SITE_WORK_WAIT); // `+0xcc3`
        let arrive = site_of(sim::orders::SITE_ARRIVE_WAIT); // `+0xdad`
        let produced = |site: u32, v: i32| -> Vec<i64> {
            if site == tile {
                // A miner's tile choice draws and then overwrites the roll
                // with the 1,000,000 that keeps it out (§6.4).
                vec![i64::from(400 + v % 200), 1_000_000]
            } else if site == chop {
                vec![i64::from(300 + v % 100)]
            } else {
                vec![i64::from(100 + v % 50)]
            }
        };

        // **Half one: the original against itself.** A `wait` that rises
        // to 100 or more is a reroll — the branch constants (32 at the
        // camp walk, 20 when no spot is free) are all below it — so every
        // one of them owes a draw at one of the three sites on the frame
        // it happened, and the draw's own value must produce it.
        let mut prev: std::collections::BTreeMap<(i64, i64), i64> =
            std::collections::BTreeMap::new();
        let (mut rerolls, mut by_site) = (0usize, std::collections::BTreeMap::new());
        let mut camps: std::collections::BTreeSet<i64> = std::collections::BTreeSet::new();
        for f in &frames {
            // `FRAME n` is the state at the end of frame n, so a rise
            // between `n − 1` and `n` was written by frame `n − 1`'s step,
            // which is the frame the trace counts.
            let step = f.n - 1;
            let mut rose: Vec<i64> = Vec::new();
            for u in &f.units {
                for o in &u.orders {
                    if o.kind != "GATHERORDER" || o.non_flat_gather != Some(1) {
                        continue;
                    }
                    let Some(wait) = o.wait else { continue };
                    camps.extend(o.build_type);
                    let was = prev.insert((u.who, u.o), wait);
                    if was.is_some_and(|w| wait > w) && wait >= 100 {
                        rose.push(wait);
                    }
                }
            }
            if rose.is_empty() {
                continue;
            }
            let mut spent: Vec<(u32, i32)> = tr
                .draws
                .iter()
                .filter(|d| {
                    d.sync()
                        && d.frame == step
                        && (d.site == tile || d.site == chop || d.site == arrive)
                })
                .filter_map(|d| d.value().map(|v| (d.site, v)))
                .collect();
            for w in rose {
                let at = spent
                    .iter()
                    .position(|&(site, v)| produced(site, v).contains(&w))
                    .unwrap_or_else(|| {
                        panic!(
                            "frame {step}: a wait rose to {w} and no gather draw of \
                             {spent:?} produces it"
                        )
                    });
                *by_site.entry(spent[at].0).or_insert(0usize) += 1;
                spent.remove(at);
                rerolls += 1;
            }
            assert!(
                spent.is_empty(),
                "frame {step}: {spent:?} drew and no order's wait rose"
            );
        }
        eprintln!("run39 rerolls: {rerolls} over 1,850 frames, by site {by_site:?}");
        // Nineteen tile choices and **five** chop rerolls, and not one
        // arrival reroll in 1,850 frames: the branch the implementation
        // used to spend every reroll on is the one the original reaches
        // essentially never, which is why the sites had to be told apart
        // by their formulas rather than by their counts.
        assert_eq!(rerolls, 24, "run39's rerolls, all of them explained");
        // And what the capture cannot speak to: every non-flat gatherer in
        // it works the **same** camp type, so `CHAR_MINE_ORE`'s early
        // return and the miner's 1,000,000 are reading-only until a
        // capture works a mine (`docs/SYNC.md` §7).
        assert_eq!(
            camps.into_iter().collect::<Vec<_>>(),
            vec![418],
            "run39 has one non-flat camp type and no mine"
        );
        assert_eq!(
            by_site.into_iter().collect::<Vec<_>>(),
            vec![(tile, 19usize), (chop, 5usize)],
            "the tile choice, the chopping guy, and no arrival roll at all"
        );

        // **Half two: ours against the record.** Field for field, on every
        // frame, for as far as it holds.
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        let mut last = 0i64;
        let (mut compared, mut first_bad, mut parted) = (0usize, None, 0i64);
        for f in &frames {
            while last < f.n {
                built.tick();
                last += 1;
            }
            for u in &f.units {
                let Some(link) = built.units.iter().find(|l| l.who == u.who && l.o == u.o) else {
                    continue;
                };
                let theirs: Vec<&crate::gamelog::OrderDump> = u
                    .orders
                    .iter()
                    .filter(|o| o.kind == "GATHERORDER" && o.non_flat_gather == Some(1))
                    .collect();
                let ours: Vec<sim::orders::GatherOrder> = built.sim.units[link.unit]
                    .orders
                    .iter()
                    .filter_map(|o| match o.body {
                        sim::orders::Body::Gather(g)
                            if matches!(
                                built.sim.building_ident(g.building),
                                sim::build::Ident::Mine | sim::build::Ident::Woodcutter
                            ) =>
                        {
                            Some(g)
                        }
                        _ => None,
                    })
                    .collect();
                if theirs.len() != ours.len() {
                    if first_bad.is_none() {
                        first_bad = Some(format!(
                            "frame {}: {}/{} has {} non-flat gather orders, the original {}",
                            f.n,
                            u.who,
                            u.o,
                            ours.len(),
                            theirs.len()
                        ));
                    }
                    continue;
                }
                for (t, o) in theirs.iter().zip(&ours) {
                    let mine = (
                        o.tile.map_or(-1, |p| i64::from(p.x)),
                        o.tile.map_or(-1, |p| i64::from(p.y)),
                        i64::from(o.wait),
                        i64::from(o.goto_build),
                        i64::from(o.been_there),
                        i64::from(o.dist_mod),
                    );
                    let his = (
                        t.tx.unwrap_or(-1),
                        t.ty.unwrap_or(-1),
                        t.wait.unwrap_or(0),
                        t.goto_build.unwrap_or(0),
                        t.been_there.unwrap_or(0),
                        t.dist_mod.unwrap_or(0),
                    );
                    compared += 6;
                    if mine != his && first_bad.is_none() {
                        first_bad = Some(format!(
                            "frame {}: {}/{} gather ours {mine:?} theirs {his:?}",
                            f.n, u.who, u.o
                        ));
                    }
                }
            }
            if first_bad.is_some() {
                parted = f.n;
                break;
            }
        }
        eprintln!(
            "run39 gather: {compared} fields to frame {parted}, first {}",
            first_bad.as_deref().unwrap_or("none disagreeing")
        );
        // The floor, and it may only rise. Frame 897 used to be the
        // successor — the human's `1/2` holding its tile with a wait ten
        // short of the original's — and item 106 walked straight past it:
        // the scout's walk to a goody box put the stream back on the
        // original's, and the same clock now runs true through 1,572.
        // 1,573 was the successor after that, and item 109's bird step
        // walked past it too: 24,738 fields to 1,573 → **26,094 to
        // 1,686**, where the human's `0/2` holds `(32, 26)` with a wait of
        // 445 against the original's 480 — the same clock, the same
        // shape, thirty-five short rather than a hundred long.
        // `docs/ORDERS.md` §6.4.
        assert!(
            compared >= 26_094 && parted >= 1_686,
            "the wood machine's record fell: {compared} fields to frame {parted}, \
             the floor is 26,094 and 1,686 — {}",
            first_bad.as_deref().unwrap_or("none disagreeing")
        );
    }
}
