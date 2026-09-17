//! The fixtures every record's tests are built from.

use super::*;
use crate::gamelog::{CityDump, Guy, LeaderDump, UnitDump};

use crate::testenv::dump;

pub(crate) fn initial() -> Initial<'static> {
    Initial {
        world: vec![("xs", "60"), ("ys", "60"), ("seed", "7236")],
        cities: vec![CityDump {
            x: 3168,
            y: 30816,
            pop: 1,
            who: 0,
            ..CityDump::default()
        }],
        units: vec![
            UnitDump {
                flags: 65,
                o: 0,
                who: 0,
                pos: LogPos {
                    x: 4248,
                    y: 32664,
                    z: 528,
                },
                angle: None,
                guys: vec![
                    Guy {
                        kind: Some(0x32 + 19),
                        ..Guy::default()
                    },
                    Guy {
                        kind: Some(0x32 + 19),
                        ..Guy::default()
                    },
                ],
                ..UnitDump::default()
            },
            UnitDump {
                flags: 1,
                o: 1,
                who: 0,
                pos: LogPos {
                    x: 4248,
                    y: 28680,
                    z: 496,
                },
                angle: None,
                guys: vec![Guy {
                    kind: Some(0x32),
                    ..Guy::default()
                }],
                ..UnitDump::default()
            },
            // An animal, which the harness ignores.
            UnitDump {
                flags: 1,
                o: 3,
                who: 255,
                pos: LogPos::default(),
                angle: None,
                guys: vec![],
                ..UnitDump::default()
            },
        ],
        leaders: vec![
            LeaderDump {
                who: 0,
                tribe: 11,
                ..LeaderDump::default()
            },
            LeaderDump {
                who: 8,
                ..LeaderDump::default()
            },
        ],
        ..Initial::default()
    }
}

/// A trace beside the dumps — `RON_GAMELOG_DIR`, where `archive.sh`
/// puts both. `None` skips the half of a check that needs it, the same
/// way [`dump`] does.
pub(crate) fn trace(name: &str) -> Option<crate::trace::Trace> {
    // The same default as [`dump`]: a machine with the captures but no
    // `RON_GAMELOG_DIR` used to skip every trace-backed half silently.
    let path = dump(name)?;
    Some(
        crate::trace::Trace::read(std::path::Path::new(&path))
            .expect("invalid finalized trace")
            .expect("missing RONT header"),
    )
}

/// The two sequences a whole frame folds to, in one place — ours from
/// [`mark_sites`], the original's from
/// [`Trace::labels`](crate::trace::Trace::labels) — and the index they
/// first part at, with a window either side.
///
/// A `Vec<String>` comparison of 175 entries prints as a wall; the
/// first difference and its neighbourhood is the whole of what a
/// reader needs, so the failure message is built rather than left to
/// `assert_eq!`.
pub(crate) fn first_parting(ours: &[String], theirs: &[String]) -> Option<(usize, String)> {
    let at = (0..ours.len().max(theirs.len())).find(|&i| ours.get(i) != theirs.get(i))?;
    let lo = at.saturating_sub(3);
    let hi = (at + 4).min(ours.len().max(theirs.len()));
    let mut s = format!("the sequences part at draw {at}:\n");
    for i in lo..hi {
        let mine = ours.get(i).map_or("—", |x| x.as_str());
        let yours = theirs.get(i).map_or("—", |x| x.as_str());
        let flag = if mine == yours { "  " } else { "≠ " };
        s.push_str(&format!("  {flag}{i:>4}  ours {mine:<52} theirs {yours}\n"));
    }
    Some((at, s))
}

/// The sibling dumps of the run9/run10/run11 map that carry what the
/// others lack (`run_traced`): run11 (the setup path's checksum trace),
/// run3 (`DUMP_ALL` — the terrain heights, the regions' coordinate
/// lists) and run12 (`DUMP_ALL` with the per-frame sync words and the
/// herds, `docs/SYNC.md`). Whichever the machine has.
pub(crate) const SIBLING_DUMPS: &[&str] = &[
    "gamelog-run11-checksum.txt",
    "gamelog-run3-fulldump-types.txt",
    "gamelog-run12-dumpall-seeds.txt",
    "gamelog-run13-window-95-105.txt",
];

pub(crate) fn sibling_texts() -> Vec<String> {
    SIBLING_DUMPS
        .iter()
        .filter_map(|n| dump(n))
        .map(crate::capture::read)
        .collect()
}

