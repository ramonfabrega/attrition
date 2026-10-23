//! **The peace pair** (DECISIONS 53 §2, 61 §6, item 2320): the successor
//! pair closed at its game's end (item 2293), and its successor is the
//! lobby that reaches the most census rows no trace has entered. East
//! Indies (`MAP_STYLE 18`), human Nubians (4) against **the Lakota (19)**
//! at Difficulty 3, seed 12345, with the successor pair's
//! `STARTING_TECHNOLOGY 8` and `STARTING_RESOURCES 7`, and **`RUSH_RULES
//! 10`**, "Peace for first 5 minutes": the nation and the rush rules are
//! what it varies (`docs/AI.md` §218). Captures: run846 (a `DUMP_ALL`
//! start) and run847 (the draw stream to the game's end, `cover=0`).

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const LAKOTA: Pair = Pair {
    name: "EastIndiesLakotaPeace",
    map_style: 18,
    ai_tribe: 19,
    difficulty: 3,
    starting_technology: 8,
    ending_technology: 7,
    starting_resources: 7,
    start: "gamelog-run846-eastindies-lakota-peace-start.txt",
    long: (
        "gamelog-run847-eastindies-lakota-peace-trace.txt",
        "rontrace-run847.log",
    ),
    length: 6822,
    // Item 2320: the first try, count and sequence both at 1384 (ours 596
    // draws, theirs 564, index 28: ours `Leader::create_buildings+0xffb`,
    // theirs `Animal::think_bird+0x82`), inside the five minutes' peace.
    // Item 2351: 1384 → 1984, the Lakota food term and its live counts
    // (`docs/AI.md` §219): ours 61 draws, theirs 62, index 28 — ours
    // `Animal::think_bird+0x82`, theirs `6bdab8` in `Leader::diplomacy`.
    // Item 2384: 1984 → 2040, `Leader::diplomacy`'s peace arm (`docs/AI.md`
    // §220): ours 37 draws, theirs 35, index 0 — ours
    // `Army::find_target+0x7df`, theirs `Unit::do_move+0xe84`.
    // Item 2388: 2040 → 3302, `release_mustering`'s pre-war gate holds the
    // navy army through the peace (`docs/AI.md` §221): ours 392 draws,
    // theirs 391, index 388 — ours `Guy::init_real+0x52`, theirs
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271`.
    // Item 2412: 3302 → 3479, `do_queue`'s silo return holds `1/2018`'s
    // second ICBM while the first stands inside (`docs/AI.md` §222): ours
    // 44 draws, theirs 46, index 0 — ours `Guy::set_anim+0x97a <
    // Unit::do_guard+0x7f4`, theirs `Guy::set_anim+0x97a < do_cast`.
    // Item 2423: 3479 → 4544, `calc_cost`'s no-rush timer (`docs/AI.md`
    // §223): who=1's Citizen `1/6` plans its crossing on 3301 with the
    // +500 on cell (15, 25), who=0's, so `path[11]` is (11928, 20376) and
    // it casts at the shore on 3479: ours 145 draws, theirs 201, index 0 —
    // ours `Leader::diplomacy+0x1168`, theirs `GameDaemon::calc_market+0x54`.
    // Item 2460: 4544 → 4859, the peace's end (`Game::do_frame:202`,
    // `docs/AI.md` §224): on 4500 every pair in play not allied goes to war
    // both ways, so who=1's view of who=0 on 4544 skips the peace coin and
    // run855's `diplos` agree (0 / 0 on 4538): ours 14 draws, theirs 15,
    // index 6 — ours `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    // `Guy::init_real+0x52` (a birth).
    // Item 2491: 4859 → 5036, `train_time`'s President arm (`docs/AI.md`
    // §225): who=1's President `1/161` stands in reach of the Airbase
    // `1/2005`, so its Strategic Bomber trains in 16500, `33000 × 100 /
    // 200`, and is born on 4859 both sides (`queued` 1 / 1 on 4860): ours
    // 164 draws, theirs 168, index 152 — ours `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271`, theirs `Guy::init_real+0x52` (a birth).
    // Item 2526: 5036 → 5135, `train_time`'s unit wonders' arm (`docs/AI.md`
    // §226): who=1's Space Program `1/2042` finishes on 5036 and halves
    // an aircraft's time, so the ICBM at `1/2030` and the Strategic Bombers
    // at `1/2032` and `1/2037` are born that frame both sides (`queued`
    // 0 / 0 on 5037): ours 24 draws, theirs 32, index 4 — ours
    // `Guy::set_anim+0x97a < Unit::do_move+0x11cf`, theirs
    // `Unit::find_attack_pos+0xea9 < Group::action_attack+0x41a`.
    // Item 2554: 5135 → 5225, `attack_dist`'s plane arm read as
    // `is_plane` (`docs/AI.md` §227): who=1's Attack Helicopter `1/145`,
    // its army's group leader on a `GROUP_ATTACK_TO`, measures the human's
    // `0/2000` edge to edge on its fifteen-frame look and hands it to the
    // group (`ATTACK` on `0/2000` for all nine members on 5136, both
    // sides): ours 24 draws, theirs 23, index 22 — ours `Guy::set_anim+0x97a
    // < Guy::inc_time+0x271`, theirs `Guy::set_anim+0x104b`.
    // Item 2609: 5225 → 5315, a shot-down aircraft's crash round
    // (`Ammo::init_crash`, `docs/AI.md` §228) holds its object number for
    // its fall: who=1's Jet Fighters `1/114` (down on 5177) and `1/120`
    // (5191) keep their slots, so the transports `Objects::find_free`
    // hands out on 5209 and 5225 are `1/189` and `1/216` both sides, where
    // ours had reused 114 and 120 (and `1/120`'s newborn figure rolled in
    // `Guy::inc_time`'s walk ahead of `1/132`): ours 30 draws, theirs 29,
    // index 0 — ours `Unit::do_move+0xe84`, theirs `Guy::set_anim+0x97a <
    // Guy::move+0x19f`.
    // Item 2638: 5315 → 5407 (count 5412), `move_step`'s snap arm
    // consumes its waypoint on a refused tile (`005fb44a` → `005fb48e`,
    // `docs/AI.md` §229): who=1's Heavy Tank `1/11` pops its sidestep
    // over the shore on 5314 where it stands (block 5315: `path` 51 and
    // `dest` 0 both sides), where ours cleared the line and re-planned a
    // frame later with `Unit::do_move+0xe84`'s grid draw. Ours 15 draws,
    // theirs 15, index 0 — ours `Guy::set_anim+0xf2f < Unit::set_anim+0x56`,
    // theirs `Guy::set_anim+0x97a < Guy::move+0x19f`.
    // Item 2677: 5407 → 5476, a crash round strikes where it comes down
    // (`init_crash`'s `splash_area` 2, `Ammo::do_damage` before the close;
    // `docs/AI.md` §230): the Jet Fighters' crashes land on the human's
    // capital `0/2000`, at its ceiling, on 5205, 5208 and 5214, so its city
    // stays attacked, its heal stays vetoed, and it holds `damage` 1560 of
    // 1560 — capture-eligible, so `valid_target` refuses it to who=1's
    // Strategic Bombers' sixteen-frame search and only the thirty-two-frame
    // look strikes it (`mandatory` 1). Ours had healed it to 1559 and every
    // bomber's search pushed a strike on it (`1/87`, `1/150`, `1/100`,
    // `1/67`, `1/195`, `1/50`, `1/48` on 5401..5408), and `1/67`'s swing
    // drew on 5407. Ours 22 draws, theirs 21, index 7 — ours
    // `Guy::set_anim+0x97a < Unit::move_step+0x823`, theirs
    // `Guy::set_anim+0x97a < Unit::do_idle+0x7d`.
    // Item 2737: 5476 → 5499, `find_upath`'s near test zeroes the goal's
    // tolerance (`683273`–`68327d`; `docs/AI.md` §231): who=1's
    // Flamethrower `1/187` (`TypeIndex` 131, land), on its group's
    // `ATTACK_TO` on the shore, collides on 5459, snaps to (29208, 38280)
    // and has its far waypoint pulled back to (29185, 38216), one 48-cell
    // away. Theirs pushes it with `tolerance` 0 and walks it (block 5461:
    // (29199, 38258), `path` 35, `dest` 1); ours kept the leg's 384,
    // popped it where it stood (`path` 34, `dest` 0), and on 5476 its
    // `move_step` swing drew at index 7. Now ours 19 draws, theirs 20,
    // index 11 — ours `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    // `Guy::init_real+0x52`: theirs births `1/263` out of `1/2032`'s
    // queue (block 5500: `queued` ours 3 against 2).
    // Item 2761: 5499 → 5564, `Unit::close`'s count on the general death
    // path (`0060f3db`; `docs/AI.md` §232): ours kept three shot-down Jet
    // Fighters in who=1's `num_units[245]` (7 against 4 from 5401, and
    // `control` 176 against 171), so the next Fighter's ramp ran twelve
    // frames long and `1/2032` handed it over on 5511. Now `1/263` is born
    // on 5499 both sides and block 5500 holds no row. Ours 26 draws,
    // theirs 24, index 7 — ours `Unit::think_scout+0xaba`, theirs
    // `Guy::set_anim+0x97a < Guy::move+0x19f`: who=1's scout `1/0` holds a
    // move ours alone on 5565 (`orders.len` 1 against 0, `group` 65
    // against 75).
    // Item 2780: 5564 → 5982, `was_really_seen`'s two leader exits
    // (`006b54f0`; `docs/AI.md` §233): who=1 holds the Space Program
    // (`leader_flags & 0x800`, run867's `0xa3f03`), so every cell is seen
    // to it, and on 5564 its Elite Special Forces `1/0` (`TypeIndex` 77,
    // land) runs `think_scout`'s region scan of region 2 and accepts no
    // cell; ours accepted (16, 6) and (21, 7), drew twice at `+0xaba` and
    // pushed group 65 with an `EXPLORE_TO`. Now ours 37 draws, theirs 287,
    // index 5 — ours `Leader::produce_building+0x1805`, theirs
    // `Build::find_gather_tiles+0x10a`.
    // Item 2779: 5982 → 5985, Capitalism's oil (`Leader::calc_gather`'s
    // `006cf454`–`006cf47f`, `CAPITALISM_OIL_PROD × 16` after the
    // refineries; `docs/AI.md` §234): who=1 took Capitalism in the gap
    // 5658..5975, so on run879's block 5976 it reads `resources[oil]`
    // 9966 against ours' 8366 (nine wells, two refineries), over its
    // 8800 cap (`over_cap` 1 against 0). Its `create_buildings` then
    // values no Oil Well, and on 5982 `make_stuff` buys `MAKE[4]` the
    // Woodcutter's Camp (`t` 418, 141000, city 1) and `find_gather_tiles`
    // shuffles its ground, where ours bought an Oil Well (421, 394500,
    // city 0) and jittered its platform. Now ours 23 draws, theirs 24,
    // index 19 — ours `Guy::set_anim+0x97a < Guy::inc_time+0x271`,
    // theirs `Guy::set_anim+0x104b`.
    // Item 2808: 5985 → 6452 (count 6455), `think_carry`'s attack arm
    // (`005f2ee0`; `docs/AI.md` §235): the Transport Freighter `1/225`
    // (`TypeIndex` 322, sea) carrying the Flamethrower `1/185` of army 1
    // loses its `ATTACK_TO` on 5955 (`do_move`'s refused plan, both
    // sides), idles on 5956, and on 5957 theirs' think sails it to army
    // 1's target's cell, (6552, 7320) for the human's `0/2000` at (6240,
    // 7008); ours had no arm and it stood idle, rolling its anim on 5985.
    // Now 22 draws each on 6452, index 0 — ours `Guy::set_anim+0x97a <
    // Unit::do_idle+0x7d` (`1/193`), theirs `Guy::set_anim+0xf2f <
    // Unit::set_anim+0x56`.
    // Item 2825: 6452 → 6808, `UnitData::max_range`'s CEO term
    // (`0060fe90`; `docs/AI.md` §236): who=1's Strategic Bomber `1/137`
    // (`TypeIndex` 305, air) strikes the human's `0/2000` from (6950,
    // 7589), `attack_dist` 918 — past three tiles' 582 — and theirs
    // releases on 6452 (`do_strafe+0x9d0`, `recharging` 41 on 6453)
    // because its owner's `0x165` hero `1/161` (type 357) stands 2371
    // off, inside its radius, and lends it `THECEO_UNIT_RANGE`, two tiles
    // (966); ours flew on and wrapped its walk. Now ours 63 draws, theirs
    // 61, index 43 — ours `Unit::find_attack_pos+0xea9 < Unit::fight+0x115e`,
    // theirs `Unit::do_move+0xe84`.
    // Item 2845: 6808 → 6821, `find_attack_pos`'s held-clear
    // `0x800000` (`601505`, OR-ed back on every exit; `docs/COMBAT.md`
    // §17.8, `docs/AI.md` §237): who=1's Rocket Artillery `1/63`
    // (`TypeIndex` 273, land) walks the bombard ring round the human's
    // `0/2000` from (6408, 8088), and with its auto-transport bit held
    // clear `invalid_loc` refuses the four water candidates (`607f28`),
    // so theirs draws 11 ring candidates to ours' 15; both pick (6648,
    // 12408). Now ours 17 draws, theirs 23, index 3 — ours
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    // `Unit::find_attack_pos+0xea9 < Unit::fight+0xcb4`, one frame before
    // the game's end (6822).
    // Item 2864: 6821 → 6822, the game's end — the draw stream agrees on
    // every frame (`docs/AI.md` §238). Three terms on who=1's MLRS
    // (`TypeIndex` 273, packed computer siege, army 1): `think`'s idle
    // search takes the army's target first (`find_nearby_target@00648da0:
    // 198`, mandatory 1) — `1/124`'s `attack[0].mandatory` on 6821, ours 0
    // against 1; a mandatory attack skips the packer's re-search and the
    // STAND_GROUND gate (`fight`'s third argument, `5fe108`/`5fe135`), so
    // `1/124` walks its six-candidate ring on 6821 as theirs does; and
    // `fight:482` asks the slot, never the health, so `1/95` at 4500 of
    // 4806 from the capital at its ceiling (`damage` 1560 of 1560) is in
    // range and unpacks (run883b's packet: `is_in_range` 1, then
    // `add_cast_order` 0x28c). On block 6822 `1/124` reads `[ATTACK
    // mandatory 1, MOVEORDER (7224, 1656)]` and `1/95` `[ATTACKTO, ATTACK,
    // CASTORDER 652]` on both sides.
    count: 6822,
    sequence: 6822,
};

