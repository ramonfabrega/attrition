//! **The finish line's own frame**: the whole-map state at 24,001, and the
//! units this crate still has wrong when the game runs out.
//!
//! `docs/DECISIONS.md` entry 29's first counter is a *length* — a fixed-seed
//! traced capture held in lockstep for its whole run — and every score the
//! queue has carried until now moves in **frames**: the word, the ticks, the
//! orders. All three say where the two simulations first part, and none of
//! them says how far apart they are eight thousand frames later. A run that
//! parts at 7,455 and one that parts at 7,455 and then loses forty units
//! score the same.
//!
//! Item 252 left the other kind of number sitting on disk, free. Ten of this
//! machine's archives carry a closing whole-map state and no ordinary frame
//! block anywhere in the file — the long traces, per-frame categories off —
//! and among them are states at **24,001 on both scored maps**. So the
//! endpoint is diffable today, without a capture: step the simulation to the
//! frame the original quit on, and count the units it has somewhere else.
//!
//! That is what this module pins. [`ENDPOINTS`] is one row a map and every
//! count in it is **pinned exactly, in neither direction** — the third
//! scoreboard line, printed past the word (see below). Unlike the word,
//! which a wrong destination can sit under for two hundred frames (item
//! 250), this is a coordinate comparison of every unit on the map, so it
//! cannot be met by drawing the right number of times.
//!
//! # The archives are three East Indies games and one Great Lakes game
//!
//! The endpoint captures are siblings rather than one capture apiece, and
//! `the_endpoint_captures_are_four_families` is the assertion that says so:
//! `Trace::frames` carries `game_random`'s word at every `do_frame` entry, so
//! two captures of the same game agree on it frame for frame, and two
//! captures of different games do not.
//!
//! | family | captures | agrees to | closing dumps |
//! | --- | --- | --- | --- |
//! | Great Lakes | run53, run18a | 24,001 | 24,001 (both) |
//! | East Indies A | run54, run21, run23 | 24,001 | 24,001 (all three) |
//! | East Indies B | run24, run25, run26, run27 | 16,007 | 16,489, 16,007 |
//! | East Indies C | run28, run29 | 15,105 | 15,401, 15,105 |
//!
//! **The three East Indies families are one game to 12,000 and diverge on
//! 12,001**, and B and C stay together to 15,020 and diverge on 15,021. So
//! the ladder rungs item 252 named are **not** rungs of the endpoint's own
//! run, and walking the endpoint's simulation to 15,401 to read run28's
//! closing dump would compare two different games. Each rung is its own walk
//! from its own capture's start block, which is what [`LADDER`] is.
//!
//! What splits them is not established here. 12,001 is a round number in a
//! run that was driven, so a click or a window opening is the obvious guess,
//! and a guess is all it is.
//!
//! # Why the counts are pinned and not ratcheted
//!
//! The word parts at 7,455 and 7,529, so past that both sides run on streams
//! that are nobody's and the endpoint is measured *deep* inside the
//! divergence. The number is real — every unit of it is a coordinate the
//! original wrote — but it is not one defect, and a change that fixes a
//! mechanic can move it in either direction for reasons that have nothing to
//! do with the mechanic.
//!
//! Item 258 booked these counts as ceilings that may only fall, on the
//! argument that coincidence does not put a unit on a tile 24,000 frames in.
//! The first landing after it (261) raised Great Lakes' `off` while lowering
//! its `unlinked`, on two value-diff-backed corrections, and the line was
//! re-pinned upward three times in two days (261, 265, 267), each time on a
//! mechanic the decompile owns. A behavioural change at 6,751 re-deals every
//! position downstream of it and `off` counts the deal; the count is
//! monotone in fidelity only as the word approaches the frame it is measured
//! on. So the 2026-09-01 rule stands — **assert up to the word, print past
//! it** — and the ratchet is reverted (`docs/DECISIONS.md` entry 36). What
//! is left is an exact pin: a count that moves in *either* direction fails
//! here and is re-pinned with the item's number and nothing more owed, so
//! the number is noticed and written down, and no direction is claimed. When
//! a map's word reaches 24,001 the pin is the finish line's own assertion,
//! and it reads zero.
//!
//! # What it costs, measured
//!
//! A 24,000-frame walk is **0.9 seconds**; all seven tests here are 4.6 s of
//! wall clock and 2.6 GiB of peak resident on their own, and the expense is
//! parsing the 11 MB dump rather than ticking. In the release gate the
//! difference does not show above run-to-run noise: 231 tests without this
//! module take 210.5 s and peak at 15,725 MiB, and 238 with it take 208.1 s
//! and peak at 16,027 MiB (2026-09-07, two test threads, `tools/memcap.sh`).
//! So these stay in the default suite — there is nothing to gate.
//!
//! # What this does not establish
//!
//! - **It is not lockstep.** Entry 29's counter is met when the *whole* run
//!   agrees, and this compares one frame at the end. A run that diverges at
//!   7,455, wanders, and happens to arrive with every unit on the original's
//!   tile would read zero here and still not be in lockstep. The endpoint is
//!   a floor under the finish line, not the finish line.
//! - **The closing dump is frame `n` and tears by one tick on at most one
//!   unit** — the finding [`super::compare_shutdown`] rests on. Its evidence
//!   is `super::shutdown`'s three tests, and it is *assumed* here. East
//!   Indies' endpoint has one torn unit and Great Lakes' none.
//! - **`torn` is printed, not pinned.** It counts units that agreed at
//!   `n − 1` and differ at `n`, so it depends on this crate's own `n − 1`
//!   as well as on the dump, and more of them is better rather than worse.
//!   A ceiling on it would have the sign backwards.
//! - **15,105 and 16,007 are missing from the ladder.** They live only in the
//!   250 MB window captures, which [`crate::gamelog::Log::parse`] reads whole
//!   (item 260), and a quarter-gigabyte parse to reach one record is the cost
//!   that item is about. The queue records run29's 15,105 as 51 off and 22
//!   unlinked; the rung goes in when the parser goes lazy.
//! - **The order half compares nothing.** These captures write no per-frame
//!   order lists, so [`compare`]'s `UNITS=3` gate never opens on a closing
//!   dump of theirs. That is the capture's detail level, not a gap here.

use super::*;

use std::collections::BTreeMap;

/// The frame both scored maps' long captures quit on — the finish line
/// `docs/DECISIONS.md` entry 29 names, 24,000 frames of game and the
/// one-based label the original writes on the way out.
pub const ENDPOINT_FRAME: i64 = 24_001;

