pub(super) use bodies::*;

mod bodies {
    use super::super::*;

    pub(in super::super) fn run33_s_long_trace_says_where_the_word_parts() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(trace)) = (
            dump("gamelog-run33-longtrace.txt"),
            trace("rontrace-run33.log"),
        ) else {
            eprintln!("skipping: no run33 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let mut built = if crate::diff::testkit::whole_capture::legacy() {
            let texts = sibling_texts();
            let text = crate::capture::read(&path);
            let log = Log::parse(&text);
            let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
            let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
            let refs: Vec<&Initial> = inits.iter().collect();
            let mut init = log.initial().unwrap();
            borrow_from_siblings(&mut init, &refs);
            build_sim(&loaded, &init, Tuning::RON)
        } else {
            crate::diff::testkit::whole_capture::road_build(&loaded, &path)
        };

        built.sim.trace_phases = true;
        let last = trace.frames.last().map_or(0, |(n, _)| *n);
        assert!(
            last >= 1_800,
            "run33's traced length is {last}, wanted 1,800+"
        );
        for _ in 0..last {
            built.tick();
        }
        let mut matched = 0usize;
        let mut parted: Vec<String> = Vec::new();
        for (frame, ours) in &built.frame_sites {
            let theirs = trace.labels(*frame);
            if *ours == theirs {
                matched += 1;
                continue;
            }
            let at = (0..ours.len().max(theirs.len()))
                .find(|&i| ours.get(i) != theirs.get(i))
                .unwrap_or(0);
            parted.push(format!(
                "frame {frame}: ours {} theirs {} — at {at}, ours {:?} theirs {:?}",
                ours.len(),
                theirs.len(),
                ours.get(at),
                theirs.get(at),
            ));
        }
        let first_part = built
            .frame_sites
            .iter()
            .find(|(f, ours)| *ours != trace.labels(*f))
            .map(|(f, _)| *f)
            .unwrap_or(last);
        let first_count = built
            .frame_sites
            .iter()
            .find(|(f, ours)| ours.len() != trace.labels(*f).len())
            .map(|(f, _)| *f)
            .unwrap_or(last);
        let words = built
            .frame_sites
            .iter()
            .filter(|(f, ours)| ours.len() == trace.labels(*f).len())
            .count();
        eprintln!(
            "run33: word parts at {first_count}, sequence at {first_part}; \
             {words} of {last} frames spend the original's number of draws, \
             {matched} of them draw for draw"
        );
        for p in parted.iter().take(4) {
            eprintln!("{p}");
        }
        // The count divergence is the score, so print its own row too: it
        // is the successor item every time this number moves.
        if let Some(p) = parted
            .iter()
            .find(|p| p.starts_with(&format!("frame {first_count}:")))
        {
            eprintln!("count: {p}");
        }
        // **The word: 307.** run14's capture agreed to its own end at 284;
        // this one carries 1,566 frames more, and the simulation's
        // per-frame draw *count* is the original's for twenty-three of
        // them before it parts.
        //
        // What parts it is a **non-flat gather**. The original's frame 307
        // spends eleven draws and this simulation nine, and the two it
        // does not spend are one unit's: a stand issued from inside
        // `Unit::do_non_flat_gather+0x10f`, and `+0x54b`, the gather's own
        // roll — so on the original a gatherer is working a non-flat
        // resource on that frame and here it is not. The other nine are
        // the same on both sides and in the same order: an
        // `Animal::do_idle` roll, a phase-7 wrap, and seven farms.
        // **345 since item 68** (2026-08-29). The two draws frame 307 was
        // short were `1/7`'s, and `1/7` was not at its camp to spend them
        // because the AI's army had marched it away on 252: `Unit::think`'s
        // tail was joining any attacker to an army, where the original
        // joins only a supply wagon or a hero, and run33's own coverage
        // says `Unit::add_to_army@005f7740` is never entered in this game
        // at all. With the tail as the listing has it the word runs to
        // **345**.
        //
        // **What parts it at 345 is not the ninth citizen.** Item 68's
        // note guessed it was — `1/8`'s Woodcutter's Camp, the row that
        // pinned the headline three frames earlier — and item 70 fixed
        // exactly that and moved this number not at all. The row is an
        // animation draw against a farm's: on 345 this simulation spends
        // eight where the original spends seven, and the very first is
        // ours `Guy::set_anim < Guy::inc_time` against the original's
        // `Farms::inc_time+0x1ae`. It is the same row before and after
        // item 70, byte for byte. The **totals** are what moved.
        //
        // **361 with item 71** (2026-08-29), and what was wrong at 345 was
        // a **length**. The human scout's guy 0 rolled `CHAR_IDLE1` on
        // frame 284; no dump has ever shown that slot's length for its
        // piece, so `Art::lengths` had none and the roll fell back to
        // `CHAR_DEFAULT` — 61 frames instead of 76. The clock wrapped on
        // 345 where the original's runs to 360, and in the five frames
        // between, the original's *dog* — whose mirrored `cur_time` had
        // run past its own `end_time` — re-rolled once a frame from
        // `Unit::do_idle`, which this simulation never reached. Four
        // draws of drift by frame 349, and the farmer `0/4`'s two `% 4`
        // re-target draws came off the wrong words (item 71's own
        // symptom, the headline).
        //
        // The lengths now come from the install: `unit_graphics.xml` for
        // every `<UNIT>` entry, placed at the piece
        // `GraphicPieces::get_unit_gpiece` hands out
        // (`crate::artdata::piece_lengths`). That exposed a second one —
        // `unit_masks & 0x78000000`, the carrying walk, which this crate
        // had been reading off the gather order's `goto_build` instead.
        // With the carrying slots' lengths known the stand-in stopped
        // being invisible: `1/7`'s first walk to its camp became a
        // `WALK_WITH_WOOD` and lost the arrival stand the original spends
        // at frame 232. The mask is `do_non_flat_gather`'s own, and with
        // it the word runs to **361**.
        //
        // **432 with item 76**, and what was wrong at 361 was a **tile**.
        // The cell filter's surface probe (`docs/SCOUT.md` §7) was reading
        // `(4x, 4y + 2)` where the listing reads the cell **centre**: the
        // `+4` in `movb 0x4(%eax,%ecx,2)` is two `TData` elements, not a
        // field offset. Cell `(48, 23)` is ocean at its first tile and
        // land at its centre, so the AI scout's re-target refused a
        // candidate the original takes — twenty-six draws against
        // twenty-seven — and went to `(48, 20)` instead. What parts the
        // word at 432 is a gather stand: ours spends
        // `Guy::set_anim < Unit::do_non_flat_gather+0xb99` where the
        // original spends a farm's `Farms::inc_time+0x1ae`, twenty-three
        // draws against twenty-two.
        //
        // **482 with item 78**, and what was wrong at 432 was a **write
        // that went to the wrong order**. `Sim::store_gather` wrote the
        // gather order's fields back to `orders.front_mut()`, but every
        // walk `do_non_flat_gather` issues goes in *front* of the gather
        // (`QUEUE_FIRST`, no action bit) — so on the return-to-camp branch
        // the `goto_build = 1` and `wait = 32` that follow
        // `add_move_order` were written to the move and silently dropped.
        // The original holds a pointer to the order object for the whole
        // function, which is the same rule `is_gathering_at` already
        // needed on the *read* side (`docs/ORDERS.md` §6, §6.4).
        //
        // The human's woodcutter `0/2` finished its shift on frame 426,
        // set off for its camp on 427 and arrived on 431 — and then, with
        // `goto_build` still 0 and `wait` still −1, re-entered the same
        // branch on 432 and spent the stand again. It never unloaded, never
        // took another tile, and re-issued that walk every other frame for
        // the remaining 1,400 frames of the capture.
        //
        // **571 with item 79**, and what was wrong at 482 was an **angle**.
        // The scout re-targets on 482 and the original spends thirty-one
        // draws over its seven rings where this spent thirty: cell
        // `(56, 28)` was already seen here and not there, so the cell
        // filter refused a candidate the original scores
        // (`run33_s_scout_re_targets_at_482_on_the_original_s_ring`).
        //
        // It was seen here because of a reveal three hundred frames
        // earlier. `Object::update_seen` throws a small land unit's disc
        // half a cell forward of its nose, and the angle it projects along
        // is `UnitData +0x50` — the unit's own heading — not the guy's
        // eased facing; the `project` at `00651d05` is handed it by
        // `movl 0x50(%ecx), %ecx` two instructions earlier. This crate had
        // the guy's. On frame 168 the scout was mid-turn, the two angles
        // were −51.6° and −83.0°, and the disc thrown along the wrong one
        // lit a fog cell the original's never reached (`docs/VISION.md`
        // §3).
        //
        // What parts the word at 571 is a **collision**: the original
        // spends a draw at `5fa882`, inside
        // `Unit::resolve_unit_collision@005f9d30`, that this simulation
        // does not — eight draws against seven.
        //
        // **776 with item 81**, and what was wrong at 576 was a **price**.
        // The original's frame 576 spends sixty draws over five
        // `ScenarioFuncSet::place_city_with_cost` calls — the script's
        // `city_placement` under `defensive.bhs` step 11, once per turn of
        // its `num_loops = 5` loop — and this simulation spent forty,
        // because its *first* call bought the city and the guard at
        // `009f5898` (`city_limit <= total_cities`) then returned −1 four
        // times without a draw. The original could not pay: a second Small
        // City costs **sixty** food and sixty timber, not twenty-two, and
        // the AI held 69 and **59**.
        //
        // `TypeData::get_cost` has two ramps. The unit arm reads
        // `UNIT_COST_FACTOR` and all four `*_RAMP_MAX`; the building arm
        // (`00665787`..`00665b5a`) reads `BUILD_COST_FACTOR` and
        // `BUILD_SUPPORT_FACTOR` and **no ceiling at all**. This crate gave
        // every building `RampClass::default()` — the military 125% — which
        // clamped the city's `SUPPORT 50` ramp to 12 and priced it at 22.
        // run40 and run41 measure both ends of it
        // (`run40_s_census_prices_the_ai_s_second_city_at_sixty`).
        //
        // What parted the word at 776 was that same block a second time:
        // the original spent five draws there and this simulation ten,
        // because on 776 it was the *original* that bought on its first
        // call and this simulation that could not — its AI was thirty-two
        // food short of the original's on every frame from 202 on. Item 74
        // is those thirty-two: twenty from the farm the AI finishes on
        // frame 166 (`Build::do_bonus`, `docs/ECONOMY.md`) and twelve from
        // the City State waiting in its library when Written Word lands on
        // 201 (`Build::refund_cost`, `docs/COSTS.md`). With them the city
        // is bought on 776 here too, and **the word parts at 780**.
        //
        // **986 with item 84**, and what was wrong at 780 was a **byte
        // that was not the guy's**. `do_gather`'s farm switch tests
        // `guy[0].cur_anim` — `*(char *)(**(int **)&this->field_0xf4 +
        // 0x9c)`, `GuyData +0x9c` — and this crate kept a `farm_anim`
        // flag written only by the farm branch and cleared only by a
        // step. The AI's `1/4` stands on farm `b10`'s cell 7, reaps it
        // until `inc_time` decays it empty on 777, re-picks a tile on 778
        // and is **blocked** on 779: the blocked stand
        // (`Unit::move_step+0x823`) replaces the guy's reap animation
        // without moving the body, so on 780 the original sows the cell
        // and this crate re-picked again. Two draws it does not spend, a
        // seventh `Farms::inc_time` from 781 — the farm keeps five empty
        // cells here and drops to four there — and what parts the word at
        // 986 is a blocked stand of its own, seven draws against six.
        //
        // **1372 with item 112**, and what was wrong at 986 was a **herd's
        // wander centre, which does not walk**. `Herd::process@00741760`
        // writes `wx = cx − 1 + p % 3` and `wy = cy − 1 + p % 3` — it
        // reads `HerdData +0x0/+0x4` and stores into `+0x8/+0xc`, so the
        // jitter is about the **home** cell every time. This crate read
        // the destination as the source and random-walked it, which is the
        // same thing until a herd's *second* walk; `(frame >> 6) % 13`
        // gives herd 0 its second on frame 832 and no other herd one at
        // all inside the capture. Its `wy` ended 34 here against the
        // original's 33, and the far wander's ring is drawn about
        // `herd_centre` (`docs/ANIM.md` §7): on 981 the sheep `8/3` was
        // sent to `(17688, 26616)` where the original sends it to
        // `(17688, 26328)`, five steps of `(+15, −5)` against five of
        // `(+12, −10)`, so the same neighbour refused it a frame early and
        // the blocked stand fell on 986 instead of 987. Pinned step for
        // step by `run33_s_herd_centre_jitters_about_its_home_cell`.
        //
        // **1802 with item 114**, and what was wrong at 1372 was a
        // **pasture nothing stocked**. `Build::activate`'s gather tail
        // reaches `Farms::add_animals` for every *food* gather building it
        // completes — `LAB_00625a36` is the fall-through from
        // `do_bonus(0, FOOD_BONUS_FOR_FARM)` and the `iVar18 == 0` arm of
        // `switchD_006259b5_caseD_2` is where every unpaid exit lands, and
        // both are the same call — so the AI's fourth farm, which
        // `Farms::add` had already made a pasture, costs twenty draws on
        // the frame it finishes: a species coin, a `y` and an `x` per
        // animal (`+0x92`, `+0x134`, `+0x182`) and `Guy::init_real`'s
        // variant inside `Objects::init_unit`. Five more follow in the
        // same frame's `Objects::inc_time`, because a newborn guy's
        // `end_time` is zero and its clock wraps at once. Twenty-five
        // draws where this crate spent none, and `Farms::inc_time`'s three
        // were all that was left. The harness had the building and the
        // type right — its `b15` activates on 1372 with `farm_type 1` —
        // and only the stocking was missing (`docs/SYNC.md` §3.11).
        //
        // The parting frame goes 1372 -> **1802**, which is past run10's
        // whole 1,772: this map now spends the original's draws in the
        // original's order for longer than its own capture runs. What
        // parts it at 1802 is one draw at
        // `Unit::do_air_physics+0x639 < Unit::do_air_patrol+0xf3`, a gaia
        // bird's flight physics, and it is the **only** time that site is
        // reached in all 1,851 frames.
        assert!(
            first_count >= FLOORS[1].word,
            "the word parts at frame {first_count}; the floor is {}\n{}",
            FLOORS[1].word,
            parted.first().cloned().unwrap_or_default()
        );
        // **The sequence: 576**, and getting there was the whole of the
        // ~~99~~ attribution swap run14's capture had always shown — the
        // same address under a different caller, ours `Unit::do_idle+0x7d`
        // where the original has `Guy::inc_time+0x271`, with the draw count
        // equal either side (queue item 62). It was not an attribution
        // question at all: the trained citizen is created in
        // `Objects::process_all`'s **second** loop, after every unit, so
        // the original never reaches it in the unit loop on its birth
        // frame and `Objects::inc_time` wraps its `end_time 0` clock
        // instead. `docs/SYNC.md` §3.16. It is a floor here so that a
        // regression that moved it would read as itself.
        // **576 -> 780 with item 100**, and the two numbers meet: naming
        // `make_stuff`'s expiry walk (`+0x221`) is what moved it, because
        // an unnamed draw takes the last mark's name and five frames of
        // `place_city_with_cost` at 576 read as `compute_sites+0x50a`.
        // Nothing about the simulation's arithmetic changed there; what
        // changed is that the comparison stopped lying about it.
        // **780 -> 986 with item 84**, the two still meeting: the farm
        // stand's byte is the guy's live `cur_anim`, and nothing between
        // 780 and 986 is a naming question.
        // **986 -> 1372 with item 112**, and the two meet a third time:
        // the herd's wander centre is jitter about the home cell, and
        // nothing between 986 and 1372 is a naming question either.
        // **1372 -> 1802 with item 114**, a fourth time: the pasture's
        // twenty draws carry names of their own from the frame they are
        // first spent on, and nothing between 1372 and 1802 is a naming
        // question. The two numbers have now been equal for four items
        // running, which is what it looks like when every hole left is a
        // mechanic rather than a label.
        assert!(
            first_part >= FLOORS[1].word,
            "the draw sequence parts at frame {first_part}; the floor is {}",
            FLOORS[1].word
        );
        // And the totals over the whole 1,850, which is what says whether a
        // change past the divergence helped or only moved the noise: 618
        // frames spend the original's number of draws and 460 of them are
        // its draws in its order. Past the part frame both sides are off
        // streams of their own, so these are weak numbers — but a fall in
        // them is worth reading.
        //
        // 668 / 556 → **618 / 460** with item 68, the only time either has
        // fallen while the score rose. Every frame before 345 matches on
        // both counts, so the whole of the fall is past the divergence: the
        // stream this simulation is on after `1/8` takes the wrong job is a
        // *different* wrong stream from the one it was on after `1/7` was
        // marched off, and it happens to coincide with the original's less
        // often. The number that is not luck is `first_count`, 307 → 345.
        //
        // 618 / 460 → **635 / 488** with item 70, and this is the useful
        // half of that item on this capture: `first_count` did not move,
        // but seventeen more frames spend the original's number of draws
        // and twenty-eight more spend them in its order, because the AI's
        // citizens are at the buildings the original has them at for the
        // rest of the run.
        //
        // 635 / 488 → **696 / 523** with item 71, the largest move either
        // has made, and the two rose together with `first_count`.
        //
        // 696 / 523 → **724 / 606** with item 76, and `matched` moved most:
        // eighty-three more frames are the original's draws in the
        // original's order, which is what a scout sent to the original's
        // cell buys downstream.
        //
        // 724 / 606 → **752 / 622** with item 78.
        //
        // 752 / 622 → **791 / 662** with item 79, and the two rose with
        // `first_count` again — thirty-nine more frames on the original's
        // word, forty more of them draw for draw, because the AI's scout
        // now walks the original's fog as well as its ground.
        //
        // 791 / 662 → 802 / 688 with item 80, and → **944 / 828** with item
        // 81 — the largest move either has made, and both rose with
        // `first_count`: a hundred and forty-two more frames spend the
        // original's number of draws because the AI's whole economy is two
        // hundred frames closer to the original's from 576 on.
        //
        // 944 / 828 → **951 / 838** with item 74, and the three rose
        // together: the AI's second city is founded on the original's
        // frame, so its ninth citizen is trained on the original's frame
        // too and the roster is one-sided again at 268 + 0.
        //
        // 951 / 838 → **954 / 843** with items 36 and 93 together. This is
        // the number item 93 was held back for: the arm alone, before the
        // farmers' angles were the original's, cost `first_count` 780 →
        // 584 and run10's orders 776 → 586. With item 36 in front of it
        // the word holds at 780 and both totals rise
        // (`docs/SYNC.md` §3.11, §3.12).
        //
        // 954 / 843 -> **938 / 841** with item 95, the animal's hurry, and
        // this is the second time either has fallen while the score rose
        // (item 68 was the first). The fall is entirely noise and it can
        // be said exactly where it starts: with the hurry in and out, this
        // capture's per-frame draw counts are **identical up to frame
        // 1108** — 328 frames past the word's own parting at 780, and past
        // every one of the twelve units' first divergence. `first_count`
        // holds at 780 and `first_part` at 99; what moved is which of two
        // wrong streams the simulation is on after 1108. The number that
        // is not luck is the *other* map's, where the same change takes
        // the word 69 -> 91.
        //
        // 938 / 841 -> **943 / 827** with the blocked animal's dropped
        // walk, and the same argument holds a second time with the same
        // shape: with that rule in and out, this capture's per-frame draws
        // are **identical — the count and the sequence both — up to frame
        // 1128**, 348 frames past the word's own parting at 780 and past
        // eleven of the twelve units' first divergence. `first_count`
        // holds at 780, `first_part` at 99, run10's ticks and orders at
        // 572/776, and the other map's word goes 91 -> 201
        // (`docs/SYNC.md` §3.14).
        //
        // 943 / 827 -> **943 / 830** with the frame's two loops (item 60),
        // and what moved with them is the *sequence*: 99 -> 576. The word
        // holds at 780, run10's ticks and orders at 572/776, and the other
        // map's word goes 219 -> 274 (`docs/SYNC.md` §3.16).
        //
        // 943 / 830 -> **964 / 864** with item 100, and again it is the
        // *sequence* that moves: **576 -> 780**, so this map's two numbers
        // are now the same frame. Two things landed together and only one
        // of them is a mechanic. `make_stuff`'s expiry walk had always
        // spent its draws and never **named** them, so every one read as
        // whatever site marked last — here `compute_sites+0x50a`, five
        // frames of it at 576 — and a hole that was only a missing label
        // sat in front of the holes that are real. The mechanic is the
        // bird's landing search (`docs/SYNC.md` §3.9), sixty draws a
        // landing, which is what takes the *other* map's word 576 -> 645.
        //
        // 964 / 864 -> **977 / 866** with item 101, the chopping guy's own
        // wait: both numbers rise while the word and the sequence hold at
        // 780, because a woodcutter that stays at its tile three times as
        // long is at the original's tile on hundreds of the frames past
        // the parting. The other map's word goes 645 -> 742.
        //
        // 977 / 866 -> **986 / 884** with item 102, the far wander's
        // literal bearing (`docs/SYNC.md` §3.19): this map's herd takes
        // the same branch East Indies' does, so nine more frames spend the
        // original's number of draws and eighteen more are draw for draw,
        // while the word and the sequence hold at 780. The other map's
        // word goes 742 -> 867.
        //
        // 986 / 884 -> **986 / 892** with item 106, the scout's walk to a
        // goody box (`docs/GOODY.md` §7): Great Lakes has 22 boxes, and
        // eight more frames past the parting come out draw for draw while
        // the word and the sequence hold at 780. The other map's word goes
        // 879 -> 1256.
        //
        // 986 / 892 -> **959 / 876** with item 109, the bird's landing
        // step (`docs/SYNC.md` §3.9) — **the first fall these two numbers
        // have taken**, and it is the noise this comment's own opening
        // paragraph names. Every frame whose verdict changed is **1209 or
        // later**, 429 past the parting: 31 gained, 47 lost. What moved
        // them is that a bird's counter is one higher after each landing,
        // so this map's landings go 904, 936, 1080, 1136, 1152, 1288,
        // 1416, 1520, 1728, 1784 to 904, 936, 1080, 1136, 1152, **1184,
        // 1232, 1456, 1640**, 1784 — a stream that is nobody's rolling a
        // different number, on frames where the original's own landings
        // (816, 904, 1024, 1040, 1144, 1208, 1344, 1448, 1456, 1592,
        // 1744, 1816, 1832) it now meets **two** of rather than one. The
        // word and the sequence hold at 780, East Indies' word goes
        // 1256 -> 1373, and run39's landings up to its word are the
        // original's exactly
        // ([`a_bird_s_landing_frames_are_the_trace_s_own`]).
        //
        // 959 / 876 -> **953 / 877** with item 69's collision pause
        // (`docs/ORDERS.md` §4.4, `goto STEP`), the second fall and the
        // same kind of noise: the word and the sequence hold at **780**,
        // and every frame whose verdict changed is past it, on a stream
        // that is nobody's. What the item did move is the score — this
        // map's ticks 572 -> 781 and East Indies' 167 -> 1374 — which is
        // the number these two are instruments beside.
        //
        // 953 / 877 -> **1182 / 1114** with item 84, the largest move
        // either has ever made and the first since item 81 that is not
        // noise: the word itself goes 780 -> 986, so two hundred of those
        // frames are frames both sides genuinely agree on rather than
        // coincidences past a parting.
        //
        // 1182 / 1114 -> **1471 / 1439** with item 112, larger again and
        // not noise either: the word goes 986 -> 1372, so three hundred
        // more of these are frames both sides genuinely agree on. The gap
        // between the two also narrows — 68 frames spend the original's
        // number of draws in some other order, against 68 before — which
        // is what a stretch of real agreement rather than coincidence
        // looks like.
        //
        // 1471 / 1439 -> **1467 / 1436** with item 113, the second fall
        // these two have taken, and the argument is exact rather than
        // statistical this time: **the word and the sequence both hold at
        // 1372**, and `first_count` is by construction the first frame
        // whose draw *count* differs, so every frame before 1372 spends
        // the original's number of draws in the original's order on both
        // sides of the change. Every one of the seven verdicts that moved
        // is at 1372 or later, past the parting, on the two wrong streams
        // this comment's opening paragraph names. What the item moved is
        // the headline: this map's ticks go 910 -> **1375** and its orders
        // 776 -> **791**, the AI's `1/1` stops parting at all, and East
        // Indies holds at 1374/1373.
        //
        // 1467 / 1436 -> **1832 / 1830** with item 114, the largest move
        // either has ever made and the last one that can be large: 1,832
        // of the 1,850 frames spend the original's number of draws and
        // 1,830 of them draw for draw, so what is left is eighteen frames
        // and all of them past 1802. The gap between the two is down to
        // **two**, from 31 — a stretch of real agreement rather than
        // coincidence has no room for a frame that spends the right
        // number of draws in the wrong order.
        assert!(
            words >= 1832 && matched >= 1830,
            "the trace floor fell: {words} frames on the original's word, \
             {matched} draw for draw; the floors are 1832 and 1830"
        );
    }

    pub(in super::super) fn run53_s_24000_frames_put_the_ceiling_where_run33_did() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(trace)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            trace("rontrace-run53.log"),
        ) else {
            eprintln!("skipping: no run53 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        let last = trace.frames.last().map_or(0, |(n, _)| *n);
        // The whole point of this capture is its length; a short one would
        // pass every assertion below and mean nothing.
        assert!(
            last >= 23_000,
            "run53's traced length is {last}, wanted 23,000+ — this is the \
             long capture, and a short file here is a wrong file"
        );
        // `RON_DEBUG_UNIT` and the attributed fold, on the long capture:
        // `Built::tick` drops the marks' unit attribution when it folds
        // them into `frame_sites`, and item 204's whole question is *which*
        // unit spends the extra draw. Both cost nothing when unset.
        let unit_window = site_window();
        // **The second squad's membership** (items 249 and 250): 6994's
        // two draws are `1/31`, `1/32` and `1/33`'s, born on 6993 and
        // given an `ATTACK_TO` apiece by the frame's end
        // (`run84_says_great_lakes_6994_belongs_to_the_second_squad`).
        // This is what said the gap was an order rather than a membership
        // bug — all six in **one army** by the frame's end — and it stays
        // as the cheap guard on that half now that
        // `great_lakes_6994_issues_the_second_squad_s_walk_to_the_army`
        // pins the order itself.
        let mut squad_army: Vec<(i64, Option<usize>)> = Vec::new();
        for _ in 0..last {
            let f = built.sim.frame;
            built.tick();
            debug_watch(&built, f);
            debug_builds(&built, f);
            debug_armies(&built, f);
            debug_leader(&built, f);
            if f == GREAT_LAKES_SECOND_SQUAD {
                squad_army = [27, 28, 29, 31, 32, 33]
                    .into_iter()
                    .map(|o| {
                        let at = built.sim.units.iter().position(|u| {
                            u.alive() && i64::from(u.owner) == 1 && i64::from(u.index) == o
                        });
                        (o, at.and_then(|i| built.sim.army_of(i)))
                    })
                    .collect();
            }
            if unit_window.is_some_and(|(lo, hi)| (lo..=hi).contains(&f)) {
                for (label, who) in attributed_sites(&built) {
                    eprintln!("  f{f} {who}: {label}");
                }
                // **And the original's, beside them.** The window used to
                // print this crate's draws alone, which answers "what did
                // we spend" and never "what is the delta" — and the delta
                // is the whole question a one-draw parting asks. Item 432
                // rebuilt exactly this by hand as a scratch test before
                // noticing the window was already here; the trace is in
                // scope, so it costs four lines.
                let theirs = trace.labels(f);
                if theirs != built.frame_sites.last().map_or(&[][..], |(_, v)| v) {
                    eprintln!("  f{f} PARTS: theirs {theirs:?}");
                }
            }
        }
        assert_eq!(squad_army.len(), 6, "the parting frame was never reached");
        assert!(
            squad_army
                .iter()
                .all(|&(_, a)| a.is_some() && a == squad_army[0].1),
            "the six Great Lakes archers are not one army at              {GREAT_LAKES_SECOND_SQUAD}: {squad_army:?}"
        );
        let first_part = built
            .frame_sites
            .iter()
            .find(|(f, ours)| *ours != trace.labels(*f))
            .map(|(f, _)| *f)
            .unwrap_or(last);
        let first_count = built
            .frame_sites
            .iter()
            .find(|(f, ours)| ours.len() != trace.labels(*f).len())
            .map(|(f, _)| *f)
            .unwrap_or(last);
        let words = built
            .frame_sites
            .iter()
            .filter(|(f, ours)| ours.len() == trace.labels(*f).len())
            .count();
        let matched = built
            .frame_sites
            .iter()
            .filter(|(f, ours)| *ours == trace.labels(*f))
            .count();
        eprintln!(
            "run53: word parts at {first_count}, sequence at {first_part} of {last}; \
             {words} frames on the original's count, {matched} draw for draw \
             (both mostly past the parting — printed, not pinned)"
        );
        // The two parting frames, named — run54 has carried this since
        // East Indies became the second map, and Great Lakes is the
        // headline one. `RON_DEBUG_SITES=<lo>-<hi>` widens it to a window
        // and `RON_DEBUG_SITES=1` prints both sequences whole.
        let window = site_window();
        for (f, ours) in built.frame_sites.iter() {
            let named = *f == first_count || *f == first_part;
            if !named && !window.is_some_and(|(lo, hi)| (lo..=hi).contains(f)) {
                continue;
            }
            let theirs = trace.labels(*f);
            let at = (0..ours.len().max(theirs.len()))
                .find(|&i| ours.get(i) != theirs.get(i))
                .unwrap_or(0);
            eprintln!(
                "  frame {f}: ours {} theirs {} — at {at}, ours {:?} theirs {:?}",
                ours.len(),
                theirs.len(),
                ours.get(at),
                theirs.get(at)
            );
            if std::env::var("RON_DEBUG_SITES").is_ok() {
                for (i, l) in ours.iter().enumerate() {
                    eprintln!("    ours  {i}: {l}");
                }
                for (i, l) in theirs.iter().enumerate() {
                    eprintln!("    thrs  {i}: {l}");
                }
            }
        }
        // **The army's release tick, entry for entry** (item 317,
        // `docs/ARMY.md` §16.8). 7930 is the first `Army::find_target`
        // draw in the whole game and the frame the AI's first army leaves
        // the muster: §6's dispatch `if`s re-read `status`, so
        // `do_mustering`'s `status = 2` reaches `do_marching` in the same
        // tick holding `target_o = -1`, and the original's `iVar4 < 0`
        // arm goes straight to `find_target`. This crate spent nothing
        // here until the arm was modelled. The head is the whole claim —
        // the difficulty gate's coin once, then the per-candidate score
        // twice — and the equality behind it is what says the rest of the
        // frame did not move to pay for it.
        let ours_7930 = built
            .frame_sites
            .iter()
            .find(|(f, _)| *f == 7930)
            .map(|(_, s)| s.clone())
            .unwrap_or_default();
        assert_eq!(
            ours_7930,
            trace.labels(7930),
            "Great Lakes 7930 is the army's release tick and must agree \
             entry for entry"
        );
        assert_eq!(
            ours_7930.iter().take(3).collect::<Vec<_>>(),
            vec![
                sim::army::SITE_FIND_TARGET_COIN,
                sim::army::SITE_FIND_TARGET_SCORE,
                sim::army::SITE_FIND_TARGET_SCORE,
            ],
            "7930 opens with find_target's coin and two candidate scores"
        );
        // **Great Lakes 9451 is two arrows on one farm** (item 394,
        // `docs/COMBAT.md` §20). The original spends
        // `Object::take_damage+0xe1` **twice** on this one frame, back to
        // back, with the identical chain — which can only happen if the
        // first-wound gate reads `damage` and ignores `damage_frac`, because
        // a Longbowman figure's hit is thirteen sixteenths and the first of
        // the two necessarily moved the fraction and nothing else.
        //
        // This crate lands the second arrow on 9452 rather than 9451 (§20.2,
        // the launch seam), so the count is taken over the three frames
        // rather than the one: **two first wounds**, whenever they land.
        // Adding `damage_frac == 0` to the gate makes it one — and makes
        // 9452 agree eight-for-eight with the original, which is exactly the
        // trap item 89(c) names: a frame past the word looking better on a
        // stream that is nobody's.
        let wound = sim::fight::SITE_FIRST_WOUND;
        assert_eq!(
            trace.labels(9451).iter().filter(|l| *l == wound).count(),
            2,
            "the original spends the first-wound roll twice on 9451"
        );
        let ours_wounds: usize = built
            .frame_sites
            .iter()
            .filter(|(f, _)| (9451..=9453).contains(f))
            .map(|(_, s)| s.iter().filter(|l| *l == wound).count())
            .sum();
        assert_eq!(
            ours_wounds, 2,
            "two arrows worth thirteen sixteenths apiece both find the \
             first-wound gate open, because it reads `damage` and the first \
             of them leaves `damage` at zero"
        );
        // **The three frames a shot hits nothing** (item 483,
        // `docs/COMBAT.md` §39). `Ammo::do_damage`'s puncture pair is the
        // only draw in the whole 24,000-frame game that neither side had a
        // name for: the original's read `678cb9`/`678cde` and this crate's
        // the tick's bare `projectiles` phase mark, so the *sequence* word
        // could not pass 10237 however the simulation behaved while the
        // *count* word had already reached 10244.
        //
        // Both sides take it on exactly these three frames and nowhere
        // else, which is what makes the pair a pin rather than a label:
        // the two frames below the word are asserted whole, and the third
        // is past it and only counted.
        let punct = [
            sim::fight::SITE_PUNCTURE_X.to_string(),
            sim::fight::SITE_PUNCTURE_Y.to_string(),
        ];
        let theirs_punct: Vec<i64> = (0..=24_000)
            .filter(|f| trace.labels(*f).iter().any(|l| punct.contains(l)))
            .collect();
        assert_eq!(
            theirs_punct,
            vec![10_237, 10_242, 10_249],
            "the original punctures ground on three frames of 24,000"
        );
        let ours_punct: Vec<i64> = built
            .frame_sites
            .iter()
            .filter(|(_, s)| s.iter().any(|l| punct.contains(l)))
            .map(|(f, _)| *f)
            .collect();
        assert_eq!(ours_punct, theirs_punct, "and this crate on the same three");
        for f in [10_237, 10_242] {
            let ours = built
                .frame_sites
                .iter()
                .find(|(g, _)| *g == f)
                .map(|(_, s)| s.clone())
                .unwrap_or_default();
            assert_eq!(
                ours.iter().rev().take(2).rev().cloned().collect::<Vec<_>>(),
                punct.to_vec(),
                "{f} ends on the puncture pair"
            );
            assert_eq!(ours, trace.labels(f), "{f}, draw for draw");
        }

        // **Great Lakes 8272, the game's first scholar** (item 338,
        // `docs/CITIES.md` §6.5.2). The frame is 38 draws on both sides —
        // the count never parted here — and the only entry that moved was
        // index 33, where `Unit::go_inside`'s forced `CHAR_DEFAULT` sits
        // between the birth at 32 and the `inc_time` wraps. The whole
        // frame is the assertion, because "one draw replaced one draw"
        // is exactly the claim a count cannot make.
        let ours_8272 = built
            .frame_sites
            .iter()
            .find(|(f, _)| *f == 8272)
            .map(|(_, s)| s.clone())
            .unwrap_or_default();
        assert_eq!(
            ours_8272,
            trace.labels(8272),
            "Great Lakes 8272 is the first scholar's seating and must \
             agree entry for entry"
        );
        assert_eq!(
            (ours_8272.get(32), ours_8272.get(33)),
            (
                Some(&sim::anim::SITE_INIT_REAL.to_string()),
                Some(&sim::anim::SITE_GO_INSIDE.to_string())
            ),
            "8272's 33rd and 34th draws are the birth and the seating"
        );
        // **And the seating's whole schedule**, which is what says the
        // gate is `is_scholar` and not something that happens to hold on
        // one frame: `report.py … when` dates the original's at exactly
        // these fourteen frames in 24,000, and every earlier birth —
        // 6612, 6993, 7212, 7439 and 7674, three units apiece — spends
        // none. Only the frames below the word are asserted; past it both
        // streams are nobody's (item 89(c)).
        let seatings: Vec<i64> = built
            .frame_sites
            .iter()
            .filter(|(_, s)| s.iter().any(|l| l == sim::anim::SITE_GO_INSIDE))
            .map(|(f, _)| *f)
            .filter(|f| *f < LONG_WORD_GREAT_LAKES)
            .collect();
        // **Against the original's own list, not a literal.** Item 352's
        // standing trio taught the shape: a literal here is
        // window-length arithmetic and breaks the moment the word moves
        // — 8680's scholar came under the word on item 354 and this
        // assertion, not the mechanic, is what failed. The trace's own
        // seatings below the word are the right right-hand side, and the
        // literal stays beside it so a reader sees real frames.
        //
        // **Five → ten on item 442**, and the shape is why the list is
        // written this way: the comparison against the original's own
        // seatings **passed unchanged**, and only the literal moved. The
        // sixth scholar is 9510 — the frame that was the word for six
        // items (§47) — and four more follow it below the new one. Every
        // one of them is on the original's own frame, which is a stronger
        // statement than the count: the scholar arm now buys when the
        // original buys, not merely as often. `docs/AI.md` §53.
        let theirs_seatings: Vec<i64> = (0..LONG_WORD_GREAT_LAKES)
            .filter(|f| {
                trace
                    .labels(*f)
                    .iter()
                    .any(|l| l == sim::anim::SITE_GO_INSIDE)
            })
            .collect();
        assert_eq!(
            seatings, theirs_seatings,
            "Great Lakes' scholar seatings below the word are not the \
             original's own"
        );
        //
        // **Ten → eleven on item 497**, and the shape held a second
        // time: the comparison against the original's own seatings
        // passed unchanged and only the literal moved. The eleventh is
        // **10306**, which the word walked past when the scholar teach
        // slot's tie-break came right (`docs/ANIM.md` §4.12) — a
        // seating this crate was already spending on the original's own
        // frame, 279 frames below the word rather than above it.
        //
        // **Eleven → thirteen on item 678**, the same shape a third time:
        // the comparison against the original's passed unchanged. The word
        // walked past **13555** and **13736** when a resumed 48-grid search
        // came to read the blocks it had copied (`docs/PATHFINDER.md` §26).
        //
        // **Thirteen → fourteen on item 785**, a fourth time: the word
        // walked past **17570** when an easy AI's wonder arm came to read
        // `wonder_mark` (`docs/AI.md` §75), and the comparison against the
        // original's passed unchanged.
        assert_eq!(
            seatings,
            vec![
                8272, 8680, 9_087, 9_201, 9_322, 9_510, 9_717, 9_861, 10_012, 10_140, 10_306,
                13_555, 13_736, 17_570
            ],
            "below the word Great Lakes seats fourteen scholars, on 8272, 8680, \
             9087, 9201, 9322, 9510, 9717, 9861, 10012, 10140, 10306, 13555, 13736 \
             and 17570"
        );
        // **Great Lakes 9134, the snap arm's blocked stand** (item 360,
        // `docs/COLLISION.md` §5.4). The frame is **one draw on each
        // side** and the count never parted here: the original's was a
        // bare `5dac7a`, `Guy::set_anim+0x97a` under a twelfth `ebp`
        // chain the table did not name, and this crate spent the eleventh
        // — `Unit::move_step+0x823`, §5's partial-step block — where the
        // original spends `+0x4e2`, the snap's. One draw replacing one
        // draw is exactly the claim a count cannot make, so the frame is
        // the assertion and the label beside it.
        let ours_9134 = built
            .frame_sites
            .iter()
            .find(|(f, _)| *f == 9134)
            .map(|(_, s)| s.clone())
            .unwrap_or_default();
        assert_eq!(
            ours_9134,
            trace.labels(9134),
            "Great Lakes 9134 is `move_step`'s snap arm and must agree \
             entry for entry"
        );
        assert_eq!(
            ours_9134.as_slice(),
            [sim::anim::SITE_SNAP_BLOCKED.to_string()],
            "9134 is one draw, and it is the snap's stand and not §5's"
        );
        // **Great Lakes 8582, the AI's first market draw** (item 348,
        // `docs/AI.md` §40). `make_stuff` calls `use_market` first thing,
        // so the draw sits at index 0 and the frame is otherwise the two
        // expiry walks it always was — which is the claim a count cannot
        // make, so the whole frame is the assertion.
        let ours_8582 = built
            .frame_sites
            .iter()
            .find(|(f, _)| *f == 8582)
            .map(|(_, s)| s.clone())
            .unwrap_or_default();
        assert_eq!(
            ours_8582,
            trace.labels(8582),
            "Great Lakes 8582 is the market's first draw and must agree \
             entry for entry"
        );
        assert_eq!(
            ours_8582.first(),
            Some(&sim::ai_make::SITE_MARKET_SELL.to_string()),
            "8582 opens with `use_market`'s sell rotation"
        );
        // **And the whole schedule below the word**, which is what says
        // the gate is Coinage and not something that happens to hold on
        // one frame: `report.py … when Leader::use_market` dates the
        // original's first two at 8582 and 8585 — two leaders three
        // frames apart — and **none** earlier in the 24,000. This crate's
        // own shortfall vector met the sell branch's condition on 8185
        // and 8382 as well, and the tech is the only thing that refuses
        // them. Past the word both streams are nobody's (item 89(c)).
        let markets: Vec<i64> = built
            .frame_sites
            .iter()
            .filter(|(_, s)| s.iter().any(|l| l == sim::ai_make::SITE_MARKET_SELL))
            .map(|(f, _)| *f)
            .filter(|f| *f < LONG_WORD_GREAT_LAKES)
            .collect();
        // The original's own list below the word, for the same reason as
        // the seatings above: 8782 and 8982 came under the word on item
        // 354 and a literal cannot follow it.
        let theirs_markets: Vec<i64> = (0..LONG_WORD_GREAT_LAKES)
            .filter(|f| {
                trace
                    .labels(*f)
                    .iter()
                    .any(|l| l == sim::ai_make::SITE_MARKET_SELL)
            })
            .collect();
        assert_eq!(
            markets, theirs_markets,
            "Great Lakes' market draws below the word are not the \
             original's own"
        );
        // **Six → nine on item 442**, and **nine → ten on item 456**. The
        // comparison above passed unchanged both times; only the literal
        // moved, because the word went 9510 → 10161 → 10232 and one more
        // of the 200-frame rotations came under it each time. That is
        // `ORDER_RESIDUE_RUN97`'s lesson in its cheapest form — a list
        // bounded by the headline grows when the headline does — and the
        // assertion above it, which compares the two *streams*, is the
        // one with the teeth. **9582 closes `docs/AI.md` §50.4's third
        // open question** — it was the parting that probe's 9582 left, one
        // `Leader::use_market+0x1ed` short, and it is a market sell on
        // both sides now.
        //
        // **Ten → eleven on item 497**, the same shape a third time and
        // the largest single step of it: the word went 10303 → 10582 and
        // **10382** came under. The comparison above passed unchanged
        // again. The rotation's next frame after it is **10582** itself,
        // which is the word — the original takes `Leader::use_market
        // +0x1ed` there and this crate `Leader::make_stuff+0x221`, so
        // the sequence parts on a market frame and the count parts one
        // frame later on `Leader::create_units+0x642`.
        //
        // **Eleven → thirteen on item 506**, and this one is not only the
        // literal following the headline: **10582 itself** is now a
        // `use_market` sell on both sides, which is the frame the
        // sequence used to part on. The AI enters it with no wealth
        // because `market_speculation` spent the purse on food five
        // frames earlier (`docs/ECONOMY.md` §13), and 10782's rotation
        // came under the word with it. The comparison above — the two
        // *streams*, entry for entry — passed unchanged again.
        //
        // **Thirteen → fourteen on item 518**, the literal following the
        // headline once more: the word went 10834 → 11185 and **10982**'s
        // rotation came under it. The streams above agree through it.
        //
        // **Fourteen → sixteen on item 327**, and **11185 is the old
        // word itself**: the Merchant offer empties slot 1 on both sides,
        // so `use_market`'s need is the original's and the frame is a
        // sell on both. The word went 11185 → 11531 and 11382's rotation
        // came under it too (`docs/AI.md` §55).
        //
        // **Sixteen → seventeen on item 545**, the literal following the
        // headline: the word went 11582 → 11757 and **11582**, the old word,
        // came under it — a `use_market` sell on both sides, the frame's
        // first draw either way (`docs/AI.md` §56).
        //
        // **Seventeen → eighteen on item 557**, the literal following the
        // headline: the word went 11757 → 11806 and **11782**'s rotation
        // came under it (`docs/GROUPS.md` §23). The streams above agree
        // through it.
        //
        // **Eighteen → nineteen on item 566**, the literal following the
        // headline: the word went 11903 → 12038 and **11982**'s rotation
        // came under it (`docs/PATHFINDER.md` §24.6).
        //
        // **Nineteen → twenty on item 657**, the literal following the
        // headline: the word went 12135 → 12184 and **12182**'s rotation
        // came under it (`docs/ARMY.md` §20). The streams above agree
        // through it.
        //
        // **Twenty → twenty-one on item 661**, the literal following the
        // headline: the word went 12184 → 12429 and **12385**'s rotation
        // came under it (`docs/AI.md` §63). The streams above agree
        // through it.
        //
        // **Twenty-one → twenty-three on item 673**, the literal following
        // the headline: the word went 12536 → 12897 and **12582** and
        // **12782**'s rotations came under it (`docs/PATHFINDER.md`
        // §21.6). The streams above agree through them.
        //
        // **Twenty-three → twenty-nine on item 678**, the literal following
        // the headline: the word went 12897 → 14382 and **12982**,
        // **13382**, **13385**, **13582**, **13782** and **13982** came
        // under it (`docs/PATHFINDER.md` §26). The streams above agree
        // through them.
        //
        // **Twenty-nine → thirty on item 688**: the word went 14382 →
        // 14529 and **14385** came under it (`docs/AI.md` §67). The
        // streams above agree through it.
        //
        // **Thirty → thirty-two on item 695**: the word went 14529 →
        // 14650 and **14582** and **14585** came under it (`docs/AI.md`
        // §68). The streams above agree through them.
        //
        // **Thirty-two → thirty-three on item 711**: the word went 15175 →
        // 15383 and **15382** came under it (`docs/GROUPS.md` §25). The
        // streams above agree through it.
        //
        // **Thirty-three → thirty-four on item 722**: the word went 15384 →
        // 15608 and **15385**, the make step after the second pass, came
        // under it (`docs/AI.md` §70). The streams above agree through it.
        assert_eq!(
            markets,
            vec![
                8582, 8585, 8782, 8982, 9_182, 9_382, 9_582, 9_782, 9_982, 10_182, 10_382, 10_582,
                10_782, 10_982, 11_185, 11_382, 11_582, 11_782, 11_982, 12_182, 12_385, 12_582,
                12_782, 12_982, 13_382, 13_385, 13_582, 13_782, 13_982, 14_385, 14_582, 14_585,
                15_382, 15_385
            ],
            "below the word Great Lakes takes exactly thirty-four market \
             draws — and 10582 is item 506's own: the frame the sequence \
             used to part on is a `use_market` sell on both sides now"
        );
        // **The first arrow in the game, on both sides** (item 389,
        // `docs/COMBAT.md` §9.0). This is the cheap permanent negative:
        // the original's **first** `Objects::add_ammo` in all 24,000
        // frames is sim-frame 9425, so a simulation that launches a shot
        // before it is wrong by that fact alone, whatever its draw count
        // says. It is what this crate did until the launch moved off
        // `Unit::fight` — `1/29` shot on 9415, the frame it came into
        // range, and the word sat there.
        //
        // Asserted as the whole schedule rather than the first, because
        // the release times are a *table* and one frame cannot tell a
        // right table from a lucky row: 9425 and 9426 are the two
        // archers' first arrows (`CHAR_ATTACK3` at 733 ms and
        // `CHAR_ATTACK2` at 333 ms, truncated to frames 10 and 4) and
        // 9439 and 9444 their second (1666 and 1533 ms, frames 24 and
        // 22). Past the word both streams are nobody's (item 89(c)).
        let ammo: Vec<i64> = built
            .frame_sites
            .iter()
            .filter(|(_, s)| s.iter().any(|l| l == sim::fight::SITE_AMMO_SCATTER_X))
            .map(|(f, _)| *f)
            .filter(|f| *f < LONG_WORD_GREAT_LAKES)
            .collect();
        let theirs_ammo: Vec<i64> = (0..LONG_WORD_GREAT_LAKES)
            .filter(|f| {
                trace
                    .labels(*f)
                    .iter()
                    .any(|l| l == sim::fight::SITE_AMMO_SCATTER_X)
            })
            .collect();
        assert_eq!(
            ammo, theirs_ammo,
            "Great Lakes' arrow launches below the word are not the \
             original's own"
        );
        // **The permanent negative, and it is one frame.** The original's
        // first `Objects::add_ammo` in all 24,000 frames is 9425; this
        // crate shot on 9415, the frame `1/29` came into range, until the
        // launch moved onto the animation's release event (§9.0).
        assert_eq!(
            ammo.first().copied(),
            Some(9_425),
            "Great Lakes' first arrow in the whole game is 9425, not \
             {:?} — the launch is the attack animation's, not `fight`'s",
            ammo.first()
        );
        // **Three tiers of evidence, and the assertion says which is
        // which** (item 396). Every frame in `ammo` is the *original's*,
        // by the equality against the trace above — that is not a list of
        // ours. What differs is how much of each launch anything on this
        // disk can check:
        //
        // 1. **Values, from run109's own `AMMO` record**: the nine at or
        //    below 9478. Launch point, landing point and flight time, all
        //    five numbers, re-read from the archive every run by
        //    `run109_says_great_lakes_s_launches_land_where_the_bow_hand_aims`.
        //    run109's window is `[9420, 9480)` and an arrow first prints in
        //    the block after it launches, so 9478 is the last it reaches.
        // 2. **Frames only, from run53's trace**: everything above 9478.
        //    The draw sites say the original launched on those frames and
        //    nothing on this disk says from where, to where or for how
        //    long.
        // 3. **Ours alone**: nothing. The equality above forbids it below
        //    the word, which is what makes this assertion worth making.
        //
        // So the count below is split at run109's window rather than
        // asserted as one list — a weaker claim about the seven than about
        // the nine, which is the claim the evidence supports.
        let (valued, traced): (Vec<i64>, Vec<i64>) = ammo.iter().partition(|f| **f <= 9_478);
        eprintln!(
            "  Great Lakes launches below the word: {} valued by run109 \
             {valued:?}, {} on the trace alone {traced:?}",
            valued.len(),
            traced.len()
        );
        assert_eq!(
            valued,
            vec![
                9_425, 9_426, 9_439, 9_444, 9_451, 9_456, 9_462, 9_467, 9_474
            ],
            "the launches run109 carries values for are not the nine its \
             window holds"
        );
        assert!(
            traced.iter().all(|f| *f > 9_478),
            "the partition is at run109's window: {traced:?}"
        );
        assert!(
            first_count >= LONG_WORD_GREAT_LAKES && first_part >= LONG_WORD_GREAT_LAKES,
            "run53's ceiling fell: word {first_count}, sequence {first_part}; \
             the floor is {LONG_WORD_GREAT_LAKES} on both"
        );
        // **A word at the trace's end is not a cap reading as agreement**
        // (item 899): the word is `unwrap_or(last)` when nothing parts, so
        // the claim is only as good as the walk. The trace holds a `FRAME`
        // record for every frame 0..=24000, the harness walked every one of
        // 0..24000 and compared it, and both sides spent the same draws —
        // a frame with draws on one side and a missing record on the other
        // would have parted.
        let traced: std::collections::BTreeSet<i64> =
            trace.frames.iter().map(|(n, _)| *n).collect();
        let walked: Vec<i64> = built.frame_sites.iter().map(|(f, _)| *f).collect();
        let (ours_draws, theirs_draws): (usize, usize) = built
            .frame_sites
            .iter()
            .map(|(f, o)| (o.len(), trace.labels(*f).len()))
            .fold((0, 0), |(a, b), (x, y)| (a + x, b + y));
        eprintln!(
            "run53: {} traced frames, {} walked, draws ours {ours_draws} theirs {theirs_draws}",
            traced.len(),
            walked.len()
        );
        if LONG_WORD_GREAT_LAKES == last {
            assert_eq!(
                (last, traced.len(), walked.len()),
                (24_000, 24_001, 24_000),
                "run53's trace is 24,000 frames and every one is walked"
            );
            assert!(
                (0..last).all(|n| traced.contains(&n)) && walked.iter().copied().eq(0..last),
                "a frame of 0..24000 is missing from the trace or the walk"
            );
            assert_eq!(ours_draws, theirs_draws, "the draw totals");
        }
    }

    pub(in super::super) fn run53_s_8186_is_find_target_s_probe_and_its_ring_walks() {
        let (Some(dpath), Some(tr)) = (
            dump("gamelog-run19-window-8174-8192.txt"),
            trace("rontrace-run53.log"),
        ) else {
            eprintln!("skipping: no run19/run53 capture (set RON_GAMELOG_DIR)");
            return;
        };
        use sim::fight::{ATTACK_POS_CAP_RANGED, SITE_ATTACK_POS_FIGHT, SITE_ATTACK_POS_GROUP};
        // **The frame, entry for entry, off the original's own trace.**
        let labels = tr.labels(PROBE_FRAME);
        let want: Vec<&str> = std::iter::once(sim::army::SITE_FIND_TARGET_COIN)
            .chain(std::iter::repeat_n(sim::army::SITE_FIND_TARGET_SCORE, 3))
            .chain(std::iter::repeat_n(SITE_ATTACK_POS_GROUP, 2))
            .chain(std::iter::once(sim::anim::SITE_IDLE_ANIMAL))
            .chain(std::iter::repeat_n(SITE_ATTACK_POS_FIGHT, 46))
            .chain(std::iter::once(sim::farms::SITE_CHANCE))
            .collect();
        assert_eq!(
            labels, want,
            "Great Lakes {PROBE_FRAME} is not the probe's frame any more"
        );
        // 17656 is the same shape with a different cast — the second and
        // last frame in 24,000 that reaches the address at all. It is here
        // so a reading of 8186 alone cannot be mistaken for the mechanic.
        let other = tr.labels(17_656);
        assert_eq!(
            (
                other.iter().filter(|l| *l == SITE_ATTACK_POS_GROUP).count(),
                other.iter().filter(|l| *l == SITE_ATTACK_POS_FIGHT).count(),
            ),
            (4, 10),
            "17656's two ring-walk families moved"
        );
        // **The bound, from the stream alone.**
        let chases = labels
            .iter()
            .filter(|l| *l == SITE_ATTACK_POS_FIGHT)
            .count();
        let at_least = chases.div_ceil(ATTACK_POS_CAP_RANGED);
        assert_eq!(
            (chases, at_least),
            (46, 4),
            "the chase family's size or its per-call ceiling moved"
        );
        // **And the cast, from the dump.** The probe's orders land in the
        // block *after* the frame that issues them, so 8186 → 8187 is the
        // value diff beside the word.
        let text = crate::capture::read(&dpath);
        let log = Log::parse(&text);
        let states = log.frame_states();
        let at = |n: i64| states.iter().find(|f| f.n == n).expect("block");
        let (before, after) = (at(PROBE_FRAME), at(PROBE_FRAME + 1));
        let attackers = |f: &crate::gamelog::Frame| -> Vec<(i64, i64)> {
            f.units
                .iter()
                .filter(|u| {
                    u.orders.iter().any(|o| {
                        o.named_index() == Some(10)
                            && o.mandatory == Some(1)
                            && (o.ox, o.whom, o.uid) == (Some(2004), Some(0), Some(4))
                    })
                })
                .map(|u| (u.who, u.o))
                .collect()
        };
        assert!(
            attackers(before).is_empty(),
            "the farm is already under a mandatory attack order at {PROBE_FRAME}: {:?}",
            attackers(before)
        );
        assert_eq!(
            attackers(after),
            PROBE_CAST,
            "the probe's cast moved — {} units take the farm order",
            attackers(after).len()
        );
        assert!(
            at_least <= PROBE_CAST.len(),
            "the draw stream needs {at_least} `find_attack_pos` calls and the \
             dump names only {} units to make them",
            PROBE_CAST.len()
        );
        // **Where each of them is sent** — the chase's own answer, one
        // plain `MOVEORDER` a unit, and the half of this frame a draw-for-
        // draw agreement could never check on its own.
        let sent: Vec<(i64, i64, i64, i64)> = after
            .units
            .iter()
            .filter(|u| PROBE_CAST.contains(&(u.who, u.o)))
            .map(|u| {
                let m = u
                    .orders
                    .iter()
                    .find(|o| o.kind == "MOVEORDER")
                    .expect("the cast's chase move");
                (u.who, u.o, m.x.unwrap_or(-1), m.y.unwrap_or(-1))
            })
            .collect();
        assert_eq!(
            sent, PROBE_SENT,
            "the six chase destinations moved; the farm `0/2004` stands at \
             (2112, 31296)"
        );
        // **Where the six come from**: `get_unit(0)` and `get_unit(n − 1)`
        // are the army group's first and last **captains**, and
        // `Group::add`'s subordinate recursion (`docs/GROUPS.md` §4.1)
        // brings each one's two followers. The army's standing group is
        // fifteen units in five squads of three, and that is what makes
        // two adds into six orders.
        let squad: Vec<(i64, i64)> = before
            .units
            .iter()
            .filter(|u| u.who == 1 && u.group == Some(64))
            .map(|u| (u.o, u.o_up.unwrap_or(i64::MIN)))
            .collect();
        assert_eq!(
            squad, PROBE_SQUADS,
            "the army's standing group is no longer five squads of three"
        );
        // The stance the probe sets, and the group it pushes: both are
        // `action_stance(5)` and `push_group`'s slot, and both are in the
        // record, so neither is a reading.
        let stance = |f: &crate::gamelog::Frame, k: (i64, i64)| {
            f.units
                .iter()
                .find(|u| (u.who, u.o) == k)
                .and_then(|u| Some((u.stance?, u.group?)))
        };
        for k in PROBE_CAST {
            assert_eq!(
                (stance(before, *k), stance(after, *k)),
                (Some((0, 64)), Some((5, 65))),
                "{k:?} does not take the probe's stance and group"
            );
        }
    }

    pub(in super::super) fn great_lakes_8186_sends_the_probe_s_six_where_the_original_does() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run53-greatlakes-24k-trace.txt") else {
            eprintln!("skipping: no run53 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        while built.sim.frame <= PROBE_FRAME {
            built.tick();
        }
        let ours: Vec<(i64, i64, i64, i64)> = PROBE_SENT
            .iter()
            .map(|&(who, o, _, _)| {
                let u = built
                    .sim
                    .units
                    .iter()
                    .position(|u| u.alive() && i64::from(u.owner) == who && i64::from(u.index) == o)
                    .unwrap_or_else(|| panic!("no live {who}/{o} at {}", PROBE_FRAME + 1));
                // The chase's move goes in **front** of the mandatory
                // attack order, so it is the unit's current order.
                let m = built.sim.units[u]
                    .orders
                    .iter()
                    .find_map(|od| match od.body {
                        sim::orders::Body::Move(m) => Some(m),
                        _ => None,
                    })
                    .unwrap_or_else(|| panic!("{who}/{o} carries no move at {}", PROBE_FRAME + 1));
                (who, o, i64::from(m.dest.x), i64::from(m.dest.y))
            })
            .collect();
        assert_eq!(
            ours, PROBE_SENT,
            "the probe's six are not standing where run19 puts them"
        );
        // And the cast is a mandatory attack on the farm, which is what
        // makes the move a chase rather than a walk.
        for &(who, o, _, _) in PROBE_SENT {
            let u = built
                .sim
                .units
                .iter()
                .position(|u| u.alive() && i64::from(u.owner) == who && i64::from(u.index) == o)
                .expect("the cast");
            assert!(
                built.sim.units[u].combat.mandatory,
                "{who}/{o} is not under a mandatory attack order"
            );
            assert_eq!(
                built.sim.units[u].combat.stance,
                sim::combat::Stance::HoldFire,
                "{who}/{o}'s stance is not the probe's `action_stance(5)`"
            );
        }
    }

    pub(in super::super) fn great_lakes_8186_plans_the_chase_and_8187_delays_the_blocked_one() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run53-greatlakes-24k-trace.txt") else {
            eprintln!("skipping: no run53 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let find = |built: &Built, who: i64, o: i64| -> usize {
            built
                .sim
                .units
                .iter()
                .position(|u| u.alive() && i64::from(u.owner) == who && i64::from(u.index) == o)
                .unwrap_or_else(|| panic!("no live {who}/{o}"))
        };
        while built.sim.frame <= PROBE_FRAME {
            built.tick();
        }
        // The frame the chase was ordered is over: every one of the six is
        // already carrying a plan, and the attack under it is latched.
        for &(who, o, _, _) in PROBE_SENT {
            let u = find(&built, who, o);
            assert!(
                !built.sim.units[u].path.is_empty(),
                "{who}/{o} planned nothing on {PROBE_FRAME}; run19's block \
                 {} has all six with a path stack",
                PROBE_FRAME + 1
            );
            let a = built
                .sim
                .action_of(u)
                .unwrap_or_else(|| panic!("{who}/{o} has no action"));
            assert_ne!(
                built.sim.units[u].orders[a].flags & sim::orders::flag::FIGHT_REENTRY,
                0,
                "{who}/{o}'s action is not latched; run19 prints `flags 20`"
            );
        }
        // Then the three frames run19 dumps of the blocked chaser and its
        // two walking neighbours, value for value.
        const BLOCKED: &[(i64, i64, i64, i64, i64)] = &[
            // frame, x, y, retry, safe
            (8187, 36_456, 23_592, 8, 30),
            (8188, 36_456, 23_592, 7, 29),
            (8189, 36_456, 23_592, 6, 28),
        ];
        const WALKING: &[(i64, i64, i64, i64, i64)] = &[
            // frame, 1/27's x and y, 1/29's x and y
            (8187, 36_486, 23_472, 36_578, 23_324),
            (8188, 36_477, 23_484, 36_556, 23_344),
            (8189, 36_468, 23_496, 36_534, 23_364),
        ];
        let mut blocked: Vec<(i64, i64, i64, i64, i64)> = Vec::new();
        let mut walking: Vec<(i64, i64, i64, i64, i64)> = Vec::new();
        for _ in 0..3 {
            let f = built.sim.frame;
            built.tick();
            let u = find(&built, 1, 28);
            let unit = &built.sim.units[u];
            let retry = match unit.orders.front().map(|o| o.body) {
                Some(sim::orders::Body::Move(m)) => i64::from(m.retry),
                _ => -1,
            };
            blocked.push((
                f,
                i64::from(unit.pos.x),
                i64::from(unit.pos.y),
                retry,
                i64::from(unit.safe),
            ));
            assert_eq!(
                (unit.collide, unit.collide_o),
                (1, 36),
                "{f}: `1/28` is not blocked by `1/36`, which run19 has it \
                 colliding with on every one of these frames"
            );
            let (a, b) = (find(&built, 1, 27), find(&built, 1, 29));
            walking.push((
                f,
                i64::from(built.sim.units[a].pos.x),
                i64::from(built.sim.units[a].pos.y),
                i64::from(built.sim.units[b].pos.x),
                i64::from(built.sim.units[b].pos.y),
            ));
        }
        assert_eq!(
            blocked, BLOCKED,
            "`1/28`'s delay is not run19's: (frame, x, y, retry, safe)"
        );
        assert_eq!(
            walking, WALKING,
            "the two walking chasers are not on run19's coordinates: \
             (frame, 1/27 x, y, 1/29 x, y)"
        );
    }

    pub(in super::super) fn great_lakes_8186_snaps_the_probe_s_walkers_onto_their_cell_centres() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run53-greatlakes-24k-trace.txt") else {
            eprintln!("skipping: no run53 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        // The three of the probe's six that were walking when it fired.
        const WALKERS: [(i64, i64); 3] = [(1, 40), (1, 41), (1, 42)];
        let at = |built: &Built| -> Vec<(i64, i64)> {
            WALKERS
                .iter()
                .map(|&(who, o)| {
                    let u = built
                        .sim
                        .units
                        .iter()
                        .position(|u| {
                            u.alive() && i64::from(u.owner) == who && i64::from(u.index) == o
                        })
                        .unwrap_or_else(|| panic!("no live {who}/{o}"));
                    let p = built.sim.units[u].pos;
                    (i64::from(p.x), i64::from(p.y))
                })
                .collect()
        };
        // Block 8186 — the frame before the probe, all three mid-cell.
        while built.sim.frame < PROBE_FRAME {
            built.tick();
        }
        assert_eq!(
            at(&built),
            vec![(39_133, 21_131), (36_915, 23_248), (36_883, 23_073)],
            "the three walkers are not on run19's block {PROBE_FRAME}              coordinates, so the snap below would prove nothing"
        );
        // And not one of them is on a cell centre, which is the whole
        // premise: a snap is invisible on a unit that is already there.
        for &(x, y) in &at(&built) {
            assert!(
                (x % 48, y % 48) != (24, 24),
                "({x}, {y}) is already a cell centre at block {PROBE_FRAME}"
            );
        }
        // Block 8187 — the probe's own frame, and the snap.
        built.tick();
        assert_eq!(
            at(&built),
            vec![(39_144, 21_144), (36_936, 23_256), (36_888, 23_064)],
            "the probe's walkers are not on run19's block {} coordinates",
            PROBE_FRAME + 1
        );
        for &(x, y) in &at(&built) {
            assert_eq!(
                (x % 48, y % 48),
                (24, 24),
                "({x}, {y}) is not a 48-unit cell centre"
            );
        }
        // Blocks 8188 and 8189: the walk resumes from the snapped point.
        const AFTER: [[(i64, i64); 3]; 2] = [
            [(39_117, 21_155), (36_932, 23_284), (36_877, 23_091)],
            [(39_090, 21_165), (36_928, 23_312), (36_866, 23_118)],
        ];
        for (i, want) in AFTER.iter().enumerate() {
            built.tick();
            assert_eq!(
                at(&built),
                want.to_vec(),
                "the walkers are not on run19's block {} coordinates",
                PROBE_FRAME + 2 + i as i64
            );
        }
        // **Block 8201, run19's last** — the frame the long word parted on
        // before this item, and the reason the snap is worth a test.
        const LAST_BLOCK: i64 = 8201;
        while built.sim.frame < LAST_BLOCK {
            built.tick();
        }
        assert_eq!(
            at(&built),
            vec![(38_766, 21_288), (36_880, 23_648), (36_734, 23_442)],
            "the walkers have drifted off run19's block {LAST_BLOCK}              coordinates"
        );
    }

    pub(in super::super) fn great_lakes_8442_musters_the_army_where_the_original_does() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run53-greatlakes-24k-trace.txt") else {
            eprintln!("skipping: no run53 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        while built.sim.frame <= MUSTER_FRAME {
            built.tick();
        }
        // The army that holds the nine — one slot, found by its members
        // rather than by its number, so a pool reshuffle says so here
        // instead of reading a stale slot.
        let u31 = built
            .sim
            .units
            .iter()
            .position(|u| u.alive() && u.owner == 1 && u.index == 31)
            .expect("1/31 is alive at 8442");
        let slot = built.sim.army_of(u31).expect("1/31 is in an army at 8442");
        let a = &built.sim.armies[1].list[slot];
        assert_eq!(
            (a.muster.x, a.muster.y),
            (58, 29),
            "the army's muster is not run97 block 8443's own cell"
        );
        assert_eq!(
            a.num_units, 9,
            "the army is not the nine run97 block 8443 orders"
        );
        // And `find_target`'s own two writes on the frame it takes a
        // target: the target is London's centre and `rally_dist` is
        // `0x1200` because the target changed (§12, "Taking it").
        assert_eq!(
            a.rally_dist, 0x1200,
            "the retarget did not re-arm rally_dist"
        );
        let rows: Vec<(i64, i64, i64, i64, i64)> = MUSTER_CAST
            .iter()
            .map(|&(who, o, _, _, _)| {
                let u = built
                    .sim
                    .units
                    .iter()
                    .position(|u| u.alive() && i64::from(u.owner) == who && i64::from(u.index) == o)
                    .unwrap_or_else(|| panic!("no live {who}/{o} at {MUSTER_FRAME}"));
                let g = built.sim.units[u]
                    .path
                    .first()
                    .unwrap_or_else(|| panic!("{who}/{o} carries no path at {MUSTER_FRAME}"));
                (
                    who,
                    o,
                    i64::from(built.sim.units[u].pos.x),
                    i64::from(built.sim.units[u].pos.y),
                    i64::from(g.to.x),
                )
            })
            .collect();
        assert_eq!(
            rows, MUSTER_CAST,
            "the army's nine are not standing where run97 block 8443 puts them"
        );
        let ys: Vec<i64> = MUSTER_CAST
            .iter()
            .map(|&(who, o, _, _, _)| {
                let u = built
                    .sim
                    .units
                    .iter()
                    .position(|u| u.alive() && i64::from(u.owner) == who && i64::from(u.index) == o)
                    .expect("the cast");
                i64::from(built.sim.units[u].path.first().expect("a path").to.y)
            })
            .collect();
        assert_eq!(
            ys, MUSTER_GOAL_Y,
            "the army's nine do not hold run97 block 8443's own path goals"
        );
    }

    pub(in super::super) fn run116_s_sweep_is_the_original_s() {
        use crate::trace::call_site::{
            COLLIDE_HERE, DETECT_UNIT_COLLISION, IS_CORNER, IS_HERE, WILL_BE_CORNER,
        };
        /// The sim-frame the word stood on when this landed. Pinned as an
        /// **event** and not off [`LONG_WORD_GREAT_LAKES`]: the bracket
        /// this reads is run116's, on this frame, and a test keyed to a
        /// moving headline reports the headline's motion as its own
        /// failure (`ORDER_RESIDUE_RUN97`'s lesson).
        const WORD: i64 = 10_161;
        let Some(inst) = install() else { return };
        let (Some(path), Some(tr)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            trace("rontrace-run116.log"),
        ) else {
            eprintln!("skipping: no run53/run116 capture (set RON_GAMELOG_DIR)");
            return;
        };
        // The log names its own sites, so a different probe build's log
        // — whose ids 8..10 are the leader's or the target's — cannot
        // pass this by answering about something else.
        assert_eq!(
            tr.site_va(DETECT_UNIT_COLLISION),
            Some(0x0061_7060),
            "site 8 is not Unit::detect_unit_collision; the log says {:?}",
            tr.proxied
        );
        assert_eq!(tr.site_va(COLLIDE_HERE), Some(0x0068_2540));
        assert_eq!(tr.site_va(WILL_BE_CORNER), Some(0x0060_9fa0));
        assert_eq!(tr.site_va(IS_HERE), Some(0x0060_a0c0));
        assert_eq!(tr.site_va(IS_CORNER), Some(0x0060_a040));

        // **The original's bracket.** One `detect_unit_collision` on
        // `1/38` on the word, `quick 0`, proposing the point its own
        // `MoveOrder::coll_x/coll_y` carries on block 10162 — and
        // refusing it.
        let brackets: Vec<&crate::trace::Call> = tr
            .calls
            .iter()
            .filter(|c| {
                c.site == DETECT_UNIT_COLLISION
                    && c.frame == WORD
                    && tr.unit_of(c.this) == Some((1, 38))
            })
            .collect();
        assert_eq!(
            brackets.len(),
            1,
            "the word is not one bracket on 1/38: {brackets:?}"
        );
        let b = brackets[0];
        assert_eq!(
            (b.args[0], b.args[1], b.args[2], b.ret),
            (42_774, 22_584, 0, 1),
            "the bracket is not the refused step (x, y, quick, ret)"
        );
        // **The bracket's own subtree, and nothing else on the frame.**
        // `Trace::calls` is in **return** order, so a callee precedes the
        // caller it was nested in: the children of this bracket are the
        // run of records immediately before it whose `depth` is greater
        // than its own, and an earlier sibling's subtree is fenced off by
        // that sibling's own record. Filtering the frame instead picks up
        // every unit's walk — 28 `is_here` records here against the
        // three that belong to `1/38` — which is a comparison that
        // answers about the cast and not about the word.
        let at = tr
            .calls
            .iter()
            .position(|c| std::ptr::eq(c, b))
            .expect("the bracket is in the log");
        let start = tr.calls[..at]
            .iter()
            .rposition(|c| c.depth <= b.depth)
            .map_or(0, |i| i + 1);
        let subtree = &tr.calls[start..at];
        let inner = |site: u32| -> Vec<&crate::trace::Call> {
            subtree.iter().filter(|c| c.site == site).collect()
        };
        let probe = inner(COLLIDE_HERE);
        assert_eq!(probe.len(), 1, "1/38 probes once on the word: {probe:?}");
        assert_eq!(
            (
                probe[0].args[2],
                probe[0].args[3],
                probe[0].args[4],
                probe[0].ret
            ),
            (891, 470, 1, 1),
            "the probe's cell, coll_size and answer"
        );
        assert_eq!(
            (probe[0].args[0], probe[0].args[1]),
            (38, 1),
            "the probe inside 1/38's bracket is asked by 1/38"
        );
        let corner = inner(WILL_BE_CORNER);
        assert_eq!(corner.len(), 1, "will_be_corner is called once");
        assert_eq!(
            tr.unit_of(corner[0].this),
            Some((1, 38)),
            "will_be_corner is the asker's own half"
        );
        assert_eq!(
            (corner[0].args[0], corner[0].args[1], corner[0].ret),
            (892, 471, 5),
            "the hit cell and 1/38's half of the corner rule"
        );
        // The 3x3 walk, named in who/o by the build's own identity
        // records, and the blocker's half of the corner rule.
        let walk: Vec<String> = inner(IS_HERE)
            .into_iter()
            .filter_map(|c| {
                tr.unit_of(c.this)
                    .map(|(w, o)| format!("{w}/{o}={}", c.ret))
            })
            .collect();
        assert_eq!(
            walk,
            vec!["1/39=0", "1/32=0", "1/31=1"],
            "the original's candidate walk on the word"
        );
        let blocker: Vec<String> = inner(IS_CORNER)
            .into_iter()
            .filter_map(|c| {
                tr.unit_of(c.this)
                    .map(|(w, o)| format!("{w}/{o} ({},{}) = {}", c.args[0], c.args[1], c.ret))
            })
            .collect();
        assert_eq!(
            blocker,
            vec!["1/31 (892,471) = 0"],
            "the blocker's own corner code — and `|5 - 0| != 4` is the \
             refusal"
        );

        // **And this crate's, recorded where that bracket sits.**
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        if let Some(t) = trace("rontrace-run53.log") {
            borrow_pasture(&mut init, &t);
        }
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        for _ in 0..WORD {
            built.tick();
        }
        built.sim.sweep_watch = Some(sim::collide::SweepWatch::new(WORD, 1, 38));
        built.tick();
        let ours = built
            .sim
            .sweep_watch
            .take()
            .expect("the recorder")
            .rendered();
        assert_eq!(
            ours,
            vec![
                "(42774,22584) at (891,470) hit (892,471) will 5 1/39=0 1/32=0 \
                 1/31=1,corner 0 hard 1/31"
                    .replace("                 ", "")
            ],
            "this crate's sweep on the word is not the original's"
        );
    }

    pub(in super::super) fn great_lakes_9134_is_the_snap_arm_s_blocked_stand() {
        const LAST: i64 = 9_136;
        let Some(inst) = install() else { return };
        let (Some(path), Some(r97)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            dump("gamelog-run97-greatlakes-valuewindow.txt"),
        ) else {
            eprintln!("skipping: no run53/run97 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let mut ix = crate::capture::indexed::IndexedCapture::open(&r97).unwrap();
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        if let Some(t) = trace("rontrace-run53.log") {
            borrow_pasture(&mut init, &t);
        }
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let mut rows = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        for f in 0..LAST {
            built.tick();
            // The block numbered `f + 1` is the state this tick left
            // behind, the same convention every run97 test here uses.
            if !(9_133..LAST).contains(&f) {
                continue;
            }
            let Some(at) = ix.frames().iter().position(|x| x.number == f + 1) else {
                continue;
            };
            let body = ix.read_frame(at).unwrap();
            let parsed = Log::parse(&body);
            let Some(block) = parsed
                .frames()
                .into_iter()
                .find(|(n, _)| *n == f + 1)
                .map(|(_, b)| b)
            else {
                continue;
            };
            let (units, _, _) = crate::gamelog::records(block, false);
            let Some(theirs) = units.iter().find(|u| u.who == 1 && u.o == 32) else {
                continue;
            };
            let ours = built
                .sim
                .units
                .iter()
                .find(|x| x.alive() && i64::from(x.owner) == 1 && i64::from(x.index) == 32)
                .unwrap_or_else(|| {
                    panic!("run97 block {} has 1/32 and this crate does not", f + 1)
                });
            rows += 1;
            let mine = (
                i64::from(ours.pos.x),
                i64::from(ours.pos.y),
                i64::from(ours.collide_o),
                i64::from(ours.collide_who),
                ours.collide_frame,
                ours.orders
                    .front()
                    .and_then(|o| match &o.body {
                        sim::orders::Body::Move(m) => Some(m.coll),
                        _ => None,
                    })
                    .flatten()
                    .map(|c| (i64::from(c.x), i64::from(c.y))),
                ours.path.len(),
            );
            let theirs_coll = theirs
                .current_order()
                .and_then(|o| match (o.coll_x, o.coll_y) {
                    (Some(0), Some(0)) | (None, _) | (_, None) => None,
                    (Some(x), Some(y)) => Some((x, y)),
                });
            let yours = (
                theirs.pos.x,
                theirs.pos.y,
                theirs.collide_o.unwrap_or(-1),
                theirs.collide_who.unwrap_or(-1),
                theirs.collide_frame.unwrap_or(0),
                theirs_coll,
                theirs.path.len(),
            );
            if mine != yours {
                wrong.push(format!("block {}: ours {mine:?} theirs {yours:?}", f + 1));
            }
        }
        assert_eq!(rows, 3, "run97's blocks 9134-9136 are not all here: {rows}");
        assert!(
            wrong.is_empty(),
            "Great Lakes 9134 is `move_step`'s snap arm standing still, and \
             the record says so field for field: {wrong:?}"
        );
    }

    pub(in super::super) fn run97_s_build_queues_are_the_original_s() {
        const FIRST: i64 = 8_030;
        const LAST: i64 = 9_349;
        let Some(inst) = install() else { return };
        let (Some(path), Some(r97)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            dump("gamelog-run97-greatlakes-valuewindow.txt"),
        ) else {
            eprintln!("skipping: no run53/run97 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let mut ix = crate::capture::indexed::IndexedCapture::open(&r97).unwrap();
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        if let Some(t) = trace("rontrace-run53.log") {
            borrow_pasture(&mut init, &t);
        }
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let mut blocks = 0usize;
        let mut compared = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        let mut first: std::collections::BTreeMap<(i64, i64), i64> = Default::default();
        for f in 0..=LAST {
            built.tick();
            if f < FIRST {
                continue;
            }
            let Some(at) = ix.frames().iter().position(|x| x.number == f + 1) else {
                continue;
            };
            let body = ix.read_frame(at).unwrap();
            let parsed = Log::parse(&body);
            let Some(block) = parsed
                .frames()
                .into_iter()
                .find(|(n, _)| *n == f + 1)
                .map(|(_, b)| b)
            else {
                continue;
            };
            blocks += 1;
            let (_, builds, _) = crate::gamelog::records(block, false);
            let alive = built.sim.buildings.iter().filter(|x| x.alive).count();
            assert_eq!(
                alive,
                builds.len(),
                "run97 block {} has {} buildings and this crate {alive}",
                f + 1,
                builds.len()
            );
            for b in &builds {
                let ours = built
                    .sim
                    .buildings
                    .iter()
                    .find(|x| x.alive && i64::from(x.owner) == b.who && i64::from(x.index) == b.o)
                    .unwrap_or_else(|| panic!("run97 block {} has no {}/{}", f + 1, b.who, b.o));
                // `queued` is how many slots are live; the rest of the
                // array holds whatever it was last left with.
                let live = usize::try_from(b.queued.unwrap_or(0)).unwrap_or(0);
                let theirs: Vec<(i64, i64)> = b
                    .queue
                    .iter()
                    .take(live)
                    .map(|q| (q.ty, q.job_counter))
                    .collect();
                let mine: Vec<(i64, i64)> = ours
                    .queue
                    .items
                    .iter()
                    .map(|i| {
                        (
                            i.tech.map_or_else(
                                || i64::from(built.sim.unit_types[i.ty].type_index),
                                |x| i64::from(loaded.type_index(x)),
                            ),
                            i64::from(i.job_counter),
                        )
                    })
                    .collect();
                compared += 1;
                if mine != theirs {
                    first.entry((b.who, b.o)).or_insert(f);
                    if wrong.len() < 12 {
                        wrong.push(format!(
                            "frame {f}: {}/{} ours {mine:?} theirs {theirs:?}",
                            b.who, b.o
                        ));
                    }
                }
                // both sides: both are the dump's (`job_counter`,
                // `constr_time`); a detail level that prints neither compares
                // no clock here, and the queue row above has already compared
                // the block whole.
                if let (Some(jc), Some(ct)) = (b.job_counter, b.constr_time) {
                    assert_eq!(
                        (jc, ct),
                        (i64::from(ours.job_counter), i64::from(ours.constr_time)),
                        "frame {f}: {}/{}'s construction clock is not run97's",
                        b.who,
                        b.o
                    );
                }
            }
        }
        eprintln!(
            "run97 queues: {blocks} blocks, {compared} building-frames, {} queue(s) wrong on \
             {} building(s) — {first:?}",
            wrong.len(),
            first.len()
        );
        for w in &wrong {
            eprintln!("  {w}");
        }
        assert!(
            blocks >= 1_300 && compared >= 36_000,
            "run97's own blocks are missing — the wrong file: {blocks} blocks, {compared} \
             building-frames"
        );
        // **Nothing at all, over the whole window** — item 385. The last
        // row, `1/2007` from 9182, was the Citizen this crate bought when
        // its make list put a Scholar at the head where the original puts
        // Mercenaries; with the met bit the head is the original's, the
        // purchase does not happen, and this window is silent. A building
        // parting anywhere in [8030, 9349] is now a divergence this test
        // was written to catch.
        assert_eq!(
            first.keys().copied().collect::<Vec<_>>(),
            Vec::new(),
            "run97's build queues part from the original's: {first:?}"
        );
        // **The batch itself, in the queue it filled.** 8985's
        // University is the frame two items were spent on: item 358's
        // `do_sell` paid for the second scholar, and item 362's `num`
        // bought the third in the same `make_this`. The row is gone from
        // `wrong` entirely now, so the assertion is the whole window's
        // silence on `1/2020` — made to fail by putting `num` back to
        // one, which restores `8985: 1/2020 ours [(52, 400), (52, 0)]
        // theirs [(52, 400), (52, 0), (52, 0)]` as the first row.
        assert!(
            !wrong.iter().any(|w| w.contains("1/2020")),
            "the AI's University parts from the original's somewhere in the \
             window — the scholar batch is not the original's: {:?}",
            wrong.first()
        );
        // And the whole window's silence, which is the same statement as
        // the empty map above and is kept because it prints. Made to fail
        // on purpose: putting the `human` skip back into `Sim::has_met`
        // restores `frame 9182: 1/2007 ours [(50, 100)] theirs []` as the
        // first row.
        assert!(
            wrong.is_empty(),
            "run97's window is not silent on the build queues: {:?}",
            wrong.first()
        );
    }
}
