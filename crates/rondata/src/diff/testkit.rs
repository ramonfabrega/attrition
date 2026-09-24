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
pub(crate) const LONG_WORD_EAST_INDIES: i64 = 15_782;

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
pub(crate) const LONG_WORD_GREAT_LAKES: i64 = 14_382;

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
pub(crate) const ORDER_RESIDUE_RUN97: usize = 28_220;

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

pub(crate) const CHAPTER_EIGHT_LEADER_KEYS: usize = 94;

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
pub(crate) const GROUND_INEXACT: &[(&str, u32)] = &[];

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
    (
        "LONG_WORD_EAST_INDIES",
        LONG_WORD_EAST_INDIES,
        None,
        694,
        None,
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
    (
        "LONG_WORD_GREAT_LAKES",
        LONG_WORD_GREAT_LAKES,
        Some("run178_s_word_frame_is_widened_whole"),
        678,
        Some(WIDENING_GREAT_LAKES_COPY),
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
];
