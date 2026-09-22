//! What a comparison yields: `FrameResult`, `Report`, and the draw arithmetic.

use super::*;

/// The outcome of one frame's comparison.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FrameResult {
    pub frame: i64,
    /// Units the log has for a player and the simulation does not.
    pub unlinked: usize,
    /// The `(who, o)` of each unlinked unit-frame — the units the original
    /// has on this frame that the simulation does not.
    pub unlinked_units: Vec<(i64, i64)>,
    /// The `(who, o)` of each unit the **simulation** has on this frame and
    /// the original does not — [`unlinked`](Self::unlinked_units)' mirror.
    ///
    /// Counting one side only is a measure that can be *gamed by
    /// over-producing*: a simulation whose AI trains its ninth citizen four
    /// hundred frames early reads as "every unit the original has, we have"
    /// from the frame the original catches up, and the early frames cost it
    /// nothing. Both directions are counted so that neither running ahead
    /// nor running behind can hide.
    pub extra_units: Vec<(i64, i64)>,
    pub compared: usize,
    pub diverged: Vec<Divergence>,
    /// The leaders' scores as logged, by `who`.
    pub scores: Vec<(i64, i64)>,
    /// Units whose order list the log carries, so both sides could be
    /// compared — zero below `UNITS=3`.
    pub order_compared: usize,
    /// Every order-list and path-stack disagreement this frame.
    pub order_diverged: Vec<OrderDivergence>,
    /// Unit-frames whose `ObjectData::mylos` the log carried, so
    /// [`sim::Sim::unit_los`] could be checked against it.
    pub los_compared: usize,
    /// Every one that disagreed — `docs/VISION.md` §2. The dump writes
    /// `mylos` at every detail level, so this is compared on every capture.
    pub los_diverged: Vec<LosDivergence>,
    /// Unit-frames whose `UnitData::unit_masks` the log carried, so the
    /// **packed** bit (`0x80000`) could be checked against
    /// [`sim::combat::Combat::packed`] — `docs/ORDERS.md` §6.9. Written at
    /// every detail level, like `mylos`, so it too is on every capture.
    pub packed_compared: usize,
    /// Every unit-frame whose packed bit disagreed: `(frame, who, o, ours)`.
    pub packed_diverged: Vec<PackedDivergence>,
    /// **`ObjectData::visible` (`+0x40`)** — the byte of players this
    /// object has made itself visible to by attacking them
    /// (`docs/VISION.md` §7). Written inside the `OBJECT` block at every
    /// detail level, so it is compared on every capture that prints a unit
    /// record at all.
    ///
    /// Compared **both ways and ungated on the position**, unlike the
    /// collision and angle blocks. Those are gated because a unit standing
    /// somewhere else collides with different things *as a consequence*,
    /// and counting that would measure the position gap twice. This one is
    /// not a consequence of standing anywhere: it is a record of who the
    /// unit has shot at, and a crate that shoots the wrong people has a
    /// defect worth naming wherever the unit happens to be.
    pub visible_compared: usize,
    pub visible_diverged: Vec<VisibleDivergence>,
    /// **The hit-point record, whole** — `ObjectData::myhits`, `damage`
    /// and `damage_frac`, on every unit-frame the dump carries them for,
    /// which is every one at every detail level (`docs/COMBAT.md` §40).
    ///
    /// Nothing here was compared until item 484. The dump has printed the
    /// three on every block of every capture since the reader existed and
    /// `run100_s_word_block_is_every_record_the_dump_carries` has carried
    /// two of them since item 464 — but the golden-record comparator had
    /// no hit-point row at all, so chapter two's widening could see that
    /// the original's hoplite `1/8` was **gone** on block 684 and not
    /// that it had been wounded on 656, 657, 660 and 682 to get there
    /// (`docs/COMBAT.md` §38.4).
    ///
    /// Compared **both ways and ungated on the position**, for
    /// [`visible_compared`](Self::visible_compared)' reason: what a unit
    /// has taken is not a consequence of where it is standing, and a
    /// crate that takes the wrong damage has a defect worth naming
    /// wherever the unit happens to be.
    pub hits_compared: usize,
    pub hits_diverged: Vec<HitsDivergence>,
    /// Angle comparisons made this frame — `UnitData::angle` for every unit
    /// record, and guy 0's `angle` for every record that carries a guy
    /// (`GUYS` at 1 or above). Two per unit-frame where both are present.
    pub angle_compared: usize,
    /// Every heading or facing that disagreed (`docs/MOVEMENT.md`).
    pub angle_diverged: Vec<AngleDivergence>,
    /// Collision-block fields compared this frame, and the ones that
    /// disagreed (`docs/COLLISION.md` §8).
    pub collide_compared: usize,
    pub collide_diverged: Vec<CollideDivergence>,
    /// **`UnitData::start_dist` (`+0x130`)**, compared on every unit-frame
    /// the dump carries it for — which is every one, at every detail level
    /// (`docs/PATHFINDER.md` §18.6). A non-zero value is the original
    /// saying a 48-grid search suspended on that unit and never came back
    /// to the goal; the field is written by `astar_path`'s suspend block
    /// alone and cleared only at a unit's birth, so it dates the suspend
    /// and then stands for the rest of that unit's life.
    pub search_compared: usize,
    pub search_diverged: Vec<SearchDivergence>,
    /// **The three gated blocks on the unit-frames whose position parted**
    /// — the collision block, `start_dist` and the two angles, compared
    /// exactly as above but filed here instead of counted (parked 453,
    /// item 448). The gate on the position is right for a residue count:
    /// a unit standing somewhere else collides with different things as a
    /// consequence. It is blind on exactly the frame a position parts,
    /// which is the frame a word is read on — `half_step` is dumped on
    /// every block of every capture and was unreadable at 10161 for this
    /// reason alone. Nothing here scores; a widening reads it beside
    /// [`diverged`](Self::diverged) so the value diff at the parting is
    /// on the record.
    pub collide_parted: Vec<CollideDivergence>,
    pub search_parted: Vec<SearchDivergence>,
    pub angle_parted: Vec<AngleDivergence>,
    /// Gather-record fields compared this frame, and the ones that
    /// disagreed. `BUILDS=7` is what writes the mining list; below it only
    /// `gather_down` is compared, and on a capture with no `BUILDDATA` at
    /// all the count is zero.
    pub gather_compared: usize,
    pub gather_diverged: Vec<GatherDivergence>,
    /// Buildings the frame names for a player and the simulation does not
    /// hold — the gather comparison's own blind spot, counted rather than
    /// assumed away.
    pub build_unlinked: usize,
    /// A building's own `x_internal`/`y_internal` and `orig_type`, compared
    /// on every linked building of every frame — the half of `BUILDDATA`
    /// that says *where the AI put it*, which nothing compared until the
    /// word reached 3021.
    pub build_compared: usize,
    pub build_diverged: Vec<BuildDivergence>,
    /// **The production queue, whole** — `queued` and, for each live slot,
    /// its `type`, `job_counter` and the three `(good, cost)` pairs
    /// (`docs/PRODUCTION.md`, "The queue record"). Written from
    /// `BUILDS=1`; a capture below it compares nothing here.
    ///
    /// Only the first `queued` slots are read: the tail of the array holds
    /// whatever it was last left with — `type −1` on a queue never used,
    /// `type 0` on one that has been — which is not state either side owns.
    pub queue_compared: usize,
    pub queue_diverged: Vec<QueueDivergence>,
    /// **The `CITY` record, whole** — every field
    /// `CityData::log_data@004895c0` writes, on every city of every frame
    /// (`docs/CITIES.md` §5.7). Written at every detail level, like `mylos`
    /// and the packed bit, so this is compared on every capture that dumps
    /// frames at all.
    pub city_compared: usize,
    pub city_diverged: Vec<CityDivergence>,
    /// Cities the frame names and the simulation cannot link to a building
    /// of its own — the city comparison's blind spot, counted rather than
    /// assumed away, exactly as [`FrameReport::build_unlinked`] is.
    pub city_unlinked: usize,
}