/// The pair's word as the handoff's `Peace pair:` line and `AI_WORDS` carry it.
pub(crate) const LAKOTA_PAIR_WORD: i64 = if LAKOTA.count < LAKOTA.sequence {
    LAKOTA.count
} else {
    LAKOTA.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn lakota_pair_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(LAKOTA.start, LAKOTA.long) else {
        return;
    };
    eprintln!(
        "LAKOTA {}: count {} sequence {} of {}; {}",
        LAKOTA.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, LAKOTA.difficulty, "the lobby read back");
    pin_eq!(w.last, LAKOTA.length, "the game's last frame");
    pin_eq!(w.count, LAKOTA.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        LAKOTA.sequence,
        "the draw sequence's first parting"
    );
}

/// run848: the word 1384's window, blocks 1378..1634 at the long's detail (item 2320).
pub(crate) const RUN848: &str = "gamelog-run848-eastindies-lakota-peace-window-1378-1635.txt";

/// The window: block 1378 through 1634; the word 1384's own block is 1385.
pub(crate) const WIDENING_LAKOTA_FRAME_1384: (i64, i64) = (1378, 1634);

pub(crate) fn lakota_frame_1384_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKOTA.start],
        true,
        LAKOTA.long,
        "run848",
        &[(RUN848, WIDENING_LAKOTA_FRAME_1384.0)],
        WIDENING_LAKOTA_FRAME_1384,
        1,
        &[1385],
        true,
    )
}