/// Retain bounded setup text plus owned observations, not whole sibling files.
/// Each scoped initial lives until the innermost callback returns.
pub(crate) fn with_sibling_initials<R>(use_initials: impl FnOnce(&[&Initial<'_>]) -> R) -> R {
    fn visit<R>(
        paths: &[String],
        inits: &[&Initial<'_>],
        use_initials: impl FnOnce(&[&Initial<'_>]) -> R,
    ) -> R {
        let Some((path, rest)) = paths.split_first() else {
            return use_initials(inits);
        };
        crate::capture::indexed::IndexedCapture::open(path)
            .unwrap()
            .with_replay_initial(|init| {
                let mut refs = inits.to_vec();
                refs.push(&init);
                visit(rest, &refs, use_initials)
            })
            .unwrap()
    }
    let paths: Vec<String> = SIBLING_DUMPS.iter().filter_map(|n| dump(n)).collect();
    visit(&paths, &[], use_initials)
}

#[test]
fn indexed_market_road_frame_preserves_world_fields_and_heights() {
    let Some(path) = dump("gamelog-run72-greatlakes-marketroad.txt") else {
        return;
    };
    let text = crate::capture::read(&path);
    let whole = Log::parse(&text);
    let mut source = crate::capture::indexed::IndexedCapture::open(&path).unwrap();
    let index = source
        .frames()
        .iter()
        .position(|f| f.number == 4_804)
        .unwrap();
    let slice = source.read_frame(index).unwrap();
    let bounded = Log::parse(&slice);
    let world_fields = |log: &Log<'_>| {
        let (_, frame) = log.frames().into_iter().find(|(n, _)| *n == 4_804).unwrap();
        frame
            .kid("FULL DUMP")
            .unwrap_or(frame)
            .kid("WORLD")
            .unwrap()
            .fields()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect::<Vec<_>>()
    };
    let expected = world_fields(&whole);
    assert!(!expected.is_empty());
    assert_eq!(world_fields(&bounded), expected);
    let heights = whole.frame_heights(4_804);
    assert!(heights.len() > 1_000);
    assert_eq!(bounded.frame_heights(4_804), heights);
}

#[test]
fn indexed_road_setups_preserve_every_initial_field_except_audit_bodies() {
    for name in [
        "gamelog-run10-world6-long.txt",
        "gamelog-run71-greatlakes-5k.txt",
    ] {
        let Some(path) = dump(name) else { continue };
        let text = crate::capture::read(&path);
        let log = crate::gamelog::Log::parse(&text);
        let mut expected = log.initial().unwrap();
        expected.frame_bodies.clear();
        let mut source = crate::capture::indexed::IndexedCapture::open(&path).unwrap();
        for _ in 0..2 {
            source
                .with_replay_initial(|actual| assert_eq!(actual, expected, "primary {name}"))
                .unwrap();
        }
    }
}

#[test]
fn indexed_siblings_preserve_every_initial_field_except_audit_bodies() {
    // Exercise reuse before comparing every retained field to whole-text parsing.
    with_sibling_initials(|_| ());
    with_sibling_initials(|inits| {
        let paths: Vec<String> = SIBLING_DUMPS.iter().filter_map(|n| dump(n)).collect();
        assert_eq!(inits.len(), paths.len());
        for (path, actual) in paths.iter().zip(inits) {
            let text = crate::capture::read(path);
            let log = crate::gamelog::Log::parse(&text);
            let mut expected = log.initial().unwrap();
            expected.frame_bodies.clear();
            assert_eq!(**actual, expected, "sibling {path}");
        }
    });
}

/// East Indies' word on run54, the headline.
///
/// **6739 since 2026-09-03, and 6715 was one line of
/// `Unit::unpack_merchant@006038e0`'s tail.** After the `MOVE_TO` goes
/// on the list comes `clear_partial_path`, then **`head = head->next`**
/// — the same rotation `add_cast_order`'s `QUEUE_FIRST` arm runs
/// (`docs/ORDERS.md` §1.5) — and only then `update_action`. So the walk
/// to the deploy spot sits in **front** of the unpack cast: the
/// merchant walks, and casts on arrival. This crate appended it
/// instead, so the cast stayed current, `update_action` stopped on it
/// (a cast is neither a plain move nor a `CHANGE_FORM`), and `1/19`'s
/// `orders_x/y` held the merchant's own position where the original's
/// names the spot. The spot itself was never wrong — run68's block
/// 6714 is the first capture ever to reach `find_merchant_spot`'s ring
/// (`docs/MERCHANT.md` §3, §6) and both sides pick tile `(168, 192)`,
/// the fourth entry of `MOVE_49`. It is also the first field of the
/// whole record to part, a frame ahead of the draw stream, which is
/// the argument for diffing fields and not only draws.
///
/// It was **6570** for one item, and the frame is a **collision the
/// original does not have**: `Guy::set_anim+0x97a <
/// Unit::move_step+0x823`, the blocked stand (`docs/COLLISION.md` §5),
/// twice — once per figure — on the AI Merchant two hundred frames
/// into its walk. The unit in its way is the AI's own citizen `1/2`,
/// standing still on a gather order 180 units off, and this crate's
/// exemption ladder does not let the two past. `PathFinder::find_wpath`
/// is first entered on the original's frame 6602, so the original's
/// merchant meets something of its own thirty frames later; whether it
/// is this citizen is not settled.
///
/// It was **6356** for one item, and the frame was the **Merchant's
/// first step**: `Guy::set_anim+0x97a < Guy::do_turn+0x4a <
/// Unit::move_step` at the head of the original's frame, three frames
/// after the unit was born. `Unit::think_merchant@005f4740` is what
/// sends it — it scores the leader's `new_rares` list, moves the
/// winner to the back of that list and issues a `MOVE_TO` at the good
/// — and nothing here reached it (`docs/MERCHANT.md`). Landing it
/// alone was not enough: the merchant then turned **twice** a frame
/// where the original turns once, because `Unit::init@00612100:549`'s
/// `set_new_location(·, ·, 1, 1)` seats a **tracked** crew figure on
/// its offset at birth and this crate seated only the figures a dump
/// or a disembark handed it. A trackless crew figure is one
/// `Guy::do_turn@005d97a0` recurses into, and a merchant's figures all
/// carry `guy_flags & 8` because the type packs — so it asked for a
/// turn its art has not got and paid the idle roll beside its driver
/// (`docs/MERCHANT.md` §5). `Sim::init_guys` now ends where
/// `Unit::init` does.
///
/// It was **6353** for one item, and the frame was a **unit the
/// original trains and this crate did not**: two `Guy::init_real+0x52`
/// under `Objects::init_unit`, a two-figure unit, and run65's window
/// names it — the AI Market `1/2013` is one `MERCHANT` deep on all
/// eighteen of its blocks, with a `job_counter` climbing a hundred a
/// frame from the caravan's hand-over on 6164 to 18,720 (`JOB_TIME`
/// 156 at `UNIT_RATE_BASE` 120) on 6353. The cause was one host
/// function: `economic.bhs` trains merchants only up to
/// `num_rare_resources_seen(who)`, and this crate's script host
/// answered a flat zero because rares were not in the simulation when
/// it was written. `Leader::new_rare@006d9e70` is the list's writer,
/// `World::reveal_fog` its caller, and `FISH`/`WHALES` are excluded
/// because they pay a fishing boat rather than a merchant
/// (`docs/ECONOMY.md`, "The rares a leader has seen"). East Indies'
/// AI has seen `CITRUS` and `HORSES` by frame 5000; the queue and its
/// clock are now an assertion
/// (`run65_s_window_is_the_original_s_unit_for_unit`).
///
/// It was **6207** for one item, and that item was neither the caravan
/// nor its turn. run65 is the capture — an eighteen-frame `DUMP_ALL`
/// window over the leg — and what it shows is both sides pushing the
/// same detour node `(38508, 40620)` on the same frame, turning
/// through the same eight bearings at the same rate, and computing the
/// same full step on sim-frame 6206 to the same point,
/// `(38750, 40508)`. The original does not take it:
/// `move_step@005faf30` at `005fb7c1` compares the step's tile against
/// the one the unit stands on and, where they differ, asks
/// `UnitData::invalid_loc` with all five flags clear — and that point
/// is a tile inside the caravan's own city's footprint. The step is
/// dropped whole, with no `set_anim` and no `set_new_location`, the
/// waypoint is kept, and the unit walks a frame later through a tile
/// it may have. It is the last of the four things `docs/MOVEMENT.md`'s
/// `move_step` section listed as read and not modelled, and the whole
/// window is now an assertion
/// (`run65_s_window_is_the_original_s_unit_for_unit`, 5,186 fields).
///
/// It was **6198** for one item, and that item was `Unit::do_trade`'s
/// **first instruction**: `set_anim(CHAR_DEFAULT, 0, 1)` at `+0x40`,
/// ahead of the caravan-slot test and every one of the function's
/// returns. It draws nothing while the unit is walking — a walk-
/// category figure whose body has not arrived leaves `Guy::set_anim`
/// without a roll — so the site is silent for the thirty frames the
/// caravan spends reaching its city and then spends three draws, one
/// through `Unit::set_anim+0x56` and two through `+0xb6`, on the frame
/// it stands still. The order queues its next leg with `QUEUE_FIRST`,
/// so `do_trade` never runs while the walk does: **its later frames
/// are its arrivals**, and 6511, 6766 and 7021 are the next three.
///
/// The nine frames after it are the legs (`docs/CARAVAN.md` §7), and
/// what carried them was not the reading: `Caravan::build_road`'s road
/// is a stack of **world** positions with a tolerance and a flag byte,
/// and this crate had been keeping tile coordinates — so `set_road_at`
/// was handed numbers thirty thousand tiles off the map and no trade
/// road in the port had ever been laid. The `CARAVAN` record prints
/// the whole stack and nothing had compared it (item 87's ledger);
/// twenty-six nodes, right on the first run.
///
/// **6189 was the danger map**, and the whole of item 177's first half
/// with it. `WorldData::danger[who]` is a half-resolution `int` grid
/// `GameDaemon::calc_danger` rebuilds every two hundredth frame, and
/// `calc_cost`'s world arm prices a step by `danger / 8`. Around a
/// leader's own city the map is 65 to 135 **negative** — its own
/// buildings subtract — so eight of the caravan's own steps were 8 to
/// 16 too dear here, the search took a seventh expansion the original
/// does not, and a route came back where the original's stops at the
/// goal's own neighbour and pushes nothing. That is why `do_trade`'s
/// move went in with a waypoint at `(37752, 41592)` and the caravan
/// set off south-west where the original heads straight at
/// `(39288, 40056)` (`crates/sim/src/danger.rs`, `docs/DANGER.md`).
/// The map itself is now diffed whole against run64's own
/// `danger[who][scan]`, 7,200 values
/// (`run64_s_danger_map_is_the_original_s`), and so are the
/// forty-seven prices of the search it broke
/// (`run64_s_frame_6167_prices_are_the_originals`).
///
/// It was **6189** for one item, and 6169 was **an empty animation packet**. Sixty of the shipped `<UNIT>` entries in
/// `unit_graphics.xml` have no `<ANIM>` child at all and every one of
/// them is a `-CREW{k}`: the piece loads a model, so
/// `get_unit_gpiece` hands it out, and its packet names nothing — so
/// every slot is `AnimationPacket::get_game_frames`' three frames and
/// `Guy::inc_time`'s `packet->ids[slot] >= 0 && loopings[id]` is false
/// for all of them. A caravan's two crew figures therefore walk three
/// frames, fall to `CHAR_DEFAULT`, roll, and are put back on the walk
/// by `Guy::move` the next frame: **two draws every third frame, for
/// the rest of the game** — 11,872 of them over run54's remaining
/// 17,800 frames, which is why one item moved the word by only
/// twenty-nine and would have blocked every later one. This reader
/// dropped the animation-less entry outright, so the walk fell to
/// `first_unit_piece` and the crew played the citizen's looping art
/// (`docs/ANIM.md` §3.6).
///
/// It was **5592** for one item, and it was **5466** until a building
/// started flattening
/// the ground under it before it planned its road. 5466 was the AI
/// Market's own placement frame, one road search of 184 nodes against
/// the original's 205, and the cause was two mechanics at once:
/// `calc_road_cost` calls `was_seen` and not `was_really_seen`, and
/// `Wall::start` runs `TerrainOut::terraform_for_building` **before**
/// `mask_me`. run62 is the capture that could say so — the road
/// search's gate and its price proxied together, so the original's own
/// 2,913 node prices are on the record (`docs/ROADS.md` §7.2–§7.4).
///
/// It was **5437** before that, and **5376** before *that*, when
/// `find_friends` started asking whose *footprint* covers a cell's
/// centre tile rather than whose centre is in the cell (`docs/AI.md`
/// §26).
///
/// **6166** since 2026-09-02, and 6164 was **a Caravan's figures**.
/// `Unit::init@00612100:471`–`508` sizes the guy stack to `crew_size +
/// squad_size`, and `squad_size` is the literal 1 `UnitType::init@
/// 0061ab50:723` writes into every type — so a unit has
/// `unitrules.xml`'s `CREW_SIZE` figures plus one, and the Caravan's
/// column is 2. `Sim::init_guys` made one figure for every unit it
/// built, so 6164 spent one `Guy::init_real+0x52` against three and
/// 6165 was missing the two `Guy::set_anim+0x97a < Unit::do_idle+0x7d`
/// the absent figures owe. A unit stood up *from* a dump never had it —
/// `build_sim` gives it the file's own `GUY` blocks — which is why
/// twenty scout dogs walked correctly throughout
/// (`docs/ANIM.md` §3.5,
/// `every_dumped_unit_has_crew_size_plus_one_figures`). The frame it
/// leaves is a road: 3,207 `PathFinder::calc_road_cost+0x46` draws on
/// 6166 that this crate does not spend.
///
/// It was **6164** for one item, and 5819 was the water a transport
/// barge is born on. `UnitType::find_nearby_spot`'s `(-1, -1)` form — the one
/// `Unit::do_cast` and `SpellType::cast_transport` ask for that water —
/// does **not** take the pairwise collision pair: `0061deb0` sets the
/// flag that selects it only when `not_o` and `not_who` are both
/// non-negative, so the sweep falls to `find_unit_with_radius`, whose
/// predicate is `vector_dist <= other.big_radius + r_coll` and whose
/// ordered sibling is skipped outright. The Chebyshev pair refused the
/// bearing the original takes — the caster itself, three cells and four
/// cells away, is inside `3 + 1` cells but 228 units from a reach of
/// 192 — so `1/18` was born two tiles closer to its destination and ran
/// ahead of the original for the rest of its life
/// (`docs/ORDERS.md` §10, `docs/TRANSPORT.md` §6.1).
///
/// It was **6571** for one item, and the frame was one crew figure's.
/// The AI Merchant `1/19` collides on 6571, and `Unit::set_anim`'s
/// blocked stand (`move_step+0x823`) asks both its figures to idle:
/// the original spends one draw and this crate spent two. The figure
/// was standing exactly on its destination on both sides, so the
/// walking-guy early return had no reason to fire — until run67's
/// `GUYS=4` window printed `des_x`/`des_y` and the answer turned out
/// to be a writer nobody had modelled. `move_step` opens with
/// `Unit::set_angle(heading)`, whose `Guy::set_angle` tail rewrites
/// every crew figure's `des` from guy 0's point and *that* angle —
/// the bearing, not the facing — so one frame's worth of turn,
/// 720,896, moves a `(-48, -192)` track one unit on each axis and the
/// figure is off its destination again before the stand asks.
/// Two siblings came with it: the cell-centre snap **teleports** the
/// crew (`Unit::set_new_location`'s `param_3` reaches
/// `set_new_location(crew, des, 1)`), and the walk slot is resolved
/// from the **asked guy's own** average speed, so a figure paid
/// `(speed * 11) / 8` to keep station jogs where its leader walks
/// (`docs/MOVEMENT.md`, "Who writes it, and when";
/// `run67_s_window_is_every_figure_s_whole_record`, 13,545 fields).
///
/// **6715 since 2026-09-02, and 6574 was the second Merchant's
/// destination.** `ObjectsData::find_unit_ordered@0065bc40` — the
/// second of `think_merchant`'s three object searches, "is one of my
/// own kind already on its way here" — measures the candidate's
/// `orders_x`/`orders_y` (`UnitData +0x70/+0x74`, the listing at
/// `0065be35`), where its twin `find_unit@0065ca80` measures the
/// object's own position (`0065cd0f`). The decompiler prints both as
/// `vector_dist(unaff_EDI, unaff_ESI)`. This crate asked the body, so
/// `1/19`'s rare read as free once `1/19` had walked 2,800 units
/// clear of its cell centre, and `1/20` was born on 6571 and sent at
/// a good already taken. It showed as a *frame* rather than a
/// destination because the original's path to the far rare is 22
/// entries and `do_move`'s `if (path.length > 10) return 1` holds its
/// first step back a frame, where a seven-entry path steps at once
/// (`docs/MERCHANT.md` §2.2.1). Item 187 — five
/// `Guy::set_anim+0x104b` coins read as gaia's birds — was the same
/// merchant's crew and closed with it.
///
/// It was **5819** before that, and 5669 was a **whale**. The AI's
/// second Fisherman settles on one on frame 5551; `Leader::calc_gather`
/// step 6 walks the idle fishermen, lights the rare's bit in
/// `rare_owned`, and `Leader::calc_unit_stats` hands every naval type
/// `WHALES_SHIPS_MOVE` — `myspeed` 38 → 45 on the same frame, in the
/// dump and now here. The third Fisherman, still walking to its own
/// deposit, then arrived twenty-three frames late and did not spend
/// `think_fish`'s 175 draws on 5669 (`crates/sim/src/rares.rs`,
/// `docs/ECONOMY.md` step 6; run63 is the capture, and run59's census
/// had the same finding as an income of 160 food and 160 wealth the AI
/// was not earning).
///
/// It was **5669** for one item, and 5592 was one AI citizen's first idle
/// frame: `think_peasant`'s tail sends a worker off to explore the
/// moment it stands in a region none of its leader's ten sites claims,
/// and waits six idle frames when one does (`docs/SCOUT.md` §11.1).
/// The original's site list gains the citizen's own cell on 5577 and
/// this crate's could not, because `blocked_location` refused every
/// city site in a region the leader had none in: `COLONIZE_BONUS` is a
/// **technology's** prerequisite — the fourth of `rules.xml`'s
/// `TECHBONUSES`, Coinage — and was carried here as a nation flag
/// nothing set. run63 is the capture that says so
/// (`run63_s_window_is_where_the_ai_s_colony_site_appears`).
/// **6739 until 2026-09-04, and its frame was a crew figure's turn.**
/// A tracked crew guy runs its own `Guy::process -> Guy::move`, and
/// standing on its offset owed a turn it reaches the standing arm's
/// `turn_towards -> do_turn(..., 1)` exactly as guy 0 does — the
/// override `guy_flags & 8` answers, which every guy of a **packing**
/// type carries. This crate turned that figure and never asked for the
/// animation, so a merchant's driver caught up with its leader for
/// free where the original pays the idle roll
/// (`docs/ANIM.md` §4.8, item 212). Great Lakes' 6463 is the frame
/// that named it; East Indies came with it.
///
/// **7448 until 2026-09-06, and the frame was a blocked stand this
/// crate reached 792 units short of** — `1/20`, the AI's Merchant,
/// walking into gaia's animal 34 frames late (run85). The 792 was made
/// on the **transport ride** between run82's 6929 and run85's 7400, and
/// run86 — the whole of that gap, 486 blocks — names it in one field:
/// the Transport Barge `1/22` prints `myspeed` **30** on a `MOVES` of
/// 25 and steps (−27, −13) a frame where this crate stepped (−23, −11).
/// [`sim::Sim::cast_transport`] set the boat's speed from the type's raw
/// `MOVES` instead of `Unit::update_speed`'s cached value, so the boat
/// alone missed the **Whales** rare's `+WHALES_SHIPS_MOVE%` — a bonus
/// this crate has had since run63 and applied everywhere a unit is born
/// except the one place a boat is. 190 frames at 25 against 30 is 933
/// units of lag, the passenger comes ashore that much later, and the
/// stand at 7448 is met on time with the fix
/// (`docs/TRANSPORT.md` §6.2, `run86_s_window_is_the_transport_ride`).
///
/// **7529 for a day, and the frame was an age.** The AI citizen `1/13`
/// carried a `GATHERORDER` `wait` of `theirs + 1` on all 55 blocks of the
/// cycle that ends there, so the original's countdown ran out on 7529
/// where this crate still held 1. The countdown's arithmetic was never
/// wrong: it is seeded by the **arrival** frame, and this crate arrived a
/// frame late. run86 has the whole of it — the *only* angle divergence in
/// its 487 blocks is `1/13`'s facing on block **6937**, which is the
/// block every one of player 1's buildings flips `max_age` 0 → 1 on.
/// `Leader::gain_tech`'s `is_age_type` arm re-places every unit of the
/// leader with `Unit::set_new_location(u, u.x, u.y, 1, 1)`, and those two
/// snap flags write the figure's facing and body outright — a free turn
/// for the one unit that was mid-turn, and nothing at all for the other
/// 21 (`docs/TECH.md`, "An age snaps every figure";
/// [`sim::Sim::gain_tech`]). It cost `1/13` one frame at the tile, 545
/// frames of `+1`, and 277 frames of this counter.
///
/// **7806 for a day, and the frame was a waypoint this crate walked to.**
/// run90 dumps the whole shuffle — 111 blocks over `[7790, 7900)` — and
/// `1/6`'s position parted on block **7805**, two blocks *below* the draw
/// stream's word, which is the standing rule in one row: a stream agrees
/// on a wrong destination for a while and only a value diff dates it. Both
/// sides push the same §6 step 4 sidestep, `(39720, 38808)` `flags 2`; the
/// original's path drops 5 → 4 on the very next block with the unit at
/// (39729, 38802), which is not that point. It takes one step along the
/// bearing and **abandons** the waypoint.
///
/// The rule is one store that is not made.
/// `resolve_unit_collision@005f9d30`'s `LAB_005fa37a` pushes the entry and
/// writes the order's `+0x2c`/`+0x30`, and touches `UnitData::tolerance`
/// **not at all**; the entry's own `tolerance 0` reaches the unit only
/// through `do_move`'s `dest == 0` take, which this waypoint never goes
/// through because `dest` is already 1 by the time `move_step` runs. So
/// the sidestep is walked under the *current leg's* tolerance — 384 for a
/// citizen on a world-grid plan — and `move_step`'s post-step Manhattan
/// test (`005fb45f`: `tolerance < |dx| + |dy|`) retires it on the first
/// successful step, from fifteen units away. This crate zeroed the
/// tolerance with the push and had to walk the remainder as a second step:
/// a five-block cycle against the original's four, one frame per
/// collision (`docs/COLLISION.md` §8.7, item 289).
/// **7812 until the suspend wired, 2026-09-17.** `find_upath`'s pre-walk
/// gate, its `500 / max(1, repaths²)` limit and `do_move`'s suspended-search
/// block (`docs/ORDERS.md` §4.4 step 2) were read, implemented and banked
/// *unwired* by item 301, because wiring them cost Great Lakes' long word
/// 7679 → 6862. Item 304 named that cost and it was not the suspend's:
/// `Unit::kill_current_path@005e31d0` frees the stash with its pop, inside
/// its own `0 < length` guard, and this crate had the pop alone — so on
/// run76's 6860, the frame the original's own squad degrades out of
/// formation, every follower kept a suspended search no order was left to
/// serve and stood in `do_move`'s block for the rest of the capture. With
/// that one call under the pop the wiring is free on Great Lakes and this
/// word runs to **8193** (`docs/PATHFINDER.md` §18.5).
pub(crate) const LONG_WORD_EAST_INDIES: i64 = 8_193;

/// Great Lakes' word on the **long** capture (run53), the second of
/// `docs/DECISIONS.md` entry 29's counters — and, since run61 put the
/// bird's birth right, no longer the same number as the scored
/// capture's. It was **1802** for as long as the flight was unmodelled;
/// `Gaia::spawn_bird`'s tile snap took it to **2419**, and the scored
/// run33 to the end of its own 1,850 frames.
///
/// It was **2419** for two days, and the frame was a **route**: the AI
/// woodcutter `1/9` turned one 48-grid step early on its walk to a
/// tree, arrived a frame early, and spent its chop clock's own draw a
/// frame early four hundred frames later. run70's `callwin` over
/// `PathFinder::calc_cost` settled it in one window — the original
/// never prices `(40776, 17592)` at all, so the search agreed and the
/// **grid** did not — and `Guy::process@005e0230`'s sixty-fourth-frame
/// repaint of the collision block is what this crate was missing
/// (`docs/COLLISION.md` §2.2, `docs/PATHFINDER.md` §17).
///
/// It was **2808** for a day, and that frame was a **job**: the AI's
/// citizen `1/1` finishes building `2010` on 2803 and the original
/// sends it straight to the next site, `2011`, where this crate sent
/// it to gather at the one it had just put up. `Unit::build_done`'s
/// AI arm is `find_build_spot() or find_repair_spot() or
/// find_gather_spot(range)` and only the last of the three was
/// modelled (`docs/ORDERS.md` §5.5). With the search in, `1/1` takes
/// the original's move order on 2803 and its point on 2804, and the
/// word runs to 2930.
///
/// It was **2930** for a session, and that frame was a **farm
/// animal's**: the pasture `2007`'s slot-0 animal, whose
/// `(o · (slot + 1) + frame) % 128` phase lands on 2930, takes
/// `Animal::think_farm_animal`'s one draw in the original and none
/// here. Neither the phase nor `build_covers_tile` was wrong — the
/// **reference object** was. `think_farm_animal@005d7700` picks it on
/// `BuildData::num_gatherers(this, 1, 0)`, and that first argument is
/// `is_gathering_at`'s `arrived`: a citizen joins the chain the moment
/// `add_gather_order` issues and sets `been_there` only when it gets
/// there (`docs/ORDERS.md` §6.1). This crate read the chain's
/// *length*, so a citizen still walking — five tiles off the pasture —
/// became the measured object, the farm did not cover its tile, and
/// the draw was dropped. With the arrived count in, the word runs to
/// **4241**, and run69's 3,000 frames go from six units ever off the
/// original's point to **none** (`docs/SYNC.md` §3.6).
///
/// The word is now past the whole of run69, so Great Lakes is owed a
/// longer full-detail capture (`docs/DECISIONS.md` 29).
///
/// It was **4241** for a session, and run71 — the capture that owed
/// answer — put the *position* parting 64 frames below it, at 4177,
/// where **two** units left the original's point on one frame. They
/// were one defect: on that frame the AI places its farm `2014` and
/// pulls a citizen off gathering to build it, and the two sides pull a
/// **different citizen**. `produce_building`'s builder loop measures
/// the distance from the **corner tile** to the unit's own tile, each
/// coordinate floored on its own (`006e28b2`–`006e28ec`); this crate
/// took the difference in world units and divided once. `1/11` and
/// `1/19` **tie at 27** under the original's arithmetic and the earlier
/// unit keeps the tie, where difference-then-divide read 28 against 25
/// and sent `1/19`. With the floors in, the word runs to **4803**, the
/// position parting to **4827**, and run71's buildings — the 425 fields
/// of `1/2015`'s `y_internal` from 4577 — come right on their own
/// (`docs/AI.md` §2.20).
///
/// It was **4803** for a day, and that frame was a **road**: player 1's
/// Market `o 2015` finishes there and plans its road to London, and the
/// original priced 277 nodes against this crate's 266. The eleven were
/// one tile. `place_roads` lays the Market's ring — sixteen tiles, the
/// border of `[224, 228] × [79, 83]` — and the original lays a
/// **seventeenth**, `(223, 79)`, which no ring puts there:
/// `Roads::set_diags@0088e9d0`, reached from every
/// `World::set_road_at` through `road_added` → `add_roads`, fills the
/// gap between a brand-new road and one standing diagonally from it
/// when neither tile between them is anything at all. The ring's
/// `(224, 79)` and the standing road at `(223, 80)` are such a pair.
/// The search priced the tile as plain ground at 387 where the original
/// priced it as road at 27 (`docs/ROADS.md` §9, `crate::mesh`).
///
/// With the mesh in, the word runs to **5502** and run71's whole
/// capture — 5,000 frames — has **no unit anywhere off the original's
/// point**, the 4827 parting included. 5502 was a **move**: this crate
/// spent a fifth draw, `Unit::do_move+0xe84`, where the original spent
/// four.
///
/// It was the citizen `1/7`, and it was **`resolve_unit_collision`'s
/// last store**. `005f9d30`'s two closing blocks both clear the order's
/// `+0x10` — `dest = 0` — and the only thing the successful one adds is
/// the pause roll; this crate took the fresh `find_upath` plan's top as
/// the waypoint instead. The top of that plan is the unit's own snapped
/// cell, so `1/7` stood **on** its waypoint with `dest` set: `do_move`'s
/// arrival test, which only runs on the frame a waypoint is taken, never
/// ran, and the frame fell through to the grid roll. Worse, the frame
/// after popped the plan and walked it back into the same collider —
/// a two-frame livelock that ran to the end of the capture. With the
/// store as the original writes it, `1/7` walks its detour and reaches
/// its farm on 5508, and the word runs to **5571**
/// (`docs/COLLISION.md` §6, item 204).
///
/// It was **5571** for a session, and that frame was **two animation
/// wraps**: the caravan `1/23`'s two crew figures, one frame ahead of
/// the original's three-frame metronome. run73 — this map's first
/// `DUMP_ALL` window — put the difference in one field, and it is a
/// call this crate never made: `Unit::move_step` asks every guy for
/// `CHAR_WALK` immediately before `set_new_location`, so a walking
/// unit's walk is requested **twice** a frame, and only the second
/// request — the same slot by then — takes an overrun length off the
/// clock (`docs/ANIM.md` §4.9,
/// `run73_s_window_clocks_are_the_original_s`). With it the word runs
/// to **5573**, and the frame past it is a road: the caravan's own
/// `build_road`, 1,754 `calc_road_cost` draws here against 1,528.
///
/// It was **5573** for a session, and the road was not the mechanic.
/// run73's `callwin` carries all eight frames of `1/23`'s search node
/// for node, and the first to part is 2,170 of **5572** — the diagonal
/// from `(216, 123)` to `(217, 124)`, a footprint tile of player 1's
/// Farm `o 2012` that was **road** here and plain ground there.
/// `valid_roadcoord`'s occupied arm lets a road through a footprint and
/// refuses everything else, so one stray tile of tarmac opened a door
/// the original keeps shut, and every node after it was somebody
/// else's. The tarmac is `BuildType::mask_me@006312a0`'s road arm,
/// which this crate did not have: a footprint tile of a type that does
/// not connect to roads has its road **taken away** when the building
/// starts (`docs/ROADS.md` §9.5). It is also the last of run72's three
/// unexplained mask residues, 770 frames earlier. With it the word runs
/// to **5786**, by draw and by sequence.
///
/// It was **5786** for a session, and the draw was the **only turn
/// either side spends in 5,800 frames**:
/// `Guy::set_anim+0x97a < Guy::do_turn+0x4a < Unit::move_step+0x389`,
/// the far turn-in-place arm, whose idle roll only a guy with
/// `guy_flags & 8` and no `CHAR_TURN_RIGHT` in its packet ever pays.
/// Six units were moving and exactly one **packs** — the AI's Merchant
/// `1/24`, `docs/ANIM.md` §4.8's own row — so the unit was named before
/// run74 was booked.
///
/// The turn was a symptom. run74's window puts the two merchants side
/// by side and they are **walking to different rares**: the original's
/// order is `MOVE_TO (40344, 14232)` on a seven-node path north-east
/// and this crate's `(31800, 21816)` on a sixteen-node path across the
/// map. `think_merchant` scores `LeaderData::new_rares`, and this
/// crate's list held two goods where the original's held three.
///
/// The missing one is the harness's, not the simulation's.
/// `Sim::reveal_fog` is reached from one place — `World::set_seen`
/// answering that `seen2` **changed** — so a good is offered to a
/// leader once for the life of a game; and [`build_sim`] *installs*
/// the dump's `seen2` rather than walking the sweeps that produced it.
/// Every offer the original had already made when the block was
/// written was therefore skipped, unrecoverably: the cells are seen,
/// so `set_seen` can never answer true for them again. The `SILK` at
/// `(40320, 14208)` was seen at game start, so it is first in the
/// original's list and worth the full 200 against the 190 and 180 of
/// the two this crate had. `Sim::seed_new_rares_from_fog` replays
/// those reveals once, after the goods and the leaders' `human` bits
/// are in, and the word runs to **6080**.
///
/// It was **6080** for a session, and that frame was **41 draws the
/// original spends none of**: the AI scout `1/0` arrives at its
/// explore target, goes idle and runs the whole of `docs/SCOUT.md`,
/// where the original's is still walking and the frame is birds and
/// farms alone. The scout was not the mechanic. run75 puts the two
/// walks side by side and they take the **same order to the same
/// cell on the same frame** — 5851, an eight-node path the capture
/// prints node for node identical — and the original's arrives
/// **seventy-one frames later**.
///
/// The seventy-one frames are a speed. `UnitData::get_speed`'s third
/// layer halves a land unit's step while it stands on a tile carrying
/// `0x800` with its own `z_internal` not above zero, and this crate
/// had **none** of that layer: [`sim::Sim::get_speed`] answered the
/// cached aura speed for everything that is not an animal. run75's
/// `z_internal` is 14 on 5948, 0 for all of 5949–6089 and 17 on 6090,
/// and the step is 34 outside that span and 17 inside it — the river
/// bed south of the AI's second city
/// (`run75_s_window_is_the_scout_s_walk_down_the_river`,
/// `docs/MOVEMENT.md`). With it the scout is on the original's point
/// for every frame of the window and the word runs to **6151**, whose
/// own frame is one figure draw: `Guy::move+0x19f` here against
/// `Guy::inc_time+0x271` there.
/// **6463 until 2026-09-04**, and that frame was one draw: the
/// original spends a `Guy::set_anim+0x97a < Guy::do_turn+0x4a <
/// Guy::turn_towards+0x69` this crate spent none of. run18b is run53's
/// own game (`rngcmp`: 6,601 frames, zero differing) and its window
/// covers it, so the spender was named from the disk rather than from
/// a capture: the Merchant `1/26`'s **crew figure**, standing on its
/// offset at (39471, 18769) and turning from -1153564672 to its
/// leader's -1605566464 in that one frame. See
/// [`LONG_WORD_EAST_INDIES`] for the mechanic.
///
/// **6582 for a session**, and that frame was one `make_stuff` draw
/// the original spends and this crate did not: `+0x63d`, the expiry
/// over the slot it just bought. The slot is the citizen at 5 and the
/// buy is gated on its `val`, which the original offers at **714** and
/// this crate at **0** — because `create_units`' `base` is
/// `LeaderData::pop × 1000 / city_num` and **nothing in this crate
/// ever wrote `pop`** (`research_techs`' `× 200` likewise). It is
/// `CityData::get_pop_value@00738450` summed over the leader's live
/// cities, 1/3/5 by level, and with it the whole tail reproduces the
/// original's 714 to the unit (`docs/AI.md` §27,
/// `cities_tests::a_leader_s_pop_is_one_three_five_by_city_level`).
///
/// **6612 for a session**, and that frame was a unit arriving: three
/// `Guy::init_real` and three `Unit::do_idle` idle rolls the original
/// spends and this crate spent none of. It was **no queue** — the AI's
/// Barracks `1/2016` finishes its construction on that exact frame
/// (`job_counter` 39600 of 42000 at run18b's last block, 100 a frame),
/// and `Build::activate`'s high-water block pays a **British** leader
/// its free archer on the first Barracks it ever holds. The trace says
/// so from the other side: the fifteen functions the original enters
/// for the *first time in 24,000 frames* on 6612 are `Army::add_unit`,
/// `Unit::think_attack`, `Unit::find_melee_target` and their
/// neighbours — the AI's first military unit.
///
/// One archer, three objects. `BRITISH_AGE_FOR_1_ARCHER` is 0 so the
/// ladder pays one at Ancient, and `Objects::init_unit` loops
/// `uber_size` times: a Bowmen is `UBER_SIZE 3, CREW_SIZE 0` and
/// `UnitTypeData::squad_size` is written **1** by `UnitType::init` and
/// never again, so the squad is three one-figure units on an
/// `o_up`/`o_down` list (`crate::nations`, `docs/CITIES.md` §4.3).
///
/// **6650 for a session**, and that frame was the squad's **first
/// order**: one `Unit::do_move+0xe84 < Unit::do_group_move+0x148 <
/// do_group_attack_to+0x11` the original spends and this crate spent
/// none of. Two mechanics stood behind it and both landed
/// (`docs/ARMY.md` §4.2, `docs/ORDERS.md` §8.3).
///
/// The first is *who gives the order*. `Unit::think_attack@005f5a80`'s
/// **head** joins an army before its target search runs, and that is
/// how the AI's first soldier gets into one; the squad lands in leader
/// 1's **army 1** — every army being empty, `find_local_army`'s
/// `90,000,000` and its `<=` hand the last valid slot the win — whose
/// tick is `frame ≡ 250 (mod 256)`, which is 6650. There
/// `Army::do_forming` issues `Group::action_siege_attack_to`.
///
/// The second is *what the order is*. A land formation of two or more
/// takes a **`GroupMoveOrder`**, and `Unit::do_group_move@005e79a0`
/// runs `do_move` for the **leader alone**: three archers marching
/// spend one grid draw between them where three independent moves
/// spend three. Two readings inside it moved the number further:
/// `Group::update_positions` rotates the slot table by the bearing to
/// the leader's **waypoint** and not by its heading (`7138e1`), and
/// §4.3's **group** arm of the soft-collision table — two members of
/// one group walking `GROUP_MOVE`/`GROUP_ATTACK_TO` pass through each
/// other — is no longer a seam now that this crate has a group to ask
/// about. Without the last one the squad stood blocked on its own
/// leader at 6716.
///
/// **6736 for a session**, and it was not a unit arriving at all — it
/// was three units **converting**. The stack names it whole:
/// `Guy::init_real+0x52 < Unit::set_type+0x40c < Leader::gain_tech
/// +0x1071`, three of them, and then three extra
/// `Guy::inc_time+0x271` because `init_real` leaves the clock at zero.
///
/// 6736 is the frame the AI reaches the **Classical Age**, and its
/// leader is British: `gain_tech` step 13's `BRITISH_ARCHER_UPGRADES`
/// block hands a British player every Barracks unit of the Bowmen line
/// whose prerequisites the gain completes — **Archers** — and step 7's
/// object half then converts the three standing Bowmen objects in
/// place. Neither half existed here: `free_rules` was **empty**
/// (the shape was implemented and the table was never loaded) and
/// `Gained::UnitUpgrade` had no consumer. `docs/TECH.md` §7, §13.
///
/// **6779 for a session**, and it was the AI's own sweep: two draws
/// inside `strategy_all` the original does not spend, on the machine's
/// **step 5** — `upgrade_units`, one roll per eligible type. The two
/// types were **Slingers** and **Javelineers**, and the reason they
/// were eligible is not in the AI at all.
///
/// `Leader::init` lays the starting position down **after** the
/// leader's nation is set, and its unit arm is `has_preq &&
/// tribe_can_type` (`docs/TECH.md`, "The starting position"), so which
/// unit types a player owns at frame 0 is a *function of the nation*.
/// This harness had the order the other way round: [`build_sim`] calls
/// `Loaded::sim`, which calls `Sim::start_techs` for every player, and
/// only then reads the `LEADER` records and calls `Sim::set_tribe`. So
/// every capture was built with its leaders' opening tech set computed
/// for **`tribe = 0`, the Aztecs**. run53's British AI started owning
/// `Atl-Atls` — the Aztec light-infantry variant, `TRIBE_MASK 0x1` —
/// and *not* `Slingers`, whose mask excludes the Aztecs; Slingers was
/// therefore RESEARCHABLE rather than AVAILABLE, and Javelineers
/// behind it, so `upgrade_units` offered both the moment the AI's
/// Barracks finished. The starting position is now re-laid once the
/// nations are known, and the two draws are gone.
///
/// Three unnamed sites were named in the same pass, because the AI's
/// draws all read as the coarse `strategy_all` mark and any one of
/// them parted the *sequence* a frame after the count: the matchup
/// bias in `create_units` and in `upgrade_units`
/// ([`sim::ai_units::SITE_UNIT_BIAS`], [`sim::ai_units::SITE_UPGRADE_BIAS`])
/// and `create_buildings`' wonder pair
/// ([`sim::ai_build::SITE_WONDER_MOD`], [`sim::ai_build::SITE_WONDER_SCALE`]).
///
/// **6782 until 2026-09-04**, and it was a building this crate bought
/// and the original did not: both sides spent `make_stuff`'s two
/// `+0x221` expiry draws over the head's type, and only this crate went
/// on to `produce_building`'s two `+0x1805` jitter draws and to the two
/// `+0x63d` expiries over the slot it had just bought. The head was the
/// same and the *buy* was not — because the head's own buy never
/// happened here. The head was two **Longbowmen**, and
/// `Leader::produce_unit`'s military-trainer arm searches
/// `mil_trainers` (`LeaderData+0x6e50`), a list **nothing in this crate
/// ever wrote**: it was `Vec::new()` at `Leader::new` and stayed empty
/// for the whole game, so the AI's Barracks was invisible and the two
/// Longbowmen were never queued or paid for. The original paid 62
/// timber and 102 metal for them, which is exactly what puts the
/// University in slot 1 out of reach on the next line of the same
/// function. `Wall::increment_stats@00643270` files an active military
/// trainer and `Wall::decrement_stats@00642da0` takes it out again
/// ([`sim::Sim::mil_trainer_open`], `docs/AI.md` §29).
///
/// It was **6848** for one item, and that item was the **march**: the
/// AI's Archer squad ran ahead of the original's because this crate
/// stepped 26 flat where the captain steps `26, 13, 26, 26, 13`. The
/// halving is `unit_masks & 0x100000`, the one-shot a **soft** collision
/// leaves behind — a squadmate in the way is squeezed past rather than
/// stopped for, and the price is the next step's half
/// (`docs/COLLISION.md` §4.3, §7, `docs/ORDERS.md` §15). The bit was
/// written and nothing read it; `move_step` reads it now.
///
/// It was **6862** for one item, and it was one missing
/// `Guy::set_anim+0x97a < Unit::move_step+0x823` — the blocked walker's
/// stand (`sim::anim::SITE_BLOCKED`, `docs/COLLISION.md` §5), where the
/// squad meets the standing citizen `1/13`. The stand was not the
/// mechanic: what the record showed was that on that frame the original
/// **ungroups the squad** and this crate did not. `move_step` answers 0
/// from exactly three places — blocked and still owing a turn, blocked
/// and handed to `resolve_unit_collision`, and a tile the world refused
/// — and `do_group_move` reads that 0 on both sides of the formation,
/// the leader's through `do_move` and the follower's straight off
/// `move_step`. Every one of them degrades the group move into N
/// independent moves (`docs/ORDERS.md` §8.3, `docs/COLLISION.md` §5.3).
/// This crate answered `Did::Something` from the two collision arms, so
/// the Archers stayed in formation for the rest of their march where the
/// original scatters them onto three world-grid paths of their own.
///
/// **6982**, and it is `Leader::produce_building+0x1805` where the
/// original is still in `Leader::make_stuff+0x221`: the AI buys a
/// building here that the original does not — the same shape as the
/// 6782 row above, and an economy question rather than a movement one.
///
/// **6994** was the second squad's own order, and it was neither the
/// marching Archers' path nor `come_out`: `Unit::add_to_army@005f7740`
/// walks a joiner to the army's first member, and this crate never
/// issued that walk (item 250, `docs/ARMY.md` §4.3).
///
/// **7176** for one item, and the unit going idle was a **builder**.
/// The AI's Tower `1/2017` finishes on that frame in the original —
/// `job_counter` 90800 → 0, `construct_hits` 749 → 750, `flags` 3 → 7 —
/// its citizen `1/20` drops the `BUILDORDER` and takes `idle 1`, and
/// `Unit::do_idle+0x7d` rolls the anim this crate did not. The clock is
/// why: `constr_time` reads **90909** there and 100000 here, the Tobacco
/// rare's ten per cent (`TOBACCO_BUILDING_SPEED`). The rare arrives on
/// the original's own frame — 6751 — and the **bake** was missing:
/// `Leader::gather@006ce280` raises `0xc000000` when the mask moves, the
/// unit-stats flag *and* the wall-stats one, and this crate raised only
/// the first (item 261, `docs/CITIES.md` §3.2).
///
/// The widening that found it refuted the briefed candidate first. The
/// field diff parts at **7163**, thirteen frames earlier and on all six
/// of the AI's soldiers: `UnitData::is_captain@0046ceb0` is `o_up < 0`
/// and this crate had it as "on the map and inside nothing", so an army
/// of two three-figure squads counted six captains where the original
/// counts two, `Army::release_mustering`'s `n < 5` let it march, and the
/// group attack-to its 256-frame tick issues on 7162 never went out
/// (`docs/ARMY.md` §3.3, §7;
/// `run79_s_window_is_every_unit_s_whole_record`).
///
/// ~~**7455**~~ — eight draws against seven, this crate spending a
/// `Guy::set_anim+0x97a < Unit::move_step+0x823` the original does not:
/// `1/36` inside `1/17`'s disc, 94 units adrift because the army's
/// 7418 tick put every follower of the formation on the wrong side of
/// its leader (item 267, `docs/GROUPS.md` §6.3's tail negation).
///
/// ~~**7584**~~ — forty-eight draws against forty-nine, the original
/// spending a `Guy::set_anim+0x97a < Guy::move+0x19f` at index 24 that
/// this crate did not: **a gaia bird's arrival stand**, and the whole
/// of it was `Unit::do_air_physics`'s own `set_new_location(x, y, 0,
/// 1)` — `param_3` zero, so the figure is told where to be and not put
/// there and lags its unit by a step. `Guy::move` then reads `des ==
/// pos` as *the bird did not move this frame*, which on a bird pinned
/// against the world's edge by `WorldData::restrict` is true. This
/// crate teleported the figure with the unit and skipped `Guy::move`
/// for a bird entirely (item 284, `docs/SYNC.md` §3.9, "The arrival
/// stand"). 7584 now agrees **49 for 49, entry for entry**, and `1/3`'s
/// spot on block 7585 — the value diff beside the word — closes with
/// it.
///
/// ~~**7585**~~ — the AI rather than the animals: seven draws against
/// nine, parting at index 2 where the original spends a
/// `Leader::make_stuff+0x63d` ([`sim::ai_make::SITE_EXPIRE_SLOT`],
/// `docs/AI.md` §2.6) this crate did not. Item 287 read that as the AI's
/// stockpile and run91 refuted it (§34); item 295 found the head of the
/// make list instead, and under the head **`census_units` reading the
/// wrong role word over the wrong objects** (§35). The sweep tested
/// `role & 0x10000` on [`sim::combat::Profile::roles`], whose `1 << 16`
/// is `CARAVAN`, and counted every figure of a squad as a unit of its
/// own — so leader 1's army read as one caravan, `land_army` came out 1
/// rather than 3, the land branch's `base × 100` arm ran, and the
/// product wrapped into `create_units`' `val < 0` guard. With the census
/// right the frame agrees nine draws for nine, the bird's stand at index
/// 4 included.
///
/// **7679** is where it stands: four draws against three, parting at
/// index 1 where this crate spends a `Guy::set_anim+0x97a <
/// Unit::move_step+0x823` against the original's `Guy::set_anim+0x97a <
/// Guy::inc_time+0x271`.
pub(crate) const LONG_WORD_GREAT_LAKES: i64 = 7679;

/// The frame Great Lakes' **second** squad joins the army on — 6994, the
/// word's own parting from 2026-09-06 to 2026-09-07 and now a landed
/// mechanic rather than a score. Two tests are written about the event
/// and not about the headline, so they name this rather than
/// [`LONG_WORD_GREAT_LAKES`], which has moved past it:
/// `run84_says_great_lakes_6994_belongs_to_the_second_squad` (the
/// original's record) and
/// `great_lakes_6994_issues_the_second_squad_s_walk_to_the_army` (this
/// crate's answer to it). `docs/ARMY.md` §4.3 and §16.7.
pub(crate) const GREAT_LAKES_SECOND_SQUAD: i64 = 6994;

/// The frame the AI's library takes its **Coinage** job on, and the
/// frame run58's `QUEUE` record used to part on: twenty-four rows of
/// `1/2005 queued ours 0 theirs 1` running to the end of the file.
/// Item 154's first half, and it was
/// `economy::Holdings::available` — knowledge is not available before
/// the Classical Age, so the original charges Coinage's `14k` as two
/// hundred and ten food and this crate asked for knowledge the AI had
/// no way to hold. Both sides now start the job here, and this
/// constant is what says so: the filter below still splits the file
/// at it, and the tail it leaves is empty but for the truncated last
/// frame.
pub(crate) const RUN58_QUEUE_TAIL: i64 = 5177;