impl FrameResult {
    /// The order-list disagreements only — what the two sides *intend*.
    pub fn order_only(&self) -> impl Iterator<Item = &OrderDivergence> {
        self.order_diverged.iter().filter(|d| !d.what.is_path())
    }

    /// The path-stack disagreements only — what the pathfinder seam costs.
    pub fn path_only(&self) -> impl Iterator<Item = &OrderDivergence> {
        self.order_diverged.iter().filter(|d| d.what.is_path())
    }
}

/// The whole run.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub frames: Vec<FrameResult>,
    /// Per player slot: the first frame a unit of theirs diverged, if any.
    pub first_divergence: Vec<(i64, Option<i64>)>,
    pub notes: Vec<String>,
    /// What the recorded order stream did, when one was fed in — orders
    /// enqueued, and every command that was carried but not acted on.
    pub applied: crate::input::Applied,
    /// Per traced frame: the sim's draw count and the original's, each
    /// `None` when the words are more than [`DRAW_CAP`] apart.
    pub rng_frames: Vec<(i64, Option<u32>, Option<u32>)>,
}

/// The furthest `draws_between` walks.
pub const DRAW_CAP: u32 = 200_000;

/// How many `Random::get` steps take the sync stream from `from` to `to`,
/// if fewer than [`DRAW_CAP`].
pub fn draws_between(from: u32, to: u32) -> Option<u32> {
    let mut r = sim::combat::Rng::new(from);
    for n in 0..=DRAW_CAP {
        if r.seed == to {
            return Some(n);
        }
        r.roll();
    }
    None
}