#[test]
fn run848_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = lakota_frame_1384_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(w.firsts.len(), 70, "initial run848 baseline");
    // Item 3333: who=1's `bucket` timber 18475/18435 and metal 15805/15765,
    // standing from 1378, close with an Oil Platform priced by the Oil Wells
    // (`docs/AI.md` §270).
    // Item 2320: 870 keys part on blocks 1378..1634, the word's own block
    // being 1385 (frame 1384: ours 596 draws, theirs 564, index 28; ours
    // `Leader::create_buildings+0xffb`, theirs `Animal::think_bird+0x82`).
    // 74 stand on 1378, among them who=1's food economy — `income[food]`
    // ours 320 theirs 1728, `rate` 20/108, `econ` 3/8 — which parts on the
    // start's block 1 already (run846: 160 against 864). On the word's
    // frame, `leader:MAKE[5].val` 28124/29387 and the unit `1/13`'s `pos`
    // (38963,38203)/(39009,38201); on 1383 the building `1/2026`
    // stands at (35904,39744) with `constr_time` 70000 in ours and at
    // (40800,41568) with 560000 in theirs (`docs/AI.md` §218).
    // Item 2351: 870 → 72 (`docs/AI.md` §219), the word now 1984, past the
    // window. Every food row closes with the Lakota term — on 1378
    // `income[food]` 1728 both sides (16 peasants + 3 scouts + 3 Stable
    // units, × 64) — and with it `MAKE[5].val` and `1/13`'s path on
    // 1382..1384. What stands from 1378: who=1's `bucket[1:timber]`
    // 18475/18435 and `bucket[4:metal]` 15805/15765, and the control's
    // `SITE[n].reg`, `form` and its groups' `held`.
}

/// run849: the word 1984's window, blocks 1978..2235 at the long's detail (item 2351).
pub(crate) const RUN849: &str = "gamelog-run849-eastindies-lakota-peace-window-1978-2236.txt";

/// The window: block 1978 through 2235; the word 1984's own block is 1985.
pub(crate) const WIDENING_LAKOTA_FRAME_1984: (i64, i64) = (1978, 2235);

pub(crate) fn lakota_frame_1984_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKOTA.start],
        true,
        LAKOTA.long,
        "run849",
        &[(RUN849, WIDENING_LAKOTA_FRAME_1984.0)],
        WIDENING_LAKOTA_FRAME_1984,
        1,
        &[1985, 2041],
        true,
    )
}

