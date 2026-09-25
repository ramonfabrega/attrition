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
        // Then **66 → 73, 16 → 7, 0 → 1 and 30 → 28** on 2026-09-17, item
        // 323's scholar gate — the same row as Great Lakes' below, and
        // **this map's word does not move on it**. The fix is in
        // `create_units` and is not map-specific, so East Indies takes the
        // same unlinked-down, off-up shape 16,000 frames past its own
        // word: nine more of the roster linked, seven positions out, one
        // building unlinked and two building field-rows closer.
        // **73 → 66, 7 → 13, 1 → 0 and 28 → 30** on 2026-09-17, item 334 —
        // `Build::process@0061edf0`'s `is_active` gate above the road
        // replan (`docs/ROADS.md` §1.1), which moves this map's own word
        // 8193 → 8466. Seven positions closer, six spurious units, one
        // building linked and two more field-rows out, 15,535 frames past
        // the word. DECISIONS 36: the number, not a trade.
        // And **66 → 62, 13 → 10 and 30 → 29** on 2026-09-18, item 338
        // — the **scholar's seating** (`docs/CITIES.md` §6.5.2).
        // `Unit::go_inside`'s tail snaps an `is_scholar` unit onto its
        // host, and `Build::train` keeps a scholar trained at a
        // university inside it; every scholar
        // in the game therefore stops standing on the exit ring. That is
        // fourteen units on Great Lakes and the same mechanism on East
        // Indies, 15,500 frames past both words — which the change moves
        // together, 8272 → 8374 and 8466 → 8495. DECISIONS 36 asks for the
        // number rather than a trade.
        // And **62 → 61 and 10 → 11** on 2026-09-18, item 340 — the **scholar's teach
        // slot** (`docs/ANIM.md` §4.11). `Guy::set_anim`'s `guy_flags &
        // 0x80` arm turns the idle roll's variant into an offset into
        // slots 25–32 for a scholar inside its host, and
        // `ObjectData::is_peasant` is `TypeIndex` `0x32`/`0x33` rather
        // than any worker, so both the variant and the slot the seated
        // scholar plays change. Every scholar in the game runs a
        // different animation from the frame it sits down, 15,500 frames
        // before this block. Great Lakes' word moves 8374 → 8382 on it
        // and East Indies' does not move. DECISIONS 36 asks for the
        // number rather than a trade.
        // Then **61 → 70 off, 11 → 2 unlinked and 29 → 31
        // build_diverged** on 2026-09-18, item 344 — the mine's mountain
        // range (`docs/ECONOMY.md`, "The mine's range"). The first mine
        // on each map now claims its range's tiles as gathered from,
        // which re-deals every gatherer's assignment from the frame it is
        // placed on. Great Lakes' word moves 8382 → 8404 on it; East
        // Indies' does not move. DECISIONS 36 asks for the number rather
        // than a trade.
        // And **70 → 61 off, 2 → 8 unlinked, 0 → 1 build_unlinked and
        // 31 → 30 build_diverged** on 2026-09-18, item 346 — a scholar's
        // slot from `0x19` up is the idle category (`docs/ANIM.md` §5.1),
        // which moves this map's own word 8495 → 9711, the largest single
        // step it has taken. Nine positions closer and one building field
        // closer, six more of the roster unlinked and one building the
        // simulation cannot link, 14,290 frames past the new word.
        // DECISIONS 36 asks for the number rather than a trade.
        // And **8 → 11 unlinked, 1 → 0 build_unlinked and 30 → 32
        // build_diverged** on 2026-09-18, item 348 — the market's ability
        // is **Coinage**, not the Market building (`docs/AI.md` §40), so
        // the AI's `use_market` gate opens a whole library tech later and
        // its one draw joins the stream. East Indies' word does not move
        // (9711 either way) and its `off` holds at 61; three more of the
        // roster unlinked and two more building field-rows out, against
        // the one building the simulation could not link now linked,
        // 14,290 frames past the word. DECISIONS 36 asks for the number
        // rather than a trade.
        // And **61 → 59 off and 11 → 12 unlinked** on 2026-09-18, item 352
        // — `find_muster_spot`'s inclusive spacing bound and `mask_me`'s
        // `0x4000` cell flag (`docs/ARMY.md` §13). Neither is
        // map-specific: the flag is written wherever a building starts and
        // the bound is read by every army's ring, so this map's AI musters
        // differently too even though its word does not move (9711 either
        // way). Two positions closer and one more of the roster unlinked,
        // 14,290 frames past the word. DECISIONS 36 asks for the number
        // rather than a trade.
        // And **59 → 61 off and 12 → 11 unlinked** on 2026-09-18, item
        // 354 — `find_wpath`'s `army` mode, whose `is_attacking` clause is
        // a `return 0` in every order vtable the executable ships
        // (`docs/PATHFINDER.md` §22). An AI army now pays 32× for a
        // `NEARBLOCK` cell again, so every military route on both maps is
        // re-planned; this map's word does not move (9711 either way) and
        // its endpoint reshuffles 14,290 frames past it. DECISIONS 36 asks
        // for the number rather than a trade.
        // Then **61 → 58 off, 11 → 14 unlinked, 0 → 1 build_unlinked and
        // 32 → 30 build_diverged** on 2026-09-18, item 358 — the market's
        // **trade** (`docs/ECONOMY.md` §12), which moves Great Lakes' word
        // 8985 → 9134. Every AI leader on both maps can now sell a hundred
        // of a good for wealth, so East Indies' whole economy is re-dealt
        // 14,290 frames before this block even though its own word does
        // not move: three positions closer, three more of the roster
        // unlinked, and two building field-rows closer. DECISIONS 36 asks
        // for the number rather than a trade.
        // And **58 → 64 off, 14 → 5 unlinked, 1 → 0 build_unlinked and
        // 30 → 32 build_diverged** on 2026-09-18, item 362 — the make
        // list's **`num`** (`docs/AI.md` §42). `create_units` sets a
        // batch size per arm and this crate's civilian arms threw it
        // away, so every scholar, caravan and citizen the AI has ever
        // offered itself was a batch of one; carrying it changes what
        // the AI buys from its first University onwards on both maps.
        // Nine fewer units the dump has and this crate has not, one
        // fewer unlinkable building — and six more positions out and two
        // more building field-rows, 14,000 frames past this map's word,
        // which does not move. DECISIONS 36 asks for the number rather
        // than a trade.
        // And **64 → 63 off, 10 → 11 unlinked, 0 → 1 build_unlinked and
        // 32 → 28 build_diverged** on 2026-09-21, item 478 —
        // `City::regen_roads`' second writer (`docs/ROADS.md` §1.2). This
        // map's own word does **not** move (9711 either side); the change
        // is to when a city replans its roads after a building of it
        // dies, which every map reaches, and the roads it then lays are
        // what the roster 14,290 frames later is standing on. One
        // position and four building field-rows closer against one more
        // of the roster unlinked and one building unlinked. DECISIONS 36
        // asks for the number rather than a trade.
        // And **63 → 59 off, 11 → 12 unlinked and 28 → 30
        // build_diverged** on 2026-09-22, item 497 — the scholar teach
        // slot's tie-break (`docs/ANIM.md` §4.12). This map's own word
        // does **not** move (9711 either side) and nothing about the
        // change is East Indies'; a scholar's fourth student slot is
        // 118 frames long and its first 30, so a chain that entered the
        // wrong one held its figure three wraps' worth of draws out of
        // the stream, on every map with a university. Four positions
        // closer against one more of the roster unlinked and two more
        // building field-rows, 14,290 frames past this map's word.
        // DECISIONS 36 asks for the number rather than a trade.
        // And **59 → 64 off, 12 → 10 unlinked, 1 → 2 build_unlinked
        // and 30 → 27 build_diverged** on 2026-09-22, item 506 —
        // `market_speculation`'s buy and sell passes
        // (`docs/ECONOMY.md` §13). This map's own word does **not**
        // move (9711 either side) and nothing about the change is East
        // Indies': the AI speculates on every map with a market and
        // Coinage, so a leader that now buys and sells off its
        // shortfall enters every later rotation holding a different
        // purse. Two of the roster linked back and three building
        // field-rows closer against five positions further out, 14,290
        // frames past this map's word. DECISIONS 36 asks for the number
        // rather than a trade; the number is on the headline, and the
        // headline is Great Lakes'.
        // And **64 → 63 off and 2 → 1 build_unlinked against 27 → 30
        // build_diverged** on 2026-09-22, item 515 — the group speed cap
        // (`docs/GROUPS.md` §18). This map's own word does **not** move
        // (9711 either side) and nothing about the change is East
        // Indies': every map marches formations, so a member that now
        // walks at its group's pace rather than its own arrives
        // elsewhere on every map. One position and one building back
        // against three building field-rows, 14,290 frames past this
        // map's word. DECISIONS 36 asks for the number rather than a
        // trade; the number is on the headline, and the headline is
        // Great Lakes'.
        // **63 → 62 on item 518**, `Groups::process` (`docs/GROUPS.md`
        // §19): one position closer; this map's own word does not move.
        // **62 → 64 and 10 → 9** on item 327, the Merchant offer
        // (`docs/AI.md` §55). This map's word holds at 9711; the endpoint
        // reshuffles 14,000 frames past it. DECISIONS 36: the number, not a trade.
        // **64 → 67 off and 9 → 3 unlinked** on item 545, `get_cost`'s
        // military discount and research arm (`docs/AI.md` §56), which move
        // Great Lakes' word 11582 → 11757. Every military price changes and
        // the AI researches unit upgrades it never could; this map's word
        // holds at 9711, and the 24,000th frame reshuffles 14,290 past it:
        // six units linked, three positions out. DECISIONS 36.
        // **67 → 63 off** on item 557, the group back-pointer
        // (`docs/GROUPS.md` §23). This map's word holds at 9711; every army's
        // list now follows `Group::add`'s per-step `get_num` and `Group::sort`,
        // so the 24,000th frame reshuffles 14,290 past it: four positions
        // closer against five of the roster unlinked. DECISIONS 36.
        // **63 → 69 off** on item 573, the caravan's one-off read at the
        // Commerce level (`docs/CARAVAN.md` §9). The AI has twenty more
        // wealth from 6512, buys its sixth Scholar on 9576, and this map's
        // word moves 9711 → 9983; the 24,000th frame reshuffles 14,018
        // past it: six positions out against six of the roster linked.
        // DECISIONS 36.
        // **69 → 45 off, 2 → 22 unlinked, 1 → 5 build_unlinked and 30 → 18
        // build_diverged** on item 576, the sea branch's dock (`docs/AI.md`
        // §57): the AI offers ships at a Dock that belongs to no city from
        // 9981, and this map's word moves 9983 → 10232. The 24,000th frame
        // reshuffles 13,769 past it: twenty-four positions closer, twenty
        // of the roster unlinked (`1/61`..`1/82`). DECISIONS 36: the
        // number, not a trade.
        // **45 → 57 off, 22 → 5 unlinked, 5 → 1 build_unlinked and 18 → 25
        // build_diverged** on item 579 (`docs/ORDERS.md` §25): the dock's
        // margin, the warship clause and the navy, and this map's word
        // moves 10232 → 10398. The 24,000th frame reshuffles 13,603 past
        // it. DECISIONS 36: the number, not a trade.
        // **57 → 62 off, 5 → 2 unlinked, 1 → 2 build_unlinked** on item 588,
        // the sea half of `detect_unit_collision`'s second arm
        // (`docs/COLLISION.md` §13), 13,419 frames past this map's new word
        // 10582. DECISIONS 36: the number, not a trade.
        // **62 → 64 off, 2 → 0 unlinked, 25 → 24 build_diverged** on item
        // 592, the census counting a barge's rider through its container
        // (`docs/AI.md` §58); this map's word holds at 10582, 13,419
        // frames before this one. DECISIONS 36: the number, not a trade.
        // **64 → 59 off, 0 → 3 unlinked** on item 597, a mine's reach
        // measured to the nearest solid mountain cell (`docs/AI.md` §59);
        // this map's word holds at 10582, 13,419 frames before this one.
        // DECISIONS 36: the number, not a trade.
        // **59 → 56 off, 3 → 6 unlinked, 2 → 1 build_unlinked, 24 → 25
        // build_diverged** on item 604, the placed mountain templates'
        // solid cells (`docs/AI.md` §60); this map's word moves 10582 →
        // 10782, 13,219 frames before this one. DECISIONS 36: the number,
        // not a trade.
        // **56 → 64 off, 6 → 0 unlinked, 0 → 1 extra, 25 → 24
        // build_diverged** on item 608, a city counting an unfinished
        // gather building's slots (`docs/AI.md` §61); this map's word moves
        // 10782 → 10982, 13,019 frames before this one. The extra is a
        // Bowmen, `1/83`. DECISIONS 36: the number, not a trade.
        // **64 → 59 off, 0 → 3 unlinked, 1 → 0 extra, 1 → 2
        // build_unlinked, 24 → 22 build_diverged** on item 613, Horses'
        // discount on a Stable unit and a mined range taken on the survey
        // (`docs/AI.md` §62); this map's word moves 10982 → 11069, 12,932
        // frames before this one. The unlinked are player 1's `80`..`82`.
        // DECISIONS 36: the number, not a trade.
        // **59 → 52 off, 3 → 6 unlinked, 2 → 1 build_unlinked, 22 → 24
        // build_diverged** on item 620, a Mine's `dist_mod` capped at 3 on
        // a list under `MTN_TINY_SIZE` (`docs/ORDERS.md` §6.4); this map's
        // word moves 11069 → 11590, 12,411 frames before this one. The
        // unlinked are player 1's `77`..`82`. DECISIONS 36: the number, not
        // a trade.
        // **52 → 49 off, 6 → 8 unlinked, 1 → 2 build_unlinked, 24 → 22
        // build_diverged** on item 629, a deployed Merchant seated on its
        // tile corner with its square blocked (`docs/MERCHANT.md` §3.2);
        // this map's word moves 11590 → 11747, 12,254 frames before this
        // one. The unlinked are player 1's `75`..`82`. DECISIONS 36: the
        // number, not a trade.
        // **49 → 45 off, 8 → 12 unlinked, 22 → 18 build_diverged** on
        // item 642: `Region::go_here` reads the human's city count, so the
        // scout sails for the human's island (`docs/TRANSPORT.md` §9.4).
        // This map's word moves 11747 → 13640, 10,361 frames before this
        // one. The unlinked are player 1's `71`..`82`. DECISIONS 36: the
        // number, not a trade.
        // **45 → 43 off, 12 → 11 unlinked** on item 673: the retry a failed
        // unit-grid search buys is gated on `is_move`, not on a move
        // without the action bit (`docs/PATHFINDER.md` §21.6). This map's
        // word holds at 13640, 10,361 frames before this one. The unlinked
        // are player 1's `72`..`82`. DECISIONS 36: the number, not a trade.
        // **43 → 46 off, 11 → 10 unlinked, 2 → 1 build_unlinked** on item
        // 643: an animation's name is found case-folded, so the Galley's
        // default plays 20 frames, not 3 (`docs/ANIM.md` §12). This map's
        // word moves 13640 → 15782, 8,219 frames before this one. The
        // unlinked are player 1's `73`..`82`. DECISIONS 36: the number, not
        // a trade.
        // **46 → 55 off, 10 → 0 unlinked, 0 → 2 extra, 1 → 2
        // build_unlinked, 18 → 16 build_diverged** on item 688: a founded
        // city quarters and halves the site values round it
        // (`City::fix_world_vals`, `docs/AI.md` §67). This map's word holds
        // at 15782, 8,219 frames before this one. DECISIONS 36: the number,
        // not a trade.
        // **55 → 47 off, 0 → 7 unlinked, 3 → 0 extra, 2 → 1
        // build_unlinked, 16 → 15 build_diverged** on item 706: every
        // nation's graft table and the Senate's government patriot
        // (`docs/TECH.md` §"The graft table", §"The government patriot"),
        // which move this map's word 15782 → 15985, 8,016 frames before
        // this one. Measured after item 703's merge. The unlinked are
        // player 1's `76`..`82`. DECISIONS 36: the number, not a trade.
        // **47 → 52 off, 7 → 3 unlinked, 15 → 16 build_diverged** on item
        // 711: a unit's own mirror, `unit_masks & 2`, which `do_guard`
        // reads off its target (`docs/GROUPS.md` §25). This map's word
        // holds at 15985. Measured on the tree before item 713's merge
        // and again after it, with the same counts. The unlinked are
        // player 1's `80`..`82`. DECISIONS 36: the number, not a trade.
        // **52 → 55 off** on item 715: `Wall::process`'s site recruiter
        // (`docs/AI.md` §69), which moves Great Lakes' word 15383 → 15384;
        // this map's holds at 15985. Measured after item 714's merge. The
        // unlinked are still player 1's `80`..`82`. DECISIONS 36: the
        // number, not a trade.
        // **55 → 51 off, 3 → 4 unlinked, 1 → 3 build_unlinked, 16 → 10
        // build_diverged** on item 722: `CityData::num_wonders` counts a
        // wonder site (`docs/AI.md` §70), which moves Great Lakes' word
        // 15384 → 15608; this map's holds at 15985 with its delta. The
        // unlinked are player 1's `79`..`82`. Measured on the spawn base,
        // `2355946`. DECISIONS 36: the number, not a trade.
        // **51 → 49 off, 10 → 12 build_diverged** on item 736:
        // `action_siege_attack_to`'s sub-group lays out on its own cleared
        // record (`docs/GROUPS.md` §26), which moves Great Lakes' word
        // 15619 → 16460; this map's holds at 15985. The unlinked are still
        // player 1's `79`..`82`. Measured after `ccc update` onto `8e75c3b`.
        // DECISIONS 36: the number, not a trade.
        // **49 → 50 off, 4 → 0 unlinked, 0 → 3 extra, 12 → 8
        // build_diverged** on item 708: the commerce cap's republic term
        // (`docs/AI.md` §72), which moves this map's word 15985 → 16683,
        // 7,318 frames before this one. The three extra are player 1's
        // `83`..`85` Longbowmen. Measured after `ccc update` onto
        // `df68f0d`. DECISIONS 36: the number, not a trade.
        off: 50,
        // And **5 → 10 unlinked** on 2026-09-21, item 442 — the scholar
        // arm's `val` chain (`docs/AI.md` §53). This map's word does not
        // move on it; Great Lakes' moves 9510 → 10161. The AI's Scholar
        // valuation is per city everywhere, so East Indies' 24,000th frame
        // drifts too, 14,000 frames past its own word, and its two ladder
        // rungs shed **eight** and **seven** spurious units — the Citizens
        // this crate used to buy instead of Scholars. DECISIONS 36 asks for
        // the number rather than a trade; the number is on the headline.
        // **3 → 8 unlinked** on item 557, beside 67 → 63 off above.
        // **8 → 2 unlinked** on item 573, beside 63 → 69 off above.
        // **2 → 22 unlinked** on item 576, beside 69 → 45 off above.
        // **22 → 5 unlinked** on item 579, beside 45 → 57 off above.
        // **5 → 2** on item 588, beside `off` above: player 1's `81` and `82`.
        // **2 → 0** on item 592, beside `off` above.
        // **0 → 3** on item 597, beside `off` above: player 1's `77`, `81`
        // and `82`.
        // **3 → 6** on item 604, beside `off` above: player 1's `77`..`82`.
        // **6 → 0** on item 608, beside `off` above.
        // **0 → 3** on item 613, beside `off` above.
        // **3 → 6** on item 620, beside `off` above.
        // **8 → 12** on item 642, beside `off` above.
        // **12 → 11** on item 673, beside `off` above.
        // **11 → 10** on item 643, beside `off` above.
        // **10 → 0** on item 688, beside `off` above.
        // **0 → 7** on item 706, beside `off` above.
        // **7 → 3** on item 711, beside `off` above.
        // **3 → 4** on item 722, beside `off` above.
        // **4 → 0** on item 708, beside `off` above.
        unlinked: 0,
        // **0 → 1** on item 608, beside `off` above.
        // **1 → 0** on item 613, beside `off` above.
        // **0 → 2** on item 688, beside `off` above.
        // **2 → 3** on item 703: a pushed unit's collision disc waits for
        // its figure (`docs/COLLISION.md` §16). This map's word holds at
        // 15782, 8,219 frames before this one; `off` holds at 55. The
        // three are player 1's `83` Bowmen and `84`, `85` Transport
        // Barges. DECISIONS 36: the number, not a trade.
        // **3 → 0** on item 706, beside `off` above.
        // **0 → 3** on item 708, beside `off` above.
        extra: 3,
        // **1 → 2** on item 588, beside `off` above.
        // **2 → 1** on item 604, beside `off` above.
        // **1 → 2** on item 613, beside `off` above.
        // **2 → 1** on item 620, beside `off` above.
        // **2 → 1** on item 643, beside `off` above.
        // **1 → 2** on item 688, beside `off` above.
        // **2 → 1** on item 706, beside `off` above.
        // **1 → 3** on item 722, beside `off` above.
        build_unlinked: 3,
        // **25 → 24** on item 592, beside `off` above.
        // **24 → 25** on item 604, beside `off` above.
        // **25 → 24** on item 608, beside `off` above.
        // **24 → 22** on item 613, beside `off` above.
        // **22 → 24** on item 620, beside `off` above.
        // **22 → 18** on item 642, beside `off` above.
        // **18 → 16** on item 688, beside `off` above.
        // **16 → 15** on item 706, beside `off` above.
        // **15 → 16** on item 711, beside `off` above.
        // **16 → 10** on item 722, beside `off` above.
        // **10 → 12** on item 736, beside `off` above.
        // **12 → 8** on item 708, beside `off` above.
        build_diverged: 8,
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
        // Then **59 → 55, 23 → 28 and 12 → 10** on 2026-09-17, items 312
        // and 314 — the `get_loc` seam (GROUPS §12, PATHFINDER §3), which
        // moves this map's own word **7679 → 7930**. Four positions closer
        // and two building field-rows closer, 16,071 frames past the word;
        // five more of the roster unlinked in exchange. DECISIONS 36 asks
        // for the number rather than a trade.
        // Then **55 → 60 and 28 → 20** on 2026-09-17, item 317 — the
        // `do_marching` empty-target arm (ARMY §18), which moves this map's
        // own word **7930 → 8030** and takes its draw-for-draw frame count
        // 8193 → 8318. Five positions out and **twenty units that had no
        // counterpart at 24001 now have one**, 15,971 frames past the word:
        // a change to what the army *targets* relinks a fifth of the
        // roster. DECISIONS 36 asks for the number rather than a trade.
        // Then **60 → 59, 20 → 23 and 10 → 8** on 2026-09-17, item 319 —
        // `scout_danger` answering a flat 0 (SCOUT, MOVEMENT), which moves
        // this map's own word 8030 → 8031. One position closer and two
        // building field-rows closer, 15,970 frames past the word; three
        // more of the roster unlinked in exchange. A one-frame word move
        // and a three-count endpoint move on the same change, which is
        // what a dense region looks like from both ends. DECISIONS 36 asks
        // for the number rather than a trade.
        // Then **59 → 56, 23 → 24 and 8 → 9** on 2026-09-17, item 322 —
        // `Wall::check_ever_seen`'s reveal (VISION §6.1), which moves this
        // map's own word **8031 → 8186**, its largest single step. Three
        // positions closer 15,815 frames past the word, against one more
        // of the roster unlinked and one more building field-row apart.
        // DECISIONS 36 asks for the number rather than a trade.
        // Then **56 → 66, 24 → 17 and 9 → 10** on 2026-09-17, item 323 —
        // `civilian_value`'s scholar gate reading the unhalved
        // `get_mod_resource_cap` (AI §38), which moves this map's floor
        // 8182 → 8186. **Unlinked down and off up on both maps**: the AI
        // now trains units it was skipping, so they link and then differ
        // in position rather than being absent, which is a different kind
        // of row from the ten before it. DECISIONS 36 asks for the number
        // rather than a trade.
        // Then **66 → 67, 17 → 14 and 10 → 9** on 2026-09-17, item 328's
        // `find_attack_pos` (COMBAT §17), which moves this map's word
        // 8186 → 8187. Two of the three improve: three more of the roster
        // linked and one building field-row closer against one position
        // out, 15,814 frames past the word. The ladder did not move.
        // Then **67 → 64, 14 → 15 and 9 → 10** on 2026-09-17, item 329 —
        // the chase planning on its order frame and the unit-grid failure
        // roll (ORDERS §7.10, PATHFINDER §21), which takes this map's word
        // 8187 → **8201** and past East Indies. Three positions closer,
        // 15,800 frames past the word, against one of the roster unlinked
        // and one building field-row out. DECISIONS 36 asks for the number
        // rather than a trade.
        // **64 → 69 and 15 → 13** on 2026-09-17, item 334 — the same
        // `is_active` gate (`docs/ROADS.md` §1.1). Great Lakes' own word
        // does not move on it; its endpoint takes five positions out and
        // two spurious units off, 15,800 frames past the word.
        // DECISIONS 36: the number, not a trade.
        // Then **69 → 64 and 13 → 17** on 2026-09-17, item 336 — the
        // cell-centre snap at `Unit::fight`'s entry (`docs/ORDERS.md`
        // §7.11), which takes this map's word **8201 → 8272**. Five
        // positions closer 15,729 frames past the word, against four more
        // of the roster unlinked; the building and city rows do not move.
        // East Indies' own endpoint is unchanged on the same commit, which
        // is what says this is a Great Lakes cast rather than a shared
        // reshuffle. DECISIONS 36 asks for the number rather than a trade.
        // And **64 → 61 and 17 → 15** on 2026-09-18, item 338 — the
        // **scholar's seating** (`docs/CITIES.md` §6.5.2).
        // `Unit::go_inside`'s tail snaps an `is_scholar` unit onto its
        // host, and `Build::train` keeps a scholar trained at a
        // university inside it; every scholar
        // in the game therefore stops standing on the exit ring. That is
        // fourteen units on Great Lakes and the same mechanism on East
        // Indies, 15,500 frames past both words — which the change moves
        // together, 8272 → 8374 and 8466 → 8495. DECISIONS 36 asks for the
        // number rather than a trade.
        // And **61 → 62 and 10 → 9** on 2026-09-18, item 340 — the **scholar's teach
        // slot** (`docs/ANIM.md` §4.11). `Guy::set_anim`'s `guy_flags &
        // 0x80` arm turns the idle roll's variant into an offset into
        // slots 25–32 for a scholar inside its host, and
        // `ObjectData::is_peasant` is `TypeIndex` `0x32`/`0x33` rather
        // than any worker, so both the variant and the slot the seated
        // scholar plays change. Every scholar in the game runs a
        // different animation from the frame it sits down, 15,500 frames
        // before this block. Great Lakes' word moves 8374 → 8382 on it
        // and East Indies' does not move. DECISIONS 36 asks for the
        // number rather than a trade.
        // Then **15 → 7 unlinked and 9 → 10 build_diverged** on
        // 2026-09-18, item 344 — the mine's mountain
        // range (`docs/ECONOMY.md`, "The mine's range"). The first mine
        // on each map now claims its range's tiles as gathered from,
        // which re-deals every gatherer's assignment from the frame it is
        // placed on. Great Lakes' word moves 8382 → 8404 on it; East
        // Indies' does not move. DECISIONS 36 asks for the number rather
        // than a trade.
        // And **62 → 58 off, 7 → 11 unlinked and 10 → 5 build_diverged**
        // on 2026-09-18, item 346 — the same one change, which moves this
        // map's word 8404 → 8582 (`docs/ANIM.md` §5.1). Four positions and
        // five building fields closer, four more of the roster unlinked,
        // 15,419 frames past the new word. DECISIONS 36 asks for the
        // number rather than a trade.
        // And **58 → 63 off, 11 → 7 unlinked and 5 → 8 build_diverged** on
        // 2026-09-18, item 348 — the market's ability is **Coinage**
        // (`docs/AI.md` §40), which moves this map's word 8582 → 8619.
        // Four fewer of the roster unlinked, five positions and three
        // building field-rows further out, 15,382 frames past the new
        // word. DECISIONS 36 asks for the number rather than a trade.
        // And **63 → 58 off and 8 → 7 build_diverged** on 2026-09-18, item
        // 350 — `Groups::push_group`'s kill out of the old group
        // (`docs/ARMY.md` §3.4), which moves this map's word 8619 → 8628.
        // The AI's army holds nine units from 8186 where it held fifteen,
        // so the six of §12's probe stop being re-ordered every time the
        // army retargets: five positions and one building field-row
        // closer 15,373 frames past the new word, and nothing else on the
        // row moves. East Indies' endpoint and both ladder rungs are
        // unchanged on the same commit — this map's AI is the only one
        // whose army has run the probe. DECISIONS 36 asks for the number
        // rather than a trade.
        // And **58 → 60 off, 7 → 8 unlinked and 7 → 6 build_diverged** on
        // 2026-09-18, item 352 — §13's ring on the muster cell the
        // original picks (`docs/ARMY.md` §16.9), which moves this map's
        // word 8628 → 8663. The army goes to (58, 29) rather than (50,
        // 27) from 8442, so nine units walk a different way across the
        // map for the 15,000 frames after it: two positions further out
        // and one more of the roster unlinked, one building field-row
        // closer, 15,338 frames past the new word. **The scholar on the
        // wrong university moved with it**, `1/53` to `1/52`, which
        // `great_lakes_scholars_sit_on_their_universities` re-pins by
        // name. DECISIONS 36 asks for the number rather than a trade.
        // Then **60 → 67 off, 8 → 0 unlinked, 0 → 1 extra and 6 → 7
        // build_diverged** on 2026-09-18, item 354 — the same `army` mode
        // (`docs/PATHFINDER.md` §22), which moves this map's word 8663 →
        // 8985. The whole roster links now where eight units used to go
        // missing, and the eight land in `off` with the rest; the counts
        // move in every column because a routing change 15,000 frames
        // before the endpoint re-deals every position after it.
        // DECISIONS 36 asks for the number rather than a trade.
        // Then **67 → 57 off, 0 → 6 unlinked, 1 → 0 extra and 7 → 9
        // build_diverged** on 2026-09-18, item 358 — the market's trade
        // (`docs/ECONOMY.md` §12), which moves this map's word 8985 →
        // 9134. Ten positions closer and the spurious unit gone, against
        // six of the roster going missing and two more building
        // field-rows out: an AI that can sell buys different things for
        // the 14,900 frames after 9134, and `great_lakes_scholars_sit_on_
        // their_universities` re-pins the one it is *about* — the four
        // seated scholars are still exact, and the one on the wrong
        // university moved `1/52` to `1/56` carrying the same vector.
        // DECISIONS 36 asks for the number rather than a trade.
        // And **6 → 10 unlinked and 9 → 10 build_diverged** on 2026-09-18,
        // item 360 — `Unit::move_step`'s *arrival* arm getting its own
        // collision block (`docs/COLLISION.md` §5.4), which moves this
        // map's word 9134 → 9182. `off` does not move at all; four more of
        // the roster go missing and one more building field-row is out,
        // 14,800 frames past the word. Every unit on this map that has ever
        // been blocked on the frame it snaps onto a waypoint now stands for
        // that frame instead of being resolved out of the way, so the
        // late-game deal shifts. DECISIONS 36 asks for the number rather
        // than a trade.
        // And **57 → 50 off and 10 → 9 unlinked** on 2026-09-18, item
        // 362 — the make list's **`num`** (`docs/AI.md` §42). This is
        // the map the item was taken on: the AI's University queues the
        // batch the original queues from 8985, run97's build-queue
        // widening falls from two residue rows to one, and seven
        // positions and one unlinkable unit come off the endpoint
        // 14,800 frames later. The long capture's `first_count` moves
        // 9182 → 9362 with it; `first_part` does not, so the headline
        // word is unchanged.
        // And **50 → 66 off and 9 → 5 unlinked** on 2026-09-18, item
        // 379 — the **attack animation's deferral** (`docs/ANIM.md`
        // §6.2). Every unit in the game that swings now asks for an
        // attack slot, and `Guy::inc_time` wraps that slot instead of
        // whatever it was playing, so the animation clock of every
        // fighting figure changes from the first engagement. **Neither
        // word moves** — this map's is 9182 and the first attack roll on
        // it is at 9416, 234 frames past the word — so every count here
        // is post-divergence reshuffle: sixteen positions out against
        // four of the roster that stop going missing, 14,800 frames past
        // the word. East Indies' endpoint and both ladder rungs are
        // unchanged on the same commit. The mechanism is the listing's
        // (`005da38a`, `005d9381`, `005da081`) and the golden record's
        // own 617 is where a diff can see it; DECISIONS 36 asks for the
        // number rather than a trade.
        // And **66 → 58 off and 5 → 7 unlinked** on 2026-09-18, item 384 —
        // the **HOPLITES melee reach** (`docs/COMBAT.md` §13.2, §18) and
        // `find_melee_target`'s two radius corrections (§12.4). The reach
        // is a constant the document has carried since the second reading
        // and the one call site passed `false` for, so until now every
        // melee unit in the simulation fought at `0x66`; `0xf6` reproduces
        // all six of the golden record's engagement-frame range verdicts.
        // **Neither long word moves** — Great Lakes is 9182 and its first
        // attack roll is at 9416 — so this is again reshuffle 14,800
        // frames past the word: eight positions closer and two more of the
        // roster missing, with `build_diverged` and the cities unchanged.
        // East Indies' endpoint and both ladder rungs are unchanged on the
        // same commit. DECISIONS 36: the number, not a trade.
        // Then **on 2026-09-18, item 385** — the **met bit**, set on first
        // contact off the fog (`docs/VISION.md` §6.2), which moves this
        // map's word 9182 → 9415. The AI has an enemy it knows about from
        // frame 7944, so `weight_total`'s war term pays from there and
        // every tech it has ranked since is re-valued.
        //
        // **The two items landed on the same day and this row is the
        // merged tree's**, not either branch's: 385 measured 66 → 49 and
        // 5 → 12 against 384's base, and 384 measured 66 → 58 and 5 → 7
        // against 385's. Two independent improvements compose, and the
        // figure below is what the merged code prints — the same rule the
        // 289/290 merge set on the row above, and a merge is not the place
        // to adjudicate a floor. The merged figure is **off 54, unlinked
        // 4, build_diverged 9** — lower than either branch on two of the
        // three columns and on neither branch's line for any of them.
        // DECISIONS 36 asks for the number rather than a trade.
        //
        // Then **54 → 43, 4 → 13** on 2026-09-18, item 389 — a unit's shot
        // launched by its attack animation's own release event rather than
        // by `Unit::fight` (`docs/COMBAT.md` §9.0), which moves this map's
        // word 9415 → 9451. It is the first change to this row that
        // reaches it through a **fight**: from 9425 the two archers' arrows
        // exist on the frames the original's exist on, land on the frames
        // they land on, and the building they are aimed at loses hit points
        // on the original's own schedule — 14,550 frames of a war that used
        // to start ten frames early on every shot. Eleven positions closer
        // and nine more of the roster unlinked, all thirteen of them
        // consecutive (`1/68`–`1/80`). `build_diverged` and the cities are
        // unchanged. DECISIONS 36 asks for the number rather than a trade.
        //
        // Then **43 → 58, 13 → 12, 9 → 10** on 2026-09-19, item 394 — the
        // **first wound's roll** (`docs/COMBAT.md` §20), one
        // `Random::get(0, 0xffff)` on the first combat damage to any
        // building, taken twice on sim-frame 9451 because a Longbowman
        // figure's thirteen sixteenths leave `damage` at zero. **This map's
        // word does not move on it** — it goes from five of the original's
        // seven draws at 9451 to six, and the seventh needs the launch
        // offset §20.3 bounds — so the whole of this row's movement is a
        // stream that shifts by one draw from 9451 and then runs 14,550
        // frames on nobody's numbers. Fifteen positions further out, one
        // fewer of the roster unlinked, one more building field-row apart.
        // **This is the direction DECISIONS 36 exists to record rather
        // than to trade away**: the draw is the original's, the listing
        // says so at `0x6520fc`, and the original's own 9451 spends it
        // twice. A row that moves the wrong way on a change the oracle
        // requires is the number, not an argument against the change.
        //
        // **Pinned at the 392/394 merge, and by neither branch's numbers.**
        // 394 measured 43 → 58 and 13 → 12 against its base, and item 392's
        // `Unit::find_melee_pos` (§19) leaves this row alone on its own
        // branch; the merged tree prints **off 57, unlinked 10,
        // build_diverged 10** — one position closer than 394 alone and two
        // fewer of the roster unlinked. Two independent landings compose,
        // so the figure is what the merged code prints, and a merge is not
        // the place to adjudicate a floor (the same rule the 289/290 and
        // 294/295 merges set above).
        //
        // Then **10 → 9 unlinked, `off` and `build_diverged` unmoved** on
        // 2026-09-19, item 395 — the captain mirror (`docs/COMBAT.md` §21),
        // which moves the *rules* headline's word 619 → 624 and neither
        // long capture's. It is the widest behavioural change this row has
        // taken: every squad member in the game stops searching for a
        // target and stops running the rest of `Unit::think` —
        // `think_peasant`, `think_caravan`, `think_fish`, the army joins —
        // because the original's `Unit::think` returns after the mirror.
        //
        // **And the two figures are a lesson in not reconciling by hand.**
        // On its own branch, whose base read `off 43, unlinked 13,
        // build_diverged 9`, item 395 measured **43 → 53, 13 → 14 and
        // 9 → 10** and reported the row as having got worse. On the merged
        // tree it is one of the roster *linked* and nothing else: 394's
        // first wound had already carried `off` and `build_diverged` to
        // 57/10 by itself, and the mirror composes with it rather than
        // adding to it. Neither branch's number is this row's, which is the
        // 289/290 and 294/295 rule again, and a worker's figure measured
        // against a base that has since moved is evidence about that base
        // and not about this tree.
        // Then **57 → 48, 9 → 11 and 10 → 9** on 2026-09-19, item 396 —
        // the measured launch offset (`docs/COMBAT.md` §22), which moves
        // this map's own word **9451 → 9510**. Nine positions closer and
        // one building field-row closer, two more of the roster unlinked
        // in exchange, 14,491 frames past the word it moves. DECISIONS 36
        // asks for the number rather than a trade, and the shape is worth
        // stating anyway: the change is a *table*, six rows measured off
        // one capture, so what it does at 24001 is fourteen thousand
        // frames of divergence downstream of six arrows and is evidence
        // about the run-up, not about the table.
        // And **48 → 51** on 2026-09-19, item 405 — the ATTACK action
        // under a move is a *ranged* attacker's (`docs/ORDERS.md` §4.4,
        // `ObjectTypeData +0x1fc max_range`), which moves the golden
        // record's word **621 → 626**. Three positions further out here,
        // 23,375 frames past the frame it is measured on and on the other
        // map: every melee unit in the game that has ever chased a target
        // into reach now walks the leg it was given instead of dropping
        // it, so this row is evidence about 24,000 frames of run-up and
        // not about the predicate. DECISIONS 36 asks for the number
        // rather than a trade; the value diff the change is booked on is
        // the golden record's own six units, exact through frame 624
        // (`testkit::GOLDEN_WORD_CHAPTER_ONE`).
        // And **51 → 55 off, 11 → 8 unlinked, 9 → 10 build_diverged** on
        // 2026-09-21, item 442 — the scholar arm's `val` chain
        // (`docs/AI.md` §53), which moves this map's own word **9510 →
        // 10161**, its largest single step since item 385. Three units
        // linked that did not and four stand further out, 13,840 frames
        // past the word. The AI now buys the scholars the original buys
        // and the endpoint has ten more of the game in it.
        // And **55 → 53 off** on 2026-09-21, item 447 — `is_seen` in
        // `valid_target`, and `Object::add_to_world`'s missing
        // `update_seen(0)` at a unit's birth (`docs/COMBAT.md` §31), which
        // moves the golden record's chapter two **616 → 624**. Two
        // positions closer and nothing else in the row moves, 13,840
        // frames past this map's word. Both halves touch every unit on
        // every map — a target out of sight is refused wherever it stands,
        // and a unit born on the map now lights its own disc — so this row
        // is evidence about 24,000 frames of run-up and not about the
        // predicate; the value diff the change is booked on is chapter
        // two's own widening at 622–624
        // (`chapter_two_s_word_frame_is_widened_whole`). DECISIONS 36 asks
        // for the number rather than a trade.
        // And **53 → 48 off and 8 → 13 unlinked** on 2026-09-21, item 456
        // — §4.3's group-mate soft arm declining for a collider that holds
        // a suspended search (`docs/COLLISION.md` §9), which moves this
        // map's own word **10161 → 10232**. Five positions closer and five
        // more of the roster without a counterpart, 13,769 frames past the
        // word. The clause fires wherever a squad walks into itself while
        // one of its members is re-planning, so like item 447's this row is
        // evidence about 24,000 frames of run-up and not about the
        // predicate; the value diff the change is booked on is the word's
        // own block, where item 448's fifteen rows go to **nought**
        // (`run100_s_word_block_is_every_record_the_dump_carries`).
        // DECISIONS 36 asks for the number rather than a trade.
        // And **48 → 51 off, 13 → 8 unlinked, 10 → 9 build_diverged** on
        // 2026-09-21, item 463 — `Unit::do_attack`'s validity test moving
        // behind `Unit::fight`'s reload gate, so a raider keeps the order
        // on a target that has just died for exactly as long as its
        // `recharging` runs (`docs/COMBAT.md` §7.12 is another lane's;
        // `docs/ORDERS.md` §7.3 carries it). It moves this map's word
        // **10232 → 10233**. Five roster slots that had no counterpart
        // now have one and three positions are further off, 13,768 frames
        // past the word — evidence about the run-up, not about the
        // predicate, which the word's own block is: six raiders' order
        // lists go to nought there
        // (`run100_s_word_block_is_every_record_the_dump_carries`).
        // DECISIONS 36 asks for the number rather than a trade.
        // And **13 → 9 unlinked and 10 → 9 build_diverged** on 2026-09-21,
        // item 462 — `find_nearby_target` walking the world cell's own
        // `down` chain instead of the unit index (`docs/COMBAT.md` §32.1)
        // and the unit half of `find_attack_pos` (§32.2), which move
        // chapter two's word **624 → 637**. `off` does not move at all:
        // four more of the roster have a counterpart at 24001 and one more
        // building field-row agrees, and not one position changed. Both
        // halves touch every chaser on every map, so this row is evidence
        // about 24,000 frames of run-up; the value diff the change is
        // booked on is chapter two's own `[620, 628)`, which goes from six
        // first-partings to **nought, on every record in both directions**
        // (`chapter_two_s_word_frame_is_widened_whole`, now at `[633,
        // 641)`). DECISIONS 36 asks for the number rather than a trade.
        // **Re-pinned at the 462/463 merge, 2026-09-21**, and by neither
        // worker's numbers. Both rows above were measured on a branch that
        // did not carry the other — 463's 51/8/9 without the cell chain
        // and the chase ring, 462's 48/9/9 without the reload gate — and
        // the merged tree agrees with neither. The figure is what the
        // merged code prints, the way the 289/290 and 294/295 merges
        // settled it: a merge is not the place to adjudicate a floor.
        // Against the common base of 48 off, 13 unlinked, 10
        // build_diverged, the two compose to **45 off, 9 unlinked, 10
        // build_diverged** — three positions closer and four more of the
        // roster linked, with the building field-rows back where the base
        // had them. Each worker over-claimed one count and under-claimed
        // another: 463 read `off` 51 where the merged tree reads 45, and
        // both read `build_diverged` 9 where it is 10. Measured on the
        // merge commit and on nothing else.
        // And **45 → 48 off, 9 → 10 unlinked** on 2026-09-21, item 464 —
        // `Unit::think`'s step 3 taking the military bit as well as the
        // attack column, and `Unit::target_opportunity`'s flee arm
        // (`docs/COMBAT.md` §34). Both go the wrong way, and both are
        // 13,768 frames past a word that did **not** move: past the word
        // the draw stream is unaligned, so every later random answer is a
        // coin flip and this row is evidence about the run-up rather than
        // about either predicate. The value diff the change is booked on
        // is the word's own block, where `0/5`'s twenty-two rows go to
        // **three** and its flight lands on the original's own
        // `(792, 31800)`
        // (`run100_s_word_block_is_every_record_the_dump_carries`).
        // DECISIONS 36 asks for the number rather than a trade.
        // Then **48 → 49 off, 10 → 9 unlinked, 10 → 8 build_diverged** on
        // 2026-09-21, item 465 — the pushed group's pool slot and the
        // `unit_masks & 4` exemption that replaces this crate's invented
        // army gate (`docs/ORDERS.md` §16), which moves this map's own
        // word **10233 → 10234**. Two counts closer and one out, 13,767
        // frames past the word and so on the far side of an unaligned
        // draw stream; the value diff the change is booked on is block
        // 10234, where `1/28`'s **fifteen** rows go to none and the
        // block is item 464's three alone. DECISIONS 36 asks for the
        // number rather than a trade.
        // And **49 → 44 off, 9 → 11 unlinked, 8 → 9 build_diverged** on
        // 2026-09-21, item 471 — the group location a `QUEUE_LAST` move
        // is laid out from (`docs/ORDERS.md` §17). Five positions closer,
        // two more of the roster unlinked and one more building field-row
        // out, 13,767 frames past a word that did **not** move — so the
        // same caveat the 464 row carries applies here and more so: past
        // the word the draw stream is unaligned and every later random
        // answer is a coin flip. The value diff the change is booked on is
        // the word's own block 10235, where `1/28`'s **sixty** rows go to
        // three and its whole 43-node route becomes the original's, node
        // for node
        // (`run100_s_word_block_is_every_record_the_dump_carries`), with
        // run97's order residue 81,534 → 28,222 and five of its eight
        // units gone beside it. DECISIONS 36 asks for the number rather
        // than a trade.
        // And **44 → 51 off, 11 → 7 unlinked** on 2026-09-21, item 478 —
        // `City::regen_roads`' second writer, `Build::remove_from_city`
        // (`docs/ROADS.md` §1.2), which moves this map's own word
        // **10234 → 10237** and its count word to 10244. Seven positions
        // out and four more of the roster linked, 13,764 frames past the
        // word and so on the far side of an unaligned draw stream. The
        // value diff the change is booked on is blocks 10231–10241,
        // where the replan flag's whole schedule — six buildings flagged
        // by a farm's death and each replanning on `12_240 - o` — is
        // this crate's now, and 10234 and 10235 spend the original's 204
        // and 78 draws node for node
        // (`run100_s_word_block_is_every_record_the_dump_carries`).
        // DECISIONS 36 asks for the number rather than a trade.
        // And **51 → 42 off, 7 unlinked unchanged** on 2026-09-22, item
        // 487 — `Unit::ungroup_move_order`'s re-head and `do_move`'s
        // `vector_dist < 0x481` gate on the dead-target re-path
        // (`docs/ORDERS.md` §20), which moves this map's own word
        // **10244 → 10277** on both the count and the sequence. Nine
        // positions closer, nothing else moved, 13,724 frames past the
        // word and so on the far side of an unaligned draw stream. The
        // value diff the change is booked on is block 10245, where item
        // 483's eleven rows of `1/27` go to **none** — `1/29` walks home
        // from 10242 the way the original's does, so nothing stands in
        // `1/27`'s way — and `1/29` leaves the window's residue set
        // entirely
        // (`run100_s_word_block_is_every_record_the_dump_carries`).
        // DECISIONS 36 asks for the number rather than a trade.
        // And **42 → 50 off, 7 → 8 unlinked, 9 → 8 build_diverged** on
        // 2026-09-22, item 489 — §4.3's group arm reading `army_of`
        // where the original reads `UnitData +0x80`, so a **pushed**
        // group's members were hard to each other
        // (`docs/COLLISION.md` §11), which moves this map's own word
        // **10277 → 10294** on both the count and the sequence. Eight
        // positions out, one more of the roster unlinked and one
        // building field-row closer, 13,707 frames past the word and so
        // on the far side of an unaligned draw stream — and the eight
        // are not scattered: `1/24`, `1/25` and `1/26` are 24 out on
        // both axes and the raiders `1/40`, `1/41`, `1/42` some 6,400
        // south, a squad that has walked a different route rather than a
        // roster that has come apart. The value diff the change is
        // booked on is block 10278, where item 487's **seventeen** rows
        // of `1/40` and `1/41` go to **none** — both take the
        // original's own step and both raise the soft one-shot
        // (`run100_s_word_block_is_every_record_the_dump_carries`).
        // DECISIONS 36 asks for the number rather than a trade.
        // And **50 → 49 off, 8 unlinked unchanged, 8 → 9
        // build_diverged** on 2026-09-22, item 494 — the human's idle
        // wait is `peasants_wait`'s *switch* and not its value, so it is
        // **12** where this crate used 2 (`docs/ORDERS.md` §21), which
        // moves this map's own word **10294 → 10303** on both the count
        // and the sequence. One position in and one building field-row
        // out, 13,698 frames past the word and so on the far side of an
        // unaligned draw stream. The value diff the change is booked on
        // is block **10295**, where item 489's six rows of `0/5` go to
        // **none**, with 10294's four under them and 10234's three
        // sixty blocks earlier — the `GATHERORDER` queued under the
        // citizen's flight, which is the cause the other two are
        // consequences of
        // (`run100_s_word_block_is_every_record_the_dump_carries`).
        // DECISIONS 36 asks for the number rather than a trade.
        // And **49 → 55 off, 8 → 5 unlinked and 9 → 10
        // build_diverged** on 2026-09-22, item 497 — the scholar teach
        // slot's tie-break (`docs/ANIM.md` §4.12), which is this map's
        // own headline: the word moves **10303 → 10582**, the largest
        // move of the chain. Three fewer of the roster unlinked against
        // six positions further out and one more building field-row,
        // 13,419 frames past the word, where the stream is nobody's.
        // DECISIONS 36 asks for the number rather than a trade; the
        // number is on the headline, and the headline is what moved.
        // And **55 → 53 off, 5 → 3 unlinked and 10 → 9
        // build_diverged** on 2026-09-22, item 506 —
        // `market_speculation`'s buy and sell passes
        // (`docs/ECONOMY.md` §13), which is this map's own headline:
        // the word moves **10582 → 10817**. Every count fell, which is
        // rare here and is what an AI that now spends its wealth the way
        // the original does looks like 13,184 frames past the word.
        // DECISIONS 36 asks for the number rather than a trade.
        // And **53 → 42 off, 3 → 5 unlinked and 9 → 10
        // build_diverged** on 2026-09-22, item 515 — the group speed
        // cap (`docs/GROUPS.md` §18), which is this map's own headline:
        // the word moves **10817 → 10834**. Eleven of the roster back on
        // the original's point 13,167 frames past the word against two
        // more unlinked and one more building field-row, and it is the
        // largest fall in `off` this counter has taken: a group whose
        // fast members now walk at the group's pace keeps its block
        // together for the rest of the game. DECISIONS 36 asks for the
        // number rather than a trade.
        // And **42 → 57 off, 5 → 4 unlinked** on item 518 —
        // `Groups::process@006fa210`'s one-slot-a-frame reset of every
        // group's cap (`docs/GROUPS.md` §19), which moves this map's word
        // **10834 → 11185**. Fifteen positions further out, 12,816 frames
        // past the new word: the reset changes the pace of every army
        // march on the map, and this row is evidence about the run-up,
        // not about the reset. DECISIONS 36 asks for the number rather
        // than a trade; the value diff the move is booked on is run100's
        // blocks 10242, 10243 and 10835, empty.
        // **57 → 50, 4 → 1 and 10 → 9** on item 327, the Merchant offer
        // (`docs/AI.md` §55), which moves this map's word 11185 → 11531: the
        // AI buys the original's Merchants, and seven positions, three units
        // and a building come back, 12,500 frames past the word.
        // **50 → 46 off, 1 → 0 unlinked, 0 → 3 extra and 9 → 10
        // build_diverged** on item 539, the collision fast path's stride
        // over an empty world cell (`docs/COLLISION.md` §4.2), which moves
        // this map's word 11531 → 11582. Every marching squad's soft flags
        // change with it, so the roster reshuffles 12,418 frames past the
        // word: four positions and a unit closer, three spurious units
        // (two Bowmen, a Citizen) and a building field-row further. The
        // value diff the move is booked on is run125's `[11250, 11599]`,
        // where no squad position parts. DECISIONS 36 asks for the number
        // rather than a trade.
        // **46 → 47 off, 3 → 2 extra, 10 → 9 build_diverged** on item 545,
        // `get_cost`'s military discount and research arm (`docs/AI.md`
        // §56), which move this map's word 11582 → 11757. The value diff
        // the move is booked on is run125's 11583, where nothing parts on
        // the leader or the queue any more; the endpoint is 12,244 frames
        // past the new word. The two extras are Merchants. DECISIONS 36.
        // **47 → 57 off** on item 557, the group back-pointer
        // (`docs/GROUPS.md` §23), which moves this map's word 11757 → 11806:
        // `1/64` leaves army 2's pool on 11512 as the original's does. The
        // 24,000th frame is 12,195 frames past the new word, so the roster
        // reshuffles; ten positions further out against one extra unit fewer.
        // DECISIONS 36: the number, not a trade.
        // **57 → 58 off** on item 560, the soft one-shot set only on a
        // sweep that ends soft (`docs/COLLISION.md` §12), which moves this
        // map's word 11806 → 11903. The 24,000th frame is 12,098 frames
        // past the new word. DECISIONS 36: the number, not a trade.
        // **58 → 42 off, 0 → 4 unlinked** on item 566, the pathfinder's
        // validity memo carried across searches (`docs/PATHFINDER.md`
        // §24), which moves this map's word 11903 → 12038. The 24,000th
        // frame is 11,963 frames past the new word; the four unlinked are
        // player 1's `77`..`80`. DECISIONS 36: the number, not a trade.
        // **42 → 41 off** on the eleventh Fable pass, with no change to
        // the simulation: 566 measured 42 on its own branch, the merged
        // tip reads 41, and the booking commit's gate said so in a task
        // whose verdict arrived after the commander had written its
        // handoff and stopped (parked 574). Re-pinned to what the tip
        // measures. DECISIONS 36: the number, not a trade.
        // **41 → 48 off, 4 → 0 unlinked, 0 → 1 extra, 9 → 8
        // build_diverged** on item 597: a mine's reach measured to the
        // nearest solid mountain cell (`docs/AI.md` §59). This map's word
        // holds at 12038, 11,963 frames before this one; its mines past the
        // word are sited by the new measure. The extra is a Merchant,
        // `1/81`. DECISIONS 36: the number, not a trade.
        // **48 → 45 off, 1 → 0 extra, 8 → 9 build_diverged** on item 608,
        // a city counting an unfinished gather building's slots
        // (`docs/AI.md` §61). This map's word holds at 12038, 11,963 frames
        // before this one; every AI city counts its gather sites from
        // placement now, on both maps. DECISIONS 36: the number, not a
        // trade.
        // **45 → 41 off, 0 → 1 extra** on item 629, a deployed Merchant
        // seated on its tile corner with its square blocked
        // (`docs/MERCHANT.md` §3.2). This map's word holds at 12038, 11,963
        // frames before this one; the extra is a Merchant, `1/81`.
        // DECISIONS 36: the number, not a trade.
        // **41 → 42 off, 1 → 0 extra, 9 → 10 build_diverged** on item 571,
        // a converted figure standing with its speeds zeroed
        // (`docs/ANIM.md` §11), which moves this map's word 12038 → 12135.
        // The 24,000th frame is 11,866 frames past the new word.
        // DECISIONS 36: the number, not a trade.
        // **0 → 2 extra** on item 657, `release_mustering` reading the
        // Military level (`docs/ARMY.md` §20), which moves this map's word
        // 12135 → 12184; `off` holds at 42 and `build_diverged` at 10. The
        // extras are the Merchants `1/81` and `1/82` again. DECISIONS 36:
        // the number, not a trade.
        // **0 → 1 unlinked, 2 → 0 extra, 10 → 7 build_diverged** on item
        // 661, the Civic level seating the Barracks and Stable in Norwich
        // (`docs/AI.md` §63), which moves this map's word 12184 → 12429;
        // `off` holds at 42. The 24,000th frame is 11,571 frames past the
        // new word. DECISIONS 36: the number, not a trade.
        // **42 → 53 off, 1 → 0 unlinked, 0 → 1 extra, 7 → 6
        // build_diverged** on item 669, `do_move`'s tile arm keeping a
        // tolerance-0 formation waypoint (`docs/ORDERS.md` §4.4,
        // `docs/AI.md` §64), which moves this map's word 12429 → 12536. The
        // extra is the Merchant `1/81`. The 24,000th frame is 11,464 frames
        // past the new word. DECISIONS 36: the number, not a trade.
        // **53 → 34 off, 6 → 8 build_diverged** on item 673: a failed
        // unit-grid search spares an action-bit move too
        // (`docs/PATHFINDER.md` §21.6, `docs/AI.md` §65), which moves this
        // map's word 12536 → 12897. The extra is still `1/81`. The 24,000th
        // frame is 11,103 frames past the new word. DECISIONS 36: the
        // number, not a trade.
        // **34 → 33 off, 8 → 4 build_diverged** on item 678: a resumed
        // 48-grid search reads the blocks it copied (`docs/PATHFINDER.md`
        // §26, `docs/AI.md` §66), which moves this map's word 12897 →
        // 14382. The extra is still `1/81`. The 24,000th frame is 9,618
        // frames past the new word. DECISIONS 36: the number, not a trade.
        // **33 → 46 off, 1 → 0 extra, 4 → 2 build_diverged** on item 688:
        // a founded city quarters and halves the site values round it
        // (`City::fix_world_vals`, `docs/AI.md` §67), which moves this
        // map's word 14382 → 14529. The 24,000th frame is 9,471 frames
        // past the new word. DECISIONS 36: the number, not a trade.
        // **46 → 30 off** on item 695: the stray-road sweep and the
        // caravan's road check (`docs/ROADS.md` §10, `docs/CARAVAN.md`
        // §10), which move this map's word 14529 → 14650. The 24,000th
        // frame is 9,350 frames past the new word. DECISIONS 36: the
        // number, not a trade.
        // **30 → 46 off, 0 → 1 unlinked** on item 698: a gatherer on a
        // suspended search gives its walk up near its point
        // (`docs/COLLISION.md` §15), which moves this map's word 14650 →
        // 14982. Measured after item 696's merge. The unlinked unit is
        // `1/80`. The 24,000th frame is 9,018 frames past the new word.
        // DECISIONS 36: the number, not a trade.
        // **46 → 47 off, 1 → 0 unlinked, 0 → 1 extra** on item 706: every
        // nation's graft table and the Senate's government patriot
        // (`docs/TECH.md` §"The graft table", §"The government patriot"),
        // which move this map's word 14982 → 15175. Measured after item
        // 703's merge. The extra unit is `1/81`, a Merchant. The 24,000th
        // frame is 8,825 frames past the new word. DECISIONS 36: the
        // number, not a trade.
        // **47 → 27 off, 1 → 2 extra, 2 → 0 build_diverged** on item 711:
        // a unit's own mirror, `unit_masks & 2` (`docs/GROUPS.md` §25),
        // which moves this map's word 15175 → 15383. Measured on the tree
        // before item 713's merge and again after it, with the same counts.
        // The extras are `1/81` and `1/82`, Merchants. The 24,000th frame
        // is 8,617 frames past the new word. DECISIONS 36: the number, not
        // a trade.
        // **27 → 32 off, 2 → 0 extra** on item 715: `Wall::process`'s site
        // recruiter (`docs/AI.md` §69), which moves this map's word 15383 →
        // 15384. Measured after item 714's merge. The 24,000th frame is
        // 8,616 frames past the new word. DECISIONS 36: the number, not a
        // trade.
        // **32 → 45 off, 0 → 4 extra** on item 722: `CityData::num_wonders`
        // counts a wonder site (`docs/AI.md` §70), which moves this map's
        // word 15384 → 15608. The extras are `1/81`..`1/83`, Citizens, and
        // `1/84`, a Merchant. The 24,000th frame is 8,392 frames past the
        // new word. Measured on the spawn base, `2355946`. DECISIONS 36:
        // the number, not a trade.
        // **45 → 22 off, 4 → 0 extra** on item 729: `Object::take_damage`
        // stamps the struck owner's `frame_attacked` (`docs/AI.md` §71),
        // which moves this map's word 15608 → 15619. The 24,000th frame is
        // 8,381 frames past the new word. Measured on the spawn base,
        // `7ec485a`. DECISIONS 36: the number, not a trade.
        // **22 → 44 off** on item 736: `action_siege_attack_to`'s sub-group
        // lays out on its own cleared record (`docs/GROUPS.md` §26), which
        // moves this map's word 15619 → 16460. The 24,000th frame is 7,540
        // frames past the new word. Measured after `ccc update` onto
        // `8e75c3b`. DECISIONS 36: the number, not a trade.
        off: 44,
        // **0 → 1** on item 661, beside `off` above.
        // **1 → 0** on item 669, beside `off` above.
        // **0 → 1** on item 698, beside `off` above.
        // **1 → 0** on item 706, beside `off` above.
        unlinked: 0,
        // **2 → 1 extra** on item 557, beside 47 → 57 off above.
        // **1 → 2 extra** on item 560, beside 57 → 58 off above: two
        // Merchants, `1/81` and `1/82`.
        // **2 → 0 extra** on item 566, beside 58 → 42 off above.
        // **0 → 1 extra** on item 597, beside 41 → 48 off above.
        // **1 → 0 extra** on item 608, beside 48 → 45 off above.
        // **1 → 0 extra** on item 571, beside 41 → 42 off above.
        // **0 → 2 extra** on item 657; `off` held at 42.
        // **2 → 0 extra** on item 661, beside `off` above.
        // **0 → 1 extra** on item 669, beside `off` above: `1/81`.
        // **1 → 0 extra** on item 688, beside `off` above.
        // **0 → 1 extra** on item 706, beside `off` above: `1/81`.
        // **1 → 2** on item 711, beside `off` above.
        // **2 → 0** on item 715, beside `off` above.
        // **0 → 4** on item 722, beside `off` above.
        // **4 → 0** on item 729, beside `off` above.
        extra: 0,
        build_unlinked: 0,
        // **9 → 8** on item 597, beside `off` above.
        // **8 → 9** on item 608, beside `off` above.
        // **9 → 10** on item 571, beside `off` above.
        // **10 → 7** on item 661, beside `off` above.
        // **7 → 6** on item 669, beside `off` above.
        // **6 → 8** on item 673, beside `off` above.
        // **8 → 4** on item 678, beside `off` above.
        // **4 → 2** on item 688, beside `off` above.
        // **2 → 0** on item 711, beside `off` above.
        build_diverged: 0,
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
        // And **46 → 43, 19 → 22 and 7 → 6** on 2026-09-18, item 338
        // — the **scholar's seating** (`docs/CITIES.md` §6.5.2).
        // `Unit::go_inside`'s tail snaps an `is_scholar` unit onto its
        // host, and `Build::train` keeps a scholar trained at a
        // university inside it; every scholar
        // in the game therefore stops standing on the exit ring. That is
        // fourteen units on Great Lakes and the same mechanism on East
        // Indies, 15,500 frames past both words — which the change moves
        // together, 8272 → 8374 and 8466 → 8495. DECISIONS 36 asks for the
        // number rather than a trade.
        // And **43 → 42 and 22 → 21** on 2026-09-18, item 340 — the **scholar's teach
        // slot** (`docs/ANIM.md` §4.11). `Guy::set_anim`'s `guy_flags &
        // 0x80` arm turns the idle roll's variant into an offset into
        // slots 25–32 for a scholar inside its host, and
        // `ObjectData::is_peasant` is `TypeIndex` `0x32`/`0x33` rather
        // than any worker, so both the variant and the slot the seated
        // scholar plays change. Every scholar in the game runs a
        // different animation from the frame it sits down, 15,500 frames
        // before this block. Great Lakes' word moves 8374 → 8382 on it
        // and East Indies' does not move. DECISIONS 36 asks for the
        // number rather than a trade.
        // And **41 → 42 off** on 2026-09-18, item 348 — the market's
        // Coinage gate (`docs/AI.md` §40). One position further out on
        // this rung, 5,690 frames past East Indies' own word, which does
        // not move. DECISIONS 36 asks for the number rather than a trade.
        // And **42 → 43 off** on 2026-09-18, item 352 — the muster ring's
        // two corrected predicates (`docs/ARMY.md` §13), which are neither
        // map- nor capture-specific. One position further out on this rung,
        // 5,690 frames past East Indies' own word, which does not move.
        // DECISIONS 36 asks for the number rather than a trade.
        // Then **43 → 41 off and 17 → 13 extra** on 2026-09-18, item 358
        // — the market's trade (`docs/ECONOMY.md` §12). Two positions
        // closer and four spurious units gone, 5,690 frames past this
        // rung's word; `unlinked` and both building counts are unmoved.
        // Then **41 → 42 off and 17 → 13 extra** on 2026-09-21, item 478
        // — `City::regen_roads`' second writer (`docs/ROADS.md` §1.2).
        // One position out and four spurious units gone, 5,690 frames
        // past this rung's word, which does not move; `unlinked` and
        // both building counts are unmoved.
        // **42 → 40 off** on item 557, the group back-pointer
        // (`docs/GROUPS.md` §23): army lists follow `Group::add`'s per-step
        // `get_num` and `Group::sort` everywhere. 5,690 frames past this map's
        // word, which holds at 9711. DECISIONS 36.
        // **40 → 43 off** on item 573, the caravan's one-off at the
        // Commerce level (`docs/CARAVAN.md` §9): this map's word moves
        // 9711 → 9983 and this rung reads 5,418 frames past it, three
        // positions out. DECISIONS 36.
        // **43 → 40 off** on item 576, the sea branch's dock (`docs/AI.md`
        // §57): the AI offers ships at a Dock of no city from 9981, and
        // this map's word moves 9983 → 10232. This rung reads 5,169 frames
        // past it. DECISIONS 36.
        // **40 → 38** on item 579 (`docs/ORDERS.md` §25).
        // **38 → 41** on item 588; see `extra` below.
        // **41 → 40** on item 592; see `extra` below.
        // **40 → 39** on item 597; see `extra` below.
        // **39 → 35** on item 613; see `extra` below.
        // **35 → 31** on item 620; see `extra` below.
        // **31 → 29** on item 629; see `extra` below.
        // **29 → 28** on item 642; see `extra` below.
        // **28 → 29** on item 643: an animation's name is found
        // case-folded (`docs/ANIM.md` §12); this map's word moves 13640 →
        // 15782, 381 frames past this rung. DECISIONS 36: the number, not
        // a trade.
        off: 29,
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
        // Then **19 → 23 and 7 → 8** on 2026-09-17, item 323's scholar
        // gate: four spurious units back on this rung and one more
        // building field-row apart, 7,872 frames past its word, on the
        // change that takes Great Lakes' floor 8182 → 8186.
        // **23 → 19 and 8 → 7** on 2026-09-17, item 334's `is_active`
        // gate (`docs/ROADS.md` §1.1): four spurious units and one
        // building field-row gone, 7,872 frames past this rung's word,
        // with `off` and `unlinked` unmoved.
        // Then **21 → 25 extra and 6 → 8 build_diverged** on
        // 2026-09-18, item 344 — the mine's mountain
        // range (`docs/ECONOMY.md`, "The mine's range"). The first mine
        // on each map now claims its range's tiles as gathered from,
        // which re-deals every gatherer's assignment from the frame it is
        // placed on. Great Lakes' word moves 8382 → 8404 on it; East
        // Indies' does not move. DECISIONS 36 asks for the number rather
        // than a trade.
        // And **42 → 41 off and 25 → 21 extra** on 2026-09-18, item 346 —
        // the scholar's idle category (`docs/ANIM.md` §5.1), which takes
        // East Indies' own word 8495 → 9711. One position closer and four
        // spurious units gone on this rung, 5,690 frames past the new
        // word, with both building counts unmoved. DECISIONS 36 asks for
        // the number rather than a trade.
        // Then **21 → 16 extra** on 2026-09-18, item 348 — the same
        // Coinage gate. Five spurious units gone on this rung, with both
        // building counts and `unlinked` unmoved. DECISIONS 36 asks for
        // the number rather than a trade.
        // Then **16 → 17 extra** on 2026-09-18, item 354 — the `army`
        // mode again (`docs/PATHFINDER.md` §22). One spurious unit on this
        // rung and nothing else on it moves; rung B does not move at all.
        // DECISIONS 36 asks for the number rather than a trade.
        // Then **17 → 13 extra** on 2026-09-18, item 358 — the market's
        // trade (`docs/ECONOMY.md` §12), which also takes two off this
        // rung's `off` above. Four spurious units gone, with `unlinked`
        // and both building counts unmoved.
        // And **13 → 25 extra** on 2026-09-18, item 362 — the make
        // list's `num` again (`docs/AI.md` §42). Twelve more units this
        // crate holds that run28's dump does not, 7,590 frames past East
        // Indies' own word, which does not move: a batch size the AI had
        // been capped at one buys more of everything, and this rung is
        // the furthest-out of the four counters it touches. DECISIONS 36
        // asks for the number rather than a trade.
        //
        // **Typed 2026-09-19, item 370, and the twelve are not an
        // overshoot.** The question 362 left was what they are: all
        // scholars and caravans would say the batch is too large, a mixed
        // bag ordinary divergence. Measured against the pre-fix tree, the
        // rosters are not nested — they are *substituted*. Scholars are 2
        // before and 2 after and caravans 0 in both, so neither type the
        // batch grows moved at all; the one in-scope type that does is the
        // citizen, 2 → 8, whose `num` is `min(deficit, room)` and bounded.
        // Of the military, Bowmen 3, Longbowmen 3 and Horse Archer 1
        // vanish outright and Hoplites 6, Light Horse 3 and Slingers 3
        // appear. Only 1/10, 1/12, 1/25 and 1/29 carry the same type in
        // both trees. A too-large batch adds units of the batched types and
        // leaves the rest alone; this is a different late production run,
        // which is what 7,590 frames past the word buys. `extra` is still 0
        // on both endpoint captures, which is the counter an overshoot
        // would move.
        // And **25 → 17 extra** on 2026-09-21, item 442 — the scholar
        // arm's `val` chain (`docs/AI.md` §53). Eight spurious units gone
        // from this rung and nothing else moved: the AI stops buying
        // Citizens where the original buys a Scholar, on both maps.
        // **13 → 14 on item 506**, `market_speculation`'s two passes
        // (`docs/ECONOMY.md` §13): one more Cataphract this crate
        // trains and the original does not, 5,690 frames past this
        // map's word. DECISIONS 36 asks for the number, not a trade.
        // **14 → 13 on item 515**, the group speed cap
        // (`docs/GROUPS.md` §18): one fewer Cataphract, the same unit
        // 506 added, 5,690 frames past this map's word.
        // **13 → 15 on item 518**, `Groups::process`'s per-frame pool
        // reset (`docs/GROUPS.md` §19), 5,690 frames past this map's word:
        // two more units this crate holds and the original does not. The
        // number, not a trade.
        // **15 → 21 on item 545** (`docs/AI.md` §56): this crate researches
        // unit upgrades now and prices military units at the original's
        // discount, and six more units stand here that the original's
        // does not hold, three of them the Slingers rung B sees upgraded.
        // 5,690 frames past this map's word. The number, not a trade.
        // **21 → 20 extra** on item 557 (`docs/GROUPS.md` §23), beside
        // 42 → 40 off above. DECISIONS 36.
        // **20 → 23 extra** on item 573, beside 40 → 43 off above.
        // **23 → 7 extra, 8 → 4 build_diverged** on item 576, beside
        // 43 → 40 off above: sixteen units this crate held and the
        // original did not are gone.
        // **7 → 19 extra** on item 579, beside 40 → 38 off above: the navy
        // and the dock's margin, 5,003 frames past this map's word.
        // **19 → 22 extra, 38 → 41 off, 4 → 5 build_diverged** on item 588,
        // the sea half of `detect_unit_collision`'s second arm
        // (`docs/COLLISION.md` §13), 4,819 frames past this map's new word.
        // **22 → 23 extra, 41 → 40 off** on item 592, the census counting
        // a barge's rider through its container (`docs/AI.md` §58).
        // **23 → 13 extra, 40 → 39 off, 5 → 4 build_diverged** on item 597,
        // a mine's reach measured to the nearest solid mountain cell
        // (`docs/AI.md` §59), 4,819 frames past this map's word.
        // **13 → 7 extra** on item 604, the placed mountain templates'
        // solid cells (`docs/AI.md` §60), 4,619 frames past this map's new
        // word 10782: three Citizens, two Scholars, a Cataphract and a
        // Transport Barge.
        // **7 → 13 extra** on item 608, a city counting an unfinished
        // gather building's slots (`docs/AI.md` §61), 4,419 frames past
        // this map's new word 10982: five Citizens, two Scholars, two
        // Horse Archers, two Cataphracts and two Light Horse.
        // **13 → 25 extra, 39 → 35 off, 4 → 2 build_diverged** on item 613,
        // Horses' discount on a Stable unit and a mined range taken on the
        // survey (`docs/AI.md` §62), 4,332 frames past this map's new word
        // 11069: three Citizens, two Scholars, two Cataphracts, two Horse
        // Archers, a Light Horse and fifteen foot (Bowmen, Slingers,
        // Hoplites, Longbowmen). DECISIONS 36: the number, not a trade.
        // **25 → 26 extra, 35 → 31 off** on item 620, a Mine's `dist_mod`
        // capped at 3 on a list under `MTN_TINY_SIZE` (`docs/ORDERS.md`
        // §6.4), 3,811 frames past this map's new word 11590. DECISIONS 36:
        // the number, not a trade.
        // **26 → 13** on item 629, a deployed Merchant seated on its tile
        // corner with its square blocked (`docs/MERCHANT.md` §3.2); this
        // map's word moves 11590 → 11747, 3,654 frames before this rung.
        // **13 → 11** on item 642: `Region::go_here` reads the human's
        // city count (`docs/TRANSPORT.md` §9.4); this map's word moves
        // 11747 → 13640, 1,761 frames before this rung.
        extra: 11,
        build_unlinked: 10,
        // **4 → 5** on item 588, beside `extra` above.
        // **5 → 4** on item 597, beside `extra` above.
        // **4 → 2** on item 613, beside `extra` above.
        build_diverged: 2,
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
        // Then **50 → 51** on 2026-09-17 with the `extra` below, on item
        // 323's scholar gate — one position out, 8,960 frames past this
        // rung's word. It surfaced only after the C rung above was
        // re-pinned, the same way it did on item 304 this morning: the
        // ladder test panics at its first moved row, so a change touching
        // both rungs is reported one rung at a time.
        // **51 → 53 and 16 → 15** on 2026-09-17, item 334's `is_active`
        // gate (`docs/ROADS.md` §1.1): two positions out and one spurious
        // unit gone, 8,960 frames past this rung's word. The C rung moved
        // the other way on the same change.
        // And **53 → 48 and 15 → 18** on 2026-09-18, item 338 — the
        // **scholar's seating** (`docs/CITIES.md` §6.5.2).
        // `Unit::go_inside`'s tail snaps an `is_scholar` unit onto its
        // host, and `Build::train` keeps a scholar trained at a
        // university inside it; every scholar in the game therefore stops
        // standing on the exit ring. Five positions closer and three more
        // spurious units, 8,000 frames past this rung's word. DECISIONS 36
        // asks for the number rather than a trade.
        // And **48 → 49 and 18 → 14** on 2026-09-18, item 340 — the
        // **scholar's teach slot** (`docs/ANIM.md` §4.11), whose rows on
        // the other three are above. This rung's only visible once the
        // C rung stops failing first.
        // Then **49 → 51** on 2026-09-18 with the `extra` below, item 344 —
        // the mine's mountain range, whose note is on the C rung above.
        // And **51 → 48 off and 20 → 17 extra** on 2026-09-18, item 346 —
        // the scholar's idle category (`docs/ANIM.md` §5.1). Three
        // positions closer and three spurious units gone on this rung,
        // 6,778 frames past a word the same change takes 8495 → 9711,
        // with `unlinked` and both building counts unmoved. DECISIONS 36
        // asks for the number rather than a trade.
        // **48 → 49 off** on 2026-09-18, item 348 — the market's Coinage
        // gate (`docs/AI.md` §40). One position further out on this rung,
        // 8,677 frames past East Indies' own word, which does not move.
        // DECISIONS 36 asks for the number rather than a trade.
        // And **49 → 48 off** on 2026-09-18, item 352 — the muster ring's
        // two corrected predicates (`docs/ARMY.md` §13). All four rows move
        // on this one, because a building's cell flag and an army's spacing
        // are neither map- nor capture-specific. One position closer,
        // 6,778 frames past East Indies' own word, which does not move.
        // DECISIONS 36 asks for the number rather than a trade.
        // Then **48 → 49 off** on 2026-09-18, item 358 — the market's
        // trade (`docs/ECONOMY.md` §12). One position out on this rung
        // where the C rung above takes two off, 6,778 frames past this
        // rung's word.
        // And **49 → 47 off and 6 → 19 extra** on 2026-09-18, item 362
        // — the make list's `num` (`docs/AI.md` §42), the same change as
        // the rung above and the same shape: two positions closer and
        // thirteen more units this crate holds that run24's dump does
        // not, 8,700 frames past East Indies' own word, which does not
        // move. DECISIONS 36 asks for the number rather than a trade.
        // And **47 → 49 off, 19 → 12 extra** on 2026-09-21, item 442 —
        // the scholar arm's `val` chain (`docs/AI.md` §53). Seven spurious
        // units gone, two positions out, 8,000 frames past this rung's
        // word.
        // And **49 → 48 off, 12 → 11 extra** on 2026-09-21, item 478 —
        // `City::regen_roads`' second writer (`docs/ROADS.md` §1.2), the
        // same row as rung C's above. One position closer and one
        // spurious unit gone, 8,000 frames past this rung's word, which
        // does not move.
        // And **48 → 45 off, 11 → 6 extra** on 2026-09-22, item 497 —
        // the scholar teach slot's tie-break (`docs/ANIM.md` §4.12).
        // The rung sheds five spurious units and three positions come
        // in; East Indies' own word does not move.
        // **45 → 48 on item 506**, `market_speculation`'s two passes
        // (`docs/ECONOMY.md` §13), 6,778 frames past this map's word.
        // And **48 → 49 off, 14 → 9 extra** on 2026-09-22, item 515 —
        // the group speed cap (`docs/GROUPS.md` §18). Five spurious
        // units gone from this rung against one position out, 6,778
        // frames past this rung's word; the C rung above sheds one.
        // DECISIONS 36 asks for the number rather than a trade.
        // **49 → 47 on item 518**, `Groups::process` (see `extra` below).
        // **47 → 49** on item 327, the Merchant offer (`docs/AI.md` §55):
        // the AI's make list changes from its first Merchant, and this rung
        // moves two positions out and one spurious unit off. DECISIONS 36.
        // **49 → 48 off, 14 → 17 extra** on item 545 (`docs/AI.md` §56):
        // one position closer and three more units, the Javelineers this
        // crate's first unit upgrade makes of rung C's Slingers. DECISIONS 36.
        // **48 → 46** on item 579 (`docs/ORDERS.md` §25).
        // **46 → 51** on item 588; see `extra` below.
        // **51 → 49** on item 592; see `extra` below.
        // **49 → 46** on item 597; see `extra` below.
        // **46 → 47** on item 608; see `extra` below.
        // **47 → 44** on item 613; see `extra` below.
        // **44 → 39** on item 620, 4,899 frames past this map's new word
        // 11590 (`docs/ORDERS.md` §6.4). DECISIONS 36: the number, not a
        // trade.
        // **39 → 36** on item 642; see `extra` below.
        // **36 → 37** on item 688 (`docs/AI.md` §67), 707 frames past this
        // map's word 15782. DECISIONS 36: the number, not a trade.
        // **37 → 36** on item 706 (`docs/TECH.md` §"The government
        // patriot"); see `extra` below.
        // **36 → 37** on item 708 (`docs/AI.md` §72); see `extra` below.
        off: 37,
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
        // Then **12 → 16** on 2026-09-17, item 323's scholar gate: four
        // spurious units back, the same count the C rung above gained, on
        // the change that takes Great Lakes' floor 8182 → 8186. The AI
        // trains what it was skipping, so both rungs gain extras.
        // Then **14 → 20** on 2026-09-18, item 344 — six spurious units
        // back on this rung, on the change that takes Great Lakes' word
        // 8382 → 8404.
        // **17 → 16 extra** on 2026-09-18, item 348, the same gate: one
        // spurious unit gone, with both building counts unmoved.
        // Then **16 → 15 extra** on 2026-09-18, item 354 — the `army`
        // mode again (`docs/PATHFINDER.md` §22). One spurious unit gone
        // on this rung where the C rung above gains one, with `off`,
        // `unlinked` and both building counts unmoved on both.
        // Then **15 → 6** on 2026-09-18, item 358's market trade — nine
        // spurious units gone on this rung, the largest single fall any
        // rung has taken, with `unlinked` and both building counts
        // unmoved. ~~**Rung B is only reached once rung C passes**: the
        // ladder test panics at its first moved row, so this row surfaced
        // on the second run.~~ **No longer true from item 370**: both
        // rungs are walked and typed before either is asserted, because
        // the two are different games and a rung C failure was taking rung
        // B's measurement with it.
        //
        // **Typed 2026-09-19, item 370.** 6 → 19 under 362 and the same
        // substitution as rung C: pre-fix Bowmen 1, Cataphract 1, Citizen
        // 1, Longbowmen 3; post-fix Cataphract 1, Citizen 6, Hoplites 6,
        // Light Horse 3, Slingers 3, with 1/12 the only object number that
        // keeps its type. **Rung B is rung C's own simulation 1,088 frames
        // later**, and the block 1/56–1/72 is type-identical on the two
        // rungs — which the ladder test now asserts, since it is the one
        // thing about `extra` that does not churn with every AI landing.
        // **6 → 14 on item 506**, and the eight are the Longbowmen,
        // Hoplites and Citizens an AI with a live market can pay for
        // and this rung's original does not train.
        // And **9 → 15 extra, 49 → 47 off** on item 518 — `Groups::process`'s
        // per-frame reset of one pool slot (`docs/GROUPS.md` §19), which
        // moves Great Lakes' word 10834 → 11185. Every army's cap now goes
        // back to its leader's own speed once every 64 frames, so every
        // map's marches change pace; 6,778 frames past this rung's word,
        // DECISIONS 36 asks for the number rather than a trade.
        // **15 → 14** on item 327, the Merchant offer (`docs/AI.md` §55).
        // **14 → 17** on item 545; see `off` above.
        // **17 → 16 extra** on item 557, the group back-pointer
        // (`docs/GROUPS.md` §23): every army's list now follows `Group::add`'s
        // per-step `get_num` and `Group::sort`, on every map. 3,440 frames
        // past this map's word. DECISIONS 36.
        // **16 → 21 extra** on item 573, the caravan's one-off at the
        // Commerce level (`docs/CARAVAN.md` §9): this map's word moves
        // 9711 → 9983, 6,506 frames under this rung. DECISIONS 36.
        // **21 → 1 extra** on item 576, the sea branch's dock
        // (`docs/AI.md` §57): this map's word moves 9983 → 10232, 6,257
        // frames under this rung, and twenty units this crate held and the
        // original did not are gone; `off` and every other field hold.
        // DECISIONS 36.
        // **1 → 12 extra** on item 579, beside 48 → 46 off above.
        // **12 → 18 extra, 46 → 51 off** on item 588, the sea half of `detect_unit_collision`'s second arm
        //  (`docs/COLLISION.md` §13), 13,419 frames past this map's new word. DECISIONS 36: the number, not a trade.
        // **18 → 16 extra, 51 → 49 off** on item 592, the census counting
        // a barge's rider through its container (`docs/AI.md` §58).
        // **16 → 22 extra, 49 → 46 off** on item 597, a mine's reach
        // measured to the nearest solid mountain cell (`docs/AI.md` §59).
        // **22 → 13 extra** on item 604, the placed mountain templates'
        // solid cells (`docs/AI.md` §60); `off` and every other field hold.
        // It was hidden behind rung C's panic until C's re-pin.
        // **13 → 9 extra, 46 → 47 off** on item 608, a city counting an
        // unfinished gather building's slots (`docs/AI.md` §61), 5,507
        // frames past this map's new word 10982. It too was hidden behind
        // rung C's panic until C's re-pin. DECISIONS 36: the number, not a
        // trade.
        // **9 → 20 extra, 47 → 44 off** on item 613, Horses' discount on a
        // Stable unit and a mined range taken on the survey (`docs/AI.md`
        // §62), 5,420 frames past this map's new word 11069. Hidden behind
        // rung C's panic again until C's re-pin. DECISIONS 36: the number,
        // not a trade.
        // **20 → 17** on item 629, a deployed Merchant seated on its tile
        // corner with its square blocked (`docs/MERCHANT.md` §3.2); this
        // map's word moves 11590 → 11747, 4,742 frames before this rung.
        // **17 → 13** on item 642: `Region::go_here` reads the human's
        // city count (`docs/TRANSPORT.md` §9.4); this map's word moves
        // 11747 → 13640, 2,849 frames before this rung.
        // **13 → 4** on item 643: an animation's name is found case-folded
        // (`docs/ANIM.md` §12); this map's word moves 13640 → 15782, 707
        // frames before this rung.
        // **4 → 5** on item 706: every nation's graft table and the
        // Senate's government patriot (`docs/TECH.md`); this map's word
        // moves 15782 → 15985, 504 frames before this rung. The fifth is
        // The Senator `1/60`, which this crate now trains and which this
        // rung's dump does not hold.
        // **5 → 7** on item 708: the commerce cap's republic term
        // (`docs/AI.md` §72); this map's word moves 15985 → 16683, 194
        // frames past this rung. The seven are player 1's Citizens `12`,
        // `59` and `61`, the Light Horse `56`, the Cataphracts `57` and
        // `58`, and The Senator `60`. DECISIONS 36: the number, not a
        // trade.
        extra: 7,
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
    /// **What the extras are**, `(who, o, type name)` in [`Self::extra`]'s
    /// order — the one question about them no dump can answer, because an
    /// `extra` is by definition a unit this crate holds and the capture's
    /// does not (item 370, `docs/AI.md` §42). [`walk_to_close`] leaves it
    /// empty; the caller fills it, because the type table is the loader's
    /// and not the simulation's.
    pub extra_types: Vec<(i64, i64, String)>,
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
        extra_types: Vec::new(),
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
        let mut r = walk_to_close(&mut built, &fin, 8);
        // **The extras, typed** (item 370). `extra` is what this crate holds
        // and the capture's dump does not, so nothing on disk says what they
        // are; the simulation that produced them is still standing here.
        r.extra_types = r
            .extra
            .iter()
            .map(|&(who, o)| {
                let name = built
                    .sim
                    .units
                    .iter()
                    .find(|u| u.alive() && i64::from(u.owner) == who && i64::from(u.index) == o)
                    .and_then(|u| u.ty)
                    .and_then(|ty| loaded.unit_names.get(ty).cloned())
                    .unwrap_or_else(|| "?".into());
                (who, o, name)
            })
            .collect();
        {
            let mut by_type: std::collections::BTreeMap<&str, usize> =
                std::collections::BTreeMap::new();
            for (_, _, name) in &r.extra_types {
                *by_type.entry(name.as_str()).or_default() += 1;
            }
            eprintln!("{} {} extra by type: {by_type:?}", row.map, r.extra.len());
            for (who, o, name) in &r.extra_types {
                eprintln!("    extra {who}/{o} {name}");
            }
        }
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
            // **Typed, not bare** (item 370): the count alone cannot tell an
            // overshoot from a substitution, and twice now the roster has
            // changed under a stable count.
            r.extra_types,
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

    /// **The scholars sit on their universities, and run80 says so** —
    /// item 338's value diff (`docs/CITIES.md` §6.5.2).
    ///
    /// Great Lakes' block 24001 has fourteen of player 1's units standing
    /// on exactly two points, `(40416, 25248)` and `(41184, 15264)`, which
    /// are buildings `1/2019`'s and `1/2020`'s own positions — seven
    /// apiece. That shape is `Unit::go_inside`'s scholar arm and nothing
    /// else, 15,700 frames past the word.
    ///
    /// **This is a value check and not a count**, which is the point: the
    /// row above it moves with every unrelated landing, and `off 62` says
    /// nothing about *which* sixty-two. Before item 338 every scholar in
    /// the game stood `(24, 552)` off — the exit ring — and not one of the
    /// eleven the endpoint compares agreed.
    ///
    /// **The roster is a pin, re-read on 2026-09-18 (item 344, then 346).**
    /// 338 left six exactly right and four off by exactly `±(768, 9984)`,
    /// which is `1/2020 − 1/2019`. 344's mine claims 207 tiles as gathered
    /// from on frame 8382 and re-deals every gatherer's assignment 15,700
    /// frames before this one, leaving **four** exact; 346 carries the
    /// word from 8404 to 8582 and the roster turns over again without
    /// changing size — `1/51` goes out and `1/50` comes in, and `1/44`,
    /// the unit 338's own frame is about, has stayed exact through all
    /// three. `1/53` is still on the wrong host by that same vector. The
    /// other five are printed with their offsets and not asserted: past
    /// the word a position is nobody's, and which university a scholar
    /// walks to is still the successor 338 named.
    ///
    /// What the two assertions buy is that a change moving `1/44`,
    /// `1/45`, `1/48` or `1/50` off the original's own coordinate, or
    /// `1/53` off the vector between the two universities, fails here
    /// rather than passing quietly into the count.
    #[test]
    fn great_lakes_scholars_sit_on_their_universities() {
        let row = &ENDPOINTS[1];
        let Some(r) = score(row) else { return };
        // The four the seating puts exactly right, `1/44` — the unit item
        // 338's own frame is about — first.
        let seated = [44, 45, 48, 50];
        let still_off: Vec<i64> = seated
            .iter()
            .copied()
            .filter(|&o| r.off.contains_key(&(1, o)))
            .collect();
        assert!(
            still_off.is_empty(),
            "Great Lakes 24001: scholars {still_off:?} are off run80's own \
             coordinates again. `1/44` sits at (40416, 25248), which \
             is building 1/2019's position; before the seating it \
             stood (24, 552) out on the exit ring (CITIES §6.5.2)"
        );
        // And the one that used to sit on the *other* university, by the
        // vector between the two rather than by a count.
        //
        // **Which scholar it was, was telemetry, and it moved every time
        // anything upstream did**: `1/53` from item 338, `1/52` from 352,
        // `1/56` from 358, `1/55` from 362, `1/58` from 384. Item 342's
        // standing instruction is to re-measure it rather than diagnose it,
        // and on the **384/385 merge** the re-measurement says something
        // better than a new index: **no unit at this endpoint is off by
        // `1/2020 − 1/2019` at all.** The nearest is `1/53` at
        // `(648, 10776)`, which is not one university.
        //
        // So the assertion is now the *vector*, over the whole roster,
        // with no index in it — which is the shape that stops costing a
        // re-measurement a session. It is **not** a claim that the
        // seating is fixed: this is 14,600 frames past the word, two
        // independent items composed to get here, and a scholar that is
        // merely off by something else reads the same. What it does say is
        // that if a later item puts a scholar exactly one university away
        // again, this fails and names it.
        //
        // **And item 389 put one back**: `1/58`, at `(768, −9984)`. That
        // is the failure this assertion was left open to catch, and it
        // caught it on the first item after the one that emptied the set,
        // so the shape is kept and the carrier is named again rather than
        // the test being widened until it cannot fail. It is still
        // telemetry — 14,550 frames past the word, on a change whose
        // whole effect upstream is *when an arrow exists* — and it is
        // still not a claim that the seating is right or wrong.
        //
        // **Item 394 moved it again, to `1/55` at `(−768, 9984)`** — the
        // same vector, a different scholar, and the fifth index this line
        // has carried. Item 342's instruction is re-measure rather than
        // diagnose, and the re-measurement is all this is: 394's whole
        // effect upstream is one `Random::get` on sim-frame 9451, so
        // everything here is that draw's stream 14,550 frames later.
        //
        // **Item 442 emptied it again**, and this time the carrier is not
        // merely displaced: the scholar arm's `val` chain (`docs/AI.md`
        // §53) changes *which* scholars exist and when, so the whole
        // roster past 9510 is a different set of units. Nothing here says
        // the seating improved — 14,550 frames past a word that itself
        // moved 9510 → 10161, this is telemetry, and the set being empty
        // is the same kind of fact as the set being `1/55`. It stays
        // pinned to empty for the reason it was pinned to a name: the next
        // item that puts a scholar exactly one university out fails here
        // and is told so.
        let delta = (768, 9984);
        let carriers: Vec<(i64, i64)> = r
            .off
            .iter()
            .filter(|(_, v)| (v.0.abs(), v.1.abs()) == delta)
            .map(|(k, _)| *k)
            .collect();
        assert_eq!(
            carriers,
            Vec::new(),
            "the set of units one university from the original's own \
             coordinates moved — `1/2020 − 1/2019` exactly, which is what \
             item 338's residue looked like, what the 384/385 merge left \
             nobody carrying"
        );
        // The rest, printed: past the word neither stream is anybody's.
        for o in [49, 51, 52, 53, 56] {
            eprintln!(
                "  scholar 1/{o}: {:?}",
                r.off.get(&(1, o)).copied().unwrap_or((0, 0))
            );
        }
    }

    /// **The two rungs under the endpoint**, each its own game and its own
    /// walk (see the module header): run28's 15,401 and run24's 16,489.
    #[test]
    fn the_east_indies_ladder_is_pinned() {
        // **Both rungs are measured before either is asserted** (item 370):
        // a failure on rung C used to take rung B's measurement with it,
        // and the two rungs are two different games — the second is not a
        // consequence of the first and is worth seeing when the first moves.
        let mut rungs = Vec::new();
        for row in &LADDER {
            let Some(r) = score(row) else { return };
            rungs.push((row, r));
        }
        // **The two rungs are one simulation, and their extras say so**
        // (item 370). run24 and run28 are the same East Indies game stood
        // up from the same borrowed setup, walked to 16,489 and 15,401 —
        // so an object number that is `extra` on both rungs is the *same
        // unit* seen 1,088 frames apart and must carry the same type.
        // `the_ladder_s_borrowed_setup_is_the_endpoint_s` checks that at
        // 6,000, under the word; this checks it past the word, where the
        // counts live, and it is the one thing about `extra` that does not
        // churn with every AI landing.
        let (c, b) = (&rungs[0].1, &rungs[1].1);
        // **A Transport Barge is not a unit that lasts, so its number is
        // not an identity** (item 518). A barge is cast for one crossing
        // and dies on the far shore (`docs/TRANSPORT.md` §13), and its
        // object number goes back into the pool. Measured, not argued:
        // walking rung B with `1/62` watched, both rungs hold the same
        // barge `1/62` (slot 196 from 14142, slot 197 from 15344), and
        // rung B alone walks on past 15401 — the barge dies on 15466 and
        // a **new** Citizen takes number 62 on 15763, in slot 198. So a
        // shared number that is a barge on either rung is two units, and
        // the check below would read the recycling as two simulations.
        let barge = |t: &str| t == "Transport Barge";
        let shared: Vec<((i64, i64), &str, &str)> = c
            .extra_types
            .iter()
            .filter(|(_, _, name)| !barge(name))
            .filter_map(|(w, o, name)| {
                b.extra_types
                    .iter()
                    .find(|(w2, o2, other)| w2 == w && o2 == o && !barge(other))
                    .map(|(_, _, other)| ((*w, *o), name.as_str(), other.as_str()))
            })
            .collect();
        // **Fifteen → ten on item 442**, and the floor is falling for the
        // right reason: the scholar arm's `val` chain (`docs/AI.md` §53)
        // took eight spurious units off rung C and seven off rung B, so
        // there are fewer shared `extra` numbers left to cross-check.
        // This guard dies by success — as the AI stops buying units the
        // original does not, the overlap it needs goes to zero — and when
        // it does, `the_ladder_s_borrowed_setup_is_the_endpoint_s` is what
        // still says the two rungs are one game, at frame 6,000 under the
        // word where the roster is not AI-dependent. Lowered with the
        // number rather than deleted, so the next fall is visible too.
        //
        // **Ten → six on item 478**, `City::regen_roads`' second writer
        // (`docs/ROADS.md` §1.2). The change is not about what the AI
        // buys at all — it is when a city replans its roads after one of
        // its buildings dies — but the roads it lays move both rungs'
        // rosters 9,000 frames later, and four of the shared numbers go
        // with them. The six that remain still agree on type, which is
        // the thing this guard is for.
        //
        // **Six → one on item 576**, the sea branch's dock (`docs/AI.md`
        // §57): the AI offers ships at a Dock that belongs to no city, from
        // 9981 on, and both rungs' rosters are a different set 5,400 frames
        // later. One shared number is left, and it agrees on type. At one
        // the check is all but vacuous; what still says the rungs are one
        // game is `the_ladder_s_borrowed_setup_is_the_endpoint_s`, at 6,000.
        //
        // **One → an exact pin, the twelfth pass** (parked 583). A floor
        // that falls with every landing is vacuous at one and dead at
        // zero, and "not empty" was the last threshold there is. So the
        // count is pinned like every other number in this file and
        // re-pinned by the item that moves it, in either direction — a
        // rise is a new shared extra and worth a look too. The claim
        // that the rungs are one game rests on
        // `the_ladder_s_borrowed_setup_is_the_endpoint_s` at 6,000, not
        // here; this row only says the extras that remain agree on type.
        //
        // **And the first run of the pin read eighteen, not one.** The
        // count had risen 1 → 18 under `!is_empty()` without a word: the
        // East Indies landings after 576 (579's navy, 592's census, 597
        // and 604's Mine, 608's gather count, 613's Horses) put both rungs
        // back on one late roster — twelve Slingers-to-Javelineers and
        // Hoplites-to-Phalanx pairs among them, the upgrade shape item 545
        // named. A floor that only falls cannot see a rise; a pin can.
        //
        // **18 → 19 on item 620**, a Mine's `dist_mod` capped at 3 on a
        // list under `MTN_TINY_SIZE` (`docs/ORDERS.md` §6.4): the late
        // roster reshuffles past East Indies' word, 11069 → 11590, and
        // one more extra number is shared, of one type on both rungs.
        //
        // **19 → 6 on item 629**, a deployed Merchant seated on its tile
        // corner with its square blocked (`docs/MERCHANT.md` §3.2): the
        // late roster reshuffles again past East Indies' word, 11590 →
        // 11747, and six shared numbers remain, each of one type on both
        // rungs.
        //
        // **6 → 3 on item 642**, `Region::go_here` reading the human's
        // city count (`docs/TRANSPORT.md` §9.4): the scout sails for the
        // human's island and the word moves 11747 → 13640. Three shared
        // numbers remain, `1/12` Citizen, `1/56` Cataphract and `1/57`
        // Light Horse, each of one type on both rungs.
        //
        // **3 → 4 on item 643**, an animation's name found case-folded
        // (`docs/ANIM.md` §12): the word moves 13640 → 15782 and four
        // shared numbers remain — `1/12` Citizen, `1/56` Light Horse,
        // `1/57` and `1/58` Cataphract — each of one type on both rungs.
        const SHARED_EXTRA_NUMBERS: usize = 4;
        assert_eq!(
            shared.len(),
            SHARED_EXTRA_NUMBERS,
            "the rungs share {} `extra` object numbers against the pin of \
             {SHARED_EXTRA_NUMBERS}: {shared:?}. Re-pin with the item that moved it",
            shared.len()
        );
        // **An upgrade re-types a standing unit** (item 545). Researching a
        // unit's upgrade converts every one standing (`Unit::set_type`,
        // `docs/ANIM.md`'s conversion pass), so one number can be the same
        // unit under two types when rung C's (15,401, the earlier) is on the
        // `FROM` line of rung B's (16,489). This crate never researched a
        // unit upgrade until item 545 routed `produce_tech`'s unit arm
        // (`docs/AI.md` §56), and the first three it showed were `1/66`–`68`,
        // Slingers on C and Javelineers on B. A number recycled into an
        // unrelated type still parts.
        let loaded = install().and_then(|i| crate::load::load(&i).ok());
        let upgraded = |earlier: &str, later: &str| -> bool {
            let Some(l) = &loaded else { return false };
            let id = |n: &str| {
                l.tree
                    .types
                    .iter()
                    .position(|d| d.kind.is_unit() && d.name == n)
            };
            matches!((id(earlier), id(later)), (Some(e), Some(x)) if l.tree.is(x, e, false))
        };
        let parted: Vec<String> = shared
            .iter()
            .filter(|(_, ours, theirs)| ours != theirs && !upgraded(ours, theirs))
            .map(|((w, o), ours, theirs)| format!("{w}/{o} C {ours} vs B {theirs}"))
            .collect();
        assert!(
            parted.is_empty(),
            "the two rungs disagree on what a shared `extra` object number \
             is, so they are not one simulation seen twice: {parted:?}"
        );
        for (row, r) in &rungs {
            pinned(row, r);
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