/// One closing dump this crate is scored against, and every count of it,
/// pinned exactly.
///
/// The scoring tests assert equality on each field rather than their own
/// literals, and the queue's handoff states the two headline counts on an
/// `Endpoint:` line that [`ENDPOINTS`]' own test parses — so a count that
/// moves without the handoff, or a handoff written off a run that is not
/// this one, fails somewhere instead of waiting for a steering pass.
pub struct Endpoint {
    pub map: &'static str,
    /// The capture whose closing dump is the oracle, and whose start block
    /// and pasture the simulation is stood up from. For [`ENDPOINTS`] this
    /// is the map's **scored long capture**, so the endpoint count and the
    /// word describe one game.
    pub capture: &'static str,
    /// The `rontrace-…` beside it, for the pasture the dump does not write.
    pub trace: &'static str,
    /// The frame the capture quit on.
    pub frame: i64,
    /// The map's own `herds` and `goods` counts once the borrow is done —
    /// the fingerprint that says the setup is *this* map's.
    ///
    /// East Indies is `(41, 66)` and Great Lakes `(13, 36)`, and offering
    /// the wrong map's start dump silently swaps one for the other. That is
    /// not a hypothetical: this module's first East Indies number was
    /// measured on Great Lakes' herds and goods, read 65 units off, and was
    /// two edits from the scoreboard.
    pub map_lists: (usize, usize),
    /// The start dumps this capture's own is allowed to borrow from.
    ///
    /// **Same map only.** [`borrow_from_siblings`] gates the world cells and
    /// the heights on the sibling's own scalars, but takes `regions`,
    /// `herds`, `goods` and `farms` from the first sibling that has them,
    /// whatever map it is — so offering a Great Lakes start dump to an East
    /// Indies capture quietly rebuilds it on the wrong map's lists — which
    /// is how this module's first East Indies number was measured. Every
    /// row here names one map's start dumps and only that map's:
    /// [`GREAT_LAKES_SETUP`] or [`EAST_INDIES_SETUP`].
    pub setup: &'static [&'static str],
    /// Other captures of the **same game** that closed on the same frame.
    /// Their closing dumps are asserted to be one state with `capture`'s,
    /// unit for unit — which is what says the oracle is the game's and not
    /// one quit's artefact.
    pub siblings: &'static [&'static str],
    /// Units the dump holds, the simulation links, and puts somewhere the
    /// dump does not — at neither `n` nor `n − 1`. **The headline.**
    pub off: usize,
    /// Units the dump holds for a real player that the simulation has no
    /// unit for at all. The second headline.
    pub unlinked: usize,
    /// Units the **simulation** holds that the dump does not — the mirror,
    /// so that over-producing cannot read as agreement.
    pub extra: usize,
    /// Buildings the dump names that the simulation cannot link, and
    /// building fields that disagreed on the ones it can.
    pub build_unlinked: usize,
    pub build_diverged: usize,
    /// The same two for cities.
    pub city_unlinked: usize,
    pub city_diverged: usize,
}

/// The Great Lakes start dumps a capture of that map may borrow from — the
/// four [`crate::diff::testkit::sibling_texts`] offers, named here so a row
/// can say *which map's* siblings it takes.
/// `the_setup_lists_are_the_suite_s` keeps the two in step.
pub const GREAT_LAKES_SETUP: &[&str] = &[
    "gamelog-run11-checksum.txt",
    "gamelog-run3-fulldump-types.txt",
    "gamelog-run12-dumpall-seeds.txt",
    "gamelog-run13-window-95-105.txt",
];

/// East Indies' — run38's `DUMP_ALL` start, which is the sibling
/// `east_indies_closing_dumps_are_the_original_s` has used since item 252.
pub const EAST_INDIES_SETUP: &[&str] = &["gamelog-run38-islands-start.txt"];