#[test]
fn run849_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = lakota_frame_1984_window() else {
        return;
    };
    pin_eq!(w.blocks, 258, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(w.firsts.len(), 85, "initial run849 baseline");
    // Item 3333: who=1's `bucket` timber 17466/17361 and metal 15317/15212,
    // standing from 1978, close with an Oil Platform priced by the Oil Wells
    // (`docs/AI.md` §270).
    // Item 2351: 1242 keys part on blocks 1978..2235, the word's own block
    // being 1985 (frame 1984: ours 61 draws, theirs 62, index 28; ours
    // `Animal::think_bird+0x82`, theirs `6bdab8` in `Leader::diplomacy`).
    // On the word's block one row: who=1's `leader:agendas[0]` ours 0
    // theirs 64. 74 stand on 1978: who=1's `bucket[1:timber]` 17466/17361
    // and `bucket[4:metal]` 15317/15212 (40 apart on run848's 1378), and
    // the control's `SITE[n].reg`, `form` and groups' `held`
    // (`docs/AI.md` §219).
    // Item 2384: 1242 → 1240, `Leader::diplomacy` (`docs/AI.md` §220). The
    // word's block 1985 holds no row: who=1's `agendas[0]` 64, `aggression[0]`
    // 1, `strong[0]` 160, `weak[0]` 0 and who=0's `got_diplo_message` 1984,
    // both sides, the last four compared from this item. The word is now
    // 2040 (block 2041, in this window): 47 keys part there, who=1's
    // `1/25`, `1/26`, `1/36` given moves in ours (`1/25` to (35112, 44088)
    // against theirs' (44424, 32904); `1/26` and `1/36` none in theirs).
    // Item 2388: 1240 → 87, `release_mustering`'s pre-war gate (`docs/AI.md`
    // §221). Block 2041 holds no row: `1/25` (`AEGISCRUISER`) holds its one
    // `ATTACKTOORDER` to (44424, 32904) both sides, `1/26` and `1/36` none.
    // 74 stand on 1978 as before and 13 part later (parked 2411). The word
    // is 3302, past this window.
}

/// run853: the word 3302's window, blocks 3296..3552 at the long's detail (item 2412).
pub(crate) const RUN853: &str = "gamelog-run853-eastindies-lakota-peace-window-3296-3553.txt";

/// The window: block 3296 through 3552; the word 3302's own block is 3303.
pub(crate) const WIDENING_LAKOTA_FRAME_3302: (i64, i64) = (3296, 3552);

pub(crate) fn lakota_frame_3302_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKOTA.start],
        true,
        LAKOTA.long,
        "run853",
        &[(RUN853, WIDENING_LAKOTA_FRAME_3302.0)],
        WIDENING_LAKOTA_FRAME_3302,
        1,
        &[3303, 3480],
        true,
    )
}

#[test]
fn run853_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = lakota_frame_3302_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(w.firsts.len(), 97, "initial run853 baseline");
    // Item 3333: who=1's `bucket` timber 13437/13252 and metal 11970/11785,
    // standing from 3296, close with an Oil Platform priced by the Oil Wells
    // (`docs/AI.md` §270).
    // Item 2412: the word 3302's own block is 3303. Without the silo
    // return, who=1's Missile Silo `1/2018` trains its second ICBM (316)
    // from 2582 and births it on 3302 (`Guy::init_real+0x52`, ours' extra
    // draw at index 388); theirs holds the entry at `job_counter` 0 on
    // every block 3296..3552 with the first ICBM `1/50` inside
    // (`inside_down` 50). With the return built (`docs/AI.md` §222) block
    // 3303 holds no row; 81 keys stand from 3296. The word is now 3479
    // (block 3480): who=1's Citizen `1/6` (type 50, group 69) casts spell
    // 650 at the shore in theirs (`CASTORDER` over its `MOVEORDER`, kind
    // 14 to (12111, 20376)) and is inside the Transport Freighter `1/140`
    // on 3480; ours holds kind 1 to (2712, 18840). Group 69 holds `[6]`
    // in ours from 3302 and nothing in theirs (parked 2411's group).
    // Item 2423: 169 → 100. With the no-rush timer built (`docs/AI.md`
    // §223) `1/6`'s `path[11].to` is (11928, 20376) on 3302 on both sides
    // (ours (11928, 19608) without it), and its 3441..3480 rows, the cast,
    // the Freighter `1/140` and its rows all close.
}

/// run855: the word 4544's window, blocks 4538..4794 at the long's detail (item 2423).
pub(crate) const RUN855: &str = "gamelog-run855-eastindies-lakota-peace-window-4538-4795.txt";

/// The window: block 4538 through 4794; the word 4544's own block is 4545.
pub(crate) const WIDENING_LAKOTA_FRAME_4544: (i64, i64) = (4538, 4794);

pub(crate) fn lakota_frame_4544_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKOTA.start],
        true,
        LAKOTA.long,
        "run855",
        &[(RUN855, WIDENING_LAKOTA_FRAME_4544.0)],
        WIDENING_LAKOTA_FRAME_4544,
        1,
        &[4545],
        true,
    )
}

#[test]
fn run855_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = lakota_frame_4544_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    // Item 2761: 128 → 122 with `Unit::close`'s count (`docs/AI.md` §232).
    // Item 3279: −14, `death:extra` on a capture that logs no `DEATHS` (`docs/AI.md` §259).
    pin_eq!(w.firsts.len(), 103, "initial run855 baseline"); // Item 3172: −2, the Sulphur siege arm (`get_cost`, a Siege Factory unit)
    // Item 3333: who=1's `bucket` timber 8265/8000 and metal 9997/9732,
    // standing from 4538, close with an Oil Platform priced by the Oil Wells
    // (`docs/AI.md` §270).
    // Item 2423: the word 4544's own block is 4545, and it holds no row:
    // the state parts before the window opens. On 4538 `leader:diplos` is
    // ours 1 against theirs 0 both ways (`0/-1 diplos[1]`, `1/-1
    // diplos[0]`), the ICBM `1/50` (inside the silo `1/2018`) holds one
    // order in theirs and none in ours, `1/0` walks a 36-leg path to
    // (15000, 11160) in theirs and idles in ours, and `1/6` has 7 damage
    // ours against 2. On 4542 theirs' `1/50` is gone and `1/2018`'s
    // `queue[0].job_counter` runs (0 / 100); on 4576 `wars` is 0 / 1.
    // 102 keys stand from 4538, 28 of them `form`.
    // Item 2460: 977 → 128 with the peace's end built. `diplos`, `wars`,
    // `1/50`'s order and `1/0`'s path close; 83 keys stand from 4538 (28
    // `form`, ten `SITE[*].reg`, 19 groups' `held` and their pools, who=1's
    // food, timber and metal buckets, `1/6`'s damage and two `mirror`s).
    // The first key past 4538 is group 80 on 4557; the human's `active`
    // parts on 4679 (ours 14, theirs 13), where `death:extra` begins.
}

/// run861: the word 4859's window, blocks 4853..5109 at the long's detail (item 2460).
pub(crate) const RUN861: &str = "gamelog-run861-eastindies-lakota-peace-window-4853-5110.txt";

/// The window: block 4853 through 5109; the word 4859's own block is 4860.
pub(crate) const WIDENING_LAKOTA_FRAME_4859: (i64, i64) = (4853, 5109);

