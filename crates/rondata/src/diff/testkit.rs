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
pub(crate) const LONG_WORD_EAST_INDIES: i64 = 9_711;

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
/// **10303 is `1/51`'s animation clock.** Ours spends **three** draws and
/// the original **four**, parting at index 3: the extra is
/// `Guy::set_anim+0x97a < Guy::inc_time+0x271`, and block 10304 is two
/// rows of the same unit — `g.cur_time` 30 against 0 and `g.last_time` 29
/// against −1, the original one wrap ahead of this crate.
pub(crate) const LONG_WORD_GREAT_LAKES: i64 = 10_817;

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
pub(crate) const ORDER_RESIDUE_RUN97: usize = 28_222;

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
/// What stands at **626** is the arrival frame itself: the original
/// spends `Guy::set_anim+0xf2f < Guy::move+0x166` there and this crate
/// does not, and the ordering of `Guy::set_anim+0x104b` beside it
/// differs. Both hoplites land exactly on their ordered points on 626 and
/// hold, so the residue is in what an arriving figure rolls, not in where
/// it arrives. `0/8` is the other live one: the dump snaps its
/// `orders_x/orders_y` to `(1044, 8076)` at block 625 with `collide_o 8`,
/// where this crate walks on toward `(1224, 8280)`.
///
/// - ~~**`0/8` plans its chase a frame late.**~~ **Closed by 405**: it
///   was the same short leg, and `0/8` now matches the dump frame for
///   frame from 619 to 624.
pub(crate) const GOLDEN_WORD_CHAPTER_ONE: i64 = 626;

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
/// What is needed to move it is **not** the zero arm on its own: the
/// bit's writer is `Unit::fight`'s "still ordered, reload open, not
/// firing" arm, and reaching it needs the frame the original has and this
/// crate does not — an attack order that keeps its **dead** target from
/// 683 to 695, a captain that searches for the replacement and spends the
/// frame doing it, and two followers that strike on it the same frame.
/// Removing [`sim::Sim::forget`]'s clear alone leaves the word here and
/// turns `near_o 0/7`/`0/8` and `chapter_two_s_hit_points` red. The three
/// land together. `docs/COMBAT.md` §43.
pub(crate) const GOLDEN_WORD_CHAPTER_TWO: i64 = 695;

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
/// walks — run100's first complete block to thirteen past the word. The
/// test reads its bounds from here and the guard reads the word against
/// them, so the two cannot disagree: a window the word has walked out of
/// fails `the_widening_behind_each_pinned_word_exists` rather than passing
/// by saying nothing (parked 449).
pub(crate) const WIDENING_GREAT_LAKES: (i64, i64) = (9_340, 10_830);
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
pub(crate) const WIDENING_CHAPTER_TWO: (i64, i64) = (606, 699);

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
    (
        "LONG_WORD_EAST_INDIES",
        LONG_WORD_EAST_INDIES,
        None,
        444,
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
    (
        "LONG_WORD_GREAT_LAKES",
        LONG_WORD_GREAT_LAKES,
        Some("run100_s_word_block_is_every_record_the_dump_carries"),
        494,
        Some(WIDENING_GREAT_LAKES),
    ),
    (
        "GOLDEN_WORD_CHAPTER_ONE",
        GOLDEN_WORD_CHAPTER_ONE,
        None,
        445,
        None,
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
    (
        "GOLDEN_WORD_CHAPTER_TWO",
        GOLDEN_WORD_CHAPTER_TWO,
        Some("chapter_two_s_word_frame_is_widened_whole"),
        491,
        Some(WIDENING_CHAPTER_TWO),
    ),
];
