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

pub(crate) const LAKOTA: Pair = Pair {
    name: "EastIndiesLakotaPeace",
    map_style: 18,
    ai_tribe: 19,
    difficulty: 3,
    starting_technology: 8,
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
    count: 5225,
    sequence: 5225,
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
    pin_eq!(w.firsts.len(), 72, "initial run848 baseline");
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
    pin_eq!(w.firsts.len(), 87, "initial run849 baseline");
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
    pin_eq!(w.firsts.len(), 100, "initial run853 baseline");
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
    pin_eq!(w.firsts.len(), 128, "initial run855 baseline");
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
    pin_eq!(w.firsts.len(), 126, "initial run861 baseline");
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
    pin_eq!(w.firsts.len(), 3550, "initial run866 baseline");
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
}