pub(crate) fn lakota_frame_4859_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKOTA.start],
        true,
        LAKOTA.long,
        "run861",
        &[(RUN861, WIDENING_LAKOTA_FRAME_4859.0)],
        WIDENING_LAKOTA_FRAME_4859,
        1,
        &[4860],
        true,
    )
}

#[test]
fn run861_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = lakota_frame_4859_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    // Item 2761: 126 → 122 with `Unit::close`'s count (`docs/AI.md` §232).
    // Item 3279: −14, `death:extra` on a capture that logs no `DEATHS` (`docs/AI.md` §259).
    // Item 3336: −6, `Wall::set_type` leaving the hits to the next `calc_wall_stats` (`docs/AI.md` §272).
    // Item 3335: −1, the group change (every attacking type in its trainer's group).
    pin_eq!(w.firsts.len(), 97, "initial run861 baseline"); // Item 3172: −2, the Sulphur siege arm (`get_cost`, a Siege Factory unit)
    // Item 3333: who=1's `bucket` timber 7707/7282 and metal 6422/5997,
    // standing from 4853, close with an Oil Platform priced by the Oil Wells
    // (`docs/AI.md` §270).
    // Item 2460: the word 4859's own block is 4860. Theirs spends a birth
    // there (`Guy::init_real < Unit::init < Objects::init_unit`): `1/50`
    // is unlinked on 4860 (the dump holds it alone), `1/2005`'s `queued`
    // is ours 2 against theirs 1 and who=1's `num_units[255]` (`ABRAMDES`)
    // ours 8 against 9. Ours' `1/50` shows on 4878. 106 keys stand from
    // 4853: the human's `control` ours 14 against theirs 0 and its
    // fourteen units' `death:extra`, who=1's food, timber and metal
    // buckets, and `1/2005`'s `queue[0].cost[0..1]` ours 240 against
    // theirs 224. On 4859 `1/168`, `1/172`, `1/177` hold `cavarch_who` 0
    // in ours and −1 in theirs.
    // Item 2526: 378 → 126 with the Space Program's air arm built. On
    // block 5037 `1/2030`, `1/2032` and `1/2037` read `queued` 0 / 0,
    // `1/209`–`1/211` are born both sides, who=1's `num_units[255]` is
    // 11 / 11 and `1/2005`'s head `job_counter` 16875 / 16875 (clamped to
    // the halved target, its handover refused). Past 5037 only two `form` rows
    // part (`1/208` on 5033, `1/213` on 5055); the 106 keys from 4853
    // stand.
}

/// run866: the word 5135's window, blocks 5129..5385 at the long's detail (item 2554).
pub(crate) const RUN866: &str = "gamelog-run866-eastindies-lakota-peace-window-5129-5386.txt";

/// The window: block 5129 through 5385; the word 5135's own block is 5136.
pub(crate) const WIDENING_LAKOTA_FRAME_5135: (i64, i64) = (5129, 5385);

pub(crate) fn lakota_frame_5135_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKOTA.start],
        true,
        LAKOTA.long,
        "run866",
        &[(RUN866, WIDENING_LAKOTA_FRAME_5135.0)],
        WIDENING_LAKOTA_FRAME_5135,
        1,
        &[5136],
        true,
    )
}

#[test]
fn run866_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = lakota_frame_5135_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    // Item 2761: 178 → 164 with `Unit::close`'s count (`docs/AI.md` §232):
    // who=1's `num_units[245]` on 5178, `[255]` and `[261]` on 5201 agree.
    // Item 2780: 164 → 163 with the scout fog read's leader exits
    // (`docs/AI.md` §233): `1/187`'s `path[2].to` on 5378 (ours (9240,
    // 4440) against (10008, 4440)) agrees — `calc_cost` reads the same
    // fog, and a Space Program sees every cell.
    // Item 3279: −14, `death:extra` on a capture that logs no `DEATHS` (`docs/AI.md` §259).
    // Item 3514: 137 → 121, `get_cost`'s domain arm: the Space Program halves
    // who=1's aircraft, so the Airbase queues' `cost` rows and the wealth/oil
    // buckets agree (run866's 5383: `1/2032` 97 against 52; `docs/AI.md` §301).
    pin_eq!(w.firsts.len(), 121, "initial run866 baseline");
    // Item 3333: who=1's `bucket` metal 5716/5231, standing from 5129, closes
    // with an Oil Platform priced by the Oil Wells (`docs/AI.md` §270); the
    // timber row, ours above theirs till now, parts on 5183 with ours below
    // (3866/4692).
    // Item 2554: the word 5135's own block is 5136, widened at 5707 keys.
    // Theirs gives who=1's naval army (group 72, nine members: the
    // Missile Cruisers `1/20`, `1/21`, `1/25`, `1/26`, `1/34`, `1/36`,
    // `1/42`, `1/49` and the Attack Helicopter `1/145`, its leader) an
    // `ATTACK` on the human's `0/2000`, where ours kept the attack-move
    // (`1/145` `order:kind` ours 21 against theirs 10, `1/21` `orders.len`
    // ours 0 against 1). `1/145`'s fifteen-frame look found nothing in
    // ours: `attack_dist` measured the helicopter centre to centre.
    // 5707 → 3550 with `is_plane` built: on 5136 the nine read `ATTACK`
    // both sides (`1/20` at (9442, 15614), `1/145` at (9158, 11227)), and
    // 5136 and 5137 hold no row. 99 keys stand from 5129 (24 `form`, 17
    // leader, among them the human's `control` 14 against 0, and `1/197`'s
    // `air.mandatory`); the first past it is group 64 on 5141, and
    // `0/2000`'s `damage_frac` parts on 5161 (ours 0, theirs 10).
    // Item 2609: the word 5225's own block is 5226, widened at 3550 keys.
    // Theirs numbers the MLRS `1/124`'s transport `1/216` (`inside` ours
    // 120 against theirs 216, `1/120` "this crate holds it alone"), and on
    // 5210 `1/194`'s `1/189` where ours took 114: both numbers were Jet
    // Fighters shot down on 5177 and 5191, whose crash rounds hold them in
    // theirs. 3550 → 1340 with the crash round laid: on 5226 `1/124`
    // reads `inside` 216 and `1/216` stands both sides (its one row is
    // `form`, ours −1 against 0, the pair's standing `form` residue), and
    // 5210 and 5219 hold no row. Still standing: `1/2014`/`1/2022`'s
    // `cost[1]` on 5183 (155 against 116, parked 2579), who=1's
    // `num_units[255]`, `[261]` on 5201 and `1/2005`'s `job_counter` on
    // 5202 (17625 against 17250).
    // Item 2638: the word 5315's own block is 5316, widened at 1340 keys.
    // Every row is who=1's Heavy Tank `1/11` (`TypeIndex` 251, land): on
    // 5315 theirs has popped the sidestep (28968, 38616) it snapped onto
    // over the shore (`path` 51, `dest` 0) where ours kept it (`path` 52,
    // `dest` 1); on 5316 theirs has taken the boarding waypoint (28728,
    // 38184) and met the Transport Freighter `1/221` (`collide_o` 221)
    // where ours re-planned. 1340 → 180 with the snap arm's refusal built:
    // `1/11` holds no row in the window, and with it go the boarding's
    // sequels (`1/229`'s birth and `1/11`'s `inside` on 5319, `1/86`,
    // `1/106`, `1/172`, `1/179`). Left: the keys standing from 5129 (the
    // human's `SITE` and `control`, `death:extra`, `mirror`, the pools and
    // held groups), 57 `form` rows, `1/228`'s `g.angle[0]` on 5316 (a
    // Scholar inside `1/114`, ours 1431655765 against 0), group 69's held
    // `1/184` on 5327, `1/187`'s `path[2]` on 5378, who=1's `MAKE` values
    // on 5381, and its buckets and queue `cost` rows on 5383.
    // Item 2677: 180 → 178 with the crash round's strike and the clamp's
    // kept fraction: `0/2000`'s `damage_frac` on 5161 (ours 0 against
    // theirs 10) and its `city_flags[0x4]` on 5206 (0 against 1, a crash
    // landing on 5205) agree.
}