/// The finish line itself, East Indies first as [`FLOORS`] is: the lower map
/// leads.
pub const ENDPOINTS: [Endpoint; 2] = [
    Endpoint {
        map: "EastIndies",
        capture: "run54-islands-24k-trace",
        trace: "rontrace-run54.log",
        frame: ENDPOINT_FRAME,
        map_lists: (41, 66),
        setup: EAST_INDIES_SETUP,
        siblings: &["run21-islands-long", "run23-islands-war"],
        // 80 → 79, 11 → 10 and 32 → 30 on 2026-09-07, the eighth steer
        // (DECISIONS 36): the ratchet let falls go unrecorded — its
        // IMPROVED line went to a captured stderr — and the exact pin found
        // them on its first run. Which landing made them is not established;
        // 267's repaint is the only one since 265's re-pin.
        //
        // Then **79 → 80, 10 → 11 and 30 → 31** the same day on item 267's
        // second row — `compute_form`'s tail negation (`docs/GROUPS.md`
        // §6.3). East Indies' word does not move on it and its endpoint
        // drifts by one in each of three counts, 16,000 frames past the
        // word; Great Lakes' word does move, by 129 frames. The trade
        // DECISIONS 26 allows, on a mechanic the listing owns.
        //
        // And **80 → 79, 31 → 32** the same day on item 271, the age snap
        // (`docs/TECH.md`, "An age snaps every figure"). One position
        // closer and one building field-row further out, 16,195 frames past
        // the word — which the change moves 7529 → 7806, its own map's
        // headline and the largest single step it has taken. DECISIONS 36
        // asks for the number rather than a trade.
        //
        // Then **79 → 78 and 11 → 13** on 2026-09-07, item 289 — the
        // sidestep waypoint's arrival rule (`docs/COLLISION.md` §8.7),
        // which moves this map's own word 7806 → 7812. One position
        // closer and two spurious units, 16,189 frames past the word.
        // DECISIONS 36: the number, not a trade.
        // And **79 → 80, 0 → 1, 11 → 0, 0 → 2 and 32 → 28** on 2026-09-07,
        // item 290 — `LeaderData::get_mod_resource_cap`'s difficulty
        // scaling (`docs/AI.md` §33.2). The AI reads half its commerce cap
        // on this lobby's Easiest setting, which moves `rate`, `best_good`
        // and every `income < cap` gate in `create_buildings`,
        // `create_units` and `research_techs`. Neither word moves; the
        // endpoint reshuffles 16,000 frames past it — eleven extra units
        // gone and one unlinked in their place, four building field-rows
        // closer, one position further out. DECISIONS 36 asks for the
        // number rather than a trade.
        // **Re-pinned at the 289/290 merge, 2026-09-07**, and by neither
        // worker's numbers. Both re-pinned this row on their own branch and
        // the merged tree agrees with neither: two independent improvements
        // compose, so the figure is what the merged code prints, and a merge
        // is not the place to adjudicate a floor.
        //
        // Then **78 → 76, 3 → 5, 4 → 3 and 23 → 25** on 2026-09-17, item
        // 294 — `move_step`'s turn-in-place return (`docs/COLLISION.md`
        // §8.8). A frame spent turning no longer probes for a collision,
        // reveals, or runs the arrival test, so every unit on both maps
        // that has ever turned in place is on a slightly different clock.
        // Two positions closer and two more of the roster unlinked, two
        // building field-rows out and one fewer building missing,
        // 16,189 frames past a word that does not move. DECISIONS 36 asks
        // for the number rather than a trade.
        // And **78 → 70, 3 → 9, 4 → 0 and 23 → 30** on 2026-09-17, item
        // 295 — `census_units` reading `UnitTypeData::role` over captains
        // (`docs/AI.md` §35). The sweep counted a caravan as the leader's
        // only soldier and every figure of a squad as its own unit, so
        // this is the AI's whole view of its army changing at once, on
        // both maps and 16,000 frames past either word. Eight positions
        // closer, six more of the roster unlinked, every unlinkable
        // building linked and seven more building field-rows apart.
        // DECISIONS 36 asks for the number rather than a trade.
        // **Re-pinned at the 294/295 merge, 2026-09-17**, and by neither
        // worker's numbers: the two land independently and compose, so the
        // figure is what the merged code prints.
        //
        // Then **72 → 70, 9 → 10 and 30 → 31** on 2026-09-17, item 302 —
        // `upgrade_units` dividing by `LeaderData::village_num` instead of
        // a recount of its own, and `age_p` defaulting to the type's own
        // age (`docs/AI.md` §36). Both halve or flatten what the AI is
        // willing to pay for an upgrade, so this is again the AI's
        // valuation changing on both maps, 16,189 frames past a word that
        // does not move — two positions closer, one more of the roster
        // unlinked, one more building field-row apart. DECISIONS 36 asks
        // for the number rather than a trade. **Branch figures**: measured
        // on `worktree-loop-302` with 301 still in flight, so the merge
        // re-runs them.
        //
        // Then **70 → 66, 10 → 16 and 31 → 30** on 2026-09-17, items 301
        // and 304 — the suspended search wired (`docs/PATHFINDER.md` §18,
        // §18.5): `find_upath`'s pre-walk gate, its `500 / max(1, repaths²)`
        // limit, `do_move`'s §4.4 step 2 block, and the `clear_partial_path`
        // under `kill_current_path`'s pop that pays their Great Lakes cost.
        // This is the change that moves this map's own word **7812 → 8193**,
        // the largest single step it has taken, and it empties run90's
        // shuffle of everything but the two Merchant constants. Four
        // positions closer and one building field-row closer, 15,808 frames
        // past the word; six more of the roster unlinked in exchange.
        // DECISIONS 36 asks for the number rather than a trade.
        off: 66,
        unlinked: 16,
        extra: 0,
        build_unlinked: 0,
        build_diverged: 30,
        city_unlinked: 3,
        city_diverged: 0,
    },
    Endpoint {
        map: "GreatLakes",
        capture: "run53-greatlakes-24k-trace",
        trace: "rontrace-run53.log",
        frame: ENDPOINT_FRAME,
        map_lists: (13, 36),
        setup: GREAT_LAKES_SETUP,
        siblings: &["run18a-startonly"],
        // 73 → 75 and 7 → 2 on 2026-09-07, item 261: see the journal. The
        // positional count rose while the structural one fell by five.
        //
        // Then **75 → 81, 2 → 0, 0 → 4 and 14 → 13** the same day on item
        // 267's world-cell repaint (`docs/COLLISION.md` §2.3). The roster
        // moves both ways — the two units this crate was missing are here
        // now, and four it should not have are with them — 16,546 frames
        // past the word, which is the reshuffle the entry above describes
        // and the trade DECISIONS 26 allows: the mechanic is the
        // decompile's own, and it takes the last unit divergence off Great
        // Lakes' run-up 200 frames before the word.
        //
        // And **81 → 78, 0 → 2, 4 → 0** on the same day on 267's second
        // row, the tail negation of `§6.3`: three positions closer, the
        // four spurious units gone and two missing again, on the change
        // that moves the word itself 7455 → 7584.
        //
        // Then **78 → 84, 2 → 0, 0 → 2 and 13 → 12** on 2026-09-07, item
        // 284 — the gaia bird's arrival stand and the figure's lagging
        // step behind it (`docs/SYNC.md` §3.9). It moves the word 7584 →
        // 7585 and closes `1/3`'s spot, the value diff beside it, 16,416
        // frames before this endpoint; here the roster's two missing units
        // are found and two spurious ones take their place, six positions
        // are further out and one building field-row is closer. DECISIONS
        // 36 asks for the number rather than a trade.
        //
        // Then **84 → 80 and 2 → 1** on 2026-09-07, item 289 — East
        // Indies' sidestep arrival rule (`docs/COLLISION.md` §8.7). Great
        // Lakes' own word does not move on it at all and its endpoint
        // falls by four positions and one spurious unit 16,416 frames
        // past that word: a mechanic every colliding unit on either map
        // walks through, and the largest fall this row has had.
        // Then **84 → 73, 0 → 8, 2 → 0** on 2026-09-07, item 290, the same
        // `get_mod_resource_cap` row as East Indies' above. Eleven
        // positions closer and eight of the roster unlinked, 16,416 frames
        // past a word that does not move — the largest single move either
        // endpoint's `off` has made, and in the good direction, on a change
        // whose whole effect is what the AI *values*.
        // **Re-pinned at the 289/290 merge, 2026-09-07**, and by neither
        // worker's numbers. Both re-pinned this row on their own branch and
        // the merged tree agrees with neither: two independent improvements
        // compose, so the figure is what the merged code prints, and a merge
        // is not the place to adjudicate a floor.
        //
        // Then **12 → 11** on 2026-09-17, item 294's turn-in-place return
        // (`docs/COLLISION.md` §8.8) — one building field-row closer and
        // nothing else on this row moved, where East Indies' own took four
        // counts on the same change.
        // Then **70 → 56, 9 → 26 and 12 → 10** on 2026-09-17, item 295 —
        // the census fix above, which on this map moves the word itself
        // **7585 → 7679**. Fourteen positions closer, the largest single
        // fall this row has had and the second in a row from a change to
        // what the AI *counts*; seventeen more of the roster unlinked in
        // exchange, and two building field-rows closer. DECISIONS 36 asks
        // for the number rather than a trade.
        // **Re-pinned at the 294/295 merge, 2026-09-17**, and by neither
        // worker's numbers: the two land independently and compose, so the
        // figure is what the merged code prints.
        // Then **55 → 57 and 28 → 23** on 2026-09-17, item 302 — the same
        // two `upgrade_units` faults as East Indies' row above
        // (`docs/AI.md` §36). Two positions out and five of the roster
        // linked again, 16,322 frames past a word that does **not** move:
        // this item closed §34's third oracle and the whole of
        // `MAKE[7].val`, and Great Lakes' 7679 is unchanged either side of
        // it. DECISIONS 36 asks for the number rather than a trade.
        // **Branch figures**, measured on `worktree-loop-302`.
        // Then **57 → 59 and 10 → 12** on 2026-09-17, items 301 and 304 —
        // the suspended search wired, the same row as East Indies' above.
        // This map's word does **not** move on it: 7679 either side, which
        // is the whole point of item 304, since the block cost 7679 → 6862
        // while `kill_current_path` popped without freeing the stash. Two
        // positions out and two building field-rows out, 16,322 frames past
        // that word, against East Indies' 7812 → 8193. DECISIONS 36 asks
        // for the number rather than a trade.
        off: 59,
        unlinked: 23,
        extra: 0,
        build_unlinked: 0,
        build_diverged: 12,
        city_unlinked: 3,
        city_diverged: 0,
    },
];

