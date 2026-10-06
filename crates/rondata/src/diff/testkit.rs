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
///
/// **8466 → 8495 on item 338**, and this map's word was the *same event*
/// as Great Lakes' 8272: run54's 8466 is East Indies' own first scholar
/// seating, one of the fourteen `Unit::go_inside+0x280` draws in its
/// 24,000 frames. Neither map's item named the other, and one change
/// moved both — see [`LONG_WORD_GREAT_LAKES`] and `docs/CITIES.md`
/// §6.5.2.
///
/// **8495 → 8555 on item 346, and it was the same event again** — the
/// seated scholar's *first wrap*. `1/22` sits down on 8466 on slot 25,
/// `Scholar Teach1`, thirty frames long, and 8466 + 30 − 1 is 8495. The
/// original re-rolls that wrap instead of restarting the slot; this crate
/// restarted it, so the frame was one `Guy::set_anim+0x97a <
/// Guy::inc_time+0x271` short. `sim::anim::Sim::seated_scholar` and
/// `docs/ANIM.md` §5.1. One change moved both maps for the second item
/// running, and again neither map's brief named the other.
///
/// **9711 → 9983 on item 573, and the frame was a purchase 135 frames
/// under it.** 9711's two extra draws are the sixth seated Scholar's
/// birth; its purchase is sim-frame 9576, where the economic script buys a
/// Scholar at 130 wealth, and this crate's player 1 held 114. It had been
/// twenty short since the caravan's first homecoming around 6512:
/// `City::new_caravan@00739750` pays `(epoch[2] + 1) · 10` — `+0xf0` of
/// `LeaderDataEncrypt`, **Commerce** — and this crate read Civic. run82's
/// `LEADERS=9` window read `bucket[2]` 226 against 246 on 6860 and reads
/// 246 on both sides now (`run82_s_leader_record_is_the_original_s_wealth`,
/// `docs/CARAVAN.md` §9). The widening's block for the move is 9712
/// ([`EAST_INDIES_SCHOLAR_BLOCK`]); 9983 is a `make_stuff` that buys a
/// building here and units in the original, 212 draws against 15.
///
/// **9983 → 10232 on item 576, and the frame was the make list two frames
/// under it.** run139 (`LEADERS=9` over [9960, 9999]) printed both lists:
/// empty on 9979, agreeing through `upgrade_units`, and parting on block
/// 9982, `create_units`' frame, where the original offers three ships —
/// types 340, 334 and 323 at `val 9999999` — and this crate none. The
/// sea branch's dock is `find_building(city, SEARCH_FRIENDLY, who,
/// 0x1800, …, FILTER_BASE_TYPE, DOCK)`, a search around the city; this
/// crate looked only in the city's own chain, and East Indies' Dock
/// `1/2010` belongs to no city (`city -1`). With the search, both lists
/// agree slot for slot on every block of run139 (`MAKE[*].city` is §52.2's
/// index shift), and block 9984's 27 rows are gone
/// (`run99_s_word_frame_is_widened_whole`, the move's value diff on
/// [`EAST_INDIES_MAKE_BLOCK`]). 10232 spends 33 draws here against 34,
/// parting at index 30: ours `Guy::set_anim+0x97a < Guy::inc_time+0x271`,
/// theirs `Unit::do_move+0xe84`. `docs/AI.md` §57.
///
/// **10232 → 10398 on item 579, and the frame was a birth 45 frames
/// under it.** Trireme `1/32` is trained at Dock `1/2010` on 10186 on both
/// sides and came out on `come_out`'s ring due south here, due east there.
/// A dock marks every ocean tile within three of its footprint `BAD_PATH`
/// (`BuildType::mask_me`'s `is(DOCK)` arm), and a sea type with an attack
/// may neither be placed on (`find_nearby_spot`) nor path through
/// (`invalid_loc`, 3) a `0x2400` tile; with both, `1/32` is born on the
/// original's (45192, 41880). The word's own draw was the navy's:
/// `create_units`' sea branch seeds one (`Armies::init_navy`, run139's
/// 9981), `1/32` joins it on 10187, and the navy's first tick on 10232
/// orders it to the muster through the margin — the straight line is
/// refused, and the original pays `Unit::do_move+0xe84` and gives up.
/// run99's widening pins the move's value diff on the old word's block,
/// 10233 (empty), and the new word's on 10399, run99's last: the Bark
/// `1/34` has walked to the navy a step behind the original's since its
/// first step on 10325 and arrives on 10398, still moving here and idle
/// there; ours 4 draws against 5, parting at index 0, theirs
/// `Guy::set_anim+0x97a < Unit::do_idle+0x7d`. `docs/ORDERS.md` §25.
///
/// **10398 → 10582 on item 588, and the frame was the Bark's first step,
/// 74 frames under it.** `1/34` stepped 20 on sim-frame 10324 where the
/// original stepped 41, and stayed 21 units behind to the navy's muster.
/// The one-shot half step came from `do_move`'s waypoint probe, which
/// found Trireme `1/32` soft beside the muster; the original's
/// `detect_unit_collision` has a second arm for a ship that never scans
/// on a probe that does not ask for `boats`, and on `move_step`'s probe,
/// which does, pushes its way through with `detect_boat_collision` — a
/// group-mate is skipped, so the navy never blocks itself. With the sea
/// half of the arm, run99's and run143's widenings pin the move's value
/// diff on the old word's blocks, 10397..10399 (empty), and run143's the
/// new word's on 10581..10583: player 1's `make_stuff` on 10582 buys
/// differently, the make list parting on 10581 (a Citizen, type 50, at
/// `val 1714` in the original's slot 3 and 5, where this crate offers a
/// Scholar), under a peasant census that parts on 10576
/// (`free_peasants` 2 against 1). `docs/COLLISION.md` §13.
///
/// **10582 → 10782 on item 604, and the frame was the Mine's spiral on
/// 10582 itself.** Items 592 and 597 each fixed a value under it without
/// moving it; the spiral still scored eleven friendless candidates
/// against the original's seven, four of them (43, 44), (42, 44), (40, 46)
/// and (40, 47) reaching a solid mountain cell no range lists. The solid
/// lists are the templates': `MountainRange::init` builds them from the
/// alpha of each `<MOUNTAIN>`'s `TEMPLATE_TEX`, and the generator's
/// placements are printed by run38's `DUMP_ALL` head. With them laid down
/// the four are refused at 1536, as run144's packet measures, the spiral
/// spends seven draws against seven, and the Mine's shuffle reads the
/// original's stream. The move's value diff is run143's block 10583:
/// 80 rows (the Mine's gather list, 40 `tx` and 40 `ty`) → **0**, and the
/// widening's floor 393 → 313 (`run143_s_word_frame_is_widened_whole`,
/// which pins it). 10782 is past run143's last block: ours 10 draws
/// against 11, parting at index 5, ours `Animal::do_idle+0x83`, theirs
/// `Leader::make_stuff+0x63d`. `docs/AI.md` §60.
///
/// **10782 → 10982 on item 608, and the frame was a census six frames
/// under it.** run149 (`LEADERS=9` over [10730, 10879]) showed one stream
/// shifted: the original's `make_stuff` bought four Citizens where this
/// crate bought one, and expired two slots for them. `create_units` on
/// 10780 had offered the Citizen at `num 4` there and 1 here. The count is
/// the city's gatherer deficit bounded by its open slots, and on 10776 the
/// census had counted the Mine `1/2018`'s three in the original and not
/// here: `reg_gather_slots` of the home region 17 against 20. The Mine was
/// placed on 10583, and the 10775 sweep is the first after it.
/// `City::count_gather_slots` walks the city's chain and sums every gather
/// building's `gather_max`, finished or not. run150's packet at 10765
/// reads the unstarted Mine holding `gather_max 3` in the chain, and this
/// crate skipped a building that was not `active`. The move's value diff
/// is run149's widening, the word's blocks 10781..10783: 20 rows → 1, the
/// `city` shift. `reg_gather_slots` is gone from 10776, and the floor went
/// 280/283/506 → 280/282/291 (`run149_s_word_frame_is_widened_whole`). On
/// run149's last block, 10879, the standing rows went 427 → 280, and the
/// other 279 are the same rows value for value. 10982 is past run149's
/// last block: ours 14 draws against 695, parting at index 2. The original
/// places a gather building and shuffles its 170-tile list
/// (`Build::find_gather_tiles+0x10a` × 680), and it scores the spiral
/// `produce_building` 4 `+0xc99` and 4 `+0x1805` against this crate's 7
/// and 1. `docs/AI.md` §61.
///
/// **10982 → 11069 on item 613, and the frame was two mechanics under
/// it.** run152 (`LEADERS=9` over [10870, 11039]) showed the make list
/// parting on 10981: `create_units` offered the Light Horse at 9999999
/// here against 6945568, because `check_income` answered 0x40 against
/// 0x100 — this crate priced it at 60/40 against 53 food, where
/// `TypeData::get_cost` takes `HORSES_STABLE_COST`, 15%, off a Stable unit
/// when the player holds Horses (51/34, two affordable). With the price
/// in, the frame's spiral still scored seven friendless sites against
/// four: `calc_gather`'s survey refuses a range another Mine already
/// gathers from, and range 2 is `1/2018`'s. The move's value diff is
/// run152's widening: 13 make-list rows → 2 from 10981, 21 unit rows → 0
/// on 10983 (the Mine `1/2019` on the original's site with its 170
/// tiles), and the floor 278/409/616 → 278/409/412
/// (`run152_s_word_frame_is_widened_whole`). 11069 is past run152's last
/// block: ours 5 draws against 6, parting at index 0, ours
/// `Guy::set_anim+0x97a`, theirs `Unit::do_move+0xe84`. `docs/AI.md` §62.
///
/// **11069 → 11590 on item 620, and the frame was a tile pick 110 frames
/// under it.** run155 (`LEADERS=9` over [11030, 11279]) showed the
/// original's head draw on 11069, `Unit::do_move+0xe84 <
/// Unit::do_explore_to`, to be the citizen `1/11`'s grid roll. Its walk
/// was (15, −29) off the original's on every block of the window, so it
/// popped its waypoint a frame late and this crate spent the same roll on
/// 11070. The walk went back to 10959. There `1/11` joined the Mine
/// `1/2018` and took (173, 183), a tile `1/6` already held, where the
/// original took (170, 182). `Unit::do_non_flat_gather` caps a Mine's
/// `dist_mod` of 10 at 3 when its list is under `MTN_TINY_SIZE`, and this
/// list is 46 tiles (`docs/ORDERS.md` §6.4). The move's value diff:
/// run152's 72 list rows and 28 walk rows go from 10959, floor
/// 278/409/412 → 278/309/312 (`run152_s_word_frame_is_widened_whole`).
/// On run155 the word's three rows on 11069..11070 go, and `1/11` agrees
/// on every block, 369/371/897 → 289/291/307
/// (`run155_s_word_frame_is_widened_whole`). No row comes in on either.
/// 11590 is past run155's last block: ours 6 draws against 5, parting at
/// index 0, ours `Guy::set_anim+0x97a < Animal::do_idle+0x19`, theirs
/// `Guy::set_anim+0x97a < Guy::inc_time+0x271`.
///
/// **11590 → 11747 on item 629, and the frame was a Merchant's seat 3,900
/// frames under it.** run159 (`LEADERS=9` over [11270, 11899]) showed the
/// extra idle to be gaia's sheep `8/1` arriving from its 11576 wander:
/// straight here in 13 frames, round a waypoint at (28884, 23724) there
/// in 21. The tile search refused its diagonal through (149, 124), one of
/// the four tiles the AI Merchant `1/20` blocked when its unpack ended on
/// 7662. `SpellType::cast_unpack`'s merchant arm snaps the trader onto
/// its tile corner and blocks the two-by-two under it, and this crate
/// had it as a seam, so `1/20` stood (24, 24) off at the unit-cell
/// centre with nothing blocked (`docs/MERCHANT.md` §3.2). The move's
/// value diff (the word's delta, here; its block is the widening's):
/// run159's sheep rows on 11578..11591 go, and so do both Merchants'
/// seats (`1/19` and `1/20`, 22 rows), city `1/2007`'s `filled` and
/// `space[2]`, the leader's `reg_land[11]` and `leftover` food and
/// metal on 11270, and 613's food and metal a unit off from 11272 and
/// 11275; the floor goes 297/391/920 → 270/361/806, and every row that
/// comes in is past 11748 (`run159_s_word_frame_is_widened_whole`).
/// 11747 is inside run159's window: ours 10 draws against 7, parting at
/// index 3, ours `Guy::set_anim+0x97a < do_cast`, theirs
/// `Guy::set_anim+0x97a < Guy::inc_time+0x271`.
///
/// **11747 → 13640 on item 642, and the frame was the scout's island,
/// chosen 198 frames under it.** The extra `do_cast` was the AI scout
/// `1/0` boarding: a boat `1/44` was born with it inside, and the
/// original's scout was still walking. Its target parted on 11549. Out of
/// land, it asks `think_civilian_transport` for an unscouted island, and
/// both sides spend the same `think_scout+0x941`. The original sends it to
/// cell (19, 13), on the human's home island, because `Region::go_here`
/// answers bit 2 where an enemy holds a city. This crate read the human's
/// `reg_cities` from a census it never runs (`docs/AI.md` §23.1), saw 0,
/// and sent it to (25, 43). `go_here` now reads other leaders through
/// `Sim::leader_reg_cities`'s recount (`docs/TRANSPORT.md` §9.4). **The
/// move's value diff (the word's delta, here; its block is the
/// widening's):** run159's scout rows go, 47 on 11550 and 20 on
/// 11578..11579. So do the 16 rows new on 11747..11748 and the three
/// one-sided animations there. Both sides board `1/44` for block 11793.
/// The floor goes 267/358/803 → 267/291/296
/// (`run159_s_word_frame_is_widened_whole`). 13640 is past run159's last
/// block and past every East Indies dump on disk. There, ours spends 34
/// draws against 33, parting at index 33: ours has one more
/// `Guy::set_anim+0x97a < Guy::inc_time+0x271` after the frame's
/// thirty `Animal::think_bird` draws, and theirs has none.
///
/// **13640 → 15782 on item 643, and the frame was an animation's name,
/// cased differently in two files.** run166 over [13580, 13700) said whose
/// wrap it was: the Galley `1/32`, a Trireme upgraded inside the window,
/// entered `CHAR_DEFAULT` on 13637 with `end_time` 3 here and 20 there,
/// and wrapped again on 13640. `unit_graphics.xml` cites "Galley Default"
/// and `anim_graphics.xml` defines "Galley default". The original resolves
/// the name with `GraphicPieces::decipher_animation@008face0`, a first-match
/// `_wcsicmp`, and this crate's case-sensitive map read a missing slot —
/// 46 of the names the install cites resolve only case-folded
/// (`docs/ANIM.md` §12). **The move's value diff (the word's delta, here;
/// its block is the widening's):** run166's four `1/32` rows and the sheep
/// `8/1`'s go, and the 187 rows the extra draw parted past the word; the
/// floor goes 295/308/495 → 295/307/307
/// (`run166_s_word_frame_is_widened_whole`), nothing parting from the
/// word to the window's end. On the way the sequence parted on 15378, a
/// bare `6c7812` against ours' unnamed draw: the Senate arm's survivor
/// roll in `research_techs`, the same draw, which this crate now names
/// (`sim::ai_research::SITE_GOV_ROLL`). **15782** is past run166 and
/// inside run78's [15700, 15900] (`LEADERS=1`): the original spends three
/// `Guy::init_real+0x52` and a wrap before the bird, the birth of the
/// three-figure unit `1/60` run78 counted, and ours births nothing.
///
/// ~~**15782 is `1/60`'s birth.**~~ **15782 → 15985 on item 706, and the
/// birth was a government patriot.** run78 prints who=1's `gov` −1 → 624,
/// Republic, on block 15783: its Senate finished the research on tick
/// 15782, and `Build::finished` trains the government's patriot there
/// (`docs/TECH.md` §"The government patriot") — The Senator, `1/60`. This
/// crate gained the tech and trained nothing. The item was booked on
/// Great Lakes, whose Despot is the same arm; this map moved with it and
/// was not widened by it. **The new word's delta: ours 5 draws and the
/// original 6, parting at index 4**: ours spends `Unit::think_scout
/// +0x941` where the original spends `Leader::make_stuff+0x63d`. **Past
/// run78** (block 15986 against its last, 15900) and past every East
/// Indies dump on disk: no widening names its block yet.
///
/// ~~**15985 is past every capture.**~~ **15985 → 16683 on item 708, and
/// the frame was the commerce cap's republic term.** run221 over [15894,
/// 16237) said whose draw it was: the original's `make_stuff+0x63d` is its
/// slot loop buying a citizen (`t 50`, cat 5) that `create_units` offered
/// on 15983 and ours never offered. Its first parting is 15977's `rate`,
/// 93 on three goods here against 118, 110 and 100 — `min(cap, income) /
/// 16` — and the cap was who=1's `resource_cap`, 2992 here against 3792 on
/// every capped good: `calc_resource_caps@006ce900` adds
/// `REPUBLIC_COMMERCE_BONUS` (50) for the highest `REPUBLIC_n` held, and
/// who=1 took Republic on 15782 (`docs/AI.md` §72). **The move's value
/// diff (the word's delta, here; its block is the widening's):** the
/// fifteen leader rows under the word and the five caps go, and the 551
/// rows past it (`run221_s_word_frame_is_widened_whole`, floor
/// 285/288/851 → 280/280/290). **The new word's delta: ours 6 draws and
/// the original 7, parting at index 5**: ours spends `Farms::inc_time
/// +0x1ae` where the original spends `Guy::set_anim+0x97a <
/// Guy::inc_time+0x271`: six `Guy::inc_time` wraps there before the farm's
/// draw against ours' five (`1/18` twice, `1/19`, `1/20`, `1/54`), so one
/// figure's wrap is missing here, whose is the capture's to say. ~~**Past
/// run221** (block 16684 against its last, 16236): no widening names its
/// block yet.~~
///
/// **16683 → 16982 on item 752, and the frame was the census's per-city
/// gatherer count.** run227 over [16230, 16935) said whose wrap it was:
/// who=1's citizen `1/46`, idle since 16529 on a 123-frame `CHAR_IDLE`,
/// rolls on the original's 16683. Ours' had left on 16594, when
/// `find_gather_spot` sent it across to the woodcutter `1/2009` of who=1's
/// second city: the crossing rule reads each city's `free + gatherers`,
/// London 14 against 10 here and 12 against 12 there. `plan_strategy`
/// counts a gatherer in its **building's** city — the listing overwrites
/// the found city with the target's `+0x72` at `6babfb` — and this crate
/// counted it in the nearest, so two woodcutters standing nearer London
/// were London's (`docs/AI.md` §73). **The move's value diff (the word's
/// delta, here; its block is the widening's):** the two cities' counts on
/// the window's first block, `1/46`'s 24 rows, `1/2009`'s chain head and
/// `1/54`'s two on the word's block go, and the 554 rows past the word
/// (`run227_s_word_frame_is_widened_whole`, floor 286/333/866 →
/// 284/306/312). **The new word's delta: ours 10 draws and the original
/// 15, parting at index 4**: the original spends `Leader::make_stuff
/// +0x63d`, a bought slot's expiry roll, where ours spends `Animal::do_idle
/// +0x83`. Under it on 16779 who=1's make-list slot 1 reads `val` 22784
/// here against 91136. ~~**Past run227** (block 16983 against its last,
/// 16934): no widening names its block yet.~~
///
/// **16982 → 17189 on item 767, and the frame was the British price of
/// Taxation.** The ×4 on 16779 was one multiplier, `check_income`'s escrow
/// arm: `0x40` when the tech cannot be paid for, `0x100` when it can, on
/// the same 67,200,000 before it. who=1 held 106 food and 76 timber, and
/// this crate priced Taxation at 88 of each. `get_cost` takes
/// `BRITISH_TAXATION_DISCOUNT` (50) off `TAXATION` and the three after it
/// for the British (`docs/AI.md` §74), so the original's price was 44.
/// **The move's value diff (the word's delta, here; its block is the
/// widening's):** `MAKE[1].val` on 16779 goes, and nothing else on run227
/// moves (`run227_s_word_frame_is_widened_whole`, floor 284/306/312 →
/// 284/306/311). **The new word's delta: ours 2 draws and the original 1,
/// parting at index 0**: ours spends `Guy::set_anim+0x97a <
/// Unit::move_step+0x823`, a blocked stand, where the original spends `<
/// Guy::inc_time+0x271`; the original spends its own stand on 17190. A
/// scratch print named ours' as who=1's `1/55`, blocked by `1/60`.
/// ~~**Past run227** (block 17190 against its last, 16934): no widening
/// names its block yet.~~ run233 over [16929, 17440] widens it
/// (`run233_s_word_frame_is_widened_whole`).
///
/// **17189 → 17403 on item 773, and the frame was `no_danger`'s order
/// arm.** run233 said whose stand it was: `1/55`, half a step ahead from
/// 17182, because the column's `1/58` was not beside it. Tick 17146 gave
/// the column an `ATTACK_TO`, and ours' `1/58` planned its world path
/// north out of the column while the original's went round the west in
/// it. run248 printed the world on 17146, and it agreed; its proxies put
/// the first priced step to part as `1/48`'s, 60 here and 116 there, which
/// is who=1's danger / 8. `astar_path` sets `no_danger` for an
/// `ATTACK_TO` or `GROUP_ATTACK_TO` current order and for gaia, as well
/// as for an attack action, and this crate carried only the last
/// (`docs/PATHFINDER.md` §28). **The move's value diff (the word's delta,
/// here; its block is the widening's):** all 28,828 priced steps of tick
/// 17146 agree (`run248_s_world_at_17146_is_the_original_s`), and every
/// row of `1/55`, `1/57` and `1/58` goes: 22, 13 and 25, with `1/55`'s
/// stand on 17190 on both sides (`run233_s_word_frame_is_widened_whole`,
/// floor 293/363/1,139 → 293/312/1,036). **The new word's delta: ours 7
/// draws and the original 6, parting at index 0**: ours spends
/// `Unit::do_move+0xe84` where the original spends `Guy::set_anim+0x97a <
/// Guy::inc_time+0x271`. Under it on 17403, `1/60` walks under an
/// `ATTACK_TO` there and a `GROUP_ATTACK_TO` here, to another spot, and
/// `1/67`..`1/69` part on `group` from 17363. It is inside run233, and
/// the widening pins its block, 17404.
///
/// **17403 → 17501 on item 800, and the frame was the anchor's sort.**
/// On tick 17402 army 1's siege arm built its sub-group of the anchor
/// `1/60` alone, and ours sorted the **army's** list `[60, 69]` — the
/// squad's tail without its captain — re-seated the squad and laid the
/// whole army out as the sub-group, so `1/60` stood under a
/// `GROUP_ATTACK_TO` of four. `Group::sort` runs on the stack group
/// (`docs/GROUPS.md` §27). **The move's value diff (the word's delta,
/// here; its block, 17404, is run233's widening's):** on block 17403
/// `1/60` holds `ATTACK_TO` (2) to (38136, 42024) on both sides, against
/// ours' 21 to (37992, 41784) before; army 1's list is `[60, 69]` on both
/// sides, against `[60, 67, 68, 69]` here before; the window's floor goes
/// 293/312/324/1,036 → 293/312/324/446. **The new word's delta: ours 13
/// draws and the original 11, parting at index 3**: ours spends
/// `Guy::set_anim+0x97a < Guy::move+0x19f` — `1/57`'s walk step, by a
/// scratch print — where the original spends `< Unit::do_idle+0x7d`.
/// Past run233's last block (17440); run251 widens its block, 17502.
///
/// **17501 → 18182 on item 811, and the frame was army 0's retarget
/// tick, 17404.** `do_marching`'s retarget arm forms the army itself and
/// the dispatch forms it again (`docs/ARMY.md` §21), so the original moved
/// the group twice and the second move's `get_loc` planned the leader
/// `1/48` from (29833, 37525), cell (38, 48). **The move's value diff (the
/// word's delta, here; its block, 17405, is run233's widening's):** on
/// block 17405 `1/48`'s chain is 9 legs on both sides, its first leg
/// (29688, 38232) against ours' 39000 before, and `1/49`, `1/53`, `1/55`,
/// `1/57` and `1/58` agree on their slot waypoints and headings; the
/// order's `group.id` reads 17404001 against 17410801 — `order_num` now
/// agrees, the pool id is 689's. run251's eleven rows under 17501 are gone.
/// **The new word's delta: ours 9 draws and the original 11, parting at
/// index 0**: ours spends `Leader::make_stuff+0x221` where the original
/// spends `Leader::use_market+0x1ed`. Past run251's last block (17752);
/// run253 widens its block, 18183.
///
/// **18182 → 18938 on item 822, and the frame was the Civic epoch's
/// border fix on tick 18032.** A fix zeroes every leader's
/// `reg_known_rares` at the next `check_borders`, so the original's
/// recompute on 18071 summed 0 and its Merchant arm was shut on tick
/// 18180 (`docs/AI.md` §76). **The move's value diff (the word's delta,
/// here; its block, 18183, is run253's widening's):** who=1's
/// `known_rares` on block 18177 read 3 here against 0 there and now 0 on
/// both; on 18181 `MAKE[0]` read (61, cat 4, val 952,380) here against
/// (590, cat 8, val 22,784) there and `MAKE[4]` the Merchant against the
/// fresh slot, and both now agree in every field but `city`; run253's
/// floor goes 310/18/327/977 → 309/4/313/319. **The new word's delta:
/// ours 7 draws and the original 4, parting at index 1**: ours spends
/// `Unit::do_guard+0x8fb` where the original spends
/// `Guy::set_anim+0x104b`. Past run253's last block (18433); run257
/// widens its block, 18939.
///
/// **18938 → 18999 on item 829, and the frame was army 1's close on tick
/// 18682**, in the gap no dump compares. `Army::close@006f8ea0` sets the
/// group's `army` to −1 and keeps its record, index and list in the pool
/// (`docs/ARMY.md` §22). This crate dropped the group, so on 18692 the
/// wagon's new army took its slot and cleared the escort's pointers.
/// **The move's value diff (the word's delta, here; its block, 18939, and
/// run257's first block are `run257_s_word_frame_is_widened_whole`'s):**
/// on block 18933 `1/67`..`1/72`'s `group` read −1 here against 71 there
/// and now 72 against 71, the old group holding `[69, 72]` on both sides
/// (the pool id is 689's); on 18939 `1/67`, `1/68`, `1/70` and `1/71` held
/// `order:kind` 12 (`GUARD`, one order, stopped) here against 2 (the
/// guard's `ATTACK_TO` leg) there, and now agree in every field: the leg
/// no longer ends on a hard collision with `1/69`, a group-mate. **The new
/// word's delta: ours 7 draws and the original 6, parting at index 4**:
/// ours spends a fifth `Guy::set_anim+0x97a < Guy::inc_time+0x271` (`1/69`'s)
/// where the original spends `Guy::set_anim+0x104b`. Inside run257; its
/// block 19000 is widened there.
///
/// **18999 → 19182 on item 837, and the frame was the word's own.** Of
/// the five figures that wrap their idle on 18999 — `1/19`, `1/20`,
/// `1/29`, `1/67`, `1/69`, all five on both sides — `1/67` holds a
/// suspended search, and `Guy::set_anim`'s roll is `openlist == 0 ? rand
/// % 100 : CHAR_DEFAULT` (`5dac5f`): the original's wrap takes `DEFAULT`
/// with no draw (`docs/ANIM.md` §14). This crate kept the stash since
/// 2026-09-17 and its idle roll never read it. **The move's value diff
/// (the word's delta, here; its block, 19000, is
/// `run257_s_word_frame_is_widened_whole`'s):** on 18999 the draws went
/// 7 against 6 → 6 against 6, and on block 19000 `1/67` reads `start_dist`
/// 480, `collide` 31 and `collide_frame` 18969 on both sides, the one
/// suspended unit of the five, with guy 0 at `cur_anim` 0, `cur_time` 0,
/// `end_time` 31, `last_time` −1 on both; on 19002 `1/9`'s `cur_anim`
/// 36 → 8 was the original's alone and is now both sides'. **The new
/// word's delta: ours 14 draws and the original 5, parting at index 0**:
/// ours spends `Leader::use_market+0x1ed` (two), `produce_building+0x1805`
/// (three) and `make_stuff+0x221` (three) where the original spends its
/// first `Guy::set_anim+0x97a < Guy::inc_time+0x271`. Inside run257; its
/// block 19183 is widened there, six blocks from the capture's end.
///
/// **19182 → 19413 on item 839, and the frame was 19176's purchase.**
/// who=1, British, took Taxation on tick 17183, and `Leader::calc_gather`
/// scales `TERRITORY_TAXES[level]` by `(BRITISH_TAXATION + 100) / 100`
/// for the British: the original's wealth rate rose 992 → 1380 there,
/// ours 992 → 1234, one tax of 146 sixteenths short (305 of 1669 tiles
/// at 50%; `docs/ECONOMY.md` §16). The purse that followed bought the
/// original a hundred food in the gap 18685..18932 and bought ours a
/// hundred timber on tick 19176, for 135 of a purse of 150, where the
/// original's held 67. **The move's value diff (the word's delta, here;
/// its blocks are `run257_s_word_frame_is_widened_whole`'s):** on block
/// 19177 who=1's `bucket[1:timber]` went 149 against 49 → 49 on both and
/// `bucket[2:wealth]` 15 against 67 → 67 on both; on 19182 `MAKE[0].val`
/// 1531 against 0 → 0 on both and `MAKE[1].t` 427 against 528 → 528 on
/// both; the draws on 19182 went 14 against 5 → 5 against 5. **The new
/// word's delta: ours 5 draws and the original 6, parting at index 3**:
/// the original spends `Guy::set_anim+0x97a < Guy::do_turn+0x4a <
/// Guy::turn_towards+0x69` where ours spends its next `Guy::set_anim+0x97a
/// < Guy::inc_time+0x271`. Past run257's end; run269 was taken for it and
/// its block 19414 is widened there.
///
/// **19413 → 19509 on item 850, and the frame was 19413's cast.** `1/77`,
/// a Merchant walking to the shore, casts its barge `1/78` in its own
/// work on tick 19413 and is cargo by the work's end. `Unit::process@
/// 00610bc0` runs `Guy::process` on every figure after the work under the
/// entry's `inside_up < 0` test alone, so the original's figures take
/// their frame once more: the second, standing on its `des`, writes
/// `last_speed` 0 and turns to `des_angle` (`Guy::turn_towards`, the
/// draw). Ours gave that frame to an aircraft alone (`docs/ANIM.md` §15).
/// **The move's value diff (the word's delta, here; its block is
/// `run269_s_word_frame_is_widened_whole`'s):** on block 19414 `1/77`'s
/// `g.angle[1]` went −541917184 against −901447680 → −901447680 on both,
/// `g.avg_speed[0]` 15 against 11 → 11, `g.avg_speed[1]` 16 against 12 →
/// 12 and `g.last_speed[1]` 1 against 0 → 0; the draws on 19413 went 5
/// against 6 → 6 against 6. **The new word's delta: ours 10 draws and the
/// original 9, parting at index 3**: ours spends `1/71`'s
/// `Guy::set_anim+0x97a < Unit::move_step+0x823` where the original spends
/// its next `Guy::set_anim+0x97a < Guy::inc_time+0x271`. Inside run269;
/// its block 19510 is widened there.
///
/// **19509 → 20007 on item 857, and the frame was 19498's blocker probe.**
/// `1/71`, its search suspended behind `1/75`, probes `coll` on the fifth
/// frame after the collision. `Unit::do_move@005f7b30` calls
/// `detect_unit_collision(coll, quick 1, boats 1, 0, nocoll 0, top_only 1)`
/// at `005f7dab`: the quick form, so `1/75`'s north-east corner on the
/// cell counts and nothing is written. Ours asked the full form, whose
/// corner rule let the two pass and cleared `collide_o` (`docs/COLLISION.md`
/// §18). **The move's value diff (the word's delta, here; its blocks are
/// `run269_s_word_frame_is_widened_whole`'s):** on block 19499 `1/71`'s
/// `collide` went 0 against 11 → 11 on both, `collide_o` −1 against 75 →
/// 75 and `collide_who` −1 against 1 → 1; on 19500 its `pos` (36866,
/// 41713) against (36888, 41736) → (36888, 41736) on both; the draws on
/// 19509 went 10 against 9 → 9 against 9. **Then 19606, one draw against
/// the same draw:** the trace printed `Unit::do_guard+0x8fb`, `1/64`'s
/// `retry` roll, bare as `5e656b`; its guard's `retry` reads 8 on block
/// 19607 on both sides, and `trace::SITES` names it now. **The new word's
/// delta: ours 8 draws and the original 7, parting at index 0**: ours
/// spends `1/67`'s `Guy::set_anim+0x97a < Unit::move_step+0x823`, a blocked
/// step's idle, where the original spends its first `Guy::set_anim+0x97a <
/// Guy::inc_time+0x271`. Past run269's end (19664); run277 was taken for
/// it.
///
/// **20007 → 20782 on item 880, and the frame was tick 20000's push.**
/// `Group::kill` clears a pool record its last member leaves, and an
/// army's group is its slot's record (`Army::add_unit`'s `push_group`),
/// so the slot records' history is the original's and 865's kept fields
/// (`copy_group` leaves `facing` and `order_num`) land (`docs/GROUPS.md`
/// §30). **The move's value diff (the word's delta, here; its blocks are
/// `run277_s_word_frame_is_widened_whole`'s):** on block 20002 slot 69
/// reads `order_num` 6 and `facing` 1 on both sides (ours was 1 and 0),
/// and `1/64`..`1/66`'s orders' `facing` 0 on both (ours was 1); the draws
/// on 20007 went 8 against 7 → 7 against 7. **The new word's delta: ours
/// 8 draws and the original 1, parting at index 0**: ours spends
/// `Leader::use_market+0x1ed` where the original spends
/// `Farms::inc_time+0x1ae`. Past run277's end (20258); run289 was taken for
/// it, and `run289_s_word_frame_is_widened_whole` widens its block 20783.
///
/// **20782 → 23182 on item 890, and the frame was 20781's wonder price.**
/// `get_cost`'s wonder arm counts every wonder the leader holds or has a
/// site of, not the type's own (`docs/COSTS.md`, "A wonder is ramped by
/// every wonder"; `docs/AI.md` §77), so who=1's Pyramids site `1/2029`
/// prices the Mausoleum and the Colossus at 260 wealth against a purse of
/// 208 and `check_income` answers 0. **The move's value diff (the word's
/// delta, here; its block is `run289_s_word_frame_is_widened_whole`'s):**
/// on block 20782 who=1's `MAKE[0]`, `[1]` and `[8]` `val` went 486, 398
/// and 486 against 0 → 0 on both sides, the Mausoleum's price `[0, 200,
/// 200]` → `[0, 260, 260]` and the Colossus's `[200, 0, 200]` → `[260, 0,
/// 260]` (the original prints no price; its `check_income` of 0 says it is
/// over 208); the draws on 20782 went 8 against 1 → 1 against 1. **The new
/// word's delta: ours 49 draws and the original 48, parting at index 46**:
/// ours spends `Leader::produce_building+0x1805`, a placement's jitter,
/// where the original spends `Leader::make_stuff+0x63d`. Past run289's end
/// (21045) and below run96's start (23960); run299 was taken for it.
///
/// **23182 → 23420 on item 904, and the frame was a rock cell.** Slot 4's
/// Farm jitter at corner (184, 196) tried (35904, 38208), whose tile
/// (188, 200) is on cell (47, 50), flags `ROCK | OIL`; `blocked_tcoord`
/// refuses a non-oil type a rock cell (`006370b2`) and this crate did not
/// (`docs/AI.md` §78). **The move's value diff (the word's delta, here;
/// its block is `run299_s_word_frame_is_widened_whole`'s):** on block 23183
/// who=1's `MAKE[4].t` went 417 against −1 → −1 on both sides, `1/79`'s
/// `g.cur_anim[0]` 3 against 1 and `g.end_time[0]` 42 against 58 → 1 and
/// 58 on both; the draws on 23182 went 49 against 48 → 48 against 48.
/// **The new word's delta: ours 7 draws and the original 31, parting at
/// index 0**: ours spends `Unit::do_guard+0x8fb` where the original spends
/// `Farms::add_animals+0x92`, a pasture's five animals. Inside run299's
/// window (block 23421), so the same test widens it.
///
/// ~~**23420 is a pasture's five animals.**~~ **Item 919 moved it 23420 →
/// 24000, the capture's own end, and the frame was 23182's farm type.**
/// `get_nearest_farm_type` reaches `find_any_building`, which keeps a
/// building only when `WallData::is_started` answers (`flags & 2`), so
/// `1/2032`, placed on 23182 beside the bare site `1/2031`, finds no
/// neighbour and is Norwich's `others == crops == 3` pasture; this crate
/// found `1/2031` and copied its crop (`docs/AI.md` §79). **The move's
/// value diff (the word's delta, here; its block is
/// `run299_s_word_frame_is_widened_whole`'s):** on block 23421 `1/78`'s
/// `guard.retry` went 8 against 6 → 6 on both, `1/80`'s `g.cur_anim[0]` 1
/// against 0 and `g.end_time[0]` 58 against 31 → 0 and 31 on both; the
/// draws on 23420 went 7 against 31 → 31 against 31, and the five coins
/// read 63335, 19739, 17423, 20547 and 9655 on both sides, five pigs.
/// **The new word's delta: none** — ours and the original spend the same
/// draws on every frame of run54's trace, 0..23999, and the word is the
/// trace's last frame. At the end block, 24001, every one of the 89 units
/// the endpoint compares stands where the original's does (`ENDPOINTS`:
/// 36 off → 0). `run96_s_word_frame_is_widened_whole` widens run96, the
/// last blocks any dump holds (the widening test), and
/// `run299_s_word_frame_is_widened_whole` keeps the move's value diff.
pub(crate) const LONG_WORD_EAST_INDIES: i64 = 24_000;

/// **The third map's long word** (DECISIONS 54 §3, item 1066): run383,
/// Great Sahara's draw stream to 24,000 at `cover=0`, walked from its own
/// start with run381's `DUMP_ALL` head (`diff::third`).
///
/// **8 → 12783 on item 1133, and the mechanism was the setup's birth**
/// (`docs/COLLISION.md` §20, `docs/AI.md` §88): a starting citizen is born
/// at its building's point and `Unit::come_out` clears the birth disc behind
/// it, so the AI camp `1/2001`'s second citizen `1/2` left a hole in `1/1`'s
/// block. **The word's delta**: on frame 8 ours 6 draws against 7, at index
/// 0 theirs `Guy::set_anim+0x97a < Unit::move_step+0x823` and ours
/// `Farms::inc_time+0x1ae` → 7 against 7. The value diff is run382's block
/// 6 (`run382_s_word_frame_is_widened_whole`): `1/2`'s `collide` 1 against
/// 0, `dest` (38232, 16056) against (38328, 15768) and `path.length` 1
/// against 2 → the original's on both. run382 now walks all 1,850 frames
/// (6/5 → 1850/1850). **The new word's delta**: frame 12783, ours 12 draws
/// against 11, at index 9 ours `Leader::make_stuff+0x63d` and theirs
/// `Guy::set_anim+0x97a < Guy::inc_time+0x271`.
///
/// **12783 → 13182 on item 1147, and the mechanism was the AI's Peacocks**
/// (`docs/AI.md` §91, `docs/ECONOMY.md` "What an owned rare does"): the
/// rare mask's change recomputes the pop cap, and the cap takes a tenth for
/// Peacocks, which the AI's merchant `1/18` holds. **The word's delta**: on
/// frame 12783 ours 12 draws against 11, eight `Leader::make_stuff+0x63d`
/// against seven → 11 against 11. The value diff is run416's block 12780
/// (`run416_s_word_frame_is_widened_whole`): who=1's `pop_cap` 50 against
/// 55, so the Mercenaries offer's `cap × 5 / 6 < effective_pop` read 41 <
/// 44 and took `×20`, and `MAKE[1].val` 9999999 against 1632000 → the
/// original's on both. **The new word's delta**: frame 13182, ours 9 draws
/// against 8, at index 3 ours `Leader::make_stuff+0x63d` and theirs
/// `GameAccess::rnd+0x20 < Unit::do_job+0x67`, widened on run417.
///
/// **13182 → 14587 on item 1163, and the mechanism was Wine's research
/// discount** (`docs/AI.md` §94, `docs/COSTS.md` "Researching an upgrade"):
/// `get_cost`'s research arm takes `WINE_UNIT_UPGRADES` off, and the AI
/// holds Wine. **The word's delta**: on frame 13182 ours 9 draws against
/// 8, one `Leader::make_stuff+0x63d` more → 8 against 8. The value diff is
/// run416's block 12784 (`run416_s_word_frame_is_widened_whole`): `1/2014`'s
/// Militia research `queue[0].cost[0]`/`[1]` 80/80 against 64/64 → 64/64,
/// and the food bucket 16 short with it, which had priced the Slingers out
/// of 13180's make list. **The new word's delta**: frame 14587, ours 9 draws
/// against 10, at index 3 ours `Guy::set_anim+0x97a < Guy::inc_time+0x271`
/// and theirs `Guy::set_anim+0x97a < Unit::move_step+0x823`, widened on
/// run418.
///
/// **14587 → 15586 on item 1171, and the mechanism was `Group::kill`'s
/// array shift** (`docs/AI.md` §97, `docs/GROUPS.md` §30.3): `push_group`'s
/// walk took the leaving squads out of an army group's list and left its
/// `off`/`curr`/`angles` behind, so a member that stayed read another's
/// slot. **The word's delta**: on frame 14587 ours 9 draws against 10, one
/// `Guy::set_anim+0x97a < Unit::move_step+0x823` fewer (`1/52`'s stop on
/// `1/30`) → 10 against 10. The value diff is run418's block 14587
/// (`run418_s_word_frame_is_widened_whole`): group 64's `curr[4]`
/// (−173, −16) against (68, 126) → (68, 126), and `1/52`'s `pos`
/// (28637, 20233) against (28623, 20241) → the original's. **The new
/// word's delta**: frame 15586, ours 7 draws against 6, at index 3 ours
/// `Guy::set_anim+0xf2f < Guy::move+0x166` and theirs `Guy::set_anim+0x97a
/// < Guy::inc_time+0x271`, widened on run426.
///
/// **15586 → 15982 on item 1177, and the mechanism was the building arm's
/// `find_collision`** (`docs/COMBAT.md` §65.8): `do_move` ends a ranged
/// chase on a building only when no other unit stands in the chaser's
/// block, and ours never asked. **The word's delta**: on frame 15586 ours
/// 7 draws against 6, one `Guy::set_anim+0xf2f < Guy::move+0x166` more
/// (`1/29`'s arrival a frame early) → 6 against 6. The value diff is
/// run426's block 15585 (`run426_s_word_frame_is_widened_whole`): `1/29`'s
/// `order:kind` 10 against 1 → 1, and its `pos` (7292,27822) against
/// (7269,27840) → the original's. **The new word's delta**: frame 15982,
/// ours 17 draws against 16, at index 0 ours `Leader::use_market+0x1ed`
/// and theirs `Leader::produce_building+0x1805`, widened on run428.
///
/// **15982 → 16681 on item 1189, and the mechanism was the Spice route**
/// (`docs/CARAVAN.md` §4): `Caravan::trade_value` scales a route by
/// `SPICE_CARAVAN_INCOME` when the computing city's owner holds Spice, and
/// ours never read the rare. who=1's route read 22 against 26 from frame
/// 14660, its wealth fell behind, and on 15982 `use_market`'s need for two
/// Senates found 96 wealth against 104 and sold. **The word's delta**: on
/// frame 15982 ours 17 draws against 16, one `Leader::use_market+0x1ed`
/// more → 16 against 16. The value diff is run418's block 14661
/// (`run418_s_word_frame_is_widened_whole`): `1/2000` and `1/2007`'s
/// `trade_val` 176 against 208 → 208; and on run428's block 15977 who=1's
/// `bucket[2:wealth]` 96 against 103 → 103. **The new word's delta**:
/// frame 16681, ours 11 draws against 8, at index 4 ours `Guy::set_anim+
/// 0xf2f < Guy::inc_time+0x271` and theirs `Guy::set_anim+0x97a <
/// Guy::inc_time+0x271`, widened on run442.
///
/// **16681 → 17623 on item 1194, and the mechanism was the Javelineers'
/// release node** (`docs/COMBAT.md` §70.2): `launch::BAYS` had no row for
/// piece 33, so every javelin left the unit's own square and flew one
/// frame long, and on 16680 the human's Farm `0/2004` outlived the round
/// that killed it in the original. **The word's delta**: on frame 16681
/// ours 11 draws against 8, `1/30`'s `Guy::set_anim+0xf2f` and its round's
/// `Ammo::init+0xcd9`/`+0xd0b` more → 8 against 8. The value diff is
/// run442's block 16681 (`run442_s_word_frame_is_widened_whole`): the
/// Farm's `build:extra` "this crate holds it alone" → gone on both sides,
/// and on 16682 `1/28`'s `order:kind` 10 against 19 → 19; and run428's
/// block 15978, the Farm's `damage` 130 against 134 → 134. **The new
/// word's delta**: frame 17623, ours 7 draws against 8, at index 2 ours
/// `Guy::set_anim+0x97a < Guy::inc_time+0x271` and theirs
/// `Guy::set_anim+0x97a < Unit::do_idle+0x7d`, widened on run449.
///
/// **17623 → 24000 on item 1206, the trace's own end, and the mechanism
/// was `refresh_group_order`'s tail** (`docs/GROUPS.md` §6.8): the member
/// that takes a formation over re-turns the slot table through
/// `update_positions` whole (`713b94`), waypoint arm included, and ours
/// turned it by the member's heading. On 17493 `1/51` left army group 64
/// and `1/62` took it over standing, 2.5° between its heading and its
/// waypoint's bearing, and who=1's soldiers walked to slots that far off
/// for 130 frames. **The word's delta**: on frame 17623 ours 7 draws
/// against 8, one `Guy::set_anim+0x97a < Unit::do_idle+0x7d` fewer → 8
/// against 8. The value diff is run457's block 17493
/// (`run457_s_gap_is_widened_whole`): group 64's `curr[8]` (627, −793)
/// against (661, −765) → the original's, and all fifteen slots with it;
/// and run449's block 17618, `1/52`'s `pos` (28322, 20471) against
/// (28344, 20484) → the original's, 1,352 keys → 157, every one on the
/// first block. **The new word's delta: none** — ours and the original
/// spend the same draws on every frame of run383's trace, 0..23999, and
/// the word is the trace's last frame. At the end block, 24001, every one
/// of the 88 units the endpoint compares stands where the original's does
/// (`ENDPOINTS`' Great Sahara row: 0 off, 0 unlinked, 0 extra).
/// `run458_s_word_frame_is_widened_whole` widens run458, the game's last
/// blocks.
pub(crate) const LONG_WORD_GREAT_SAHARA: i64 = 24_000;

/// **The second pair's East Indies word** (DECISIONS 53 §2, item 971):
/// run346, run54's game with the lobby at Toughest, walked from its own
/// start dump by `diff::second`.
///
/// **0 on the capture**: the original spent 182 draws on frame 0 against
/// this crate's 175, parting at index 24 — `Unit::think_spellcaster+0x413
/// < Unit::think_scout+0x7c`, the special arm's coin, which a computer's
/// caster scout throws from difficulty 2 up and no Easiest capture ever
/// reached (`docs/AI.md` §80.5).
///
/// **0 → 10 on item 971**, the coin built (`sim::spellcaster`). **The
/// move's value diff (the word's delta, here; its block is
/// `run349_s_word_frame_is_widened_whole`'s):** on block 1 the AI scout
/// `1/0`'s `orders_x`/`orders_y` went 41976/36600 against 35832/42744 →
/// 35832/42744 on both, its path 9 slots against 3 → 3, and all 28 of its
/// rows on blocks 1..3 closed (the next parts on block 97); frame 0's draws
/// went 175 against 182 → 182 against 182. **The new word's delta: ours
/// 197 draws and the original 188 on frame 10, parting at index 183**:
/// ours spends `PathFinder::calc_road_cost+0x46` where the original spends
/// `Farms::inc_time+0x1ae`. Inside run349's window (block 11), where no
/// row first parts on blocks 4..25.
///
/// **10 → 1576 on item 979**, a harness fix (`diff::setup::same_lobby`,
/// `docs/AI.md` §81): the walk had installed run38's frame words — an
/// Easiest game's — over this Toughest one. **The move's value diff (the
/// word's delta, here; its block is `run349_s_word_frame_is_widened_whole`'s):**
/// on run349 the keys parted went 576 → 72; block 1's 46 gaia `cur_anim`
/// rows and `1/1`'s `g.end_time[0]` 232 against 33 closed (they were
/// run38's clocks), and the AI scout `1/0`'s `orders_x` 38136 against
/// 41976 on block 97 closed with no scout row parting on blocks 1..250.
/// **The new word's delta: ours 272 draws and the original 216 on frame
/// 1576, parting at index 192**: ours spends
/// `Build::find_gather_tiles+0x10a` where the original spends
/// `Animal::think_bird+0x82`. **The lower word of the pair again.**
///
/// **1576 → 5606 on item 989** (`docs/AI.md` §82, `docs/ECONOMY.md` §17):
/// `Build::close` gives a closed camp's ground back. **The move's value
/// diff (the word's delta, here; its block is
/// `run352_s_word_frame_is_widened_whole`'s):** `place_woodcutter` places
/// a camp at (38016, 36480) on frame 976 and destroys it the same frame;
/// its 48 tiles stayed marked here, so on 1576 the re-placed camp `1/2009`
/// went to (31680, 34752) with 62 tiles — 248 `find_gather_tiles+0x10a`
/// draws against 192 — and now lands on (38016, 36480) with the original's
/// 48; frame 1576's draws went 272 against 216 → 216 against 216. On
/// run352 the keys parted went 573 → 90: block 1577's 115 (the camp's
/// tiles, damage 1 against 0 and city −1 against 1, which were this
/// crate's dead frame-976 site linked by number, and citizen `1/2`'s
/// `orders_x` 31944 against 37752) all closed, and block 1571's capital
/// `2000` `ter[1]` 0 against 2 with them. **The new word's delta: ours 4
/// draws and the original 5 on frame 5606, parting at index 0**: ours
/// spends `Guy::set_anim+0x97a < Guy::inc_time+0x271` where the original
/// spends `Unit::do_move+0xe84`. **Now the higher word of the pair**
/// (Great Lakes' is 4555); widened on run357
/// (`run357_s_word_frame_is_widened_whole`).
///
/// **5606 → 5773 on item 1106** (`docs/AI.md` §84): a border fix reaches
/// the map at the daemon's 256 cells a frame (`sim::border_pass`), and a
/// city counts in its region's `reg_cities` on the frame `City::init` makes
/// it. **The move's value diff (the word's delta, here; its block is
/// `run357_s_word_frame_is_widened_whole`'s):** the AI citizen `1/14`
/// finishes the city `1/2017` on tick 5517 on both sides; ours had seated
/// it in pool group 67 on 5518 with its colonist move to (29568, 23424),
/// where the original's group is stamped 5521 with (30336, 24960) — run407's
/// packet reads its cell (48, 29) unowned and every region's index 0 on
/// logger frame 5518, and the pass reaches the cell on 5521. On run357's
/// block 5601 the keys went 172 → 139 and none arrived: `1/14`'s 27 rows
/// (`pos` (36525, 23051) against (36908, 23630), `order:move.y` 23304
/// against 23352), group 67's four (`ox` 29568 against 30336, `stamp` 5518
/// against 5521), who=0's `gather_stamp` 5520 against 5528 (a region's
/// completion raises the economy flag) and `1/2017`'s `city:peasant_dist`
/// 2 against 1. Blocks 5605 and 5607 part on nothing: `1/14`'s
/// `order:move.dest` and `path[2].to` rows are gone. Frame 5606's draws went 4 against 5 →
/// agreeing. **The new word's delta: ours 4 draws and the original 3,209
/// on frame 5773, parting at index 0**: ours spends `Guy::set_anim+0x97a <
/// Guy::inc_time+0x271` where the original spends
/// `PathFinder::calc_road_cost+0x46`. Inside run357's window (block 5774,
/// 173 blocks after its first and 83 before its last), where the Caravan
/// `1/33`, trained on 5772, holds a route of two orders there and none
/// here.
///
/// **5773 → 5975 on item 1115** (`docs/AI.md` §85, `docs/CARAVAN.md` §11):
/// `do_trade`'s region tests forgive a difference when the caravan
/// `can_transport` (5773 → 5776), and `calc_road_cost` charges the sea only
/// on the step off land (5776 → 5975). **The move's value diff (the word's
/// delta, here; its block is `run357_s_word_frame_is_widened_whole`'s):**
/// on run357's block 5774 `1/33`'s `orders.len` 0 against 2 closed with its
/// route, `1/2017`'s and `1/2000`'s `city:vans.length`; its `order:flags` 4
/// against 0 (5773) and `order:move.angle`/`dest_angle` 1073741824 against
/// 546111488 (5774) closed with the trade order's bit and the bearing; the
/// window's keys went 472 → 167. run413's packet at logger 5776 holds the
/// original's parked search equal to ours, 1,437 nodes against 1,437. Frame
/// 5773's draws went 4 against 3,209 → agreeing, and the search's sixteen
/// frames to 5788 agree. **The new word's delta: ours 63 draws and the
/// original 61 on frame 5975, parting at index 49**: ours spends
/// `Unit::think_scout+0xaba` where the original spends
/// `Guy::init_real+0x52`. The AI sea scout `1/35`'s region scan accepts 46
/// cells here against 45 there; past run357's window, widened on run414
/// (`run414_s_word_frame_is_widened_whole`, block 5976).
///
/// **5975 → 6151 on item 1120** (`docs/VISION.md` §11, `docs/AI.md` §86):
/// `cast_transport`'s `set_new_location(boat, x, y, 1, 1)` crosses a
/// half-cell onto the water and throws the barge's whole disc at the spot.
/// **The move's value diff (the word's delta, here; its block is
/// `run414_s_word_frame_is_widened_whole`'s):** no dump prints fog, so the
/// value is run415's packet at logger 5975 — `seen2` on half-cells (90, 80)
/// and (91, 81) ours 0 against 2 and (45, 73) ours 0 against 2 → all
/// 14,400 half-cells agreeing after tick 5974, and run413's at 5776
/// agreeing (`run413_s_and_run415_s_fog_grids_are_ours`). The sea scout
/// `1/35`'s scan accepts 45 cells to 45, and block 5976's newborn `1/40`'s
/// three `g.cur_time` rows (ours 1 against 0) closed; run414's keys went
/// 675 → 327. Frame 5975's draws went 63 against 61 → agreeing. **The new
/// word's delta: ours 9 draws and the original 8 on frame 6151, parting at
/// index 0**: ours spends `Guy::set_anim+0x97a < Unit::move_step+0x823`
/// where the original spends `Guy::set_anim+0x97a <
/// Unit::do_non_flat_gather+0x10f`. Inside run414 (block 6152, 182 blocks
/// after its first and 74 before its last), where the Caravan `1/15`
/// stands against `1/33` here and walks on there.
///
/// **6151 → 6321 on item 1127** (`docs/COLLISION.md` §19, `docs/AI.md`
/// §87): `detect_unit_collision`'s first soft row — both actions
/// `TRADE_ROUTE`, both fronts a move — lets one caravan step through
/// another. **The move's value diff (the word's delta, here; its block is
/// `run414_s_word_frame_is_widened_whole`'s):** on block 6152 the Caravan
/// `1/15`'s `pos` ours (35597,37251) against (35617,37267), `collide_o` 33
/// against -1, `half_step` 0 against 1 and its three figures' `stopped` 1
/// against 0 → all 35 of its rows agreeing; run414's keys went 327 → 176,
/// and none parts earlier than before. Frame 6151's draws went 9 against 8
/// → agreeing. **The new word's delta: ours 9 draws and the original 8 on
/// frame 6321, parting at index 2**: ours spends `Guy::set_anim+0x97a <
/// Unit::do_idle+0x7d` where the original spends `Guy::set_anim+0x97a <
/// Guy::inc_time+0x271`. Past run414's window, widened on run419
/// (`run419_s_word_frame_is_widened_whole`, block 6322).
///
/// **6321 → 6609 on item 1143** (`docs/ORDERS.md` §4.4, `docs/AI.md` §90):
/// the waypoint take's region check — a final waypoint in another tile
/// region than the unit's is re-pushed with `flags | 4` — lets the barge
/// `1/42` keep its goal ashore rather than `find_path` pulling it back to
/// the water. **The move's value diff (the word's delta, here; its block is
/// `run419_s_word_frame_is_widened_whole`'s):** on run419's first block
/// 6316 the barge's `pos` ours (38028,11677) against (38016,11689) →
/// agreeing, its leg (38028,11542) `flags 1` against (38016,11136) `flags 5`
/// → agreeing; on 6321 its `orders.len` 0 against 1 → agreeing; on 6322
/// `1/32`'s `inside` 42 against -1 and `pos` (37976,15372) against
/// (38040,11400) → agreeing, and the barge closed on both sides. run419's
/// first block went 169 → 160 keys, the window's 992 → 317. Frame 6321's
/// draws went 9 against 8 → agreeing. **The new word's delta: ours 9 draws
/// and the original 7 on frame 6609, parting at index 2**: ours spends
/// `Animal::do_idle+0x83` where the original spends `Guy::set_anim+0x97a <
/// Unit::move_step+0x823`. Past run419's window, widened on run420
/// (`run420_s_word_frame_is_widened_whole`, block 6610).
///
/// **6609 → 6743 on item 1156** (`docs/SCOUT.md` §8.1, `docs/AI.md` §93):
/// `find_unit_ordered` measures a sibling's `orders_x`/`orders_y`, not its
/// body (`0065be35`), so the citizen `1/28`'s region scan on 6576 counts
/// `1/22` by its walk's end (27384,25080), rejects cell (37,32) and takes
/// (34,29). **The
/// move's value diff (the word's delta, here; its block is
/// `run420_s_word_frame_is_widened_whole`'s):** on run420's first block
/// 6604 `1/28`'s `pos` ours (29017,24361) against (28893,24233) → agreeing,
/// its move's `angle` -969342976 against -1046609920 → agreeing, and group
/// 65's `o_angle` with it; on 6610 its `collide_o` -1 against 3 → agreeing,
/// and on 6611 the animal `8/3`'s `pos` (28661,24120) against (28680,24120)
/// → agreeing. run420's first block went 263 → 249 keys, the window's
/// 1444 → 963. Frame 6609's draws went 9 against 7 → agreeing. **The new
/// word's delta: ours 4 draws and the original 8 on frame 6743, parting at
/// index 2**: ours spends `Guy::set_anim+0x104b` where the original spends
/// `Guy::set_anim+0x97a < Guy::inc_time+0x271`. Inside run420's window
/// (block 6744, 140 blocks after its first and 116 before its last).
///
/// **6743 → 7382 on item 1164** (`docs/TRANSPORT.md` §6.4, `docs/AI.md`
/// §95): a passenger comes ashore at the `avg_speed` it boarded with —
/// `Guy +0x84` has no writer in either `set_new_location` — and a computer
/// player's takes `come_out`'s tail `update_action` from the spot
/// (`618813`..`618836`). **The move's value diff (the word's delta, here;
/// its block is `run420_s_word_frame_is_widened_whole`'s):** on run420's
/// block 6735 the AI merchant `1/33`'s `g.avg_speed[0..2]` ours 0 against
/// 12 → agreeing, and its `orders_x/y` (37439,33407) against (38952,24840)
/// → agreeing; on 6736 its guys' `angle` -120852736 against 93895616 →
/// agreeing; on 6743 its `pos` (38944,24864) against (38952,24840) →
/// agreeing, the original still turning to face its path. The window's
/// keys went 963 → 303. Frame 6743's draws went 4 against 8 → agreeing.
/// **The new word's delta: ours 15 draws and the original 9 on frame 7382,
/// parting at index 0**: ours spends `Leader::use_market+0x1ed` where the
/// original spends `Leader::make_stuff+0x221`. Past run420's window (its
/// last block 6860), widened on run425 (block 7383).
///
/// **7382 → 7512 on item 1174** (`docs/AI.md` §98): `gain_tech`'s
/// buildings cascade runs `check_upgrade` on every city when the gain is a
/// prerequisite of the Large City (`0x6dec2f`), and `do_gather` feeds the
/// escrow the producers then draw on (`006ce450`'s tail; `Build::queue_up`'s
/// `pay_cost(…, escrow, …)`). **The move's value diff (the word's delta,
/// here; its block is `run425_s_word_frame_is_widened_whole`'s):** on
/// run425's first block 7377 who=1's `pop` ours 3 against 7 → agreeing and
/// its `escrow` 0/0/0/0/0 against 39/44/15/47/36 → agreeing; on 7379 its
/// `MAKE[0].val` ours 1800000 against 4194000 → agreeing (every slot was
/// three sevenths), its buckets agreeing on both sides before and after
/// (93, 183, 58, 306, 208 there); on 7383 its buckets ours 43/39/118/109
/// against 34/124/8/149 → agreeing. run425's keys went 1182 → 614, and each
/// earlier widening of the game lost the five `escrow` rows from its first
/// block. Frame 7382's draws went 15 against 9 → agreeing. **The new word's
/// delta: ours 39 draws and the original 3243 on frame 7512, parting at
/// index 30**: ours spends `Guy::set_anim+0x97a < Guy::inc_time+0x271` where
/// the original spends `PathFinder::calc_road_cost+0x46`, 3206 times under
/// `astar_caravan_road`. Inside run425's window (block 7513).
///
/// **7512 → 8519 on item 1185** (`docs/ROADS.md` §9.5):
/// `World::set_blocked_at@006b4900` takes the road off every tile it
/// blocks, `set_road_at(x, y, 0, 0, 0)` through the mesh's door. The Temple
/// `1/2025`, started on 7479 over the caravan road, kept its road under the
/// blocked tile (200, 202) here, so on 7512 the caravan `1/15` taking that
/// tile's waypoint did not verify its route. **The move's value diff (the
/// word's delta, here; its block is `run425_s_word_frame_is_widened_whole`'s):**
/// on run425's block 7513 `1/15`'s `path[0].flags` ours 33 against 1 →
/// agreeing, and `path[1..22].flags` 32 against 0 → agreeing; the window's
/// keys went 614 → 317. Frame 7512's draws went 39 against 3243 →
/// agreeing. **The new word's delta: ours 2 draws and the original 3 on
/// frame 8519, parting at index 0**: ours spends `Guy::set_anim+0x97a <
/// Guy::inc_time+0x271` where the original spends `Guy::set_anim+0x97a <
/// Guy::move+0x19f`. Past run425's window (its last block 7633), widened on
/// run439 (block 8520).
///
/// **8519 → 8820 on item 1191** (`docs/TRANSPORT.md` §6.4): a boat that steps
/// ashore and dies in its own think still takes its figures' `Guy::move`,
/// because `Unit::process@00610bc0` runs `Guy::process` after the think and
/// `Object::die(0)` leaves the guy array alone. The boat `1/56` (carrying
/// `1/55`) stood on its `des` on the walk with `stopped` set on run439's
/// block 8519 and is gone on 8520; the original's index 0 on 8519 is its
/// arrival stand, which ours gave to `1/21`'s wrap. **The move's value diff
/// (the word's delta, here; its block is
/// `run439_s_word_frame_is_widened_whole`'s):** on run439's block 8520
/// `1/21`'s `g.cur_anim[0]` ours 26 against 25 → agreeing and
/// `g.end_time[0]` 100 against 30 → agreeing; run439's keys went 1243 → 608.
/// Frame 8519's draws went 2 against 3 → agreeing. **The new word's delta:
/// ours 2 draws and the original 3 on frame 8820, parting at index 2**: the
/// original spends a third `Guy::set_anim+0x97a < Guy::inc_time+0x271`
/// where ours has none. Past run439's window (its last block 8770), widened
/// on run445 (block 8821).
///
/// **8820 → 8907 on item 1197** (`docs/COMBAT.md` §59.3): every close
/// holds the dead number thirty frames — `Object::close@00647160`'s
/// `hold_frames = 0x1e` — and this crate held only a combat death's. The
/// transports that died putting their passengers ashore in the gap before
/// run439 (the barge `1/62` on 8195, the Merchant Fleet `1/59` on 8411) had
/// their numbers handed on at once here, so the three King's Longbowmen the
/// original numbers `1/68`–`1/70` stood at `1/59`, `1/68`, `1/69`, and the
/// idle rolls went to other figures. **The move's value diff (the word's
/// delta, here; its block is `run445_s_word_frame_is_widened_whole`'s):** on
/// run445's first block 8815 `1/68`'s `g.cur_time[0]` ours 21 against 25 →
/// agreeing and `1/70`'s `g.cur_anim[0]` ours 35 against 1 → agreeing (both
/// through 8944); the block's standing keys went 504 → 183 and run445's
/// 1176 → 627; run439's went 608 → 198, and 1185's `1/68`/`1/69` `dest`
/// rows on 8516 and 8519 agree. Frame 8820's draws went 2 against 3 →
/// agreeing. **The new word's delta: ours 2 draws and the original 28 on
/// frame 8907, parting at index 1**: the original spends
/// `Unit::think_scout+0x941` and 26 `Unit::think_scout+0xaba` where ours
/// spends one `Guy::set_anim+0x97a < Guy::inc_time+0x271`. Inside run445's
/// window (block 8908).
///
/// **8907 → 10183 on item 1214** (`docs/COMBAT.md` §86):
/// `Object::check_target@00649e00`'s head refuses a candidate in another
/// region that is out of range, and this crate's idle search did not ask
/// it. The computer's Caravel `1/35`, idle on the sea at (7392, 480), took
/// the human's building `0/2004` at (4992, 4992) inland, where the
/// original's think went on to `think_scout`. **The move's value diff (the
/// word's delta, here; its block is `run445_s_word_frame_is_widened_whole`'s):**
/// on run445's block 8908 `1/35`'s `group` ours 65 against 79 → agreeing,
/// `order:kind` 10 against 3 → agreeing, `orders_x`/`orders_y` (7392, 480)
/// against (504, 504) → agreeing; `1/35` parts no key through 9071, and
/// run445's keys went 627 → 195. Frame 8907's draws went 2 against 28 →
/// agreeing. **The new word's delta: ours 24 draws and the original 9 on
/// frame 10183, parting at index 0**: ours spends
/// `Leader::create_units+0x642` where the original spends `Guy::set_anim+0x97a
/// < do_cast`. Past run445's window (its last block 9071), widened on
/// run462 (block 10184).
///
/// **10183 → 10185 on item 1228** (`docs/TRANSPORT.md` §16): a boat is
/// counted in `num_units`, `control` and `active` by `Unit::set_type` when
/// its type has population and taken out by `Unit::close`, and this
/// crate's `cast_transport` and `disembark` did neither. So a Merchant
/// Fleet at sea left who=1's `control` one below the original's, and the
/// second `create_units` pass on frame 10183 passed the population gate
/// here (`effective_pop` 75 against `pop_cap` 75) where the original's did
/// not (76). **The move's value diff (the word's delta, here; its block is
/// `run462_s_word_frame_is_widened_whole`'s):** on run462's first block
/// 10178 who=1's `control` ours 72 against 73 → agreeing, `effective_pop`
/// 74 against 75 → agreeing and `num_units[268]` 0 against 1 → agreeing;
/// on 10184 `MAKE[0].t` ours 228 against 597 → agreeing and `active` 70
/// against 71 → agreeing. Frame 10183's draws went 24 against 9 →
/// agreeing. **The new word's delta: ours 9 draws and the original 10 on
/// frame 10185, parting at index 1**: ours spends `Leader::make_stuff+0x221`
/// where the original spends a second `Leader::use_market+0x1ed` and then
/// buys the Senate (`Leader::produce_building+0x1805` twice). Inside
/// run462's window (block 10186).
///
/// **10185 → 10985 on item 1243** (`docs/TECH.md`, "The queue loop"): a
/// unit research finishes through `Build::do_queue@0061e410`, which calls
/// `Build::finished` (and so `gain_tech`) before `Build::unqueue`, and every
/// queued-count decrement is guarded against zero. This crate unqueued
/// first, so who=1's Pikemen research at `1/2020` on 9143 re-targeted the
/// Hoplites entry behind it by the `jump` chain with nothing for its `−1`
/// to take, and `num_queued[84]` stood one high. The ramp priced the
/// second Pikemen a step dearer, goods stood 8 short, and the Senate was
/// unaffordable on 10184. **The move's value diff (the word's delta, here;
/// its block is `run462_s_word_frame_is_widened_whole`'s):** on run462's
/// first block 10178, who=1's `num_queued[84]` ours 1 against 0 →
/// agreeing, `bucket[0:food]` 125 against 133 and `bucket[4:metal]` 107
/// against 115 → agreeing, `1/2020`'s `queue[0].cost` 86/66 against 78/58
/// → agreeing; on 10185 `MAKE[0].val` 1,200,000 against 4,800,000 →
/// agreeing; on 10186 `bucket[2:wealth]` 57 against 7 → agreeing, and
/// `1/2026` linked. run462's keys went 1398 → 198. Frame 10185's draws
/// went 9 against 10 → agreeing. **The new word's delta: ours 10 draws and
/// the original 11 on frame 10985, parting at index 2**: ours spends
/// `Leader::make_stuff+0x221` where the original spends a third
/// `Leader::use_market+0x1ed`. Past run462's window (its last block
/// 10434), widened on run480 (block 10986).
///
/// **10985 → 11328 on item 1264** (`docs/TECH.md` step 8): a building type
/// gained converts every in-use building whose type's `upgrade` is it
/// (`Leader::gain_tech`'s loop at `6dde64`–`6ddf2b`, `Wall::set_type`), and
/// this crate converted none. who=1's Tower `1/2014` is a Keep in the
/// original's `num_buildings` from the Keep's gain (Tower 1, Keep 0 on
/// run357's 5601; Tower 0, Keep 1 on run425's 7377) and stayed a Tower
/// here, so `create_buildings` read a first Keep to offer and on 10984
/// valued it at 900,000. **The move's value diff (the word's block before,
/// here; its block is `run480_s_word_frame_is_widened_whole`'s):** on 10985
/// who=1's `MAKE[1].t` ours 440 against 134 → agreeing and `MAKE[1].val`
/// 900000 against 611022 → agreeing; `use_market`'s `need` over three slots
/// takes the timber short again. run480's keys went 1490 → 220. Frame
/// 10985's draws went 10 against 11 → agreeing. **The new word's delta:
/// ours 62 draws and the original 60 on frame 11328, parting at index 57**:
/// ours spends `Guy::set_anim+0x97a < Guy::inc_time+0x271` where the
/// original spends `Guy::set_anim+0x104b`. Past run480's window (its last
/// block 11236), widened on run490 (block 11329).
///
/// **11328 → 11549 on item 1281** (`docs/TECH.md`, "The piece moves with
/// the age"): who=1's third age, on frame 11328, re-pieces every figure one
/// bracket up (`Leader::gain_tech:2372`, `Unit::update_gpiece`), and this
/// crate re-pieced none; its snap wrote the order's `dest_angle` as well;
/// and a figure's `guy_flags & 8` stays the piece `init_real` saw, where
/// this crate read the new one. **The move's value diff (the word's block,
/// run490's 11329):** who=1's `1/1` `g.gpiece[0]` ours 6336 against 8448
/// → agreeing, and the 65 such rows with it; `1/0`'s `dest_angle`
/// −1615724544 against −1771962368 → agreeing, and fourteen more; block
/// 11329's rows 115 → 16 (the two merchants' over-time piece stands).
/// Frame 11328's draws went 62 against 60 → agreeing. On the way, 11349:
/// `1/15`'s `g.cur_anim[0]` ours 21 (`CHAR_TURN_LEFT`) against 8 → agreeing.
/// run490's keys went 1311 → 382. **The new word's delta: ours 11 draws
/// and the original 4 on frame 11549, parting at index 0**: ours spends
/// `Unit::think_scout+0x941` (and six `+0xaba`, all `1/35`'s) where the
/// original spends `Guy::set_anim+0x97a < Unit::do_guard+0x7f4`. Inside
/// run490's window, widened there (block 11550).
///
/// **11549 → 11637 on item 1297** (`docs/SCOUT.md` §13 item 1b): `1/35` is
/// a Caravel (`TypeIndex` 325), and on frame 11523 its region scan found no
/// cell (one `+0x941`, no `+0xaba`, both sides). `think_scout`'s tail sends
/// a **sea** unit to `Unit::add_to_army` (`5f6db5`), which this crate held
/// as a seam: the original's `1/35` joined army 3's group 72 (4 → 5
/// members, `stamp` 11523) and walked to its first member on an
/// `ATTACK_TO`, and ours stood idle until its next think on 11549, whose
/// stride found six cells. **The move's value diff (block 11524, the state's
/// first parting, walked back from the word's 11550):** `1/35`'s `group` ours
/// 64 against 72 → agreeing; `orders_x` 24288 against 43128 → agreeing;
/// `path:length` 0 against 24 → agreeing; `dest_angle` 295960576 against
/// 1076035584 → agreeing; and on 11550 `order:kind` ours 3 against 2 →
/// agreeing. run490's keys went 382 → 230. Frame 11549's draws went 11
/// against 4 → agreeing. **The new word's delta: ours 2 draws and the
/// original 38 on frame 11637, parting at index 1**: ours spends
/// `Guy::set_anim+0x97a < Guy::inc_time+0x271` where the original spends
/// `Guy::init_real+0x52 < Unit::init+0xb97 < Objects::init_unit+0xbd`
/// (eighteen of them, and seventeen idle stands). Past run490's window
/// (its last block 11579), widened on run506 (block 11638).
///
/// **11637 → 12582 on item 1302** (`docs/AI.md` §99.14): the eighteen were
/// six **decoy** squads (Peltasts, King's Yeomanry, Pikemen, `unit_masks`
/// 1), made by who=1's General `1/98`, whose Create Decoys this crate never
/// cast: `Army::use_generals` gives an army's General a `think_spellcaster`
/// turn every 128 frames, and its hero arm lays the cast when the General
/// stands. Army 4's turn laid it on 11410 and on 11538; the tick's
/// `ATTACK_TO` replaced the first on 11508, handing the craft back. **The
/// move's value diff (run490's block 11411, the state's first parting,
/// walked back from the word):** `1/98`'s `spell_time` ours 0 against 1 →
/// agreeing, and `mana_burn` 0 against 1000 → agreeing; on 11509 both 0,
/// and on 11638 `mana_burn` 901 on both; on run506's block 11638 group 70
/// lists 37 on both, and `1/104`..`1/120` and `1/93` agree in type and
/// place (the copies stand where the General's collision pair allows).
/// run506's keys went 1375 → 224, run490's 229 → 226. Frame 11637's
/// draws went 2 against 38 → agreeing; on the way, 11764's army-4 tick
/// (`num_decoys` off the standard line) and 11780's `create_units` (a
/// decoy is no unit of the census) agree. **The new word's delta: ours 9
/// draws and the original 10 on frame 12582, parting at index 4**: the
/// original spends `Leader::make_stuff+0x63d` where ours goes on to
/// `Guy::set_anim+0x97a < Guy::inc_time+0x271`. Past run506's window (its
/// last block 11888), widened on run508 (block 12583).
///
/// **12582 → 13385 on item 1326** (`docs/CITIES.md` §13 item 14): the
/// Forbidden City and the Red Fort are hero-masked (`obj_masks`
/// `0x4000000`), and `BuildType::init` skips their `B[from].to = this`,
/// so the Small City's successor is the Large City. The loader linked
/// them anyway, `get_buildings(VILLAGE)` counted no Large City, and
/// who=1 offered no Citizen from the frame its last Small City grew.
/// **The move's value diff (run508's block 12581, `create_units`' frame
/// 12580):** who=1's `MAKE[5]` `t` ours −1 against 50, `val` −1 against
/// 280519, `num` 1 against 4, `cat` 0 against 5, `escrow` 0 against 1 →
/// agreeing; on 12583 `1/2017`'s `queue:queued` 0 against 4 and
/// `num_queued[0]` 0 against 4 → agreeing, and `bucket[0:food]` 356
/// against 122 → agreeing. run508's keys went 1817 → 218 (the make
/// list's `city` compared as the leader's own index takes six more).
/// Frame 12582's draws went 9 against 10 → agreeing. **The new word's
/// delta: ours 4 draws and the original 54 on frame 13385, parting at
/// index 0**: the original's step-11 `make_stuff` spends
/// `Leader::use_market+0x1ed`, then a Farm's `produce_building` (42
/// `+0xc99`, 4 `+0x1805`, `Farms::add`) and two `+0x221`; ours' step 11
/// finds its head empty and spends nothing. Past run508's window (its
/// last block 12833), widened on run523 (block 13386).
///
/// **13385 → 14141 on item 1341** (`docs/AI.md` §56.5): `get_cost`'s
/// bump loop. An owned type is charged, per resource, the base of any
/// upgrade of it that is queued; the Bombard's research at `1/2024`
/// makes the Trebuchet 8m/8t, so `1/2028`'s first costs 80 × 95/100 and
/// the second is past who=1's timber. **The move's value diff (run523's
/// block 13383, step 8's purchase on frame 13382):** who=1's
/// `num_queued[216]` ours 2 against 1 → agreeing, `1/2024`'s `queued` 2
/// against 1 → agreeing, `1/2028`'s `queue[0].cost[0]`/`[1]` 66 against
/// 76 → agreeing, and `bucket[1:timber]` 14 against 89 and
/// `bucket[4:metal]` 59 against 134 → agreeing; on 13385 `MAKE[0]` takes
/// the Farm on both. run523's keys went 1015 → 138 (on the tree merged with 1330's). Frame 13385's draws
/// went 4 against 54 → agreeing. **The new word's delta: ours 5 draws
/// and the original 4 on frame 14141, parting at index 1**: ours spends
/// a second `Guy::set_anim+0x97a < Guy::inc_time+0x271` where the
/// original goes on to `Guy::set_anim+0x104b` (seed `c55509fc`). Past
/// run523's window (its last block 13636), widened on run535 (block
/// 14142).
///
/// **14141 → 15862 on item 1351** (`docs/GOLDEN.md` §48): a decoy closes
/// at `(general_upgrade + 2) × DECOY_TIME / 2`. `Unit::process`'s decoy
/// arm takes its age up a frame and, at 2,500, calls `Unit::close(0, −1,
/// 0.0)` — no death draw — and returns. This crate aged its copies and
/// never closed them, so who=1's `1/110` was still standing to spend 14141's
/// second `Guy::set_anim+0x97a`. **The move's value diff (run535's block
/// 14137, the state's first parting, walked back from the word):**
/// `1/104`..`1/120` ours alone (each `mana_burn` 2499 on 14136 on both
/// sides) → closed on both, and `1/93` ours alone on 14138 → closed on
/// both. run535's keys went 1730 → 163. Frame 14141's draws went 5
/// against 4 → agreeing. **The new word's delta: ours 11 draws and the
/// original 7 on frame 15862, parting at index 0**: ours spends four
/// `Army::find_target+0x7df` before the original's first, `Guy::set_anim+
/// 0x97a < Animal::do_idle+0x19` (seed `5faaa95a`). Past run535's window
/// (its last block 14392), widened on run544 (block 15863).
///
/// **15862 → 15883 on item 1362** (`docs/TRANSPORT.md` §8.3): the escort.
/// A land army whose `find_target` takes an enemy target in another cell
/// region than its own point hands the target to every navy of more than
/// two captains whose sea coasts both (`Armies::send_navy`). Army 0 took
/// who=0's Napata on 15612 from region 12; the original gave it to the
/// navy, army 3 (`1/30`, `1/35` corvettes, `1/61`, `1/67` fireships, `1/57`
/// a frigate), and on army 3's own turn, 15862, `do_marching` kept an
/// enemy's city. This crate had left the call a seam, so army 3 still
/// held who=1's own Newcastle, and an untroubled city of one's own is
/// retargeted: four `find_target` scores. **The move's value diff (run544's
/// block 15863, the word's own):** `1/30`'s `orders.len` ours 0 against
/// 1 → agreeing (its `orders_x/y` (33240, 25800) on both), the same for
/// `1/35`, `1/57`, `1/61` and `1/67`, and group 72's `order_num` 55
/// against 56 and `o` (33590, 25667) against (33225, 25790) → agreeing.
/// run544's block 15863 went 88 → 0 keys. Frame 15862's draws went 11
/// against 7 → agreeing. **The new word's delta: ours 11 draws and the
/// original 8 on frame 15883, parting at index 5**: ours spends three
/// more `Guy::set_anim+0x97a < Guy::inc_time+0x271`, all `1/132`'s, where
/// the original goes on to `Farms::inc_time+0x1ae` (seed `34d7c105`).
/// Inside run544's window (block 15884).
///
/// **15883 → 15985 on item 1370** (`docs/ORDERS.md` §6.9.2): the pack
/// before a march. `Unit::work`'s pack arm — a packing type standing
/// unpacked under a move-family head — puts a `0x28b` cast on top of the
/// move, and the unit stands through its 80 frames before it walks
/// (`cast_pack` sets the bit). Army 0's march reached its Bombard `1/132`
/// (type 267) deployed on 15868; this crate had no pack, so it turned and
/// walked at once. **The move's value diff (run544's block 15869, the
/// state's first parting, walked back from the word):** `1/132`'s
/// `orders.len` ours 1 against 2 (the original's `CASTORDER` `spell 651`
/// over the `GROUPATTACKTOORDER`) → agreeing, its `g.cur_anim` ours 22
/// against 23 (`CHAR_PACK`) → agreeing, and group 71's `speed` ours 23
/// against 25 → agreeing; and on the word's own block 15884, `1/132`'s
/// `pos` ours (42029, 40414) against (42031, 40399), `g.cur_time` 1
/// against 14 and `g.stopped` 0 against 1 → agreeing. `1/132` parts on no
/// scored key to run544's end, the pack's end on 15947 included (block
/// 15948: the bit, `mylos` 4, the packed piece). run544's keys went 1380
/// → 982. Frame 15883's draws went 11 against 8 → agreeing. **The new
/// word's delta: ours 50 draws and the original 49 on frame 15985,
/// parting at index 46**: ours spends `Guy::set_anim+0x97a <
/// Unit::move_step+0x823` (`1/88`'s) where the original goes on to
/// `Guy::set_anim+0x97a < Guy::inc_time+0x271` (seed `732a71ea`). Inside
/// run544's window (block 15986).
///
/// **15985 → 16009 on item 1377** (`docs/AI.md` §100): the Tower line in
/// the placement. `Leader::produce_building`'s `local_84` is `is(0x1b7,
/// 0)` inside the `e` arm and `find_friends`' tower arm is `is(0x1b7, 0)`
/// — the **line**, so a Keep (440, `FROM` Tower) answers both; this crate
/// asked `ident == Tower`. Who=1's Keep `1/2047` for its city at (49, 30)
/// scored every friendless cell 1255 and took the last tie; the original
/// gives a Keep beside two farms `(4 + 2)² × 1000` and takes (51, 28).
/// **The move's value diff (run544's block 15986, the word's own):**
/// `1/2047`'s `build:x_internal`/`y_internal` ours (35904, 25728) against
/// (39744, 21888) → agreeing; `1/88`'s order kind 3 against 7 and
/// `1/122`'s 7 against 3 (the builder each side called) → agreeing, with
/// both units' figures; who=1's `defense` ours 1 against 2 →
/// agreeing (`Build::init`'s `+1`, item 1377). run544's keys went 982 →
/// 662. Frame 15985's draws went 50 against 49 → agreeing. **The new
/// word's delta: ours 9 draws and the original 6 on frame 16009, parting
/// at index 1**: ours spends `Guy::set_anim+0x97a < Unit::do_move+0x11cf`
/// where the original goes on to `Guy::set_anim+0x97a <
/// Guy::inc_time+0x271`. Inside run544's window (block 16010), where the
/// Supply Wagon `1/153` (type 63) of army 0's group 71 parts on its
/// figures and its move's `pause`, 14 against 15.
///
/// **16250 → 16482 on item 1418** (`docs/AI.md` §107): `CollCheck::
/// move_unit@00682ad0` takes a **one-block form** when both discs lie in
/// one world cell — the block is found by the *old* tile's region and both
/// passes run on it ungated — where this crate gated the set pass by the
/// new tile's. A Galleon born on the land half of a coastal cell and moved
/// onto its water half in one step (run572's `1/170`, born 16171 at
/// (604, 805) and moved to (603, 810)) leaves its new disc set in the
/// original and nothing in ours. **The move's value diff** (run578's
/// `RON_COLLIDE_PROBE` over tick 16238): `1/109`'s `collide_here` hit
/// cell (606, 809) in the original against (606, 811) in ours — the
/// ghost's bit against `1/119`'s — and `is_here(1/119)` 0 against 1;
/// block 16239's `1/109` `half_step` ours 1 against 0 → agreeing, run572's
/// keys 1691 → 263. **The new word's delta: ours 9 draws against 17 on
/// frame 16482, at index 0**: ours `Guy::set_anim+0x97a <
/// Guy::inc_time+0x271` where the original spends `Guy::init_real+0x52`;
/// past run572's window (16155..16411).
///
/// **16221 → 16250 on item 1410** (`docs/AI.md` §106):
/// `resolve_unit_collision`'s step 5 waits on a collider whose current
/// order is a transport cast (spell `0x28a`, `005fa4f4`) as on a move,
/// where this crate repathed. East Indies' `1/110` (block 16192: path
/// length ours 57 against 51, `path[50]` (28488, 37608) flags 2 against
/// (29016, 38376) flags 4, `dest` 0 against 1, position (29256, 38520)
/// against (29239, 38541)) walked into `1/111`, which stands casting. The
/// 16192 block's 9 keys, 16193's 15, 16216's 9, 16217's 15, 16221's 6 and
/// 16222's 4 → 0; run572's keys 2279 → 1694. **The new word's delta:
/// ours 12 draws against 13 on frame 16250, at index 1**: ours
/// `Guy::set_anim+0x97a < Unit::move_step+0x823` where the original
/// spends `Guy::set_anim+0x97a < do_cast`; `1/109` `order:kind` 2 against
/// 14, `orders.len` 1 against 2, `orders_x/y` (7704, 4872) against
/// (29003, 38668).
///
/// **16179 → 16221 on item 1407** (`docs/AI.md` §105): `cast_transport`
/// clears the handed-over path's top embark flag when
/// `get_tregion(top) == get_tregion(spot)`, and `get_tregion` is the
/// coastal refinement ([`World::tregion_alt`]), where this crate had read
/// the plain region. East Indies' Galleon `1/171` (born 16175) took the
/// top (29160, 38040) with flags 4 and tolerance 0 where the original's
/// has flags 0, and the original's first `find_path` of the boat answered
/// that flagged top's relaxed water test differently: `path_recursion` 2
/// and an extra top (29124, 37884) in the original (run572 block 16176:
/// path length ours 49 against 50, `path[48].flags` 4 against 0, move
/// `dest` (29160, 38040) against (29124, 37884), `last_x` −1 against
/// 29352), and the boat was ashore on 16178 here. Block 16176's 18 keys →
/// 2, 16179's 40 → 0, 16180's 17 → 0; run572's keys 3416 → 2279. **The
/// new word's delta** is on `docs/AI.md` §105.
///
/// **16160 → 16179 on item 1401** (`docs/AI.md` §104): `is_castable`'s
/// head answers 0 to a decoy (`unit_masks & 1`) for every craft but pack
/// and unpack, and `do_cast` had asked only `can_transport`. East Indies'
/// `1/128` (Arquebusiers, a decoy of the General's) held a transport cast
/// on 16160 that the original's `SpellType::cast` refuses: it spent its
/// figure's draw and built no boat, where this crate built a second
/// Galleon (`1/168`) and boarded both squads. **The move's value diff**
/// (run572, block 16161): `1/128`'s `inside` ours 167 against −1,
/// `orders.len` 0 against 2, `path:length` 0 against 50; `1/119` and
/// `1/120` `inside` 167 against −1; `1/142`..`1/144` `inside` 168 against
/// 167; `1/168` ours alone → agreeing (the block's 107 keys → 1). Frame
/// 16160's draws went 46 against 44 → agreeing. **The new word's delta:
/// ours 11 draws and the original 11 on frame 16179, parting at index 2**
/// (the sequence; the count parts later, on 16187, 4 against 5 at index
/// 0): ours spends `Guy::set_anim+0x97a < Unit::do_idle+0x7d` where the
/// original spends `Guy::set_anim+0x104b`. Ours' Galleon `1/171` (born on
/// 16175 by `1/87`'s cast, in both) is gone on 16178 and its boarders
/// `1/79`, `1/86`, `1/87` stand ashore.
///
/// **16009 → 16160 on item 1383** (`docs/AI.md` §101): the re-plan's
/// `TAKE` lands past `do_move`'s pause check (`do_move:699` → `:743`; the
/// check at `:729` is the `else` of the `masks & 8 == 0` block), so a unit
/// that re-plans and verifies its line steps with its `pause` untouched
/// and stands no frame. **The move's value diff (run544's block 16010, the
/// word's own):** the Supply Wagon `1/153`'s move `pause` ours 14 against
/// 15 → agreeing, its figures' `g.cur_anim` ours 0 against 7/9/9, `g.stopped`
/// 1 against 0, `angle`, `heading`, `g.angle`, `g.des_angle`, `g.cur_time`,
/// `g.end_time` and `g.last_time` → agreeing (the turn in place the
/// original's step takes). Block 16010 went 23 → 0 keys; run544's keys
/// went 662 → 243. Frame 16009's draws went 9 against 6 → agreeing. **The
/// new word's delta: ours 46 draws and the original 44 on frame 16160,
/// parting at index 4**: ours spends `Guy::init_real+0x52` (after `1/128`'s
/// `do_cast`) where the original spends the second `Guy::set_anim+0x97a <
/// do_cast` (`1/143`'s). Past run544's window; widened on run572 (block
/// 16161).
/// Item 1427: the measured 16760 moves to 16762 (five draws against six,
/// first differing label at index 1); run583 widens both. The coastal
/// candidate-region fix restores 1/189's block-16700 position from
/// (40374,24330) to the original's (40374,24328). COLLISION §13.3.
/// Item 1431: the large-unit recovery stride moves 16762 to 16878;
/// Six draws against twelve, index 0 move_step against Army::find_target.
/// run583 pins the whole new word and restores 1/189's path (9 -> 16
/// entries) and start_dist (1056 -> 0) on block 16761. PATHFINDER §30.
/// Item 1432: numeric region order restores group 75's army 6 -> 7,
/// stance 0 -> 1 on block 16879, and advances 16878 -> 16940. Four draws
/// against five, index 0 inc_time against Unit::do_idle; run583 widens it.
/// Item 1433 measures 17507 (eight draws against seven, index 1 do_idle
/// against inc_time). It kept floor 16940 pending item 1434/run585:
/// run583 ends at 17010. Transport 172's path and
/// passenger 91's disembarkation now agree throughout run583.
/// Item 1434: reading the retained collision-chain slot moves 17507 to
/// 17653 (14 draws against 13, index 5 inc_time against set_anim+0x104b).
/// run585 widens both words; transport 111's recovery and its passengers
/// agree at the old word. COLLISION §22.
/// Item 1435: including the cavalry lineage in target strength moves
/// 17653 to 17698 (8 draws vs 7, index 5 take_damage vs Farms::inc_time).
/// run585 covers both; army 7's target and army 5's orders agree. ARMY §25.
/// Item 1436: measured Yeomanry release nodes move 17698 to 17785
/// (21 draws vs 23, index 8 idle wrap vs Guy::init_real+0x52).
/// run588 widens the successor; run587 pins the booked arrows. COMBAT §87.
/// Item 1437: the President's own decoy radius moves 17785 to 17907
/// (5 draws vs 7, index 0 inc_time wrap vs do_cast). run589 widens the
/// successor; run588 pins restored member 154 and group 66. GOLDEN §60.
/// Item 1438: reused figures clear their retained point at birth; 17944
/// has 100 draws vs 118, index 92 do_guard vs find_attack_pos. run589
/// widens it; run585 pins the corrected landing. COLLISION §23.
/// Item 1439: packed siege takes its army target; 18076 has 20 draws
/// versus 19, index 6 inc_time wrap versus Unit::set_anim. run594 widens
/// the successor through the closing state. COMBAT §88.
/// Item 1440: the strike re-seats its crew before animation; 18089
/// has 21 draws vs 22, index 5 inc_time vs move_step+0x823. run594
/// widens the new word; the crew timer now agrees. COMBAT §89.
/// Item 1441: supply upgrade speed closes the word at 18140. run594
/// pins the old blocked step and the closing record; 191 endpoint units
/// have zero off/unlinked/extra/torn. SUPPLY, "Upgrade speed".
pub(crate) const SECOND_WORD_EAST_INDIES: i64 = 18_140;

/// Third pair baseline, item 1442: East Indies 986, 387 vs 6 draws;
/// index 0 is road-cost versus farm-clock. run602 block 987 widens it.
/// Item 1443: French timber capacity preserves the camp and moves 986 to
/// 7356, 4 vs 2 draws, init_real versus inc_time. run603 block7357 widens it.
/// Item 1445: university admission refuses an eighth scholar; 8182 has
/// 15 vs 47 draws, index 2 produce_building+1805 versus +c99. run610 block8183.
/// Item 1446: a wonder's start lights it for every player, which is first
/// contact (VISION §6.4): 8182 → 8236, 4 vs 5 draws, index 1 Guy::set_anim
/// under Guy::inc_time versus under Unit::move_step. run612 block 8237.
/// Item 1449: each building's queue runs inside its own Build::process, so
/// a citizen trained this frame is there for a wonder's recruiter: 8236 →
/// 8385, 8 vs 4 draws, index 0 Leader::make_stuff+0x221 versus
/// Guy::set_anim under Guy::inc_time. run616 block 8386.
/// Item 1451: `largest_gather` is 1, not 0, in every gather building's
/// value (AI §115): 8385 → 8840, 100 vs 98 draws, index 96 Guy::set_anim
/// under Guy::inc_time versus Guy::set_anim+0x104b. run617 block 8841.
/// Item 1452: a ship pushed within four frames pushes back from where it
/// stands (COLLISION §24): 8840 → 9655, 8 vs 9 draws, index 0 Guy::set_anim
/// under Guy::inc_time versus Unit::do_air_physics+0x639. run622 block 9656.
/// Item 1453: the gull flies toward its dock from a snapped birth
/// (SYNC §3.29): 9655 → 9777, 7 vs 9 draws, index 0 Guy::set_anim under
/// do_trade versus Leader::make_stuff+0x221. run623 block 9778.
/// Item 1454: the Pyramids' city limit and discount, and `already_built`
/// (AI §116, TECH): 9777 → 10131, 4 vs 6 draws, index 0 Guy::set_anim
/// under Guy::inc_time versus under Unit::do_non_flat_gather. run624 block
/// 10132.
/// Item 1455: a Citizen is ramped by the Militia line too (COSTS), and a
/// French unit of the Siege Factory line moves 20% faster (MOVEMENT):
/// 10131 → 10802, 6 vs 5 draws, index 0 Guy::set_anim under
/// Unit::move_step versus under Guy::inc_time. run629 block 10803.
/// Item 1458: a member's slot across a coast is re-placed on slot 0's, and
/// a recycled slot keeps its `path_recursion` (GROUPS §37): 10802 → 11582,
/// 188 vs 189 draws, index 0 Leader::make_stuff+0x221 versus
/// Leader::use_market+0x1ed. run631 block 11583.
/// Item 1460: a French unit of the Siege Factory line costs
/// `FRENCH_SIEGE_COST` less (COSTS): 11582 → 12794, 17 vs 18 draws,
/// index 8 Guy::set_anim under Guy::inc_time versus under Unit::do_guard.
/// run634 block 12795.
pub(crate) const THIRD_PAIR_WORD_EAST_INDIES: i64 = 12_794;
/// Great Lakes 2576, 36 vs 42 draws; index 4 bird versus make_stuff.
/// run601 block 2577 widens it. These are frames, not mechanism bookings.
/// Item 1444: the goody-ruins placement gate closes the stream at 5638.
/// run601 holds the corrected sites; run598 scores the closing state.
pub(crate) const THIRD_PAIR_WORD_GREAT_LAKES: i64 = 5638;
pub(crate) const WIDENING_FRENCH_CAPACITY: (i64, i64) = (981, 993);
pub(crate) const WIDENING_FRENCH_SCHOLAR: (i64, i64) = (7351, 7363);
pub(crate) const WIDENING_FRENCH_EAST_INDIES_8182: (i64, i64) = (8177, 8189);
pub(crate) const WIDENING_FRENCH_CONTACT: (i64, i64) = (8030, 8043);
pub(crate) const WIDENING_FRENCH_CONTACT_7946: (i64, i64) = (7940, 7952);
pub(crate) const WIDENING_FRENCH_BUILDER_8156: (i64, i64) = (8140, 8159);
pub(crate) const WIDENING_FRENCH_BUILDER_7963: (i64, i64) = (7958, 7999);
pub(crate) const WIDENING_FRENCH_TRANSPORT_8430: (i64, i64) = (8420, 8439);
pub(crate) const WIDENING_FRENCH_SHIP_6141: (i64, i64) = (6136, 6157);
pub(crate) const WIDENING_FRENCH_EAST_INDIES_8236: (i64, i64) = (8231, 8243);
pub(crate) const WIDENING_FRENCH_EAST_INDIES_8385: (i64, i64) = (8380, 8392);
pub(crate) const WIDENING_FRENCH_EAST_INDIES_8840: (i64, i64) = (8835, 8847);
pub(crate) const WIDENING_FRENCH_EAST_INDIES_9655: (i64, i64) = (9650, 9662);
pub(crate) const WIDENING_FRENCH_EAST_INDIES_9777: (i64, i64) = (9772, 9784);
pub(crate) const WIDENING_FRENCH_FOOD_7782: (i64, i64) = (7776, 7788);
pub(crate) const WIDENING_FRENCH_HOPLITE_9985: (i64, i64) = (9980, 9992);
pub(crate) const WIDENING_FRENCH_MUSTER_10765: (i64, i64) = (10760, 10796);
pub(crate) const WIDENING_FRENCH_EAST_INDIES_10131: (i64, i64) = (10126, 10138);
pub(crate) const WIDENING_FRENCH_EAST_INDIES_10802: (i64, i64) = (10797, 10809);
pub(crate) const WIDENING_FRENCH_EAST_INDIES_11582: (i64, i64) = (11577, 11589);
pub(crate) const WIDENING_FRENCH_EAST_INDIES: (i64, i64) = (12789, 12801);
pub(crate) const WIDENING_FRENCH_GREAT_LAKES: (i64, i64) = (2571, 2583);
pub(crate) const WIDENING_FRENCH_LAKES_CLOSING: (i64, i64) = (5633, 5639);

/// **The third map's word at Toughest** (item 1221, DECISIONS 56 §1):
/// run470, Great Sahara in the second pair's lobby, walked from run468's
/// `DUMP_ALL` start (`diff::sahara_toughest`).
///
/// **Item 1426 measures 12538; the pin stays 11985** until item 1429
/// captures the new word, beyond run574's window (`docs/AI.md` §109).
/// Democracy's research discount removes 11985's five extra draws and
/// run574's keys fall 979 -> 144. Monotheism's queued food/wealth cost
/// 165 against 132 -> 132 each; Medicine's offer 59400 against 237600
/// -> 237600. New sequence delta: 7 draws each at 12538, index 1,
/// ours `Guy::set_anim+0x97a < Guy::move+0x19f`, theirs
/// `Guy::set_anim+0x97a < Guy::do_turn+0x4a < Guy::turn_towards+0x69`.
/// Count first parts at 12569. No mechanism is booked for the new word.
///
/// **12569 since item 1429** (`docs/AI.md` §118): a standing turn runs
/// inside guy 0's own `Guy::move`, ahead of the crew's. The measured word
/// 12538 (item 1426, seven draws each, index 1: ours `Guy::move+0x19f`,
/// theirs `Guy::do_turn+0x4a < Guy::turn_towards+0x69`) was **the Bombard
/// `1/139`'s three draws in another order, with no dumped field parted**:
/// guys 1 and 3, the untracked crew, stood `stopped` 1 on `cur_anim` 8 at the
/// head of 12538 and this crate paid each its arrival stand in `guys_follow` ahead
/// of guy 0's `turn_towards`, whose `do_turn` recursion re-slots them — the
/// original draws guy 0's turn and the crew's two `do_turn+0xe5`. The
/// dump's own coordinates are `1/139` guys 0, 1 and 3's `cur_anim` 8 → 0 on
/// block 12539 after `stopped` 1 on guys 1 and 3 on block 12538, all
/// agreeing on both sides before and after. **The new word's delta:
/// ours 13 draws and the original 18 on frame 12569, parting at index 6**:
/// ours `Guy::set_anim+0x97a < Guy::do_turn+0x4a < Guy::turn_towards+0x69`
/// where the original spends `Guy::set_anim+0x97a < Unit::do_idle+0x7d`,
/// and four more `Guy::set_anim+0x104b` at its end. Widened on run584
/// (block 12570); the first state part on 12569 is `1/63` and `1/76`
/// `half_step` (ours 1 and 0 against 0 and 1), `1/130`'s `g.cur_anim[0]`
/// (ours 9 against 8) and `1/90`'s tracked crew figure.
///
/// **11985 since item 1416** (`docs/AI.md` §108): the other arm of
/// `Build::finished`'s Senate tail. Who=1's Senate finishes Democracy on
/// 11882 with The Senator `1/80` standing, and the original `set_type`s it
/// to The President (353 → 355) where this crate left it. **The move's
/// value diff** (run574, block 11883): who=1's `num_units[303]` ours 1
/// against 0 and `[305]` 0 against 1, `1/80`'s `gpiece[0..2]` 303/12975/
/// 25647 against 305/12977/25649, `cur_anim`, `end_time`, `des_x/y`,
/// `track_dx` → agreeing; run574's keys 1,280 → 979. Frame 11882's draws
/// went 7 against 11 parting at index 2 → agreeing. **The new word's
/// delta: ours 15 draws and the original 10 on frame 11985, parting at
/// index 3**: ours spends `Leader::produce_building+0x1805` where the
/// original spends `Guy::set_anim+0x97a < Animal::do_idle+0x19`. The
/// first state part is `1/124` on block 11976 (`g.angle[0]` ours
/// 1431655765 against 0, `orders_x/y` 21984/18912 against 22008/18936);
/// who=1's `MAKE[2]` parts on 11979 (`t` 561 against 604). Widened on
/// run574 (block 11986).
///
/// **11882 since item 1398** (`docs/AI.md` §103): the census's live half.
/// `Leader::track_unit_type@006e0dd0` and `Unit::set_type@00612fa0` move
/// `peasants`, `scholars` and `caras` the frame a unit is born or closed,
/// and this crate recounted them at the sweep alone — who=1's `peasants`
/// 42 against 43 on 11345 and `caras` 3 against 4 on 11372. And the
/// economy's dirty flag (`leader_flags & 0x2000000`) is raised by a
/// Scholar's `go_inside` and an exit from a University or an Oil Platform,
/// never by `Build::queue_up` or an ordinary trained unit's birth: ours
/// reassembled the holdings on 11183, the original not until 11391, and
/// the rare count `calc_gather` sums (`known_rares` 5 against 0) put a
/// Merchant offer (`t` 61, 931,034) above the Bombard on block 11381.
/// **The move's value diff** (run571): `peasants` 42 → 43 on 11345,
/// `caras` 3 → 4 on 11372, `gather_stamp` and `known_rares` on 11184 and
/// `MAKE[2]`/`MAKE[3]` on 11381 → agreeing; 419 keys → 122. **The new
/// word: ours 7 draws against the original's 11 on frame 11882, parting
/// at index 2** (`Guy::set_anim+0x97a < Guy::inc_time+0x271` against
/// `Guy::init_real+0x52`), past run571's window.
///
/// **11382 before it, since item 1388** (`docs/AI.md` §102): `Muster::library_cities`
/// was never written, so the first library's queue advanced one slot where
/// `LeaderData::get_building_cities@006e06f0` lets it advance one per city
/// holding a library — who=1's Trade (`560`) and Conscription (`575`)
/// ran together in the original from before block 10774. **The move's
/// value diff** (run562, block 10774): `1/2005`'s `queue[0].job_counter`
/// ours 13500 against 18700 and `queue[1]` 0 against 13500 → agreeing, and
/// with them who=1's `epoch[2]` and `epochs` on 10903, `queued` on 10903,
/// `resource_cap` on 10904 and `MAKE[3].t` on 10985 (run562's keys 144 →
/// 101); run571's block 11181 — the original's head Scholars and
/// Citizens at 9,999,999 — and the word's block 11183 agree, 1,270 keys →
/// 419 with nothing parting before block 11184. Frame 11182's draws went
/// 11 against 12 parting at index 0 → agreeing. **The new word's delta:
/// ours 17 draws and the original 16 on frame 11382, parting at index 0**,
/// the same pair (`Leader::use_market+0x1ed` where the original spends
/// `Leader::make_stuff+0x221`); who=1's `MAKE[2]` parts on block 11381 —
/// ours `t` 61 at 931034, the original's the Bombard (267) at 604160 — and
/// `caras` ours 3 against 4 on 11372, `peasants` 42 against 43 on 11345;
/// widened on run571 (block 11383).
///
/// **11182 before it (item 1379)** (`docs/AI.md` §28.1): `upgrade_units` asks
/// `researching(t, −1, 0, 0)` (`6c657e`), whose unit arm counts a rung of
/// `t`'s line in research (`6db5d9`..`6db62c`) — who=1's Heavy Horse
/// Archers were, so the original never reaches the Dragoon's roll, where
/// this crate asked the tech equality alone. **The move's value diff**
/// (run562, block 10780): who=1's `MAKE[0].val` ours 1152000 against
/// 672000 (`t` 189 against 229), `MAKE[2].cat` 7 against 8, `0/3`'s
/// `orders_x` 5112 against 5304 → agreeing; nothing parts on 10780 or
/// 10781. run562's keys went 1,803 → 145. Frame 10779's draws went 15
/// against 14 parting at index 4 → agreeing. **The new word's delta: ours
/// 11 draws and the original 12 on frame 11182, parting at index 0**: ours
/// spends two `Leader::use_market+0x1ed` before `make_stuff`'s own,
/// where the original spends `Leader::make_stuff+0x221
/// < Leader::production_ai+0x1fa < Leader::plan_strategy+0x47` (seed
/// `340a6de6`) — two at `+0x221` and four at `+0x63d` against ours' two
/// and one. Past run562's window (its last block 11030); widened on run571
/// (block 11183).
///
/// It was **10779** after item 1371 (`docs/COLLISION.md` §5.1, `docs/CARAVAN.md`
/// §7): `do_move`'s waypoint take kills a move whose **final** node is
/// occupied when the action under it is a `TRADE_ROUTE` — `5f8721`'s `cmpl
/// $0xf`, the first of the four the listing tests — where this crate
/// widened the tolerance instead. **The move's value diff** (run547): on
/// block 10391 the caravan `1/52` (`CARA`), its last node under the
/// Citizen `1/87`, `pos` ours (28260,24052) against (28261,24026),
/// `orders.len` 2 against 1, `collide_o` −1 against 87, `tolerance` 144
/// against 96 → agreeing. run547's keys went 156 → 87, nothing parting from
/// 10383 to its last block 10395. Frame 10391's draws went 6 against 7
/// parting at index 0 → agreeing. **The new word's delta: ours 15 draws
/// and the original 14 on frame 10779, parting at index 4**: ours spends a
/// fifth `Leader::upgrade_units+0x5a4 < Leader::production_ai+0x1ca <
/// Leader::plan_strategy+0x47` where the original goes on to `0/3`'s
/// `GameAccess::rnd+0x20 < Unit::do_job+0x67` (seed `8d12ace5`). Past
/// run547's window (its last block 10395), widened on run562 (block
/// 10780).
///
/// It was **10391** after item 1365 (`docs/PRODUCTION.md`, "The tail's first
/// caller"): `train_time`'s speed-upgrade step, `t = (10 − n) × t / 10`
/// with `n` the troops' ladder for a foot or mounted type — and a Citizen
/// is foot. who=1 takes Herbal Lore (`TROOPS_FASTER_1`) on 9782 and its
/// next Citizen, at `1/2022`, queued at the ramp's ceiling of 18,000.
/// **The move's value diff** (run547): on block 10144 `1/2022`'s
/// `queue[0].job_counter` is 16200 on both sides, the original's target
/// and ours 18000; on the word 10144's block 10145 the Citizen `1/89` the
/// original's alone and `1/2022`'s `queue:queued` ours 1 against 0 →
/// agreeing. run547's keys went 1,850 → 156. Frame 10144's draws went 42
/// against 42 parting at index 33 → agreeing. **The new word's delta: ours
/// 6 draws and the original 7 on frame 10391, parting at index 0**: the
/// original spends `Guy::set_anim+0x97a < Unit::set_anim+0x56 <
/// Unit::do_trade+0x40` (seed `10810bb6`) where ours spends
/// `Unit::do_non_flat_gather+0xcc3`; `1/52`'s `pos` parts on block 10391,
/// ours (28260, 24052) against (28261, 24026), its order a move (kind 1,
/// two long) against kind 15. Inside run547's window (block 10392).
///
/// It was **10144** after item 1354 (`docs/VISION.md` §2): `Unit::update_los`'s
/// troops term — `TROOPS_UPGRADE_LOS` per `TROOPS_LOS_n` held, for a unit
/// the Barracks, the Stable or the Auto Plant trains — and `mylos` kept as
/// the cache the original keeps, refreshed by `calc_unit_stats` on the
/// leader pass after `gain_tech`'s `|= 0xc000000`. who=1 takes Herbal Lore
/// on 9782. **The move's value diff** (run529): on block 9784 the Explorer
/// `1/0`'s `mylos` ours 12 against 14, the Elite Longbowmen `1/28`'s 11
/// against 13, the Elite Javelineers `1/38`'s 8 against 10 — 29 keys →
/// agreeing (the term computed live parted them on 9783 instead); the
/// Explorer's route on 9839 (`path:length` 5 against 6, `path[1].to`
/// (26616, 13560) against (26616, 14328)) and its `pos` from 9840 →
/// agreeing; on the word 9999's block 9999 its `orders.len` 0 against 1
/// and `orders_x/y` (28128, 14304) against (28152, 14328) → agreeing.
/// run529's keys went 262 → 114. Frame 9999's draws went 56 against 8 →
/// agreeing. **The new word's delta: ours 42 draws and the original 42 on
/// frame 10144, parting at index 33** (the count parts on 10162): the
/// original spends `Guy::init_real+0x52 < Unit::init+0xb97 <
/// Objects::init_unit+0xbd` (seed `8530fb47`), a birth, where ours spends
/// `1/44`'s `Guy::set_anim+0x97a < Guy::inc_time+0x271`. Past run529's
/// window (its last block 10015), widened on run547 (block 10145).
///
/// It was **9999** after item 1346 (`docs/AI.md` §99.16, `docs/COSTS.md` "The
/// discounts"): who=1 holds Silver, and `get_cost` takes `SILVER_AGE_COST`
/// off an age, so the Gunpowder Age is 382 food and 382 knowledge rather
/// than 450; and `plan_strategy`'s cheap research tick asks
/// `Leader::can_pay(0)` with the head's own escrow flag, so the escrowed
/// head is priced against the whole bucket. The tick of frame 9955 (phase
/// 180) buys it. **The move's value diff** (run529): on block 9956 who=1's
/// `MAKE[0].val` ours 4590000 against 45900 → agreeing, `bucket[0:food]`
/// 402 against 20 → agreeing, `bucket[3:knowledge]` 698 against 316 →
/// agreeing, `escrow[0:food]` 93 against 0 → agreeing, `1/2005`'s
/// `queue:queued` 1 against 2 → agreeing; on the word's block 9983
/// `escrow[1:timber]` 0 against 57 and `MAKE[5].t` −1 against 50 →
/// agreeing. run529's keys went 557 → 314. Frame 9982's draws went 13
/// against 16 → agreeing. **The new word's delta: ours 56 draws and the
/// original 8 on frame 9999, parting at index 0**: ours opens on two
/// `Guy::set_anim+0x97a < Unit::do_idle+0x7d`, a `think_spellcaster+0x413`
/// and 45 `Unit::think_scout`, where the original opens on
/// `Guy::set_anim+0x97a < Unit::set_anim+0x56 < Unit::do_guard+0x7f4`.
/// Inside run529's window (block 10000).
///
/// It was **9982** after item 1338 (`docs/ANIM.md` §11, §4): the Catapult `1/84`
/// is upgraded to a Trebuchet (`TypeIndex` 266) on 9710, and
/// `Unit::set_type` kills its whole crew and seats fresh figures on the new
/// pieces' tracks — none for a Trebuchet's — where this crate kept the
/// Catapult's two tracks, (−120, 0) and (72, 216), and walked them; and a
/// guy still turning on slot `0x15`/`0x16` is not idled, so the Trebuchet's
/// guy 0 rolls on 9755 and 9764 and not between. **The move's value diff**
/// (run529): on block 9759 `1/84`'s `g.track_dx[1]` ours −120 against 0 →
/// agreeing, `g.cur_anim[1]` 8 against 22 → agreeing, `g.cur_time[0]` 1
/// against 4 → agreeing; on the word's block 9765 `g.cur_time[2]` 10
/// against 1 → agreeing (with the crew alone, the word 9756, ours 15
/// against 13: guy 0's roll on its turn). run529's keys went 1,075 → 557.
/// Frame 9764's draws went 10 against 11 → agreeing. **The new word's
/// delta: ours 13 draws and the original 16 on frame 9982, parting at
/// index 0**: the original opens on two `Leader::produce_building+0x1805 <
/// Leader::make_this+0x328 < Leader::make_stuff+0xf6` (seeds `1b6bb976`,
/// `a952625d`) ahead of `make_stuff+0x221`, where ours opens on
/// `make_stuff+0x221`. Inside run529's window (block 9983).
///
/// It was **9764** before that (item 1332, `docs/COLLISION.md` §13.3): the Supply
/// Wagon `1/86` pushes gaia's peacock `8/2` (`HERDPEACOCK`, 413) on tick
/// 9347 in the original, whose stranger refusal (`5fad3c`, `cmpb $8`)
/// spares gaia, and this crate refused gaia and pushed nothing; and the
/// pushed idle unit's guy 0 is turned to the push (`turn_angles(bearing,
/// &out, 1, 1)`, `do_turn`), which this crate left a seam. **The move's
/// value diff** (run511): on block 9348 `8/2`'s `gaia:pos` ours (38712,
/// 20232) against (38711, 20231) → agreeing; on 9349 its `cur_anim` 0
/// against 7 → agreeing; on 9355 `1/86`'s `collide_o` 2 against −1 →
/// agreeing (with the push alone, `8/2`'s `cur_anim` 7 against 0 on 9354,
/// and the word 9354). run511's keys went 1,802 → 159. Frame 9352's draws
/// went 35 against 36 → agreeing. **The new word's delta: ours 10 draws
/// and the original 11 on frame 9764, parting at index 4**: the original
/// spends `Guy::set_anim+0x97a < Unit::set_anim+0xb6 < Unit::do_idle+0x7d`
/// (seed `c84aaf40`), a crew figure's idle roll, where ours goes on to
/// `Guy::set_anim+0x97a < Guy::inc_time+0x271`. Past run511's window (its
/// last block 9574), widened on run529 (block 9765).
///
/// It was **9352** before that (item 1318), ours 35 against 36 at index 0:
/// the original opens on `Guy::set_anim+0x97a < Unit::set_anim+0x56 <
/// Animal::do_idle+0x19` (seed `0e2eda7d`, 55735), an animal's idle roll
/// ours did not spend. Inside run511's window (block 9353): gaia's `8/2`
/// stood a unit apart on 9348, its animation parted on 9349.
///
/// It was **9323** before that (item 1305), ours 7 against 5 at index 1:
/// the AI scout `1/0` collided with the King's Longbowman `1/69` and
/// stood twice (`Unit::move_step+0x823`), because who=1's army near the
/// Senator `1/80` had walked 9112..9262 at its own speed where the
/// original's Forced March lifts every land unit within the hero's radius
/// to `FORCED_MARCH_SPEED`, 42 (`UnitData::speed@0060aae0`), and the
/// group's `+0x4b` lets only those near it report (`docs/AI.md` §99.15).
/// **The move's value diff** (run517, the gap 9032..9323 no dump had
/// printed): see `run517_s_gap_is_widened_whole`; run511 block 9318,
/// `1/69`'s `pos` ours (33412, 19136) against (33056, 18955) → agreeing
/// to 9432; block 9324, `1/0`'s `collide_o` 69 against −1 → agreeing;
/// run511 parts on 1,802 keys where it parted on 2,069, 133 standing on
/// its first block where 571 stood. 9323 was widened on run511 (block
/// 9324), past run500's last block 9037.
///
/// It was **8856** before that (item 1293), ours 34 against 35 at index 0:
/// the original opened on `Unit::think_spellcaster+0x589 <
/// Army::use_generals+0xfc < Army::process+0x8d` — the Senator `1/80`
/// (`TypeIndex` 353, `FROM General`) in who=1's army, moving, throwing the
/// hero arm's coin on the army's 128-frame turn — which this crate held as
/// two seams (`docs/AI.md` §99.13). **The move's value diff**, frame 8856:
/// the coin 16250 (seed `909f318c`) thrown both sides, even, the Senator's
/// own cell, no cast; run500 block 8858, `1/54`'s `g.cur_anim[0]` ours 30
/// against 31 and `g.end_time[0]` 70 against 80 → agreeing; run500 parts
/// on 149 keys where it parted on 737, and on nothing from 8858 to 8962.
/// On the way, 9112: the coin 25911, odd, casts Forced March both sides —
/// a hero's craft plays `CHAR_ATTACK2` with no reroll — and 9240: the
/// march's `leader_flags & 0x8000` holds the next coin back.
///
/// It was **8786** before that (item 1286), ours 10 against 49 at index 1:
/// the squad `1/73`..`1/75`'s follower slot fell in a wood, and the original
/// sent up a flock of three birds — `Unit::do_group_move+0xb03`'s roll,
/// `Objects::add_flock`'s 36 — where ours only ungrouped (`docs/AI.md`
/// §99.12). **The move's value diff**, run500 block 8787: who=1's
/// `flock_stamp` ours 0 against 8786 → both 8786 (compared from this
/// item); the three birds' `set_new_location` in run500's proxy, every
/// frame from 8787 to their landings on 8968, 8977 and 8992 → agreeing;
/// run500 parts on 745 keys where it parted on 1,816, and nothing on
/// 8789.
///
/// It was **8377** before that (item 1275), ours 17 against 19 at index 0:
/// ours' `found_cities` bought who=1's fourth city at once where the
/// original's `make_stuff` went to the market (two `use_market+0x1ed`) and
/// bought nothing. `get_cost` prices a city by every city of the line —
/// Small, Large and Major, built and queued — and this crate counted Small
/// Cities alone: one, against the original's three, so 60 food and timber
/// against 160 (`docs/COSTS.md`, "A city is ramped by every city";
/// `docs/AI.md` §99.11). **The move's value diff**, run491 block 8378:
/// who=1's `MAKE[0].val` ours −1 against 1981477 and `bucket[2:wealth]`
/// 172 against 53 → agreeing; run491 parts on 165 keys where it parted on
/// 530, and on no block past 8277.
///
/// It was **8182** before that (item 1264), ours 8 against 7 at index 2:
/// a `Leader::make_stuff+0x63d` the original did not spend. The Barracks
/// `1/2017` took one King's Longbowman and then the Hoplites where the
/// original took two: the second's 74 wealth was the unit who=1's
/// `leftover` paid in on 8182, parted since 6591, when caravan `1/52`'s new
/// route 2 ↔ 3 had the original's `do_trade` re-sum city 2's delivered
/// route at today's value, 176 → 184 (`docs/AI.md` §99.10). **The move's
/// value diff**: run495 block 6592, who=1's `income[2:wealth]` ours 992
/// against 1000 → both 1000, and the goods agree on every block
/// 6030..7066; run491 block 8183, `num_queued[128]` 2 against 3 and
/// `1/2017`'s `queue[2].type` 132 against 178 → agreeing; run491 parts on
/// 530 keys where it parted on 967.
///
/// **The move 7785 → 8182** (item 1264, `docs/TECH.md` step 8): who=1's Tower is a
/// Keep in the original's `num_buildings` from the Keep's gain (Tower 1,
/// Keep 0 through run476's 6033; Tower 0, Keep 1 from run483's 7065), and
/// this crate kept a Tower, so `create_buildings` saw no Keep and the make
/// list took a Senate ahead of a Mine. **The move's value diff**, run488:
/// on 7785 who=1's `MAKE[2].t` ours 438 against 419 → agreeing; on 7786
/// the Senate `1/2030`'s `x_internal` 38976 against 38784 and `y_internal`
/// 19584 against 17760 → agreeing; run488's keys went 876 → 150. Frame
/// 7785's draws went 23 against 17 → agreeing. **The new word's delta:
/// ours 8 draws and the original 7 on frame 8182, parting at index 2**:
/// ours spends `Leader::make_stuff+0x63d` where the original spends
/// `Guy::set_anim+0x97a < Guy::inc_time+0x271`. Past run488's window (its
/// last block 8036), widened on run491 (block 8183).
///
/// It was **7785** before that (item 1260): run470 parts on frame 7785, **ours 23 draws
/// against the original's 17, at index 4** — both spend two
/// `Leader::use_market+0x1ed` and open a placement
/// (`Leader::produce_building+0x1805 < make_this`), ours three of its draws
/// against two, where the original's next is `Leader::make_stuff+0x221`.
/// Widened on run488 (block 7786).
///
/// It was **7070** before that (item 1251), ours 3213 against 2552 at index
/// 2545: caravan `1/52`'s road from `1/2007` to `1/2022` ran to the budget
/// where the original's arrived, because ours had swept away (148..150, 123)
/// on 6949 — the tile beside the Farm's footprint claimed nothing once its
/// east was zeroed, until `RoadsOut::leech_codes` lent it back (`docs/ROADS.md`
/// §11). **The move's value diff**, run483's trace on 7070: node 1129,
/// tile (150, 123), ours 114 against 37 → both 37, and the search 3204
/// nodes against 2543 → 2543 node for node; run483 parts on 134 keys where
/// it parted on 745.
///
/// It was **5782** before that (item 1241), ours 14 against 13 at index 3:
/// a `Leader::make_stuff+0x63d` the original did not spend, who=1 queuing a
/// Hoplite at `1/2017` its food could not pay for in the original. The food
/// was 36 short from block 4577, where frame 4576's Empire cost 144 here and
/// 108 there: `get_cost`'s library-line tail takes Dye's quarter off a civic
/// epoch (`docs/AI.md` §99.8). **The move's value diff**, run482 block 4577:
/// who=1's `bucket[0:food]` ours 161 against 197 → both 197; run476 block
/// 5783, `1/2017`'s `queued` 2 against 1 → agreeing, `bucket[4:metal]` 7
/// against 43 → agreeing. Widened on run476 (block 5783).
///
/// It was **5376** before that (item 1221), ours 45 against 40 at index 1:
/// four `Leader::produce_building+0x1805` against one, the Granary `1/2023`
/// laid friendless three cells south of the original's, until
/// `find_friends`' enhancer arm counted the farms beside it (block 5377,
/// widened on run471).
pub(crate) const THIRD_WORD_GREAT_SAHARA_TOUGHEST: i64 = 12_569;

/// **The second pair's Great Lakes word** (item 971): run347, run53's game
/// at Toughest (its second take, without `-config`).
///
/// **0 on the capture**: 121 draws against 120 on frame 0, parting at
/// index 24 on the same coin. **0 → 1 on item 971**: frame 0's draws went
/// 120 against 121 → 121 against 121 (no dump of this game held a record
/// of block 1 when it moved). **The new word's delta: ours 54 draws and
/// the original 85 on frame 1, parting at index 44**: ours spends
/// `Unit::do_non_flat_gather+0x54b` where the original spends
/// `Leader::produce_building+0xc99`. **The lower word of the pair**; its
/// block is `run350_s_word_frame_is_widened_whole`'s: who=1's script at
/// step 6 there against 11 here, 92 timber against 28, a tech stamped on
/// frame 1.
///
/// **1 → 3776 on item 979**, a harness fix (`diff::setup::same_lobby`,
/// `docs/AI.md` §81): the walk had installed run12's frame-0 word — the
/// Easiest game's, one draw short of the coin — so frame 1's eight
/// `rand_int(1, 10)` rolls read 9 9 3 4 9 6 3 2 where the trace's own seeds
/// give 9 3 4 9 6 3 2 4, and `defensive`'s `rush_build1` 9 against 3.
/// **The move's value diff (the word's delta, here; its block is
/// `run350_s_word_frame_is_widened_whole`'s):** on block 2 who=1's
/// `script_step` went 11 against 6 → 6 on both, timber 28 against 92 → 92,
/// wealth 50 against 100 → 100, `gatherers` 4 against 3 → 3, citizen
/// `1/2`'s walk and `1/2001`'s 90 gather slots closed, and the site `2007`
/// the rush's second farm now stands on both; frame 1's draws went 54
/// against 85 → 85 against 85, and run350's keys parted 576 → 64. **The
/// new word's delta: ours 223 draws and the original 217 on frame 3776,
/// parting at index 180**: ours spends `Build::find_gather_tiles+0x10a`
/// where the original spends `Animal::think_bird+0x82`. Past run350's
/// window; widened on run355.
///
/// **3776 → 4555 on item 989**, the same close tail as East Indies' 1576
/// (`docs/AI.md` §82). **The move's value diff (the word's delta, here;
/// its block is `run355_s_word_frame_is_widened_whole`'s):** on run355 the
/// word's block 3777 went 127 keys → 0 — the camp `1/2010`'s 79 tile keys
/// and damage 1 against 0, `2007`'s `city_down`, city `2000`'s
/// `gatherers` 7 against 8, citizens `1/6` (`orders_x` 42504 against
/// 41064) and `1/28` — and nothing of who=1 parts before 3805. **The new
/// word's delta: ours 7 draws and the original 3 on frame 4555, parting at
/// index 1**: ours spends `Unit::find_attack_pos+0xea9 < Unit::fight+0xcb4`
/// where the original spends `Farms::inc_time+0x1ae`. **The lower word of
/// the pair**; widened on run356 (`run356_s_word_frame_is_widened_whole`).
///
/// **4555 → 4593 on item 997**: an attack-move's look passes over an
/// unarmed building (`find_melee_target`'s `0x20010`, `docs/COMBAT.md`
/// §64). **The move's value diff (the word's delta, here; its block is
/// `run356_s_word_frame_is_widened_whole`'s):** on frame 4554 `1/21`'s
/// look (`(4554 + 21) % 15 == 0`) had taken the human's Woodcutter's Camp
/// `0/2001` at (4224, 28608) and pushed an `ATTACK` over its `ATTACK_TO`;
/// block 4555's `order:kind` 10 against 2, `orders_x/y` (9005, 29653)
/// against (4824, 34584) and two orders against one → one `ATTACK_TO` to
/// (4824, 34584) on both, and on block 4556 its position (9000, 29640)
/// against (8986, 29671) → (8986, 29671) on both. Frame 4555's draws went
/// 7 against 3 → 3 against 3. **The new word's delta: ours 4 draws and
/// the original 5 on frame 4593, parting at index 2**: ours spends
/// `Farms::inc_time+0x1ae` where the original spends `Guy::set_anim+0xf2f
/// < Guy::move+0x166`. Inside run356's window, whose block 4592 parts on
/// `1/26`'s order (kind 1 here against 10 there).
///
/// **4593 → 4605 on item 1002**: a ranged chase on a building is asked
/// `is_in_range` without the `0x90` margin (`do_move@005f7b30`'s building
/// arm, `5f7f27`–`5f7faa`; `docs/COMBAT.md` §65). **The move's value diff
/// (the word's delta, here; its block is
/// `run356_s_word_frame_is_widened_whole`'s):** `1/26`, a ranged soldier
/// chasing the human's city `0/2000` at `attack_dist` 1128 against a reach
/// of 1158, now stops at (5079, 31628) on block 4592 as the original does,
/// where ours had walked on to (5052, 31644) toward its attack position
/// (4872, 31752); its `order:kind` 1 against 10, `orders_x/y` and three
/// orders against two → one `ATTACK` on both, and on block 4593 its
/// position, `recharging` 33 and `hold_attack` 1 agree. Frame 4593's
/// draws went 4 against 5 → 5 against 5. **The new word's delta: ours 2
/// draws and the original 43 on frame 4605, parting at index 0**: ours
/// spends `Farms::inc_time+0x1ae` where the original spends
/// `Unit::find_attack_pos+0xea9 < Group::action_attack+0x41a` (four, then
/// 37 under `Unit::fight+0xcb4`). Inside run356's window, whose block 4606
/// parts on the army's members' order (kind 21 here against 10 there).
///
/// **4605 → 4618 on item 1012**: an army group's attack-move looks, and
/// hands its find to the group (`do_group_attack_to@005e74e0`,
/// `find_nearby_target`'s add arm, `Group::action_attack`'s `QUEUE_FIRST`);
/// the group retarget searches buildings for a building target (the word
/// 2, `00712490:470`); a ring started on an edge starts mid-face
/// (`601ea0`). `docs/COMBAT.md` §66. **The move's value diff (the word's
/// delta, here; its block is `run356_s_word_frame_is_widened_whole`'s):**
/// `1/15`'s look on 4605 finds the city `0/2000`, and on block 4606 all
/// eighteen members of group 65 hold its `ATTACK` over a re-issued
/// `GROUP_ATTACK_TO` on both sides, where ours had held the bare
/// `GROUP_ATTACK_TO` (kind 21 against 10, one order against two);
/// `1/9`–`1/11`'s target is the city, not the human's scout `0/0`; `1/17`
/// walks to (3912, 30840), not (3864, 31512). Frame 4605's draws went 2
/// against 43 → 43 against 43. **The new word's delta: ours 94 draws and
/// the original 4 on frame 4618, parting at index 0**: ours spends
/// `Guy::set_anim+0x97a < Unit::do_idle+0x7d` where the original spends
/// `Unit::do_non_flat_gather+0xcc3`. Inside run356's window, whose block
/// 4619 parts on `1/0`'s group (66 against 67) and `1/7`'s gather `wait`.
///
/// **4618 → 4673 on item 1014**: `think_scout`'s rival multiplier asks
/// `treaties[L] & 3` (`LeaderData +0x94`, the met bit, `005f6688`), not
/// `diplos` as war; `docs/SCOUT.md` §8.3, read on run360's packet at tick
/// 4506. **The move's value diff (the word's delta, here; its block is
/// `run356_s_word_frame_is_widened_whole`'s):** on 4506 the AI scout `1/0`,
/// at (4320, 28896) on both sides, scores the human's cells undoubled —
/// (4, 41) 97, (2, 40) 82, (3, 42) 80 — and block 4507 prints its target
/// (2808, 32760) in run360, where ours took (2040, 31224) at 116 against
/// 125; on run356 its `orders_x` is 2808 on both sides from block 4550,
/// where ours read 2040, and it agrees to block 4737. Frame 4618's draws
/// went 94 against 4 → agreeing. **The new word's delta: ours 4 draws and
/// the original 3 on frame 4673, parting at index 0**: ours spends
/// `Unit::fight+0x9b0` (the chaser `1/24`) where the original spends
/// `Farms::inc_time+0x1ae`. Inside run356's window, whose block 4673 parts
/// on `1/24`'s order (kind 10 against 1, two orders against three) and
/// 4674 on its `stopped`; its chase spot has parted since block 4617.
///
/// **4673 → 4688 on item 1023**: `check_target_path`'s review, on the
/// sixteen-frame phase, re-aims a ranged chase whose unit target is running
/// away and out of reach of the walk spot (`orders_x/orders_y`) —
/// `repath`, `find_attack_pos`, a `QUEUE_FIRST` move (`5e24e2`–`5e2873`);
/// `docs/COMBAT.md` §67. **The move's value diff (the word's delta, here;
/// its block is `run356_s_word_frame_is_widened_whole`'s):** on 4616,
/// `(4616 + 24) % 16 == 0`, `1/24` chasing the citizen `0/3` at (2736,
/// 31924) re-aims, and block 4617 prints its move at (4104, 31512), dest
/// (4872, 30744), on both sides, where ours kept (4200, 31608), dest
/// (4968, 30840); `1/24`'s rows agree from block 4605 to 4688, its order on
/// 4673 (kind 10 against 1) and `stopped` on 4674 included. Frame 4673's
/// draws went 4 against 3 → agreeing. **The new word's delta: ours 35
/// draws and the original 35 on frame 4688, parting at index 31** (the
/// count parts on 4690, 6 against 4): after `1/24`'s `Unit::fight+0x9b0`
/// on both sides, ours spends `Guy::set_anim+0x97a < Guy::move+0x19f`
/// where the original spends `Farms::inc_time+0x1ae`. Inside run356's
/// window, whose block 4689 parts on `1/24`: `recharging` 0 here against
/// 33, `hold_attack` 0 against 1, `orders_x/y` (3763, 31600) against (3768,
/// 31608).
///
/// **4688 → 4690 on item 1028**: `Object::compare_target`'s RAID arm — a
/// computer's raider (stance 3) weighs a peasant `+900,000`, anything
/// neither a peasant, a caravan nor a combat unit a tenth, and an active
/// building `/ 20` with no ×5; `Group::action_attack` adds a `Build` target
/// to every member with no search; a building's `targeted` decays `/4` on
/// its owner's eighth frame; `docs/COMBAT.md` §68, read on run368's packet
/// at logger frame 4688 and run369's at 4605. **The move's value diff (the
/// word's delta, here; its block is `run356_s_word_frame_is_widened_
/// whole`'s):** on 4688 `1/24`'s re-search scores, on both sides, the
/// citizen `0/3` at 9800 (score 1225), the scout `0/0` at 15 (score 5) and
/// the city `0/2000` at 15 with `targeted` 0, where ours scored 1800, 1032
/// and 1866 with the city's `targeted` 31, and took the scout; block 4689's
/// `recharging` (0 against 33) and `hold_attack` no longer part. Frame
/// 4688's draws went 35 against 35 parting at 31 → agreeing. **The new
/// word's delta: ours 5 draws and the original 4 on frame 4690, parting at
/// index 1**: ours spends `Object::take_damage+0xe1` where the original
/// spends `Farms::inc_time+0x1ae`. Inside run356's window, where nothing
/// parts on block 4691 and the citizen `0/3`'s damage first parts on 4723.
///
/// **4690 → 4781 on item 1034**: `Object::take_damage`'s `city_flags |=
/// 0xe` (`00652561`) when another player hits a building of a city, and
/// `Build::process`'s 200-frame decay on the city building, `0x4` first and
/// then `0x2`, the city heal's veto; `docs/COMBAT.md` §69. ~~Nothing parts
/// on block 4691~~: the human's city `0/2000` had parted since block 4661,
/// healed to `damage` 1/0 here against 2/5 after `1/26`'s first strike on
/// 4657, and to 0 by 4665, so the strike on 4690 was a first wound again.
/// **The move's value diff (the word's delta, here; its block is
/// `run356_s_word_frame_is_widened_whole`'s):** block 4658 prints the
/// city's `city_flags` 18463 (`0x481f`) on both sides, where ours held
/// `0x2`, `0x4` and `0x8` clear, and they agree to block 4806, `0x4`'s
/// clear on 4801 included; the city's `damage` holds 2/5 on both sides
/// from 4658 to 4689, where ours healed it 2/5 → 1/0 → 0/0 on 4661 and
/// 4665. Frame 4690's draws went 5 against 4 → agreeing. **The new word's
/// delta: ours 5 draws and the original 4 on frame 4781, parting at index
/// 0**: ours spends `Unit::fight+0x9b0` where the original spends
/// `Farms::inc_time+0x1ae`. Inside run356's window, whose block 4780 has
/// the citizen `0/4` struck there (`damage_frame` 4779, `damage` 3/5) and
/// not here, and whose block 4781 parts on `0/4`'s order (kind 10 against
/// 1).
///
/// **4781 → 4846 on item 1040**: the Slinger's three release bays (piece
/// 32, `launch::BAYS`, from run17's 27 stones), and `Unit::
/// target_opportunity`'s `on_duty` return (`600863`), a busy unit that is
/// not on duty does not answer a hit; `docs/COMBAT.md` §70. **The move's
/// value diff (the word's delta, here; its block is
/// `run356_s_word_frame_is_widened_whole`'s):** `1/24`'s stone launched on
/// 4775 flies 5 frames on both sides, where ours flew 6 from the unit's own
/// square, so the citizen `0/4` is struck on 4779 on both (`damage_frame`
/// 4779, `damage` 3/5 on block 4780, where ours read 0 and 0/0); `0/4`
/// keeps its `GATHER` and walk on both, where ours pushed an `ATTACK` on
/// 4780 (kind 10, three orders, against a move and two), and it does not
/// part again in the window. `1/26`'s strikes on the city land on 4689,
/// 4716, 4722, 4750 and 4782 on both sides (ours a frame late from 4690
/// until 1040). Frame 4781's draws went 5 against 4 → agreeing. **The new
/// word's delta: ours 8 draws and the original 9 on frame 4846, parting at
/// index 2**: the original spends `Guy::set_anim+0xf2f < Unit::set_anim+
/// 0x56 < Unit::fight+0x19f6`, ours `Guy::set_anim+0x104b`. Past run356's
/// window; run373 is its widening.
///
/// **4846 → 4852 → 4877, still item 1040**: `Unit::fight`'s building arm
/// (`5fe8a7`–`5feb4c`), a building struck square to its side, and the
/// one-in-five retarget's frozen mark (`005fdf68`–`005fdfea`);
/// `docs/COMBAT.md` §70.6 and §70.7. **The moves' value diff (their
/// blocks are `run373_s_word_frame_is_widened_whole`'s):** the Hoplite
/// `1/19` strikes the city from (3432, 30120) on 4846 facing `0x80000000`
/// on both sides, where ours turned to `0x8ec5…` (the centre's bearing),
/// held the swing and struck on 4847; its figure first parts on 4861.
/// `1/24`'s attack slot holds at `cur_time` 32 of 33 on block 4853 on both
/// sides (`unit_masks2` 16 there), where ours wrapped to the idle and
/// rolled; its clock no longer parts. Frames 4846 and 4852 went 8 against
/// 9 and 6 against 5 → agreeing. **The new word's delta: ours 7 draws and
/// the original 8 on frame 4877, parting at index 1**: the original spends
/// `Guy::set_anim+0x97a < Unit::move_step+0x823`, ours `Guy::set_anim+
/// 0x104b`. Inside run373's window (block 4878); the earliest block to
/// part past its standing rows is 4861, the army's tick.
///
/// **4877 → 4924 on item 1052**: `Army::march_to_target`'s origin is the
/// army's point a cell **behind** it, `muster_angle − 0x80000000`
/// (`6f4daa`–`6f4e2b`: the `sub` before the fold is a reversal);
/// `docs/ARMY.md` §23. On 4860 army 0 is engaged: its first move now lands
/// at (3352, 32331) rather than on the city, so the second move's
/// `get_loc` starts the leader's plan at (3996, 30614) and the chain has
/// the original's eight legs. **The move's value diff (the word's delta,
/// here; its block is `run373_s_word_frame_is_widened_whole`'s):** on
/// block 4861 `1/13` stands at (3936, 31125) walking toward (4588, 31018)
/// and `1/16` at (3884, 30135) toward (5007, 30915) on both sides, with
/// `dest` 1 and eight legs apiece, where ours held `1/12`–`1/14` a frame
/// and gave `1/16`–`1/18` a ninth leg 768 west; 121 keys that first
/// parted there agree. Frame 4877's draws went 7 against 8 → agreeing.
/// **The new word's delta: ours 8 draws and the original 7 on frame 4924,
/// parting at index 0**: ours spends `Guy::set_anim+0xf2f < Guy::move+
/// 0x166`, the original `Guy::set_anim+0x97a < Unit::move_step+0x823`.
/// Inside run373's window (block 4925); block 4923 parts on `1/11`'s
/// order (kind 10 against 1).
///
/// **4924 → 4978 on item 1061**: `do_move`'s flank clause
/// (`5f7fbe`–`5f7ff6`): a chase is not ended for being in reach while its
/// unit target, moving, runs from the chaser's own heading
/// (`docs/COMBAT.md` §71). On frame 4922 `1/11` chases `0/1` walking east,
/// `e = 0xd3290000`, `flanking` 2: ours ended the chase, the original
/// walked on. **The move's value diff (the word's delta, here; its block
/// is `run373_s_word_frame_is_widened_whole`'s):** on block 4923 `1/11`
/// stands at (5032, 30200) with its chase on top (three orders) on both
/// sides, where ours held (5046, 30225) under its `ATTACK` (two); on 4924
/// it stands there under a fresh `ATTACK` on `0/1` (`in_range 0`,
/// `new_ord 1`) on both, and on 4925 at (5016, 30216) on `0/2` with
/// `in_range 1`, `new_ord 0`, `recharging` 30 and `hold_attack` 1 on
/// both. Frame 4924's draws went 8 against 7 → agreeing; run373's keys
/// parted 1,093 → 1,052. **The new word's delta: ours 9 draws and the
/// original 8 on frame 4978, parting at index 1**: ours spends
/// `Unit::close+0xcb6`, the original `Farms::inc_time+0x1ae`. Inside
/// run373's window (block 4979), where the citizen `0/2` is dead on ours'
/// side alone (`death:extra`, `hold_frames` 1): ours reads 42 damage
/// against 37, and the gap stands from the window's first block. No
/// mechanism is named.
///
/// **4978 → 5042 on item 1072**: the civilian heal, the last on-map arm of
/// `Unit::process_healing@005e0670`: a worker, caravan, merchant or
/// fisherman captain on an ally's ground takes a point of damage and its
/// fraction off every `CIVILIAN_HEAL_RATE` (45) frames, `(o + frame) % 45
/// == 0` (`docs/COMBAT.md` §72). **The move's value diff (the word's
/// delta, here; its block is `run373_s_word_frame_is_widened_whole`'s):**
/// `0/2` reads 5/5 on block 4841, 4/0 on 4859, 3/0 on 4904, 16/0 on 4949
/// and 39/5 on 4994 on both sides, where ours read 6/10 from 4841 and died
/// on 4978 at 42 against 37; both sides now lose it on frame 5000 (the
/// original's `DEATH_OBJS` `first_frame` 5000). On run356 `0/3` heals on
/// 4722 (3/5 → 2/0, block 4723) on both. Frame 4978's draws went 9 against
/// 8 → agreeing; run373's keys parted 1,052 → 471. **The new word's
/// delta: ours 8 draws and the original 6 on frame 5042, parting at index
/// 0**: ours spends `Ammo::do_damage+0xc59`, the original
/// `Farms::inc_time+0x1ae`. Inside run373's window (block 5043, 202 after
/// its first and 54 before its last); the citizen `0/1` takes 3/5 on the
/// original's side alone on 5040 (2/0 → 5/5 on block 5041, `damage_frame`
/// still 5038). No mechanism is named.
///
/// **5042 → 5066 on item 1081**: `Ammo::init@0067bbf0`'s lead asks the
/// target's order, `UnitData::order_type` into `is_move@0046f050` and then
/// `is_air@0046f000` (`67ce62`-`67ce99`), and leads along its `angle`
/// (`UnitData +0x50`, `67ceba`), where this crate asked the body's
/// `movement.dest` and led along the figure's facing (`docs/COMBAT.md`
/// §74). **The move's value diff (the word's delta, here; its block is
/// `run373_s_word_frame_is_widened_whole`'s):** the citizen `0/1` fled
/// between two legs on 5024, a `FLEE_TO` head with no destination, and
/// `1/11`'s round fired then came down on (6225, 28816) unled, missed it on
/// 5040 and punctured the ground on 5042. Led, it lands on (6497, 28561)
/// and strikes `0/1` at (6260, 28466) on 5040: `0/1` reads 5/5 on block
/// 5041, 8/10 on 5045 and 12/4 on 5077 (`damage_frame` 5076) on both
/// sides, where ours read 2/0, 5/5 and 5/5. Frame 5042's draws went 8
/// against 6 → agreeing; run373's keys parted 471 → 455. **The new word's
/// delta: ours 9 draws and the original 10 on frame 5066, parting at index
/// 0**: ours spends `Guy::set_anim+0xf2f < Guy::move+0x166`, the original
/// `Guy::set_anim+0x97a < Guy::move+0x19f`. Inside run373's window (block
/// 5067, 226 after its first and 30 before its last): the original's
/// `1/13` stands under a fresh `ATTACK` there where ours walks on under
/// its move, which has parted since block 5012. No mechanism is named.
///
/// **5066 → 5075 on item 1086**: `Unit::check_target_path@005e22d0` asks a
/// unit target's vslot `+0x8`, `SubObjectData::is_active` (`flags & 1`),
/// at `5e2429`, and a dead one jumps the flank triple (`je 5e24e8`) into
/// the re-aim: `Object::close` leaves the corpse in its slot, so on every
/// review the chase is re-aimed at where it fell (`docs/COMBAT.md` §76).
/// This crate returned for a target that was not active. **The move's
/// value diff (the word's delta, here; its block is
/// `run373_s_word_frame_is_widened_whole`'s):** `1/13` chases the citizen
/// `0/2`, dead since 5000. On block 5012 its move reads (4872, 29928) via
/// (4872, 31464), tolerance 384 and nine legs, on both sides, where ours
/// kept (4824, 29928) via its detour's (4104, 31992), tolerance 0 and
/// nineteen; (4920, 29928) on 5028, (4968, 29976) on 5044 and (4776,
/// 29880) on 5060 on both. On block 5066 both stand at (4764, 31013),
/// 1,133 from that goal, under the `ATTACK` (`do_move`'s dead-target
/// `0x480` arm popped the walk), where ours walked on at (4702, 31263);
/// on 5067 both take `0/4`. Frame 5066's draws went 9 against 10 →
/// agreeing; run373's keys parted 455 → 461. **The new word's delta: ours
/// 11 draws and the original 10 on frame 5075, parting at index 0**: ours
/// spends `Guy::set_anim+0x97a < Unit::move_step+0x823` (`1/20`), the
/// original `Guy::set_anim+0x97a < Guy::inc_time+0x271`, which ours
/// spends second. Inside run373's window (block 5076, 235 after its first
/// and 21 before its last): `1/20` reads `collide 1` on `1/18` there on
/// ours alone, and `1/10` and `1/11` have ended their chase on ours alone.
/// No mechanism is named.
///
/// **5075 → 5105 on item 1074**: `Unit::do_move@005f7b30`'s action block
/// asks of its `ATTACK`'s target only that the slot be live — `o`/`who`
/// non-negative, `flags & 1`, the `uid` (`5f7ece`–`5f7f11`) — and nothing
/// else reaches the dead target's arm at `5f8221` (`docs/COMBAT.md` §77).
/// This crate asked `valid_target`, whose `is_seen` refused the citizen
/// `0/1` once player 1 had lost sight of it on frame 5075. **The move's
/// value diff (the word's delta, here; its block is
/// `run373_s_word_frame_is_widened_whole`'s):** on block 5076 `1/10` stands
/// at (5043, 29856), `1/11` at (4898, 29588) and `1/18` at (6084, 29076),
/// each under `MOVE, ATTACK, ATTACK_TO` on `0/1`, and `1/20` at (5965,
/// 29093) with `collide 0`, on both sides, where ours read (5028, 29880),
/// (4901, 29616) and (6084, 29077) under two orders and `1/20` at (5928,
/// 29112) with `collide 1` on `1/18`. Frame 5075's draws went 11 against
/// 10 → agreeing; run373's keys parted 461 → 203. **The new word's delta:
/// frame 5105, 11 draws on each side, parting at index 3**: ours spends
/// `Farms::inc_time+0x1ae`, the original `Unit::fight+0x9b0`; on 5106 ours
/// spends 10 against 11. Block 5106 is past run373's window, nine after its
/// last: run396 widens it (`WIDENING_SECOND_GREAT_LAKES_5105`). No
/// mechanism is named.
///
/// **5105 → 5161 on item 1089**: `Unit::find_new_target@005ff6a0` calls
/// `find_melee_target(−1, NULL, 0, 1, 0)`, whose head hands a follower its
/// captain's `ATTACK` target with no search (`005ff9c0:32-91`;
/// `docs/COMBAT.md` §79). `fight`'s invalid-target arm searched. On tick
/// 5066 `1/13`'s target `0/2` was dead: the original took its captain
/// `1/12`'s `0/4` (run400's packet, entered on `1/13`), and ours searched
/// and took `0/5`, whose `targeted` read 2 on both sides. `1/13`'s chase
/// goal (3384, 31464) then refused `1/21`'s spot on 5090 through
/// `find_ordered_collision`, and that was 5105. **The move's value diff
/// (the word's delta, here; its block is
/// `run373_s_word_frame_is_widened_whole`'s):** on block 5067 `1/13` holds
/// `ATTACK 0/4` on both sides, where ours held `0/5`; on 5068 it stands at
/// (4751, 31019) aimed at (2136, 31608), tolerance 384, on both, where ours
/// stood at (4749, 31041) aimed at (3384, 31464), tolerance 0; on 5091
/// `1/21`'s goal is (3336, 31368) with `order:flags` 0 on both, where ours
/// read (3528, 31512) and 16, and on 5092 both stand at (3025, 31501).
/// Frame 5105's draws went 11 against 11 at index 3 → agreeing; run373's
/// keys parted 203 → 156. **The new word's delta: ours 12 draws against
/// the original's 13 on frame 5161, parting at index 3**: the original
/// spends a second `Guy::set_anim+0x97a < Guy::inc_time+0x1ed`, ours
/// `Farms::inc_time+0x1ae`. Inside run396's window (block 5162, 62 after
/// its first and 194 before its last): `0/5` died on 5147, and on block
/// 5162 the original's `1/24` strikes the scout `0/0` (`recharging` 33)
/// and its follower `1/26` holds it, where ours' `1/24` has re-searched
/// onto the city `0/2000` and `1/26` has followed. No mechanism is named.
///
/// **5161 → 5930, the game's end, on item 1099** (`docs/COMBAT.md` §80):
/// `Unit::fight@005fd4d0:413`'s one-in-five re-search is
/// `find_new_target(this, &who, 0)`, which kills the attack before it
/// searches, so the search runs under the attack-move's flags word
/// (`0x20010`) and a building scores half; this crate searched under the
/// attack and re-pointed it in place. And `Build::check_capture`'s tally
/// asks `Search::valid_filter(…, 8)` of every object, which passes only
/// what is armed (`0067de47`); this crate counted the human's unarmed
/// buildings at 7 apiece and never took its city. **The word's delta (its
/// block is `run396_s_word_frame_is_widened_whole`'s):** on block 5162
/// `1/24` strikes the scout `0/0` (`in_range` 1, `recharging` 33,
/// `hold_attack` 1) and `1/26` holds `0/0`, on both sides, where ours read
/// the city `0/2000`, `recharging` 0; frame 5161's draws went 12 against
/// 13 → agreeing. **No frame of run347's trace parts**: the draw stream
/// agrees to 5930, its last, the frame the AI takes the human's city
/// `0/2000` as `1/2017` and the human is defeated. The value diff at the
/// end is run347's own closing whole-map state, block 5931: 42 units
/// compared, 0 off, 0 unlinked, 0 extra; every building linked, 0
/// diverged, the seven the capture hands over (`1/2017`..`1/2023`, at
/// ours' `0/2000`..`0/2006` points) among them, where ours held all seven
/// as the human's; three cities unlinked, the closing `CITY` record
/// carrying no `o` (`run347_is_great_lakes_at_toughest_and_its_word_holds`).
/// run403 widens the last blocks before it
/// (`WIDENING_SECOND_GREAT_LAKES_5930`). **The pair's lower map closes.**
pub(crate) const SECOND_WORD_GREAT_LAKES: i64 = 5_930;

/// `run349_s_word_frame_is_widened_whole`'s window (item 971): run349
/// whole, blocks 1..250 over run346's game. The word is frame 0, which
/// writes block 1, and no block stands before the start dump; the window
/// opens at −1 only so the word sits strictly inside it, as
/// `the_widening_behind_each_pinned_word_exists` asks.
pub(crate) const WIDENING_SECOND_EAST_INDIES: (i64, i64) = (-1, 250);

/// `run350_s_word_frame_is_widened_whole`'s window (item 971): run350
/// whole, blocks 1..250 over run347's game; the word is frame 1, which
/// writes block 2.
pub(crate) const WIDENING_SECOND_GREAT_LAKES: (i64, i64) = (0, 250);

/// `run352_s_word_frame_is_widened_whole`'s window (item 979): run352 over
/// run346's game, blocks 1571..1827 — six blocks before the word 1576's
/// block 1577 and 250 past it.
pub(crate) const WIDENING_SECOND_EAST_INDIES_1576: (i64, i64) = (1_571, 1_827);

/// `run355_s_word_frame_is_widened_whole`'s window (item 979): run355 over
/// run347's game, blocks 3771..4027 — six blocks before the word 3776's
/// block 3777 and 250 past it.
pub(crate) const WIDENING_SECOND_GREAT_LAKES_3776: (i64, i64) = (3_771, 4_027);

/// `run357_s_word_frame_is_widened_whole`'s window (item 989): run357 over
/// run346's game, blocks 5601..5857 — six blocks before the word 5606's
/// block 5607 and 250 past it. **Since item 1106 the word is 5773**, block
/// 5774, inside it (173 blocks after its first and 83 before its last).
pub(crate) const WIDENING_SECOND_EAST_INDIES_5606: (i64, i64) = (5_601, 5_857);

/// `run414_s_word_frame_is_widened_whole`'s window (item 1115): run414 over
/// run346's game, blocks 5970..6226 — six blocks before the word 5975's
/// block 5976 and 250 past it. **Since item 1120 the word is 6151**,
/// block 6152, inside it (182 blocks after its first and 74 before its
/// last).
pub(crate) const WIDENING_SECOND_EAST_INDIES_5975: (i64, i64) = (5_970, 6_226);

/// `run419_s_word_frame_is_widened_whole`'s window (item 1127): run419 over
/// run346's game, blocks 6316..6572 — six blocks before the word 6321's
/// block 6322 and 250 past it.
pub(crate) const WIDENING_SECOND_EAST_INDIES_6321: (i64, i64) = (6_316, 6_572);

/// `run420_s_word_frame_is_widened_whole`'s window (item 1143): run420 over
/// run346's game, blocks 6604..6860 — six blocks before the word 6609's
/// block 6610 and 250 past it. **Since item 1156 the word is 6743**, block
/// 6744, inside it (140 blocks after its first and 116 before its last).
pub(crate) const WIDENING_SECOND_EAST_INDIES_6609: (i64, i64) = (6_604, 6_860);

/// `run425_s_word_frame_is_widened_whole`'s window (item 1164): run425 over
/// run346's game, blocks 7377..7633 — six blocks before the word 7382's
/// block 7383 and 250 past it; the word 7512's block 7513 since item 1174.
pub(crate) const WIDENING_SECOND_EAST_INDIES_7382: (i64, i64) = (7_377, 7_633);
/// **run439's window** (item 1185): the second pair's East Indies at
/// run425's detail, blocks 8514..8770 — six blocks before the word 8519's
/// block 8520 and 250 past it.
pub(crate) const WIDENING_SECOND_EAST_INDIES_8519: (i64, i64) = (8_514, 8_770);

/// **run445's window** (item 1191): the second pair's East Indies at
/// run439's detail, blocks 8815..9071 — six blocks before the word 8820's
/// block 8821 and 250 past it; the word 8907's block 8908 since item 1197.
pub(crate) const WIDENING_SECOND_EAST_INDIES_8820: (i64, i64) = (8_815, 9_071);

/// **run462's window** (item 1214): the second pair's East Indies at
/// run445's detail, blocks 10178..10434 — six blocks before the word
/// 10183's block 10184 and 250 past it; the word 10185's block 10186 since
/// item 1228.
pub(crate) const WIDENING_SECOND_EAST_INDIES_10183: (i64, i64) = (10_178, 10_434);

/// **run480's window** (item 1243): the second pair's East Indies at
/// run462's detail, blocks 10980..11236 — six blocks before the word
/// 10985's block 10986 and 250 past it.
pub(crate) const WIDENING_SECOND_EAST_INDIES_10985: (i64, i64) = (10_980, 11_236);

/// **run490's window** (item 1264): the second pair's East Indies at
/// run480's detail, blocks 11323..11579 — six blocks before the word
/// 11328's block 11329 and 250 past it.
pub(crate) const WIDENING_SECOND_EAST_INDIES_11328: (i64, i64) = (11_323, 11_579);

/// **run506's window** (item 1297): the second pair's East Indies at
/// run490's detail, blocks 11632..11888 — six blocks before the word
/// 11637's block 11638 and 250 past it.
pub(crate) const WIDENING_SECOND_EAST_INDIES_11637: (i64, i64) = (11_632, 11_888);

/// **run508's window** (item 1302): the second pair's East Indies at
/// run506's detail, blocks 12577..12833 — six blocks before the word
/// 12582's block 12583 and 250 past it.
pub(crate) const WIDENING_SECOND_EAST_INDIES_12582: (i64, i64) = (12_577, 12_833);

/// **run523's window** (item 1326): the second pair's East Indies at
/// run508's detail, blocks 13380..13636 — six blocks before the word
/// 13385's block 13386 and 250 past it.
pub(crate) const WIDENING_SECOND_EAST_INDIES_13385: (i64, i64) = (13_380, 13_636);

/// **run535's window** (item 1341): the second pair's East Indies at
/// run523's detail, blocks 14136..14392 — six blocks before the word
/// 14141's block 14142 and 250 past it.
pub(crate) const WIDENING_SECOND_EAST_INDIES_14141: (i64, i64) = (14_136, 14_392);

/// **run544's window** (item 1351): the second pair's East Indies at
/// run535's detail, blocks 15857..16113 — six blocks before the word
/// 15862's block 15863 and 250 past it.
pub(crate) const WIDENING_SECOND_EAST_INDIES_15862: (i64, i64) = (15_857, 16_113);

/// **run572's window** (item 1383): the second pair's East Indies at
/// run544's detail, blocks 16155..16411 — six blocks before the word
/// 16160's block 16161 and 250 past it.
pub(crate) const WIDENING_SECOND_EAST_INDIES_16160: (i64, i64) = (16_155, 16_411);

/// **run579's window** (item 1418): the second pair's East Indies at
/// run572's detail, blocks 16476..16733 — six blocks before the word
/// 16482's block 16483 and 250 past it.
pub(crate) const WIDENING_SECOND_EAST_INDIES_16482: (i64, i64) = (16_476, 16_733);

/// run583, item 1427: the 16760 word and its successor 16762, whole records.
pub(crate) const WIDENING_SECOND_EAST_INDIES_16760: (i64, i64) = (16_754, 17_011);

/// run585, item 1434: the 17507 word and its successor, whole records.
pub(crate) const WIDENING_SECOND_EAST_INDIES_17507: (i64, i64) = (17_501, 17_758);

/// run588, item 1436: whole records and projectiles around the new word.
pub(crate) const WIDENING_SECOND_EAST_INDIES_17785: (i64, i64) = (17_754, 17_818);

/// run589, item 1437: whole records and projectiles at the successor.
pub(crate) const WIDENING_SECOND_EAST_INDIES_17907: (i64, i64) = (17_890, 17_954);

/// run594, item 1439: the successor through the natural ending.
pub(crate) const WIDENING_SECOND_EAST_INDIES_18076: (i64, i64) = (18_060, 18_142);

/// `run421_s_gap_is_walked_whole`'s window (item 1156): run421 over
/// run346's game, blocks 6567..6610 — the gap 6573..6603 between run419's
/// last block and run420's first, where the word 6609's `1/28` first parted,
/// with six blocks of run419 before it and run420's first seven after.
pub(crate) const GAP_SECOND_EAST_INDIES_6573: (i64, i64) = (6_567, 6_610);

/// `run356_s_word_frame_is_widened_whole`'s window (item 989): run356 over
/// run347's game, blocks 4550..4806 — six blocks before the word 4555's
/// block 4556 and 250 past it.
pub(crate) const WIDENING_SECOND_GREAT_LAKES_4555: (i64, i64) = (4_550, 4_806);

/// `run373_s_word_frame_is_widened_whole`'s window (item 1040): run373 over
/// run347's game, blocks 4841..5097 — six blocks before the word 4846's
/// block 4847 and 250 past it. The word moved to 4877 (block 4878) inside
/// it, in the same item, to 4924 (block 4925) on item 1052, and to 4978
/// (block 4979) on item 1061, and past it, to 5105, on item 1074.
pub(crate) const WIDENING_SECOND_GREAT_LAKES_4846: (i64, i64) = (4_841, 5_097);

/// `run396_s_word_frame_is_widened_whole`'s window (item 1074): run396 over
/// run347's game, blocks 5100..5356 — six blocks before the word 5105's
/// block 5106 and 250 past it.
pub(crate) const WIDENING_SECOND_GREAT_LAKES_5105: (i64, i64) = (5_100, 5_356);

/// `run403_s_word_frame_is_widened_whole`'s window (item 1099): run403 over
/// run347's game, blocks 5925..5930, the last six before the game's end.
/// The word is 5930, run347's last traced frame, and its own block 5931 is
/// the game's closing whole-map state, which run347 carries and
/// `walk_second` scores; run403's quit block prints no record.
pub(crate) const WIDENING_SECOND_GREAT_LAKES_5930: (i64, i64) = (5_925, 5_931);

/// The frame East Indies' **first scholar** — `1/22` — is seated inside
/// its university, one of the fourteen `Unit::go_inside+0x280` draws in
/// run54's 24,000 frames (item 338) and the one frame in the game where a
/// capture carries the seated scholar's own animation clock: run98's
/// `[7879, 8788]` window spans it, and its `GUY` block reads `cur_anim
/// 25, cur_time 1, end_time 30`.
///
/// That is `Guy::set_anim`'s scholar arm by value rather than by
/// arithmetic — slot 25 is `variant 0 + 0x19`, the first of
/// `SCHOLAR`'s four `Scholar Teach` files — and it is what
/// `run98_s_window_clocks_are_the_original_s` pins (`docs/ANIM.md`
/// §4.11).
pub(crate) const EAST_INDIES_FIRST_SCHOLAR: i64 = 8_466;

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
/// **7679 until the `get_loc` seam closed, 2026-09-17** (items 312 and
/// 314). The site above is a **blocked** step and not the walk (ANIM §4's
/// caller table), so the frame was a position question: on run89 the AI
/// squad `1/37`, `1/38`, `1/39` parts on 7675, the block after a
/// `GROUP_ATTACK_TO`, because the leader's chain carries one extra head
/// waypoint from an identical start and goal. run92 — captured for this,
/// `GROUPS` over [7670, 7686) with `DEATHS` off — says the head is the
/// seam and not `find_wpath_from`: on 7674 group 64 carries
/// `(ox, oy) = (36303, 23348)`, 6981 from the leader at (43174, 24583) so
/// `get_loc`'s first arm refuses, while the order's own destination
/// (36600, 23400) is 301 away and inside the `0x180` the second arm
/// allows. The original's group location is therefore (42877, 24531) —
/// cell (55, 31) against the leader's own (56, 32), one diagonal cell,
/// and exactly the cell of the waypoint this crate emitted. Both
/// `vector_dist` operand pairs came off the listing; the decompiler loses
/// them. `MoveOrder +0x4`/`+0x8` is `x`/`y` by the type record and not
/// `orig_x`/`orig_y` — GROUPS §6.3 said "origin" and was wrong.
/// **7930 until `do_marching`'s empty-target arm, 2026-09-17** (item 317).
/// 7930 is the **first** `Army::find_target` draw in the whole 24,000-frame
/// game — dated off run53's own trace with no capture — and every one after
/// it falls on `frame % 256` in {250, 248}, ARMY §5's army slot.
/// `Army::process` re-reads status between its dispatch ifs, so
/// `do_mustering`'s `status = 2` reaches `do_marching` in the **same** tick
/// still holding `target_o = -1`, and the original's
/// `if (iVar4 < 0) goto LAB_006f3fb2` makes an empty target the retarget
/// branch. One clause — `retarget = target.is_none()` — and the frame
/// agrees ten draws for ten, entry for entry. run92's `GROUPS` window,
/// taken for item 314 and never read for this, is what killed the muster
/// hypothesis: group 64 is num 12 on 7673/7674 and 15 on 7676, matching
/// this crate's `num_standard` exactly, so both sides release on the same
/// tick. Two of the ten draws were unnamed on **both** sides until this
/// item — `rondata::trace` carried neither `006f6dc0` nor `006f718f`, so
/// the sequence word had been comparing a symbol against a hex string.
/// **8030 for one item, and the frame was a whole subsystem returning
/// zero** (item 319). The extra `Unit::do_move+0xe84` was not the army's
/// at all but the AI scout `1/0`'s: [`sim::Sim::scout_danger`] answered a
/// flat 0 behind a comment claiming `World::danger_half` was keyed
/// differently, and it is keyed identically — the grid itself has been
/// diff-backed against run64 the whole time. run94's block 8002 is the
/// value diff: the original sends the scout to (4344, 32760), cell
/// (5, 42), where this crate sent it to (2808, 31992), cell (3, 41), with
/// frame 8001 spending the same 38 draws on both sides entry for entry.
/// The word moves **one frame**, which is what a dense region looks like:
/// the successor is already dated and asserted at 8002.
/// **8031 until a building's own memory was parsed** (item 322). The
/// writer nothing had named is `Wall::check_ever_seen@0063ce70` →
/// `Wall::update_local_seen@0063ed50`: a building ors the **current**
/// line-of-sight plane over its footprint into `WallData::ever_seen`
/// every eighth frame (`frame & 7 == who`, `Wall::process`'s first
/// statement), and the first time a non-owner leader appears there it ors
/// `ever_seen | visible | (1 << who)` into `seen2` over that footprint
/// grown one tile each way. Not `visible`, not a disc, not
/// `update_seen(0)` — and the dump had printed it all along: `BUILDDATA`
/// writes `ever_seen`/`ever_seen_completed` under `BUILDS=1` and nothing
/// had ever parsed them. At block 8002 exactly two of player 0's seven
/// buildings read 3 — the capital `0/2000` at half-cell (8,80) and
/// `0/2001` at (11,74) — and those are the centres of the two patches
/// item 320 could not account for. `VISION.md` §6.1.
///
/// **The figure is 8182 and not 8186**, and the difference is this
/// constant's own contract: `run53_…_put_the_ceiling_where_run33_did`
/// asserts it as a floor on the **word and the sequence both**, and 322's
/// landing parts them — the draw *count* first differs at 8186 (ours 6
/// against 54, at index 4 on an unnamed `602129`) while the draw
/// *sequence* first differs at 8182 (`Guy::set_anim+0x97a <
/// Unit::do_idle+0x7d` against `Leader::make_stuff+0x63d`). A floor is
/// the lower of the two or it is not a floor, so the gain over item 319's
/// 8031 is **+151**. The worker reported 8186 in good faith; taking it
/// unread would have pinned a ceiling this tree does not reach, and the
/// test said so on the first run.
///
/// **8182 → 8186 on item 323**, which closes the gap the paragraph above
/// describes: the sequence now parts where the count does.
/// `civilian_value`'s scholar gate is
/// `bucket[knowledge] <= (resource_cap[food] / 16) * 3 / 2`
/// (`create_units@006c40a0:1122-1127`, both operands straight out of
/// `LeaderDataEncrypt` — `+0xc` is `bucket[3]`, `+0x30` is
/// `resource_cap[0]`, settled by the type record), and this crate called
/// `Sim::mod_resource_cap`, which halves on Easiest. 2000 → 1000 turned
/// the threshold 187 into 93 against a stockpile of 159, so the AI's
/// Scholar was never offered at sim-frame 8180, `make_stuff` bought no
/// slot at 8182, and step 6's expiry walk never spent
/// `Leader::make_stuff+0x63d`. `LeaderData::get_mod_resource_cap` is
/// called exactly **twice** in that whole function, :1229 and :1232, and
/// both are the citizen branch's — an exhaustive enumeration inside the
/// function rather than a sample. `docs/AI.md` §38.
/// **8186 → 8187 on item 328**, which implemented `find_attack_pos`
/// (COMBAT §17) against item 324's own oracle. 8186 now agrees **entry
/// for entry** and the value diff is all six of run19's destinations
/// exact — `1/27` (4344, 29736), `1/28` (4440, 29880), `1/29`
/// (4200, 29496), `1/40` (2424, 30888), `1/41` (2280, 30888), `1/42`
/// (2568, 31320) — from a 46-draw chase split 15+12+11+2+2+4. One frame
/// for a whole mechanic, because 8187 was already behind it.
///
/// **Building it is what found the arithmetic.** 324 specified §17 from a
/// single reading and declined to implement; the implementation then
/// found four errors no reader had — the far arm's `(range+2)*0xc0`
/// needing a *unit* target and so unreachable for buildings, a missing
/// `+ big_radius − 0x30` under `range < 10`, `+0x244` being `big_radius`
/// rather than block radius, and `find_building`'s `0x200` being the flag
/// word and not a radius (read as a radius it found no farm at all). The
/// last five draws were `find_ordered_collision`'s pass over the unit's
/// own group members, skipped here under a seam whose premise had
/// expired: "every unit in every capture so far is ungrouped" stayed
/// true-looking for a month after this probe stopped it being a reason
/// (`docs/COLLISION.md` §9).
/// **8187 → 8201 on item 329, and the crossover.** 8201 is past East
/// Indies' 8193, so this stops being the lower map and the queue's
/// "lower map first" order changes hands for the first time.
///
/// Two causes, not one. `Unit::fight@005fd4d0`'s tail re-enters
/// `Unit::work` through vtable `+0x188` (latched on the action order's
/// `0x10`), so a chase **plans and walks on the frame it is ordered** —
/// the dump's `STACK<TYPE>` block has all six of the probe's units
/// holding path stacks at the end of 8186. And `astar_path`'s unit-grid
/// failure roll (`retry`, `MoveOrder +0x1c`) is what spares `find_upath`
/// from killing `1/28`'s chase. The value diff is `1/28`'s retry 8/7/6
/// and safe 30/29/28, frozen at (36456, 23592), with `1/27` and `1/29` on
/// run19's coordinates for all three frames.
///
/// Shared behaviour on both halves: every chase in the game now plans a
/// frame earlier, and every failed 48-grid search under a transit order
/// delays rather than cancels. `docs/ORDERS.md` §7.10 and
/// `docs/PATHFINDER.md` §21.
///
/// **8272 → 8374 on item 338, and the two counters meet.** 8272 is the
/// game's **first scholar**, and both halves of the difference are one
/// mechanic. `Unit::go_inside@0061a2e0`'s tail is gated on
/// `ObjectData::is_scholar` (`UnitTypeData +0x4` in `0x34`/`0x35`) and
/// snaps the unit onto its host, faces it to angle 0 and calls
/// `set_anim(CHAR_DEFAULT, 1, 1)` — a **fourteenth** caller of
/// `Guy::set_anim+0x97a`, which [`crate::trace::SITES`] had never carried.
/// The forced idle spends its roll in the container's own phase *and*
/// leaves the new guy a real `end_time`, so the wrap this crate used to
/// spend in `Objects::inc_time` never fires: one draw replaced one draw,
/// which is why the **count** never parted here and only the sequence did.
/// Then `Build::train`'s own scholar arm — a scholar trained at a
/// university stays in it, `check_gatherers` rather than `come_out`,
/// unless `BuildData::gather_max` (`+0x80`) is exceeded — without which
/// the same unit `1/44` walked back in and seated itself a second time on
/// 8285.
///
/// **The value diff is run80's**, Great Lakes 23960–24001, 15,700 frames
/// past the word: block 24001 has fourteen scholars sitting on their two
/// universities' exact points. `1/44` — this item's own unit — stood
/// `(24, 552)` off it, the exit ring, and now agrees exactly at
/// `(40416, 25248)`, which is building `1/2019`'s own position; `1/45`,
/// `1/48`, `1/49`, `1/50` and `1/56` join it, six of the eleven compared
/// where none agreed before. The four still out are off by exactly
/// `±(768, 9984)`, which *is* `1/2020 − 1/2019`, so the seating arithmetic
/// is right and which university a scholar walks to is the successor.
/// `docs/CITIES.md` §6.5.2.
///
/// **8374 → 8382 on item 340**, and the same scholar paid for it twice.
/// 8374 is the frame `1/44`'s seating animation ends, and the original's
/// is not an idle at all: `Guy::init_real@005db6b0`'s last statement sets
/// `guy_flags & 0x80` for a `TypeIndex` `0x34`/`0x35` guy, and
/// `Guy::set_anim@005da300`'s own arm on that bit turns the idle roll's
/// variant into an **offset** — `variant + 0x19` for the head of the
/// host's inside chain, `variant + 0x1d` for everyone under it. Those are
/// `SCHOLAR`'s four `Scholar Teach` files, and slot 27 is **103** frames
/// against `CHAR_IDLE1`'s 232: exactly 8272 + 103 − 1.
/// The variant was wrong too — `ObjectData::is_peasant@0046d310` is
/// `UnitTypeData +0x4` in `{0x32, 0x33}` and this crate read it as *any*
/// worker, so the scholar took the peasant-on-a-masked-tile collapse to
/// `IDLE1` where the original's `787 % 100 = 87` gives `IDLE2`; slot 26
/// is 100 frames and only slot 27 lands on 8374. `docs/ANIM.md` §4.11.
///
/// **8382 → 8404 on item 344**, and the frame is the game's first
/// **mine**: 46 draws against 865. `BuildTypeData::calc_gather@00639e40`
/// sends `0x1a3` past the circle walk before it starts, so a mine's site
/// is refused on the distance to the nearest **mountain tile** —
/// `MountainsData::find_nearest@0089cd30`, against `gather_radius * 0xc0`
/// — and not on the camp's cell survey, which passed eighteen of
/// `produce_building`'s friendless candidates where the original passed
/// nine. Then `find_gather_tcoords@0063bdc0`'s metal arm takes the
/// range's tiles whole: 244 in the component, 37 of them
/// `SURFACE_FOREST`, **207** left, and `4 × 207` shuffle draws this crate
/// had never spent. The mine lands at `(41088, 25920)`, the original's own
/// position for `1/2021`, and `great_lakes_first_mine_lists_its_mountain_range`
/// pins the list against run80's own record. `docs/ECONOMY.md`, "The
/// mine's range".
///
/// **8404 → 8464 on item 346: the scholar's teach slot does not
/// restart.** 8404 is `1/44`'s first wrap on slot 25 — `Scholar Teach1`,
/// thirty frames, entered at the 8374 wrap — and the original spends a
/// second `Guy::set_anim+0x97a < Guy::inc_time+0x271` on it that this
/// crate did not. run97 is what settled it: Great Lakes' first value
/// window over `[8029, 9348]`, and `1/44`'s slot over its 1,320 blocks
/// goes 27→25→25→28→27→25→25→25→27→25→…→26, which a
/// `set_anim(same, 0, 1)` restart cannot produce.
/// [`sim::anim::Sim::seated_scholar`], `docs/ANIM.md` §5.1.
///
/// **8582 was the AI's first market draw** (item 348). `Leader::make_stuff`
/// calls `use_market` first thing, and `use_market@006c91c0+0x1ed` is the
/// one `game_random` step in the whole market — the offset its sell
/// rotation starts from. It fires on 34 frames of run53 and on none below
/// 8582, because the ability is **Coinage** (`BUY_SELL`'s own `PREQ0`,
/// Commerce 2) and not the Market building (Barter, Commerce 1) this crate
/// had been reading it off. A wealth shortfall enters the sell branch
/// without a price, so the draw is exact where it is taken.
/// [`sim::ai_make::SITE_MARKET_SELL`], `docs/AI.md` §40.
///
/// **8628 since item 350**, which is `Unit::do_idle` again and a different
/// unit: 8619's extra idle draw was `1/39`'s, and `1/39` was standing at
/// its destination nine frames early because the AI army it marches with
/// held **fifteen** units where the original's holds nine.
/// `Groups::push_group` kills each member out of the group it was in
/// (`docs/GROUPS.md` §3.2), so §12's probe at 8186 takes its six out of
/// the army for good — and 8442's retarget then turns six units the
/// original leaves walking. `docs/ARMY.md` §3.4.
///
/// **8663 since item 352**, which closed 8442 itself: the army's muster
/// was cell (50, 27) against the original's (58, 29), and two
/// `find_muster_spot` predicates were why. The ring's same-owner spacing
/// test is `vector_dist <= 4` (`6f633a`: `cmp $4` / `jle`), not `< 4`;
/// and `BuildType::mask_me`'s **first** write — `W.flags |= 0x4000` on
/// the building's own cell — was in `docs/CITIES.md` §3.6 from the first
/// reading and in no code, so the ring scored every cell of the AI's own
/// town as open ground. With both, run97's walk-slot band is **empty**
/// and every one of the nine units stands on the original's own
/// coordinates on 8442. `docs/ARMY.md` §13 and §16.9.
///
/// **8985 since item 354**, which put `find_wpath`'s `army` mode on for
/// an AI army: `UnitData::is_attacking` calls the current order's `+0x18`
/// virtual, and that slot is a bare `return 0` in every one of the
/// seventeen order vtables the executable ships, so the clause that
/// turned the mode off cannot fire. `docs/PATHFINDER.md` §22.
///
/// **9182 since item 360**, which gave `Unit::move_step`'s *arrival*
/// arm its own collision block. 9134 was one draw on each side and the
/// original's was a bare `5dac7a` — `Guy::set_anim+0x97a` under
/// `Unit::move_step+0x4e2`, a second `set_anim(CHAR_DEFAULT)` call the
/// trace's table did not name and this crate did not have. The snap
/// arm's block resolves nothing and widens nothing: it consumes the
/// waypoint where the unit stands. `docs/COLLISION.md` §5.2.
///
/// Before that, **9134 since item 358**, which gave the AI the market's trade. The
/// word sat on a `Leader::use_market+0x1ed` this crate spent where the
/// original spent a third `Leader::make_stuff+0x221`, and run97's
/// `BUILDDATA` — widened whole, every building, every frame of the
/// window — said why in one line: the original's University queues two
/// more scholars on 8985 than this crate can pay for. The wealth is a
/// **timber sale** on 8982, which `docs/AI.md` §40 had read as a refusal
/// because it cost only one draw. One draw is also what a sale that
/// covers the need looks like. `docs/ECONOMY.md` §12.
///
/// **9451 since item 389**, which took a unit's shot out of `Unit::fight`.
/// 9415 was five draws against two, and two of ours were
/// `combat::scatter_point` inside `fire_ammo`: this crate's `1/29`
/// launched an arrow at the building `0/2004` on the frame it came into
/// range. The original launches nothing there — its **first**
/// `Objects::add_ammo` in all 24,000 frames is 9425, and the trace's own
/// chain says why: `Ammo::init+0xcd9 < Objects::add_ammo+0x119 <
/// GraphicEvents::execute_game_events+0x40d`. A unit's arrow is added by
/// its attack **animation's** release event, and the frames are in the
/// install — `unit_graphics.xml`'s `<RELEASEEVENT starttime=>`, at
/// `ms × 3 / 200` truncated. `docs/COMBAT.md` §9.0.
///
/// The thirty-six frames it bought are a real fight: 9425 and 9426 are
/// the two archers' first arrows, 9439 and 9444 their second, and every
/// frame between agrees draw for draw. ~~**9451 is the first one landing**
/// — two draws at `Object::take_damage+0xe1` this crate does not spend.~~
///
/// **9451 → 9510 on item 396**, and the two draws were a *launch point*.
/// The original adds the release node's own world position to the guy's
/// before `Objects::add_ammo`, so `1/28`'s `CHAR_ATTACK2` shot flew 27
/// frames here against 26 there and its arrow arrived a frame after the
/// draw it owed. run109 (`AMMO=5` over `[9420, 9480)`) prints the arrow's
/// own `sx, sy`, so the offset is measured rather than derived:
/// `docs/COMBAT.md` §22, and `crates/sim/src/launch.rs` is the table.
///
/// The fifty-nine frames it bought are the rest of that fight — **sixteen
/// launches below the word against four**, of which the first nine have
/// their launch point, landing point and flight time checked against
/// run109's own record
/// (`run109_says_great_lakes_s_launches_land_where_the_bow_hand_aims`);
/// the last seven are past run109's window, so the trace's draw sites are
/// all that speaks to them. ~~**9510 parts on `Guy::init_real+0x52`**, a
/// unit coming into existence, which is not this mechanic at all.~~
///
/// **9510 → 10161 on item 442**, and the unit coming into existence was
/// the sixth scholar after all — §47's own reading, closed six items
/// later. The AI's Scholar offer on 9380 is now the original's own
/// number in **both** of its cities, 4,891,136 and 5,755,741 off run114's
/// `RON_LEADER_PROBE` trace (`docs/AI.md` §52.2, §53), and it takes two
/// corrections that had to land together:
///
/// - `City::count_gather_slots` subtracts `BuildData::num_gatherers`,
///   which counts a University's **seated** scholars as well as the
///   `gather_down` chain, so `k` is 6 and 3 rather than 7 and 7 (§51);
/// - and the arm's third multiply is an **independent `if`**, not an
///   `else if` — the listing over `006c528c..006c52f5` — so a city below
///   two thirds takes ×60 ×10 ×5 (§53.1).
///
/// Either alone is worse than neither: the first clamps both cities to
/// §45's 9,999,999 and costs **925 frames** (item 438 measured it and
/// declined to land it), the second leaves the two cities tied on one
/// number and §50's duplicate purchase standing. Six hundred and
/// fifty-one frames, and the first Great Lakes word past ten thousand.
///
/// ~~**10161 parts on `Guy::set_anim+0x97a`**, seven draws against eight —
/// this crate reaches it from `Guy::inc_time+0x271` where the original
/// reaches it from `Unit::move_step+0x4e2`. An animation clocked off the
/// wrong caller, which is not the AI at all.~~ Item 448's widening read
/// the block whole and the animation was **downstream**: the original
/// refused a step this crate took, and §5.4's snap arm rolled the idle.
///
/// **10161 → 10232 on item 456**, and the arm is named by run116, the
/// first `RON_COLLIDE_PROBE` capture (`docs/COLLISION.md` §9). §4.3's
/// group-mate soft arm declines when the *collider* holds a suspended
/// 48-grid search — `UnitData +0x104`, which the PDB calls `openlist` —
/// and `1/31` suspends one on that very frame, which the trace's own
/// `astar_path` proxy prints as a −1 return on `1/31`'s path stack.
/// [`crate::Unit::search`] **is** that field and has been since
/// `docs/PATHFINDER.md` §18; the seam comment that said otherwise
/// outlived it by a fortnight.
///
/// ~~**10232 parts on `Unit::do_move+0xe84`**, 99 draws against 95~~ —
/// item 456's reading of the block, and the squad's walk was the
/// *consequence*. The order list parts a block earlier: on **10232**
/// six of the AI's raiders hold one order where the original holds two
/// (item 463), and the three that then walk are the three whose freed
/// group move had somewhere to go.
///
/// **10232 → 10233 on item 463**, and the arm is
/// `Unit::do_attack@005f1b80`'s own gate. The human's building `0/2004`
/// dies on block 10231 and the original's raiders keep their
/// `ATTACKORDER` on it — `ox 2004 whom 0 uid 4 mandatory 1` — for
/// exactly as long as their reload runs: `1/42` drops it on 10231,
/// `1/28` on 10233, `1/27` on 10239, each on the block its
/// `recharging` reaches nought. Both of `do_attack`'s aliveness tests
/// sit under `ObjectTypeData +0x1e8 attack == 0`, so a unit that can
/// attack never asks; the question is `Unit::fight@005fd4d0:196`'s,
/// and `:102`'s recharging arm returns before it.
/// [`sim::Sim::do_attack`] asked it first, and killed six orders two to
/// eight blocks early.
///
/// ~~**10233 parts on `Unit::fight+0x9b0`**, six draws against four, and
/// the extra pair is the *human* citizen `0/5`~~ — item 464 closed the
/// citizen's pair and the word **held at 10233**, because the frame
/// carries two residues and only one of them was `0/5`'s.
///
/// **Held at 10233 on 2026-09-21, item 464.** `Unit::think`'s step 3 now
/// takes the military bit as well as the attack column, so a citizen —
/// `attack 40`, no `role & 0x10000` — never enters `think_attack`; and
/// `Unit::target_opportunity`'s **flee arm** is modelled, so the hit
/// citizen runs the way the original's does (`docs/COMBAT.md` §34). The
/// frame's draws go **6/4 → 5/4**, and the value diff beside it is block
/// 10234, where `0/5`'s twenty-two rows go to **three**: its `FLEE_TO`
/// lands on the original's own `(792, 31800)`, on the original's own
/// frame, with its heading, its guy clock and its position all agreeing.
///
/// ~~The one draw left is `1/28`'s `Unit::do_move+0xe84`, one frame early
/// — and item 464 named its mechanism … It becomes its group's own leader
/// on the frame it does nothing and plans on the next, which is
/// `do_group_move` running `do_move` for the leader alone.~~ Item 465
/// widened the frame and the mechanism dissolved: `oxx 40 → 28` is the
/// whole of **group 65** on that block — `1/27`, `1/28`, `1/29`, `1/40`,
/// `1/41` and `1/42` every one — so it is not `1/28` promoting itself but
/// `Group::refresh_group_order` re-seating the block, and its early
/// return is what costs the frame (`docs/ORDERS.md` §16).
///
/// **10233 → 10234 on 2026-09-21, item 465.** `Group::action_move_near`
/// carried an invented `g.army.is_some()` line that the original's gate
/// at `705f00`–`705f61` does not have, so the probe's six raiders —
/// pushed out of army 1 on frame 8186, `group 65` in the dump for the
/// next two thousand blocks — held plain `MOVE_TO`s and never entered
/// `do_group_move` at all. With the pushed group given a pool slot and a
/// record, and `unit_masks & 4` carried in its place (which is what the
/// original really exempts a `go_to` group by), `1/28` spends 10233 in
/// the follower arm's re-seat and plans on 10234 like the original's.
///
/// **The value diff**: block 10234's `1/28` rows go **fifteen to none** —
/// `pos ours (4801,30176) theirs (4776,30168)` and `path:length ours 43
/// theirs 1` among them — and the word's own block is now item 464's
/// three `0/5`/`0/2001` rows alone. Across the widening's window the
/// parted keys go **655 → 440**.
///
/// **What 10234 is**: a frame whose whole dumped record agrees and whose
/// draw stream does not — the original spends **204** draws there,
/// `PathFinder::calc_road_cost+0x46` over and over, and this crate spends
/// six. ~~for the same 43-node route~~ — **item 475 struck that clause**:
/// the count is right, the owner is not. ~~`1/28`'s own rows reopen one block
/// later, on 10235, where the route is 240 out in `x` because the
/// formation slot is (`order:move.off_x ours 120 theirs 648`).~~
///
/// **Held at 10234 on 2026-09-21, item 471**, and 10235 is closed:
/// `off_x` is `x mod 0x300`, the destination said twice, so that row was
/// never a slot of its own, and the six slots were the right table turned
/// by the wrong angle. §12's probe queues its walk home at `QUEUE_LAST`,
/// the one queue position `Group::action_move_near` asks
/// `GroupData::get_loc_to` for — where the leader **ends up**, through
/// `UnitData::get_final_loc` — and this crate asked `get_loc`, found the
/// destination under the leader's own feet and took no bearing at all.
/// The bearing is `find_angle` from the **farm** the probe has queued an
/// attack on, `0/2004` at `(2112, 31296)` in run19's own block 8187, and
/// it reproduces the dump's `group_angle 890830848` to the bit
/// (`docs/ORDERS.md` §17).
///
/// **The value diff**: block 10235's `1/28` rows go **sixty to three** —
/// the whole 43-node route is the original's, `path[1..42]` every one,
/// with `path:length` 44 → 43, `order:move.dest_x` 5496 → 6024 and
/// `heading` 1372520448 → 1252851712. What is left is `pos ours
/// (4801,30175) theirs (4800,30175)`, one unit in `x`, said three times.
/// Across the widening's window the parted keys go **440 → 322**.
///
/// **What still holds the word** is the same thing as before this item and
/// it is not the route: 204 draws against six, all of them
/// `calc_road_cost`. ~~The same cost function is what
/// `PROBE_PLAN_PARTED`'s twenty-two entries are — a `y` one or two cells
/// south over three stretches that re-converge, on a route whose start,
/// end and every `x` agree — so one residue now stands in two places and
/// closing it is the next item on this frame.~~
///
/// **Item 475 falsified that, and named the word instead** (`docs/ORDERS.md`
/// §18). `calc_road_cost` has one caller in the executable —
/// `astar_caravan_road`, under `find_road` — so it is the **road**
/// search's, and block 8186, where `PROBE_PLAN_PARTED`'s twenty-two
/// waypoints are planned, prices no road node at all. Two residues. What
/// holds this word is a road search the original runs on 10234 (198
/// nodes) and 10235 (73) and **this crate does not run at all**: the two
/// streams share the frame's first four draws and part on the fifth,
/// which is that search's first. A missing behaviour, not a divergent
/// one, and it is item 478.
///
/// The number is unchanged and the stream was never unaligned —
/// `FLOORS[1].word = 1850` is the *scored* capture's word, where
/// `unwrap_or(last)` means "never parted inside it", and reading that row
/// for this one is the trap `docs/ORDERS.md` §18.2 writes down.
/// [`great_lakes_s_word_draws_are_a_road_search_and_8186_spends_none`]
/// keeps all of it.
///
/// **10234 → 10237 on item 478**, and the search is this crate's now:
/// `BuildType::place_roads` is `find_road`'s third caller and the one
/// that runs, from `Build::process`'s deferred `build_masks & 0x100`
/// arm. The flag is set by `City::regen_roads`, whose **second** writer
/// is `Build::remove_from_city` — the human's farm `0/2004` falls on
/// 10230 and the city's six survivors each replan on the frame
/// `12_240 - o` names. This crate had only `Build::activate`'s call, so
/// nothing was ever flagged; with both writers in, 10234 spends 204
/// draws against 204 and 10235 spends 78 against 78, node for node
/// (`docs/ROADS.md` §1.2, `docs/ORDERS.md` §19).
///
/// ~~**The two numbers this pin is the floor of have separated.** The
/// *count* word is **10244**; the *sequence* word is **10237**~~ — and
/// **they have met again at 10244 on item 483**, which changed no
/// simulation behaviour at all. What parted at 10237 was a *label*: both
/// sides spent seven draws and the sixth and seventh are
/// `Ammo::do_damage@00678060`'s puncture pair, which
/// [`trace::SITES`](crate::trace::SITES) did not carry — so the
/// original's read `678cb9`/`678cde` against this crate's unattributed
/// `projectiles` phase mark. 10242 and 10249 were the same shape, and
/// those three frames are the only ones in 24,000 where either side
/// takes the draw. With `sim::fight::SITE_PUNCTURE_X`/`_Y` named and
/// both addresses in the table, all three agree draw for draw, the
/// sequence word rises to the count's 10244, and the whole
/// 24,000-frame label dump is unchanged on every other frame
/// (`docs/COMBAT.md` §39).
///
/// So this is again **one** number, and ~~the next thing on it is a
/// simulation disagreement: 10244 spends four draws against three and
/// parts at index 1, ours `Guy::set_anim+0x97a < Unit::move_step+0x823`
/// — the blocked stand — against the original's `Guy::inc_time+0x271`
/// wrap.~~
///
/// **Item 487 closed 10244 and the two numbers moved together to
/// 10277** (`docs/ORDERS.md` §20). The blocked stand was `1/27` walking
/// into `1/29`, and `1/29` was standing because this crate held it
/// there: `Unit::ungroup_move_order@005fd140` **re-heads** the plain
/// move it makes (`remove_current` then `LinkListBase::add`, which
/// prepends) where this crate rewrote the order where it stood, and
/// `do_move@005f7b30`'s dead-target re-path is gated on
/// `vector_dist(unit − move.dest) <= 0x480` where this crate re-pathed
/// unconditionally. With both, `1/29` takes the original's own 43-node
/// walk home on 10240 and `1/27` walks past.
///
/// ~~The word's shape is unchanged one raider over: **10277 spends seven
/// draws against six**~~ — **item 489 closed 10277 too, and the whole
/// blocked-stand family with it**. `1/40`'s hard collision with `1/41`
/// and the two `half_step` rows beside it were one arm declining:
/// [`sim::Sim::same_group_soft`] asked `army_of` where
/// `detect_unit_collision@00617060:307` asks `UnitData +0x80`, a
/// `Groups::list` slot, and a **pushed** group's members are in no army
/// at all — every raider of this probe since item 465 put them in the
/// pool (`docs/COLLISION.md` §11).
///
/// ~~**10294 is a different mechanism.**~~ **Item 494 closed it, and the
/// draw the brief named was the last link of a chain sixty blocks long.**
/// `Unit::do_idle` ran there in the original because its citizen `0/5`
/// had arrived with an **empty** order list; this crate's had a second
/// order under the flight, because `think_peasant`'s human gate read
/// `LeaderOptions +0x8` as the number of idle frames to wait when it is
/// the **index of** that number — `2` selects `0xc`, so the wait is 12
/// and this crate used 2 (`docs/ORDERS.md` §21). Nothing in `do_idle`
/// was wrong; the citizen simply should not have been idle-tasked at all.
///
/// ~~**10303 is `1/51`'s animation clock.**~~ **Item 497 closed it, and
/// the clock was never the mechanism.** Ours spent three draws against
/// the original's four, the extra `Guy::set_anim+0x97a <
/// Guy::inc_time+0x271`, and block 10304 held two rows of the same unit
/// — `g.cur_time` 30 against 0, `g.last_time` 29 against −1. `1/51` is a
/// **scholar** seated in a university: on 10274, thirty blocks under the
/// word, both sides wrapped a slot together and re-rolled, the original
/// took `0x1d` and this crate `0x20`, and thirty frames later the
/// original's 30-frame teach slot ran out where this crate's 118-frame
/// one had ninety left. `set_anim:332`'s `param_2 == 0x20` walk examines
/// **every** member of the host's chain, the guy asking included
/// (`docs/ANIM.md` §4.12).
///
/// ~~**10582 is a production frame, and the market says so twice.**~~
/// **Item 506 closed it, and the cause spends no draw at all.** The
/// sequence parted at index 0 with the counts equal — ours
/// `[make_stuff+0x221 ×2, make_stuff+0x63d]` against theirs
/// `[use_market+0x1ed, make_stuff+0x221 ×2]` — the count parted one
/// frame later at eight against three on five
/// `Leader::create_units+0x642`, and block 10583 held one row,
/// `1/2018 queue:queued` 1 against 0. All three are **one purchase five
/// frames earlier**: `Leader::market_speculation@006c8110`, the
/// rotation's `Setup` step, buys a hundred food for **128** wealth on
/// sim-frame 10576 (run117, block 10575 `93 86 128 269 100 0` → block
/// 10577 `194 86 0 269 100 0`) and this crate's buy and sell passes were
/// a declared seam. With no wealth the original's `can_pay` refuses the
/// head this crate bought, its `use_market` spends the sell draw, and
/// its step machine disarms where this crate ran a second pass
/// (`docs/ECONOMY.md` §13).
///
/// ~~**10817 is `1/28`'s collision.**~~ **Item 515 closed it, and the
/// collision was a consequence of a position 370 units out.** This
/// crate's `1/28` stood 370 east and 242 north of the original's — about
/// fifteen frames of its own walk — and so collided with a gaia animal
/// that sits in the same place on both sides and never moves. The block
/// could not say so, because `1/28 pos` first parted **583 blocks
/// earlier**, on 10235, which was parked 477. Both sides start that walk
/// from the same point with the same facing and the same `last_speed`;
/// the step parts at once, ours `(25, 7)` against theirs `(24, 7)`, and
/// the original's own sine table gives `(25, 7)` for a distance of 26 and
/// `(24, 7)` for 25 — so the trig agrees to the bit and only the distance
/// does not. `1/28`'s `myspeed` is 26 and it walks at 25, its group's
/// (`docs/GROUPS.md` §18).
///
/// ~~**10834 is the same raider, and the residue is one frame of the
/// cap.**~~ **Item 518 closed it, and the writer was neither a member
/// nor the order stack.** Ours spent three draws against two there, the
/// extra `Guy::set_anim+0x97a < Unit::move_step+0x823`, over a position
/// one world unit out from 10242. 515 booked it on `1/40` holding an
/// `ATTACK` where the original holds its `GROUP_MOVE`; the dump holds the
/// `ATTACK` at the head too — the log writes newest first — and no member
/// of group 65 reports on either side. What drops the cap to 25 on frame
/// **10241** is `Groups::process@006fa210`, one pool slot per player a
/// frame, and 10241 is slot 1's frame: player 1's slot 1 is `group 65`
/// (run120's per-frame coverage; `docs/GROUPS.md` §19).
///
/// **11185 is a market frame, 351 frames on.** Ours spends **nine**
/// draws and the original **eight**, parting at index **2**: ours
/// `Leader::use_market+0x1ed` against the original's
/// `Leader::make_stuff+0x221`. No dump on disk reaches it — run100 ends
/// on block 10899 — so its widening was owed ([`WIDENINGS`]). Item 520
/// paid it with run123, and the word **did not move**: the delta above is
/// still the word's, and the block — 11185, where slot 1 of the make list
/// parts — is `run123_s_word_frame_is_widened_whole`'s.
///
/// **11185 → 11531 on item 327, and the widening named the mechanism.**
/// run123's slot 1 was an emptied Merchant slot in the original and a
/// Cataphract here, because `create_units`' merchant arm read a
/// `known_rares` nothing wrote. `plan_strategy`'s step 9 is the writer,
/// `calc_gather` sums it, and the arm now offers the original's Merchant
/// on every block of nine captures (`docs/AI.md` §55). The new word's
/// delta: ours **four** draws and the original **three**, parting at
/// index **1** — ours `Guy::set_anim+0x97a < Unit::do_idle+0x7d` against
/// the original's `Guy::set_anim+0x97a < Guy::inc_time+0x271`. **Past
/// every dump on disk**: run123 ends on block 11459, so the word's block
/// is owed a capture ([`WIDENINGS`]); 11185's own block stays in
/// `run123_s_word_frame_is_widened_whole` as the value diff of the move.
/// **Item 533 took the capture, run125, and held the word**: its block is
/// in `run125_s_word_frame_is_widened_whole`.
///
/// ~~**11531 is `1/34`'s arrival, two frames early.**~~ **Item 539 moved
/// it 11531 → 11582, and the mechanism was the collision probe, not the
/// march.** The formation hop 533 named was the leader `1/37` reaching its
/// waypoint a frame early, 22 units ahead on half steps it missed; each
/// miss was a soft-collision flag, and the two flags that parted with
/// every unit cell agreeing were `collide_here`'s fast path stepping two
/// cells past an **empty** world cell, where the original does not
/// advance at all (`docs/COLLISION.md` §4.2). The new word's delta: ours
/// **948** draws and the original **9**, parting at index **5** — ours
/// `Leader::produce_building+0xc99` against the original's
/// `Guy::set_anim+0x104b`. **Its block is on disk**, inside run125, and
/// `run125_s_word_frame_is_widened_whole` holds it; this is the delta, the
/// widening is the block.
///
/// ~~**11582 is a Mine the original does not place.**~~ **Item 545 moved
/// it 11582 → 11757, and the mechanism was two arms of `get_cost`, not
/// the make list's order** (`docs/AI.md` §56). `MILITARY_UNIT_DISCOUNT`
/// was never applied, so from 11183 this crate held three metal fewer;
/// on 11579 Militia (80 metal) read unaffordable, its offer took
/// `0x40` for `0x100` and sank below the Mine. With the discount the
/// lists agree, and the frame asked for the Phalanx's **research**, which
/// needed `get_cost`'s research arm and `produce_tech`'s unit route: the
/// original queues it at `1/2016` for 90 food and 54 metal, and so does
/// this crate now. The new word's delta: ours **7** draws and the
/// original **8**, parting at index **0** — ours
/// `Guy::set_anim+0x97a < Guy::inc_time+0x271` against the original's
/// `Guy::set_anim+0x97a < Unit::move_step+0x823`. **Past every dump on
/// disk**: run125 ends on block 11599, so the word's block is owed a
/// capture ([`WIDENINGS`]); 11583 stays in
/// `run125_s_word_frame_is_widened_whole` as the move's value diff.
/// **Item 554 took the capture, run130, and held the word**: its block is
/// in `run130_s_word_frame_is_widened_whole`.
///
/// ~~**11757 is army 2's `1/62` stopping a frame late.**~~ **Item 557
/// moved it 11757 → 11806, and the mechanism was the back-pointer**
/// (`docs/GROUPS.md` §23). `UnitData +0x80` is its own state in the
/// original: `Group::add` normalizes a small group at every step, so the
/// squad `1/62`–`1/64` joined army 2 on 11424 as `1/64` alone, and
/// `Group::sort` on 11512 killed that stray follower — clearing its
/// pointer — and re-added the squad without writing one. `1/64` then
/// walked uncapped and `1/62`'s detour of 11688 went around it where the
/// original's does. The new word's delta: ours **7** draws and the
/// original **6**, parting at index **1** — ours
/// `Guy::set_anim+0x97a < Unit::move_step+0x823` against the original's
/// `Unit::resolve_unit_collision+0xb52`. **Past every dump on disk**:
/// run130 ends on block 11799, so the word's block is owed a capture
/// ([`WIDENINGS`]); 11758 stays in `run130_s_word_frame_is_widened_whole`,
/// pinned empty, as the move's value diff, and
/// `run134_s_pool_list_is_the_original_s` holds the pool lists the move
/// rests on.
///
/// ~~**11806 is `1/37` stepping where the original's waits.**~~ **Item
/// 560 moved it 11806 → 11903, and the mechanism was where the soft
/// one-shot is set** (`docs/COLLISION.md` §12). `detect_unit_collision`
/// raises `unit_masks & 0x100000` only when its nine-cell walk ends
/// without a hard hit; this crate raised it on any soft candidate, so
/// `1/37`'s sweep of frame 11804 — a group-mate soft, then `1/64` hard —
/// left a half step the original never has, and `1/37` stepped on 11805
/// where the original's waited to 11809. The new word's delta: ours
/// **5** draws and the original **4**, parting at index **1** — ours
/// `Guy::set_anim+0x97a < Unit::move_step+0x823` (`1/64`'s stop) against
/// the original's `Guy::set_anim+0x97a < Guy::inc_time+0x271`. **Past
/// every dump on disk**: run135 ends on block 11859, so the word's block
/// is owed a capture ([`WIDENINGS`]); 11807 stays in
/// `run135_s_word_frame_is_widened_whole`, pinned empty with every block
/// after it, as the move's value diff.
///
/// ~~**11903 is `1/62`'s detour round `1/27` going south where the
/// original's goes north.**~~ **Item 566 moved it 11903 → 12038, and the
/// mechanism was the pathfinder's validity memo** (`docs/PATHFINDER.md`
/// §24.6). `valid_ucoord` caches its verdicts in `PathFinder +0x50`, and
/// only `kill_lists` empties it. Every finder calls that after its search
/// and on none of its early returns. So `1/27`'s `find_upath` on 11901,
/// whose goal pre-walk probed row 441 and returned early, left its refused
/// verdicts for `1/62`'s search, which never probed S (run138). This crate
/// gave each pre-walk and each search a fresh memo. The new word's delta:
/// ours **4** draws and the original **5**, parting at index **4** — ours
/// nothing against the original's `Guy::set_anim+0x97a <
/// Unit::move_step+0x823`. **Past every dump on disk**: run136 ends on
/// block 11959, so the word's block is owed a capture ([`WIDENINGS`],
/// item 571). The value diff is nearer than the word: run136 agrees on
/// every record from 11902 through 11921 and parts on **11922**, army 1's
/// squad stopping in the original, and `run136_s_word_frame_is_widened_whole`
/// pins both.
///
/// ~~**12038 is `1/62`'s arrival stand against `1/64`.**~~ **Item 571
/// moved it 12038 → 12135, and the mechanism was a converted figure's
/// stand** (`docs/ANIM.md` §11). who=1's barracks research lands on
/// sim-frame 11921 and converts the nine walking type-82 figures, and the
/// original's `Guy::init_real(guy, 1)` stands each kept guy `stopped`
/// with its speeds zeroed. This crate reset the clock alone, so `1/62` did
/// not turn at once on 11922, parted by 11923, and walked past its stand
/// on 12038. The new word's delta: ours **7** draws and the original **6**,
/// parting at index **6**. Ours spends an extra `Guy::set_anim+0x97a <
/// Guy::inc_time+0x271` there, `1/68`'s idle wrap. **Inside run163**
/// (block 12136); `run163_s_word_frame_is_widened_whole` pins the block
/// and its chain back to `1/68`'s birth on 12057.
///
/// ~~**12135 is `1/68`'s idle wrap.**~~ **Item 657 moved it 12135 →
/// 12184, and the mechanism was which level `release_mustering` reads**
/// (`docs/ARMY.md` §20). The original reads `data_encrypted->epoch[0]`,
/// the Military library level, where this crate read `ages`. On sim-frame
/// 12024 who=1 has age 1 and Military 2, so its `rush` rule keeps army 2's
/// six standard mustering (`n >= 16`) where this crate released them to
/// defend. On 12057 `find_local_army` then takes army 2, which is
/// mustering, for the newborn `1/68`. This crate found only the empty army
/// 0. So `1/68` now walks to army 2's first unit and is seated in pool
/// group 66, as the original's is. The new word's delta: ours **47** draws
/// and the original **95**, parting at index **0**. Ours spends six pairs
/// of `Leader::create_buildings+0xffb`/`+0x1017` that the original does
/// not. The original spends a bird's thirty-round `Animal::think_bird+0x2aa`/
/// `+0x2d3` arm that ours does not. **Inside run163** (block 12185,
/// [`GREAT_LAKES_MAKE_BLOCK`]); `run163_s_word_frame_is_widened_whole`
/// pins the block's chain, which opens on who=1's production list on
/// 12181.
///
/// ~~**12184 is who=1's Scholar, bought a pass early.**~~ **Item 661 moved
/// it 12184 → 12429, and the mechanism was a trade route's worth**
/// (`docs/AI.md` §63). On sim-frame 12180 the original's purse held 55
/// wealth against the Scholar's 56 and this crate's 57, so this crate
/// bought it on 12182 and ran a second Units / Buildings / Make pass. The
/// purse was parked 514's 32 income a frame: `trade_val` 160 against 176
/// on both of who=1's cities, because `Leader::gain_tech`'s Civic arm —
/// re-mask every city, then `City::find_buildings` — was missing, and the
/// Barracks `1/2016` and Stable `1/2018` never joined Norwich, where the
/// original seats them on block 8734. The new word's delta: ours **14**
/// draws and the original **12**, parting at index **5**: ours spends
/// `Guy::set_anim+0x97a < Unit::move_step+0x823` where the original
/// spends `Guy::set_anim+0x97a < Guy::inc_time+0x271`. **Past run163**
/// (block 12430 against its last, 12399): no dump on disk prints it, and
/// item 669 owes the capture and its widening. run163's test pins the
/// move's value diff: the rows on 12181–12185 are gone.
///
/// ~~**12429 is `1/67`'s stop against `1/68`.**~~ **Item 669 moved it
/// 12429 → 12536, and the mechanism was which waypoints `do_move`'s tile
/// arm unwinds** (`docs/ORDERS.md` §4.4, `docs/AI.md` §64). Before
/// `find_tpath` the original pops a non-final top whose tolerance is 1 to
/// `0x60`, unsigned (`0x5f8ad2`); this crate popped `tolerance < 0x60`, so
/// on 12322 it dropped `1/68`'s tolerance-0 formation waypoint and planned
/// the tiles to the final goal. From 12422 `1/68` walked a leg the
/// original's never does, and on 12429 `1/67` collided with it and
/// stopped. The new word's delta: ours **93** draws and the original
/// **94**, parting at index **92**. Ours spends `Farms::inc_time+0x1ae`
/// where the original spends `PathFinder::astar_path+0x1697`. **Inside
/// run174** (block 12537); `run174_s_word_frame_is_widened_whole` pins the
/// block's first-parting rows, the squad `1/27`–`1/29`'s orders, and
/// the move's value diff.
///
/// ~~**12536 is the squad `1/27`–`1/29`'s orders.**~~ **Item 673 moved it
/// 12536 → 12897, and the mechanism was the gate on the retry a failed
/// unit-grid search buys** (`docs/PATHFINDER.md` §21.6, `docs/AI.md`
/// §65). `astar_path`'s roll and `find_upath`'s reprieve both test vslot
/// `+0x14`, `is_move`, which every move class answers 1; this crate tested
/// a move *without* the action bit. So on 12536 the captain `1/27`,
/// blocked by `1/64` under a `GROUP_ATTACK_TO`, lost its order where the
/// original rolls `retry` 6, ungroups the squad and re-plans `1/28` and
/// `1/29` on the world grid, which was the missing `astar_path+0x1697`.
/// The new word's delta: ours **8** draws and the original **9**, parting
/// at index **2**. Ours spends `Guy::set_anim+0x97a < Guy::inc_time+0x271`
/// where the original spends `Guy::set_anim+0x97a < Unit::move_step+0x823`.
/// **Inside run174** (block 12898, its last but one);
/// `run174_s_word_frame_is_widened_whole` pins `1/41`'s rows on the
/// block and the move's value diff.
///
/// ~~**12897 is `1/41` against `1/15`.**~~ **Item 678 moved it 12897 →
/// 14382, and the mechanism was the pathfinder's copies of the collision
/// blocks** (`docs/PATHFINDER.md` §26, `docs/AI.md` §66). A `nocoll`
/// probe reads a world cell's copy from `pathfinder +0x4c` when the tree
/// holds one, and copies what it read live when it does not
/// (`CollCheck::fill_slots@006820e0`); only `kill_lists` empties the tree,
/// and a suspend hands it to the unit with the memo. On 12623 `1/41`'s
/// 48-grid search copied five world cells and suspended; on 12624 its
/// resume read `1/66` and the rest where they had stood a frame before,
/// and this crate read the live blocks. So `1/41`'s sidestep on 12626 was
/// 23 entries against 20, it trailed the original's by three frames, the
/// squad ungrouped on 12825 there and not here, and on 12897 the
/// original's `1/41` met `1/15`. The new word's delta: ours **11** draws
/// and the original **13**, parting at index **2**: ours spends
/// `Leader::make_stuff+0x221` where the original spends
/// `Leader::produce_building+0x1805`. **Past run174** (block 14383 against
/// its last, 12899) and past every Great Lakes dump on disk below run80's
/// 23960, so item 678 took **run178** and widened it:
/// `run178_s_word_frame_is_widened_whole` pins the word's block 14383 —
/// who=1 places a building, `1/2025`, at (39552, 17472) here and (42624,
/// 19776) there — and `run174_s_word_frame_is_widened_whole` keeps the
/// move's value diff.
///
/// **The block's own rows are not restated here**, and that is
/// deliberate: block 10818's eight rows of `1/28` are *asserted* in
/// [`crate::diff::harness`]'s
/// `run100_s_word_block_is_every_record_the_dump_carries`, so a copy in
/// this comment would be prose nothing checks and the two would drift
/// the first time the word moved. The draw delta above is the half that
/// lives nowhere else — it is computed by
/// `run53_s_24000_frames_put_the_ceiling_where_run33_did` and printed,
/// never pinned as text. **The rule this comment now follows: the word's
/// *delta* here, the word's *block* in the widening, and each says
/// which.**
///
/// ~~**14382 is who=1's Barracks.**~~ **Item 688 moved it 14382 → 14529,
/// and the mechanism was the site values a founded city wears down**
/// (`City::fix_world_vals`, `docs/AI.md` §67). `City::init` quarters
/// `WData.val` out to ring `(radius + 3) / 4` of the circle round the
/// centre and halves it on three rings beyond; this crate never wrote the
/// value after load. So Norwich's founding left cell (55, 25) at `val` 4
/// here against 2, the spiral round the capital took (51, 22) at 1252
/// over (55, 25) at 1251, and the Barracks `1/2025` went up four cells
/// west. **The new word's delta: ours 7 draws and the original 3,213,
/// parting at index 0**: ours `Guy::set_anim+0x97a < Guy::inc_time+0x271`
/// where the original spends `PathFinder::calc_road_cost+0x46`. **Inside
/// run178** (block 14530 against its last, 14899);
/// `run178_s_word_frame_is_widened_whole` pins the word's block — `1/23`'s
/// path flags and `1/28`'s figure — and the move's value diff on 14383.
///
/// ~~**14529 is the capital's move.**~~ **Item 695 moved it 14529 →
/// 14650, and the mechanism was the caravan's road** (`docs/CARAVAN.md`
/// §10, `docs/ROADS.md` §10). The 3,206 draws were not the Senate's: the
/// capital's move to Norwich is drawless. They are `1/23`'s route verified
/// from `Unit::do_move`, because the tile under its next waypoint is no
/// longer road — the stray-road sweep, `Roads::scan_and_kill_stray_roads`,
/// had eroded 17 tiles of the trade road since 5905, which this crate kept
/// (run189's packet and `WORLD` block). **The new word's delta: ours 2
/// draws and the original 3, parting at index 0**: ours
/// `Guy::set_anim+0x97a < Guy::inc_time+0x271` where the original spends
/// `Unit::do_non_flat_gather+0x54b`. **Inside run178** (block 14651);
/// `run178_s_word_frame_is_widened_whole` pins the word's block — the
/// Woodcutter's Camp `1/2009`'s gather list — and the move's value diff on
/// 14530.
///
/// ~~**14650 is the camp's gather list.**~~ **Item 698 moved it 14650 →
/// 14982, and the mechanism was `do_move`'s `GATHER` park**
/// (`docs/COLLISION.md` §15). The citizen `1/43`, stopped by the standing
/// `1/18` on 14643 with its search suspended, gives its walk up on 14649,
/// six frames on and 259 short of its point, and draws a tile afresh on
/// 14650; this crate kept the walk. **The new word's delta: ours 11 draws
/// and the original 15, parting at index 0**: the original spends three
/// `Guy::init_real+0x52`, a three-figure birth, before the frame's
/// `Guy::inc_time` wraps, and ours spends none. **Past run178** (block
/// 14983 against its last, 14899), so item 698 took **run192**; the
/// widening names the word's block, and
/// `run178_s_word_frame_is_widened_whole` keeps the move's value diff on
/// 14651.
///
/// ~~**14982 is `1/79`'s birth.**~~ **Item 706 moved it 14982 → 15175,
/// and the mechanism was a government patriot, reached through a graft
/// table.** Two builds, one landing. Under the word, on 14946, who=1's
/// free archers were Bowmen here and Longbowmen there: `Tribe::graft` was
/// the identity, so the British `get_graft(Archers)` never answered
/// Longbowmen (`docs/TECH.md` §"The graft table", every nation's table
/// diffed against run3's `DUMP_ALL`). On the word, who=1's Senate finishes
/// Despotism on tick 14982 and `Build::finished` trains The Despot, `1/79`
/// (`docs/TECH.md` §"The government patriot"); this crate gained the tech
/// and trained nothing. **The new word's delta: ours 4 draws and the
/// original 3, parting at index 2**: ours spends `Guy::set_anim+0x97a <
/// Unit::do_guard+0x7f4` where the original spends `Guy::set_anim+0x97a <
/// Guy::inc_time+0x271`. **Past run192** (block 15176 against its last,
/// 15039), so item 706 took **run196**; `run196_s_word_frame_is_widened_whole`
/// names the word's block — nothing first-parts on it, and the nearest
/// parting under it is the three free Longbowmen's guard posts on 15095,
/// handed out the other way round — and
/// `run192_s_word_frame_is_widened_whole` keeps the move's value diff on
/// 14946 and 14983.
///
/// ~~**15175 is the free Longbowmen's guard posts.**~~ **Item 711 moved it
/// 15175 → 15383, and the mechanism was a unit's own mirror,
/// `unit_masks & 2`** (`docs/GROUPS.md` §25). On 15095 who=1's army sends
/// The Despot `1/79` on and its three Longbowmen guard it. `Unit::do_guard`
/// negates a guard's `dx` when its target carries the bit (`5e5fed`), and
/// `Unit::set_angle` had set it on the Despot's first turn, 14985. This
/// crate did not carry the bit, so `1/77` and `1/78` traded posts; the
/// extra `Unit::do_guard` stand on 15175 was theirs (run196). **The
/// new word's delta (this constant's comment): ours 4 draws and the
/// original 3, parting at index 1**: ours spends `Guy::set_anim+0x97a <
/// Unit::do_idle+0x7d` where the original spends `Guy::set_anim+0x97a <
/// Guy::inc_time+0x271`. **Past run196** (block 15384 against its last,
/// 15232), so item 711 took **run202**. The new word's block is in
/// `run202_s_word_frame_is_widened_whole` (the widening test), and
/// `run196_s_word_frame_is_widened_whole` keeps the move's value diff:
/// nothing parts on run196's own blocks, 15040..15232.
///
/// ~~**15383 is the citizen `1/70`'s order.**~~ **Item 715 moved it 15383
/// → 15384, and the mechanism was `Wall::process`'s site recruiter**
/// (`docs/AI.md` §69). On 15382 who=1 places the Pyramids `1/2026` and
/// swarms `1/70` onto it; `do_move` refuses the approach and kills the
/// build beneath it, on both sides. The site's own phase is the same
/// frame, and an unfinished wonder calls in the nearest citizen that is
/// not busy with `add_build_order(QUEUE_NEW, 0)`. This crate had no
/// recruiter, so `1/70` stood idle on 15383 — the extra `Unit::do_idle`
/// stand — and went back to its camp. **The new word's delta (this
/// constant's comment): ours 51 draws and the original 46, parting at
/// index 4**: ours spends a third `Leader::create_buildings+0xffb` where
/// the original spends `Animal::think_bird+0x82`. It is inside run202
/// (block 15385 against its last, 15440); the new word's block is in
/// `run202_s_word_frame_is_widened_whole` (the widening test), which also
/// keeps the move's value diff on 15383 and 15384.
///
/// ~~**15384 is who=1's wonder arm.**~~ **Item 722 moved it 15384 →
/// 15608, and the mechanism was `CityData::num_wonders`** (`docs/AI.md`
/// §70): it counts a wonder *site*, and this crate counted finished
/// wonders. On 15384's second `create_buildings` pass who=1's Pyramids
/// site shuts its city out of the Colossus and the Hanging Gardens, so the
/// original values two (city, wonder) pairs and this crate valued four.
/// **The new word's delta (this constant's comment): ours 40 draws and
/// the original 38, parting at index 0**: ours throws
/// `Army::find_target+0x410`, the difficulty gate's coin, where the
/// original's first draw is the score, `+0x7df`. **Past run202** (block
/// 15609 against its last, 15440), so item 722 took **run211**; the new
/// word's block is in `run211_s_word_frame_is_widened_whole` (the
/// widening test), and `run202_s_word_frame_is_widened_whole` keeps the
/// move's value diff: nothing parts on 15385..15440.
///
/// ~~**15608 is who=1's difficulty coin.**~~ **Item 729 moved it 15608 →
/// 15619, and the mechanism was `Object::take_damage`'s stamp**
/// (`docs/AI.md` §71): a combat hit at difficulty below 2 writes the
/// struck object's owner's `frame_attacked`, and this crate wrote the
/// field only from `Army::find_target`. The human's stood at 8186 here and
/// 10233 there, so on 15608 army 2's `find_target` passed the 7,200-frame
/// stamp for L=0 and threw the coin the original never reaches. **The new
/// word's delta (this constant's comment): ours 4 draws and the original
/// 5, parting at index 0**: the original spends `Guy::set_anim+0x97a <
/// Unit::move_step+0x823` first. It is inside run211 (block 15620 against
/// its last, 15859); the new word's block is in
/// `run211_s_word_frame_is_widened_whole` (the widening test), which also
/// keeps the move's value diff on 15608 and 15609.
///
/// ~~**15619 is the escort's move step.**~~ **Item 736 moved it 15619 →
/// 16460, and the mechanism was `Group::action_siege_attack_to`'s stack
/// sub-group** (`docs/GROUPS.md` §26): it is `Group::clear(-1)`'s record
/// carrying the parent's `id` and `army`, so the anchor's `ATTACK_TO` is
/// laid out on `facing` 0; this crate read army 3's 1. The Despot's move
/// of 15350 carried it (parked 716), the dying move handed it into
/// `unit_masks & 2`, and `do_guard` mirrored the escort's posts off it on
/// 15606. **The new word's delta (this constant's comment): ours 1 draw
/// and the original 3, parting at index 1**: the original spends
/// `Guy::set_anim+0x97a < Unit::move_step+0x823`, a blocked step, which
/// ours does not. **Past run211** (block 16461 against its last, 15859),
/// so item 736 took **run218**; `run211_s_word_frame_is_widened_whole`
/// keeps the move's value diff: nothing parts on 15441..15859.
///
/// ~~**16460 is `1/23`'s blocked step.**~~ **Item 742 moved it 16460 →
/// 17099, and the mechanism was `Unit::move_step`'s give-up**
/// (`docs/COLLISION.md` §17): a unit blocked 26 frames (`collide >=
/// 0x1a`) widens `tolerance` and jumps to the arrival tail
/// (`005fb7bb jmp 005fb82c`) **without stepping**, so the leg pops where
/// it stands. This crate took the step first, and run218's `1/23`,
/// blocked by The Despot, walked 26 north on 16459. **The new word's
/// delta (this constant's comment): ours 217 draws and the original 225,
/// parting at index 0**: the original spends `Guy::set_anim+0x97a <
/// Unit::move_step+0x823`, a blocked step, where ours spends
/// `Guy::set_anim+0x97a < Unit::do_move+0x11cf`. **Past run218** (block
/// 17100 against its last, 16711), so item 742 took **run226**;
/// `run218_s_word_frame_is_widened_whole` keeps the move's value diff:
/// nothing parts on 15860..16711.
///
/// ~~**17099 is `1/5`'s blocked stand.**~~ **Item 757 moved it 17099 →
/// 17128, and the mechanism was the Pyramids** (`docs/ECONOMY.md` §15):
/// who=1's `1/2026` activates on sim-frame 17084, and the original's
/// `calc_resource_caps` adds `PYRAMIDS_COMMERCE` to food and wealth from
/// 17085 (300 a good against 250) and `calc_resource_bonuses` pays
/// `PYRAMIDS_FOOD`'s fifth on the 17087 reassembly (1920 against 1600).
/// This crate had no wonder term and never read `has_wonder`. **The new
/// word's delta (this constant's comment): ours 37 draws and the original
/// 34, parting at index 30**: ours spends `Unit::do_move+0xe84` where the
/// original spends `Guy::set_anim+0x97a < Unit::do_move+0x11cf`. **Inside
/// run226** (block 17129 against its last, 17350);
/// `run226_s_word_frame_is_widened_whole` pins the new word's block and
/// keeps the move's value diff.
///
/// ~~**17128 is `1/9`'s re-plan.**~~ **Item 776 moved it 17128 → 17181,
/// and the mechanism was `invalid_loc`'s cell arm**
/// (`docs/PATHFINDER.md` §27): on the land domain, a tile whose cell is
/// flagged mountain, forest or `0x40` (`WData.flags & 0x70`) refuses when
/// the caller passes `param_3` and `param_6`, which is `valid_wcoord`'s
/// probe. This crate read the tile alone, so tick 17087's world searches
/// for `1/9` and `1/72` walked through forest-flagged cells (56, 18) and
/// (55, 19) that the original's refused, and both took other paths to
/// `1/2022`. run240 (a `WORLD` window and a packet at 17087) showed the
/// world agreeing and its trace put all 303 of the tick's priced steps
/// beside ours. **The new word's delta (this constant's comment): ours
/// 11 draws and the original 5, parting at index 0**: ours spends three
/// `Leader::create_buildings+0xffb`/`+0x1017` pairs the original does
/// not. **Inside run226** (block 17182 against its last, 17350);
/// `run226_s_word_frame_is_widened_whole` pins the new word's block (the
/// widening test) and keeps the move's value diff: nothing parts on
/// 17088..17181.
///
/// ~~**17181 is who=1's wonder offers.**~~ **Item 785 moved it 17181 →
/// 20568, and the mechanism was `wonder_mark`** (`docs/AI.md` §75):
/// `Wonders::init_wonder` raises it when a wonder activates, and
/// `create_buildings`' wonder arm refuses for an easy AI outside a wonder
/// victory once it is up. who=1's Pyramids raised it on 17085, and this
/// crate read it as zero, so on 17181 ours spent three (city, wonder)
/// pairs the original did not. **The new word's delta (this constant's
/// comment): ours 37 draws and the original 38, parting at index 31**:
/// ours spends `Guy::set_anim+0x97a < Guy::inc_time+0x271` where the
/// original spends `< Unit::move_step+0x823`, a blocked step. **Past
/// run226** (block 20569 against its last, 17350): run243 is captured
/// over [20500, 20819), sized to the word, and no dump compares a value
/// on 17351..20499. `run226_s_word_frame_is_widened_whole` keeps the
/// move's value diff: nothing parts on run226's own blocks, to 17350.
///
/// ~~**20568 is `1/40`'s blocked step.**~~ **Item 795 moved it 20568 →
/// 20800, and the mechanism was `do_move`'s TAKE** (`docs/GROUPS.md` §32):
/// once `find_path` verifies the line to the new top, the original reads
/// the stack again (`5f8c3d`–`5f8c5d`) and walks at a detour the check
/// pushed. Ours walked at the world entry under it, and `1/40` left the far
/// point on 19875 for (3912, 30984) where the original went for
/// (3732, 31380), three frames behind to the old word (run294, 19840..
/// 19999). **The new word's delta (this constant's comment): ours 36
/// draws and the original 35, parting at index 1**: ours spends
/// `Guy::set_anim+0x97a < Unit::move_step+0x823`, a blocked step, where
/// the original spends `Animal::think_bird+0x82`. **Inside run243**
/// (block 20801 against its last, 20818): ours' `1/60` stands blocked by
/// `1/64`, and its world route parts on run294's first block already.
/// `run243_s_word_frame_is_widened_whole` pins the new word's block (the
/// widening test), and `run294_s_departure_is_widened_whole` keeps the
/// move's value diff.
///
/// ~~**20800 is `1/60`'s blocked step.**~~ **Item 899 moved it 20800 →
/// 24000, the capture's own end, and the mechanism was `find_wpath`'s
/// `is_attacking`** (`docs/PATHFINDER.md` §29): `AttackOrder::is_attack`
/// answers 1, so a group's queued walk home planned while its leader's
/// current order is the `ATTACK` plans as a citizen. On 17656 ours planned
/// `1/60`'s walk home from the far point as an army and took another road
/// (run294's first block, 22 path slots); on 20800 it stood blocked by
/// `1/64` where the original's walked. **The new word's delta (this
/// constant's comment): none** — ours and the original spend the same
/// draws on every frame of run53's trace, 0..23999, and the word is the
/// trace's last frame. **The value diff on the old word's block, 20801**:
/// `1/60` reads `collide 0`, `collide_who −1`, `collide_o −1` on both sides
/// (ours was 1, 1, 64), and none of its keys parts on run294 or run243.
/// At the end block, 24001, every one of the 87 units the endpoint compares
/// stands where the original's does (`ENDPOINTS`: 14 off → 0).
/// `run80_s_word_frame_is_widened_whole` widens run80, the last blocks any
/// dump holds (the widening test), and `run243_s_word_frame_is_widened_
/// whole` and `run294_s_departure_is_widened_whole` keep the move's value
/// diff.
pub(crate) const LONG_WORD_GREAT_LAKES: i64 = 24_000;

/// The floor `run97_s_window_orders_are_the_original_s` holds — item 368's
/// widening of Great Lakes' order stacks, the 36,483 `GATHERORDER` records
/// and their neighbours that had never been compared on this map.
///
/// A floor rather than a zero, because the window opens 412 frames before
/// the word and this crate's army is off its position from 8442 already
/// (`docs/ARMY.md` §3.4's successor): a unit standing somewhere else holds
/// the order that took it there. **Measured, not guessed.**
///
/// **48,698 → 53,622 on item 385**, and the rise is the window's, not the
/// simulation's: the word moved 9182 → 9415, so the comparison now runs to
/// run97's own last complete block (9349) instead of stopping at the word,
/// 1,319 blocks against 1,152. Per block it **fell**, 42.3 → 40.7, and the
/// eight-unit set the assertion is really about did not move. A floor that
/// counts rows over a window the word controls cannot be read as a rate
/// without the block count beside it; both are printed.
/// **53,622 → 81,534 on 2026-09-21, item 465**, and every row of the
/// rise is a field that was **never compared before**. The comparator
/// stops at the order's kind: while this crate held a `MOVE_TO` where the
/// original holds a `GROUP_MOVE`, the six raiders' 6,978 order-slots
/// scored one `Kind` row apiece and nothing underneath. With the kind
/// right, `Kind` goes **6,978 → 0** and the move's own fields come into
/// view for the first time — `group_angle` 6,978, `angle` +6,978, `x`
/// 5,815, `off_x` 5,815, `y` 4,652, `off_y` 4,652 — while every family
/// that was already scoring is unchanged to the row (`Action` 1,319,
/// `PathField` 2,410, `PathLength` 1,165, `PathTo` 40,435). Four times
/// 6,978 is the whole of the +27,912.
///
/// So the number went up and the simulation did not get worse: this is
/// `run100_s_word_block_is_every_record_the_dump_carries`' own lesson one
/// record over — a widening's first run fails, because what it uncovers
/// was never being checked. ~~What it uncovered is the successor: this
/// crate's formation **slot offsets** are not the original's (`off_x`
/// 120 against 648 on `1/28`), so every member's destination is 240
/// out.~~ The offsets were the original's all along — `off_x` is
/// `x mod 0x300`, the destination said twice, so that row was the `x`
/// above it and not a slot of its own (`docs/ORDERS.md` §17.1).
///
/// **81,534 → 28,222 on 2026-09-21, item 471**, and the whole of the fall
/// is the +27,912 above coming back out plus the five members' path rows
/// with it. The six slots were the right table rotated by the wrong
/// angle: §12's probe queues its walk home at `QUEUE_LAST`, which is the
/// one queue position `action_move_near` asks `GroupData::get_loc_to` for
/// (`00704990:361`–`364`), and that answers where the leader **ends up**
/// — the farm at the other end of Great Lakes — where this crate asked
/// where it stands, found the destination under its own feet and took no
/// bearing at all. `find_angle` from the farm is the dump's
/// `group_angle 890830848` to the bit.
///
/// The residue's **set** fell with it, eight units to three: `1/27`,
/// `1/28`, `1/29`, `1/41` and `1/42` leave it entirely. `1/40` stays
/// because the group's one plan is made from *its* slot, and what is left
/// of it is the pathfinder's tie-break — same start, same end, same `x`,
/// a `y` one or two cells south over three stretches that re-converge.
///
/// **28,222 → 28,220 on item 539**: `1/33`'s two rows leave, with `1/33`
/// itself leaving the set. It is a bowman of the squad whose soft flags
/// the collision probe's stride over an empty world cell had parted
/// (`docs/COLLISION.md` §4.2).
///
/// **28,220 → 2,634 on item 899**, and every row of the fall is `1/40`'s,
/// which leaves the set: the pathfinder's tie-break this comment named was
/// the `army` mode, which the original leaves off for a walk home planned
/// while the leader's current order is the `ATTACK` (`docs/PATHFINDER.md`
/// §29). What stands is `1/23`'s `Action` and `move/angle` rows alone.
///
/// **2,634 → 0 on item 1115**: `1/23` is the caravan, and its trade order
/// carries no bit 4 and its leg faces its bearing (`docs/CARAVAN.md`
/// §11.3). run97's order records agree on every block.
pub(crate) const ORDER_RESIDUE_RUN97: usize = 0;

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

/// **Great Lakes' sixth scholar** — the frame the AI's University `1/2019`
/// finishes the job it queued on 9382, and the frame that *was* the Great
/// Lakes word from item 396 to item 442.
///
/// It is pinned as an event rather than read off
/// [`LONG_WORD_GREAT_LAKES`] because the word has now moved past it, and
/// the two run100 tests written about it — the job's own counter and the
/// word block's unit set — are about **this birth**, not about wherever
/// the headline stands. `GREAT_LAKES_SECOND_SQUAD` is the same shape one
/// mechanic over, and it is the shape `ORDER_RESIDUE_RUN97`'s own lesson
/// argues for: a test keyed on a moving headline reports the headline's
/// motion as its own failure.
///
/// **This crate births it here too, since item 442** (`docs/AI.md` §53),
/// which is what closes item 408's window: the job is queued on 9382 on
/// both sides now, and `run53`'s seating list has 9510 sixth of ten.
pub(crate) const GREAT_LAKES_SIXTH_SCHOLAR: i64 = 9_510;

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

/// **Chapter one's golden word** — the frame the golden record's draw
/// stream parts from the original's, on the basis
/// `crate::diff::golden` states: the setup borrowed from the siblings, the
/// per-frame stream refused, the script staged at `do_frame`'s entry.
///
/// **617 on item 364**, the first pin, and it is two frames past the
/// second spawn — the chapter's whole staged half is in.
///
/// Three numbers on the way there, each a thing the interpreter had to
/// model:
///
/// - **1** with nothing staged at all. The original's silenced Leader AI
///   spends twelve draws on frame 1 and this crate's spends fifty-four —
///   eight `MathUtilFuncSet::rand_int` and thirty-six
///   `Leader::produce_building`. `ai off` is the only line that matters
///   for the first six hundred frames, and the control makes it a
///   measurement rather than an observation: run104 is chapter one with
///   `0 !ai off` deleted, and **its frame 1 is this crate's own, draw for
///   draw, all fifty-four**.
/// - **610** with `ai off` modelled and nothing else — `add hoplite who=0
///   4,40`'s frame, where the original spends three `Guy::init_real+0x52`
///   draws for the squad's three units and this crate spent none.
/// - **617** with `add` modelled: `find_nearby_spot` for the spot and
///   `Objects::init_unit` for the squad, staged at `do_frame`'s entry so
///   the three figure draws are the frame's first.
///
/// - **617** stood through items 379, 384 and 386, each closing something
///   real without moving it: what it was short of was **two** draws, the
///   `Unit::fight+0x9b0` re-search roll and `Guy::set_anim+0xf2f <
///   Guy::move+0x166`, and both were one cause. `Unit::target_opportunity`
///   answers a hit on the victim's **captain**, and this crate answered on
///   the figure that took it — so `0/7` struck a frame early, its hit
///   reached `Armies::emergency`, and the army walked `1/6` off the seat it
///   should have been swinging from (item 391, `docs/COMBAT.md` §18.2).
///
/// - **618** was that second `Unit::fight+0x9b0`, `0/6`'s, and it was not
///   §17's ring at all: a melee asker with a **unit** target never reaches
///   the ring, it takes `Unit::find_melee_pos@006010b0` and returns
///   (item 392, `docs/COMBAT.md` §19). With the chase planned, `0/6`'s
///   current order is the move from 618 and `do_attack` is not reached
///   again until 650.
///
/// - **619** was that `1/7` destination, and the mechanism was **not**
///   `Group::target_opportunity`: that function only ever reaches a
///   group's *captains*, and `1/6` is the only captain in group 64. What
///   hands a squad member its captain's target is `Unit::think@005f6e40`'s
///   **first statement** — a non-captain mirrors the captain's standing
///   ATTACK order and the think ends there, so a member never searches at
///   all (item 395, `docs/COMBAT.md` §21). With it, who=1's `1/7` and
///   `1/8` carry `0/7` from the frame they are born, `1/7` is ordered to
///   the dump's own `(1320, 7800)`, and `0/6`'s ring then answers
///   `(1080, 8280)` in the live harness — the word 619 → **624**.
///
/// What stood at **624** was `Guy::set_anim+0x97a < Unit::move_step+0x823`,
/// a walking figure's step, and the value diff moved with it to 625. Two
/// residues were live on frame 618, both measured against the dump's own
/// coordinates (`docs/COMBAT.md` §21, "What stands at 624").
///
/// - **624 → 621, and this is a word paid for a destination** (item 399,
///   `docs/ARMY.md` §15.8, `docs/COMBAT.md` §22). The first of those two
///   residues is closed: `Armies::emergency` is the **city alarm's**, and
///   `Object::do_damage`'s `local_30` — the flag its call reads — has
///   exactly two writers between its `= 0` at `0064a8dc` and the test at
///   `0064bbf3`, both inside the **building** branch's city-alarm arms,
///   beside `S_CITY_BEING_ATTACKED` and `S_YOUR_CAPITAL_ATTACKED`. A unit
///   taking a hit never reaches it, so who=1's army does not tick at 618
///   and does not march. Measured on the original's own group pool
///   (`docs/RUNS.md` run109): group `64` carries `army 0` and
///   **`order_num 0` on every block 616..629** — the army issues no order
///   in the window at all.
///
///   The **value diff is why this lands with the word down.** Blocks 616,
///   617, 618 and 619 now carry the dump's own coordinates for all six
///   units, where before three of the six were on a 200-tile march from
///   618 and further wrong every frame:
///
///   | block 619 | dump | before 399 | after |
///   | --- | --- | --- | --- |
///   | `1/6` pos | `1368, 7992` | `1379, 7970` | **`1368, 7992`** |
///   | `1/6` `orders_x, orders_y` | `1368, 7992` | `38664, 13320` | **`1368, 7992`** |
///   | `1/7` pos | `1451, 7935` | `1471, 7954` | **`1451, 7935`** |
///   | `1/7` `orders_x, orders_y` | `1320, 7800` | `38760, 13224` | **`1320, 7800`** |
///   | `1/8` pos | `1332, 8121` | `1360, 8126` | **`1332, 8121`** |
///   | `1/8` `orders_x, orders_y` | `1176, 8088` | `38568, 13416` | **`1176, 8088`** |
///
///   The three frames the march was buying were a draw stream agreeing on
///   a destination the original never takes, which is the case
///   `CLAUDE.md`'s "a word that moved lands with the value diff beside it"
///   exists for.
///
/// What stood at **621** was `Guy::set_anim+0xf2f < Guy::move+0x166`, an
/// extra draw this crate spent and the original does not, under the two
/// legs that stop **171** short: `1/8` ending at `(1332, 8121)` on frame
/// 619 and `1/7` at `(1431, 7915)` on 620, where the dump walks both to
/// the end.
///
/// ~~That is the parked-collider tolerance of `docs/COLLISION.md` §5.1
/// (`other.big_radius × 3` = `0xc0`) firing on a waypoint re-taken
/// mid-leg after `resolve_unit_collision` cleared `has_waypoint`.~~
/// **Falsified on the record, item 405.** run110 already printed the
/// fields: `tolerance` is **0** and `collide`, `collide_o`,
/// `collide_who`, `collide_frame` are all clear for `1/7` and `1/8` on
/// every block 616..629 — in the dump *and* in this crate. §5.1's
/// widening never fired on either side, and nothing about the waypoint
/// re-take is implicated.
///
/// - **621 → 626: the ATTACK action under a move is a RANGED
///   attacker's** (item 405, `docs/ORDERS.md` §4.4). What ends both legs
///   short is `do_move`'s action block, which this crate entered for
///   every type. The original gates the whole of it —
///   `do_move@005f7b30:212`, `if (*(int *)(*(int *)&this->field_0x18 +
///   0x1fc) != 0)`; `SubObjectData +0x18 ptype`, `ObjectTypeData +0x1fc
///   max_range` by the type record, and `ObjectType::backup@0065fac0`
///   writes `param_1->max_range = this->field_0x1fc` in the engine's own
///   words. `1/8` stands at `(1332, 8121)` with `attack_dist` **246** to
///   `0/7` — exactly `0xf6`, the HOPLITES reach of `docs/COMBAT.md`
///   §13.2, in range by a single unit — so this crate killed the chase
///   171 short of `(1176, 8088)`; `1/7` did the same a frame later at
///   240. A melee type never abandons a leg because the target came into
///   reach.
///
///   The **value diff**, blocks 620..625, both legs and the dump's own
///   coordinates:
///
///   | block | dump `1/7` | before 405 | dump `1/8` | before 405 |
///   | --- | --- | --- | --- | --- |
///   | 620 | `1431, 7915` | `1431, 7915` | `1304, 8116` | `1332, 8121` |
///   | 621 | `1411, 7895` | `1431, 7915` | `1276, 8111` | `1320, 8136` |
///   | 623 | `1371, 7855` | `1416, 7896` | `1220, 8099` | `1320, 8136` |
///   | 626 | `1320, 7800` | `1416, 7896` | `1176, 8088` | `1320, 8136` |
///
///   After it every one of those cells is the dump's, `orders_x/orders_y`
///   included, and **all six units carry the dump's own coordinates
///   through frame 624** — `0/6` and `0/8` as well, `0/8` walking to
///   `(1044, 8076)` where it had stopped at `(972, 7988)`. That closes
///   the second residue below, which was never `0/8`'s own plan.
///
/// ~~What stands at **626** is the arrival frame itself: the original
/// spends `Guy::set_anim+0xf2f < Guy::move+0x166` there and this crate
/// does not, and the ordering of `Guy::set_anim+0x104b` beside it
/// differs. Both hoplites land exactly on their ordered points on 626 and
/// hold, so the residue is in what an arriving figure rolls, not in where
/// it arrives.~~ **Falsified by the widening, item 445**: the arriving
/// hoplites spend their rolls on 627 on both sides. The extra 626 roll is
/// `0/8`'s attack animation, and `0/8` had already parted on block 625.
/// `0/8` is the other live one: the dump snaps its
/// `orders_x/orders_y` to `(1044, 8076)` at block 625 with `collide_o 8`,
/// where this crate walks on toward `(1224, 8280)`.
///
/// - ~~**`0/8` plans its chase a frame late.**~~ **Closed by 405**: it
///   was the same short leg, and `0/8` now matches the dump frame for
///   frame from 619 to 624.
///
/// - **626 → 774: a bump from another enemy ends the chase when the
///   target is already in reach** (item 445, `docs/COMBAT.md` §48). On
///   tick 624 `0/8`, walking under its attack on `1/6`, hard-collides
///   with `1/8`. Both sides spend the blocked step's idle roll. Then
///   `Unit::resolve_unit_collision@005f9d30`'s **enemy ladder** (§6 step
///   3, arm B, `:176-187`) finds `1/6` in range and kills the move. This
///   crate had no step 3 at all: it sidestepped, counted `collide 1` and
///   walked on, so the original's `0/8` swung on 625 and rolled its attack
///   animation on 626 while this crate's walked south.
///
///   The **value diff**, block 625, `0/8`, the dump's own record:
///
///   | field | dump | before 445 | after |
///   | --- | --- | --- | --- |
///   | pos | `1044, 8076` | `1032, 8088` | **`1044, 8076`** |
///   | `orders_x, orders_y` | `1044, 8076` | `1224, 8280` | **`1044, 8076`** |
///   | orders | `ATTACK` | `MOVE_TO, ATTACK` | **`ATTACK`** |
///   | path | none | four entries | **none** |
///   | `collide`, `collide_o`, `collide_who` | `0, 8, 1` | `1, 8, 1` | **`0, 8, 1`** |
///
///   and on 626–627 its clock, `recharging 32`, `visible 2` and
///   `damage 1/6` follow. All close; the block is in
///   `chapter_one_s_word_frame_is_widened_whole`'s map.
///
/// ~~What stands at **774** is `Guy::set_anim+0x97a`, a blocked step's
/// idle roll~~ — it was not a blocked step (item 530).
///
/// - **774 → 900: the squad in danger marches alone, looks around, and
///   stands to reload** (item 530, `docs/ORDERS.md` §22). The constant's
///   delta is **+126**, and 900 is run105's trace end: no draw parts on
///   any frame of chapter one, and no word does. The block is the whole
///   capture, `chapter_one_s_word_frame_is_widened_whole` over
///   [`WIDENING_CHAPTER_ONE`]. Four links, each read off the frame the
///   last one left:
///
///   1. **617**: `Unit::set_in_danger` marks a hit unit's squad and an
///      attacker's with `unit_masks & 4`, and this crate had neither
///      writer. So on 765 the army's march gave `1/7` and `1/8` a
///      `GROUP_ATTACK_TO` where the in-danger exemption gives the
///      original's a plain `ATTACK_TO` each.
///   2. **617**: `Unit::fight` turns through `Unit::set_angle`, and a
///      group leader turning past 90° toggles the group's `facing`. This
///      crate wrote the heading bare, so the march laid `1/8`'s slot out
///      mirrored.
///   3. **773**: the attack-move looks around one frame in fifteen
///      (`do_attack_to`), and this crate's never did. `1/7` finds `0/8`.
///   4. **774**: recharging, `1/7` does not turn to it. `fight`'s
///      reloading arm asks for the idle, and that roll is the word.
///
///   The **value diff**, block 765, the dump's own record:
///
///   | field | dump | before 530 | after |
///   | --- | --- | --- | --- |
///   | `1/7`, `1/8` order | `ATTACK_TO` | `GROUP_ATTACK_TO` | **`ATTACK_TO`** |
///   | `1/7`, `1/8` `stance` | 1 | 0 | **1** |
///   | `1/8` pos | `1189, 8067` | `1179, 8056` | **`1189, 8067`** |
///   | `1/8` `orders_x, orders_y` | `38568, 13416` | `38760, 13224` | **`38568, 13416`** |
///   | `1/8` path | 54 entries from `(38544, 13407)` | 2, formation | **the dump's 54** |
///
///   And on 774, `1/7`: `ATTACK` on `0/8` above the attack-move, `near_o
///   8`, `orders_x/y` its own cell, heading unchanged. All close.
pub(crate) const GOLDEN_WORD_CHAPTER_ONE: i64 = 900;

/// **The block window chapter one's word is widened over** —
/// `chapter_one_s_word_frame_is_widened_whole`, item 445, `[first, last)`.
///
/// The floor is run105's own first block, 605, which is the only floor
/// that cannot hide a row (chapter two's 470 lesson). The ceiling is
/// run110's last block plus one: run110 (`~/ron-golden/g6`) is the same
/// game at `GUYS=9` over `[610, 630)`, and it is the only chapter-one
/// capture that prints a figure's animation clock. The word is spent in
/// that clock, and run105 at `GUYS=2` stops a `GUY` block after `ox`.
/// It was `[605, 630)` while the word stood at 626: the ceiling was where
/// the clock stops, three blocks past the word's own block of 627.
///
/// **The ceiling followed the word to 779 in the same landing**, when the
/// ladder moved it 626 → 774. It is four frames past the word, as chapter
/// two's has been. The clock rows still cover `[610, 630)` only; above
/// that the `GUY` block is run105's `GUYS=2` fields.
///
/// **And to 901 on item 530, the end of run105**, with the word 774 →
/// 900. A word at the capture's end has nothing above it to straddle, so
/// the window is the whole capture from its first block. run105 carries
/// 605..899 and its `!quit` block at 901, and the test knows block 900 is
/// the one it lacks. The map fell from 88 keys to 46.
pub(crate) const WIDENING_CHAPTER_ONE: (i64, i64) = (605, 901);

/// **Chapter two's golden word** — the ranged line, run112, item 415.
/// `docs/GOLDEN.md` §6.
///
/// Its own constant rather than a composition with chapter one's: the
/// handoff carries one `Golden:` line and how it reads with two chapters
/// pinned is the steering pass's (parked 417).
///
/// **616 → 624 on 2026-09-21, item 447** — `UnitData::is_seen` is the
/// fifth test of `ObjectData::valid_target_const` and this crate had no
/// term for it, so its hoplite captain accepted three bowmen the original
/// refused on its birth frame; and `Object::add_to_world`'s third job,
/// `update_seen(0)`, was missing from [`sim::Sim::add_unit`], so a unit
/// born on the map lit no fog at all. `docs/COMBAT.md` §31. The value diff
/// beside the move is `chapter_two_s_word_frame_is_widened_whole`: the
/// draw stream holds to 624 and the *values* part at **622**, on the
/// slinger squad's chase destination alone.
///
/// **Held at 624 on 2026-09-21, item 457** — `ObjectData::visible` and its
/// 32-frame clear landed (`docs/VISION.md` §9) and the word did not move.
/// What it moved is the sub-score: `chapter_two_s_first_attack_orders_are_
/// the_dump_s` went from six of nine rows to **nine of nine**, the dump's
/// own timeline. 624's extra draw is on the original's side and is
/// `Guy::set_anim+0x97a < Unit::move_step+0x823` — the chase *destination*
/// of §31.6's 622 residue, which is upstream of anything that field
/// reaches.
/// **637 → 645 on 2026-09-21, item 472** — `Unit::work@0060d180:440`'s
/// sixteen-frame review of a walking unit's chase, `(o + frame) % 16`,
/// and `Unit::check_target_path` under it; landed together with
/// `is_in_range`'s `mandatory` margin, which §35.3 had measured and not
/// landed. The two are a pair: the margin stops `do_move` killing the
/// chase early, the review kills it on the original's own frame.
/// `docs/COMBAT.md` §36. The value diff beside the move is
/// `chapter_two_s_word_frame_is_widened_whole`, whose map over
/// `[606, 641)` goes from **thirteen first-partings to nought** — the
/// draw stream and the values both part at `0/9` now, 645 and 646.
///
/// **645 → 680 on 2026-09-21, item 479** — `do_move@005f7b30`'s captain
/// retarget (`005f803f`-`005f8216`) and `Unit::change_target@005e36c0`
/// under it. A ranged captain walking to a target it cannot reach asks
/// every frame whether the incumbent its last search left in
/// `ObjectData::near_o` is one it *can*, on the **plain** reach where the
/// kill above it uses the reach less `0x90`; and `change_target` then
/// writes that answer down the whole `o_down` chain **in place**, which is
/// why run112's three slingers retarget on one block with two of them
/// deciding nothing. `near_o` was a field this crate did not carry at all.
/// `docs/COMBAT.md` §37. The value diff beside the move is
/// `chapter_two_s_word_frame_is_widened_whole`, whose map over the old
/// window `[606, 649)` went from **five first-partings to nought** — every
/// record run112 carries, both directions, on every frame under the new
/// word.
///
/// **680 → 683 on 2026-09-22, item 481** — and the mechanism is the
/// previous item's own nesting. `do_move@005f7b30:207` opens
/// `if (ptype->max_range != 0) { … }` and the brace closes **past** the
/// captain retarget at `005f803f`, so both the in-range kill and the
/// retarget are a ranged attacker's; 479 wrote the gate as a conjunct of
/// the kill alone and a **melee** captain reached the retarget. run112's
/// hoplite captain `1/6` carries `near_o 10` for the whole window, so this
/// crate switched all three hoplites off `0/11` down the `o_down` chain on
/// 671, where the dump has `ox 11 whom 0 uid 18` on every block of the
/// capture and never retargets them at all. `docs/COMBAT.md` §38. The
/// value diff beside the move is the same widening, whose map over
/// `[606, 684)` went from **six first-partings to nought**; the three rows
/// that stand now are past the old window's ceiling and stood with the fix
/// reverted — `extra 1/8` at 684 (the original's hoplite dies on 683 and
/// this crate's does not), and a citizen's `order 1/4`/`pos 1/4` at
/// 685/686.
/// **683 → 695 on 2026-09-22, item 491**, and it took both sides of one
/// parting. The original's draw at 683 was `Unit::close@0060ee50+0xcb6`,
/// the death animation's own roll, which this crate's `close` had never
/// taken; ours were §39's puncture pair, spent by the *second* of the two
/// arrows that land on 683 — and the original does not spend them,
/// because an arrow at a land unit carries `Ammo` flag `4` and **rolls
/// on** when it finds nothing rather than puncturing the ground. One draw
/// gained, two lost, and twelve frames. `docs/COMBAT.md` §42.
///
/// **What stands at 695**, measured rather than named — a successor reads
/// its delta here rather than asking a lane that has been reaped. Ours
/// spends **21 draws against the original's 20**, parting at draw **2**:
/// `Guy::set_anim+0x97a < Guy::inc_time+0x1ed` where the original has
/// `< Guy::inc_time+0x271`. The whole delta is one extra `+0x1ed` in the
/// attack-end column — 3 against 2, with the 12 plain wraps, the one
/// attack roll and the five `Farms::inc_time` identical — and
/// `game_random`'s own word agrees entering 695 and parts entering 696,
/// so the extra draw is spent inside the frame.
///
/// **And every value on 695 agrees except item 495's own three standing
/// rows** — `damage 1/7` 0 against 19, `damage_frac 1/7` 0 against 5,
/// `damage_frame 1/7` 0 against 685, unchanged since 686. Nothing else
/// parts on the frame; `damage 1/6` at 680 is a single block and not a
/// standing row, so (490) is not live here either, and the first *new*
/// value rows are on 696. This is a draw-stream parting rather than a
/// value one, which is a different item shape. No mechanism is named
/// here, by design.
///
/// **Item 496 named it and did not move it**, so the delta above still
/// stands and the cause is now known rather than open. The extra draw is
/// `0/6`'s: this crate's three bowmen end an attack animation together on
/// 695 and the original ends two, because the original's `0/6` **does not
/// step its clock** — `Guy::inc_time`'s zero arm under `unit_masks2 &
/// 0x10`, which `docs/ANIM.md` §5 has stated since the mechanic was
/// written and [`sim::Sim::guy_inc_time`] has never implemented. run118
/// measured the step itself (`last_time 29` against `cur_time 29` where
/// every other block has `last = cur − 1`) and run112's own `OBJECT`
/// block carries `unit_masks2 16` on `0/6` at that block.
///
/// **Item 502 landed the arm whole and the word moved 695 → 725**, thirty
/// frames, on four changes that only score together: [`sim::Sim::forget`]
/// stops clearing a unit's attack target, so the order keeps its dead
/// `1/8` from 684 to 695 as the dump does; `do_attack` gained
/// `Unit::fight`'s **follower inherit**, the block between the cell-centre
/// snap and `:196`'s `valid_target` where a non-captain takes its
/// captain's action target in place, which is how `0/7` and `0/8` strike
/// on the frame `0/6` spends searching; the invalid-target arm sets
/// [`sim::combat::umask2::NOT_FIRING`]; and
/// [`sim::Sim::guy_inc_time`] steps by **zero** while the unit carries it.
/// The first alone leaves the word here, as item 496 measured.
///
/// **What stands at 725.** Ours spends **7** draws against the original's
/// **9**, parting at draw **1**: `0/7` and `0/8` each end an attack
/// animation (`Guy::set_anim+0x97a < Guy::inc_time+0x1ed`) and the
/// original spends a second draw after each of them —
/// `Guy::set_anim+0xf2f < Guy::inc_time+0x271`, [`sim::anim::SITE_ATTACK_WRAP`],
/// the **queued attack** `Guy::inc_time`'s attack-end arm plays when
/// `queued_attack` is non-zero. So the delta is two, one per figure, and
/// it is `docs/ANIM.md` §6.2's deferred attack rather than anything in
/// §43: run118's block 696 already prints `hold_attack 1` on both of
/// them. `game_random`'s own word agrees entering 725 and parts entering
/// **726**, so both draws are spent inside the frame.
///
/// **The value diff beside the move**, on the widening's own window: the
/// map falls from thirteen rows to ten. All three `recharging` rows
/// (`0/6` at 697, `0/7` and `0/8` at 696) close; `order 0/6`, `0/7` and
/// `0/8` move **684 → 696**, twelve blocks of real agreement recovered;
/// and `chapter_two_s_hit_points`'s two sides now name the **same**
/// wounded set — this crate reaches `0/10`, which the dump wounds and it
/// did not. What survives at 696 is the parked arrow's: this crate's
/// `1/7` is unwounded (`damage 1/7` ours 0 theirs 19 from 686), so §33's
/// damage weight sends all three bowmen to `1/7` where the dump sends
/// them to `1/6`, and the `angle` rows follow the target.
/// `docs/COMBAT.md` §43.
///
/// **Item 510 measured what the two draws are and did not move the
/// word**, because what they are is the parked arrow (`docs/COMBAT.md`
/// §45). The delta stands at **7 against 9, parting at draw 1**. What
/// changed is that it now has a value diff beside it: the widening had
/// never opened a `GUY` record — `compare` builds none — and run112 is a
/// `GUYS=2` capture that prints no animation clock at all, so the record
/// the word is spent in was invisible twice over for eleven items. With
/// the clock borrowed from run118 the word's own block says it plainly:
/// at 726 the original's `0/7` and `0/8` come out of the attack-end wrap
/// on `cur_anim 11` and `12` with `hold_attack 0`, and this crate's come
/// out on the idle with `hold_attack 1`.
///
/// The mechanism, measured in both directions. `Guy::set_anim@005da300`
/// defers an attack to `hold_attack` while the figure's `des_angle`
/// differs from its `angle` and **only then** queues it to
/// `queued_attack` when an attack is already playing; only the queued one
/// is paid inside `Guy::inc_time`'s loop, and only that payment rolls.
/// The original's bowmen target `1/6`, which has stood at (1608, 8040)
/// since before 690, so their facing on 725 is the one their last swing
/// on 695 set and the attack is queued. This crate's target `1/7`, which
/// **walked** from (1800, 8472) to (1656, 8472) between 690 and 700, so
/// its attack angle on 725 is not the one 695 set and the attack is
/// held. Forcing the facing settled on 725–727 alone moves the word to
/// **743**, where the next parting is a death draw (`Unit::close+0xcb6`)
/// — also the arrow's. `docs/COMBAT.md` §42.5 is what both wait on.
///
/// **Item 495 landed the arrow and the word moved 725 → 762** — the
/// constant's delta, +37 (`docs/COMBAT.md` §46). Four things, each read
/// off the frame the last one left: the rolled shot comes down on the
/// original's step through the arc and `find_data_z` in
/// [`sim::single::Single`]; the ammo pool is stepped in slot order, so the
/// right bowman's shot rolls; `get_damage`'s height bonus reads each
/// side's `tile_z`; and `compare_target` ranks at the real bearing, where
/// the decompiler printed `find_angle(0, 0)`. On 686 every record now
/// agrees, and on 696 the bowmen take `1/6`. At 762 the original spends a
/// `Unit::fight+0x9b0` first, 8 draws against 9, and nothing parts on
/// 762 or 763; the widening's block is its window's (see
/// [`WIDENING_CHAPTER_TWO`]).
///
/// **Item 523 moved the word 762 → 900**, the constant's delta **+138**,
/// and 900 is run112's trace end: no draw parts on any frame of chapter
/// two, and no word does (`docs/COMBAT.md` §47). Three links, each read
/// off the frame the last one left. On 743 the original's `1/6` died and
/// `Unit::close` handed its squad to `1/7`, which this crate never
/// promoted, so on 762 `1/7` failed `is_captain` and skipped the
/// one-in-five roll. On 771 `0/9`'s shot struck `1/7` at the target's
/// current bearing where the original strikes at the shot's own `ex − sx,
/// ey − sy`. And the shot's landing led `1/7` by its unit speed where the
/// original leads by its first figure's `avg_speed`. The last two were
/// register pairs the decompiler dropped. The widening's block for this
/// move is the whole capture (see [`WIDENING_CHAPTER_TWO`]).
pub(crate) const GOLDEN_WORD_CHAPTER_TWO: i64 = 900;

/// **Chapter five's golden word** — the water (`docs/GOLDEN.md` §9, item
/// 535, run127). Two triremes and a fishing boat on sea region 70, the
/// first ships in any capture on this disk.
///
/// **The first walk parted at 617**, the frame after who=1's trireme is
/// born: the original spent 8 draws and this crate 6. The original's first
/// was `Guy::set_anim+0xf2f < Guy::move+0x166`, and its trailing
/// `Farms::inc_time+0x1de` was the value shift one draw makes. The widening
/// named the record on block 617. Both sides hold `1/6` attacking `0/6`
/// from the same seat, but the original's hull faces `671481856` and this
/// crate's was still turning to `−402259968`, the true bearing. The
/// original's is that bearing plus a quarter turn: a ship with `g` ("attacks
/// sideways") turns **broadside**, to the nearer side
/// (`Unit::fight@005fd4d0:698–714`, `docs/COMBAT.md` §49), so its figure
/// arrived on 617 and `Guy::move`'s arrival arm spent the draw.
///
/// **Item 535 landed the broadside and the word moved 617 → 621**, +4. On
/// 621 the original spends the landing scatter's two draws, `Ammo::init+
/// 0xcd9` and `+0xd0b`, 8 against 6: who=1's first round is in the air on
/// block 622, from a release point off the hull at (12445, 35807), and in
/// this crate's a frame later from the hull's own square.
///
/// **Item 542 moved it 621 → 664, +43** (`docs/COMBAT.md` §50). The
/// **delta**: the frame and the point were two causes, and the draw
/// named only the first. The event list holds a release at `starttime /
/// 67`, floored at 1 (`GraphicEvents::init_unit_events@008e2520`), not
/// `× 3 / 200`. The two agree on every release measured before run127 and
/// part on the Trireme's `400`: frame 5, one before this crate's 6. The
/// point is node 0 walking the keel, measured from run127 into
/// `sim::launch`'s table (piece 290, three rows). The widening then found
/// a third defect the draw never named: a sea figure's `z` is 0, not the
/// lake bed's `find_data_z`, so every trireme round had left at `sz −185`
/// against 88. With all three, the seven rounds launched through 662 agree
/// on every `AMMO` field this crate carries, `v1z` to the last digit.
///
/// **The block**, from [`WIDENING_CHAPTER_FIVE`]'s test on the new word:
/// on 664 the original spends `Guy::set_anim+0x97a < Guy::inc_time+0x271`
/// (28 draws against 27), and the only rows on block 665 are the fisher
/// `0/7`'s. Its orders are gone, `unit_masks`' packed bit is cleared, and
/// `mylos` is 4 → 6: the cast it was given at birth has ended. This crate
/// gave it no orders on 621 (parked 543), and nothing else parts under
/// the word. The window is run127 whole, so the ceiling does not move.
///
/// **Item 543 moved it 664 → 739, +75** (`docs/ORDERS.md` §23). The
/// **delta**: the fisher's cast was its **deploy**, the Fishermen's own
/// unpack `0x292` with `JOB_TIME` 40, and what issues it for a *human's*
/// boat is `Unit::think@005f6e40:163`–`199`, the rare-collector arm
/// between the caravan and the computer block. It is gated on
/// `leader_flags & 4` (human) and the packed bit, and it sits above `ai
/// off`'s exit, where this crate's only deploy, `think_fish`, sits below
/// it. The arm asks `calc_gather` and then `unpack_merchant(this, 4)`:
/// `[MOVE_TO, CAST]` on 621, arrival on 625, `spell_time` 1…39 over
/// 626–664, and on block 665 the orders gone, `packed` cleared and `mylos`
/// 4 → 6. All of it is the dump's own, and the leader's food and wealth
/// rows from 673 with it.
///
/// **The block**, from [`WIDENING_CHAPTER_FIVE`]'s test on the new word:
/// on 739 this crate spends `Guy::set_anim+0x97a < Guy::inc_time+0x1ed`,
/// an attack running out, and the original spends none (7 draws against
/// 6). No dumped record parts on 739 or 740. This crate's `1/6` is in
/// `CHAR_ATTACK3` (`anim 13`) with `end_time 3` on 739, where every swing
/// before it was `CHAR_ATTACK2`'s forty. The first row is `1/6 ammo[0]`,
/// which the dump holds alone on 742 (737 + 5, a full-length swing's first
/// release). That is the next item's hypothesis and no more. The window is run127 whole; the ceiling does not move.
///
/// **Item 549 moved it 739 → 900, +161** (`docs/ANIM.md` §4.13), and 900
/// is run127's trace end: no draw parts on any frame of chapter five. The
/// **delta**: `Guy::set_anim`'s attack arm replaces a rolled slot the
/// packet does not name with `CHAR_ATTACK2` (`get_animobj` null →
/// `cmove` of `0xc` at `0x5db279`). The Trireme's packet names `ATTACK1`
/// and `ATTACK2` only, so the original's `ATTACK3` roll on 737 played the
/// forty-frame swing, where this crate's played the three a missing slot
/// gets and ran out on 739. 543's hypothesis was the right one; the
/// length is art this crate already reads (`Art::piece_lengths`).
///
/// **The block**, from [`WIDENING_CHAPTER_FIVE`]'s test: the word is the
/// capture's end, so there is no block above it, and the window was
/// already run127 whole. The map fell from 224 keys to 30: the 29
/// standing rows, and `1/0`'s explore-order `facing` on 847, the declared
/// non-scoring formation mirror (parked 275). On 742 `1/6 ammo[0]` is in
/// both airs and all 506 of the capture's rounds agree on every field.
pub(crate) const GOLDEN_WORD_CHAPTER_FIVE: i64 = 900;

/// **Chapter six's golden word** — the air and the bird (`docs/GOLDEN.md`
/// §10, run168): **900 of 901, closed**. 900 is run168's trace end, and
/// no draw parts on any frame of the chapter.
///
/// **The delta** (item 652): 700 → 900, +200. `bird` is staged: `run_cmd`
/// case `0x52`'s `init_unit(9, BIRD)` and air patrol, at the channel's
/// cursor, through the sampling's own entry point
/// ([`crate::golden::STAGED_CURSOR`], `sim::Sim::spawn_bird_at`). The
/// cursor is **(0, 6)**, read off run169's packet at logger frame 701. The
/// `ConsoleWin`'s `mouse_coord_x/y` and the fresh `AirPatrolOrder`'s
/// waypoint agree. The bird's birth draw on 700 puts the AI scout's
/// `think_scout` roll back on the original's draw. Its edge coins land on
/// 750, 791 and 894 with the original's. At (0, 0) the walk parts on 894,
/// two frames early: `Unit::init` seats both cursors on (24, 24), and only
/// the patrol point moves.
///
/// Item 650 moved it 616 → 700 (`docs/COMBAT.md` §61). Neither aircraft's
/// idle search takes the other: `Object::poor_target`'s plane arm and
/// `valid_target_const`'s air ladder.
///
/// **The block**, from [`WIDENING_CHAPTER_SIX`]'s test on run168 whole
/// (605 to 899): the 701 rows are gone, the AI scout `1/0`'s move order
/// and path. Past the first block's 26 standing rows only the two
/// aircraft's `form` on their birth blocks part, 611 and 616. Both
/// aircraft hold their seats and no order to 899 on both sides, and no
/// round is in either air.
pub(crate) const GOLDEN_WORD_CHAPTER_SIX: i64 = 900;

/// `chapter_six_s_word_frame_is_widened_whole`'s window: **run168 whole**,
/// its first block, 605, to its last, 899 (item 652). Nothing parts
/// but the standing rows of the first block and the two aircraft's `form`
/// on their birth blocks.
pub(crate) const WIDENING_CHAPTER_SIX: (i64, i64) = (605, 901);

/// **Chapter six-b's golden word** — the air line from a base
/// (`docs/GOLDEN.md` §10, run175): **1250 of 1250, closed**. 1250 is
/// run175's trace end, and nothing parts on any frame of it: sequence
/// 1250, no value part.
///
/// **The delta** (item 680): 632 → 1250, +618. On 632 ours spent 34 draws
/// against 24, parting at draw 18 on ten `Unit::find_attack_pos+0xea9 <
/// Unit::fight+0xcb4` of the Fighter `0/6`'s. run177's packet at logger
/// frame 632 named the arm: `find_attack_pos` there answers 1 with the
/// same ten draws, and it is never called. `fight`'s captain arm runs
/// `find_new_target` drawlessly on every frame of an attack on a
/// **building** (`LAB_005fddf7`, `005fdeb4` → `005fdf50` → `005fdeea`),
/// and the Fighter's search, at its attack point, finds nothing, so
/// the attack dies (`docs/COMBAT.md` §62). With that arm built, the
/// Bomber `1/6`'s alternating `ATTACK` and `MOVE` from 700 to 1249 also
/// agrees: its search re-finds who=0's Airbase every frame.
///
/// **The block**, from [`WIDENING_CHAPTER_SIX_B`]'s test on run175 whole
/// (605 to 1250): the 633 and 634 rows are gone (the Fighter's order
/// stack, `dest_angle` and `idle`). Past the first block's 13 standing
/// rows only each aircraft's `form` and `order:target` on its birth block
/// part, 611 and 616, and the `order:target` rows are the harness's
/// (`Built::build_ids`, parked 681).
pub(crate) const GOLDEN_WORD_CHAPTER_SIX_B: i64 = 1250;

/// `chapter_six_b_s_word_frame_is_widened_whole`'s window: **run175
/// whole**, its first block, 605, to its last, 1250 (item 680). Item 651
/// pinned (605, 635) on the open word 632.
pub(crate) const WIDENING_CHAPTER_SIX_B: (i64, i64) = (605, 1251);

/// **Chapter eleven's golden word** — the guard line, an issuer the AI
/// never uses from a command (`docs/GOLDEN.md` §19, run190): **1250 of
/// 1250, closed**. 1250 is run190's trace end.
///
/// **The delta** (item 696): a new chapter, first walked at **724** with
/// the guard command's entry built (`input::group_guard` →
/// `Sim::group_action_guard`, siege filter off): 14 draws against 11 on
/// the wagon's figures, the value parting on 722 where the original's
/// supply wagon pushes the guard standing on its post. 724 → 726: the
/// land half of `detect_boat_collision` (`docs/COLLISION.md` §13.3,
/// §13.5), which a siege engine, a hero or a supply wagon takes as a ship
/// does. 726 → **734**: §4.3's escort row, a collider whose action is a
/// `GUARD` on me is soft. Both read off run191's brackets on the
/// original's own tick, not off a reading.
///
/// 734 → **1036** (item 703, `docs/COLLISION.md` §16): the collision
/// disc follows guy 0, not the unit. The wagon pushed the guard on tick
/// 732, and the original's guard, whose figures still stood on the old
/// cell, was refused its own step on tick 733 by a bit its old disc still
/// held. This crate had moved the bits with the push and stepped it
/// through. Beside it, a trackless crew guy's destination is guy 0's
/// position, so the blocked stand on tick 734 rolls the crew's idle and
/// not guy 0's: one `move_step+0x823` draw, not none. The chase to the
/// post, the stand on (3480, 12264) and the fight to 1036 then agree on
/// every row.
///
/// 1036 → **1133** (item 707, `docs/COMBAT.md` §63): **a guard's attack
/// is leashed to its post**. When the activity is a `GUARD`,
/// `Unit::fight` asks `check_target` with its guarding argument once
/// `valid_target` passes. That refuses a target past
/// `UNIT_GUARD_RESPOND_RANGE × 2 × 0x60` = 1,536 of the post. The reload
/// gate sits above it, so the guard met it on tick 1036 with `1/6` ≈1,780
/// off: the attack is killed with no draw. Its search, leashed too and
/// centred on the post, found nothing and wrote `near` −1. The same leash
/// keeps the sixteen-frame search from re-engaging (parked 705).
/// `target_opportunity`'s action gate keeps a human's `GUARD` from
/// answering the hit on 1091.
///
/// 1133 → **1139** (item 709, `docs/VISION.md` §10): **the hundredth-frame
/// resync forgets, and relights an attacker for its victims.**
/// `update_all_seen` runs on `frame % 100 == 33` and opens with
/// `World::clear_seen`; this crate skipped the clear. Its whole-disc
/// `update_seen(0)` relights a unit's `visible` cells first; this crate
/// did not. On 1033 the guard's byte still held who=1's bit, so the
/// resync kept its cell lit for who=1 and `1/6` fired on 1058, 1083 and
/// 1108 through the fog. On 1133 the byte was 0 and `1/6`'s own disc,
/// radius 4 from half-cell (8, 36), stops short of the guard's (9, 31):
/// `valid_target`'s fog test fails and the attack is dropped, drawless.
/// The clear alone fell to ~1050; the relight with it closes every 1134
/// row. A building target is seen through its `ever_seen` byte
/// (`BuildData::is_seen`), not the fog plane, which keeps Great Lakes'
/// word from falling to 9401 under the clear.
///
/// 1139 → **1250** (item 713, `docs/ANIM.md` §13): **`guy_flags & 0x20`
/// is guy 0's.** `Unit::set_in_danger` sets it on figures `0 .. guy_mark`
/// alone, and this crate held one flag for the unit. On tick 1100 the
/// guard's crew figure, re-rolling under the mirror, drew p91: `IDLE1`
/// (81 frames) with the flag, `IDLE2` (71) without. 71 runs out on block
/// 1138, tick 1138 is `do_guard`'s search arm, and the stand on 1139 is
/// the original's re-roll, `Guy::set_anim+0x97a < Unit::do_guard+0x7f4`.
///
/// **The delta**, this constant's: +111, 1139 → 1250, closed. Nothing
/// parts on any frame of run190's trace. The value rows the widening
/// dropped are its block's.
pub(crate) const GOLDEN_WORD_CHAPTER_ELEVEN: i64 = 1250;

/// `chapter_eleven_s_word_frame_is_widened_whole`'s window: **run190
/// whole**, 605 through 1250 (item 713, the word closed). The first pin
/// was (605, 726) on the word 724, then (605, 736) on 734, (605, 1038) on
/// 1036, (605, 1135) on 1133 and (605, 1141) on 1139.
pub(crate) const WIDENING_CHAPTER_ELEVEN: (i64, i64) = (605, 1251);

/// **Chapter twelve's golden word** — the follow line, an issuer the AI
/// never uses (`docs/GOLDEN.md` §20, item 714, run204): a Chariot
/// following a Supply Wagon and a Hoplite squad following a Chariot, each
/// leader walking, stopping and turning, through the DLL's `@follow`.
///
/// **The first walk, 717, open.** This crate has no follow: the harness
/// skips both `@follow` lines, so its followers stand and count `idle`
/// where the original's hold one `FOLLOWORDER` each. The sequence parts
/// first on 703, at an equal count: the original's chariot stands under
/// `do_follow` (`set_anim` at `5dac7a`) where this crate's idles
/// (`Unit::do_idle`). The value parts on 711, the chariot's first leg; the
/// count on **717**, ours 6 draws against 8, at draw 0: ours
/// `Farms::inc_time+0x1ae`, theirs two `5dac7a`s first.
///
/// 717 → **1150, closed** (item 714, `docs/ORDERS.md` §28): **the follow,
/// built.** `rondata::input::group_follow` pushes the group and calls
/// `Sim::group_action_follow`, which gives each orderable member one
/// `FOLLOW` on the leader; `Sim::do_follow` stands within `s + 0xc0` and
/// otherwise lays a `MOVE_TO` leg to the standoff point. `s` comes from the
/// follower's `los`, halved for a slower follower and again while the
/// leader walks. Nothing parts on any frame of run204's trace.
///
/// **The delta**, this constant's: +433, 717 → 1150, closed.
pub(crate) const GOLDEN_WORD_CHAPTER_TWELVE: i64 = 1150;

/// `chapter_twelve_s_word_frame_is_widened_whole`'s window: **run204
/// whole**, 605 through 1150 (item 714, the word closed). The first pin
/// was (605, 719) on the word 717.
pub(crate) const WIDENING_CHAPTER_TWELVE: (i64, i64) = (605, 1151);

/// **Chapter thirteen's golden word** — the garrison line, an issuer the
/// AI never uses from a command (`docs/GOLDEN.md` §21, item 718, run208):
/// a Chariot and a Hoplite squad garrisoning one Barracks through the
/// DLL's `@garrison`, then the building's Eject through `@eject`.
///
/// **The first walk, 640, open.** This crate cannot take either command:
/// the harness skips both `@garrison` lines and the `@eject`, so its
/// chariot stands and counts `idle` where the original's holds a
/// `GARRISONORDER` under a plain leg from 622 and walks from 623. On
/// **640** this crate spends 37 draws against 36, parting at draw 30: an
/// extra `Guy::set_anim+0x97a < Guy::inc_time+0x271`, the standing
/// chariot's idle roll, where the original's walks. The walk's value
/// compare parts on 641; the widening's first rows are on 622.
///
/// 640 → **1000, closed** (item 718, `docs/ORDERS.md` §29): **the
/// garrison command and the Eject, entered.** `rondata::input::
/// group_garrison` pushes the group and calls `Sim::group_action_garrison`,
/// one `GARRISON` a member that `can_garrison` the building;
/// `group_eject_all` defers the building's ejection. Four fixes under
/// them, each from run208's widening: `check_target_path`'s GARRISON arm
/// cuts the walk at the door on the sixteen-frame review (699, 761); the
/// door is `adjacent_to`, not a ring of tiles; `kill_garrison_order`
/// walks the captain's chain; and `come_out` keeps the unit's angles and
/// turns a member to its captain's. Nothing parts on the draw stream or
/// the walk's values to run208's end.
///
/// **The delta**, this constant's: +360, 640 → 1000, closed.
pub(crate) const GOLDEN_WORD_CHAPTER_THIRTEEN: i64 = 1000;

/// `chapter_thirteen_s_word_frame_is_widened_whole`'s window: **run208
/// whole**, 605 through 1000 (item 718, the word closed). The first pin
/// was (605, 642) on the word 640.
pub(crate) const WIDENING_CHAPTER_THIRTEEN: (i64, i64) = (605, 1001);

/// **Chapter fourteen's golden word** — the formation line, an issuer the
/// AI never uses (`docs/GOLDEN.md` §22, item 723, run210): a three-squad
/// group told Envelop standing and Line on the move through the DLL's
/// `@form`, with a right-click `@move` between.
///
/// **The first walk, 631, open.** This crate could not take the
/// formation command: the harness skipped both `@form` lines, so its nine
/// stood where the original's each hold a plain move to an Envelop slot
/// round the leader from 622 and walk from 623. On **631** this crate
/// spent 9 draws against 10, parting at draw 3: the original's leader
/// `0/6`, stopped on its slot, rolls an idle (`Guy::set_anim+0x97a <
/// Unit::do_idle+0x7d`).
///
/// 631 → **1150, closed** (item 723, `docs/ORDERS.md` §30): **the
/// formation command, entered**, and two fixes under it.
/// `rondata::input::group_form` pushes the group and calls
/// `Sim::group_action_form` — the byte on every member and a group move
/// laid out in it — which alone moved the word to 740. Then:
/// `Groups::push_group` keeps an **equal group's slot and record** (the
/// player's last pushed slot, `equals_group`), so the right-click on 701
/// lays Envelop out from the standing layout's `(ox, oy)` and slot bytes
/// and **mirrors** it, 740 → 764; and `finish_insert` replays a copied
/// group move to its **`orig`**, the group's own point, not the leader's
/// slot, 764 → 1150. Nothing parts on the draw stream or the walk's
/// values to run210's end.
///
/// **The delta**, this constant's: +519, 631 → 1150, closed.
pub(crate) const GOLDEN_WORD_CHAPTER_FOURTEEN: i64 = 1150;

/// `chapter_fourteen_s_word_frame_is_widened_whole`'s window: **run210
/// whole**, 605 through 1150 (item 723, the word closed). The first pin
/// was (605, 633) on the word 631.
pub(crate) const WIDENING_CHAPTER_FOURTEEN: (i64, i64) = (605, 1151);

/// **Chapter fifteen's golden word** — the group attack, an issuer the
/// AI rarely takes whole (`docs/GOLDEN.md` §23, item 731, run215): two
/// Hoplite squads walked in under a right-click `@move`, told to attack a
/// who=1 Chariot through the DLL's `@attack`, then an attack-move on the
/// ground through `@amove`.
///
/// **The first walk, 753, open.** This crate could not take the attack
/// command: the harness skipped `@attack`, so its six walked on under the
/// right-click's group move where the original's each hold an
/// `ATTACKORDER` on `1/6`, `mandatory 1`, over an approach leg from 736.
/// The charge spends no draw until **753**, where this crate spends 6
/// against 7, parting at draw 0: the original's extra `Guy::set_anim
/// +0x97a < Unit::move_step+0x823`.
///
/// 753 → **1250, closed** (item 731): **the attack command, entered.**
/// `rondata::input::group_attack` pushes the group and calls
/// `Sim::group_action_attack` with `mandatory` 1, one `ATTACK` a member
/// with the action bit, and that alone took the word to run215's trace
/// end: sequence 1250, no value part. The attack-move needed nothing:
/// `input::group_move_to` already took `orders` 2 into
/// `GroupAttackToOrder`s. The pool's widening then found `push_group`'s
/// fresh slot unstamped, which `Groups::copy_group@006fa690` stamps with
/// the frame; it spends no draw.
///
/// **The delta**, this constant's: +497, 753 → 1250, closed.
pub(crate) const GOLDEN_WORD_CHAPTER_FIFTEEN: i64 = 1250;

/// `chapter_fifteen_s_word_frame_is_widened_whole`'s window: **run215
/// whole**, 605 through 1250 (item 731, the word closed). The first pin
/// was (605, 755) on the word 753. Its pool half, `widen_pool`, reads
/// who=0's `GROUPDATA` on the same blocks.
pub(crate) const WIDENING_CHAPTER_FIFTEEN: (i64, i64) = (605, 1251);

/// **Chapter sixteen's golden word** — explore and flee, the move
/// issuer's trailing selector (`docs/GOLDEN.md` §24, item 738, run219): a
/// Chariot and a Hoplite squad told to explore through the DLL's
/// `@explore` and to flee through `@flee`, `issue_move_to` with `orders` 3
/// and 4, each explorer walking past a goody box.
///
/// **The first walk, 838, open.** The crate took both selectors from the
/// start (`input::group_move_to` with `orders` 3 and 4), and every class,
/// box leg and box opening agreed. It parted on the **re-issue behind the
/// box leg**: `Group::finish_insert@0070e620` replays the copied explore
/// to its `orig`, the click, and this crate, which carried no `orig` on a
/// plain move, replayed it to the snapped `dest`. The original's chariot
/// stands on its click on 838 and goes idle — two `Guy::set_anim+0x97a <
/// Unit::do_idle+0x7d` rolls, 7 draws against this crate's 5, parting at
/// draw 0 — while this crate's walks on to (2424, 17304).
///
/// 838 → **1250, closed** (item 738): **a plain move's `orig`, carried.**
/// `MoveOrder::orig` takes the click in `action_move_near`'s plain arm,
/// and `finish_insert` replays a copied move to it. That alone took the
/// word to run219's trace end: sequence 1250, no value part.
///
/// **The delta**, this constant's: +412, 838 → 1250, closed.
pub(crate) const GOLDEN_WORD_CHAPTER_SIXTEEN: i64 = 1250;

/// `chapter_sixteen_s_word_frame_is_widened_whole`'s window: **run219
/// whole**, 605 through 1250 (item 738, the word closed). The first pin
/// was (605, 840) on the word 838. Its pool half, `widen_pool`, reads
/// who=0's `GROUPDATA` on the same blocks.
pub(crate) const WIDENING_CHAPTER_SIXTEEN: (i64, i64) = (605, 1251);

/// **Chapter seventeen's golden word** — the flight line (`docs/GOLDEN.md`
/// §25, item 746, run223): a Fighter and a Bomber pair sent home to a
/// staged Airbase through the DLL's `@flight`, and at who=1's Barracks
/// through `@strike`, both `CommandManager::issue_flight@00941d40`.
///
/// **The first walk, 642, open.** The harness skips both verbs by name,
/// since this crate does not enter the flight command. On 642, the first
/// frame of `0/6`'s `StrafeOrder` home, the original spends the Fighter's
/// `cruising_alt` redraw — `Random::get` at `Unit::do_air_physics+0xba`
/// (`0x5e878a`), which a non-bomber throws when `(o + frame) & 7 == 0` —
/// 7 draws against this crate's 6, parting at draw 0. On 644 this crate
/// spends an idle roll for the Fighter, standing without an order, that
/// the original does not.
///
/// **The command entered, 642 held** (item 746, `docs/ORDERS.md` §32):
/// `input::group_flight` → `Sim::group_action_flight` gives each aircraft
/// its `StrafeOrder` home on its processed block, whole and compared, and
/// the refused strike nothing. The word stays on the draw: the flight
/// itself, `do_air_physics` and the landing, is not built, and a
/// `cruising_alt` draw without the step it belongs to would move the word
/// with the positions already parted.
///
/// **The flight home, flown: 805, open** (item 759, `docs/ORDERS.md`
/// §33). `Unit::do_strafe` takes a flight home into `do_air_physics`:
/// the Fighter's redraw on every eighth frame, `check_fuel`'s approach,
/// the returning bank, `pitch_aircraft`, the step, and `land_plane` into
/// its base. On 642 every aircraft agrees whole — `0/6`'s point (11664,
/// 16262), heading, bank 10, pitch 2 and altitude 2, the pair still on
/// their pads — and so does every redraw to 714, the Fighter's landing on
/// 722 and the pair's flight home on 662–665. The word is the first bomb:
/// on 805 the original spends `Guy::set_anim+0xf2f < Unit::set_anim+0x56
/// < Unit::do_strafe+0x9d0`, 5 draws against 4 at draw 0, the patrol's
/// strafe on `1/2006` going to `CHAR_ATTACK2`.
///
/// **The pair's patrol, flown: 821, open** (item 763, `docs/ORDERS.md`
/// §34). `do_strafe` turns the flying strike on the unseen Barracks into
/// an `AirPatrolOrder` over its point and `work` flies it the same frame
/// (666); the patrol flies through `do_air_physics`' non-returning arms,
/// and a plane's step now lights the fog as it flies (`set_new_location`'s
/// ring pass), so the Barracks is first seen between the pair's searches
/// on 760 and 776. The search pushes the strike `QUEUE_FIRST` on 776 and
/// 777 — with no `update_action`, so `orders_x/y` keep the frame's own —
/// and `0/8` bombs on 805 (`do_strafe+0x9d0`, `recharging 31`), all as
/// the original. The word is the bomb's landing: on 821 the original
/// spends `Object::take_damage+0xe1 < Object::do_damage < Ammo::do_damage`,
/// 5 draws against 4 at draw 0, and the Barracks is at `damage 45` on
/// block 822 where ours is 0. The bomb's round is released by the
/// animation's event (`anim.rs`'s `guy_release_events`, which fires for
/// an `ATTACK`/`ATTACK_GROUND` front order only), fenced this tranche.
///
/// **The strafe's round: 1400, closed** (item 770, `docs/ORDERS.md` §35).
/// A strafe with a target releases: `Guy::execute_events`' `+0xdc` arm
/// takes a `StrafeOrder`'s own `ox/whom/uid` (its `UnitOrder` vtable at
/// `0xb47b08` answers `+0x18` 1, `+0x2c` 0), and the bomb leaves from the
/// plane's figure `z` and one of its two bays, lands one tile ahead along
/// the heading with no draw, and falls in `(int)sqrtf(2(ez − sz) /
/// GRAV_Z)`, 17 frames. On 821 the original's `take_damage+0xe1` is ours
/// too, and the stream agrees to run223's last frame: sequence 1400, no
/// value part. The widening, moved to run223 whole, has the Barracks'
/// `damage` agreeing on every block to its death on 1080; what stands is
/// the tank from 1212 (parked 765), which spends no draw to 1400. run235
/// (`AMMO=5`) diff-backs every bomb's record on 800–1100.
///
/// **The delta**, this constant's: +579, 821 → 1400, closed.
pub(crate) const GOLDEN_WORD_CHAPTER_SEVENTEEN: i64 = 1400;

/// `chapter_seventeen_s_word_frame_is_widened_whole`'s window: **run223
/// whole**, (605, 1401), since item 770 closed the word at 1400; (605,
/// 823) since item 763, the word 821 and the bomb's damage on 822;
/// (605, 807) since item 759, the word 805 and the bomb on 806; (605,
/// 667) before it, the word 642 and the pair's strafe on 662 and strike
/// on 666 (item 746); the first pin was (605, 645). Its pool half,
/// `widen_pool`, reads who=0's `GROUPDATA` on the same blocks.
pub(crate) const WIDENING_CHAPTER_SEVENTEEN: (i64, i64) = (605, 1401);

/// **Chapter eighteen's golden word** — the build line, an issuer the AI
/// takes through its own planner (`docs/GOLDEN.md` §26, item 779,
/// run241): **1450 of 1450, closed**. Sequence 1450, no value part.
///
/// **The first pin, 642, open**: the harness skipped both `@build` lines,
/// the command having no entry into this simulation. On 642 this crate
/// spent 7 draws against 6, parting at draw 0 on an idle citizen's roll
/// (`Guy::set_anim+0x97a < Guy::inc_time+0x271`) that the original's,
/// walking to its site, does not spend.
///
/// **The command entered, 1450, closed** (item 779, `docs/ORDERS.md`
/// §36): `input::group_build` → `Sim::group_action_build` places and pays
/// for the site once and swarms the group at `QUEUE_NEW` — a `MOVEORDER`
/// for a human's builder and the `BUILDORDER` behind it — and the
/// one-unit swarm asks whose builder it is, so `find_build_spot`'s help
/// on 1097 is a `MOVEORDER` too. The walk agrees to run241's end: the
/// ring spots, the first frames of construction on 709 and 721,
/// `construct_hits` block for block, the completions on 948 and 1141.
///
/// **The delta**, this constant's: +808, 642 → 1450, closed.
pub(crate) const GOLDEN_WORD_CHAPTER_EIGHTEEN: i64 = 1450;

/// `chapter_eighteen_s_word_frame_is_widened_whole`'s window: **run241
/// whole**, (605, 1451), since the command entered; (605, 645) at the
/// first pin, the word 642. Its pool half, `widen_pool`, reads who=0's
/// `GROUPDATA` on the same blocks.
pub(crate) const WIDENING_CHAPTER_EIGHTEEN: (i64, i64) = (605, 1451);

/// **Chapter nineteen's golden word** — the cast line, a Spy's Informer
/// on an enemy building (`docs/GOLDEN.md` §27, item 790, run245).
/// ~~669, open~~ with the harness skipping `@spell`: an idle Spy's
/// animation roll where the original's walks to its ring spot. **1100,
/// closed**, the spell command entered (`docs/ORDERS.md` §37).
pub(crate) const GOLDEN_WORD_CHAPTER_NINETEEN: i64 = 1100;

/// `chapter_nineteen_s_word_frame_is_widened_whole`'s window.
pub(crate) const WIDENING_CHAPTER_NINETEEN: (i64, i64) = (605, 1101);

/// **Chapter twenty's golden word** — the board line: the transport
/// toggle and the move it gates (`docs/GOLDEN.md` §28, item 803, run249).
/// **1300, closed on the first walk**: the command's entry,
/// `input::group_set_transport` over `Sim::set_transport`, landed with the
/// chapter before the capture, and the stream agrees to run249's end — the
/// toggle, both plans, the unflagged Chariot stopping at the shore, the two
/// Transport casts and barges, and the disembark on the east bank.
///
/// **The delta**, this constant's: 1300 at its first pin, closed.
pub(crate) const GOLDEN_WORD_CHAPTER_TWENTY: i64 = 1300;

/// `chapter_twenty_s_word_frame_is_widened_whole`'s window: **run249
/// whole**, (605, 1301). Its pool half, `widen_pool`, reads who=0's
/// `GROUPDATA` on the same blocks.
pub(crate) const WIDENING_CHAPTER_TWENTY: (i64, i64) = (605, 1301);

/// **Chapter twenty-one's golden word** — the repair line: a player's
/// right-click on a damaged building (`docs/GOLDEN.md` §29, item 813,
/// run255). ~~1141, open, on the first walk~~: the command's entry,
/// `input::group_swarm_around` over `Sim::group_swarm_around`, and piece
/// 120's launch bays landed with the chapter before the capture, and the
/// stream agreed through the whole repair line — the arrows, the peace,
/// both swarms, the repair on 930–931 and the three late orders dying on
/// 968, 969 and 1023 — to the idle citizens' own gather walks, where on
/// 1141 the original's `0/7` spent a walk step's animation draw beside
/// `0/8`'s and ours did not. **1300, closed** (item 824): a human's found
/// gather drops its group (`think_peasant@005f5760`'s `+0x80` write at
/// `5f590c`), so `0/7`'s camp approach on 1131 is no longer refused the
/// spot `0/8` was sent to by `find_ordered_collision`'s own-group arm —
/// `MOVEORDER` `x` 4104 here against 4296 there, from `group` 0 here
/// against −1 there on 1130.
///
/// **The delta**, this constant's: +159, 1141 → 1300, closed.
pub(crate) const GOLDEN_WORD_CHAPTER_TWENTY_ONE: i64 = 1300;

/// `chapter_twenty_one_s_word_frame_is_widened_whole`'s window: **run255
/// whole**, (605, 1301). Its pool half, `widen_pool`, reads who=0's
/// `GROUPDATA` on the same blocks.
pub(crate) const WIDENING_CHAPTER_TWENTY_ONE: (i64, i64) = (605, 1301);

/// **Chapter twenty-two's golden word** — the launch line: a strike from
/// inside a base (`docs/GOLDEN.md` §31, item 836, run265). ~~778, open, on
/// the first walk~~: the original spent the launched Fighter's
/// `cruising_alt` redraw (`do_air_physics+0xba`), 5 draws against 4, with
/// `0/6`'s stack parted on 768 (a `STRAFEORDER` laid inside there, none
/// here). **923, open** (item 836, the launch line built): the strike
/// laid inside, the tank's gate, `do_launch`, the EXIT at an Airbase and
/// the flight agree through 797; on 798 the climb's `pitch` parts, 40.0
/// here against 38.0 there, and on 923 the original's Fighter plays its
/// attack (`Guy::set_anim+0xf2f < Unit::set_anim+0x56`), 6 draws against
/// 5 at draw 0. **1500, closed** (item 842, `docs/ORDERS.md` §39): the
/// Fighter's type strafes (`w`, `unit_flags & 0x400000`), so on its strike
/// `pitch_aircraft` wants half the `cruising_alt` — 798's `pitch` 38.0 on
/// both sides — and its round takes no scatter, so 923 is `set_anim` and
/// five `Farms::inc_time` on both. The stream agrees to run265's end.
///
/// **The delta**, this constant's: +577, 923 → 1500, closed.
pub(crate) const GOLDEN_WORD_CHAPTER_TWENTY_TWO: i64 = 1500;

/// `chapter_twenty_two_s_word_frame_is_widened_whole`'s window.
pub(crate) const WIDENING_CHAPTER_TWENTY_TWO: (i64, i64) = (605, 1501);

/// **Chapter twenty-three's golden word** — the repeat line: an Airbase's
/// repeat toggled off between two landings (`docs/GOLDEN.md` §32, item
/// 867, run281). **1840, closed, on the first walk**, with the
/// `@buildmask` line skipped: the toggle, the landings off the
/// non-repeating base and `0/6`'s kill at its full tank draw nothing, so
/// the stream agrees to run281's end (it is run265's game on all 1,501
/// frames the two share). What parts is values, by the widening: 1442,
/// `0/2007`'s `build:repeat_air` (ours 1, theirs 0); 1489 and 1513, `0/7`'s
/// and `0/8`'s kept patrol here against none there. **The command entered
/// in the same item** (`Sim::action_buildmask`, the repeat button's toggle)
/// closes all three; the word does not move.
///
/// **The delta**, this constant's: 1840 on the first walk, closed.
pub(crate) const GOLDEN_WORD_CHAPTER_TWENTY_THREE: i64 = 1840;

/// `chapter_twenty_three_s_word_frame_is_widened_whole`'s window:
/// **run281 whole**, (605, 1841). Its pool half, `widen_pool`, reads
/// who=0's `GROUPDATA` on the same blocks.
pub(crate) const WIDENING_CHAPTER_TWENTY_THREE: (i64, i64) = (605, 1841);

/// **Chapter twenty-four's golden word** — the queue line: a Barracks'
/// infinite queue toggled on between two finishes (`docs/GOLDEN.md` §33,
/// item 877, run285). **855, open, on the first walk**, with both
/// `@queueup` lines skipped and `@buildmask` 0x40 writing nothing: on 855
/// the original's Hoplites finish and train a squad this crate does not
/// make (it queued nothing), and the draws part there. What parts before
/// it is values, by the widening: 622, `0/2007`'s `queued` (ours 0,
/// theirs 1) and who=0's food and timber (254 and 241 here, 203 and 203
/// there); 642, wealth (114 against 61); 856, the three Hoplites the dump
/// holds alone. **1560, closed** (the commands entered in the same item:
/// `input::group_queue_up` → `Sim::action_queue_up`, the 0x40 toggle in
/// `Sim::action_buildmask`, and `do_queue`'s re-queue,
/// `Sim::requeue_infinite`, `docs/PRODUCTION.md` "The infinite queue"):
/// the three finishes, the re-queue and its refusal agree, and the stream
/// agrees to run285's end.
///
/// **The delta**, this constant's: +705, 855 → 1560, closed.
pub(crate) const GOLDEN_WORD_CHAPTER_TWENTY_FOUR: i64 = 1560;

/// `chapter_twenty_four_s_word_frame_is_widened_whole`'s window: ~~run285
/// from its first block to two past the first walk's word, (605, 858)~~
/// **run285 whole**, (605, 1561), once the word closed. Its pool half,
/// `widen_pool`, reads who=0's `GROUPDATA` on the same blocks.
pub(crate) const WIDENING_CHAPTER_TWENTY_FOUR: (i64, i64) = (605, 1561);

/// **Chapter twenty-five's golden word** — the cancel line: a Barracks'
/// queue cancelled inside a run, across two types, on an infinite queue
/// and from the end (`docs/GOLDEN.md` §34, item 884, run292). **855,
/// open, on the first walk**, with the four `@unqueue` lines skipped: the
/// head Hoplite the original cancelled on 761 finishes here and trains a
/// squad the original never makes, and the draws part there. What parts
/// before it is values, by the widening: 702, `0/2007`'s `queued` (ours
/// 3, theirs 2) and who=0's food and timber (157 and 121 here, 210 and
/// 162 there); 842, `build:infinite_queue` (ours 1, theirs 0). **1466,
/// closed** (the cancel entered in the same item: `input::unqueue` →
/// `Sim::action_unqueue`, `docs/PRODUCTION.md` "The player's cancel"):
/// the four arms, the Bowmen's finish without a re-queue and the last
/// Hoplite's agree, and the stream agrees to run292's end.
///
/// **The delta**, this constant's: +611, 855 → 1466, closed.
pub(crate) const GOLDEN_WORD_CHAPTER_TWENTY_FIVE: i64 = 1466;

/// `chapter_twenty_five_s_word_frame_is_widened_whole`'s window: ~~run292
/// from its first block to two past the first walk's word, (605, 858)~~
/// **run292 whole**, (605, 1467), once the word closed. Its pool half,
/// `widen_pool`, reads who=0's `GROUPDATA` on the same blocks.
pub(crate) const WIDENING_CHAPTER_TWENTY_FIVE: (i64, i64) = (605, 1467);

/// **Chapter twenty-six's golden word** — the research line: a
/// technology through the player's command at who=0's Library
/// (`docs/GOLDEN.md` §35, item 883, run296). **1492, closed, on the first
/// walk**, with a technology's `@queueup` skipped: a research spends no
/// draw, and the Hoplites are queued on both sides. What the research
/// moves is values, by the widening: 622, `0/2005`'s `queued` (ours 0,
/// theirs 1) and who=0's food (254 and 134); 652, timber and wealth; 822,
/// `epochs`, `epoch[0]` and `discovered`; 1023, `epoch[3]`; 1024, `0/0`'s
/// `mylos`; 1242, `epoch[2]`; 1243, the commerce cap. **The research
/// entered** in the same item (`input::group_queue_up` →
/// `Sim::action_queue_research`, `docs/PRODUCTION.md` "The player's
/// research"): every one of those rows agrees, and the word stays closed.
///
/// **The delta**, this constant's: none, 1492 closed on the first walk
/// and after the build; the widening's value rows are the move.
pub(crate) const GOLDEN_WORD_CHAPTER_TWENTY_SIX: i64 = 1492;

/// `chapter_twenty_six_s_word_frame_is_widened_whole`'s window: **run296
/// whole**, (605, 1493). Its pool half, `widen_pool`, reads who=0's
/// `GROUPDATA` on the same blocks.
pub(crate) const WIDENING_CHAPTER_TWENTY_SIX: (i64, i64) = (605, 1493);

/// **Chapter twenty-seven's golden word** — the upgrade line: a unit
/// upgrade through the player's command at who=0's Barracks, and the
/// finish that converts the line (`docs/GOLDEN.md` §36, item 901, run300).
/// **1102, open, on the first walk**: on 922 `Leader::gain_tech`'s queue
/// loop re-targets the queued Slingers entry to Javelineers in place
/// (`queue[0].type`, ours 82 and theirs 83), which this crate does not do,
/// so the entry trains a Slinger squad here on 1102 and the draws part
/// there, the births' `Guy::init`.
///
/// **1560, closed** (the queue loop built in the same item,
/// `Sim::retarget_queued_to`, `docs/TECH.md` "The queue loop"): the entry
/// is re-targeted on 922, both squads are Javelineers on the original's
/// blocks, and the stream agrees to run300's end.
///
/// **The delta**, this constant's: +458, 1102 → 1560, closed.
pub(crate) const GOLDEN_WORD_CHAPTER_TWENTY_SEVEN: i64 = 1560;

/// `chapter_twenty_seven_s_word_frame_is_widened_whole`'s window: ~~run300
/// from its first block to two past the first walk's word, (605, 1105)~~
/// **run300 whole**, (605, 1561), once the word closed. Its pool half, `widen_pool`, reads who=0's
/// `GROUPDATA` on the same blocks.
pub(crate) const WIDENING_CHAPTER_TWENTY_SEVEN: (i64, i64) = (605, 1561);

/// **Chapter twenty-eight's golden word** — two buildings under one
/// command: `Group::action_queue_up`'s sort and passes across two
/// Barracks, the infinite toggle on two, and the command's building group
/// of two in the pool (`docs/GOLDEN.md` §37, item 888, run304).
///
/// **1580, closed, on the first walk**: a pool slot spends no draw, and
/// the queues and bits agree on every block. The widening parts on 642,
/// who=0's slot 0 `held` — the command's building group of two, which
/// `Sim::push_command_buildings` did not seat — and on 825 and 876 the
/// trained squads' `group`, one slot off.
///
/// **The delta**, this constant's: the first pin, 1580, closed.
pub(crate) const GOLDEN_WORD_CHAPTER_TWENTY_EIGHT: i64 = 1580;

/// `chapter_twenty_eight_s_word_frame_is_widened_whole`'s window: run304
/// whole, (605, 1581). Its pool half, `widen_pool`, reads who=0's
/// `GROUPDATA` on the same blocks.
pub(crate) const WIDENING_CHAPTER_TWENTY_EIGHT: (i64, i64) = (605, 1581);

/// **Chapter twenty-nine's golden word** — the repeat launch: chapter
/// twenty-two's game with no toggle, its three kept patrols relaunched
/// under the bit at their full tanks, and a Biplane trained at the
/// Airbase (`docs/GOLDEN.md` §38, item 915, run308).
///
/// **1745, open, on the first walk**: on 1746 the Biplane `0/9` trained
/// at the Airbase is out on the EXIT's point here and idling (ours alone
/// `Guy::set_anim+0x97a < Unit::do_idle+0x7d` on 1745's tick), and inside
/// `0/2007` there with no order. The three relaunches (1585, 1789, 1813)
/// agree whole.
///
/// **2070, closed** (item 915, `Build::train@0062f9b0`'s `CARRY_AIR` arm,
/// `docs/PRODUCTION.md` "The trained aircraft"): the Biplane stays in the
/// base, and the stream agrees to run308's end.
///
/// **The delta**, this constant's: +325, 1745 → 2070, closed.
pub(crate) const GOLDEN_WORD_CHAPTER_TWENTY_NINE: i64 = 2070;

/// `chapter_twenty_nine_s_word_frame_is_widened_whole`'s window: run308
/// whole, (605, 2071). Its pool half, `widen_pool`, reads who=0's
/// `GROUPDATA` on the same blocks.
pub(crate) const WIDENING_CHAPTER_TWENTY_NINE: (i64, i64) = (605, 2071);

/// **Chapter thirty's golden word** — the gather point (item 928,
/// `docs/GOLDEN.md` §39, run312).
///
/// **822, open, on the first walk** (`@gatherpoint` skipped): theirs 7
/// draws against ours 6, theirs alone `Guy::set_anim+0x97a <
/// Unit::do_idle+0x7d` — the Citizen `0/10` idling at its Woodcutter,
/// which it walked to under the City's gather point from 760, where here
/// it stands south of the City with no order. The first value parting is
/// 618, 2007's list.
///
/// **1450, closed** (item 928, `docs/PRODUCTION.md` "The gather point":
/// `Sim::action_gather_point`, `come_out`'s gather block and routing, and
/// `Build::train`'s "inside" arm): the stream agrees to run312's end.
///
/// **The delta**, this constant's: +628, 822 → 1450, closed.
pub(crate) const GOLDEN_WORD_CHAPTER_THIRTY: i64 = 1450;

/// `chapter_thirty_s_word_frame_is_widened_whole`'s window: run312 whole,
/// (605, 1451). Its pool half, `widen_pool`, reads who=0's `GROUPDATA` on
/// the same blocks.
pub(crate) const WIDENING_CHAPTER_THIRTY: (i64, i64) = (605, 1451);

/// **Chapter thirty-one's golden word** — the gather point's other arms
/// (item 955, `docs/GOLDEN.md` §40, run338).
///
/// **740, open, on the first walk**: theirs 12 draws against ours 10, ours
/// alone `Guy::set_anim+0x97a < Unit::do_idle+0x7d` — the Citizen `0/11`
/// idling at its ground point here and still walking to it there, from
/// (3960, 31368), where the lone arm's re-seat left it: `FILTER_ALL`
/// counts the seeker (falsifier 3). The first value parting is 718,
/// `0/11`'s `pos`.
///
/// **1400, closed** (item 955, `docs/PRODUCTION.md` "The gather point"):
/// `FILTER_ALL` counts the seeker (740 → 884); the squad's target sweep
/// is a squad placement with its own default span (884 → 892 → 1400);
/// with the waypoints, the third re-seat, and each re-seat's spot as the
/// leg's origin. The stream agrees to run338's end.
///
/// **The delta**, this constant's: +660, 740 → 1400, closed.
pub(crate) const GOLDEN_WORD_CHAPTER_THIRTY_ONE: i64 = 1400;

/// `chapter_thirty_one_s_word_frame_is_widened_whole`'s window: run338
/// whole, (605, 1401). Its pool half, `widen_pool`, reads who=0's
/// `GROUPDATA` on the same blocks.
pub(crate) const WIDENING_CHAPTER_THIRTY_ONE: (i64, i64) = (605, 1401);

/// **Chapter thirty-two's golden word** — an Airbase's gather point
/// (item 947, `docs/GOLDEN.md` §41, run344).
///
/// **1751, open, on the first walk**: the stream parts where run344's
/// Biplane, launched on 1747 on a patrol over the list, flies and this
/// crate's sits inside with no order. The first value parting is 1602,
/// the three planes' stacks (the hangar loop of `Build::add_gather_point`).
///
/// **2360, closed** (item 947, `docs/PRODUCTION.md` "The gather point"):
/// the hangar loop and `clear_gather`'s half, `Build::train`'s `CARRY_AIR`
/// arm, and a patrol that holds its points and walks them. The stream
/// agrees to run344's end.
///
/// **The delta**, this constant's: +609, 1751 → 2360, closed.
pub(crate) const GOLDEN_WORD_CHAPTER_THIRTY_TWO: i64 = 2360;

/// `chapter_thirty_two_s_word_frame_is_widened_whole`'s window: run344
/// whole, (605, 2361). Its pool half, `widen_pool`, reads who=0's
/// `GROUPDATA` on the same blocks.
pub(crate) const WIDENING_CHAPTER_THIRTY_TWO: (i64, i64) = (605, 2361);

/// **Chapter thirty-three's golden word** — an Airbase's launch issuers
/// (item 976, `docs/GOLDEN.md` §42, run358).
///
/// **2287, open, on the first walk**: theirs 5 draws against ours 4,
/// theirs alone `Unit::do_air_physics+0xba` — the Biplane `0/9`'s
/// altitude redraw on `(9 + 2287) & 7 == 0`, flying the patrol over P1
/// the Airbase's launch patrol gave it on 2282, where here it stands inside
/// with no order. The first value parting is 2262, `0/8`'s stack: the
/// launch strike.
///
/// **2740, closed** (item 976, `docs/PRODUCTION.md` "The launch
/// commands": `Sim::group_action_launch_patrol`,
/// `group_action_launch_flight`, action 3's launch at an Airbase and
/// `do_strafe`'s escort). The stream agrees to run358's end, and so does
/// every word of the value stream.
///
/// **The delta**, this constant's: +453, 2287 → 2740, closed.
pub(crate) const GOLDEN_WORD_CHAPTER_THIRTY_THREE: i64 = 2740;

/// `chapter_thirty_three_s_word_frame_is_widened_whole`'s window: run358
/// whole, (605, 2741). Its pool half, `widen_pool`, reads who=0's
/// `GROUPDATA` on the same blocks.
pub(crate) const WIDENING_CHAPTER_THIRTY_THREE: (i64, i64) = (605, 2741);

/// **Chapter thirty-four's golden word** — the launch commands' other
/// arms (item 1009, `docs/GOLDEN.md` §43, run362).
///
/// **2322, open, on the first walk**: theirs 16 draws against ours 15,
/// theirs alone `Unit::do_air_physics+0xba` — `0/7`'s flight home to
/// `0/2008`, which the right-click on that base gave it on 2307, where here
/// it stands inside with no order. The first value parting is 2307,
/// `0/7`'s stack.
///
/// **2850, closed** (item 1009, `docs/PRODUCTION.md` "The launch
/// commands": `action_launch_flight`'s `MOVE_TO` arm, and the base's
/// height at 0 or above in the approach home). The stream agrees to
/// run362's end, and so does every word of the value stream.
///
/// **The delta**, this constant's: +528, 2322 → 2850, closed.
pub(crate) const GOLDEN_WORD_CHAPTER_THIRTY_FOUR: i64 = 2850;

/// `chapter_thirty_four_s_word_frame_is_widened_whole`'s window: run362
/// whole, (605, 2851). Its pool half, `widen_pool`, reads who=0's
/// `GROUPDATA` on the same blocks.
pub(crate) const WIDENING_CHAPTER_THIRTY_FOUR: (i64, i64) = (605, 2851);

/// **Chapter thirty-five's golden word** — the Helicopter's and missiles'
/// launch arms (item 1019, `docs/GOLDEN.md` §44, run371).
///
/// **2445, open, on the first walk**: theirs 7 draws against ours 4 —
/// `Unit::do_spec_anim`'s two for a Helicopter's exit (`5e59ef`,
/// `5e5a0f`) and `Guy::set_anim+0x97a < Unit::do_idle+0x7d`: the first
/// Helicopter, `0/11`, trained at `0/2008` with no gather point, comes out
/// at once and idles, where here it stays inside. The first value parting
/// is 2227, the silo's queue (one V2 there, two here).
///
/// **2675, open** (item 1019, `docs/PRODUCTION.md` "The Helicopter and
/// the missile under a point": the Helicopter's exit at its birth and its
/// two draws, the first-point block of `Build::train`, the Helicopter's
/// attack-move from `add_air_patrol_order`, and the silo's one missile):
/// theirs 2 draws against ours 1, theirs alone `Guy::set_anim+0x97a <
/// Guy::move+0x19f` — the second Helicopter, `0/12`, out of `0/2008` on
/// 2674 on its attack-move to P_h, whose figure stands a frame before it
/// walks there and walks at once here. The first value parting is 2659,
/// `0/12`'s birth seat (parked 646's family).
///
/// **2701, open** (item 1048, `docs/PRODUCTION.md` "The Helicopter's
/// flight"): the planners' straight path for a type that flies like a
/// helicopter (`find_path`, `find_wpath`, `find_tpath`), `Unit::work`'s
/// separation of two of one type within `0x180`, and the figure's climb
/// in `Guy::set_new_location`. `0/12` flies straight to P_h and `0/11` is
/// pushed off it, both to the unit on every block; `0/12`'s figure, put
/// on its unit by the push, takes the arrival stand on 2675. Theirs 4
/// draws against ours 3 on 2701, theirs first `Ammo::init+0xae8` and
/// `+0xb25`: the V2's round off the silo (parked 1050, 1051). The first
/// value parting is 2702, `0/10`'s round.
///
/// **3260, closed** (item 1050, `docs/PRODUCTION.md` "The missile's
/// launch and round"): the flight command's missile arm and
/// `add_strafe_order`'s head, an `AIRATTACKGROUNDORDER` on `1/2006`'s point;
/// the silo's `recharging` 30 at the launch and `do_missile_launch`'s
/// countdown; the missile out on the silo's own point, its one step, and
/// `Ammo::init`'s missile arm — the launch offset, the doubled scatter
/// (`Ammo::init+0xae8`/`+0xb25` on 2701, `s` 22), the spline's 120 frames
/// — and its end, `Object::die` with no death draw. The stream agrees to
/// run371's end, and so does every word of the value stream. What the
/// widening still holds of the V2 is its blast: on 2821 the dump's
/// Barracks `1/2006` is gone, where here it stood at 400 damage of 1200.
/// **Item 1077 built it, and the word does not move**: the round has no
/// target, and `Ammo::check_hit`'s `find_building_at` on its landing tile
/// makes the Barracks its own, struck whole rather than as a fringe
/// (`docs/COMBAT.md` §73). The widening's V2 row on 2821 is gone (43 → 42,
/// its block in `chapter_thirty_five_s_word_frame_is_widened_whole`).
///
/// **The delta**, this constant's: +559, 2701 → 3260, closed; item 1077,
/// +0.
pub(crate) const GOLDEN_WORD_CHAPTER_THIRTY_FIVE: i64 = 3260;

/// `chapter_thirty_five_s_word_frame_is_widened_whole`'s window: run371
/// whole, (605, 3261). Its pool half, `widen_pool`, reads who=0's
/// `GROUPDATA` on the same blocks.
pub(crate) const WIDENING_CHAPTER_THIRTY_FIVE: (i64, i64) = (605, 3261);

/// **Chapter thirty-six's golden word** — the missile's other arms
/// (`docs/GOLDEN.md` §45, item 1078, run390): chapter thirty-five whole,
/// two more silos with a V2 each, a strike pressed twice on one silo, the
/// redraw on the V2's launch frame, and `MISSILE_DEFENSE_BONUS` at the
/// blast and at the order.
///
/// **3111, open, on the first walk** (with `action_launch_flight`'s
/// missile pass-over already built, item 1078): theirs spends no draw of
/// V2c `0/15`'s, which the shield refused an order on 3081; ours ordered
/// it and fires its two scatter draws on its launch, 3111. The value
/// stream's first parting under the word is 2722, the V2's price.
///
/// **3169, open** (item 1078, `docs/PRODUCTION.md` "The missile's other
/// arms"): a missile that fires leaves `num_units` (`Unit::close`'s
/// `track_unit_type(·, −1)`), so the two new V2s are priced 100 and 120
/// as in run390; and the shield refuses a missile's order on its holder,
/// so V2c `0/15` stays inside with no order on 3082. The word is V2b's
/// round coming down on T_home `1/2007` in who=1's land: run390 closes
/// it under the shield, and this crate wounds the Barracks (its first
/// wound's `% 100`). The widening goes 284 → 266 rows, all past 3169.
///
/// **3420, closed on the draw stream** (item 1078): the shield closes a
/// missile's round in its holder's land (`Sim::land`'s missile arm,
/// `678337`..`67845d`), so V2b's round comes down on T_home on 3169 and
/// strikes nothing, as in run390. The stream agrees to run390's end, and
/// so does every frame's word (`values at None`). The widening goes 266
/// → 59 rows: chapter thirty-five's forty-two, and the new units' `form`
/// and the V2s' seats inside their silos (parked 646's family).
///
/// **The delta**, this constant's: the first walk, 3111; +58, 3169;
/// +251, 3420, closed.
pub(crate) const GOLDEN_WORD_CHAPTER_THIRTY_SIX: i64 = 3420;

/// `chapter_thirty_six_s_word_frame_is_widened_whole`'s window: run390
/// whole, (605, 3421).
pub(crate) const WIDENING_CHAPTER_THIRTY_SIX: (i64, i64) = (605, 3421);

/// **Chapter thirty-seven's golden word** — the nuke (`docs/GOLDEN.md`
/// §46, item 1091, run397): a cast of its own on the golden start, the
/// nuke researched at a silo and trained, its strike with the shield given
/// in the countdown, a round with no scatter, and forty frames of
/// `Nuke::do_damage`.
///
/// **3490, closed, on the first walk with the nuke built** (item 1091):
/// the draw stream agrees to the window's end, and so does every frame's
/// word. Unbuilt, the walk read **3081**, the V2's doubled scatter's two
/// draws on the nuke's launch, which the original never spends (a nuke's
/// scatter is 0, `67c64b`), and the first value parting on 3082, the
/// round's landing off the target's point.
///
/// **The delta**, this constant's: 3081 → 3490.
pub(crate) const GOLDEN_WORD_CHAPTER_THIRTY_SEVEN: i64 = 3490;

/// `chapter_thirty_seven_s_word_frame_is_widened_whole`'s window: run397
/// whole, (605, 3491).
pub(crate) const WIDENING_CHAPTER_THIRTY_SEVEN: (i64, i64) = (605, 3491);

/// **Chapter thirty-eight's golden word** — the air line under fire
/// (`docs/GOLDEN.md` §47, item 1102, run404): two Bombers flown at a who=1
/// Barracks past a Radar Air Defense, an Anti-Aircraft Battery and an
/// Infantry squad, each flak round's roll by the Bomber's altitude, and
/// both Bombers shot down.
///
/// **742, the first walk** (item 1102): the Anti-Aircraft Battery's first
/// attack spends `Unit::fight`'s jam roll, `GameAccess::rnd(100)` against
/// `jam_unit_radar_prob` (`5fee89`), which this crate did not.
///
/// **776, open** (item 1102, the jam roll built in `Sim::swing_anim`):
/// theirs starts a guy's walk (`Guy::set_anim+0xf2f < Guy::move+0x166`)
/// that ours does not — the Infantry squad's, whose `myspeed` has read 32
/// against 34 since its birth on 621, and whose army move has parted in
/// its offsets since 765. The flak roll and the crash are built beside
/// it: the Battery's first two rounds, misses on 753 and 755, agree field
/// for field with their flags.
///
/// **777, open** (item 1109). 776's draw was the **Battery's**, not the
/// Infantry's: `Guy::move+0x166` is the queued attack's
/// `set_anim(CHAR_ATTACK1, 0, 1)`. On 773 the Battery re-attacks Bomber
/// `0/7` while its hull is still turning, and theirs keeps the unit's angle
/// (`Unit::fight`'s `5fe81f`: an `ANTI_AIR` shooter at an air-domain target),
/// where ours' pivot test failed and re-headed it. Built with it: the
/// gunpowder foot line's `×34/32` in `Unit::update_speed` (the Infantry's
/// `myspeed` 34 from 621) and `do_move`'s Modern Infantry `×5/4` (its first
/// step 42 on 765). 777: the Radar Air Defense acquires `0/7` and theirs
/// winds up (`recharging` 1..19, `Wall::inc_time`), drawing nothing, where
/// ours fires at once — ours 10 draws, theirs 6, four `buildings` first.
///
/// **779, open** (item 1112, `docs/COMBAT.md` §84). The Radar Air
/// Defense's round is its animation's: `Build::do_attack` neither counts
/// down nor fires for an `ANTI_AIR` building other than a Lookout or an
/// Observation Post, and `Wall::inc_time` winds `recharging` up to 19 and
/// swings it −1..−10, the `<UNIT>`'s release on frame 2 a round. The value
/// diff: block 778 prints `recharging 1`, `attack_ox 7`, `attack_whom 0`
/// on both sides, and ours spends no draw on 777 (six, the farms', as
/// theirs). 779: the Battery `1/9` releases `CHAR_ATTACK2`'s frame-4 round
/// on node 0 in ours; theirs holds it, its turret short of its aim
/// (`node_flags` 14, bit 0 clear) — ours 9 draws, theirs 6.
///
/// **794, open** (item 1117, `docs/COMBAT.md` §85). The release gate is
/// the event's: `execute_game_events` holds a pivot piece's release on
/// node `n` while `node_flags` lacks bit `n & 3`, whether or not the
/// piece's `get_position` vectors are measured, and the Battery's are
/// not. The value diff on block 780, both sides: `1/9` at `cur_anim 12`,
/// `cur_time 4`, `node_flags 14`, `des_node_flags 1`, and no round of
/// `1/9` in the air (ours had one). 782's node-1 round is released on
/// both sides; it leaves from the figure's square in ours. 794 is `1/7`'s
/// walk, parted since its `ATTACKTO` point on 765 (parked 1113): ours is
/// still walking at (21185, 17322) and spends `Guy::set_anim+0x97a <
/// Unit::move_step+0x823`, where theirs has stood at (20604, 16968) since
/// 777 — ours 12 draws, theirs 8.
///
/// **878, open** (item 1113, `docs/GROUPS.md` §34). 794's walk was two
/// Modern Infantry mechanisms. `Form::compute_dests` scatters a
/// modern-infantry **follower** off its captain by the destination, its
/// object number and its list index (`72d454`): block 765's points, both
/// sides, `1/7` (38568, 13368) and `1/8` (38712, 13176), where ours stood a
/// cell past each (off_y 360 against 312). And `Unit::do_move` **packs** a
/// walking one every 128 frames on its phase `(o · 0x11 + frame) & 0x7f ==
/// 0` (`5f82df`): block 778, both sides, `1/7` at (20604, 16968) with
/// `cur_anim 23`, `retry 22`, `attempts −3` — theirs's stand from 777,
/// which ours walked through into the Battery. run404's twenty packs agree.
/// 878: ours shoots `0/7` down (`Unit::close+0xcb6`,
/// `Ammo::init_crash+0x305`), theirs does not until 1019 — ours 7 draws,
/// theirs 5. Walked back: the Radar Air Defense's target on 837
/// (`attack_ox` 7 against 6, the draw site `+0x432` against `+0x463`),
/// `0/7`'s patrol leg on 820, its hit a frame late on 784 (parked 1125).
///
/// **1762, closed** (item 1131, `docs/COMBAT.md` §8.6, §12.3, §46.2): the
/// draw stream agrees to the window's end, and so does every frame's word.
/// Four listing reads, each walked back from a first parted field:
/// - `compare_target`'s armed-building ×5 is a **computer** attacker's
///   (`64f1a2`): ours gave it to the human `0/7`, whose strafe re-pointed
///   at the Radar on tick 818 (block 820, `path[0].to` (22272, 16512)
///   against (21120, 16512)). The word **fell to 842**.
/// - an object's own `z` is `find_tcoord_z` clamped at 0 (`00606598`):
///   ours doubled every hit on a plane over the river (the Radar's 64
///   against 32 on 802). **906**.
/// - a building's hit records its own `o` as `damage_o` (block 829, 2007).
/// - `Build::do_attack` re-finds an unordered building's target on every
///   call (`622a3f`), and a dead target is not cleared: block 837, both
///   sides, the Radar's `attack_ox 6`; block 1007, both sides,
///   `attack_ox 6` after `0/6`'s crash, and 7 on 1008. **1007, then 1762**.
///
/// **The delta**, this constant's: 878 → 1762.
pub(crate) const GOLDEN_WORD_CHAPTER_THIRTY_EIGHT: i64 = 1762;

/// `chapter_thirty_eight_s_word_frame_is_widened_whole`'s window: run404
/// whole, (605, 1763).
pub(crate) const WIDENING_CHAPTER_THIRTY_EIGHT: (i64, i64) = (605, 1763);

/// **Chapter thirty-nine's word** (item 1111, `docs/GOLDEN.md` §48,
/// run422): the spell issuer's untargeted crafts.
pub(crate) const GOLDEN_WORD_CHAPTER_THIRTY_NINE: i64 = 1100;

/// `chapter_thirty_nine_s_word_frame_is_widened_whole`'s window: run422
/// whole, (605, 1101).
pub(crate) const WIDENING_CHAPTER_THIRTY_NINE: (i64, i64) = (605, 1101);

/// **Chapter forty's word** (item 1167, `docs/GOLDEN.md` §49, run430):
/// the casts' other arms.
pub(crate) const GOLDEN_WORD_CHAPTER_FORTY: i64 = 1150;

/// Chapter forty-one's golden word (items 1182 and 1200,
/// `docs/GOLDEN.md` §50, run437): **1770, closed** — 839 → 1770 (item
/// 1200), sequence and values. 839 was ours 4 draws against 3: the
/// Biplane over a wood (`TData` `0x7138`), where `is_in_range@006486b0`'s
/// world-cell test answers no. Past it the strafe's re-point on 852
/// (`find_new_air_target`), the round's own target struck whoever owns it
/// (`1/2003`'s first wound, 858), `guy_radius` in the splash (`0/7` 21
/// sixteenths, not 11), an aircraft's `check_hit` passing its own side
/// (969), the Biplane's nose guns (970's flight, five frames not six),
/// the tail's dry-tank kill (1108), the trireme's turned nodes (1460)
/// and a strafer's round that never rolls (1529).
pub(crate) const GOLDEN_WORD_CHAPTER_FORTY_ONE: i64 = 1770;

/// Chapter forty-two's golden word (item 1209, `docs/GOLDEN.md` §51,
/// run460): a ring of who=1's Barracks, `Unit::resolve_block`'s arms —
/// **1160, closed** on the first take, sequence and values. It stood on
/// 819 once, a value word: the peace walk's move `dest_x`/`dest_y`
/// (42234, 20790) against (42720, 20064), tolerance 96 against 0 —
/// `TAKE` wrote the new top before its line was verified (`do_move:694`).
pub(crate) const GOLDEN_WORD_CHAPTER_FORTY_TWO: i64 = 1160;

/// `chapter_forty_two_s_word_frame_is_widened_whole`'s window: run460
/// whole, 605 to its end.
pub(crate) const WIDENING_CHAPTER_FORTY_TWO: (i64, i64) = (605, 1161);

/// Chapter forty-three's golden word (item 1223, `docs/GOLDEN.md` §52,
/// run466): the landing's arms no walk held — **2200, closed**, sequence
/// and values.
///
/// **The delta: 1356 → 1552, item 1235** — the squad boards whole
/// (`Unit::go_inside@0061a2e0`'s `o_down` walk, `docs/TRANSPORT.md` §17).
/// The value diff on block 1357: `0/8` and `0/9` `inside` 10 on both
/// sides (ours had −1), and `0/9`'s `orders.len` 0 on both (ours had 1,
/// its `orders_x` 11928 against the original's 7870).
///
/// **1552 → 1901, item 1248: a dead gatherer leaves its chain.** 1552 was
/// ours 31 draws against 30, parting at index 27: ours' who=1 Citizen
/// `1/2` rerolled its chop at `Unit::do_non_flat_gather+0xcc3` where the
/// original set −1 and walked home on 1553. The state first parted on
/// block 1488: `1/2001`'s `gather_down` ours 6 (the Citizen `1/6`, dead
/// on 1487) against 2 — `Unit::close` closes a dead unit's orders, and
/// the gather arm's `remove_gatherer` takes it off the chain — and
/// `Build::all_gathering` prunes before it walks. On 1488 `gather_down`
/// is 2 on both sides, and on 1553 `1/2`'s `order:gather.wait` −1 on
/// both.
///
/// **1901 → 2200, closed, the same item: the DLL's refusal 3.** 1901 was
/// ours 4 draws against 3, parting at index 0 with the barge `0/10`'s
/// `Guy::set_anim+0x97a < Guy::move+0x19f`: ours moved `0/10` ashore
/// on `1900 @move 0 15360 31200 10 11`, and the original's trace refuses
/// the whole line for the `0/11` it never made. The value diff on block
/// 1902: `0/7`..`0/9` `inside` 10 on both sides (ours had −1, put
/// ashore), and `0/10` at (14572, 31200) on both to the capture's end.
pub(crate) const GOLDEN_WORD_CHAPTER_FORTY_THREE: i64 = 2200;

/// **Chapter forty-four** (item 1254, `docs/GOLDEN.md` §53): run484, the
/// idle search's head on a computer's fleet. The computer's Bomb Vessel
/// `1/6` takes the human's Barracks `0/2007` across the regions and out
/// of its range on 610, its first idle think, and both sides agree on
/// the attack, the 19-frame closing walk and the first strike from 630;
/// the human's Bomb Vessel `0/6` refuses the computer's `1/2007` on both
/// sides to the capture's end.
///
/// ~~**The word was 670**~~ (item 1254): ours 9 draws against 10 at
/// index 3, where the original spends `Object::take_damage+0xe1`, the
/// first round landing on `0/2007`.
///
/// **Item 1257: the chapter closes at 1450**, the capture's end, word,
/// sequence and values. The round flew 40 frames here against 39 because
/// it left from the ship's own square: `1/6`'s release goes through
/// `get_position`'s pivot branch (`docs/COMBAT.md` §55) with the Bomb
/// Vessel's piece 296, which `sim::pivot` had no rows for, and at
/// `fast_angle_to_degrees` of its facing, 353, not `angle_to_degrees`'
/// 354. The value diff on block 633, the round's first: `sx, sy, sz`
/// ours `(21288, 14712, 0)` against `(21301, 14867, 134)`, `total_time`
/// 40 against 39, `angle` −2123169792 against −2120286208, `ex` 21154
/// against 21155; all agree now. The turret on block 631, ours 94568448
/// against 96862208, was the same piece's missing pivot node.
pub(crate) const GOLDEN_WORD_CHAPTER_FORTY_FOUR: i64 = 1450;

/// **Chapter forty-five's golden word** (`docs/GOLDEN.md` §54, item
/// 1268, run492): **closed at 1650**, the capture's end — word, sequence
/// and values — `get_cost`'s library-line tail on the golden start. The
/// value diff: who=0's food 770 → 668 on block 802 (The Art of War, 120
/// less Despotism's 15 %), → 578 on 804 (City State, 120 less Dye's
/// 25 %), and 579 → 519 with timber 753 → 693 on 806 (Barter whole), on
/// both sides. The two Nubian arms the staging met (`Sim::unit_hits`,
/// `economy::calc_rare`'s `nubian`) were built in the same item: the
/// Merchant `0/6`'s `myhits` ours 90 against 135 from 603, and who=0's
/// `resources[3:knowledge]` 160 against 240 from 769, both agree.
pub(crate) const GOLDEN_WORD_CHAPTER_FORTY_FIVE: i64 = 1650;

/// **Chapter forty-six's golden word** (`docs/GOLDEN.md` §55, items
/// 1278 and 1291, run496): **closed at 2200**, the capture's end — word,
/// sequence and values; it opened at 1902, the squad's landing, where
/// ours drew 3 against 2 (ours `0/8`'s `Guy::set_anim+0x97a <
/// Guy::move+0x19f`, the original's `Farms::inc_time+0x1ae`). The value
/// diff on block 1902, both sides now: `0/7`, `0/8` and `0/9` hold one
/// `GroupMoveOrder` each (group 1901300, `form_id` 0, 1, 2); `0/7` at
/// (14712, 31224), `0/8` at (14712, 31368), `0/9` at (14904, 31224) —
/// ours had (14856, 31224) — and `0/8`'s and `0/9`'s heading 1084948480,
/// `0/7`'s own on 1901, where ours had the boat's 1073741824. Item 1291:
/// a captain's `come_out` takes its squad ashore, each member on the
/// captain's ring, so `eject_contents` gives the squad arm once.
pub(crate) const GOLDEN_WORD_CHAPTER_FORTY_SIX: i64 = 2200;

/// **Chapter forty-seven's golden word** (`docs/GOLDEN.md` §56, item
/// 1310, run514): **open at 712**, word and sequence; the chapter opens
/// with a parting there, ours 35 draws against 31, at index 25: ours
/// spends four in the buildings phase — the Keep site `0/2008`'s round,
/// fired unfinished and landing on who=1's `1/6` — where the original's
/// next draw is `Farms::inc_time+0x1ae` (seed `0x8fad03fd`). The value
/// diff on block 713: `1/6`'s `hits:damage` ours 8 against 0,
/// `damage_frame` 712 against 0, `hits_left` 112 against 120.
///
/// **712 → 838** (item 1323): a site does not shoot — `Build::process`
/// returns before `do_attack` unless `WallData::is_active` (`flags & 4`),
/// `docs/COMBAT.md` §8.6. Now ours 6 draws against 7 at index 1: the
/// original spends `Guy::set_anim+0x97a < Unit::do_idle+0x7d` where ours'
/// next is `Guy::set_anim+0x97a < Guy::inc_time+0x271` (seed
/// `0x3de49d86`). The value diff, walked back to block 838: the Citizen
/// `0/9`'s `orders.len` ours 1 against 0, `orders_x/orders_y` (3864,
/// 36888) against (3840, 36864).
///
/// **838 → 904** (item 1330): the builders' walk. `Unit::init` bears a
/// Citizen in form 9, so the three walk plain moves to the Mob's rings
/// (`form.rs`'s `MobRing`, from the listing), laid out unmirrored because
/// `do_build`'s turn toggles the leader's group mirror through
/// `Unit::set_angle` (`docs/GROUPS.md` §6.3, §6.4, §24.3). Now ours 35
/// draws against 32 at index 25 (seed `0x041e0e77`): ours spends `1/6`'s
/// `Guy::set_anim+0xf2f < Guy::inc_time+0x271`, where the original's next
/// is the bird's `Guy::set_anim+0x104b`. The value diff on block 905:
/// `1/6`'s `order:kind` ours 10 against 2, `orders.len` 2 against 1,
/// `orders_x/orders_y` (7368, 33816) against (38664, 13320), `near`
/// (2008, 0) against (−1, −1); the site `0/2008` is ours alone from 902
/// (the original's died on 901).
///
/// **904 → 1400, closed** (item 1350): the capture's end — word, sequence
/// and values. The site died late because its blows were light: walked
/// back to block 706, its first blow, `0/2008`'s `damage`/`damage_frac`
/// ours 3/10 against 4/5 and `job_counter` 9900 against 9850. A
/// building's `armor()` is `WallData::armor@0063fa60`, which halves it
/// while the building is not `is_active` (`63fb31`–`63fb41`): the Keep
/// site's 4 is 2, a Hoplite deals 13 where ours dealt 11. And a site loses
/// `lost × 50` of progress, `lost` with the sixteenths' carry
/// (`Object::take_damage@00652020`, `65230b`, `65239b`): on 738 the
/// original took 250 and ours 200. Both built (`Sim::armor_of`,
/// `Sim::damage_building`); the site dies on 901 on both sides.
pub(crate) const GOLDEN_WORD_CHAPTER_FORTY_SEVEN: i64 = 1400;

/// **Chapter forty-eight's golden word** (item 1358, `docs/GOLDEN.md`
/// §57, run551): ~~**1450**~~, ours 13 draws against 11, walked back to
/// block 935, the Elite Pikemen's first blow on the Tower site `1/2006`:
/// `damage`/`damage_frac` ours 71/10 against 72/5, 122 sixteenths against
/// 133.
///
/// **Item 1375: 1481.** The site stands on who=0's claim, and
/// `get_damage` step 18 asks `WallData::in_unfriendly_territory@0063eca0`
/// (the target's vslot `+0x184`), which zeroes its armour; this crate
/// never set the flag and dealt at the site's halved armour 2. With it the
/// blow is 133 on both sides, the site finishes on 1447 on both, and the
/// word walks to **1481**, ours 4 draws against 8, parting at index 1:
/// the original spends the Tower's second shot (`64cc85`, `64ccb6`,
/// `Ammo::init+0xcd9`, `+0xd0b`) where ours spends `Guy::set_anim+0x104b`.
/// The value diff, walked back: block 1451, the Tower's first shot on the
/// Scout `0/7`, `ammo[0].total_time` ours 7 against 16 and `v1z` ours
/// 2.849108 against 69.087502, its ends agreeing. No mechanism is named.
///
/// **Item 1380: 1750, closed.** `BuildData::get_shot@0062dd90` is 0 for a
/// Tower in the first three ages (the owner's `ages` ≤ 2), and
/// `Ammo::init@0067bbf0`'s building arm then flies the arrow at
/// `unit_move_speed × 0x5a` — 16 frames over 1,463 units where the type's
/// `PROJ_SPEED` 200 made ours 7. The word walked to 1503 on that alone;
/// the Scout's lead (`0067cf1a`, shared by a unit shooter and a building)
/// was the second, the Tower's second shot landing 374 and 572 units
/// short of the original's on 1482. With both the walk reaches the end of
/// the trace.
///
/// **The delta**, this constant's: +269, 1481 → 1750 (the trace's end).
pub(crate) const GOLDEN_WORD_CHAPTER_FORTY_EIGHT: i64 = 1750;

/// Chapter forty-nine's golden word (item 1393, run576, `docs/GOLDEN.md` §58).
///
/// **Closed at the trace's end by item 1403.** The word stood at 934 on a
/// Scout's order stack: `Unit::target_opportunity@005fffc0`'s group arm
/// (`param_3 == 0`, `group >= 0`, type not combat-role, `GroupData::member`)
/// hands a hit on a group member to `Group::target_opportunity` and
/// returns, so a Scout walked by `@move` takes its fourth wound on 912
/// with no `FLEE_TO` where this crate fled it out of the Tower's range.
///
/// **The delta**, this constant's: +566, 934 → 1500 (the trace's end).
pub(crate) const GOLDEN_WORD_CHAPTER_FORTY_NINE: i64 = 1500;

/// `chapter_forty_nine_s_word_frame_is_widened_whole`'s window: run576 whole.
pub(crate) const WIDENING_CHAPTER_FORTY_NINE: (i64, i64) = (605, 1501);

/// **Chapter fifty** (item 1404, `docs/GOLDEN.md` §59, run577): two
/// Stockades at the first age and a Scout led through their range by a
/// group attack-move, a Hoplite squad behind it.
///
/// **Closed at the trace's end by item 1419.** The word stood at 819 (686
/// before item 1415): the Hoplite captain `0/7` took an `ATTACK` on the
/// Stockade `1/2006` on 700 under `Group::target_opportunity`, because
/// `Object::valid_target` asks the target's `is_seen`, and a building's is
/// `visible & (1 << who)` first. `Build::do_attack@006228f0` (`622b5b`..
/// `622bd0`) writes it after each round it fires (`1/2006` prints
/// `visible 1` from 687); this crate never set it, so the Stockade, 26 tiles
/// from the Hoplite and never in the human's sight, was no target.
///
/// **The delta**, this constant's: +281, 819 → 1100 (the trace's end).
pub(crate) const GOLDEN_WORD_CHAPTER_FIFTY: i64 = 1100;

/// `chapter_fifty_s_word_frame_is_widened_whole`'s window: run577 whole.
pub(crate) const WIDENING_CHAPTER_FIFTY: (i64, i64) = (605, 1101);

/// `chapter_forty_eight_s_word_frame_is_widened_whole`'s window: run551
/// whole, 605 to its end (block 1749 is the one the dump does not carry).
pub(crate) const WIDENING_CHAPTER_FORTY_EIGHT: (i64, i64) = (605, 1751);

/// `chapter_forty_seven_s_word_frame_is_widened_whole`'s window: run514
/// whole, 605 to its end (block 1399 is the one the dump does not carry).
pub(crate) const WIDENING_CHAPTER_FORTY_SEVEN: (i64, i64) = (605, 1401);

/// `chapter_forty_six_s_word_frame_is_widened_whole`'s window: run496
/// whole, 605 to its end (block 2199 is the one the dump does not carry).
pub(crate) const WIDENING_CHAPTER_FORTY_SIX: (i64, i64) = (605, 2201);

/// `chapter_forty_five_s_word_frame_is_widened_whole`'s window: run492
/// whole, 600 to its end (block 1649 is the one the dump does not carry).
pub(crate) const WIDENING_CHAPTER_FORTY_FIVE: (i64, i64) = (600, 1651);

/// `chapter_forty_four_s_word_frame_is_widened_whole`'s window: run484
/// whole, 605 to its end (block 1449 is the one the dump does not carry).
pub(crate) const WIDENING_CHAPTER_FORTY_FOUR: (i64, i64) = (605, 1451);

/// `chapter_forty_three_s_word_frame_is_widened_whole`'s window: run466
/// whole, 605 to its end (block 2199 is the one the dump does not carry).
pub(crate) const WIDENING_CHAPTER_FORTY_THREE: (i64, i64) = (605, 2201);

/// `chapter_forty_one_s_word_frame_is_widened_whole`'s window: run437
/// whole, 605 to its end.
pub(crate) const WIDENING_CHAPTER_FORTY_ONE: (i64, i64) = (605, 1771);

/// `chapter_forty_s_word_frame_is_widened_whole`'s window: run430 whole,
/// (605, 1151).
pub(crate) const WIDENING_CHAPTER_FORTY: (i64, i64) = (605, 1151);

/// `run265_s_rounds_are_the_original_s_record_for_record`'s window
/// (item 853): run265's blocks from the Fighter's first release, 923, to
/// the capture's end.
pub(crate) const RUN265_ROUNDS: (i64, i64) = (920, 1501);

/// `run235_s_bombs_are_the_original_s_record_for_record`'s window
/// (item 770): run235's blocks from the first bomb's release, 805, to its
/// last, 1100 — the four attacks on the Barracks and its death on 1080.
pub(crate) const RUN235_BOMBS: (i64, i64) = (800, 1101);

/// **Chapter ten's golden word** — the patrol line, an issuer the AI
/// never uses (`docs/GOLDEN.md` §18, run184): **1250 of 1250, closed**.
/// 1250 is run184's trace end, and nothing parts on any frame of it:
/// sequence 1250, no value part.
///
/// **The delta** (item 693): 640 → 1250, +610. On 640 this crate spent 37
/// draws against 36, parting at draw 30 on an extra `Guy::set_anim+0x97a
/// < Guy::inc_time+0x271`: the harness skipped both `@patrol` lines, the
/// command having no entry into this simulation, so the Chariot `0/6`
/// idled, animating, where run184's walked its first leg. The same item
/// built the patrol (`docs/ORDERS.md` §27) — `input::group_patrol`,
/// `Sim::group_action_patrol`, `add_patrol_order`, the leader's
/// `do_patrol` legs, `redo_patrol_order` in a group `QUEUE_FIRST` — and
/// the walk agrees to the capture's end: the chariot's four turns and the
/// squad's three, block for block.
///
/// **The block**, from [`WIDENING_CHAPTER_TEN`]'s test on run184 whole:
/// past the first block's 26 standing rows, the births' `form`; the
/// patrol and first-leg ids on 622 and 642, a pushed group's id (parked
/// 689); and the scout `1/0`'s `facing` from 847 (parked 275). Item 693
/// pinned 640 on its first walk, with the missing patrols on 622 and 642.
pub(crate) const GOLDEN_WORD_CHAPTER_TEN: i64 = 1250;

/// `chapter_ten_s_word_frame_is_widened_whole`'s window: **run184
/// whole**, its first block, 605, to its last, 1250 (item 693). The first
/// pin was (605, 642) on the open word 640.
pub(crate) const WIDENING_CHAPTER_TEN: (i64, i64) = (605, 1251);

/// **Chapter nine's golden word** — the move line, the first issuer
/// chapter (`docs/GOLDEN.md` §17, run180): **1100 of 1100, closed**. 1100
/// is run180's trace end, and nothing parts on any frame of it: sequence
/// 1100, no value part.
///
/// **The delta** (item 676): 693 → 1100, +407. On 693 the original spent
/// one `Unit::do_move+0xe84` first in the frame, 7 draws against 6: the
/// Chariot `0/6`, walking a plan laid **straight through the unseen
/// sand** of region 65, re-planned round the lake a cell short of it.
/// Two human-only arms of the pathfinder were missing. `invalid_loc@
/// 00607c30`'s fog arm makes a tile valid when its cell's four fog
/// half-cells are unseen by a human leader; with it alone the plan ran
/// straight and the re-plan came on 694, a frame late. `find_wpath@
/// 00688fc0`'s human variant then pops the plan until an entry is in
/// the start's region and seen — here the order's own goal — where the
/// AI's pull-back walk had dragged the goal onto the shore and returned
/// unchanged. With both, the chariot re-plans on 693, walks the lake's
/// north end and arrives on 1040, and the squad on 938–941.
///
/// **The block**, from [`WIDENING_CHAPTER_NINE`]'s test on run180 whole
/// (605 to 1100): past the first block's 26 standing rows, the staged
/// units' `form` on their birth blocks; the squad's `order:group.id` on
/// 642, 641000 against 647600, a pushed group's id (this crate's `64 +`
/// index against the original's pool slot); and the scout `1/0`'s
/// non-scoring `order:move.facing` from 847 (parked 275). Item 676 pinned
/// 693 on its first walk, with the plan's rows on 622.
pub(crate) const GOLDEN_WORD_CHAPTER_NINE: i64 = 1100;

/// `chapter_nine_s_word_frame_is_widened_whole`'s window: **run180
/// whole**, its first block, 605, to its last, 1100 (item 676). The first
/// pin was (605, 696) on the open word 693.
pub(crate) const WIDENING_CHAPTER_NINE: (i64, i64) = (605, 1101);

/// **Chapter eight's golden word** — the commanders and a declared war
/// (`docs/GOLDEN.md` §12, run171): **900 of 901, closed**. The walk
/// reaches the trace's last frame: `ally 1` on 900 hands both leaders an
/// allied victory and the game closes after block 901, so 901 is the
/// chapter's length.
///
/// **The word's delta** (item 668): **659 → 900**. On 659 the original
/// spent `Unit::fight+0x9b0`, the one-in-five re-search roll, and this
/// crate did not. The value diff was a frame earlier, in run171's own
/// coordinates: who=1's hoplite `1/6`, chasing the General `0/9`, is
/// blocked by who=0's `0/7` on 658 on both sides (`move_step+0x823`).
/// On block 659 the original's stands at (1780, 7844) with a lone
/// `ATTACK` on `0/7`, `collide_o 7`, `collide 0`. This crate's had taken
/// §6 step 6, `collide 1` and the snap to (1800, 7848), and kept its
/// walk and its attack on `0/9`. `resolve_unit_collision`'s enemy
/// ladder, arm C (`005f9d30:189-252`), was unmodelled: a captain bumped
/// by an enemy that is not its target, with its target out of range,
/// calls `find_new_target(this, NULL, 1)` while its leader's
/// `retargets` is under ten, and takes what it can strike from where it
/// stands (`docs/COLLISION.md` §14). With it, block 659 is run171's
/// field for field, the roll and the first blow land on 659, and every
/// draw agrees to 900.
///
/// Before that, item 664 moved it **617 → 659**: `ObjectData::is(SPY,
/// 0)` had been a seam answering false, so the Spy `1/9`'s birth think
/// spent none of the region scan (`docs/SCOUT.md` §15).
pub(crate) const GOLDEN_WORD_CHAPTER_EIGHT: i64 = 900;

/// `chapter_eight_s_word_frame_is_widened_whole`'s window: **run171
/// whole**, its first block, 605, to its last, 900 (item 668; item 664's
/// was 605 to 660, item 660's 605 to 618). What parts past the standing
/// rows and the birth-block `form`s is the first blow's size on `0/7` and
/// the leaders' `treaties[·]`, both on 660.
pub(crate) const WIDENING_CHAPTER_EIGHT: (i64, i64) = (605, 901);

/// The leader keys chapter eight's dump prints and the leader diff reads:
/// 94 until item 706 read `gov` (95), 96 since item 785 compares
/// `wonder_mark` (`docs/AI.md` §75), and 99 since item 883 reads the tech
/// counters `ages_get()`, `epochs_get()` and `discovered_get()`; 101
/// since item 1209 compares `agendas[·]`, one a player.
pub(crate) const CHAPTER_EIGHT_LEADER_KEYS: usize = 101;
/// The leader keys `LEADERS=2` prints and the leader diff reads, a
/// player: the goods block's 88, `gov` since item 706, and the three tech
/// counters since item 883 (`docs/GOLDEN.md` §35).
pub(crate) const LEADERS_TWO_KEYS: usize = 92;

/// **Chapter four's golden word** — the border and the bleed
/// (`docs/GOLDEN.md` §8, item 552, run132 and run133): **1277 of 1501**,
/// in run133's window. The two captures are one game, draw for draw on
/// all 1501 frames (`chapter_four_s_two_captures_are_one_game`), so the
/// walk reads either; the border window's own first parting is 301, the
/// budgeted sweep (`chapter_four_s_border_is_widened_cell_for_cell`).
///
/// **What the first walk met, and the four fixes before it was pinned.**
/// The interpreter's building arm ordered an unstarted site rather than
/// placing and activating one; the temple border level was a constant 1;
/// `set_leader_epoch` skipped `gain_tech`'s tail, so `civic` never
/// re-read the border; and on the bleed, `Leader::calc_attrition` was
/// never wired to the tech tree (every period 0) and `curr_uber_size` was
/// a stored 1 (16/16 a tick for 6/16). The first three were in before the
/// first walk; the last two were measured and moved no draw — the walk read
/// 1277 with them out and in. They are the widening's rows, and with
/// them the border agrees cell for cell once each lever's sweep settles
/// and the bleed tick for tick to 1337.
///
/// **Item 567 moved it 1277 → 1416: the escort.** On 1277 the original's
/// hoplite squad `1/6..1/8` took a `GUARDORDER` on its Supply Wagon `1/10`
/// and this crate's kept its `AttackTo`. The frame is the army's own tick
/// (765, 1021, 1277, 256 apart), and the call is `Army::process` →
/// `Group::action_siege_attack_to`, whose wagon anchor this crate reached
/// and whose closing `action_guard` it did not have: GUARD was on
/// `docs/ORDERS.md`'s not-implemented list. It is built now —
/// `Group::action_guard`, `add_guard_order`, `do_guard`, and the review's
/// sixty-four-frame arm (`docs/ORDERS.md` §24). The old delta was +1:
/// `Guy::set_anim+0x97a < Guy::move+0x19f`, 5 draws against the farm's 4.
///
/// **The delta at 1416 is +1 again, 32 draws against 31**, and it is the
/// wagon's: the original spends three `Guy::set_anim+0x97a < Unit::
/// set_anim < Unit::do_move+0x11cf` stands on `1/10` (three guys) and one
/// bird coin from `Guy::inc_time`'s wrap, where this crate spends five
/// bird coins (`Guy::set_anim+0x104b`) on the gaia bird `9/6` and no
/// stand. The block, from [`WIDENING_CHAPTER_FOUR`]'s test: on 1416 the
/// original's wagon holds `pause 15` — a collision wait, with the escort
/// now at its heels — and this crate's holds 0. This crate's wagon has
/// stood apart from the original's since 1102 (its birth path, one leg
/// short on 1101: parked since item 552), and on 1277 it is 700 units
/// away, so the escort's posts differ from the first block. No mechanism
/// is named for 1416; the wagon is the frame's first suspect because the
/// widening says so, not because a reading does.
///
/// **Item 569 moved it 1416 → 1500, +84, and 1500 is run133's trace end:
/// chapter four is closed.** No draw, no site and no dumped value parts on
/// any later frame. Two seams in the original's own wagon code, both
/// named in this crate and both reached for the first time by this
/// chapter's Supply Wagon, which is the one unarmed attack-mover in the
/// corpus:
///
/// - **`find_wpath`'s army mode has an `is_supply` arm**
///   (`docs/PATHFINDER.md` §25). An AI unit with type attack 0 still plans
///   as an army when `unit_flags2 & 0x40` is set, and an army pays
///   `base << 5` for every `0x200` cell. The wagon's first route on 1101
///   went (7, 43) north-east through them. The original's went (7, 45)
///   south-east round them, one leg longer. That was the 1101 parting
///   under the old word, and the 700 units on 1277.
/// - **`do_attack_to`'s unarmed arm is `do_attack_to_pause`**
///   (`docs/ORDERS.md` §24.9). Every fifteen frames, a wagon whose group
///   has an armed captain within `0x600` of it, and no more than half of
///   those fighting, sets its `pause` to 15. `do_move` then stands it out
///   with one `set_anim(CHAR_DEFAULT)` a figure per frame.
///
/// **The delta at 1416**: +1, 32 draws against 31. The original spent
/// three `Guy::set_anim+0x97a < Unit::do_move+0x11cf` stands on `1/10`
/// that this crate did not. Now 31 against 31, site for site, once the
/// stand and `do_guard`'s three stands were named on both sides (the trace
/// printed them as a bare `5dac7a`, and this crate as `unit 1/10` and
/// `unit 1/7`). **The value diff on the frame it moved**: the wagon on
/// block 1416 at (9610, 29508) with `pause` 15, and on 1417 at the same
/// point with `pause` 14, on both sides. Before, this crate's wagon held 0
/// and stepped to (9625, 29489). The widening's block for the move is in
/// [`WIDENING_CHAPTER_FOUR`]'s test.
pub(crate) const GOLDEN_WORD_CHAPTER_FOUR: i64 = 1500;

/// **Chapter seven's golden word** — the civilians (`docs/GOLDEN.md` §11,
/// item 578, run141 and its control run142): **1200 of 1201, the trace's
/// end, on both captures. Chapter seven is closed at its first walk.**
///
/// The chapter was designed to show `ai off` silencing a human's
/// civilians, and it does not: the citizen takes the same `GATHERORDER` on
/// block 763 at `idle 12` with the cheat on and off, from `think_peasant`
/// above the cheat's block, and the other four take no order in either run
/// (`docs/INPUT.md` §11.9). What is pinned is the record, not the premise:
/// a human's five idle civilians, a gather issued, walked and delivered,
/// and four that stand, agree with the original draw for draw and value
/// for value to the end of both traces
/// (`chapter_seven_holds_to_the_golden_word`,
/// `chapter_seven_s_control_holds_to_the_golden_word`). There is no move
/// to report a value diff for; the widening's block for the citizen's
/// gather, 763–764, is printed both sides in
/// `chapter_seven_s_word_frame_is_widened_whole`.
pub(crate) const GOLDEN_WORD_CHAPTER_SEVEN: i64 = 1200;

/// `chapter_seven_s_word_frame_is_widened_whole`'s window: **run141 and
/// run142 whole**, 605..1199 and the `!quit` block at 1201. The word is the
/// captures' end, so the floor is their own first block, for the reason
/// [`WIDENING_CHAPTER_TWO`]'s is.
pub(crate) const WIDENING_CHAPTER_SEVEN: (i64, i64) = (605, 1201);

/// **Chapter seven-b's golden word** — the computer's civilians under the
/// cheat (`docs/GOLDEN.md` §11, item 628, run156 and its control run157).
///
/// **1148 → 1200 on item 629, closed** (the word's delta, here; its block
/// is the widening's). The Merchant `1/8`'s unpack ends on 1070, and
/// `SpellType::cast_unpack`'s merchant arm seats it on its tile corner and
/// blocks the square under it (`docs/MERCHANT.md` §3.2); this crate left
/// it 24 units off at the unit-cell centre, and the fur trapper `1/10`
/// was then handed another `dest_y` on 1091 and turned on 1148. With the
/// arm both agree to the end of the trace.
pub(crate) const GOLDEN_WORD_CHAPTER_SEVEN_B: i64 = 1200;

/// `chapter_seven_b_s_word_frame_is_widened_whole`'s window on run156:
/// **the capture whole**, 605..1199 and the `!quit` block at 1201, since
/// the word is the capture's end (item 629) — chapter seven's shape.
pub(crate) const WIDENING_CHAPTER_SEVEN_B: (i64, i64) = (605, 1201);

/// **Chapter seven-b's control's word** — run157, the same five with the
/// Leader AI on (item 628).
///
/// **Item 632 moved it 1036 → 1176**, and this comment carries the delta
/// (the widening's block for it is in
/// `chapter_seven_b_s_word_frame_is_widened_whole`). On 990 the goody
/// look halts who=1's citizen `1/1` on its way to a site. The original's
/// `Group::finish_insert` re-issues the `BUILDORDER` behind the box's walk
/// (case 6, `action_swarm_around(…, QUEUE_LAST, BUILD_AT, 1)`), and this
/// crate dropped it. So on 1036 ours stood orderless and rolled an idle
/// where the original walked on to its site. The value diff on the moved
/// frame, run157's own coordinates: on 1037 `1/1` stands at (36480, 22656)
/// both sides, holding explore-to (37032, 23160) and the `BUILDORDER` on
/// `1/2007`, `idle 0`. On 990 ours held one order, explore-to
/// (36504, 22680), with `orders_x` 36504 against 37080 and `form_mod` −1
/// against 50 (`docs/GROUPS.md` §24). On its own tree the fix stopped at
/// 1148, run156's word and shape. With item 629's merchant arm that
/// frame agrees as well, so the word is **1176**: who=1's
/// `Leader::produce_building`, 69 draws against 249, parting at draw 34.
/// There the original spends a run of `Build::find_gather_tiles+0x10a`
/// and this crate goes on to `Animal::think_bird`.
///
/// **Item 644 moved it 1176 → 1187**, and this comment carries the delta
/// (the widening's block for it is in
/// `chapter_seven_b_s_word_frame_is_widened_whole`). The 180 draws are a
/// Woodcutter's Camp's shuffle: `place_woodcutter` places one at the
/// unfinished Small City `1/2007` on 1176, finds it under five workers and
/// destroys it in the same frame, so no dump block prints its `1/2009`.
/// This crate could not afford it, at 23 food against a price of 70. The
/// original had 98, because `library who=1 2` at 600 pays the Classical
/// age's starting grant (knowledge and metal, 100 each) through
/// `gain_tech`'s tail, and this crate's handler skipped the tail. On 1018
/// a goody pile of 75 then went to metal here, the good at 0, and to food
/// there (`docs/INPUT.md` §11.11). With the tail, the camp is bought on
/// 1176 and draws its 180. With the uid liveness test, `1/7` keeps its
/// walk and `BUILDORDER` on the dead camp (1176 → 1177 was the grant,
/// 1177 → 1187 the liveness). **The value diff on the moved frames, in
/// run157's own coordinates**: on 1177 `1/7` stands at (40271, 18360)
/// on both sides and holds explore-to (40248, 23160) and the
/// `BUILDORDER` on `1/2009`. who=1's food agrees from 1018 (116 on
/// 1018), knowledge and metal from 605 (100 and 100), and the caps from
/// 605 (2992). The word is now **1187**, the scout `1/0`: ours 10 draws
/// against 9, parting at draw 0 on `Unit::do_move+0xe84`. Its explore
/// path has parted since 1077, value only.
///
/// **Item 647 moved it 1187 → 1200, closed**, and this comment carries the
/// delta (the widening's block for it is in
/// `chapter_seven_b_s_word_frame_is_widened_whole`). The scout's path was
/// built on 1076, and run157's proxied `calc_cost` priced nine of its
/// steps apart, every one into cell (48,31) or (47,31): 304/312 there,
/// 1/9 here. Those are the south half of the Small City `1/2007`'s
/// footprint, which the AI placed on 1069 out of its own line of sight.
/// `Wall::start@0063e810` ors the owner's bit into `seen2` over the
/// footprint, and this crate skipped it, so its scout priced the city as
/// unseen ground and cut across it
/// (`docs/SCOUT.md` §14, `docs/VISION.md` §6.1). With the write the
/// search is 945 of 945 steps, each priced as the original's. **The value
/// diff on the moved frames, in run157's own coordinates**: on 1077 `1/0`
/// holds 48 path nodes on both sides, `path[41]` (35832, 24312) to
/// `path[47]` (40440, 25080); on 1116 it stands at (39984, 25092) with
/// `dest_y` 25080 on both, and on 1138 its `dest_x` is 38136 on both.
/// run157 agrees draw for draw and value for value to the end of its
/// trace: **the last open golden chapter is closed.**
pub(crate) const GOLDEN_WORD_CHAPTER_SEVEN_B_CONTROL: i64 = 1200;

/// The control's widening window: **the capture whole**, 605..1199, since
/// the word is the capture's end (item 647), chapter seven-b's shape.
pub(crate) const WIDENING_CHAPTER_SEVEN_B_CONTROL: (i64, i64) = (605, 1201);

/// **Chapter three's golden word** — the mounted and siege lines, run145
/// (`docs/GOLDEN.md` §7): **900, the capture's end, draw for draw and
/// value for value. Chapter three is closed on run145.**
///
/// **The delta: 706 → 900, item 602** — a pivot piece releases through
/// its turret (`docs/COMBAT.md` §55). `get_position`'s pivot branch
/// carries the Chariot's release node on its pivot node, rotated by the
/// facing and `fast_angle_to_degrees(turret_angles[0])`, and the turret
/// is now carried (`set_all_pivots` writes it, `Guy::process` steps it).
/// The value diff on the frame it moved, 706: `0/6`'s round in pool slot 1
/// reads `sx 856, sy 7754, sz 477`, `angle 1267531776` and `v1z`
/// 7.629168 on both sides (ours had `888, 7800, 281`, `1225588736` and
/// 40.295834), and `0/7`'s landed round reads `flags 14` on both, so the
/// original's scatter pair on 706 is this crate's too. The widening's
/// blocks are 651–652, whole and quiet, and 753–754, `0/8`'s turned
/// launch a unit off (§55.5), in `chapter_three_s_word_frame_is_widened_whole`.
///
/// **The delta: 684 → 706, item 603** — the pivot bears from its node
/// (`docs/COMBAT.md` §54). `Guy::set_all_pivots` measures each node's
/// bearing from the unit's point plus the node's truncated vector, the
/// Chariot's `(−102, −59)` at its 120° facing (run147's own `MISC=10`
/// line, and its packet's `Unit::set_attack(0/8, 7, 1)` answering 1). The
/// value diff on the frame it moved, 685: `0/8`'s `angle`, `heading` and
/// both figures' `g.angle` read `1431655765` on both sides (ours had
/// `834011136`), and `1/4`'s move order, which followed 684's draws,
/// agrees until 804. **Each place says which**: this comment the delta,
/// the widening its block (the word's two blocks, whole, since item 603).
///
/// **The delta: 682 → 684, item 601** — two readings of `get_damage`
/// that the listing overturned (`docs/COMBAT.md` §53). **The flank
/// reduction is keyed on the target's mask**: a chariot flanking hoplites
/// deals the whole 50 % per level, so every candidate on 635 gets the
/// height bonus, and `0/6` takes `1/8` on the chain's order, as the dump's
/// does. **The ranking skips the overkill step**: on 685 `0/7` takes the
/// wounded `1/7`, as the dump's does (with the third, it took `1/6`). The
/// value diff on the frame it moved, 635: `0/6`'s `ATTACKORDER` reads `ox
/// 8 whom 1` on both sides (ours had `ox 7`). On 685, `0/7`'s reads `ox 7
/// whom 1` on both. The rows are pinned in
/// `chapter_three_s_word_frame_is_widened_whole`.
///
/// **The delta before it: 633 → 682, item 595.** On 633 the original's chariot
/// `0/8` swung without turning, and this crate turned and deferred the
/// swing. Item 595 took `Unit::set_attack`'s pivot verdict
/// (`docs/COMBAT.md` §52): a type with a `<RESTRICTION>` pivot that bears
/// within ±45° shoots on its own heading. The swing's two rolls are
/// `Guy::set_anim+0xf2f < Unit::set_anim+0x56` and `+0xb6`, named now.
/// The value diff on the frame it moved: `0/8`'s `angle`, `heading` and
/// both figures' `g.angle` on 634 read `1431655765` on both sides (ours
/// had `991232000`), and its crew figure's `g.ox`/`g.whom` read −1 on both
/// (ours had `8`/`1`). The word's blocks, and 635's row, are pinned in
/// `chapter_three_s_word_frame_is_widened_whole`.
///
/// **The delta before it: 621 → 633, item 590.** 587 pinned 621: this crate put the
/// packed catapult `0/9` into `Unit::fight` on its birth block. Item 590
/// took `Unit::think_attack`'s packed-unit arm (`docs/COMBAT.md` §51): a
/// human's packed siege engine returns before the target search and
/// unpacks at `idle 7`. The catapult now agrees with the dump on every
/// row to the capture's end, its cast on 696 and its unpack on 776
/// included. The word's block is pinned in
/// `chapter_three_s_word_frame_is_widened_whole`.
pub(crate) const GOLDEN_WORD_CHAPTER_THREE: i64 = 900;

/// `chapter_three_s_word_frame_is_widened_whole`'s window: **run145
/// whole**, 605..899 and the `!quit` block at 901. The floor is the
/// capture's own first block, for the reason [`WIDENING_CHAPTER_TWO`]'s
/// is; the ceiling is the end, because the chariots' first rounds (651)
/// are after the word and an `AMMO` reader over a window without a round
/// in it would agree by not looking.
pub(crate) const WIDENING_CHAPTER_THREE: (i64, i64) = (605, 901);

/// **Chapter three's restage** — `chapter3b.cmd`, run146, item 587: the
/// same three unit types in two arenas, staged so that §7's minimum-range
/// and speed falsifiers can fire (neither does). **Closed at 1000, the
/// capture's end**: word, sequence and values (item 627).
///
/// **The delta: 865 → 1000, item 627** — `Object::find_nearby_target`'s
/// `local_24` (the listing `64911b`–`64918d`, `6495c2`–`64963b`): a
/// searcher in STAND_GROUND, entrenched, or an **unpacked packer** must
/// reach what it takes, and this crate read the flag the other way round,
/// as "takes anything". And `Unit::fight:1051`: a packer's chase tail
/// starts with `find_new_target`, which kills the attack and runs that
/// search again (`docs/COMBAT.md` §60). On 865 ours had spent a
/// `Unit::fight+0x9b0` on an attack its idle search took on 864, on a
/// hoplite inside the catapult's minimum, and chased away for its range;
/// the dump's holds nothing. The value diff on the frame it moved: on
/// 865–867 `0/6` holds no order on both sides at `idle` 1, 2, 2; on 868,
/// 870 and 871 it holds one `ATTACKORDER` (`in_range 0`, `new_ord 1`) on
/// `1/9`, `1/11` and `1/10`, the block after each hit, and nothing on 869
/// and 872; it stands on (888, 7992) with `orders_x` 888 throughout
/// (`chapter_three_s_catapult_after_its_reload`, 858–880). The word's
/// blocks are in `chapter_three_s_restage_is_widened_whole`.
///
/// **The delta: 792 → 865, item 617** — `Objects::find_free` skips a
/// dead number whose `hold_frames` is not zero, and `DeathObj::inc_time`
/// holds it for as long as the death object lives (`docs/COMBAT.md`
/// §59). This crate handed arena A's three hoplites the dead 6–8 on 771,
/// so their idle (`(frame + o) & 15`) reached 4 three frames late. And
/// the ground shot's scatter is named at its own two sites,
/// `Ammo::init+0xae8`/`+0xb25` (§59.5), which the sequence word met on
/// 797. The value diff on the frame it moved: on 792 the dump's and
/// ours' `1/9`–`1/11` stand on (2424, 7992), (2568, 7992) and (2472,
/// 8136) at `idle 4` with an `ATTACKORDER` (index 10), and on 790 and
/// 791 they read `idle` 3, 3, 4 and 3, 4, 4 on both sides
/// (`chapter_three_s_arena_a_hoplites_take_the_dump_s_numbers`; the
/// numbers are `chapter_three_s_restage_numbers_its_objects`). The
/// word's block is in `chapter_three_s_restage_is_widened_whole`.
///
/// **The delta: 782 → 792, item 625** — an unpacked packer's moving
/// figure asks for no walk (`Guy::move:176–181`, the listing
/// `5d9565`–`5d9581`; `docs/COMBAT.md` §58). The catapult's crew,
/// pulled round by its turn in place on 781, kept their slot, and
/// `Guy::inc_time`'s mirror copied figure 0's `TURN_LEFT` into them,
/// where this crate put them on `CHAR_WALK` and rolled the idle twice on
/// 782. The value diff on the frame it moved: on 781–785 the crew stand
/// on the dump's points (`(938, 7883)`, `(660, 7965)` on 781) and play
/// figure 0's slot on its clock (`chapter_three_s_crew_mirror_the_turn`;
/// `GUYS=2` prints no clock, so the slot is read off run44's `0/15`,
/// `a_turning_catapult_s_crew_mirror_and_never_walk`). And the round's
/// landing on 798, the point plus two scatter draws, now reads the
/// dump's (2413, 8276) (ours had (2655, 8142)). The word's block is in
/// `chapter_three_s_restage_is_widened_whole`.
///
/// **The delta: 782 → 782, item 621** — the siege arm and
/// `Unit::do_attack_ground` (`docs/COMBAT.md` §57), which the dump
/// showed on 781 as an `ATTACKGROUNDORDER` over the attack, a figure at
/// `ox −1` and a reload of 83. Built; the word did not move.
///
/// **The delta: 780 → 782, item 616** — `SpellType::cast_unpack` lights
/// the whole disc at the new line of sight (`update_seen(0)`,
/// `docs/COMBAT.md` §56). Before it, the catapult kept its packed
/// four-tile disc and its 780 search refused all three of arena A's
/// hoplites, seven tiles off, in `valid_target`'s fog test. The value
/// diff on the frame it moved: on 780 `0/6` holds an `ATTACKORDER`
/// (index 10) on the hoplite at (2472, 8136) on both sides (ours had no
/// order), and on 781 its `angle`, `heading`, the three figures'
/// `g.angle` and the crew's positions read the dump's (`1131216896`,
/// `1372003669`, `(938, 7883)`, `(660, 7965)`; ours had `1431655765`
/// and `(948, 7887)`, `(663, 7945)`). The value test is
/// `chapter_three_s_unpacked_catapult_sees_its_hoplites`; the word's
/// block is in `chapter_three_s_restage_is_widened_whole`.
///
/// **The delta: 664 → 780, item 602** — the crew swings with its leader
/// (`Guy::inc_time`'s foot, `docs/ANIM.md` §5.2), with the release
/// through the turret (`docs/COMBAT.md` §55). The value diff on the frame
/// it moved, 664: `0/7`'s and `0/9`'s rounds read `553, 13160, 378` and
/// `606, 13618, 370` on both sides (ours had `600, 13176, 182` and
/// `648, 13608, 162`); the crew's roll at `Unit::set_anim+0xb6` is gone,
/// its swing copied from figure 0. The widening's blocks, 651–652 quiet
/// and 780–781 the catapult's, are in
/// `chapter_three_s_restage_is_widened_whole`.
///
/// **The delta: 633 → 664, item 595** — the pivot (`docs/COMBAT.md`
/// §52), as on run145. The value diff on 634 is run145's shape: `0/8`
/// keeps `1431655765` on both sides (ours had `998768640`), and its crew
/// figure's aim reads −1 on both (ours had `6`/`1`).
///
/// **The chapter's word is [`GOLDEN_WORD_CHAPTER_THREE`]**, 900 on
/// run145, the capture's end. This one is pinned beside it so the
/// restage cannot fall unseen, and it is the lower of the two.
pub(crate) const GOLDEN_WORD_CHAPTER_THREE_RESTAGE: i64 = 1000;

/// `chapter_three_s_restage_is_widened_whole`'s window: run146 whole,
/// 605..999 and the `!quit` block at 1001.
pub(crate) const WIDENING_CHAPTER_THREE_RESTAGE: (i64, i64) = (605, 1001);

/// **How many height reads each replay may make on a corner this crate
/// cannot pin to the original's single**, by test name — `docs/COMBAT.md`
/// §46.3, and the `Drop` of [`crate::diff::Built`] that holds every replay
/// to it. A test not named here is pinned at **zero**, which is every
/// window the harness replays on the day of the pin: the rolled shots and
/// launch heights of every scored run land on corners above 16, whose
/// six-decimal print names one single, and on no corner a terraform has
/// rewritten. The residue the zero stands over is 2,058 of Great Lakes'
/// 58,081 corners, every one under 16 in magnitude, where two to four
/// singles print alike and [`sim::single::Single::from_millionths`] takes
/// the nearest — at most a few ulp, under 2e-6 of a world unit.
///
/// **Chapter seventeen's bombs** (item 770, `docs/ORDERS.md` §35.3). A
/// Bomber's round lands where `find_data_z` answers under open ground at
/// 0 to 10, on the corners the zero stands over, and its height feeds
/// one thing: the fall time, `(int)sqrtf(2(ez − sz) / GRAV_Z)`. Every one
/// of run223's 49 bombs falls from 1601–1632 and takes 17 frames with
/// `ez` one either side as well (measured, item 770's journal); 17 holds
/// for any drop in 1516–1698. So the reads are counted, and pinned.
pub(crate) const GROUND_INEXACT: &[(&str, u32)] = &[
    // Chapter fifty (item 1404): the Stockades' arrows over run577's ground.
    ("chapter_fifty_holds_to_the_golden_word", 7),
    ("chapter_fifty_s_word_frame_is_widened_whole", 7),
    // East Indies' long walk past the word 16179 (item 1401): 25 reads of
    // a non-exact corner in the diverged tail to the endpoint, which the
    // decoy's refused cast had kept out of the walk until 16160.
    ("run346_is_east_indies_at_toughest_and_its_word_holds", 25),
    ("run413_s_and_run415_s_fog_grids_are_ours", 25),
    // Chapter forty-three (item 1223): the Chariot's three shots at the
    // who=1 Citizen from (15384, 31224), 1419, 1445 and 1469.
    ("chapter_forty_three_holds_to_the_golden_word", 3),
    ("chapter_forty_three_s_word_frame_is_widened_whole", 3),
    // Chapter forty-six (item 1278) is chapter forty-three's cast: the same
    // three shots at the who=1 Citizen X.
    ("chapter_forty_six_holds_to_the_golden_word", 3),
    ("chapter_forty_six_s_word_frame_is_widened_whole", 3),
    // Chapter forty-one (item 1182): the Biplane's two EXITs from its
    // Airbase on the computer's sorties, 808 and 1470, each a read of the
    // ground under the base's point.
    ("chapter_forty_one_holds_to_the_golden_word", 2),
    ("chapter_forty_one_s_word_frame_is_widened_whole", 2),
    ("chapter_seventeen_holds_to_the_golden_word", 34),
    ("chapter_seventeen_s_word_frame_is_widened_whole", 34),
    ("run235_s_bombs_are_the_original_s_record_for_record", 34),
    // Chapter twenty-two is chapter seventeen's game with the Fighter
    // launched: the pair's same 49 bombs, and the launch's one read of
    // the ground under the Airbase's point, (11616, 13920), the EXIT's
    // `find_data_z` (item 836).
    ("chapter_twenty_two_holds_to_the_golden_word", 35),
    ("chapter_twenty_two_s_word_frame_is_widened_whole", 35),
    ("run265_s_launch_is_the_original_s_field_for_field", 35),
    ("run265_s_rounds_are_the_original_s_record_for_record", 35),
    // Chapter twenty-three is chapter twenty-two's game to 1840 (item
    // 867): the same 35, and nothing past 1500 flies.
    ("chapter_twenty_three_holds_to_the_golden_word", 35),
    ("chapter_twenty_three_s_word_frame_is_widened_whole", 35),
    // Chapter twenty-nine is chapter twenty-two's game to 2070 with no
    // toggle (item 915): the same 35, one EXIT read for each of the three
    // relaunches (1585, 1789, 1813). The first walk read 39: the fourth
    // was the Biplane's EXIT on 1746, which the built arm keeps inside.
    ("chapter_twenty_nine_holds_to_the_golden_word", 38),
    ("chapter_twenty_nine_s_word_frame_is_widened_whole", 38),
    // Chapter thirty-two is chapter twenty-nine's game with three gather
    // points on the Airbase (item 947): the first walk read its 38; the
    // built arm launches the Biplane on 1747, one EXIT read more.
    ("chapter_thirty_two_holds_to_the_golden_word", 39),
    ("chapter_thirty_two_s_word_frame_is_widened_whole", 39),
    // Chapter thirty-three is chapter thirty-two's game to 2360 and five
    // presses on the Airbase after it (item 976): the first walk reads 43,
    // and the built arms 43 (pinned at 42, it fails).
    ("chapter_thirty_three_holds_to_the_golden_word", 43),
    ("chapter_thirty_three_s_word_frame_is_widened_whole", 43),
    // Chapter thirty-four is chapter thirty-three's game to its Barracks
    // and nine presses after it (item 1009): the first walk reads 43, and
    // the built arms 43 (pinned at 42, it fails).
    ("chapter_thirty_four_holds_to_the_golden_word", 43),
    ("chapter_thirty_four_s_word_frame_is_widened_whole", 43),
    // Chapter thirty-five is chapter thirty-four's game to its second
    // Airbase and a Missile Silo and twelve lines after it (item 1019):
    // the first walk reads 39, and the built arms 39 (pinned at 38, it
    // fails). **40 with the V2's round** (item 1050): `Ammo::init`'s
    // `find_data_z` at its point, one read more (pinned at 39, it fails).
    ("chapter_thirty_five_holds_to_the_golden_word", 40),
    ("chapter_thirty_five_s_word_frame_is_widened_whole", 40),
    // The V2's launch, field for field (item 1050): run371 to 2703, the
    // round's read included.
    (
        "chapter_thirty_five_s_v2_is_counted_out_and_fired_field_for_field",
        40,
    ),
    // The V2's blast, field for field (item 1077): run371 to 2822, the
    // same reads as the walk to there.
    (
        "chapter_thirty_five_s_v2_blast_is_compared_field_for_field",
        40,
    ),
    // Chapter thirty-six is chapter thirty-five's game whole and twelve
    // lines after it (item 1078): the first walk reads 40 (pinned at 39,
    // it fails).
    ("chapter_thirty_six_holds_to_the_golden_word", 40),
    ("chapter_thirty_six_s_word_frame_is_widened_whole", 40),
    // Chapter thirty-seven (item 1091): one read, the nuke's round's `ez`
    // at ground zero (pinned at 0, it fails).
    ("chapter_thirty_seven_holds_to_the_golden_word", 1),
    ("chapter_thirty_seven_s_word_frame_is_widened_whole", 1),
    (
        "chapter_thirty_seven_s_nuke_is_launched_and_fired_field_for_field",
        1,
    ),
    // Chapter thirty-eight (item 1102): the two Bombers' bombs on T and R,
    // chapter seventeen's reads on other ground (pinned at 40, it fails).
    // Item 1131: 44, the Bombers' strikes held on T to their deaths (pinned
    // at 43, it fails).
    ("chapter_thirty_eight_holds_to_the_golden_word", 44),
    ("chapter_thirty_eight_s_word_frame_is_widened_whole", 44),
    // Item 1117's word block stops at 780, before the bombs: one read, by
    // 780 (pinned at 0, it fails).
    (
        "chapter_thirty_eight_s_battery_is_short_of_its_aim_on_the_word_s_block",
        1,
    ),
    // Item 1113's squad test walks run404 whole to its last block but
    // one, the Bombers' bombs with it: 31 reads (pinned at 0, it fails);
    // 44 with item 1131's strikes held on T (pinned at 43, it fails).
    (
        "chapter_thirty_eight_s_squad_stands_on_its_points_and_packs_on_its_phase",
        44,
    ),
    // Chapter forty-eight (item 1358): the Bomber's four passes on T, on
    // run551's ground: 32 reads (pinned at 31, it fails).
    ("chapter_forty_eight_holds_to_the_golden_word", 32),
    ("chapter_forty_eight_s_word_frame_is_widened_whole", 32),
];

/// **Every pinned word names the test that widened its frame whole, or
/// the open item that will** (`docs/DECISIONS.md` 43). A word is the frame
/// the draw stream parts; its widening is `compare` over every dumped
/// record on that frame and its neighbours, and it is the **first** item
/// on a new word. Item 415 pinned 616 and read a value diff at 622
/// instead; five items then chased a mechanism that the whole-cast
/// widening at 616 (item 441) retired in one probe — while 9510's
/// widening ran first (item 408) and that chain converged on a measured
/// defect. `floors::the_widening_behind_each_pinned_word_exists` reads
/// this table: a named test must be a `fn` under `crates/rondata/src/
/// diff/`, and a `None` must name an item that is open in `docs/QUEUE.md`
/// or parked in `docs/PARKED.md`. The last column is the item — the one
/// that landed the widening, or the one that owes it.
/// The block window `run100_s_word_block_is_every_record_the_dump_carries`
/// walks — run100's first complete block to thirteen past the word it
/// last widened, 10834. The test reads its bounds from here. **Item 518
/// moved the word to 11185, past run100's last block (10899)**, so the
/// Great Lakes row of [`WIDENINGS`] no longer names this test: a window
/// the word has walked out of would pass by saying nothing (parked 449).
/// The test stays, as the widening of the blocks it does reach, and the
/// row names the item that owes the new word's.
pub(crate) const WIDENING_GREAT_LAKES: (i64, i64) = (9_340, 10_847);
/// `chapter_two_s_word_frame_is_widened_whole`'s window, on the same
/// terms: run112's window opens at 605 and one past the last frame
/// compared. It straddled the word at 624 until item 462 moved the word
/// to 637 and moved the window with it, which is parked 449's lesson
/// applied at the move rather than after it.
///
/// **The floor is run112's own first block since item 470**, and not a
/// few frames under the word. At 633 it was three frames above a live
/// divergence — `order 0/10` and `pos 0/10` part at 630 and had never
/// been reported — so a window sized to the word was reporting agreement
/// it had not measured (`docs/COMBAT.md` §35.2). That row is closed and
/// the ceiling followed the word to 645 on item 472; the floor has not
/// moved and will not, because it is the only one that cannot hide a row.
/// The ceiling followed the word to 680 on item 479, and to **683** on
/// item 481 — whose three surviving rows all sit *above* the old ceiling
/// at 684 and stood with the fix reverted, so the ceiling was hiding them
/// exactly the way the floor once hid `0/10`'s. `extra 1/8` is the first,
/// and it is a **death**: the original's hoplite `1/8` dies on 683 and
/// this crate's does not. Item 484 gave [`crate::diff::compare`] the
/// hit-point row that walk was missing, and the cause is now on the map:
/// `1/8` runs one hit behind the dump from **656**, twenty-seven blocks
/// under the word and well inside this window, which nothing had ever
/// compared (`docs/COMBAT.md` §40).
///
/// **The ceiling followed the word to 729 on item 502** — the word moved
/// 695 → 725 and the window came with it on the same frame, which is the
/// rule rather than a courtesy. Four frames past the word, as every
/// ceiling here has been.
///
/// **And to 766 on item 495**, with the word 725 → 762 — the widening's
/// block for the move. Its map fell from thirty-nine rows to twenty; the
/// first past the word are `0/10`'s clock on 764 and `1/7`'s walk on 765,
/// and run118's clock still covers the whole window.
///
/// **And to 901 on item 523, the end of run112**, with the word 762 → 900.
/// A word at the capture's end has nothing above it to straddle, so the
/// window is the whole capture from its first block: run112 carries
/// 605..899 and its `!quit` block at 901, and the test knows block 900 is
/// the one it lacks. run118's clocks stop at 846 and cover the window up
/// to there. The map fell from twenty rows to eleven, the two residues
/// standing since 606 and 680.
pub(crate) const WIDENING_CHAPTER_TWO: (i64, i64) = (606, 901);

/// `chapter_five_s_word_frame_is_widened_whole`'s window: **run127 whole**,
/// its first block to one past its last. A word this close to the
/// capture's first block leaves nothing worth trimming, and the floor is
/// the capture's own first block for the reason [`WIDENING_CHAPTER_TWO`]'s
/// is: it is the only floor that cannot hide a row. run127 carries
/// 605..899 and its `!quit` block at 901.
pub(crate) const WIDENING_CHAPTER_FIVE: (i64, i64) = (605, 901);

/// `chapter_four_s_border_is_widened_cell_for_cell`'s window: **run132
/// whole**, the border capture of chapter four (item 552). It spans the
/// three border levers — the Temple at 300, Religion at 400 and Civic 3 at
/// 500 — and the budgeted recompute after each, which the original spreads
/// over about five blocks. `docs/GOLDEN.md` §8 designed it as `[295, 345)`,
/// which sees the Temple alone.
pub(crate) const WIDENING_CHAPTER_FOUR_BORDER: (i64, i64) = (295, 545);

/// `chapter_four_s_word_frame_is_widened_whole`'s window: **run133
/// whole**, the bleed capture of chapter four (item 552): 595..1499 and its
/// `!quit` block at 1501. The floor is the capture's own first block for
/// the reason [`WIDENING_CHAPTER_TWO`]'s is.
pub(crate) const WIDENING_CHAPTER_FOUR: (i64, i64) = (595, 1501);

/// `run123_s_word_frame_is_widened_whole`'s window: run123's own first
/// and last blocks (item 520). The capture was sized to the word, 11185,
/// with 425 blocks under it — back past the 10782 and 10982 rotations —
/// and 274 over it, and the floor is the capture's first block for the
/// reason [`WIDENING_CHAPTER_TWO`]'s is: it is the only floor that cannot
/// hide a row.
pub(crate) const WIDENING_GREAT_LAKES_MARKET: (i64, i64) = (10_760, 11_459);
/// The block [`WIDENING_GREAT_LAKES_MARKET`] was taken to widen: the word
/// 11185, where it stood until item 327 moved it past run123's last block.
/// `run123_s_word_frame_is_widened_whole` holds the move's value diff on it
/// and the coverage driver reads run123 around it, so the two stay keyed
/// on a block the capture carries whatever the headline does next.
pub(crate) const GREAT_LAKES_MARKET_BLOCK: i64 = 11_185;
/// `run125_s_word_frame_is_widened_whole`'s window (item 533): run123 from
/// **11250**, the blocks before army 1's squad `1/31..1/39` takes the group
/// order it marches to the word under, and run125 from its own first block
/// 11440 to its last, 11599. The floor is under the march rather than at
/// run125's first block because the squad's partings are positions and
/// soft-collision flags that spend no draw, and a widening that opened on
/// the march would print them as standing residue with no first block.
pub(crate) const WIDENING_GREAT_LAKES_ARMY: (i64, i64) = (11_250, 11_599);
/// The block [`WIDENING_GREAT_LAKES_ARMY`] was taken to widen: the word
/// 11531, where it stood until item 539 moved it to 11582. The coverage
/// driver read run125 around it until item 554 moved the driver to
/// run130's word block ([`GREAT_LAKES_ARMY_TWO_BLOCK`]); the widening pins
/// it empty as the move's value diff.
pub(crate) const GREAT_LAKES_ARMY_BLOCK: i64 = 11_531;
/// `run130_s_word_frame_is_widened_whole`'s window (item 554): run123 from
/// **11400**, under army 2's first parting on 11424, then run125 from its
/// own first block 11440, then run130 from 11560 to its last block, 11799.
/// The floor is under the army because its partings (`group`, `form`, the
/// slotted orders point) spend no draw, and a walk that opened above them
/// would print them as standing residue with no first block.
pub(crate) const WIDENING_GREAT_LAKES_ARMY_TWO: (i64, i64) = (11_400, 11_799);
/// The block [`WIDENING_GREAT_LAKES_ARMY_TWO`] was taken to widen: the word
/// 11757's frame writes block 11758. The coverage driver reads run130
/// around it, keyed here rather than on the headline so the pin stays on a
/// block the capture carries whatever the headline does next.
pub(crate) const GREAT_LAKES_ARMY_TWO_BLOCK: i64 = 11_758;
/// `run135_s_word_frame_is_widened_whole`'s window (item 560): run123
/// from **11400**, under army 2's first pool parting, then run125, run130
/// and run135 from 11800 to its last block, 11859. The same floor as
/// [`WIDENING_GREAT_LAKES_ARMY_TWO`] and for the same reason: the partings
/// under the word's crossing spend no draw, and a walk that opened above
/// them would print them as standing residue with no first block.
pub(crate) const WIDENING_GREAT_LAKES_CROSSING: (i64, i64) = (11_400, 11_859);
/// The block [`WIDENING_GREAT_LAKES_CROSSING`] was taken to widen: the
/// word 11806's frame writes block 11807. The coverage driver reads run135
/// around it, keyed here rather than on the headline.
pub(crate) const GREAT_LAKES_CROSSING_BLOCK: i64 = 11_807;
/// `run136_s_word_frame_is_widened_whole`'s window (item 563): run123
/// from **11400**, then run125, run130, run135 and run136 to its last
/// block, 11959. The floor is [`WIDENING_GREAT_LAKES_CROSSING`]'s, for the
/// same reason: the partings under the word spend no draw, and a walk
/// that opened above them would print them as standing residue with no
/// first block.
pub(crate) const WIDENING_GREAT_LAKES_DETOUR: (i64, i64) = (11_400, 11_959);
/// The block [`WIDENING_GREAT_LAKES_DETOUR`] was taken to widen: the word
/// 11903's frame writes block 11904. The coverage driver reads run136
/// around it, keyed here rather than on the headline.
pub(crate) const GREAT_LAKES_DETOUR_BLOCK: i64 = 11_904;
/// `run163_s_word_frame_is_widened_whole`'s window (item 571): run123
/// from **11400**, then run125, run130, run135, run136 and run163 to its
/// last block, 12399. The floor is [`WIDENING_GREAT_LAKES_DETOUR`]'s, for
/// the same reason: the partings under the word spend no draw, and a walk
/// that opened above them would print them as standing residue with no
/// first block.
pub(crate) const WIDENING_GREAT_LAKES_UPGRADE: (i64, i64) = (11_400, 12_399);
/// The block [`WIDENING_GREAT_LAKES_UPGRADE`] was taken to widen: the word
/// 12038's frame writes block 12039. The coverage driver reads run163
/// around it, keyed here rather than on the headline.
pub(crate) const GREAT_LAKES_UPGRADE_BLOCK: i64 = 12_039;
/// The block of the word item 657 moved to: 12184's frame writes block
/// 12185, inside [`WIDENING_GREAT_LAKES_UPGRADE`]. The coverage driver
/// reads run163 around it too.
pub(crate) const GREAT_LAKES_MAKE_BLOCK: i64 = 12_185;
/// `run174_s_word_frame_is_widened_whole`'s window (item 669): run123
/// from **11400**, then run125, run130, run135, run136, run163 and run174
/// to its last block, 12899. The floor is [`WIDENING_GREAT_LAKES_UPGRADE`]'s,
/// for the same reason.
pub(crate) const WIDENING_GREAT_LAKES_CIVIC: (i64, i64) = (11_400, 12_899);
/// The block [`WIDENING_GREAT_LAKES_CIVIC`] was taken to widen: the word
/// 12429's frame writes block 12430. The coverage driver reads run174
/// around it.
pub(crate) const GREAT_LAKES_CIVIC_BLOCK: i64 = 12_430;
/// The block of the word item 669 moved to: 12536's frame writes block
/// 12537, inside [`WIDENING_GREAT_LAKES_CIVIC`], where the squad
/// `1/27`–`1/29` takes its orders. Item 673 put it in the coverage driver,
/// which 669's landing had left on [`GREAT_LAKES_CIVIC_BLOCK`] alone.
pub(crate) const GREAT_LAKES_SQUAD_BLOCK: i64 = 12_537;
/// The block of the word item 673 moved to: 12897's frame writes block
/// 12898, run174's last but one. The coverage driver reads run174 from
/// two blocks under it to the capture's last, four blocks.
pub(crate) const GREAT_LAKES_RETRY_BLOCK: i64 = 12_898;
/// `run178_s_word_frame_is_widened_whole`'s window (item 678): run123
/// from **11400**, then run125, run130, run135, run136, run163, run174 and
/// run178 to its last block, 14899. The floor is
/// [`WIDENING_GREAT_LAKES_CIVIC`]'s, for the same reason.
pub(crate) const WIDENING_GREAT_LAKES_COPY: (i64, i64) = (11_400, 14_899);
/// The block [`WIDENING_GREAT_LAKES_COPY`] was taken to widen: the word
/// 14382's frame writes block 14383. The coverage driver reads run178
/// around it.
pub(crate) const GREAT_LAKES_COPY_BLOCK: i64 = 14_383;
/// The block of the word item 688 moved to: 14529's frame writes block
/// 14530, inside run178 and so inside [`WIDENING_GREAT_LAKES_COPY`]. The
/// coverage driver reads run178 around it too.
pub(crate) const GREAT_LAKES_VALS_BLOCK: i64 = 14_530;
/// The block of the word item 695 moved to: 14650's frame writes block
/// 14651, inside run178 and so inside [`WIDENING_GREAT_LAKES_COPY`]. The
/// coverage driver reads run178 around it too.
pub(crate) const GREAT_LAKES_ROAD_BLOCK: i64 = 14_651;
/// `run192_s_word_frame_is_widened_whole`'s window (item 698):
/// [`WIDENING_GREAT_LAKES_COPY`]'s chain, then run192 from run178's last
/// block to its own, 15039. The floor is [`WIDENING_GREAT_LAKES_CIVIC`]'s,
/// for the same reason.
pub(crate) const WIDENING_GREAT_LAKES_BIRTH: (i64, i64) = (11_400, 15_039);
/// The block [`WIDENING_GREAT_LAKES_BIRTH`] was taken to widen: the word
/// 14982's frame writes block 14983, past run178. The coverage driver
/// reads run192 around it.
pub(crate) const GREAT_LAKES_BIRTH_BLOCK: i64 = 14_983;
/// `run196_s_word_frame_is_widened_whole`'s window (item 706):
/// [`WIDENING_GREAT_LAKES_BIRTH`]'s chain, then run196 from run192's last
/// block to its own, 15232.
pub(crate) const WIDENING_GREAT_LAKES_PATRIOT: (i64, i64) = (11_400, 15_232);
/// The block [`WIDENING_GREAT_LAKES_PATRIOT`] was taken to widen: the word
/// 15175's frame writes block 15176, past run192. The coverage driver
/// reads run196 around it.
pub(crate) const GREAT_LAKES_PATRIOT_BLOCK: i64 = 15_176;
/// `run202_s_word_frame_is_widened_whole`'s window (item 711):
/// [`WIDENING_GREAT_LAKES_PATRIOT`]'s chain, then run202 from run196's last
/// block to its own, 15440.
pub(crate) const WIDENING_GREAT_LAKES_MIRROR: (i64, i64) = (11_400, 15_440);
/// The block [`WIDENING_GREAT_LAKES_MIRROR`] was taken to widen: the word
/// 15383's frame writes block 15384, past run196. The coverage driver
/// reads run202 around it.
pub(crate) const GREAT_LAKES_MIRROR_BLOCK: i64 = 15_384;
/// The word 15384's frame writes block 15385 (item 715), inside
/// [`WIDENING_GREAT_LAKES_MIRROR`]; the coverage driver reads run202 from
/// two blocks under [`GREAT_LAKES_MIRROR_BLOCK`] to two over this one.
pub(crate) const GREAT_LAKES_RECRUIT_BLOCK: i64 = 15_385;
/// `run211_s_word_frame_is_widened_whole`'s window (item 722):
/// [`WIDENING_GREAT_LAKES_MIRROR`]'s chain, then run211 from run202's last
/// block to its own, 15859 — 250 blocks of runway past the word's block.
pub(crate) const WIDENING_GREAT_LAKES_WONDER: (i64, i64) = (11_400, 15_859);
/// The block [`WIDENING_GREAT_LAKES_WONDER`] was taken to widen: the word
/// 15608's frame writes block 15609, past run202. The coverage driver
/// reads run211 around it.
pub(crate) const GREAT_LAKES_WONDER_BLOCK: i64 = 15_609;
/// The word 15619's frame writes block 15620 (item 729), inside
/// [`WIDENING_GREAT_LAKES_WONDER`]; the coverage driver reads run211 from
/// two blocks under [`GREAT_LAKES_WONDER_BLOCK`] to two over this one.
pub(crate) const GREAT_LAKES_ATTACKED_BLOCK: i64 = 15_620;
/// `run218_s_word_frame_is_widened_whole`'s window (item 736):
/// [`WIDENING_GREAT_LAKES_WONDER`]'s chain, then run218 from run211's last
/// block to its own, 16711 — 250 blocks of runway past the word's block.
pub(crate) const WIDENING_GREAT_LAKES_ESCORT: (i64, i64) = (11_400, 16_711);
/// The block [`WIDENING_GREAT_LAKES_ESCORT`] was taken to widen: the word
/// 16460's frame writes block 16461, past run211. The coverage driver
/// reads run218 around it.
pub(crate) const GREAT_LAKES_ESCORT_BLOCK: i64 = 16_461;
/// `run226_s_word_frame_is_widened_whole`'s window (item 742):
/// [`WIDENING_GREAT_LAKES_ESCORT`]'s chain, then run226 from run218's last
/// block to its own, 17350 — 250 blocks of runway past the word's block.
pub(crate) const WIDENING_GREAT_LAKES_GIVEUP: (i64, i64) = (11_400, 17_350);
/// The block [`WIDENING_GREAT_LAKES_GIVEUP`] was taken to widen: the word
/// 17099's frame writes block 17100, past run218. The coverage driver
/// reads run226 around it.
pub(crate) const GREAT_LAKES_GIVEUP_BLOCK: i64 = 17_100;
/// The block [`WIDENING_GREAT_LAKES_GIVEUP`] widens since item 757: the
/// word 17128's frame writes block 17129, inside run226. The coverage
/// driver reads run226 around it too.
pub(crate) const GREAT_LAKES_PYRAMIDS_BLOCK: i64 = 17_129;
/// The block [`WIDENING_GREAT_LAKES_GIVEUP`] widens since item 776: the
/// word 17181's frame writes block 17182, inside run226. The coverage
/// driver reads run226 around it too.
pub(crate) const GREAT_LAKES_FOREST_CELL_BLOCK: i64 = 17_182;
/// `run243_s_word_frame_is_widened_whole`'s window (item 785): run226's
/// last six blocks (17345..17350), then run243 whole, 20500..20818 — 68
/// blocks into the word 20568, its block, and 250 of runway past it.
/// **Sized to the word, not to the gap**: no dump holds 17351..20499, so
/// the walk reads nothing there, and its first run243 block carries
/// whatever the gap left.
pub(crate) const WIDENING_GREAT_LAKES_STAND: (i64, i64) = (17_345, 20_818);
/// run243's first block.
pub(crate) const GREAT_LAKES_STAND_FIRST: i64 = 20_500;
/// `run294_s_departure_is_widened_whole`'s window (item 795): run226's
/// last six blocks (17345..17350), then run294 whole, 19840..19999 — the
/// four walkers' return leg leaving the far point. **A bisection of the
/// gap, not a word's window**: 17351..19839 is compared by no dump.
pub(crate) const WIDENING_GREAT_LAKES_DEPART: (i64, i64) = (17_345, 19_999);
/// run294's first block.
pub(crate) const GREAT_LAKES_DEPART_FIRST: i64 = 19_840;
/// The word 20568's block on run243: its frame, the original's blocked
/// step (`Unit::move_step+0x823`) where ours spends an idle roll, writes
/// block 20569. The coverage driver reads run243 around it.
pub(crate) const GREAT_LAKES_STAND_BLOCK: i64 = 20_569;
/// The word 20800's block on run243 (item 795): ours' `1/60` stands blocked
/// by `1/64` (`Unit::move_step+0x823`) where the original's walks, and
/// its frame writes block 20801. The coverage driver reads run243 around
/// it, and [`crate::diff::harness`]'s word window walks it.
pub(crate) const GREAT_LAKES_RETURN_BLOCK: i64 = 20_801;
/// `run80_s_word_frame_is_widened_whole`'s window (item 899): run80 whole,
/// 23960..23999 and the end block 24001. Great Lakes' draw stream agrees on
/// every frame of run53's trace since item 899, so the word is the
/// capture's own end, 24000, and the widening is the last blocks any dump
/// holds. No `GROUPDATA` is on run80, so the pool walk reads nothing here.
pub(crate) const WIDENING_GREAT_LAKES_END: (i64, i64) = (23_960, 24_001);
/// `run96_s_word_frame_is_widened_whole`'s window (item 919): run96 whole,
/// 23960..23999 and the end block 24001. East Indies' draw stream agrees on
/// every frame of run54's trace since item 919, so the word is the
/// capture's own end, 24000, and the widening is the last blocks any dump
/// holds. No `GROUPDATA` is on run96.
pub(crate) const WIDENING_EAST_INDIES_END: (i64, i64) = (23_960, 24_001);
/// `run99_s_word_frame_is_widened_whole`'s window (item 573): run98 from
/// its own first block, then run99 from 8789 to its last block, 10399.
/// The floor is the first capture's first block for the reason
/// [`WIDENING_CHAPTER_TWO`]'s is: it is the only floor that cannot hide a
/// row.
pub(crate) const WIDENING_EAST_INDIES: (i64, i64) = (7_880, 10_399);
/// The block [`WIDENING_EAST_INDIES`] was taken to widen: the word 9711's
/// frame writes block 9712, the sixth seated Scholar's birth. Item 573
/// moved the word past it; `run99_s_word_frame_is_widened_whole` holds the
/// move's value diff on it.
pub(crate) const EAST_INDIES_SCHOLAR_BLOCK: i64 = 9_712;
/// The block run139 was taken to widen: the word 9983's frame writes block
/// 9984, the `make_stuff` that bought a Mine here and ships there. Item
/// 576 moved the word past it; `run139_s_word_frame_is_widened_whole`
/// holds the make lists on it and the coverage driver reads run139 around
/// it.
pub(crate) const EAST_INDIES_MAKE_BLOCK: i64 = 9_984;
/// The word 10398's block on run99: its frame writes block 10399, run99's
/// last. Item 588 moved the word past run99; its widening keeps the move's
/// value diff on this block.
pub(crate) const EAST_INDIES_WORD_BLOCK: i64 = 10_399;
/// `run139_s_word_frame_is_widened_whole`'s window (item 576): run139
/// whole, 9960..9999, the one East Indies capture that prints the leader's
/// make list over the word. The floor is the capture's first block for the
/// reason [`WIDENING_CHAPTER_TWO`]'s is, and it is under the production
/// cycle that fills the list the word's `make_stuff` spends (9975..9983).
pub(crate) const WIDENING_EAST_INDIES_MAKE: (i64, i64) = (9_960, 9_999);
/// `run143_s_word_frame_is_widened_whole`'s window (item 588): run143
/// whole, 10380..10739 — run99's line at `LEADERS=9`, overlapping run99's
/// last twenty blocks, then the word and 340 blocks past it. The floor is
/// the capture's first block for the reason [`WIDENING_CHAPTER_TWO`]'s is.
pub(crate) const WIDENING_EAST_INDIES_BARK: (i64, i64) = (10_380, 10_739);
/// The block run143 was taken to widen: the word 10398's frame writes
/// block 10399, where the Bark `1/34` idles in the original and was still
/// moving here. Item 588 moved the word past it; the coverage driver reads
/// run143 around it.
pub(crate) const EAST_INDIES_BARK_BLOCK: i64 = 10_399;
/// The word 10582's block on run143: its frame, the Mine `1/2018`'s
/// placement, writes block 10583. Item 604 moved the word past run143; its
/// widening keeps the move's value diff on this block.
pub(crate) const EAST_INDIES_MINE_BLOCK: i64 = 10_583;
/// `run149_s_word_frame_is_widened_whole`'s window (item 608): run149
/// whole, 10730..10879 — run143's line past its last block, overlapping it
/// on 10730..10739, then the word 10782 and 96 blocks past it. The floor is
/// the capture's first block for the reason [`WIDENING_CHAPTER_TWO`]'s is.
pub(crate) const WIDENING_EAST_INDIES_MERCS: (i64, i64) = (10_730, 10_879);
/// The word 10782's block on run149: its frame, player 1's `make_stuff`,
/// writes block 10783. Item 608 moved the word past run149; its widening
/// keeps the move's value diff on this block, and the coverage driver reads
/// run149 around it.
pub(crate) const EAST_INDIES_MERCS_BLOCK: i64 = 10_783;
/// `run152_s_word_frame_is_widened_whole`'s window (item 613): run152
/// whole, 10870..11039 — run149's line past its last block, overlapping it
/// on 10870..10879, then the word 10982 and 56 blocks past it. The floor is
/// the capture's first block for the reason [`WIDENING_CHAPTER_TWO`]'s is.
pub(crate) const WIDENING_EAST_INDIES_GATHER: (i64, i64) = (10_870, 11_039);
/// The word 10982's block on run152: its frame, player 1's `make_stuff`
/// placing a building, writes block 10983. The coverage driver reads
/// run152 around it.
pub(crate) const EAST_INDIES_GATHER_BLOCK: i64 = 10_983;
/// `run155_s_word_frame_is_widened_whole`'s window (item 620): run155
/// whole, 11030..11279 — run152's line past its last block, overlapping it
/// on 11030..11039, then the word 11069 and 209 blocks past it. The floor
/// is the capture's first block for the reason [`WIDENING_CHAPTER_TWO`]'s
/// is.
pub(crate) const WIDENING_EAST_INDIES_EXPLORE: (i64, i64) = (11_030, 11_279);
/// The word 11069's block on run155: its frame, the explore's grid roll,
/// writes block 11070. The coverage driver reads run155 around it.
pub(crate) const EAST_INDIES_EXPLORE_BLOCK: i64 = 11_070;
/// `run159_s_word_frame_is_widened_whole`'s window (item 629): run159
/// whole, 11270..11899 — run155's line past its last block, overlapping it
/// on 11270..11279, then the word 11590 and 308 blocks past it. The floor
/// is the capture's first block for the reason [`WIDENING_CHAPTER_TWO`]'s
/// is.
pub(crate) const WIDENING_EAST_INDIES_IDLE: (i64, i64) = (11_270, 11_899);
/// The word 11590's block on run159: its frame, the sheep `8/1`'s
/// arrival idle, writes block 11591. The coverage driver reads run159
/// around it.
pub(crate) const EAST_INDIES_IDLE_BLOCK: i64 = 11_591;
/// The word 11747's block on run159 (item 642): its frame, the AI scout
/// `1/0`'s boarding cast, writes block 11748. The coverage driver reads
/// run159 around it.
pub(crate) const EAST_INDIES_CAST_BLOCK: i64 = 11_748;
/// `run166_s_word_frame_is_widened_whole`'s window (item 643): run166
/// whole, 13580..13699 — a narrow window over the word 13640, sized to the
/// word and not to the 1,741-block gap from run159's last block, so it
/// shares no block with any other capture. The floor is the capture's
/// first block, which carries every key that parted anywhere in the gap.
pub(crate) const WIDENING_EAST_INDIES_WRAP: (i64, i64) = (13_580, 13_699);
/// The word 13640's block on run166: its frame, three `Guy::inc_time`
/// wraps in ours against two, writes block 13641. The coverage driver
/// reads run166 around it.
pub(crate) const EAST_INDIES_WRAP_BLOCK: i64 = 13_641;
/// `run221_s_word_frame_is_widened_whole`'s window (item 708): run221
/// whole, 15894..16236 — six blocks shared with run78, the 86 up to the
/// word 15985, its block, and 250 of runway past it. The floor is the
/// capture's first block, which carries every key that parted since
/// run166's last block that run78's `LEADERS=1` does not print.
pub(crate) const WIDENING_EAST_INDIES_SLOT: (i64, i64) = (15_894, 16_236);
/// The word 15985's block on run221: its frame, the original's slot-loop
/// buy and its `make_stuff+0x63d` expiry roll, writes block 15986. The
/// coverage driver reads run221 around it.
pub(crate) const EAST_INDIES_SLOT_BLOCK: i64 = 15_986;
/// `run227_s_word_frame_is_widened_whole`'s window (item 752): run227
/// whole, 16230..16934 — seven blocks shared with run221, the 447 up to
/// the word 16683, its block, and 250 of runway past it. The floor is the
/// capture's first block, which carries every key standing on run221's
/// last.
pub(crate) const WIDENING_EAST_INDIES_WRAPWORD: (i64, i64) = (16_230, 16_934);
/// The word 16683's block on run227: its frame, six `Guy::inc_time`
/// idle wraps there against five here, writes block 16684. The coverage
/// driver reads run227 around it.
pub(crate) const EAST_INDIES_WRAPWORD_BLOCK: i64 = 16_684;
/// `run233_s_word_frame_is_widened_whole`'s window (item 767): run233
/// whole, 16929..17440 — six blocks shared with run227, the 255 up to the
/// word 17189, its block, and 250 of runway past it. The floor is the
/// capture's first block, which carries every key standing on run227's
/// last.
pub(crate) const WIDENING_EAST_INDIES_BLOCKWORD: (i64, i64) = (16_929, 17_440);
/// The word 17189's block on run233: its frame, ours' blocked stand
/// (`Unit::move_step+0x823`) a frame before the original's, writes block
/// 17190. The coverage driver reads run233 around it.
pub(crate) const EAST_INDIES_BLOCKWORD_BLOCK: i64 = 17_190;
/// The word 17403's block on run233 (item 773): its frame, ours'
/// `Unit::do_move+0xe84` where the original spends a `Guy::inc_time`
/// wrap, writes block 17404. Under it, on 17403, the original gives `1/60`
/// an `ATTACK_TO` where ours gives a `GROUP_ATTACK_TO`. The coverage
/// driver reads run233 around it.
pub(crate) const EAST_INDIES_GROUPWORD_BLOCK: i64 = 17_404;
/// `run251_s_word_frame_is_widened_whole`'s window (item 800): run251
/// whole, 17496..17752 — the six blocks before the word 17501's block, its
/// block, and 250 of runway past it. No dump shares a block with it:
/// 17441..17495 is compared by none. The floor is the capture's first
/// block, which carries every key standing when the walk arrives.
pub(crate) const WIDENING_EAST_INDIES_COLUMNWORD: (i64, i64) = (17_496, 17_752);
/// The word 17501's block on run251 (item 800): its frame, where ours
/// spends `1/57`'s walk step (`Guy::move+0x19f`) and the original an idle
/// roll, writes block 17502. The coverage driver reads run251 around it.
pub(crate) const EAST_INDIES_COLUMNWORD_BLOCK: i64 = 17_502;
/// `run253_s_word_frame_is_widened_whole`'s window (item 811): run253
/// whole, 18177..18433 — the six blocks before the word 18182's block, its
/// block, and 250 of runway past it. No dump shares a block with it:
/// 17753..18176 is compared by none.
pub(crate) const WIDENING_EAST_INDIES_MARKETWORD: (i64, i64) = (18_177, 18_433);
/// The word 18182's block on run253 (item 811): its frame, where ours
/// spends `Leader::make_stuff+0x221` and the original
/// `Leader::use_market+0x1ed`, writes block 18183. The coverage driver
/// reads run253 around it.
pub(crate) const EAST_INDIES_MARKETWORD_BLOCK: i64 = 18_183;
/// `run257_s_word_frame_is_widened_whole`'s window (item 822): run257
/// whole, 18933..19189 — the six blocks before the word 18938's block, its
/// block, and 250 of runway past it. No dump shares a block with it:
/// 18434..18932 is compared by none.
pub(crate) const WIDENING_EAST_INDIES_GUARDWORD: (i64, i64) = (18_933, 19_189);
/// The word 18938's block on run257 (item 822): its frame, where ours
/// spends `Unit::do_guard+0x8fb` and the original `Guy::set_anim+0x104b`,
/// writes block 18939. The coverage driver reads run257 around it.
pub(crate) const EAST_INDIES_GUARDWORD_BLOCK: i64 = 18_939;
/// `run261_s_gap_is_widened_whole`'s window (item 829): run261 whole,
/// 18428..18684 — six blocks before the gap 18434..18932 that no dump had
/// compared, and 250 into it, over army 1's close on tick 18682.
pub(crate) const WIDENING_EAST_INDIES_CLOSE: (i64, i64) = (18_428, 18_684);
/// The block after army 1's close on run261 (item 829): tick 18682 writes
/// block 18683, where the original's group 71 first reads `army −1`.
pub(crate) const EAST_INDIES_CLOSE_BLOCK: i64 = 18_683;
/// The word 18999's block on run257 (item 829): its frame, where ours
/// spends a fifth `Guy::inc_time` wrap and the original
/// `Guy::set_anim+0x104b`, writes block 19000. The coverage driver reads
/// run257 around it.
pub(crate) const EAST_INDIES_FIGUREWORD_BLOCK: i64 = 19_000;
/// The word 19182's block on run257 (item 837): its frame, where ours
/// spends `Leader::use_market+0x1ed` and the original a first
/// `Guy::inc_time` wrap, writes block 19183. The coverage driver reads
/// run257 around it.
pub(crate) const EAST_INDIES_LEADERWORD_BLOCK: i64 = 19_183;
/// `run269_s_word_frame_is_widened_whole`'s window (item 839): run269
/// whole, 19408..19664 — six blocks before the word 19413's block and 250
/// past it. run257 ends on 19189, so 19190..19407 is compared by no dump.
pub(crate) const WIDENING_EAST_INDIES_TURNWORD: (i64, i64) = (19_408, 19_664);
/// The word 19413's block on run269 (item 839): its frame, where the
/// original spends a turn's `Guy::set_anim` and ours a wrap, writes block
/// 19414. The coverage driver reads run269 around it.
pub(crate) const EAST_INDIES_TURNWORD_BLOCK: i64 = 19_414;
/// The word 19509's block on run269 (item 850): its frame, where ours
/// spends `1/71`'s walk start (`Unit::move_step+0x823`) and the original a
/// wrap, writes block 19510. The coverage driver reads run269 around it.
pub(crate) const EAST_INDIES_WALKWORD_BLOCK: i64 = 19_510;
/// `run277_s_word_frame_is_widened_whole`'s window (item 857): run277
/// whole, 20002..20258 — six blocks before the word 20007's block and 250
/// past it. run269 ends on 19664, so 19665..20001 is compared by no dump.
pub(crate) const WIDENING_EAST_INDIES_BLOCKEDWALK: (i64, i64) = (20_002, 20_258);
/// The word 20007's block on run277 (item 857): its frame, where ours
/// spends `1/67`'s blocked step (`Unit::move_step+0x823`) and the original
/// a first wrap, writes block 20008. The coverage driver reads run277
/// around it.
pub(crate) const EAST_INDIES_BLOCKEDWALK_BLOCK: i64 = 20_008;
/// `run289_s_word_frame_is_widened_whole`'s window (item 880): run289
/// whole, 20777..21033 — six blocks before the word 20782's block and 250
/// past it. run277 ends on 20258, so 20259..20776 is compared by no dump.
pub(crate) const WIDENING_EAST_INDIES_USEMARKET: (i64, i64) = (20_777, 21_033);
/// The word 20782's block on run289 (item 880): its frame, where ours
/// spends `Leader::use_market+0x1ed` and the original a farm's
/// `Farms::inc_time+0x1ae`, writes block 20783. The coverage driver reads
/// run289 around it.
pub(crate) const EAST_INDIES_USEMARKET_BLOCK: i64 = 20_783;
/// `run299_s_word_frame_is_widened_whole`'s window (item 890): run299
/// whole, 23177..23433 — six blocks before the word 23182's block and 250
/// past it. run289 ends on 21045, so 21046..23176 is compared by no dump.
pub(crate) const WIDENING_EAST_INDIES_WONDERPRICE: (i64, i64) = (23_177, 23_433);
/// The word 23182's block on run299 (item 890): its frame, where ours
/// spends `Leader::produce_building+0x1805` and the original
/// `Leader::make_stuff+0x63d`, writes block 23183. The coverage driver
/// reads run299 around it.
pub(crate) const EAST_INDIES_WONDERPRICE_BLOCK: i64 = 23_183;
/// The word 23420's block on run299 (item 904): its frame, where ours
/// spends `Unit::do_guard+0x8fb` and the original
/// `Farms::add_animals+0x92`, writes block 23421. The coverage driver
/// reads run299 around it.
pub(crate) const EAST_INDIES_ADDANIMALS_BLOCK: i64 = 23_421;
/// One pinned word's row: `(constant, word, widening test, item, window)`.
pub(crate) type Widening = (
    &'static str,
    i64,
    Option<&'static str>,
    u32,
    Option<(i64, i64)>,
);

/// **A named test carries
/// the block window it walks**, and the guard requires the word to sit
/// strictly inside it — `the_widening_behind_each_pinned_word_exists`
/// checked only that the test *existed* until the eighth pass, so a row
/// could go stale by success: run100's word-frame test widened 9382, item
/// 442 moved the word 651 frames past it, and the row kept reading as
/// pinned (parked 449). The window is a shared constant the test itself
/// reads, which is what keeps the declaration from going stale the same way.
pub(crate) const WIDENINGS: &[Widening] = &[
    // Parked 444 since the seventh pass; booked as item 573 by the eleventh,
    // the day the lower-map rule became a guard — East Indies had been the
    // lower word since 2026-09-21 with its widening still owed. Item 573
    // paid it on run99, which had carried the word since 2026-09-18 and
    // which no item had walked: the queue's booking asked for a capture
    // the disk already held.
    //
    // Item 576 widened 9983 on **run139**, the one East Indies capture that
    // prints the leader's record over the word, with a sibling test rather
    // than an extension: `run139_s_word_frame_is_widened_whole` adds the
    // leader half — the stockpile, the step, the muster and the make list,
    // slot for slot. The lists parted on block 9982, `create_units`' ship
    // offers (`docs/AI.md` §57). The fix moved the word to **10232**, past
    // run139's last block and inside run99's, so the row names run99's
    // test again: it now pins the move's value diff on 9984 (empty) and
    // the new word's blocks, 10231..10233.
    //
    // Item 579 moved it to **10398** within run99's test: the birth of
    // Trireme `1/32` on 10187 and the navy's first order on 10232. The
    // test pins the move's value diff on the old word's block, 10233
    // (empty), and the new word's on 10399 — run99's last block, so the
    // window still holds the word, and the next move past it owes a
    // capture.
    //
    // Item 588 took it: **run143** is run99's line at `LEADERS=9` over
    // [10380, 10739], and `run143_s_word_frame_is_widened_whole` is a
    // sibling that straddles the word's block — every record, every unit,
    // both leaders, and the one-sided animation changes, on 360 blocks
    // twenty of which run99 shares. run99's test keeps the moves' value
    // diffs under it. The same item moved the word to **10582**, inside
    // run143's window: the test pins the move's value diff on 10399
    // (empty) and the new word's blocks, 10581..10583.
    //
    // Item 604 moved it to **10782**, past run143's last block (10739):
    // run143's test keeps the move's value diff on 10583 (80 rows → 0).
    //
    // Item 608 paid it: **run149** is run143's line over [10730, 10879],
    // and `run149_s_word_frame_is_widened_whole` is a sibling through the
    // same walk (`widen_east_indies`), with gaia's animals added — every
    // record, every unit, both leaders, and the one-sided animation
    // changes, on 150 blocks ten of which run143 shares. It pins the word's
    // blocks, 10781..10783 (the Citizen's `num`), and the census under
    // them on 10776. The same item moved the word to **10982**, past
    // run149's last block (10879). run149's test keeps the move's value
    // diff on 10776 and 10781..10783.
    //
    // Item 613 paid it: **run152** is run149's line over [10870, 11039],
    // and `run152_s_word_frame_is_widened_whole` is a sibling through the
    // same walk, gaia included, on 170 blocks ten of which run149 shares.
    // The same item moved the word to **11069**, past run152's last block
    // (11039). run152's test keeps the move's value diff on 10981..10983.
    //
    // Item 620 paid it: **run155** is run152's line over [11030, 11279],
    // and `run155_s_word_frame_is_widened_whole` is a sibling through the
    // same walk, gaia included, on 250 blocks ten of which run152 shares,
    // with `compare`'s container row in it (parked 598). It pinned the
    // word's blocks, 11069..11070: the citizen `1/11`'s waypoint popped a
    // frame late, its walk (15, −29) off since 10959. The same item moved
    // the word to **11590**, past run155's last block (11279); run155's
    // and run152's tests keep the move's value diff.
    //
    // Item 629 paid it: **run159** is run155's line over [11270, 11899],
    // and `run159_s_word_frame_is_widened_whole` is a sibling through the
    // same walk, gaia included, on 630 blocks ten of which run155 shares.
    // It pins the word's blocks, 11578..11592: gaia's sheep `8/1` walks a
    // wander straight here and round a waypoint there, eight frames
    // longer, past the Merchant `1/20` standing (24, 24) off its tile
    // corner since 7663.
    //
    // Item 642 widened 11747's blocks on run159 (the scout's boat), and
    // moved the word to **13640**, past run159's last block (11899) and
    // past every East Indies dump on disk; run159's test keeps the move's
    // value diff.
    //
    // Item 643 paid it: **run166** is run159's line with `GROUPS=1` over
    // [13580, 13699], sized to the word rather than to the gap, and
    // `run166_s_word_frame_is_widened_whole` is a sibling through the same
    // walk, gaia and the pool included, on 120 blocks it shares with no
    // other capture. The same item moved the word to **15782**, past
    // run166's last block (13699) and inside run78's [15700, 15900],
    // whose records no test has widened: item 694 owes it, and captures
    // only what run78's `LEADERS=1` cannot answer. run166's test keeps
    // the move's value diff.
    //
    // **Item 706 moved it 15782 → 15985**, past run78's last block
    // (15900): `1/60` was The Senator, a Senate's government patriot
    // (`docs/TECH.md` §"The government patriot"), built for Great Lakes'
    // Despot. `run78_s_old_word_keeps_its_value_diff` keeps the move's
    // value diff on 15783. Item 708 owes the capture over the new word.
    //
    // **Item 708 paid it**: run221 is run166's line without `DEATHS` over
    // [15894, 16236], six blocks shared with run78 and 250 past the word,
    // and `run221_s_word_frame_is_widened_whole` walks it through the same
    // walk, gaia and the pool included, and pins the word's block, 15986.
    // **The same item moved it 15985 → 16683**, past run221's last block
    // (16236): the commerce cap's republic term (`docs/AI.md` §72).
    // run221's test keeps the move's value diff. Item 752 owes the capture
    // over the new word.
    //
    // **Item 752 paid it**: run227 is run221's line over [16230, 16934],
    // seven blocks shared with run221 and 250 past the word, and
    // `run227_s_word_frame_is_widened_whole` walks it through the same
    // walk, gaia and the pool included, and pins the word's block, 16684:
    // the sixth wrap is the idle citizen `1/46`, which ours had sent to a
    // woodcutter on 16594. **The same item moved it 16683 → 16982**, past
    // run227's last block (16934): the census counts a gatherer in its
    // building's city (`docs/AI.md` §73). run227's test keeps the move's
    // value diff. Item 767 owes the capture over the new word.
    //
    // **Item 767 moved it and paid it**: the British price of Taxation
    // (`docs/AI.md` §74) moved the word 16982 → 17189, past run227's last
    // block, and run233 is run227's line over [16929, 17440], six blocks
    // shared with run227 and 250 past the word.
    // `run233_s_word_frame_is_widened_whole` walks it through the same
    // walk, gaia and the pool included, and pins the word's block, 17190:
    // who=1's `1/55` takes its blocked stand by `1/60` a frame early here,
    // a half step ahead from 17182. run227's test keeps the move's value
    // diff.
    //
    // **Item 773 moved it inside the window**: `no_danger`'s order arm
    // (`docs/PATHFINDER.md` §28) moved the word 17189 → 17403, and run233
    // still carries 36 blocks past it. The same test pins the new word's
    // block, 17404, beside the old one's value diff.
    //
    // **Item 800 moved it past run233**: the anchor's sub-group sorts its
    // own list (`docs/GROUPS.md` §27), 17403 → 17501, 61 blocks past
    // run233's last. run233's test keeps the move's value diff; item 800
    // owes run251 over the new word's block, 17502 — and paid it: run251
    // is run233's line over [17496, 17752], six blocks before the word and
    // 250 past it, and `run251_s_word_frame_is_widened_whole` walks it
    // through the same walk, gaia and the pool included, and pins the
    // word's block.
    //
    // **Item 811 moved it past run251**: a retarget forms the army twice
    // (`docs/ARMY.md` §21), 17501 → 18182, 430 blocks past run251's last.
    // run233's test keeps the move's value diff (block 17405) and run251's
    // its rows under the old word; run253 is run251's line over [18177,
    // 18433], six blocks before the word and 250 past it, and
    // `run253_s_word_frame_is_widened_whole` pins the word's block.
    //
    // **Item 822 moved it past run253**: a border fix zeroes the rares
    // until the next census (`docs/AI.md` §76), 18182 → 18938, 505 blocks
    // past run253's last. run253's test keeps the move's value diff (the
    // make list on 18181); run257 is run253's line over [18933, 19189],
    // six blocks before the word and 250 past it, and
    // `run257_s_word_frame_is_widened_whole` pins the word's block.
    //
    // **Item 829 moved it inside run257**: a closed army's group stays in
    // the pool (`docs/ARMY.md` §22), 18938 → 18999. run257's test keeps
    // the move's value diff (18933 and 18939) and pins the new word's
    // block, 19000, in the same walk.
    //
    // **Item 837 moved it inside run257 again**: a unit holding a
    // suspended search idles without a draw (`docs/ANIM.md` §14), 18999
    // → 19182. run257's test keeps the move's value diff (19000, `1/67`)
    // and pins the new word's block, 19183, six blocks from the
    // capture's end.
    //
    // **Item 839 moved it past run257's end**: the British take the
    // territory tax twice (`docs/ECONOMY.md` §16), 19182 → 19413. run257's
    // test keeps the move's value diff (19177 and 19183), and run269 was
    // taken to widen the new word's block, 19414.
    //
    // **Item 850 moved it inside run269**: a unit that goes inside in its
    // own work takes its figures' frame (`docs/ANIM.md` §15), 19413 →
    // 19509. run269's test keeps the move's value diff (19414, `1/77`) and
    // pins the new word's block, 19510, in the same walk.
    //
    // **Item 857 moved it past run269's end**: `do_move`'s blocker probe
    // is the quick form (`docs/COLLISION.md` §18), and the trace names
    // `Unit::do_guard+0x8fb`, 19509 → 20007. run269's test keeps the
    // move's value diff (19499, `1/71`), and run277 was taken to widen the
    // new word's block, 20008.
    //
    // **Item 880 moved it past run277's end**: `Group::kill` clears an
    // emptied pool record and an army's group is its slot's record
    // (`docs/GROUPS.md` §30), 20007 → 20782. run277's test keeps the
    // move's value diff (20002, slot 69 and `1/64`..`1/66`), and run289 was
    // taken to widen the new word's block, 20783.
    //
    // **Item 890 moved it past run289's end**: `get_cost`'s wonder arm
    // counts every wonder held or sited (`docs/COSTS.md`, "A wonder is
    // ramped by every wonder"), 20782 → 23182. run289's test keeps the
    // move's value diff (20782, `MAKE[0]`, `[1]` and `[8]` `val`), and
    // run299 was taken to widen the new word's block, 23183.
    //
    // **Item 904 moved it inside run299's window**: `blocked_tcoord`'s rock
    // arm (`docs/AI.md` §78), 23182 → 23420. The same test keeps the move's
    // value diff (23183, `MAKE[4].t` and `1/79`) and widens the new word's
    // block, 23421, and the runway to it.
    //
    // **Item 919 moved it to 24000, the capture's own end**:
    // `get_nearest_farm_type` does not see an unstarted farm site
    // (`docs/AI.md` §79), 23420 → 24000. run299's test keeps the move's
    // value diff (23421, `1/78` and `1/80`), and run96, the last blocks
    // any dump holds, is widened whole.
    (
        "LONG_WORD_EAST_INDIES",
        LONG_WORD_EAST_INDIES,
        Some("run96_s_word_frame_is_widened_whole"),
        919,
        Some(WIDENING_EAST_INDIES_END),
    ),
    // Item 448 paid the widening 442 owed: `run100_s_word_block_is_every_
    // record_the_dump_carries` compares every field of every record run100
    // carries on the word's own block, both directions and ungated by the
    // position — on **10162** it was fifteen rows, all of them `1/38`
    // (`docs/AI.md` §54). Item 456 closed those fifteen and moved the word
    // to **10232**; the same test's window was widened to straddle the new
    // word rather than left naming a block this word has left, which is
    // parked 449's lesson applied at the move. The row it replaced named
    // `run100_s_word_frame_is_the_original_s`, which widens **9382** and
    // went stale by succeeding when the word moved 651 frames past it;
    // the guard checks that a named test exists, not that it widens the
    // current word, so that stale name passed and read as pinned
    // (parked 449).
    //
    // Item 463 moved it to **10233** and widened the record itself: the
    // `UNITDATA` line the whole item turned on — `recharging`, the reload
    // clock — was not in the comparison at all, so a squad that kept a
    // dead target's order for the length of its reload read as a pathing
    // divergence for two items. It agrees on every AI unit of all 908
    // blocks now, which is what says the kill's *timing* is the
    // original's and not a coincidence.
    //
    // Item 464 held the word and widened it again: `myhits` is the
    // record's **maximum** and `damage` the accumulator beside it, and
    // this comparison read the first as "hits left" — so it agreed only
    // for an untouched unit and printed every wound as a divergence.
    // `hits_left` and `myhits` are both rows now, 54,000 more
    // comparisons, and both agree on all 909 blocks.
    //
    // Item 487 moved the word 10244 → **10277** and moved the window's
    // ceiling with it, 10247 → 10290, in the same landing — parked 449's
    // lesson applied at the move. The widening is what killed item 483's
    // reading before a line of it was implemented: 483 wrote "`1/29`
    // drops its move order on 10241" and the record says the opposite —
    // `order:kind ours 10 theirs 1` is **this crate** still holding an
    // `ATTACK` where the original is already walking, and the dump's own
    // `1/29` never moves in this crate at all. The frame was right and
    // the mechanism was not, which is `docs/DECISIONS.md` 42 for the
    // fourth time in this chain.
    //
    // Item 489 moved it 10277 → **10294** and the ceiling 10290 →
    // 10307. Here the frame was right and the row 487 declined to call a
    // mechanism *was* the mechanism: the soft one-shot on both raiders
    // and `1/40`'s hard `collide_o 41` are the same arm, seen from the
    // two sides. What the widening added this time was the **position
    // gate** — `1/41 pos` parts on 10279, so its `half_step` row on
    // 10278 is read on a unit still standing where the original's does,
    // which is the only reason it could be trusted at all
    // (`docs/COLLISION.md` §8.4, §11.1).
    //
    // Item 494 moved it 10294 → **10303** and the ceiling 10307 →
    // 10316, and this time the widening's value was that it ran
    // **wider than the booking**: 489 booked block 10295's six rows of
    // `0/5`, and the re-run over the same window printed four more on
    // **10294** — the arrival turn's `heading`, `dest_angle` and
    // `g.des_angle` — and, sixty blocks under them, item 464's three on
    // **10234**. Those three are the cause and the other ten the
    // consequence: the human's idle wait is `peasants_wait`'s *switch*
    // and not its value, so a citizen re-tasked at `idle == 2` carried a
    // `GATHERORDER` under its flight and could never arrive holding the
    // single order [`sim::Sim::arrive`] faces the order's angle for
    // (`docs/ORDERS.md` §21). One number closed all thirteen.
    //
    // **Item 518 moved it 10834 → 11185, past every dump on disk**:
    // run100 ends on block 10899, so no test can widen 11185 whole, and
    // the row names the item that owes it — 520, booked for the capture.
    // `run100_s_word_block_is_every_record_the_dump_carries` still walks
    // [`WIDENING_GREAT_LAKES`], and 10242, 10243 and 10835 are pinned
    // empty there — the value diff beside this move.
    //
    // **Item 520 paid it**: run123 is the same game at `LEADERS=9` over
    // [`WIDENING_GREAT_LAKES_MARKET`], and the test compares every unit
    // record and the whole leader record on every block of it. The
    // readings the stanza named died three of four on the word's own
    // block — the purse, the commerce level, the stock — and the one left
    // is slot 1 of the make list: an emptied Merchant slot in the
    // original, a Cataphract here (`docs/ECONOMY.md` §14).
    //
    // **Item 327 moved it 11185 → 11531, past every dump on disk again**:
    // run123 ends on block 11459. The Merchant offer closed 11185's slot 1
    // (`docs/AI.md` §55), and run123's test keeps that block as the value
    // diff of the move, keyed on its own `WORD_BLOCK` rather than on the
    // headline.
    //
    // **Item 533 paid it**: run125 is the same game at run123's detail over
    // [11440, 11599], and `run125_s_word_frame_is_widened_whole` walks it
    // from run123's 11250 — under army 1's squad march — so the window is
    // [`WIDENING_GREAT_LAKES_ARMY`]. The word's two blocks part on one unit,
    // `1/34`, which reaches its `ATTACK_TO` point on block 11531 here and
    // 11533 in the original (`docs/GROUPS.md` §20).
    //
    // **Item 539 moved the word to 11582, still inside run125**, and the
    // same test holds the new word's block: every key first parting on
    // 11580..=11583, both directions, with 11531's pinned empty as the
    // move's value diff (`docs/GROUPS.md` §21).
    //
    // **Item 545 moved it 11582 → 11757, past run125's last block
    // (11599)**, so the widening is owed a capture and this row names the
    // item that owes it and no test. The same test keeps 11583 as the
    // move's value diff (`docs/AI.md` §56), on its own literal rather than
    // on the headline.
    //
    // **Item 554 paid it**: run130 is the same game at run125's detail
    // over [11560, 11799], and `run130_s_word_frame_is_widened_whole`
    // walks it from run123's 11400, under army 2's first pool parting, so
    // the window is [`WIDENING_GREAT_LAKES_ARMY_TWO`]. The word's block
    // parts on `1/62`, which stops against `1/23` a frame late because it
    // walks a detour planned around `1/64`, and `1/64` stands 21/20 behind
    // the original's from army 2's group order of frame 11512
    // (`docs/GROUPS.md` §22).
    //
    // **Item 557 moved it 11757 → 11806, past run130's last block
    // (11799)**, so the widening is owed a capture again and this row names
    // the item that owes it and no test. The same test keeps 11758 pinned
    // empty as the move's value diff (`docs/GROUPS.md` §23), on its own
    // `GREAT_LAKES_ARMY_TWO_BLOCK` rather than on the headline.
    //
    // **Item 560 paid it**: run135 is run134's line over [11760, 11859],
    // and `run135_s_word_frame_is_widened_whole` walks it from run123's
    // 11400 across four captures, with every player-1 pool list on
    // run135's blocks, so the window is [`WIDENING_GREAT_LAKES_CROSSING`].
    // The word's block parts on `1/37` alone: it carries the soft
    // one-shot out of a sweep that also found `1/64` hard, and steps
    // where the original's waits (`docs/COLLISION.md` §12).
    //
    // **And item 560 moved it 11806 → 11903, past run135's last block
    // (11859)**, so the widening is owed a capture again and this row
    // names the item that owes it and no test. The same test keeps 11807
    // pinned empty, with every block after it, as the move's value diff,
    // on its own `GREAT_LAKES_CROSSING_BLOCK`.
    //
    // **Item 563 paid it**: run136 is run135's line over [11840, 11959],
    // and `run136_s_word_frame_is_widened_whole` walks it from run123's
    // 11400 across five captures, so the window is
    // [`WIDENING_GREAT_LAKES_DETOUR`]. The word's blocks part first on
    // `1/62`'s path on 11902: its flag-2 detour round `1/27` goes north in
    // the original and south here, and `1/64` waits on it there and steps
    // and stops here (`docs/PATHFINDER.md` §23).
    //
    // **Item 566 moved it 11903 → 12038, past run136's last block
    // (11959)**, so the widening is owed a capture again and this row
    // names the item that owes it (571) and no test.
    // `run136_s_word_frame_is_widened_whole` keeps 11902..11921 pinned
    // empty as the move's value diff, and pins block 11922's 29 rows —
    // army 1's squad stopping in the original and walking here, which
    // spends no draw — as the nearest parting (`docs/PATHFINDER.md` §24.6).
    //
    // **Item 571 paid it**: run163 is run136's line over [11950, 12399],
    // and `run163_s_word_frame_is_widened_whole` walks it from run123's
    // 11400 across six captures, so the window is
    // [`WIDENING_GREAT_LAKES_UPGRADE`]. The word's block parts on `1/62`,
    // which stands against `1/64` in the original and walks here; the
    // chain runs back to 11922, where who=1's barracks research converts
    // the nine and the original's `Guy::init_real` stands them
    // (`docs/ANIM.md` §11). With the stand, the word moved **12038 →
    // 12135**, inside the same window (block 12136), and the test pins
    // the move's value diff and the new word's block both.
    //
    // **Item 657 moved it 12135 → 12184**, inside the same window again
    // (block 12185): `release_mustering` reads the Military library level,
    // not the age (`docs/ARMY.md` §20). The test pins 12136 empty for
    // `1/68` as the move's value diff, and the new word's chain from 12059
    // to its block.
    //
    // Item 661 moved it **12184 → 12429**, past run163's last block (12399)
    // and past every Great Lakes dump on disk: a Civic level seats who=1's
    // Barracks and Stable in Norwich (`docs/AI.md` §63). run163's test
    // keeps the move's value diff, 12181–12185 empty. Item 669 owes the
    // capture and its widening.
    //
    // **Item 669 paid it**: run174 is run163's line over [12390, 12899],
    // and `run174_s_word_frame_is_widened_whole` walks it from run123's
    // 11400 across seven captures, so the window is
    // [`WIDENING_GREAT_LAKES_CIVIC`]. The word's block parts on `1/67`,
    // which hard-collides with `1/68` here and walks on in the original;
    // `1/68` walks a leg the original does not, because its tile plan on
    // 12322 dropped the group's exact formation point. With the unwind
    // read as the listing has it, the word moved **12429 → 12536**, inside
    // the same window (block 12537), and the test pins the move's value
    // diff and the new word's block both.
    //
    // **Item 673 moved it 12536 → 12897**, inside the same window again
    // (block 12898, run174's last but one): a failed unit-grid search
    // spares an action-bit move too (`docs/PATHFINDER.md` §21.6). The
    // test pins 12537's value diff and `1/41`'s every row on 12898. The
    // next move past 12898 owes a capture.
    //
    // **Item 678 moved it 12897 → 14382**, past run174's last block and
    // past every Great Lakes dump on disk below run80: the pathfinder's
    // block copies (`docs/PATHFINDER.md` §26). run174's test keeps the
    // move's value diff. **Item 678 paid it**: run178 is run174's line
    // over [12894, 14899], and `run178_s_word_frame_is_widened_whole` walks
    // it from run123's 11400 across eight captures, so the window is
    // [`WIDENING_GREAT_LAKES_COPY`] and the test pins the word's block.
    //
    // **Item 688 moved it 14382 → 14529**, inside the same window again
    // (block 14530): a founded city wears the site values down
    // (`docs/AI.md` §67). The test pins the new word's block and keeps the
    // move's value diff on 14383.
    //
    // **Item 695 moved it 14529 → 14650**, inside the same window again
    // (block 14651): the caravan verifies a road the stray-road sweep took
    // (`docs/ROADS.md` §10). The test pins the new word's block and keeps
    // the move's value diff on 14530.
    //
    // **Item 698 moved it 14650 → 14982**, past run178's last block
    // (14899): a gatherer on a suspended search gives its walk up
    // (`docs/COLLISION.md` §15). run178's test keeps the move's value diff
    // on 14651. **Item 698 paid it**: run192 is run178's line over
    // [14894, 15039], and `run192_s_word_frame_is_widened_whole` walks it
    // from run123's 11400 across nine captures, so the window is
    // [`WIDENING_GREAT_LAKES_BIRTH`] and the test pins the word's block.
    //
    // **Item 706 moved it 14982 → 15175**, past run192's last block
    // (15039): the nation graft table and the Senate's government patriot
    // (`docs/TECH.md` §"The graft table", §"The government patriot").
    // run192's test keeps the move's value diff on 14946 and 14983. **Item
    // 706 paid it**: run196 is run192's line over [15034, 15232], and
    // `run196_s_word_frame_is_widened_whole` walks it from run123's 11400
    // across ten captures, so the window is [`WIDENING_GREAT_LAKES_PATRIOT`]
    // and the test pins the word's block.
    //
    // **Item 711 moved it 15175 → 15383**, past run196's last block
    // (15232): a unit's own mirror, `unit_masks & 2`, which `do_guard`
    // reads off its target (`docs/GROUPS.md` §25). run196's test keeps
    // the move's value diff on 15095. **Item 711 paid it**: run202 is
    // run196's line over [15227, 15440], and
    // `run202_s_word_frame_is_widened_whole` walks it from run123's 11400
    // across eleven captures, so the window is
    // [`WIDENING_GREAT_LAKES_MIRROR`] and the test pins the word's block.
    //
    // **Item 715 moved it 15383 → 15384**, inside run202: `Wall::process`'s
    // site recruiter (`docs/AI.md` §69). The same test pins the new word's
    // block, 15385, and keeps the move's value diff on 15383 and 15384.
    //
    // **Item 722 moved it 15384 → 15608**, past run202's last block
    // (15440): `CityData::num_wonders` counts a wonder site (`docs/AI.md`
    // §70). run202's test keeps the move's value diff on 15385..15440.
    // **Item 722 paid it**: run211 is run202's line over [15435, 15859],
    // and `run211_s_word_frame_is_widened_whole` walks it from run123's
    // 11400 across twelve captures, so the window is
    // [`WIDENING_GREAT_LAKES_WONDER`] and the test pins the word's block.
    //
    // **Item 729 moved it 15608 → 15619**, inside run211:
    // `Object::take_damage` stamps the struck owner's `frame_attacked`
    // (`docs/AI.md` §71). The same test pins the new word's block, 15620,
    // and keeps the move's value diff on 15608 and 15609.
    //
    // **Item 736 moved it 15619 → 16460**, past run211's last block
    // (15859): `action_siege_attack_to`'s sub-group lays out on its own
    // cleared record (`docs/GROUPS.md` §26). run211's test keeps the
    // move's value diff (nothing parts on 15441..15859). **Item 736 paid
    // it**: run218 is run211's line over [15854, 16711], and
    // `run218_s_word_frame_is_widened_whole` walks it from run123's 11400
    // across thirteen captures and pins the word's block.
    //
    // **Item 742 moved it 16460 → 17099**, past run218's last block
    // (16711): `move_step`'s give-up takes no step (`docs/COLLISION.md`
    // §17). run218's test keeps the move's value diff (nothing parts on
    // 15860..16711). **Item 742 paid it**: run226 is run218's line over
    // [16706, 17350], and `run226_s_word_frame_is_widened_whole` walks it
    // from run123's 11400 across fourteen captures and pins the word's
    // block.
    //
    // **Item 757 moved it 17099 → 17128**, inside run226: the Pyramids'
    // commerce cap and food terms (`docs/ECONOMY.md` §15). The same test
    // pins the new word's block, 17129, and keeps the move's value diff.
    //
    // **Item 776 moved it 17128 → 17181**, inside run226: `invalid_loc`'s
    // cell arm under `valid_wcoord` (`docs/PATHFINDER.md` §27). The same
    // test pins the new word's block, 17182, and keeps the move's value
    // diff (nothing parts on 17088..17181).
    //
    // **Item 785 moved it 17181 → 20568**, past run226's last block:
    // `wonder_mark`'s writer (`docs/AI.md` §75). run226's test keeps the
    // move's value diff (nothing parts on run226's own blocks). **Item 785
    // paid it**: run243 is run226's line over [20500, 20819), sized to the
    // word with no dump over 17351..20499, and
    // `run243_s_word_frame_is_widened_whole` walks run226's last six
    // blocks and run243 whole and pins the word's block.
    //
    // **Item 795 moved it 20568 → 20800**, inside run243: `do_move`'s TAKE
    // reads the stack again (`docs/GROUPS.md` §32). The same test pins
    // the new word's block, 20801, and `run294_s_departure_is_widened_
    // whole` (run294, 19840..19999, a bisection of the gap) keeps the
    // move's value diff. run243's runway past the new word is 18 blocks.
    //
    // **Item 899 moved it 20800 → 24000**, the trace's own end:
    // `find_wpath`'s `is_attacking` (`docs/PATHFINDER.md` §29). No frame of
    // run53's trace parts. `run80_s_word_frame_is_widened_whole` widens
    // run80, 23960..24001, the last blocks any dump holds, and pins the
    // end; run243's and run294's tests keep the move's value diff.
    (
        "LONG_WORD_GREAT_LAKES",
        LONG_WORD_GREAT_LAKES,
        Some("run80_s_word_frame_is_widened_whole"),
        899,
        Some(WIDENING_GREAT_LAKES_END),
    ),
    // Item 445 paid the widening chapter one had never had: the word
    // stood at 626 from item 405 on, and every test behind it pinned one
    // mechanism (the seating, the reach, the hit, the hand-off), not the
    // cast. `docs/COMBAT.md` §48.
    (
        "GOLDEN_WORD_CHAPTER_ONE",
        GOLDEN_WORD_CHAPTER_ONE,
        Some("chapter_one_s_word_frame_is_widened_whole"),
        445,
        Some(WIDENING_CHAPTER_ONE),
    ),
    // Item 447 moved this word 616 → 624 and wrote its widening in the
    // same landing rather than leaving the row owing one — the lesson
    // parked 449 drew on Great Lakes the same day, applied at the move
    // instead of after it. The name that stood here was item 441's, and it
    // widens **616**, a frame this word has left.
    // Item 479 moved it 645 → **680** and widened the new word in the
    // same landing, the third chapter-two move in a row to do so: the
    // window's ceiling is four frames past 680 and its floor is still
    // run112's own first block, which is the only floor that cannot hide
    // a row. Item 481 moved it 680 → **683** on the same terms, and the
    // ceiling's move is what surfaced `1/8`'s death on 683 — four frames
    // past the word is enough to say where the parting opens and not
    // enough to see what the next one is.
    //
    // **Item 484 widened the record rather than the window**, and the
    // map went from three rows to eight with the earliest at **656** —
    // twenty-seven blocks *under* the word. `compare` carried no
    // hit-point row at all, so `myhits`, `damage` and `damage_frac`
    // went uncompared on every capture ever taken and this window was
    // reporting agreement it had never measured for the third distinct
    // reason: the floor (470), the ceiling (481), and now a field
    // missing from the comparison, which no widening of the window
    // could have found. `docs/COMBAT.md` §40.
    //
    // **Item 485 answered it and the answer was two facts.** `recharging`
    // agrees on every unit-frame of the window, so the swing frames were
    // never the fault; `damage_frame` — the dump's own stamp, also new to
    // `compare` — parted by exactly one on both wounded units; and
    // run112's `AMMO` records, printed since the capture was taken and
    // read by nothing, name the launch point. The original's arrows leave
    // the bow hand eighty-odd units ahead of the shooter and this crate's
    // left the shooter's own square, which lengthened four of the window's
    // nine flights by a frame. §7.3's divide is real and it is 684's fact,
    // not 656's. The map goes from eight rows to **five**.
    // `docs/COMBAT.md` §41.
    //
    // **Item 491 moved it 683 → 695** and moved the ceiling with it, 687
    // → 699, the sixth chapter-two move in a row to widen the new word in
    // the same landing. Both halves of 683's parting were real and both
    // were 485's hypotheses: the death draw `Unit::close` takes was
    // named right, and the puncture's mechanism was named wrong. §9.2's
    // "a target that is no longer active has its `hold_frames` bumped" is
    // the **shooter**'s slot, not the target's (`Object::die@00647080`
    // matches on the ammo's `+0x3c`/`+0x40`, which `Ammo::init` fills
    // from the shooter), and it has nothing to do with the puncture. What
    // does is `Ammo::inc_time`'s flag-4 arm, and run112 had printed it
    // since the day the capture was taken: `flags 6` on every arrow at a
    // land unit, `flags 14` and `whom -1 ox -1` on the one that missed.
    // `docs/DECISIONS.md` 42 for the fifth time in this chain.
    //
    // **Item 496 widened the record rather than the window again**, and
    // the map's first parting went 686 → **684**: the three `order` rows
    // move 696 → 684 because `compare_orders` reported a target only when
    // *both* sides named one, so this crate's attack order carrying
    // nothing against the dump's `ox 8 whom 1` read as agreement for the
    // twelve frames between `1/8`'s death and the bowmen's next reload.
    // The same shape item 462 fixed one level down in `unit_ids`.
    // `docs/COMBAT.md` §43.
    //
    // **Item 510 widened the record a third time, and it was the biggest
    // hole yet**: the walk had never built a `GUY` row at all, and
    // `compare` builds none either, so the *animation clock* — the record
    // chapter two's word has been spent in since 695 — was compared
    // nowhere. Worse, run112 is a `GUYS=2` capture and prints none of it,
    // so a walk that had built the rows would still have compared
    // `Some(ours)` against `None` on every one. The clock now comes from
    // run118, the same game at `GUYS=4`, keyed and asserted block by
    // block. Eleven rows became thirty-nine. `docs/COMBAT.md` §45.
    //
    // **Item 523 widened the record a fourth time** (`o_up`, `o_down`)
    // and the window to the whole of run112, because the word reached the
    // capture's end: 762 → 900. Twenty rows became eleven, the two
    // standing residues. `docs/COMBAT.md` §47.
    (
        "GOLDEN_WORD_CHAPTER_TWO",
        GOLDEN_WORD_CHAPTER_TWO,
        Some("chapter_two_s_word_frame_is_widened_whole"),
        523,
        Some(WIDENING_CHAPTER_TWO),
    ),
    (
        "GOLDEN_WORD_CHAPTER_FIVE",
        GOLDEN_WORD_CHAPTER_FIVE,
        Some("chapter_five_s_word_frame_is_widened_whole"),
        535,
        Some(WIDENING_CHAPTER_FIVE),
    ),
    (
        "GOLDEN_WORD_CHAPTER_FOUR",
        GOLDEN_WORD_CHAPTER_FOUR,
        Some("chapter_four_s_word_frame_is_widened_whole"),
        567,
        Some(WIDENING_CHAPTER_FOUR),
    ),
    // Item 648: run168, chapter six's first walk, on the Bomber's birth.
    // Item 652 closed it at 900 with `bird` staged, and widened run168 whole.
    (
        "GOLDEN_WORD_CHAPTER_SIX",
        GOLDEN_WORD_CHAPTER_SIX,
        Some("chapter_six_s_word_frame_is_widened_whole"),
        652,
        Some(WIDENING_CHAPTER_SIX),
    ),
    // Item 651: run175, chapter six-b's first walk, on the Fighter's
    // arrival at its attack point beside the enemy Airbase. Item 680
    // closed it at 1250 (a building target's re-search) and widened
    // run175 whole.
    (
        "GOLDEN_WORD_CHAPTER_SIX_B",
        GOLDEN_WORD_CHAPTER_SIX_B,
        Some("chapter_six_b_s_word_frame_is_widened_whole"),
        680,
        Some(WIDENING_CHAPTER_SIX_B),
    ),
    // Item 676: run180, chapter nine's first walk, on the Chariot's
    // re-plan at the sand — the first issuer chapter; closed at 1100 by
    // the same item (a human's fog arm and `find_wpath` pop).
    (
        "GOLDEN_WORD_CHAPTER_NINE",
        GOLDEN_WORD_CHAPTER_NINE,
        Some("chapter_nine_s_word_frame_is_widened_whole"),
        676,
        Some(WIDENING_CHAPTER_NINE),
    ),
    // Item 693: run184, chapter ten's first walk, on the chariot's idle
    // animation where the original's walks its patrol's first leg; closed
    // at 1250 by the same item (the ground patrol, built).
    (
        "GOLDEN_WORD_CHAPTER_TEN",
        GOLDEN_WORD_CHAPTER_TEN,
        Some("chapter_ten_s_word_frame_is_widened_whole"),
        693,
        Some(WIDENING_CHAPTER_TEN),
    ),
    // Item 696: run190, chapter eleven's first walk at 724, on the wagon's
    // first steps under a player's move pushing its guard; 734 by the
    // land push and the escort's soft row, the same pair one step on.
    (
        "GOLDEN_WORD_CHAPTER_ELEVEN",
        GOLDEN_WORD_CHAPTER_ELEVEN,
        Some("chapter_eleven_s_word_frame_is_widened_whole"),
        696,
        Some(WIDENING_CHAPTER_ELEVEN),
    ),
    // Item 714: run204, chapter twelve's first walk at 717, on the
    // follow this crate could not take: the chariot's first leg on 711.
    // Closed at 1150 by the same item (the follow, built).
    (
        "GOLDEN_WORD_CHAPTER_TWELVE",
        GOLDEN_WORD_CHAPTER_TWELVE,
        Some("chapter_twelve_s_word_frame_is_widened_whole"),
        714,
        Some(WIDENING_CHAPTER_TWELVE),
    ),
    // Item 718: run208, chapter thirteen's first walk at 640, on the
    // garrison this crate could not take: the chariot's leg on 622.
    (
        "GOLDEN_WORD_CHAPTER_THIRTEEN",
        GOLDEN_WORD_CHAPTER_THIRTEEN,
        Some("chapter_thirteen_s_word_frame_is_widened_whole"),
        718,
        Some(WIDENING_CHAPTER_THIRTEEN),
    ),
    // Item 723: run210, chapter fourteen's first walk at 631, on the
    // formation this crate could not take: the leader's idle roll on its
    // Envelop slot, its plain move laid on 622. Closed at 1150 by the
    // same item (the formation command, an equal group's slot kept, and
    // the replay to `orig`).
    (
        "GOLDEN_WORD_CHAPTER_FOURTEEN",
        GOLDEN_WORD_CHAPTER_FOURTEEN,
        Some("chapter_fourteen_s_word_frame_is_widened_whole"),
        723,
        Some(WIDENING_CHAPTER_FOURTEEN),
    ),
    // Item 731: run215, chapter fifteen's first walk at 753, on the
    // attack this crate could not take: the six's `ATTACKORDER`s laid on
    // 736, the charge's first roll on 753; and the first golden pool,
    // widened on the same blocks. Closed at 1250 by the same item (the
    // attack command entered, a fresh slot's stamp).
    (
        "GOLDEN_WORD_CHAPTER_FIFTEEN",
        GOLDEN_WORD_CHAPTER_FIFTEEN,
        Some("chapter_fifteen_s_word_frame_is_widened_whole"),
        731,
        Some(WIDENING_CHAPTER_FIFTEEN),
    ),
    // Item 738: run219, chapter sixteen's first walk at 838, on the
    // re-issue behind a goody-box leg: `finish_insert` replays a plain
    // move to its `orig`, which this crate did not carry; the chariot's
    // plan goal on 685, its arrival on 838; and the pool on the same
    // blocks. Closed at 1250 by the same item (`orig` carried).
    (
        "GOLDEN_WORD_CHAPTER_SIXTEEN",
        GOLDEN_WORD_CHAPTER_SIXTEEN,
        Some("chapter_sixteen_s_word_frame_is_widened_whole"),
        738,
        Some(WIDENING_CHAPTER_SIXTEEN),
    ),
    // Item 746: run223, chapter seventeen's first walk at 642, on the
    // flight this crate did not take: the Fighter's first `cruising_alt`
    // draw on its strafe home. Item 759 flew the flight home and moved
    // the word to 805, the first bomb, and widened it whole on
    // (605, 807): the first value parting is the pair's patrol on 666.
    // Item 770 released the strafe's round and closed it at 1400, widened
    // on run223 whole: what stands is the tank from 1212 (parked 765).
    (
        "GOLDEN_WORD_CHAPTER_SEVENTEEN",
        GOLDEN_WORD_CHAPTER_SEVENTEEN,
        Some("chapter_seventeen_s_word_frame_is_widened_whole"),
        770,
        Some(WIDENING_CHAPTER_SEVENTEEN),
    ),
    // Item 779: run241, chapter eighteen's first walk at 642, on the
    // build command this crate did not take: an idle citizen's animation
    // roll where the original's walks to its site; the sites, the prices
    // and each builder's two orders on 622 and 642. Closed at 1450 by the
    // same item (the command entered, a human's approach a move).
    (
        "GOLDEN_WORD_CHAPTER_EIGHTEEN",
        GOLDEN_WORD_CHAPTER_EIGHTEEN,
        Some("chapter_eighteen_s_word_frame_is_widened_whole"),
        779,
        Some(WIDENING_CHAPTER_EIGHTEEN),
    ),
    // Item 790: run245, chapter nineteen's first walk at 669, on the spell
    // command this crate did not take: an idle Spy's animation roll where
    // the original's walks to its ring spot; its cast and move on 622.
    // Closed at 1100 with the command entered; the widening is run245
    // whole.
    (
        "GOLDEN_WORD_CHAPTER_NINETEEN",
        GOLDEN_WORD_CHAPTER_NINETEEN,
        Some("chapter_nineteen_s_word_frame_is_widened_whole"),
        790,
        Some(WIDENING_CHAPTER_NINETEEN),
    ),
    // Item 803: run249, chapter twenty's first walk, closed at 1300: the
    // set-transport command entered with the chapter, before the capture;
    // the widening is run249 whole.
    (
        "GOLDEN_WORD_CHAPTER_TWENTY",
        GOLDEN_WORD_CHAPTER_TWENTY,
        Some("chapter_twenty_s_word_frame_is_widened_whole"),
        803,
        Some(WIDENING_CHAPTER_TWENTY),
    ),
    // Item 813: run255, chapter twenty-one's first walk, open at 1141:
    // the swarm command entered with the chapter, before the capture; the
    // word past the repair, in the idle citizens' gather walks. Item 824
    // closed it at 1300: a human's found gather drops its group. The
    // widening is run255 whole.
    (
        "GOLDEN_WORD_CHAPTER_TWENTY_ONE",
        GOLDEN_WORD_CHAPTER_TWENTY_ONE,
        Some("chapter_twenty_one_s_word_frame_is_widened_whole"),
        824,
        Some(WIDENING_CHAPTER_TWENTY_ONE),
    ),
    // Item 836: run265, chapter twenty-two's first walk, open at 778:
    // the launched Fighter's first redraw; the launch line built in the
    // same item moved it to 923, the Fighter's first attack. Item 842
    // closed it at 1500: the strafer's half altitude and exact round. The
    // widening is run265 whole.
    (
        "GOLDEN_WORD_CHAPTER_TWENTY_TWO",
        GOLDEN_WORD_CHAPTER_TWENTY_TWO,
        Some("chapter_twenty_two_s_word_frame_is_widened_whole"),
        836,
        Some(WIDENING_CHAPTER_TWENTY_TWO),
    ),
    // Item 867: run281, chapter twenty-three's first walk.
    (
        "GOLDEN_WORD_CHAPTER_TWENTY_THREE",
        GOLDEN_WORD_CHAPTER_TWENTY_THREE,
        Some("chapter_twenty_three_s_word_frame_is_widened_whole"),
        867,
        Some(WIDENING_CHAPTER_TWENTY_THREE),
    ),
    // Item 877: run285, chapter twenty-four's first walk, open at 855:
    // the Hoplites' training, which this crate did not queue; the
    // commands entered in the same item closed it at 1560. The widening
    // is run285 whole.
    (
        "GOLDEN_WORD_CHAPTER_TWENTY_FOUR",
        GOLDEN_WORD_CHAPTER_TWENTY_FOUR,
        Some("chapter_twenty_four_s_word_frame_is_widened_whole"),
        877,
        Some(WIDENING_CHAPTER_TWENTY_FOUR),
    ),
    // Item 884: run292, chapter twenty-five's first walk, open at 855:
    // the head Hoplite the original cancelled; the cancel entered in the
    // same item closed it at 1466. The widening is run292 whole.
    (
        "GOLDEN_WORD_CHAPTER_TWENTY_FIVE",
        GOLDEN_WORD_CHAPTER_TWENTY_FIVE,
        Some("chapter_twenty_five_s_word_frame_is_widened_whole"),
        884,
        Some(WIDENING_CHAPTER_TWENTY_FIVE),
    ),
    // Item 883: run296, chapter twenty-six's first walk, closed at 1492
    // with the research skipped: the research spends no draw, and the
    // widening's value rows are its measure. The widening is run296 whole.
    (
        "GOLDEN_WORD_CHAPTER_TWENTY_SIX",
        GOLDEN_WORD_CHAPTER_TWENTY_SIX,
        Some("chapter_twenty_six_s_word_frame_is_widened_whole"),
        883,
        Some(WIDENING_CHAPTER_TWENTY_SIX),
    ),
    // Item 901: run300, chapter twenty-seven's first walk, open at 1102:
    // the Slingers entry the original re-targets to Javelineers on 922
    // trained a Slinger squad here; the queue loop built in the same item
    // closed it at 1560. The widening is run300 whole.
    (
        "GOLDEN_WORD_CHAPTER_TWENTY_SEVEN",
        GOLDEN_WORD_CHAPTER_TWENTY_SEVEN,
        Some("chapter_twenty_seven_s_word_frame_is_widened_whole"),
        901,
        Some(WIDENING_CHAPTER_TWENTY_SEVEN),
    ),
    // Item 888: run304, chapter twenty-eight, two buildings under one
    // command. The widening is run304 whole.
    (
        "GOLDEN_WORD_CHAPTER_TWENTY_EIGHT",
        GOLDEN_WORD_CHAPTER_TWENTY_EIGHT,
        Some("chapter_twenty_eight_s_word_frame_is_widened_whole"),
        888,
        Some(WIDENING_CHAPTER_TWENTY_EIGHT),
    ),
    // Item 915: run308, chapter twenty-nine, the repeat launch. The
    // widening is run308 whole.
    (
        "GOLDEN_WORD_CHAPTER_TWENTY_NINE",
        GOLDEN_WORD_CHAPTER_TWENTY_NINE,
        Some("chapter_twenty_nine_s_word_frame_is_widened_whole"),
        915,
        Some(WIDENING_CHAPTER_TWENTY_NINE),
    ),
    // Item 928: run312, chapter thirty, the gather point. The widening
    // is run312 whole.
    (
        "GOLDEN_WORD_CHAPTER_THIRTY",
        GOLDEN_WORD_CHAPTER_THIRTY,
        Some("chapter_thirty_s_word_frame_is_widened_whole"),
        928,
        Some(WIDENING_CHAPTER_THIRTY),
    ),
    // Item 955: run338, chapter thirty-one, the gather point's other
    // arms. The widening is run338 whole.
    (
        "GOLDEN_WORD_CHAPTER_THIRTY_ONE",
        GOLDEN_WORD_CHAPTER_THIRTY_ONE,
        Some("chapter_thirty_one_s_word_frame_is_widened_whole"),
        955,
        Some(WIDENING_CHAPTER_THIRTY_ONE),
    ),
    // Item 947: run344, chapter thirty-two, an Airbase's gather point. The
    // widening is run344 whole.
    (
        "GOLDEN_WORD_CHAPTER_THIRTY_TWO",
        GOLDEN_WORD_CHAPTER_THIRTY_TWO,
        Some("chapter_thirty_two_s_word_frame_is_widened_whole"),
        947,
        Some(WIDENING_CHAPTER_THIRTY_TWO),
    ),
    // Item 976: run358, chapter thirty-three, an Airbase's launch issuers.
    // The widening is run358 whole.
    (
        "GOLDEN_WORD_CHAPTER_THIRTY_THREE",
        GOLDEN_WORD_CHAPTER_THIRTY_THREE,
        Some("chapter_thirty_three_s_word_frame_is_widened_whole"),
        976,
        Some(WIDENING_CHAPTER_THIRTY_THREE),
    ),
    // Item 1009: run362, chapter thirty-four, the launch commands' other
    // arms. The widening is run362 whole.
    (
        "GOLDEN_WORD_CHAPTER_THIRTY_FOUR",
        GOLDEN_WORD_CHAPTER_THIRTY_FOUR,
        Some("chapter_thirty_four_s_word_frame_is_widened_whole"),
        1009,
        Some(WIDENING_CHAPTER_THIRTY_FOUR),
    ),
    // Item 1019: run371, chapter thirty-five, the Helicopter's and
    // missiles' launch arms. The widening is run371 whole.
    (
        "GOLDEN_WORD_CHAPTER_THIRTY_FIVE",
        GOLDEN_WORD_CHAPTER_THIRTY_FIVE,
        Some("chapter_thirty_five_s_word_frame_is_widened_whole"),
        1019,
        Some(WIDENING_CHAPTER_THIRTY_FIVE),
    ),
    // Item 1078: run390, chapter thirty-six, the missile's other arms.
    // The widening is run390 whole.
    (
        "GOLDEN_WORD_CHAPTER_THIRTY_SIX",
        GOLDEN_WORD_CHAPTER_THIRTY_SIX,
        Some("chapter_thirty_six_s_word_frame_is_widened_whole"),
        1078,
        Some(WIDENING_CHAPTER_THIRTY_SIX),
    ),
    // Item 1091: run397, chapter thirty-seven, the nuke. The widening is
    // run397 whole.
    (
        "GOLDEN_WORD_CHAPTER_THIRTY_SEVEN",
        GOLDEN_WORD_CHAPTER_THIRTY_SEVEN,
        Some("chapter_thirty_seven_s_word_frame_is_widened_whole"),
        1091,
        Some(WIDENING_CHAPTER_THIRTY_SEVEN),
    ),
    // Item 1102: run404, chapter thirty-eight, the air line under fire.
    // The widening is run404 whole.
    (
        "GOLDEN_WORD_CHAPTER_THIRTY_EIGHT",
        GOLDEN_WORD_CHAPTER_THIRTY_EIGHT,
        Some("chapter_thirty_eight_s_word_frame_is_widened_whole"),
        1102,
        Some(WIDENING_CHAPTER_THIRTY_EIGHT),
    ),
    // Item 1111: run422, chapter thirty-nine, the spell issuer's
    // untargeted crafts. The widening is run422 whole.
    (
        "GOLDEN_WORD_CHAPTER_THIRTY_NINE",
        GOLDEN_WORD_CHAPTER_THIRTY_NINE,
        Some("chapter_thirty_nine_s_word_frame_is_widened_whole"),
        1111,
        Some(WIDENING_CHAPTER_THIRTY_NINE),
    ),
    // Items 1310 and 1350: run514, chapter forty-seven, the Keep on a
    // standing Tower site, a squad ashore facing away from its barge, and
    // an age's snap mid-move; closed at 1400. The widening is run514 whole.
    (
        "GOLDEN_WORD_CHAPTER_FORTY_SEVEN",
        GOLDEN_WORD_CHAPTER_FORTY_SEVEN,
        Some("chapter_forty_seven_s_word_frame_is_widened_whole"),
        1350,
        Some(WIDENING_CHAPTER_FORTY_SEVEN),
    ),
    // Item 1358: run551, chapter forty-eight, a who=1 Tower site on who=0's
    // land, struck and bombed while its builders work. The widening is
    // run551 whole.
    (
        "GOLDEN_WORD_CHAPTER_FORTY_EIGHT",
        GOLDEN_WORD_CHAPTER_FORTY_EIGHT,
        Some("chapter_forty_eight_s_word_frame_is_widened_whole"),
        1358,
        Some(WIDENING_CHAPTER_FORTY_EIGHT),
    ),
    // Item 1404: run577, chapter fifty, two Stockades at the first age and a
    // group attack-move through their range. The widening is run577 whole.
    (
        "GOLDEN_WORD_CHAPTER_FIFTY",
        GOLDEN_WORD_CHAPTER_FIFTY,
        Some("chapter_fifty_s_word_frame_is_widened_whole"),
        1404,
        Some(WIDENING_CHAPTER_FIFTY),
    ),
    // Item 1393: run576, chapter forty-nine, a Tower's arrow at its owner's
    // third age and a Catapult trained under a queued Bombard; open at 934.
    // The widening is run576 whole.
    (
        "GOLDEN_WORD_CHAPTER_FORTY_NINE",
        GOLDEN_WORD_CHAPTER_FORTY_NINE,
        Some("chapter_forty_nine_s_word_frame_is_widened_whole"),
        1393,
        Some(WIDENING_CHAPTER_FORTY_NINE),
    ),
    // Items 1278 and 1291: run496, chapter forty-six, the squad's landing
    // and `all_gathering`'s prune, closed at 2200. The widening is run496
    // whole.
    (
        "GOLDEN_WORD_CHAPTER_FORTY_SIX",
        GOLDEN_WORD_CHAPTER_FORTY_SIX,
        Some("chapter_forty_six_s_word_frame_is_widened_whole"),
        1291,
        Some(WIDENING_CHAPTER_FORTY_SIX),
    ),
    // Item 1268: run492, chapter forty-five, `get_cost`'s library-line
    // tail on the golden start. The widening is run492 whole.
    (
        "GOLDEN_WORD_CHAPTER_FORTY_FIVE",
        GOLDEN_WORD_CHAPTER_FORTY_FIVE,
        Some("chapter_forty_five_s_word_frame_is_widened_whole"),
        1268,
        Some(WIDENING_CHAPTER_FORTY_FIVE),
    ),
    // Item 1254: run484, chapter forty-four, the idle search's head on a
    // computer's fleet. The widening is run484 to the word's block and
    // 250 past it.
    (
        "GOLDEN_WORD_CHAPTER_FORTY_FOUR",
        GOLDEN_WORD_CHAPTER_FORTY_FOUR,
        Some("chapter_forty_four_s_word_frame_is_widened_whole"),
        1254,
        Some(WIDENING_CHAPTER_FORTY_FOUR),
    ),
    // Item 1223: run466, chapter forty-three, the landing's arms; closed at
    // 2200 (item 1248). The widening is run466 whole.
    (
        "GOLDEN_WORD_CHAPTER_FORTY_THREE",
        GOLDEN_WORD_CHAPTER_FORTY_THREE,
        Some("chapter_forty_three_s_word_frame_is_widened_whole"),
        1223,
        Some(WIDENING_CHAPTER_FORTY_THREE),
    ),
    // Item 1209: run460, chapter forty-two, a ring of Barracks and four
    // walkers; closed at 1160. The widening is run460 whole.
    (
        "GOLDEN_WORD_CHAPTER_FORTY_TWO",
        GOLDEN_WORD_CHAPTER_FORTY_TWO,
        Some("chapter_forty_two_s_word_frame_is_widened_whole"),
        1209,
        Some(WIDENING_CHAPTER_FORTY_TWO),
    ),
    // Items 1182 and 1200: run437, chapter forty-one, the computer's
    // sortie, the build-site spill and the Biplane's strafes; closed at
    // 1770. The widening is run437 whole.
    (
        "GOLDEN_WORD_CHAPTER_FORTY_ONE",
        GOLDEN_WORD_CHAPTER_FORTY_ONE,
        Some("chapter_forty_one_s_word_frame_is_widened_whole"),
        1182,
        Some(WIDENING_CHAPTER_FORTY_ONE),
    ),
    // Item 1167: run430, chapter forty, the casts' other arms. The
    // widening is run430 whole.
    (
        "GOLDEN_WORD_CHAPTER_FORTY",
        GOLDEN_WORD_CHAPTER_FORTY,
        Some("chapter_forty_s_word_frame_is_widened_whole"),
        1167,
        Some(WIDENING_CHAPTER_FORTY),
    ),
    // Item 660: run171, chapter eight's first walk, on the Spy's birth.
    (
        "GOLDEN_WORD_CHAPTER_EIGHT",
        GOLDEN_WORD_CHAPTER_EIGHT,
        Some("chapter_eight_s_word_frame_is_widened_whole"),
        660,
        Some(WIDENING_CHAPTER_EIGHT),
    ),
    // Item 587: run145, the catapult's birth frame.
    (
        "GOLDEN_WORD_CHAPTER_THREE",
        GOLDEN_WORD_CHAPTER_THREE,
        Some("chapter_three_s_word_frame_is_widened_whole"),
        587,
        Some(WIDENING_CHAPTER_THREE),
    ),
    // Item 587: run146, chapter three's restage, the chariots' chase.
    (
        "GOLDEN_WORD_CHAPTER_THREE_RESTAGE",
        GOLDEN_WORD_CHAPTER_THREE_RESTAGE,
        Some("chapter_three_s_restage_is_widened_whole"),
        587,
        Some(WIDENING_CHAPTER_THREE_RESTAGE),
    ),
    // Item 578: closed at its first walk, on both run141 and its control.
    (
        "GOLDEN_WORD_CHAPTER_SEVEN",
        GOLDEN_WORD_CHAPTER_SEVEN,
        Some("chapter_seven_s_word_frame_is_widened_whole"),
        578,
        Some(WIDENING_CHAPTER_SEVEN),
    ),
    // Item 628: chapter seven-b, run156, and its control run157 — two
    // words, one widening over both captures.
    (
        "GOLDEN_WORD_CHAPTER_SEVEN_B",
        GOLDEN_WORD_CHAPTER_SEVEN_B,
        Some("chapter_seven_b_s_word_frame_is_widened_whole"),
        628,
        Some(WIDENING_CHAPTER_SEVEN_B),
    ),
    (
        "GOLDEN_WORD_CHAPTER_SEVEN_B_CONTROL",
        GOLDEN_WORD_CHAPTER_SEVEN_B_CONTROL,
        Some("chapter_seven_b_s_word_frame_is_widened_whole"),
        632,
        Some(WIDENING_CHAPTER_SEVEN_B_CONTROL),
    ),
    // **The second pair** (DECISIONS 53 §2, item 971): the AI track's word
    // on run54's and run53's games at Toughest. East Indies' frame-0
    // parting is widened on run349; Great Lakes' shares its frame and its
    // coin.
    // Item 979's harness fix moved both past their windows: East Indies to
    // 1576, widened on run352, and Great Lakes to 3776, on run355. Item
    // 989's close tail moved both again: East Indies to 5606, widened on
    // run357, and Great Lakes to 4555, on run356. Item 1040 moved Great
    // Lakes past run356's window to 4846, widened on run373, and item 1074
    // past run373's to 5105, widened on run396; item 1089 moved it to 5161,
    // inside run396's window, and item 1099 to 5930, the game's end,
    // widened on run403 (its last six blocks) and scored whole on run347's
    // closing state. Item 1106 moved East Indies to 5773, inside run357's
    // window, and re-pinned the word's block there. Item 1115 moved it to 5975,
    // widened on run414, and item 1120 to 6151, inside run414's window.
    // Item 1127 moved it to 6321, past it, widened on run419; item 1143 to
    // 6609, past that, widened on run420; item 1156 to 6743, inside it;
    // item 1164 to 7382, past it, widened on run425; item 1174 to 7512,
    // inside it; item 1185 to 8519, past it, widened on run439; item 1191
    // to 8820, past it, widened on run445; item 1197 to 8907, inside it;
    // item 1214 to 10183, past it, widened on run462; item 1228 to 10185,
    // inside it; item 1243 to 10985, past it, widened on run480; item 1264
    // to 11328, past it, widened on run490; item 1281 to 11549, inside it;
    // item 1297 to 11637, past it, widened on run506; item 1302 to 12582,
    // past it, widened on run508; item 1326 to 13385, past it, widened on
    // run523; item 1341 to 14141, past it, widened on run535; item 1351
    // to 15862, past it, widened on run544; item 1362 to 15883, inside it;
    // item 1370 to 15985, inside it; item 1377 to 16009, inside it; item
    // 1383 to 16160, past it, widened on run572; item 1401 to 16179,
    // inside it; item 1418 to 16482, past it, widened on run579.
    (
        "THIRD_PAIR_WORD_EAST_INDIES",
        THIRD_PAIR_WORD_EAST_INDIES,
        Some("run634_s_word_frame_is_widened_whole"),
        1460,
        Some(WIDENING_FRENCH_EAST_INDIES),
    ),
    (
        "THIRD_PAIR_WORD_GREAT_LAKES",
        THIRD_PAIR_WORD_GREAT_LAKES,
        Some("run609_s_closing_frame_is_widened_whole"),
        1444,
        Some(WIDENING_FRENCH_LAKES_CLOSING),
    ),
    (
        "SECOND_WORD_EAST_INDIES",
        SECOND_WORD_EAST_INDIES,
        Some("run594_s_word_frame_is_widened_whole"),
        1441,
        Some(WIDENING_SECOND_EAST_INDIES_18076),
    ),
    (
        "SECOND_WORD_GREAT_LAKES",
        SECOND_WORD_GREAT_LAKES,
        Some("run403_s_word_frame_is_widened_whole"),
        1099,
        Some(WIDENING_SECOND_GREAT_LAKES_5930),
    ),
    // **The third map, Great Sahara** (DECISIONS 54 §3, item 1066): its
    // word was frame 8 on both run382 and run383, and run382 carries every
    // block from 1 to 259 at run10's detail, so the score capture was its
    // own widening (`diff::third`). Item 1133 moved it to **12783**, past
    // every block run382 holds, and took run416 over 12778..13034; run382's
    // test keeps the move's value diff on block 6. Item 1147 moved it to
    // **13182**, past run416's last block, and took run417 over
    // 13177..13433; run416's test keeps the move's value diff on 12780.
    // Item 1163 moved it to **14587**, past run417's last block, and took
    // run418 over 14582..14838; run416's test keeps the move's value diff
    // on 12784. Item 1171 moved it to **15586**, past run418's last block,
    // and took run426 over 15581..15837; run418's test keeps the move's
    // value diff on 14587. Item 1177 moved it to **15982**, past run426's
    // last block, and took run428 over 15977..16233; run426's test keeps
    // the move's value diff on 15585. Item 1189 moved it to **16681**,
    // past run428's last block, and took run442 over 16676..16932;
    // run418's test keeps the move's value diff on 14661 and run428's on
    // 15983. Item 1194 moved it to **17623**, past run442's last block,
    // and took run449 over 17618..17874; run442's test keeps the move's
    // value diff on 16681 and 16682, and run428's on 15978. Item 1206
    // moved it to **24000**, the trace's own end, on `refresh_group_order`'s
    // tail; run457 (17140..17618, the gap run449's first block stood on)
    // and run449 keep the move's value diff, and run458, the game's last
    // blocks, is widened whole.
    (
        "LONG_WORD_GREAT_SAHARA",
        LONG_WORD_GREAT_SAHARA,
        Some("run458_s_word_frame_is_widened_whole"),
        1206,
        Some(crate::diff::third::WIDENING_GREAT_SAHARA_END),
    ),
    // **The third map at Toughest** (DECISIONS 56 §1): its first word, 5376,
    // past run469's 1,850 blocks, was widened on run471 over 5371..5627
    // (item 1221); the word 5782 on run476 over 5777..6033 (item 1241); the
    // word 7070 on run483 over 7065..7321 (item 1251); the word 7785 on
    // run488 over 7780..8036 (item 1260); the word 8182 on run491 over
    // 8177..8433 (item 1264), and the word 8377 on the same blocks (item
    // 1275); the word 8786 on run500 over 8781..9037 (item 1286), and the
    // word 8856 on the same blocks (item 1293); the word 9323 on run511
    // over 9318..9574 (item 1305), whose run500 test keeps the move's
    // value diff on 8858; the word 9352 on the same blocks (item 1318),
    // whose run517 test keeps the move's value diff over the gap
    // 9032..9323; the word 9764 on run529 over 9759..10015 (item 1332),
    // whose run511 test keeps the move's value diff on 9348..9355; the
    // word 9982 on the same blocks (item 1338), whose run529 test keeps
    // the move's value diff on 9759..9765; the word 10144 on run547 over
    // 10139..10395 (item 1354), whose run529 test keeps the move's value
    // diff on 9784..9999; the word 10391 on the same blocks (item 1365),
    // whose run547 test keeps the move's value diff on 10144..10145; the
    // word 10779 on run562 over 10774..11030 (item 1371), whose run547
    // test keeps the move's value diff on 10391..10395; the word 11182 on
    // run571 over 11177..11433 (item 1379), whose run562 test keeps the
    // move's value diff on 10780..10781.
    (
        "THIRD_WORD_GREAT_SAHARA_TOUGHEST",
        THIRD_WORD_GREAT_SAHARA_TOUGHEST,
        // Item 1398 moved the word to 11882, past run571's window; item
        // 1416 widened it on run574 over 11876..12133, whose test keeps the
        // move's value diff on 11876..11988; item 1429 widened 12538 → 12569
        // on run584 over 12532..12789 (the word's block 12570).
        Some("run584_s_word_frame_is_widened_whole"),
        1429,
        Some(crate::diff::sahara_toughest::WIDENING_GREAT_SAHARA_TOUGHEST_12538),
    ),
];

/// **A test that holds several pins reports every one that moved** (parked
/// 973, built by the nineteenth pass). `assert_eq!` stops its test at the
/// first pin that moved, so a re-pin read off one failing run leaves the
/// test's later pins unseen: items 882, 904, 989, 1034 and 1072 each paid a
/// second gate for a pin that sat behind an earlier one's panic, and one
/// of 1072's nine made a killer look as if it had fired.
///
/// A test takes the guard first — `let _pins = Pins::hold();` — and pins
/// with [`pin_eq!`], [`pin_ne!`] and [`pin!`], which read as the `assert`
/// macros do. A pin that moved is recorded and the test goes on; the guard
/// fails the test once, when it drops, with every one. **With no guard on
/// the thread a pin panics where it stands**, so a test that pins and
/// forgot the guard fails as it always did and can never pass on a moved
/// pin. A test that panics past a moved pin prints what it had collected.
pub(crate) mod pins {
    use std::cell::RefCell;

    thread_local! {
        static HELD: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
    }

    /// The guard. One to a test, taken on its first line.
    pub(crate) struct Pins(());

    impl Pins {
        pub(crate) fn hold() -> Pins {
            let was = HELD.with(|h| h.borrow_mut().replace(Vec::new()));
            assert!(was.is_none(), "a second `Pins::hold()` on one thread");
            Pins(())
        }
    }

    /// Record a pin that moved, or panic where no guard holds.
    pub(crate) fn moved(what: String) {
        let unheld = HELD.with(|h| match h.borrow_mut().as_mut() {
            Some(held) => {
                held.push(what);
                None
            }
            None => Some(what),
        });
        if let Some(what) = unheld {
            panic!("{what}");
        }
    }

    impl Drop for Pins {
        fn drop(&mut self) {
            let moved = HELD.with(|h| h.borrow_mut().take()).unwrap_or_default();
            if moved.is_empty() {
                return;
            }
            let report = format!(
                "{} pin{} moved:\n{}",
                moved.len(),
                if moved.len() == 1 { "" } else { "s" },
                moved.join("\n")
            );
            if std::thread::panicking() {
                eprintln!("before the panic, {report}");
            } else {
                panic!("{report}");
            }
        }
    }

    #[test]
    fn a_test_that_holds_several_pins_reports_every_one_that_moved() {
        use crate::diff::testkit::{pin, pin_eq, pin_ne};
        let caught = std::panic::catch_unwind(|| {
            let _pins = Pins::hold();
            pin_eq!((1, 2, 3), (1, 2, 4), "the floor");
            pin_eq!(7, 7, "a pin that holds");
            pin!(1 + 1 == 3, "every row standing on block {}", 20_800);
            pin_ne!(5, 5);
        })
        .expect_err("three pins moved and the guard let the test pass");
        let said = caught.downcast_ref::<String>().cloned().unwrap_or_default();
        assert!(said.starts_with("3 pins moved:"), "{said}");
        for want in [
            "the floor",
            "got  (1, 2, 3)",
            "want (1, 2, 4)",
            "every row standing on block 20800",
            "both 5",
        ] {
            assert!(said.contains(want), "the report names no {want:?}: {said}");
        }
        assert!(!said.contains("a pin that holds"), "{said}");
        // The guard is gone with its test: a pin with none panics in place.
        let bare = std::panic::catch_unwind(|| pin_eq!(1, 2, "no guard"))
            .expect_err("a pin with no guard passed");
        let bare = bare.downcast_ref::<String>().cloned().unwrap_or_default();
        assert!(
            bare.contains("no guard") && !bare.contains("pins moved"),
            "{bare}"
        );
        // A guard whose pins all hold says nothing.
        let _pins = Pins::hold();
        pin_eq!(2 + 2, 4, "arithmetic");
    }
}

pub(crate) use pins::Pins;

/// [`pins`]' `assert_eq!`: both sides by reference, printed when they part.
macro_rules! pin_eq {
    ($got:expr, $want:expr $(,)?) => {
        $crate::diff::testkit::pin_eq!($got, $want, "pin_eq")
    };
    ($got:expr, $want:expr, $($arg:tt)+) => {{
        match (&$got, &$want) {
            (got, want) => {
                if *got != *want {
                    $crate::diff::testkit::pins::moved(format!(
                        "  {}:{}: {}\n    got  {:?}\n    want {:?}",
                        file!(),
                        line!(),
                        format_args!($($arg)+),
                        got,
                        want
                    ));
                }
            }
        }
    }};
}
pub(crate) use pin_eq;

/// [`pins`]' `assert_ne!`.
macro_rules! pin_ne {
    ($got:expr, $not:expr $(,)?) => {
        $crate::diff::testkit::pin_ne!($got, $not, "pin_ne")
    };
    ($got:expr, $not:expr, $($arg:tt)+) => {{
        match (&$got, &$not) {
            (got, not) => {
                if *got == *not {
                    $crate::diff::testkit::pins::moved(format!(
                        "  {}:{}: {}\n    both {:?}",
                        file!(),
                        line!(),
                        format_args!($($arg)+),
                        got
                    ));
                }
            }
        }
    }};
}
pub(crate) use pin_ne;

/// [`pins`]' `assert!`.
macro_rules! pin {
    ($ok:expr $(,)?) => {
        $crate::diff::testkit::pin!($ok, "{}", stringify!($ok))
    };
    ($ok:expr, $($arg:tt)+) => {{
        if !$ok {
            $crate::diff::testkit::pins::moved(format!(
                "  {}:{}: {}",
                file!(),
                line!(),
                format_args!($($arg)+)
            ));
        }
    }};
}
pub(crate) use pin;

/// **The AI track's words, one row a game** (parked 1121, 1080 and 1108;
/// the nineteenth pass). Until this table the handoff's guards each read
/// two constants by name, and none of them knew a game's length: when East
/// Indies' second word passed Great Lakes' 5,930 — that game's **end**,
/// closed by item 1099 — the default-map guard asked the queue to name the
/// map no item can move; nothing read the third map's line at all; and
/// the compared pin's window was held to the word by a comment.
///
/// A row is *closed* when its word is its game's length. A closed row owes
/// the test that scored its closing whole-map state: a draw stream that
/// agrees to a game's end says nothing of the end itself (run347's last
/// event was wrong under an agreeing stream, item 1099). An open row names
/// the function that walks its word's window with the recorder on, and
/// `coverage`'s compared pin walks every one.
pub(crate) struct AiWord {
    /// The handoff line that carries it, without its colon.
    pub line: &'static str,
    /// The map as that line spells it, and as the queue's prose does.
    pub map: &'static str,
    pub named: &'static str,
    pub word: i64,
    /// The trace's last frame, or the frame the game ends on.
    pub length: i64,
    /// Owed once the row is closed: the closing state's own test.
    pub endpoint: Option<&'static str>,
    /// Owed while the row is open: the word's window, under `src/diff/`.
    pub window: Option<&'static str>,
}

impl AiWord {
    pub(crate) fn closed(&self) -> bool {
        self.word >= self.length
    }
}

/// A game's length by its line's spelling of the map; the walks that read
/// each trace to its end hold the table to it.
pub(crate) fn ai_word_length(map: &str) -> i64 {
    AI_WORDS
        .iter()
        .find(|w| w.map == map)
        .unwrap_or_else(|| panic!("AI_WORDS has no row for {map}"))
        .length
}

/// The pair whose lower open word is the AI track's default
/// (`docs/DECISIONS.md` 41 §1, 53 §2).
pub(crate) const NEWEST_PAIR: &str = "Third pair";

pub(crate) const AI_WORDS: &[AiWord] = &[
    AiWord {
        line: "Third pair",
        map: "EastIndiesFrench",
        named: "East Indies (French)",
        word: THIRD_PAIR_WORD_EAST_INDIES,
        length: 17_379,
        endpoint: None,
        window: Some("french_east_indies_word_window"),
    },
    AiWord {
        line: "Third pair",
        map: "GreatLakesFrench",
        named: "Great Lakes (French)",
        word: THIRD_PAIR_WORD_GREAT_LAKES,
        length: 5_638,
        endpoint: Some("run598_french_great_lakes_closing_state"),
        window: None,
    },
    AiWord {
        line: "Second pair",
        map: "EastIndies",
        named: "East Indies",
        word: SECOND_WORD_EAST_INDIES,
        length: 18_140,
        endpoint: Some("run346_is_east_indies_at_toughest_and_its_word_holds"),
        window: None,
    },
    AiWord {
        line: "Second pair",
        map: "GreatLakes",
        named: "Great Lakes",
        word: SECOND_WORD_GREAT_LAKES,
        length: 5_930,
        endpoint: Some("run347_is_great_lakes_at_toughest_and_its_word_holds"),
        window: None,
    },
    AiWord {
        line: "Third map",
        map: "GreatSahara",
        named: "Great Sahara",
        word: LONG_WORD_GREAT_SAHARA,
        length: 24_000,
        endpoint: Some("great_sahara_endpoint_is_pinned"),
        window: None,
    },
    // **The closed map in the newest pair's lobby** (DECISIONS 56 §1, item
    // 1221): run470 ends when the idle human is defeated, its trace's last
    // frame 15432 and its closing dump block 15433.
    AiWord {
        line: "Third map",
        map: "GreatSaharaToughest",
        named: "Great Sahara at Toughest",
        word: THIRD_WORD_GREAT_SAHARA_TOUGHEST,
        length: 15_432,
        endpoint: None,
        window: Some("great_sahara_toughest_word_window"),
    },
];