/// run867: the word 5407's window, blocks 5401..5657 at the long's detail (item 2677).
pub(crate) const RUN867: &str = "gamelog-run867-eastindies-lakota-peace-window-5401-5658.txt";

/// The window: block 5401 through 5657; the word 5407's own block is 5408.
pub(crate) const WIDENING_LAKOTA_FRAME_5407: (i64, i64) = (5401, 5657);

pub(crate) fn lakota_frame_5407_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKOTA.start],
        true,
        LAKOTA.long,
        "run867",
        &[(RUN867, WIDENING_LAKOTA_FRAME_5407.0)],
        WIDENING_LAKOTA_FRAME_5407,
        1,
        &[5408],
        true,
    )
}

#[test]
fn run867_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = lakota_frame_5407_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    // Item 3514: 161 → 137, `get_cost`'s domain arm: the Space Program halves
    // who=1's aircraft, so the Airbase queues' `cost` rows and the wealth/oil
    // buckets agree (run866's 5383: `1/2032` 97 against 52; `docs/AI.md` §301).
    pin_eq!(w.firsts.len(), 137, "initial run867 baseline");
    // Item 3333: who=1's `bucket` metal 5815/5330, standing from 5401, closes
    // with an Oil Platform priced by the Oil Wells (`docs/AI.md` §270); the
    // timber row stands, ours below theirs.
    // Item 2677: the word 5407's own block is 5408, widened at 3418 keys.
    // Every who=1 Strategic Bomber (`TypeIndex` 305, air) on its
    // `AIRPATROLORDER` round the human's capital `0/2000` (6240, 7008)
    // parts: in ours its sixteen-frame search pushes a `STRAFEORDER` on
    // the capital (`mandatory` 0, kept), in theirs only the thirty-two-frame
    // look does (`mandatory` 1, popped the next frame) — `1/87` on 5402,
    // `1/150` 5403, `1/100` 5405, `1/67` and `1/195` 5406, `1/50` 5407,
    // `1/48` 5409 (`order:kind` ours 16 against 17, `orders.len` 2 against
    // 1); on 5408 `1/67` reads `recharging` ours 41 against 0, its swing.
    // The capital reads `damage` ours 1559 against theirs 1560 of `myhits`
    // 1560, `damage_frac` 0 against 4 and `city_flags[0x2]` 0 against 1:
    // theirs is at its ceiling, capture-eligible, and refused by
    // `valid_target`. 3418 → 2430 with the crash round's strike: no bomber
    // and no `0/2000` row stands in the window. Left: the keys standing
    // from 5401 (163, the bombers' `mirror` among them), the `form` rows,
    // `1/187` on 5460 (`path[34].tolerance` ours 384 against 0, on the
    // shore at (29208, 38280)) and its move from 5461, the new word's.
    // Item 2737: the word 5476's own block is 5477, widened at 2430 keys.
    // Its four rows are `1/187`'s collision (`collide_o` ours 177 against
    // −1, `order:coll`), the sequel of 5460's `path[34].tolerance` (ours
    // 384 against 0), walked back: block 5461 `pos`, `dest`, `path:length`
    // and `tolerance`, 5466 the re-plan, 5477 the collision. 2430 → 1813
    // with `find_upath`'s zeroed near goal: on 5460 `path[34]` reads
    // (29185, 38216) `tolerance` 0 both sides, on 5477 `1/187` stands at
    // agreeing coordinates, and it holds no row in the window. Left: the
    // keys standing from 5401, the `form` rows, `1/261`'s `g.angle[0]` on
    // 5471, and from 5500 `1/263`'s birth in theirs alone (who=1's
    // `num_queued[245]` ours 7 against 6, `1/2032` `queued` 3 against 2),
    // the new word's.
    // Item 2761: the word 5499's own block is 5500, widened at 1813 keys.
    // Its four rows are `1/263`'s birth in theirs alone, `1/2032`'s
    // `queued`, who=1's `num_queued[245]` and `active`; walked back, the
    // Airbase's `job_counter` agrees to 11700 on 5499 and the count that
    // parts is the ramp's input, who=1's `num_units[245]` (7 against 4)
    // and `control` (176 against 171), standing from 5401: three Jet
    // Fighters shot down before the window, still counted in ours. 1813 →
    // 276 with `Unit::close`'s count: `1/263` is born on 5499 both sides,
    // no leader row stands from 5401 but the human's `SITE` and who=1's
    // buckets, and from 5565 `1/0`'s scouting move parts, the new word's.
    // Item 2780: the word 5564's own block is 5565, widened at 276 keys.
    // Its fourteen rows are `1/0`'s, on 5565 and 5566 (`group` ours 65
    // against 75, `orders.len` 1 against 0, `orders_x/y` (16632, 5880)
    // against (14976, 11136), `path:length` 6 against 0, then its walk):
    // ours' region scan accepted two cells theirs did not, and the
    // standing `group:65.held`/`pool:65` from 5542 are the dump's blind
    // `held` (theirs prints `None` on every group). 276 → 183 with the fog
    // read's leader exits: `1/0` holds no row, and with it go every row
    // from 5565 the scout's draws shifted — `1/264` and `1/263`'s
    // flights, `1/93` and `1/276`'s moves and who=1's `MAKE` on 5583,
    // `1/176` on 5627 — and the standing `1/187` `path[2].to` from 5401
    // and `1/234`'s on 5522. Left: the standing keys from 5401, the
    // `form` rows, the three `g.angle[0]` births, two `mirror`s, the
    // queues' `cost` on 5583, `1/210`/`1/211`'s `mandatory`. The new word
    // 5982 is past the window (5657): its widening is owed (parked 2779).
}