/// The two rungs under East Indies' endpoint, each its own game (see the
/// module header) and so each its own walk.
///
/// They are here because a single number at the end cannot say whether it is
/// a plateau this crate reaches early or a slope it slides down. 45 off at
/// 15,401 and 51 at 16,489 against 80 at 24,001 says **slope**, and says it
/// across three sibling games rather than three readings of one — the last
/// 7,500 frames cost as much again as the first 15,000.
///
/// Neither rung's capture carries its own world cells, so both are stood up
/// on run38's start dump through [`borrow_from_siblings`], and
/// `the_ladder_s_borrowed_setup_is_the_endpoint_s` is what says that borrow
/// is the real thing rather than a plausible one: without it run28 read 12
/// of 71 units compared and `1/0` was 24,789 units out.
pub const LADDER: [Endpoint; 2] = [
    Endpoint {
        map: "EastIndies C",
        capture: "run28-islands-engagement",
        trace: "rontrace-run28.log",
        frame: 15_401,
        map_lists: (41, 66),
        setup: EAST_INDIES_SETUP,
        siblings: &[],
        // 45 → 46 and 24 → 20 on 2026-09-07, item 265: the same trade the
        // entry above describes, one item later — `CityData::free` wrapping
        // to 255 the way the original's byte does costs one position here,
        // 7,872 frames past the word, and takes four spurious units off.
        // Then 46 → 47 on item 289; see `extra` below.
        // **Re-pinned at the 289/290 merge, 2026-09-07**, and by neither
        // worker's numbers. Both re-pinned this row on their own branch and
        // the merged tree agrees with neither: two independent improvements
        // compose, so the figure is what the merged code prints, and a merge
        // is not the place to adjudicate a floor.
        // Then **46 → 47** on 2026-09-17, item 295's census fix
        // (`docs/AI.md` §35): one position out, 7,872 frames past this
        // rung's word, on the change that moves Great Lakes' own word
        // 7585 → 7679 and takes fourteen off its endpoint. **Re-pinned at
        // the 294/295 merge**: 46 again — 294's own row moved `extra` and
        // not `off`, and the merged tree keeps this rung's position.
        off: 46,
        unlinked: 17,
        // 20 → 21 on 2026-09-07, item 267's repaint: one spurious unit
        // back, 7,872 frames past the word, and nothing else on this rung
        // moved. Then 21 → 23 on 267's second row, the `§6.3` tail
        // negation — two more, and again nothing else on the rung. Then
        // **23 → 22 and 9 → 8** on item 271's age snap, both falls. Then
        // **22 → 24 and 46 → 47** on item 289's sidestep arrival rule
        // (`docs/COLLISION.md` §8.7): two spurious units and one position
        // out, 7,872 frames past this rung's own word, on the change that
        // moves East Indies' 7806 → 7812 and takes four off Great Lakes'
        // endpoint. The B rung moved the other way on the same change,
        // one spurious unit fewer and nothing else.
        // **22 → 13** on item 290's `get_mod_resource_cap` (`docs/AI.md`
        // §33.2) — nine spurious units gone and nothing else on this rung
        // moved, which is the same shape as East Indies' own endpoint
        // losing all eleven of its extras. Then **7 → 18** on 2026-09-17,
        // item 294's turn-in-place return — eleven spurious units back on
        // this rung alone, 7,872 frames past its word, with `off`,
        // `unlinked` and both building counts unmoved. It is the largest
        // single move this rung's `extra` has made, and in the wrong
        // direction; DECISIONS 36 asks for the number rather than a trade.
        // The **roster** says which shape it is, and it was read on both
        // sides of the change: the seven were `1/10 1/12 1/25 1/29 1/48
        // 1/49 1/54` and the eighteen are the same seven plus `1/55` …
        // `1/65`. Not one existing extra moved, the additions are a
        // contiguous run at the top of the production roster, and `off` is
        // unmoved at 46 — over-production, not units that stopped
        // colliding, which would move positions and scatter through the
        // roster (`docs/COLLISION.md` §8.8).
        // Item 295's census fix (`docs/AI.md` §35) takes this rung to the
        // same 18 on its own branch, for its own reason — the AI counting
        // captains instead of figures changes what it thinks it has, so
        // what it builds moves with it. Two changes, one figure:
        // **re-pinned at the 294/295 merge, 2026-09-17** and unmoved.
        // Then **18 → 19** on 2026-09-17, items 301 and 304's suspended
        // search: one spurious unit back on this rung, 7,872 frames past
        // its word, with `off`, `unlinked` and both building counts
        // unmoved — the only count either rung moves on a change that takes
        // East Indies' own word 7812 → 8193.
        extra: 19,
        build_unlinked: 10,
        build_diverged: 7,
        city_unlinked: 3,
        city_diverged: 0,
    },
    Endpoint {
        map: "EastIndies B",
        capture: "run24-islands-raid",
        trace: "rontrace-run24.log",
        frame: 16_489,
        map_lists: (41, 66),
        setup: EAST_INDIES_SETUP,
        siblings: &[],
        // 50 → 52 and 26 → 25 on 2026-09-07, item 267's second row (§6.3's
        // tail negation): two positions out and one spurious unit gone,
        // 8,960 frames past the word. The C rung above moved on the same
        // change and Great Lakes' endpoint moved the other way.
        // **Re-pinned at the 289/290 merge, 2026-09-07**: `extra` 9 -> 8.
        // The ladder test panics at its first moved row, so this rung only
        // surfaced once the C rung above it was re-pinned.
        // Then **52 → 50 and 16 → 12** on 2026-09-17, items 301 and 304's
        // suspended search: two positions closer and four spurious units
        // gone, 8,960 frames past this rung's word. The C rung above moved
        // one count the other way on the same change, and this rung only
        // surfaced once that one was re-pinned — the ladder test panics at
        // its first moved row.
        off: 50,
        unlinked: 16,
        // 19 → 33 on 2026-09-07, item 261, the same reshuffle, then 33 →
        // **27** the same day on item 265's byte — this rung took the
        // structural half of that trade and none of the positional one.
        // `off` fell 51 → 50 on 261's run and was left pinned at 51; the
        // exact pin of the eighth steer (DECISIONS 36) took it, and 27 → 26
        // with it, on its first run. Then **25 → 20** on item 271's age
        // snap — five spurious units gone, 8,960 frames past the word, and
        // the largest single fall this rung has had. Then **20 → 19** on
        // item 289's sidestep arrival rule, one more gone and `off`
        // unmoved; the C rung above took two the other way on the same
        // change.
        // the largest single fall this rung has had. Then **20 → 9** on
        // item 290's `get_mod_resource_cap` (`docs/AI.md` §33.2), which is
        // larger again: eleven more spurious units gone and nothing else on
        // this rung moved. All three East Indies rows lost extras on that
        // change — 11, 9 and 11 — and its own endpoint lost every one it
        // had. Then **8 → 16** on 2026-09-17, item 294's turn-in-place
        // return, with nothing else on this rung moved — eight spurious
        // units back where the C rung above took eleven, so both East
        // Indies rungs gave back most of what 290 won them and the map's
        // own endpoint, which has no extras to give, did not. Same roster
        // shape as the C rung: the eight were `1/12 1/56` … `1/60 1/63
        // 1/64` and the sixteen are the same eight plus `1/61 1/62 1/65` …
        // `1/70`, with `off` unmoved at 52.
        // Item 295's census fix (`docs/AI.md` §35) takes this rung the
        // same way on its own branch, 8 → 14, and the C rung above eleven
        // — both East Indies rungs gain extras where Great Lakes' own
        // word moves 7585 → 7679. **Re-pinned at the 294/295 merge,
        // 2026-09-17**: 17, one more than either worker's branch (294's
        // said 16 and 295's 14), so the merged tree agrees with neither.
        // Then **17 → 16** on 2026-09-17, item 302's `upgrade_units` fixes
        // (`docs/AI.md` §36): one spurious unit gone, 8,960 frames past
        // this rung's word, with nothing else on it moved and the C rung
        // above unmoved entirely. **Branch figure.**
        // Then **16 → 12** on 2026-09-17 with the `off` above, on items
        // 301 and 304's suspended search: four spurious units gone, the
        // largest fall this rung's `extra` has had, on the change that
        // moves East Indies' own word 7812 → 8193. The C rung above gained
        // one on the same change.
        extra: 12,
        build_unlinked: 19,
        build_diverged: 0,
        city_unlinked: 4,
        city_diverged: 0,
    },
];