/// The simulation's own draws as a **sequence of site labels**, one entry
/// per draw — the harness's side of
/// [`crate::trace::Trace::run_in`](crate::trace::Trace::run_in).
///
/// `marks` is [`sim::Sim::phase_marks`], each pair a label and the stream's
/// word *before* that label's draws; `end` is the word after the last of
/// them. A mark that drew nothing contributes nothing, which is what makes
/// a guarded site read correctly — `sim::scout` skips its phase draw on
/// rings under four — rather than as a zero-length hole.
///
/// This is the primitive the seed-anchored check is built on: seed the sim
/// with the trace's own word, run one mechanic, and compare this against
/// the trace's sites. A count cannot tell four rotations and two phases
/// from three and three; this can.
pub fn mark_sites(marks: &[(String, u32)], end: u32) -> Option<Vec<String>> {
    let mut out = Vec::new();
    for (i, (label, from)) in marks.iter().enumerate() {
        let to = marks.get(i + 1).map_or(end, |m| m.1);
        for _ in 0..draws_between(*from, to)? {
            out.push(label.clone());
        }
    }
    Some(out)
}

impl Report {
    /// Ticks before divergence: the last frame on which every compared unit
    /// agreed, or the number of frames if none ever disagreed.
    pub fn ticks_before_divergence(&self) -> i64 {
        self.frames
            .iter()
            .find(|f| !f.diverged.is_empty())
            .map_or(self.frames.len() as i64, |f| f.frame - 1)
    }

    /// The frame each `(who, o)` first disagreed on, for the units that ever
    /// did — the breakdown the single score cannot show.
    ///
    /// The score is a minimum over every unit of both players, so one unit
    /// the simulation cannot yet drive (an AI-ordered scout, a woodcutter
    /// whose tile list needs `BUILDS=7`) pins it at the floor while every
    /// other unit may be tracking perfectly. Reading which units diverge, and
    /// when, is what says whether a mechanic landed.
    /// Whether any frame carried an order list at all — `UNITS=3`. Without
    /// it every order figure below is zero and means nothing.
    pub fn orders_seen(&self) -> bool {
        self.frames.iter().any(|f| f.order_compared > 0)
    }

    /// The order score: the last frame on which every compared unit's order
    /// list **and path stack** agreed.
    ///
    /// It is a second score beside [`Self::ticks_before_divergence`], not a
    /// replacement, and it is the stricter of the two — two simulations can
    /// agree on every position for a whole dump and still have given every
    /// unit the wrong job. Path-stack disagreements **score** since the
    /// pathfinder landed (`docs/PATHFINDER.md` §10): the stack is now a
    /// modelled output, and how many frames it survives is the mechanic's
    /// grade — exactly the acceptance test the brief named in advance.
    pub fn order_ticks_before_divergence(&self) -> i64 {
        self.frames
            .iter()
            .find(|f| f.order_diverged.iter().any(|d| d.what.scores()))
            .map_or(self.frames.len() as i64, |f| f.frame - 1)
    }

    /// The frame each `(who, o)` first disagreed on an order, and what about
    /// — the breakdown, as [`Self::first_divergence_by_unit`] is for
    /// positions. Path-stack entries are included and marked by their
    /// variant.
    pub fn order_divergence_by_unit(&self) -> Vec<(i64, i64, i64, OrderMismatch)> {
        let mut seen: std::collections::BTreeMap<(i64, i64), (i64, OrderMismatch)> =
            std::collections::BTreeMap::new();
        for f in &self.frames {
            for d in &f.order_diverged {
                seen.entry((d.who, d.o)).or_insert((f.frame, d.what));
            }
        }
        seen.into_iter()
            .map(|((w, o), (f, what))| (w, o, f, what))
            .collect()
    }

    pub fn first_divergence_by_unit(&self) -> Vec<(i64, i64, i64)> {
        let mut seen: std::collections::BTreeMap<(i64, i64), i64> =
            std::collections::BTreeMap::new();
        for f in &self.frames {
            for d in &f.diverged {
                seen.entry((d.who, d.o)).or_insert(f.frame);
            }
        }
        seen.into_iter().map(|((w, o), f)| (w, o, f)).collect()
    }
}