/// run879: the word 5982's window, blocks 5976..6232 at the long's detail (item 2779).
pub(crate) const RUN879: &str = "gamelog-run879-eastindies-lakota-peace-window-5976-6233.txt";

/// The window: block 5976 through 6232; the word 5982's own block is 5983.
pub(crate) const WIDENING_LAKOTA_FRAME_5982: (i64, i64) = (5976, 6232);

pub(crate) fn lakota_frame_5982_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKOTA.start],
        true,
        LAKOTA.long,
        "run879",
        &[(RUN879, WIDENING_LAKOTA_FRAME_5982.0)],
        WIDENING_LAKOTA_FRAME_5982,
        1,
        &[5983],
        true,
    )
}

#[test]
fn run879_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = lakota_frame_5982_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    // Item 2779: the word 5982's own block is 5983, widened at 4745 keys.
    // Its first parting is who=1's oil, standing from 5976: `resources[oil]`
    // ours 8366 against theirs 9966, `income` 8366 against 8800, `over_cap`
    // 0 against 1, `leftover` 6630 against 1592, and `rate` on 5977 (522
    // against 550) — Capitalism's `CAPITALISM_OIL_PROD × 16`, held from the
    // gap 5658..5975 (run867's 5401..5657 agrees on every oil row). On block
    // 5982 `MAKE[4]` parts: ours the Oil Well (`t` 421, `val` 394500, city
    // 0), theirs the Woodcutter's Camp (418, 141000, city 1). 4745 → 2854
    // with Capitalism's oil and gift: no oil row but `bucket` stands, block
    // 5982 holds no `MAKE` row, and the word's block 5986 holds none. Left:
    // the keys standing from 5976 (the human's `SITE`, `death:extra`, the
    // held pools, who=1's buckets, the `form` and `mirror` rows, the
    // births' `g.angle[0]`), the queues' `cost` rows, and the Transport
    // Freighter `1/225` (`TypeIndex` 322, sea), idle in ours (`idle` 3)
    // and on a move to (6552, 7320) in theirs from 5976 — the new word's
    // candidate; the first row past it is `1/238`'s path on 5990.
    // Item 2808: the word 5985's own block is 5986, widened at 2854 keys;
    // it holds no row. The draws part on `1/225`, whose rows stand from
    // 5976 (above): ours idle at (16200, 27864) with no order, theirs on a
    // `MOVEORDER` to (6552, 7320) at (15864, 27440), 27 legs. 2854 → 191
    // with `think_carry`'s attack arm: `pos`, the order, the path and the
    // headings of `1/225` agree on 5976, and so does every row the
    // freighter's idle draws shifted after it; `1/225` keeps only its
    // standing `form`. Left: the standing keys from 5976, and from 6045
    // `1/164`'s `attrition` (ours 6, theirs 0) and then its `hits`.
    // Item 3514: 174 → 160, `get_cost`'s domain arm: the Space Program halves
    // who=1's aircraft, so the Airbase queues' `cost` rows and the wealth/oil
    // buckets agree (run866's 5383: `1/2032` 97 against 52; `docs/AI.md` §301).
    pin_eq!(w.firsts.len(), 160, "initial run879 baseline");
    // Item 3333: who=1's `bucket` metal 4998/4453, standing from 5976, closes
    // with an Oil Platform priced by the Oil Wells (`docs/AI.md` §270); the
    // timber row stands, ours below theirs.
}

/// run881: the word 6452's window, blocks 6446..6701 at the long's detail (item 2825).
pub(crate) const RUN881: &str = "gamelog-run881-eastindies-lakota-peace-window-6446-6702.txt";

/// The window: block 6446 through 6701; the word 6452's own block is 6453.
pub(crate) const WIDENING_LAKOTA_FRAME_6452: (i64, i64) = (6446, 6701);

pub(crate) fn lakota_frame_6452_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKOTA.start],
        true,
        LAKOTA.long,
        "run881",
        &[(RUN881, WIDENING_LAKOTA_FRAME_6452.0)],
        WIDENING_LAKOTA_FRAME_6452,
        1,
        &[6453],
        true,
    )
}

#[test]
fn run881_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = lakota_frame_6452_window() else {
        return;
    };
    pin_eq!(w.blocks, 256, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    // Item 2825: the word 6452's own block is 6453, widened at 2138 keys.
    // On it `1/137`, who=1's Strategic Bomber (`TypeIndex` 305), reads
    // `recharging` ours 0 against theirs 41: theirs bombed `0/2000` on
    // 6452 at `attack_dist` 918, ours flew on, and from 6470 the five
    // bombers of its flight (`1/137`, `1/205`, `1/290`, `1/293`, `1/304`)
    // read `order:kind` ours 16 (`STRAFE`) against 17 (`AIRPATROL`).
    // Standing from 6446: `1/137`/`1/293` `mandatory` (ours 0, theirs 1),
    // the flight's `mirror`, the held pools, who=1's buckets, the human's
    // `SITE`. Walked back: `pos`, heading and `recharging` agree through
    // 6452, and `in_range` was the gate (a reach of 582 from three
    // tiles). 2138 → 252 with `max_range`'s CEO term: `recharging` 41 /
    // 41 on 6453 and no order row from 6470. Left: the standing keys from
    // 6446, the `attrition`/`sheltered` rows from 6453 (`1/44` ours 6,
    // theirs 0, as run879's `1/164` on 6045), the `form` and
    // `dest_angle` rows.
    // Item 3514: 235 → 227, `get_cost`'s domain arm: the Space Program halves
    // who=1's aircraft, so the Airbase queues' `cost` rows and the wealth/oil
    // buckets agree (run866's 5383: `1/2032` 97 against 52; `docs/AI.md` §301).
    pin_eq!(w.firsts.len(), 227, "initial run881 baseline");
    // Item 3333: who=1's `bucket` metal 5193/4648, standing from 6446, closes
    // with an Oil Platform priced by the Oil Wells (`docs/AI.md` §270); the
    // timber row stands, ours below theirs.
}