/// One closing dump compared against a simulation that walked to it, with
/// the dump's own one-tick tear separated out.
///
/// The three populations [`ShutdownResult`] carries, plus the records beside
/// the units: a closing dump is a *whole-map* state, so its buildings and its
/// cities are compared too and reported here rather than thrown away.
#[derive(Debug, Default, Clone)]
pub struct EndpointResult {
    pub frame: i64,
    pub compared: usize,
    pub unlinked: Vec<(i64, i64)>,
    pub extra: Vec<(i64, i64)>,
    /// Units the dump has at the simulation's `n − 1` position: its own
    /// tear, and not a divergence.
    pub torn: Vec<(i64, i64)>,
    /// Units the dump agrees with at neither `n − 1` nor `n`, with
    /// `ours − theirs` at `n`.
    pub off: BTreeMap<(i64, i64), (i32, i32)>,
    pub build_unlinked: usize,
    pub build_diverged: usize,
    pub city_unlinked: usize,
    pub city_diverged: usize,
}

impl EndpointResult {
    /// The counts in [`Endpoint`]'s own order, for the comparison and
    /// the failure message.
    pub fn counts(&self) -> [usize; 7] {
        [
            self.off.len(),
            self.unlinked.len(),
            self.extra.len(),
            self.build_unlinked,
            self.build_diverged,
            self.city_unlinked,
            self.city_diverged,
        ]
    }
}

impl Endpoint {
    /// The pinned counts, in [`EndpointResult::counts`]' order.
    pub fn counts(&self) -> [usize; 7] {
        [
            self.off,
            self.unlinked,
            self.extra,
            self.build_unlinked,
            self.build_diverged,
            self.city_unlinked,
            self.city_diverged,
        ]
    }
}

/// The names of [`EndpointResult::counts`], in its order.
pub const COUNT_NAMES: [&str; 7] = [
    "off",
    "unlinked",
    "extra",
    "build_unlinked",
    "build_diverged",
    "city_unlinked",
    "city_diverged",
];

/// Walk `built` from wherever it stands to `fin.n` and compare the whole
/// closing state against it.
///
/// The unit half is [`compare_shutdown`], which steps the last frame itself
/// so that the dump's one-tick tear stays decidable; the building and city
/// halves are read from a second [`compare`] at `n`, which
/// `compare_shutdown` has already left the simulation standing on.
pub fn walk_to_close(built: &mut Built, fin: &Frame, players: usize) -> EndpointResult {
    while built.sim.frame < fin.n - 1 {
        built.tick();
    }
    let r = compare_shutdown(built, fin, players);
    let whole = compare(built, fin, players);
    EndpointResult {
        frame: r.frame,
        compared: r.compared,
        unlinked: r.unlinked,
        extra: whole.extra_units,
        torn: r.torn,
        off: r.off,
        build_unlinked: whole.build_unlinked,
        build_diverged: whole.build_diverged.len(),
        city_unlinked: whole.city_unlinked,
        city_diverged: whole.city_diverged.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::diff::testkit::trace;
    use crate::gamelog::Log;
    use crate::testenv::{dump, install};

    use std::collections::BTreeSet;

    /// **The setup is this map's, and it is complete.**
    ///
    /// Every list `build_sim` reads is checked non-empty, and the two that
    /// identify the map are checked against [`Endpoint::map_lists`]. Both
    /// halves were written by making them fail: without a sibling, run28's
    /// world has no cells and it walks a region-less map to 12 units
    /// compared of 71; with the *wrong* map's sibling, run54 gets Great
    /// Lakes' 13 herds and 36 goods and reads a plausible, wrong 65 off.
    /// A setup defect looks exactly like a fidelity number, which is why it
    /// is asserted here and not eyeballed.
    fn check_setup(row: &Endpoint, init: &Initial<'_>) {
        // Length first, name second: `writers::compared_fields_have_writers`
        // reads a `("name", …)` tuple in a diff source as a comparison of
        // whatever simulation field the body names, and these are the
        // *dump's* setup lists rather than anything `crates/sim` owns.
        // The herd list, named first, failed the gate on exactly that —
        // and the scanner reads comments too, so this one says no more.
        let empty: Vec<&str> = [
            (
                crate::gamelog::world_cells(&init.world).len(),
                "world cells",
            ),
            (init.heights.len(), "heights"),
            (init.regions.len(), "regions"),
            (init.herds.len(), "herds"),
            (init.goods.len(), "goods"),
            (init.farms.len(), "farms"),
            (init.units.len(), "units"),
        ]
        .into_iter()
        .filter(|(n, _)| *n == 0)
        .map(|(_, k)| k)
        .collect();
        assert!(
            empty.is_empty(),
            "{}'s setup is short of {empty:?} after borrowing from {:?} — a \
             capture taken for a closing state writes the world block's \
             seventeen scalars and stops, and every list here has to come \
             from a start dump of the same map",
            row.capture,
            row.setup,
        );
        assert_eq!(
            (init.herds.len(), init.goods.len()),
            row.map_lists,
            "{}'s borrowed setup is not {}'s map: `borrow_from_siblings` takes \
             `regions`, `herds`, `goods` and `farms` from the first sibling \
             that has them, whatever map it is",
            row.capture,
            row.map,
        );
    }

    /// Stand one row's simulation up from its own capture's start block,
    /// walk it to the row's frame, and score the whole closing state.
    ///
    /// The siblings are checked **before** the walk: if two captures of the
    /// same game disagree about the state they quit on, the oracle is wrong
    /// and there is nothing worth walking to.
    fn score(row: &Endpoint) -> Option<EndpointResult> {
        let inst = install()?;
        let (Some(path), Some(tr)) = (
            dump(&format!("gamelog-{}.txt", row.capture)),
            trace(row.trace),
        ) else {
            eprintln!("skipping {}: set RON_GAMELOG_DIR", row.map);
            return None;
        };
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let fin = log.final_state().expect("the census says it has one");
        assert_eq!(
            fin.n, row.frame,
            "gamelog-{}.txt's closing dump",
            row.capture
        );

        // **The siblings, first.** Every other capture of this game that
        // quit on this frame holds the same whole-map state, unit for unit
        // and field for field — which is what says the record is the game's
        // and not one quit's artefact. `sibling` is a *different run of the
        // same game*, so this is stronger than item 252's within-file pair
        // (`a_closing_dump_and_its_own_block_are_one_state`): those two
        // records are written at the same instant, and these are not.
        for name in row.siblings {
            let Some(p) = dump(&format!("gamelog-{name}.txt")) else {
                eprintln!("skipping {}'s sibling {name}", row.map);
                continue;
            };
            let t = crate::capture::read(&p);
            let sib = Log::parse(&t).final_state().expect("a closing dump");
            assert_eq!(sib.n, row.frame, "gamelog-{name}.txt's closing dump");
            let mine: BTreeSet<(i64, i64)> = fin.units.iter().map(|u| (u.who, u.o)).collect();
            let theirs: BTreeSet<(i64, i64)> = sib.units.iter().map(|u| (u.who, u.o)).collect();
            assert_eq!(
                mine, theirs,
                "{name} and {} hold different units at {}",
                row.capture, row.frame
            );
            let mut bad: Vec<String> = Vec::new();
            for u in &fin.units {
                let v = sib
                    .units
                    .iter()
                    .find(|v| v.who == u.who && v.o == u.o)
                    .expect("the id sets agree");
                if v.pos != u.pos || v.flags != u.flags {
                    bad.push(format!(
                        "{}/{} pos {:?}/{:?} flags {}/{}",
                        u.who, u.o, u.pos, v.pos, u.flags, v.flags
                    ));
                }
            }
            assert!(
                bad.is_empty(),
                "{name} and {} part at {}: {bad:?}",
                row.capture,
                row.frame
            );
        }

        let loaded = crate::load::load(&inst).unwrap();
        // **The setup siblings, same map only** — see [`Endpoint::setup`].
        // A capture taken for a closing state alone writes the world
        // block's seventeen scalars and stops, so its own start dump has no
        // cells and no per-tile masks: every unit stands in a region-less
        // world, nothing can gather, and the walk scores noise (run28 read
        // 12 units of 71 that way).
        let sib_texts: Vec<String> = row
            .setup
            .iter()
            .filter_map(|n| dump(n))
            .map(crate::capture::read)
            .collect();
        let sib_logs: Vec<Log> = sib_texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = sib_logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().expect("a capture carries its start block");
        borrow_from_siblings(&mut init, &refs);
        borrow_pasture(&mut init, &tr);
        check_setup(row, &init);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let started = std::time::Instant::now();
        let r = walk_to_close(&mut built, &fin, 8);
        eprintln!(
            "{} {}: {} compared, {} off, {} unlinked, {} extra, {} torn; \
             builds {}/{} unlinked/diverged, cities {}/{} — {:.1}s",
            row.map,
            r.frame,
            r.compared,
            r.off.len(),
            r.unlinked.len(),
            r.extra.len(),
            r.torn.len(),
            r.build_unlinked,
            r.build_diverged,
            r.city_unlinked,
            r.city_diverged,
            started.elapsed().as_secs_f64(),
        );
        Some(r)
    }

    /// The equality against the pinned row, count by count.
    ///
    /// **Either direction fails, and neither is a verdict.** Item 258 made
    /// this a `<=` — a rise failed, a fall printed — on the claim that there
    /// is no way to make the endpoint worse by making the simulation better.
    /// The first landing after it (261) did exactly that, and the line was
    /// re-pinned upward three times in two days; the module header and
    /// `docs/DECISIONS.md` entry 36 have the reasoning. The 2026-09-01 rule
    /// stands: assert up to the word, print past it. This is past the word,
    /// so what is asserted is only that the pinned number is the measured
    /// one — a moved count is noticed and written down, in
    /// `ENDPOINTS`/`LADDER` and on the queue's `Endpoint` line together,
    /// with the item's number beside it, and nothing more is owed.
    fn pinned(row: &Endpoint, r: &EndpointResult) {
        let got = r.counts();
        let want = row.counts();
        let moved: Vec<String> = (0..got.len())
            .filter(|&i| got[i] != want[i])
            .map(|i| format!("{} {} (pinned {})", COUNT_NAMES[i], got[i], want[i]))
            .collect();
        assert!(
            moved.is_empty(),
            "{}'s endpoint at {} moved: {}. This is the third scoreboard line, \
             pinned exactly and asserted in no direction (DECISIONS 36): re-pin \
             `ENDPOINTS`/`LADDER` and the queue's `Endpoint` line together, \
             naming the item.
  off {:?}
  unlinked {:?}
  extra {:?}",
            row.map,
            row.frame,
            moved.join(", "),
            r.off,
            r.unlinked,
            r.extra,
        );
    }

    /// **East Indies at 24,001** — the finish line's own frame, every unit,
    /// every building, every city, against run54's closing dump and its two
    /// siblings' agreement that the record is the game's.
    #[test]
    fn east_indies_endpoint_is_pinned() {
        let row = &ENDPOINTS[0];
        let Some(r) = score(row) else { return };
        pinned(row, &r);
    }

    /// **Great Lakes at 24,001**, the same, against run53's.
    #[test]
    fn great_lakes_endpoint_is_pinned() {
        let row = &ENDPOINTS[1];
        let Some(r) = score(row) else { return };
        pinned(row, &r);
    }

    /// **The two rungs under the endpoint**, each its own game and its own
    /// walk (see the module header): run28's 15,401 and run24's 16,489.
    #[test]
    fn the_east_indies_ladder_is_pinned() {
        for row in &LADDER {
            let Some(r) = score(row) else { return };
            pinned(row, &r);
        }
    }

    /// **The ladder's borrowed setup is the endpoint's own** — the three
    /// East Indies simulations, built from three captures' start blocks,
    /// are one simulation.
    ///
    /// run54 carries its own world cells and per-tile masks; run24 and run28
    /// were taken for a closing state and write the world block's seventeen
    /// scalars and stop, so their start dumps borrow the rest from run38's
    /// through [`borrow_from_siblings`]. That borrow is not a formality: the
    /// first run of the ladder used the Great Lakes sibling set, nothing
    /// matched, and run28 walked a region-less world to **12 units compared
    /// of 71** with its AI leader 24,789 units off. A setup that wrong is
    /// still a number, and it would have gone on the scoreboard.
    ///
    /// The three captures are the same game to 12,000 (see
    /// `the_endpoint_captures_are_four_families`), and a simulation knows
    /// nothing of the split that comes after — it is built from a frame-0
    /// state and ticks. So the three builds are the *same simulation*, and
    /// 6,000 frames is where that is checked: under East Indies' word
    /// (7,529), so agreement here is agreement on a stretch the original
    /// has already ratified.
    #[test]
    fn the_ladder_s_borrowed_setup_is_the_endpoint_s() {
        const AT: i64 = 6_000;
        /// One simulation's units at [`AT`]: `(who, o, (x, y))`, sorted.
        type Roster = Vec<(i64, i64, (i32, i32))>;
        let rows: Vec<&Endpoint> = std::iter::once(&ENDPOINTS[0]).chain(&LADDER).collect();
        let mut states: Vec<(&str, Roster)> = Vec::new();
        for row in rows {
            let Some(inst) = install() else { return };
            let (Some(path), Some(tr)) = (
                dump(&format!("gamelog-{}.txt", row.capture)),
                trace(row.trace),
            ) else {
                eprintln!("skipping: set RON_GAMELOG_DIR");
                return;
            };
            let text = crate::capture::read(&path);
            let log = Log::parse(&text);
            let loaded = crate::load::load(&inst).unwrap();
            let sib_texts: Vec<String> = row
                .setup
                .iter()
                .filter_map(|n| dump(n))
                .map(crate::capture::read)
                .collect();
            let sib_logs: Vec<Log> = sib_texts.iter().map(|t| Log::parse(t)).collect();
            let inits: Vec<Initial> = sib_logs.iter().filter_map(|l| l.initial()).collect();
            let refs: Vec<&Initial> = inits.iter().collect();
            let mut init = log.initial().expect("a start block");
            borrow_from_siblings(&mut init, &refs);
            borrow_pasture(&mut init, &tr);
            check_setup(row, &init);
            let mut built = build_sim(&loaded, &init, Tuning::RON);
            while built.sim.frame < AT {
                built.tick();
            }
            let mut state: Roster = built
                .sim
                .units
                .iter()
                .filter(|u| u.alive() && u.owner < 8)
                .map(|u| (i64::from(u.owner), i64::from(u.index), (u.pos.x, u.pos.y)))
                .collect();
            state.sort_unstable();
            eprintln!("{}: {} units at {AT}", row.capture, state.len());
            states.push((row.capture, state));
        }
        assert!(
            states.len() >= 2,
            "the ladder's setup check needs at least two captures"
        );
        // The count is pinned rather than floored: 24 is what East Indies
        // holds for its two real players at 6,000, and a setup that lost
        // the map does not land on it by accident.
        assert_eq!(
            states[0].1.len(),
            24,
            "{} stands up {} units at {AT}, not the 24 East Indies has there",
            states[0].0,
            states[0].1.len()
        );
        for (name, state) in &states[1..] {
            assert_eq!(
                state, &states[0].1,
                "{name}'s borrowed setup is not {}'s at {AT}",
                states[0].0
            );
        }
    }

    /// **The endpoint captures are four families**, and the ladder's rungs
    /// belong to two of them rather than to the endpoint's run.
    ///
    /// `Trace::frames` carries `game_random`'s word at every `do_frame`
    /// entry, so two captures of the same game agree on it frame for frame.
    /// This is `tools/gamelog/rngcmp.py` as an assertion, and it is here
    /// because the ladder's shape rests on it: walking run54's simulation to
    /// 15,401 and reading run28's closing dump there would compare two
    /// different games, and every count it produced would be noise wearing
    /// the endpoint's clothes.
    ///
    /// The three East Indies families are one game to 12,000 and part on
    /// **12,001**; B and C stay together to 15,020 and part on **15,021**.
    /// Both are pinned, because "they differ somewhere" and "they differ from
    /// the frame a driven run was interfered with" are different claims and
    /// only the second explains the families.
    #[test]
    fn the_endpoint_captures_are_four_families() {
        // Each family's captures, and the frame the *next* family parts on.
        let families: &[(&str, &[&str])] = &[
            ("GreatLakes", &["run53", "run18a"]),
            ("EastIndies A", &["run54", "run21", "run23"]),
            ("EastIndies B", &["run24", "run25", "run26", "run27"]),
            ("EastIndies C", &["run28", "run29"]),
        ];
        let mut words: BTreeMap<&str, BTreeMap<i64, u32>> = BTreeMap::new();
        for (_, runs) in families {
            for run in *runs {
                // One trace at a time: `Trace::parse` builds every draw, and
                // eleven of these at once is a gigabyte for no reason.
                let Some(t) = trace(&format!("rontrace-{run}.log")) else {
                    eprintln!("skipping: no rontrace-{run}.log (set RON_GAMELOG_DIR)");
                    return;
                };
                words.insert(run, t.frames.iter().copied().collect());
            }
        }
        // The word's own length, so a truncated trace cannot pass by holding
        // no frames either side disagrees on.
        for (run, w) in &words {
            let last = w.keys().next_back().copied().unwrap_or(0);
            assert!(
                last >= 15_000,
                "rontrace-{run}.log reaches frame {last}; every capture here is 15,000+"
            );
        }
        // First differing frame over the overlap, `None` when they agree.
        let part = |a: &str, b: &str| -> Option<i64> {
            let (x, y) = (&words[a], &words[b]);
            x.iter()
                .filter(|(f, _)| y.contains_key(f))
                .find(|(f, w)| y[f] != **w)
                .map(|(f, _)| *f)
        };
        for (name, runs) in families {
            for other in &runs[1..] {
                assert_eq!(
                    part(runs[0], other),
                    None,
                    "{name}: {other} is not the same game as {}",
                    runs[0]
                );
            }
        }
        assert_eq!(
            (
                part("run54", "run24"),
                part("run54", "run28"),
                part("run24", "run28"),
            ),
            (Some(12_001), Some(12_001), Some(15_021)),
            "the East Indies families do not part where they did: A|B and A|C \
             on 12001, B|C on 15021"
        );
    }

    /// **[`GREAT_LAKES_SETUP`] is the suite's own sibling set**, named
    /// rather than borrowed so that a row can say which map it takes — and
    /// checked here so the two cannot drift apart in silence.
    #[test]
    fn the_setup_lists_are_the_suite_s() {
        // Compare the contract, not gigabytes of file contents. This also
        // checks absent captures and distinct names with identical contents.
        assert_eq!(
            GREAT_LAKES_SETUP,
            crate::diff::testkit::SIBLING_DUMPS,
            "GREAT_LAKES_SETUP names different files from testkit::sibling_texts"
        );
    }

    /// **The queue's handoff states the endpoint, verbatim** — the
    /// `Endpoint:` line against [`ENDPOINTS`]. Static: no install, no dump,
    /// every machine, exactly as `the_handoff_s_scoreboard_is_the_floors` is
    /// for the other two lines.
    ///
    /// Only `off` and `unlinked` are on the line. The other five counts are
    /// pinned in [`ENDPOINTS`] and held by the same pin; putting all
    /// seven in the handoff would cost four lines of a section bounded at
    /// thirty-two to say what one failure message says better.
    #[test]
    fn the_handoff_s_endpoint_is_the_pinned_counts() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/QUEUE.md");
        let q = std::fs::read_to_string(path).expect("docs/QUEUE.md");
        let line = q.lines().find(|l| l.starts_with("Endpoint ")).expect(
            "docs/QUEUE.md has no `Endpoint ` line in the handoff; write \
             `Endpoint <frame>: <map> <off> off/<unlinked> unlinked · <map> …`, \
             one part per row of rondata::diff::ENDPOINTS, in order",
        );
        let (head, body) = line.split_once(':').expect("`Endpoint <frame>: …`");
        assert_eq!(
            head.trim_start_matches("Endpoint ").trim().replace(',', ""),
            ENDPOINT_FRAME.to_string(),
            "the handoff's `Endpoint` line names a frame that is not ENDPOINT_FRAME"
        );
        let mut stated = Vec::new();
        for part in body.split('\u{b7}') {
            let t: Vec<&str> = part.split_whitespace().collect();
            // The number in front of each keyword, whatever punctuation the
            // line's prose puts after it.
            let before = |kw: &str| -> usize {
                t.iter()
                    .position(|w| w.trim_end_matches([',', ';']) == kw)
                    .and_then(|i| i.checked_sub(1))
                    .and_then(|i| t[i].parse::<usize>().ok())
                    .unwrap_or_else(|| {
                        panic!(
                            "unreadable endpoint part {part:?}: no count before {kw:?}; \
                             want `<map> <n> off, <n> unlinked`"
                        )
                    })
            };
            assert!(!t.is_empty(), "empty endpoint part in {line:?}");
            stated.push((t[0].to_string(), before("off"), before("unlinked")));
        }
        let pinned: Vec<_> = ENDPOINTS
            .iter()
            .map(|e| (e.map.to_string(), e.off, e.unlinked))
            .collect();
        assert_eq!(
            stated, pinned,
            "the handoff's endpoint line is not the pinned counts: left is the \
             queue's line, right is rondata::diff::ENDPOINTS"
        );
    }
}