/// run882: the word 6808's window, blocks 6802..6821 at the long's detail (item 2845).
pub(crate) const RUN882: &str = "gamelog-run882-eastindies-lakota-peace-window-6802-6822.txt";

/// The window: block 6802 through 6821; the word 6808's own block is 6809.
pub(crate) const WIDENING_LAKOTA_FRAME_6808: (i64, i64) = (6802, 6821);

pub(crate) fn lakota_frame_6808_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKOTA.start],
        true,
        LAKOTA.long,
        "run882",
        &[(RUN882, WIDENING_LAKOTA_FRAME_6808.0)],
        WIDENING_LAKOTA_FRAME_6808,
        1,
        &[6809],
        true,
    )
}

#[test]
fn run882_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = lakota_frame_6808_window() else {
        return;
    };
    pin_eq!(w.blocks, 20, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    // The word 6808's own block is 6809 (item 2845). Before the fix it
    // held two rows — who=0's `bucket[2:wealth]` (20891 / 20892) and
    // `1/264`'s `air.cruising_alt` (1700 / 1500) — and the draw delta
    // was `1/63`'s ring: ours 15 candidates, theirs 11, both `MOVEORDER`
    // to (6648, 12408) on 6809. With the held-clear bit the ring agrees
    // and 259 rows fall to 248: block 6809 keeps only the leader's
    // bucket, and the `cruising_alt`, `1/61` `gather.wait`, `1/289` and
    // `1/23` rows go. 235 stand from 6802 (`form`, `attrition` and
    // `sheltered`, `death:extra`, the human's `SITE`, pools, buckets,
    // territory); later ones are `1/189` `form` (6811), `0/2000`'s
    // `damage` and `city_flags` (6816) and `1/124`'s
    // `attack[0].mandatory` on 6821, the new word's frame.
    // Item 2864: 248 → 247, `1/124`'s `attack[0].mandatory` goes with
    // `think`'s army shortcut (`docs/AI.md` §238).
    // Item 3514: 230 → 222, `get_cost`'s domain arm: the Space Program halves
    // who=1's aircraft, so the Airbase queues' `cost` rows and the wealth/oil
    // buckets agree (run866's 5383: `1/2032` 97 against 52; `docs/AI.md` §301).
    pin_eq!(w.firsts.len(), 222, "initial run882 baseline");
    // Item 3333: who=1's `bucket` metal 5378/4833, standing from 6802, closes
    // with an Oil Platform priced by the Oil Wells (`docs/AI.md` §270); the
    // timber row stands, ours below theirs.
}

/// run883: the word 6821's window, blocks 6815..6822 and the closing block
/// 6823 at the long's detail (item 2864).
pub(crate) const RUN883: &str = "gamelog-run883-eastindies-lakota-peace-window-6815-6823.txt";

/// The window: block 6815 through the closing block 6823; the word 6821's
/// own block is 6822.
pub(crate) const WIDENING_LAKOTA_CLOSING: (i64, i64) = (6815, 6823);

pub(crate) fn lakota_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKOTA.start],
        true,
        LAKOTA.long,
        "run883",
        &[(RUN883, WIDENING_LAKOTA_CLOSING.0)],
        WIDENING_LAKOTA_CLOSING,
        1,
        &[6822],
        true,
    )
}

#[test]
fn run883_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = lakota_closing_window() else {
        return;
    };
    pin_eq!(w.blocks, 8, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    // Item 2864: the word 6821's own block is 6822, and the closing block
    // 6823 follows it. The first walk, before the fix: 260 keys, 240
    // standing on 6815 (the pair's `form`, `mirror`, `attrition` and
    // `sheltered`, `g.angle`, pools, buckets, `1/2005`'s queue costs, the
    // human's `SITE`), then `0/2000`'s `damage` and `city_flags` (6816),
    // `1/131` `sheltered` (6818), who=0's `bucket[2:wealth]` (6820),
    // `1/124`'s `attack[0].mandatory` (6821, ours 0 against 1), and on the
    // word's block 6822 `1/124`'s order (ours none against `[ATTACK,
    // MOVEORDER (7224, 1656)]`, path 0 against 5) and `1/95`'s (ours
    // `ATTACKTO` alone against `[ATTACKTO, ATTACK, CASTORDER 652]`). With
    // the army shortcut, the mandatory chase and `fight:482`'s slot test,
    // 260 → 246: block 6822 keeps one row, the Armored Cavalry `1/106`'s
    // `cavarch_o` (ours −1, theirs 2000 — its fire-on-the-move target,
    // no draw, no order).
    // Item 3514: 229 → 221, `get_cost`'s domain arm: the Space Program halves
    // who=1's aircraft, so the Airbase queues' `cost` rows and the wealth/oil
    // buckets agree (run866's 5383: `1/2032` 97 against 52; `docs/AI.md` §301).
    pin_eq!(w.firsts.len(), 221, "run883 baseline");
    // Item 3333: who=1's `bucket` metal 5389/4844, standing from 6815, closes
    // with an Oil Platform priced by the Oil Wells (`docs/AI.md` §270); the
    // timber row stands, ours below theirs.
}

/// **The pair's closing state** (item 2864, DECISIONS 55): run883's
/// closing block 6823, walked to and compared whole.
#[test]
fn run883_lakota_pair_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) =
        (dump(RUN883), dump(LAKOTA.start), trace(LAKOTA.long.1))
    else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(path);
    let start_text = crate::capture::read(start);
    let log = Log::parse(&text);
    let start_log = Log::parse(&start_text);
    let mut init = start_log.initial().unwrap();
    let sibling = start_log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sibling]);
    borrow_pasture(&mut init, &tr);
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    let fin = log
        .final_state()
        .or_else(|| log.frame_states().pop())
        .expect("closing whole-map state");
    let e = endpoint::walk_to_close(&mut built, &fin, 8);
    eprintln!(
        "Lakota pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
        e.frame,
        e.compared,
        e.counts(),
        e.torn,
        e.off,
        e.unlinked,
        e.extra
    );
    eprintln!(
        "Closing fields: {}",
        harness::compare(&built, &fin, 8).closing_fields()
    );
    pin_eq!(e.frame, 6822, "the closing block");
    // Item 2864, the first walk: 286 units compared, none apart, and no
    // building or city field-row — before the fix and after it. The
    // residue of the game's end is `1/106`'s `cavarch_o` and the
    // standing rows run883's widening holds, none a position.
    pin_eq!(e.compared, 286, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
}
