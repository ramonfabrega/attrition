//! **The first melee: a held Gunpowder age on Great Sahara, every seat a
//! computer** (item 3733, `docs/GOLDEN.md` §149; the shape is §155).
//! Great Sahara, `MAP_STYLE 7`, seed 12345, `--profile PLAYERS=2`: the idle
//! human Nubians (4) in seat 0, the British (11, London) as who 1 and the
//! Inca (2, Cuzco) as who 2, all three at war from frame 0 with no `war`
//! line; `STARTING_TECHNOLOGY 3`, `ENDING_TECHNOLOGY 3`,
//! `STARTING_RESOURCES 7`. `tools/gamelog/golden/melee-gunpowder.cmd`
//! grants each computer 100,000 of every resource on 600 and 602 and spawns
//! twelve of the age's types a side from 610, two frames apart, between the
//! two territories, with the age's two never-seen lines (Nubian) on who 0.
//!
//! Captures: run1412, the `DUMP_ALL` start; run1413, the staged game to its
//! end at `cover=0`, `end:UNITS=3,GUYS=2,DEATHS=1,AMMO=5` on a stride of
//! 100, walked from run1412's initial state with the script staged by
//! [`Built::tick`] ([`crate::diff::setup::with_staged_script`]).

use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

/// run1412: the lobby's `DUMP_ALL` start, 37 frames.
pub(crate) const MELEE_START: &str = "gamelog-run1412-greatsahara-gunpowder-melee-start.txt";

/// The melee's script, read from the tree.
pub(crate) fn melee_script() -> crate::golden::Script {
    let path = format!(
        "{}/../../tools/gamelog/golden/melee-gunpowder.cmd",
        env!("CARGO_MANIFEST_DIR")
    );
    crate::golden::Script::read(std::path::Path::new(&path))
        .unwrap_or_else(|e| panic!("tools/gamelog/golden/melee-gunpowder.cmd: {e}"))
}

/// The script's own line count: two `resource` lines, twelve `add` twins
/// and the two Nubian lines on who 0.
pub(crate) const MELEE_STAGED: usize = 28;

/// **The staging, walked before any capture holds it** (`docs/GOLDEN.md`
/// §3: a chapter's staging is walked first): run1412's start stood up with
/// the script staged by the harness's own tick, to frame
/// `RON_MELEE_STAGE` (default 700), printing where every `add` landed and
/// every line the interpreter carried and did not act on.
#[test]
fn gunpowder_melee_staging_walk() {
    let inst = install();
    let Some(inst) = inst else {
        return;
    };
    let Some(path) = dump(MELEE_START) else {
        eprintln!("skipping: no {MELEE_START} (set RON_GAMELOG_DIR)");
        return;
    };
    let script = melee_script();
    assert_eq!(
        script.lines().len(),
        MELEE_STAGED,
        "the melee's staged lines"
    );
    let to: i64 = std::env::var("RON_MELEE_STAGE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(700);
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let log = Log::parse(&text);
    let mut init = log.initial().expect("a start dump");
    let own = log.initial().expect("a start dump");
    borrow_from_siblings(&mut init, &[&own]);
    let mut built =
        crate::diff::setup::with_staged_script(script, || build_sim(&loaded, &init, Tuning::RON));
    while built.sim.frame <= to {
        built.tick();
    }
    let st = built.staged.as_ref().expect("the script is staged");
    eprintln!(
        "melee staged to {to}: {} line(s) ran, {} unit(s), {} building(s); {} carried and not acted on",
        st.applied.ran,
        st.applied.units,
        st.applied.buildings,
        st.applied.skipped_total()
    );
    for ((word, why), k) in &st.applied.skipped {
        eprintln!("  {k:5} {word}: {why}");
    }
    for line in &st.applied.landed {
        eprintln!("  {line}");
    }
    for (w, n) in built.sim.nation.iter().enumerate() {
        eprintln!(
            "  who={w} human={} age {}",
            n.human, built.sim.players[w].age
        );
    }
    assert_eq!(
        st.applied.skipped_total(),
        0,
        "every staged line reached the simulation"
    );
}

/// **The starting `Farms` list is the start dump's** (item 3757,
/// `docs/GOLDEN.md` §149.1): `Farms::inc_time` walks the array in index
/// order and spends one chance roll on every farm it does not skip, and
/// `Array<FarmStruct>::walk_data` prints the same array, so run1412's
/// frame-0 list — player 1's three farms, player 0's, player 2's — is the
/// order the rolls are spent in. The two-seat sort this replaced reads
/// 2, 1, 0 here.
#[test]
fn gunpowder_melee_s_farms_stand_in_the_dump_s_order() {
    let Some(inst) = install() else {
        return;
    };
    let Some(path) = dump(MELEE_START) else {
        eprintln!("skipping: no {MELEE_START} (set RON_GAMELOG_DIR)");
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let log = Log::parse(&text);
    let init = log.initial().expect("a start dump");
    let want: Vec<(i64, i64)> = init.farms.iter().map(|f| (f.who, f.o)).collect();
    assert_eq!(
        want.iter().map(|w| w.0).collect::<Vec<_>>(),
        [1, 1, 1, 0, 0, 0, 2, 2, 2],
        "run1412's list, as the dump prints it"
    );
    let built = build_sim(&loaded, &init, Tuning::RON);
    let got: Vec<(i64, i64)> = built
        .sim
        .farm_order
        .iter()
        .map(|&b| {
            let bd = &built.sim.buildings[b];
            (i64::from(bd.owner), i64::from(bd.index))
        })
        .collect();
    assert_eq!(got, want, "the Farms list the rolls are spent in");
}

/// run1413: the staged game, the draw stream to its end at `cover=0`, the
/// melee's records on a stride of 100.
pub(crate) const MELEE_LONG: (&str, &str) = (
    "gamelog-run1413-greatsahara-gunpowder-melee-trace.txt",
    "rontrace-run1413.log",
);

/// Where the melee's draw stream first parts (item 3733, first try).
pub(crate) struct Melee {
    pub length: i64,
    pub count: i64,
    pub sequence: i64,
}

pub(crate) const GUNPOWDER_MELEE: Melee = Melee {
    length: 2600,
    // Item 3733, first try: **182 of 2600**, the count and the sequence
    // together — before the script's first line on 600, in the unstaged
    // three-seat lobby. Ours 51 draws against 49: at index 40 ours spends
    // two `GameAccess::rnd+0x20 < Unit::do_job+0x67`, both the idle
    // human's Citizen `0/4`'s (TI 50, land, on a `Gather` of building 5,
    // `been_there`), and theirs goes straight to the `Guy::set_anim+0x97a`
    // ours draws third (`docs/GOLDEN.md` §149).
    //
    // Item 3757: **182 → 684**, both together. The starting `Farms` list
    // is ordered as the start dump prints it — player 1's farms, then
    // 0's, then 2's — not `start_of_game`'s higher-slot-first 2, 1, 0,
    // which handed the frame-37 sprout's chance roll to `1/2002` where
    // theirs' walk spends it two farms later; the cells parted from 37
    // and `0/2003`'s farmer re-picked on 182 alone (§149.1). Past the
    // script's lines on 600–660, all 28 staged alike.
    //
    // Item 3759: **684 → 759** (sequence; count 760). The human's Apedemak
    // Archers (TI 176, piece 126) loose from the bow hand, not the
    // figure's square — `launch::BAYS`' three piece-126 rows, off run1438's
    // packet — so `0/14`'s round of 674 leaves (16634, 15605, 640), flies
    // 10 frames against 11 and strikes the British Bombard `1/23` on 683,
    // whose fight takes `Unit::fight+0x9b0` on 684 as theirs does
    // (§149.2). On 759 who=2's Bombard `2/23` re-searches alike and then
    // takes `Guy::set_anim+0xf2f < Unit::set_anim+0xb6` twice where theirs
    // takes `< Guy::move+0x166` twice: index 1, 17 draws a side.
    //
    // Item 3762: **759 → 760** (sequence; the count stays 760). Theirs did
    // not move the Bombard: `< Guy::move+0x166` is the deferred attack
    // paid (`anim::SITE_ATTACK_STAND`). `2/23` fires on the Camel Raider
    // `0/17` through `do_attack_ground`, whose `Unit::set_angle(…, 0)`
    // leaves the two trackless crew's `des_angle` apart from their
    // `angle`, so `Guy::set_anim` defers their swing and guy 0's
    // `Guy::move` pays it on the same frame; `Unit::fight`'s strike
    // re-seats the figures first and so plays its crew at once
    // (`Movement::crew_des`, §149.3). On 760 ours spends 123 draws against
    // 122: index 40 is ours' `Army::find_target+0x7df`, who=2's army
    // `2/0` going status 17 → 18 on its own city `2/2000`, its muster
    // (7, 32) → (1, 36) — where ours' `2/23` walks by 801.
    //
    // Item 3763: **760 → 839** (sequence; count 840). Both armies release
    // on 760 (`num_standard` 6 against `pop_cap / 8`); theirs takes
    // `DEFENDING` (run1413b's coverage enters `Army::do_defending` there)
    // because its region word carries `& 4`, the war loop's difficulty-0
    // bit, and the loop runs only for a leader already met. Theirs meets
    // on 620's first strike (`Unit::set_attacking` writes `treaties` both
    // ways; no trace enters `Leader::meet`), and ours' strike did not
    // (`Sim::set_attacking`, §149.4). On 839 ours spends 17
    // draws a side and parts at index 4 on the human's `0/15`
    // (`Guy::set_anim+0xf2f < Guy::inc_time+0x271` against theirs'
    // `+0x97a < +0x1ed`); theirs spends `Ammo::do_damage+0xc59`/`+0xc7e`
    // at 7–8, which ours does not.
    //
    // Item 3767: **839 → 863**, both together. On 809 the archers' captain
    // `0/14` re-searches when `1/23` dies, and `Object::compare_target`
    // scores the pack-casting Bombard `2/23` past `i32`: the original's
    // `int` wraps negative and clamps to 9,999,999, so the tail reads
    // 100,000 (run1440's packet runs it: 100,000 for `2/23`, 278,027 for
    // the Elite Pikemen `2/18`, 194,000 for `2/21`), where ours carried
    // `i64` and read 7,027,388 — the squad took `2/23` where theirs took
    // `2/18` (run1440's window: `whom 2 ox 18` from block 810). On 863 ours
    // spends 13 draws against 14 and parts at index 5: theirs spends
    // `Unit::close+0xcb6`, `2/23`'s death, a frame before ours
    // (`docs/GOLDEN.md` §149.5).
    //
    // Item 3770: **863 → 947**, both together. The human's Camel Raider
    // `0/17` (TI 193, piece 143) is a pivot piece and looses through
    // `pivot::RELEASES`' four piece-143 rows, off run1440's packet: its
    // round of 859 leaves (17282, 15746, 742), not the figure's square
    // (17400, 15672, 477), flies 5 frames against 6 and strikes `2/23` on
    // 863 as theirs does (§149.6).
    //
    // Item 3779: **947 → 954**, both together. who=2's Inti Macemen
    // `2/29`, `2/30` and `2/31` (TI 92), a `DEFENSIVE` squad on `1/22`
    // since 913, keep their posts: `find_new_target`'s post arm writes the
    // post into the attack it re-finds or walks home to it, and `fight`'s
    // own leash (`005fe18c`) re-searches from `reach × 0xc0` off the post
    // and walks home on the same target — run1451's packet runs `2/31` on
    // 945 through both, and its window has the three `MOVEORDER`s to the
    // posts with `flags 8` on 946, 947 and 948 (§149.7). On 954 ours
    // spends 11 draws against 12 and parts at index 4: theirs spends
    // `Unit::close+0xcb6`, the British Elite Knight `1/22`'s death, a
    // frame before ours.
    //
    // Item 3781: **954 → 958**, both together. who=2's Arquebusiers (TI
    // 98, piece 752) loose from the gun, not the figure's square —
    // `launch::BAYS`' three piece-752 rows, off run1451's packet — so
    // `2/16`'s round of 953 at `1/22` leaves (15694, 16697, 779), flies 2
    // frames against 3 and lands in 954's round pass, after the Elite
    // Knight `2/22`'s blow of 954 has left `1/22` 7/8 of a point (§149.8).
    // On 958 ours spends 14 draws against 18 and parts at index 5: theirs
    // spends three `Guy::init_real+0x52` at 5–7, the figures of who=2's
    // government hero `2/28` re-made as it turns from The Monarch (TI 354)
    // to The Despot (TI 352), and a fourth `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271` at 10; ours turns it on the same frame and
    // spends none of them.
    //
    // Item 3783: **958 → 1001**, both together. The three draws were not
    // `2/28`'s: who=2's Senate turns it on 957 in both, and theirs' 958
    // `Build::finished < Unit::set_type` is **who=1's** Monarch `1/28`
    // (run1451's block 959: its crew at 0/0, TI 352). Ours' who=1 bought
    // Despotism 25 frames after who=2, as theirs, and finished on 982: the
    // research block's held-elsewhere term (`train_time`'s
    // `LAB_006513c0`, `TypeData::count_discovered`) takes a plain tech
    // another seat holds to `(num_players − n + 1) / (num_players + 1)`,
    // so on 958 who=1's 37,700 meets a target of 30,000 (§149.9). On 1001
    // ours spends 230 draws against 242 and parts at index 4: theirs
    // spends a third `Guy::set_anim+0x97a < Unit::do_idle+0x7d` where
    // ours goes on to `PathFinder::calc_road_cost+0x46`.
    //
    // Item 3784: **1001 → 1912** (sequence; count 1913), two builds. The
    // Dragoon `2/20` (TI 189), a `DEFENSIVE` attack on `1/18` with its post
    // (14808, 16968), drops the attack on 997 when its target is gone:
    // `fight`'s invalid-target branch (`LAB_005fdb9e`) is `find_new_target`
    // whole, whose post arm walks it home — 998..1001, theirs' angle
    // −1605566464 — and the idle on 1001 is theirs' third `do_idle` roll.
    // Then 1706 (ours 152 against 153): who=2's trained Arquebusiers squad
    // of 1568 took slot 23, the Bombard dead since 863, where theirs
    // numbers it `2/34`..`2/36` — `DeathObj::inc_time` bumps the number's
    // `hold_frames` once per figure's record, so a four-figure crew holds
    // its number long past the cull (§149.10). On 1912 ours spends 44
    // draws a side and parts at index 30: theirs spends three
    // `Guy::init_real+0x52 < Objects::init_unit`, a three-figure birth,
    // where ours goes on to `Guy::set_anim+0x97a < Guy::inc_time+0x271`.
    //
    // Item 3786: **1912 → 2160**, both together. The birth is who=1's
    // government hero: its Despot `1/28` (TI 352) dies on 1463 in both, and
    // `Unit::close` stamps `gov_hero_frame` 1463 + 30 × 15 = 1913, so the
    // Senate `1/2007`'s `Build::process` trains The Monarch (TI 354) into
    // slot 15 on 1912 (`Sim::gov_hero_respawn`, §149.11, no capture). On
    // 2160 ours spends 42 draws against 41 and parts at index 0: ours
    // spends `Leader::use_market+0x1ed` ahead of the `Leader::make_stuff
    // +0x221` both spend.
    //
    // Item 3788: **2160 → 2600, whole**, two builds. who=2 (the Inca)
    // sells on 2160 because its wealth reads 27 against theirs' 342; the
    // stockpile parts on run1461b's 860, where theirs' who=2 takes +16
    // food and +11 timber, and on 864 (+25 timber, +25 metal, the Bombard
    // `2/23`'s death): `take_damage`'s Inca refund, `INCA_REFUND` 25 % of
    // the dead unit's price back to its own leader. With it built the
    // buckets part on 1464 (+100 wealth): who=1's Despot `1/28` dies on
    // 1463, and a hero victim pays its killer `GENERAL_PLUNDER` 100. Both
    // built, who=1's and who=2's buckets agree on every frame of run1461b
    // and run1461 (`docs/GOLDEN.md` §149.12).
    count: 2600,
    sequence: 2600,
};

/// The melee's word: the lower of the two partings.
pub(crate) const GUNPOWDER_MELEE_WORD: i64 = if GUNPOWDER_MELEE.count < GUNPOWDER_MELEE.sequence {
    GUNPOWDER_MELEE.count
} else {
    GUNPOWDER_MELEE.sequence
};

/// **The melee's first parting**, walked from run1412's start with the
/// script staged by the harness's own tick.
#[test]
fn gunpowder_melee_first_parting() {
    let _pins = Pins::hold();
    let m = &GUNPOWDER_MELEE;
    let Some(w) = crate::diff::setup::with_staged_script(melee_script(), || {
        third::walk_from(MELEE_START, MELEE_LONG)
    }) else {
        return;
    };
    eprintln!(
        "GUNPOWDER_MELEE: count {} sequence {} of {}; {}",
        w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.last, m.length, "the game's last frame");
    pin_eq!(w.count, m.count, "the draw count's first parting");
    pin_eq!(w.sequence, m.sequence, "the draw sequence's first parting");
}

/// **The word's window** (item 3733, moved by 3757, 3759, 3762, 3763, 3767,
/// 3770, 3779, 3781, 3783, 3784 and 3786): run1413's stride blocks
/// 2101..2401 — the block before the word's frame 2160 through 250 frames
/// past it. 3733's was 101..432, on 182; 3757's 601..934, on 684; 3759's
/// 701..1009, on 759; 3762's 701..1010, on 760; 3763's 801..1089, on 839;
/// 3767's 801..1113, on 863; 3770's 901..1197, on 947; 3779's 901..1204,
/// on 954; 3781's 901..1208, on 958; 3783's 901..1251, on 1001; 3784's
/// 1901..2162, on 1912; 3786's 2101..2410, on 2160. Item 3788 closes the
/// game: the window is 2101 through the closing block 2601.
pub(crate) const WIDENING_GUNPOWDER_MELEE: (i64, i64) = (2101, 2601);

/// run1413's blocks over [`WIDENING_GUNPOWDER_MELEE`], walked from
/// run1412's start with the script staged and every round compared.
pub(crate) fn gunpowder_melee_word_window() -> Option<harness::tests::Widened> {
    crate::diff::setup::with_staged_script(melee_script(), || {
        harness::tests::widen_on_siblings_with_rounds(
            &[MELEE_START],
            true,
            MELEE_LONG,
            "run1413",
            &[(MELEE_LONG.0, WIDENING_GUNPOWDER_MELEE.0)],
            WIDENING_GUNPOWDER_MELEE,
            1,
            &[2601],
            true,
        )
    })
}

#[test]
fn gunpowder_melee_run1413_s_word_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = gunpowder_melee_word_window() else {
        return;
    };
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
        eprintln!("  rounds read {:?}", w.rounds_read);
        for ((who, o, what), (f, r)) in &w.rounds {
            eprintln!("  round {f} {who}/{o} {what}: {r}");
        }
        for (b, rows) in &w.standing {
            for ((who, o, what), r) in rows {
                eprintln!("  standing {b} {who}/{o} {what}: {r}");
            }
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3733, first try: **224** keys over the four stride blocks 101,
    // 201, 301 and 401. Block 101 carries the standing families alone —
    // `form` on every unit of the three seats (ours −1 against theirs 9,
    // 0 on `0/0`), `build:extra` on every building and the two computers'
    // first groups' `held` and pools. **The word's own unit parts first**:
    // on 201 the human's Citizen `0/4` (TI 50) stands at (39096, 36408) in
    // ours against (39288, 36600) in theirs, `heading`/`dest_angle`
    // −473038848 against 1181417472 and `mirror` 1 against 0 — ours a
    // gather leg ahead, where theirs' 0/4 stands on 301 — the two
    // `Unit::do_job+0x67` draws ours spends on 182. From 301 the parting
    // fans out to `0/3` and who=1's citizens (`1/0`'s move order).
    //
    // Item 3757, on the word moved to 684 (window 601..934): **818** keys.
    // Block 601 carries the standing families alone — the staging's 28
    // lines and the squads stand alike. **On 701 the fight parts a frame
    // late in ours**: who=1's Bombard `1/23` reads `damage_frame` 684
    // against theirs' 683, `pos` (15768, 14616) against (15816, 14664),
    // `path:length` 3 against 4 and `spell_time` 15 against 16; `2/17`
    // dies on 696 against 695, `1/14` is dead in theirs alone, and
    // `0/4`'s next re-pick reads the shifted stream. 801 and 901 fan out.
    //
    // Item 3759, on the word moved to 759 (window 701..1009): **758** keys,
    // rounds read (29, 30), 31 parting (9 on 801, 1 on 901, 21 on 1001).
    // Block 701 carries the standing families alone: piece 126's bays put
    // `0/14`'s round of 674 down on 683, and `1/23`, `2/17` and `1/14`
    // agree. **On 801 who=2's Bombard `2/23`, the word's unit, walks
    // another road**: its move order's goal (1320, 28152) in ours against
    // (5736, 24792), `path:length` 24 against 13, `hits:damage` 97 against
    // 87 — and 801 fans out over 26 units of the three seats.
    //
    // Item 3762, on the word moved to 760 (window 701..1010): the same
    // **758** keys, row for row — the crew's deferred swing on 759 writes
    // nothing a stride block prints, and 801's `2/23` rows stand as 3759
    // left them: the goal (1320, 28152) against (5736, 24792) is the muster
    // ours' army `2/0` moves on 760.
    //
    // Item 3763, on the word moved to 839 (window 801..1089): **427** keys,
    // rounds read (23, 26), 23 parting. **Block 801 carries the standing
    // families alone** — `2/23`'s goal (1320, 28152) against (5736, 24792)
    // is gone with the march: both armies hold their musters. On 901 the
    // word's unit parts first — the human's `0/15` reads `mirror` 0 in ours
    // against 1 — and the fight around it: `2/23` dies on 864 against 863
    // (`death:cur_anim` 17 against 18), `0/14`'s `mirror` 1 against 0, and
    // the two computers' squads' hits.
    //
    // Item 3767, on the word moved to 863 (window 801..1113, four blocks):
    // **669** keys, rounds read (27, 30), 22 parting. Block 801 carries the
    // standing families alone, as before. **On 901 the archers' `mirror`
    // rows are gone** — `0/14`, `0/15` and `0/16` take `2/18` on 809 as
    // theirs does — and the word's own row leads: `2/23`'s
    // `death:first_frame` 864 against 863 (`death:cur_anim` 17 against
    // 18), then `2/18`'s death roll (865 both, `cur_anim` 18 against 17),
    // `2/22`'s attack and `1/0`'s and `2/5`'s moves. 1001 and 1101 fan out.
    //
    // Item 3770, on the word moved to 947 (window 901..1197, three blocks):
    // **537** keys, rounds read (17, 17), 14 parting. **Block 901 carries
    // the standing families alone** — `2/23`'s `death:first_frame` reads
    // 863 both sides, the Camel Raider's round of 859 landing as theirs.
    // On 1001 the human's citizens `0/3` and `0/5` stand apart, `0/17`'s
    // `damage_frame` reads 1000 against 991, `1/16`'s order 10 against 1,
    // and `1/18` dies on 987 against 983; 1101 fans out.
    //
    // Item 3779, on the word moved to 954 (window 901..1204, four blocks):
    // **645** keys, rounds read (36, 38), 84 parting — block 1201 joins.
    // Block 901 still carries the standing families alone. **On 1001 the
    // three Inti Macemen's rows are gone** (11, 12 and 10 before): `2/29`
    // stands on its post (21624, 18168), `2/30` on (21720, 18264) and
    // `2/31` on (21624, 18360) both sides, tolerance 0, where ours had them
    // idle at (21312, 19171), (20394, 19290) and (20150, 19350). The word's
    // own row leads: `1/22`'s `death:first_frame` 955 against 954
    // (`cur_anim` 18 against 17); then who=2's `2/19`, `2/20`, `2/22` and
    // the human's citizens; 1101 and 1201 fan out.
    //
    // Item 3781, on the word moved to 958 (window 901..1208, the same four
    // blocks): **685** keys, rounds read (36, 43), 113 parting. Block 901
    // still carries the standing families alone. **On 1001 `1/22`'s death
    // rows are gone** — `2/16`'s round of 953 leaves piece 752's bay and
    // lands on 954, so `death:first_frame` reads 954 both sides. The block
    // reads 176 rows against 81: the units that had held `1/22` — the
    // Arquebusiers `2/14`..`2/16`, the Macemen `2/29`..`2/31`, `2/22` and
    // `2/19` — hold moves in ours where theirs stands or walks home, and
    // every one agrees frame for frame with run1451's window through 958,
    // the word: the fan-out is the stream past it.
    //
    // Item 3783, on the word moved to 1001 (window 901..1251, the same four
    // blocks): **660** keys, rounds read (36, 30), 82 parting. Block 901
    // still carries the standing families alone. **1001 reads 16 rows
    // against 176**: who=1's Despot `1/28` is re-made on 958, as run1451's
    // block 959 prints it — TI 352, `cur_anim` 1, `gpiece` 302/12974/25646,
    // the crew at (17856, 13275) and (18008, 13395), clocks 0/60, 0/0, 0/0,
    // where ours had The Monarch at 1/30 and `gpiece` 304/12976/25648 —
    // and the stream from 958 is theirs. What parts on 1001 is who=2's
    // Dragoon `2/20`: `pos` (14856, 16920) against (14808,
    // 16968), `idle` 2 against 0, `cavarch_who` 0 against −1; `2/19`'s
    // attack `def_x`/`def_y`; and the two `build:extra` rows.
    //
    // Item 3784, on the word moved to 1912 (window 1901..2162, three
    // blocks): **1126** keys, rounds read (0, 0), none parting. Walked back
    // from 1912 over 901..2162: 901..1801 carry the standing families
    // alone. **On 1001 `2/20` stands on its post** (14808, 16968) both
    // sides, where ours stood at (14856, 16920) with `idle` 2 against 0;
    // the old window 901..1251 reads 99 keys, rounds (36, 36), none
    // parting. **On 1601 who=2's squad is `2/34`..`2/36`** both sides, at
    // (7512, 34296), (7512, 34440) and (7656, 34392), where ours had its
    // captain in slot 23 (`2/23 extra`) and `2/34`/`2/35` on the members'
    // squares. Block 1901 carries the standing families alone; 2001 and
    // 2101 fan out (the human's `0/3`, who=1's `1/11` and `1/15`).
    //
    // Item 3786, on the word moved to 2160 (window 2101..2410, four
    // blocks): **532** keys, rounds read (0, 0), none parting. The old
    // window 1901..2162 reads **121** keys (1126 before): **on 2001 `1/15`
    // is The Monarch** (TI 354, uid 81, three figures, 109 hits at (26245,
    // 7585)) both sides, where ours had a Citizen (`guys.len` 1, 50 hits);
    // 2001 keeps nine `build:extra` rows and who=2's group 133 `held`, and
    // the human's `0/3` and who=1's `1/11` agree. Block 2101 carries the
    // standing families alone (`form`, `build:extra`, the groups' `held`
    // and pools); 2201 is the first to part a unit — the human's `0/5`,
    // who=1's `1/0`, `1/5` and `1/6` on their move orders — and 2301 and
    // 2401 fan out.
    //
    // Item 3788, the game whole (window 2101..2601, six blocks with the
    // closing block): **160** keys, rounds read (0, 0), none parting. The
    // old window 2101..2410 reads 131 keys (532 before): block 2201's move
    // orders on `0/5`, `1/0`, `1/5` and `1/6` are gone. 2101 carries the
    // standing families alone (`form`, `build:extra`); 2201 adds the groups'
    // `held` and who=2's Supply Wagon `2/24` (TI 63) at `myhits` 90 against
    // 108 — theirs' rises on run1461's logger frame 2135 with nothing else
    // on the frame, and who=1's `1/24` stays 90 in both (parked 3788a); the
    // closing block 2601 has the human's citizens `0/1`..`0/7` holding a
    // one-step order in ours where theirs' are idle with none, and `0/3`
    // at (38136, 36600) against (38119, 36586) (parked 3788b).
    pin_eq!(w.blocks, 6, "every stride block in the window");
    pin_eq!(w.firsts.len(), 160, "run1413's word window");
    pin_eq!(w.rounds_read, (0, 0), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// **A strike is first contact** (item 3763, `docs/GOLDEN.md` §149.4).
/// `Unit::set_attacking@005ff5b0` ORs the met bit into both leaders'
/// `treaties`; run1413b enters it first on 620 and never enters
/// `Leader::meet`. `plan_strategy`'s `REVEAL_MAP` step 6 walks buildings
/// and walls only, and on 750 who=2 has seen none of the British ones, so
/// the strike is what meets. Step 15's war loop then gives the region word
/// the difficulty-0 `& 4`, and on 760 army `2/0` releases into
/// `DEFENDING`, not `MARCHING`, and spends no `Army::find_target` —
/// theirs' 760, draw for draw.
#[test]
fn gunpowder_melee_s_strike_meets_before_the_census() {
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(sib), Some(tr)) = (
        dump(MELEE_LONG.0),
        dump(MELEE_START),
        crate::diff::testkit::trace(MELEE_LONG.1),
    ) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", MELEE_LONG.0);
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let sib_text = crate::capture::read(&sib);
    let log = Log::parse(&text);
    let sib_log = Log::parse(&sib_text);
    let sib_init = sib_log.initial().expect("a start dump");
    let mut init = log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sib_init]);
    crate::diff::setup::borrow_pasture(&mut init, &tr);
    let mut built = crate::diff::setup::with_staged_script(melee_script(), || {
        build_sim(&loaded, &init, Tuning::RON)
    });
    built.sim.trace_phases = true;
    assert_ne!(built.sim.lobby.reveal_map, 0, "the lobby is REVEAL_MAP 1");
    // The frame each pair's met bit lands on: the tick that processes it.
    let mut first = [[None::<i64>; 3]; 3];
    while built.sim.frame < 751 {
        let f = built.sim.frame;
        built.tick();
        for (a, row) in first.iter_mut().enumerate() {
            for (b, at) in row.iter_mut().enumerate() {
                if a != b && at.is_none() && built.sim.has_met(a as u8, b) {
                    *at = Some(f);
                }
            }
        }
    }
    assert_eq!(
        first[1][2],
        Some(620),
        "the British and the Inca meet on 620's strike"
    );
    assert_eq!(first[2][1], Some(620), "both ways");
    assert_eq!(
        (first[0][1], first[0][2]),
        (Some(659), Some(661)),
        "the staged Nubians meet each computer as it strikes them"
    );
    assert!(
        !(0..built.sim.buildings.len()).any(|b| {
            let bd = &built.sim.buildings[b];
            bd.owner != 2 && bd.alive && built.sim.build_is_seen(b, 2)
        }),
        "who=2's 750 census sees no other seat's building: step 6 meets nobody"
    );
    const WORD: i64 = 760;
    while built.sim.frame < WORD {
        built.tick();
    }
    built.tick();
    let (f, ours) = built.frame_sites.last().expect("760's draws");
    assert_eq!(*f, WORD);
    assert!(
        !ours.iter().any(|l| l == sim::army::SITE_FIND_TARGET_SCORE),
        "army 2/0 does not march on 760"
    );
    assert_eq!(*ours, tr.labels(WORD), "760's whole draw sequence");
    let a = &built.sim.armies[2].list[0];
    assert_eq!(
        a.status & sim::army::status::MARCHING,
        0,
        "army 2/0 is released, not marching"
    );
}

/// **The captain's search wraps as the original's `int` does** (item 3767,
/// `docs/GOLDEN.md` §149.5). On 809 the archers' captain `0/14` (TI 176)
/// re-searches when `1/23` dies. `Object::compare_target@0064e5c0` holds
/// its value in a 32-bit `int`: for the pack-casting Bombard `2/23` the
/// threat term and the caster arm's `× 20` carry it past 2³¹, it wraps
/// negative, the tail clamps it to 9,999,999 and returns 100,000. run1440's
/// packet (logger frame 809) runs the original on this very searcher:
/// **100,000** for `2/23`, **278,027** for the Elite Pikemen `2/18`,
/// **194,000** for the Elite Light Cavalry `2/21`. So the squad takes
/// `2/18`, as run1440's window prints from block 810 (`whom 2 ox 18`).
#[test]
fn gunpowder_melee_s_captain_scores_the_bombard_in_an_int() {
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(sib), Some(tr)) = (
        dump(MELEE_LONG.0),
        dump(MELEE_START),
        crate::diff::testkit::trace(MELEE_LONG.1),
    ) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", MELEE_LONG.0);
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let sib_text = crate::capture::read(&sib);
    let log = Log::parse(&text);
    let sib_log = Log::parse(&sib_text);
    let sib_init = sib_log.initial().expect("a start dump");
    let mut init = log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sib_init]);
    crate::diff::setup::borrow_pasture(&mut init, &tr);
    let mut built = crate::diff::setup::with_staged_script(melee_script(), || {
        build_sim(&loaded, &init, Tuning::RON)
    });
    while built.sim.frame < 809 {
        built.tick();
    }
    let unit = |sim: &sim::Sim, who: u8, o: usize| {
        sim.units
            .iter()
            .position(|u| u.alive() && u.owner == who && u.index as usize == o)
            .unwrap_or_else(|| panic!("{who}/{o} on 809"))
    };
    let at = |u| sim::combat::Obj::Unit(u);
    let captain = at(unit(&built.sim, 0, 14));
    let pikemen = at(unit(&built.sim, 2, 18));
    for (o, want, what) in [
        (23, 100_000, "2/23: the int wraps and clamps"),
        (18, 278_027, "2/18, as run1440's packet"),
        (21, 194_000, "2/21, as run1440's packet"),
    ] {
        let t = at(unit(&built.sim, 2, o));
        assert_eq!(
            built.sim.compare_target(captain, t, true, false),
            want,
            "{what}"
        );
    }
    built.tick();
    let squad: Vec<usize> = (14..=16).map(|o| unit(&built.sim, 0, o)).collect();
    for (o, u) in (14..=16).zip(squad) {
        assert_eq!(
            built.sim.units[u].combat.target,
            Some(pikemen),
            "0/{o} takes 2/18 on 809, as run1440's block 810"
        );
    }
}

/// **The Camel Raider looses from its pivot** (item 3770, `docs/GOLDEN.md`
/// §149.6). The human's Camel Raider `0/17` (TI 193, piece 143) is a pivot
/// piece: its round leaves through `get_position`'s pivot branch, the pivot
/// node at the figure's facing plus the release node at the facing and the
/// turret. Its round of 859 at the Inca Bombard `2/23` leaves (17282,
/// 15746) at 742 — `CHAR_ATTACK3` at turret step 4, the point run1440's
/// window prints for the round of 799 — flies 5 frames, and `2/23` dies on
/// 863 as theirs does (run1413's block 901, `death:first_frame` 863). From
/// its own square (17400, 15672) at 477 the round flew 6 and `2/23` died
/// on 864.
#[test]
fn gunpowder_melee_s_camel_raider_looses_from_its_pivot() {
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(sib), Some(tr)) = (
        dump(MELEE_LONG.0),
        dump(MELEE_START),
        crate::diff::testkit::trace(MELEE_LONG.1),
    ) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", MELEE_LONG.0);
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let sib_text = crate::capture::read(&sib);
    let log = Log::parse(&text);
    let sib_log = Log::parse(&sib_text);
    let sib_init = sib_log.initial().expect("a start dump");
    let mut init = log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sib_init]);
    crate::diff::setup::borrow_pasture(&mut init, &tr);
    let mut built = crate::diff::setup::with_staged_script(melee_script(), || {
        build_sim(&loaded, &init, Tuning::RON)
    });
    let unit = |sim: &sim::Sim, who: u8, o: usize| {
        sim.units
            .iter()
            .position(|u| u.alive() && u.owner == who && u.index as usize == o)
    };
    while built.sim.frame < 859 {
        built.tick();
    }
    let raider = sim::combat::Obj::Unit(unit(&built.sim, 0, 17).expect("0/17 on 859"));
    let bombard = sim::combat::Obj::Unit(unit(&built.sim, 2, 23).expect("2/23 on 859"));
    let find = |sim: &sim::Sim| {
        sim.projectiles
            .iter()
            .find(|p| p.shooter == raider && p.target == Some(bombard))
            .copied()
    };
    while find(&built.sim).is_none() && built.sim.frame < 862 {
        built.tick();
    }
    let round = find(&built.sim).expect("0/17's round at 2/23 by 861");
    assert_eq!(
        ((round.launch.x, round.launch.y), round.sz, round.total_time),
        ((17282, 15746), 742, 5),
        "the pivot's point, and five frames"
    );
    while built.sim.frame < 863 {
        built.tick();
    }
    assert!(unit(&built.sim, 2, 23).is_some(), "2/23 stands into 863");
    built.tick();
    assert!(
        unit(&built.sim, 2, 23).is_none(),
        "2/23 dies on 863, as run1413's block 901"
    );
}

/// **The Arquebusiers loose from the gun** (item 3781, `docs/GOLDEN.md`
/// §149.8). who=2's Arquebusiers `2/16` (TI 98, piece 752) fire on the
/// British Elite Knight `1/22` on 953 at `CHAR_ATTACK2`'s frame 25: the
/// round leaves piece 752's bay, (15694, 16697) at 779 — run1451's window
/// prints it there on 954 — and flies 2 frames. `2/22`'s blow of 954 leaves
/// `1/22` at damage 134 and 2/16 of its 135 on both sides, and the round,
/// stepped after the units, kills it on 954 as theirs does (`Unit::close
/// +0xcb6` at index 4). From its own square (15576, 16776) at 628 the round
/// flew 3 and `1/22` died on 955.
#[test]
fn gunpowder_melee_s_arquebusiers_loose_from_the_gun() {
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(sib), Some(tr)) = (
        dump(MELEE_LONG.0),
        dump(MELEE_START),
        crate::diff::testkit::trace(MELEE_LONG.1),
    ) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", MELEE_LONG.0);
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let sib_text = crate::capture::read(&sib);
    let log = Log::parse(&text);
    let sib_log = Log::parse(&sib_text);
    let sib_init = sib_log.initial().expect("a start dump");
    let mut init = log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sib_init]);
    crate::diff::setup::borrow_pasture(&mut init, &tr);
    let mut built = crate::diff::setup::with_staged_script(melee_script(), || {
        build_sim(&loaded, &init, Tuning::RON)
    });
    let unit = |sim: &sim::Sim, who: u8, o: usize| {
        sim.units
            .iter()
            .position(|u| u.alive() && u.owner == who && u.index as usize == o)
    };
    while built.sim.frame < 953 {
        built.tick();
    }
    let gun = sim::combat::Obj::Unit(unit(&built.sim, 2, 16).expect("2/16 on 953"));
    let knight = sim::combat::Obj::Unit(unit(&built.sim, 1, 22).expect("1/22 on 953"));
    let find = |sim: &sim::Sim| {
        sim.projectiles
            .iter()
            .find(|p| p.shooter == gun && p.target == Some(knight))
            .copied()
    };
    while find(&built.sim).is_none() && built.sim.frame < 954 {
        built.tick();
    }
    let round = find(&built.sim).expect("2/16's round at 1/22 on 953");
    assert_eq!(
        ((round.launch.x, round.launch.y), round.sz, round.total_time),
        ((15694, 16697), 779, 2),
        "piece 752's bay, and two frames"
    );
    while built.sim.frame < 954 {
        built.tick();
    }
    assert!(unit(&built.sim, 1, 22).is_some(), "1/22 stands into 954");
    built.tick();
    assert!(
        unit(&built.sim, 1, 22).is_none(),
        "1/22 dies on 954, as run1413's block 1001"
    );
}

/// **The second Despot is researched at the held-elsewhere rate** (item
/// 3783, `docs/GOLDEN.md` §149.9). Both Senates buy Despotism 25 frames
/// apart, the cadence's own phase, at 100 a frame of 40,000. who=2's
/// finishes on 957 and re-makes The Monarch `2/28`; who=1's, at 37,700 on
/// 958, then reads `train_time`'s `LAB_006513c0` — one seat of three holds
/// it, `3 × 40000 / 4` — and finishes there, re-making `1/28` as The
/// Despot with three `Guy::init_real` draws: run1451's block 959 has its
/// crew at 0/0 and `gpiece` 12974 and 25646. Without the term ours ran on
/// to 982.
#[test]
fn gunpowder_melee_s_second_despot_is_researched_at_the_held_elsewhere_rate() {
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(sib), Some(tr)) = (
        dump(MELEE_LONG.0),
        dump(MELEE_START),
        crate::diff::testkit::trace(MELEE_LONG.1),
    ) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", MELEE_LONG.0);
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let sib_text = crate::capture::read(&sib);
    let log = Log::parse(&text);
    let sib_log = Log::parse(&sib_text);
    let sib_init = sib_log.initial().expect("a start dump");
    let mut init = log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sib_init]);
    crate::diff::setup::borrow_pasture(&mut init, &tr);
    let mut built = crate::diff::setup::with_staged_script(melee_script(), || {
        build_sim(&loaded, &init, Tuning::RON)
    });
    let ti = |sim: &sim::Sim, who: u8| {
        sim.units
            .iter()
            .find(|u| u.alive() && u.owner == who && u.index == 28)
            .map(|u| u.type_index)
    };
    while built.sim.frame < 958 {
        built.tick();
    }
    assert_eq!(
        (ti(&built.sim, 2), ti(&built.sim, 1)),
        (Some(352), Some(354)),
        "2/28 is The Despot from 957, 1/28 still The Monarch"
    );
    built.tick();
    assert_eq!(ti(&built.sim, 1), Some(352), "1/28 is The Despot on 958");
    let u = built
        .sim
        .units
        .iter()
        .find(|u| u.alive() && u.owner == 1 && u.index == 28)
        .unwrap();
    assert_eq!(
        u.guys
            .iter()
            .map(|g| (g.gpiece, g.cur_time, g.end_time))
            .collect::<Vec<_>>(),
        [(302, 0, 60), (12974, 0, 0), (25646, 0, 0)],
        "run1451's block 959, figure for figure"
    );
}

/// **The Inti Macemen walk home to their posts** (item 3779, `docs/GOLDEN.md`
/// §149.7). who=2's Inti Macemen `2/29`, `2/30` and `2/31` (TI 92, land,
/// `DEFENSIVE`) take a defensive attack on the British Elite Knight `1/22`
/// on 913, each with its post — where it stood — in the attack. Strayed
/// past `max(respond range, max range) × 0xc0` from it, each re-searches
/// and goes home: `find_new_target`'s post arm keeps the post in the attack
/// it re-finds, and `fight`'s own leash (`005fe18c`) walks the unit home
/// when the re-find is the same target. run1451's window has the three
/// `MOVEORDER`s to the posts, `flags 8`, on blocks 946 (`2/31`), 947
/// (`2/30`) and 948 (`2/29`); run1413's block 1001 has all three on them.
/// Without the arms, `2/29` stood idle where it was and `2/30`'s post moved
/// with it.
#[test]
fn gunpowder_melee_s_inti_macemen_walk_home_to_their_posts() {
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(sib), Some(tr)) = (
        dump(MELEE_LONG.0),
        dump(MELEE_START),
        crate::diff::testkit::trace(MELEE_LONG.1),
    ) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", MELEE_LONG.0);
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let sib_text = crate::capture::read(&sib);
    let log = Log::parse(&text);
    let sib_log = Log::parse(&sib_text);
    let sib_init = sib_log.initial().expect("a start dump");
    let mut init = log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sib_init]);
    crate::diff::setup::borrow_pasture(&mut init, &tr);
    let mut built = crate::diff::setup::with_staged_script(melee_script(), || {
        build_sim(&loaded, &init, Tuning::RON)
    });
    let unit = |sim: &sim::Sim, o: usize| {
        sim.units
            .iter()
            .position(|u| u.alive() && u.owner == 2 && u.index as usize == o)
            .expect("the Maceman stands")
    };
    let home = |sim: &sim::Sim, o: usize| {
        let u = unit(sim, o);
        match sim.units[u].orders.front() {
            Some(sim::orders::Order {
                flags,
                body: sim::orders::Body::Move(m),
            }) if flags & sim::orders::flag::POST != 0 => Some((m.dest.x, m.dest.y)),
            _ => None,
        }
    };
    const POSTS: [(usize, (i32, i32), i64); 3] = [
        (31, (21624, 18360), 946),
        (30, (21720, 18264), 947),
        (29, (21624, 18168), 948),
    ];
    while built.sim.frame < 945 {
        built.tick();
    }
    for (o, _, _) in POSTS {
        assert_eq!(home(&built.sim, o), None, "2/{o} still on its attack, 945");
    }
    for (o, post, block) in POSTS {
        while built.sim.frame < block {
            built.tick();
        }
        assert_eq!(
            home(&built.sim, o),
            Some(post),
            "2/{o} walks home on block {block}, as run1451's window"
        );
    }
}

/// **The Dragoon walks home when its target dies** (item 3784,
/// `docs/GOLDEN.md` §149.10). who=2's Dragoon `2/20` (TI 189, land,
/// `DEFENSIVE`) fires on `1/18` from its post (14808, 16968), closes to
/// (14856, 16920) on 964, and loses its target on 997 (`1/18` dead on
/// 983). `Unit::fight`'s invalid-target branch (`LAB_005fdb9e`) is
/// `find_new_target` whole, and its post arm walks the unit home: run1413's
/// block 1001 has it on the post, `angle` −1605566464 — the heading of
/// the step (−48, +48). Ours dropped the attack and stood where it was.
#[test]
fn gunpowder_melee_s_dragoon_walks_home_when_its_target_dies() {
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(sib), Some(tr)) = (
        dump(MELEE_LONG.0),
        dump(MELEE_START),
        crate::diff::testkit::trace(MELEE_LONG.1),
    ) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", MELEE_LONG.0);
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let sib_text = crate::capture::read(&sib);
    let log = Log::parse(&text);
    let sib_log = Log::parse(&sib_text);
    let sib_init = sib_log.initial().expect("a start dump");
    let mut init = log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sib_init]);
    crate::diff::setup::borrow_pasture(&mut init, &tr);
    let mut built = crate::diff::setup::with_staged_script(melee_script(), || {
        build_sim(&loaded, &init, Tuning::RON)
    });
    let dragoon = |sim: &sim::Sim| {
        let u = sim
            .units
            .iter()
            .find(|u| u.alive() && u.owner == 2 && u.index == 20)
            .expect("the Dragoon stands");
        ((u.pos.x, u.pos.y), u.movement.heading.0)
    };
    while built.sim.frame < 997 {
        built.tick();
    }
    assert_eq!(dragoon(&built.sim).0, (14856, 16920), "closed on 1/18, 997");
    while built.sim.frame < 1001 {
        built.tick();
    }
    assert_eq!(
        dragoon(&built.sim),
        ((14808, 16968), -1605566464),
        "home on its post, run1413's block 1001"
    );
}

/// **A dead crew holds its number past the cull** (item 3784,
/// `docs/GOLDEN.md` §149.10). who=2's Bombard `2/23` dies on 863 with four
/// `DeathObj` records, each bumping the number's `hold_frames` a frame
/// while `process_all` takes one off. Its records end by 1601, but the
/// hold they built keeps 23 taken on 1568, when who=2's Arquebusiers
/// squad (TI 98) is born: run1413 numbers it `2/34`..`2/36`. One bump a
/// frame freed 23 about thirty frames after the cull, and the captain took
/// it.
#[test]
fn gunpowder_melee_s_arquebusiers_squad_skips_the_held_bombard_number() {
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(sib), Some(tr)) = (
        dump(MELEE_LONG.0),
        dump(MELEE_START),
        crate::diff::testkit::trace(MELEE_LONG.1),
    ) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", MELEE_LONG.0);
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let sib_text = crate::capture::read(&sib);
    let log = Log::parse(&text);
    let sib_log = Log::parse(&sib_text);
    let sib_init = sib_log.initial().expect("a start dump");
    let mut init = log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sib_init]);
    crate::diff::setup::borrow_pasture(&mut init, &tr);
    let mut built = crate::diff::setup::with_staged_script(melee_script(), || {
        build_sim(&loaded, &init, Tuning::RON)
    });
    let squad = |sim: &sim::Sim| {
        let mut o: Vec<i16> = sim
            .units
            .iter()
            .filter(|u| u.alive() && u.owner == 2 && u.type_index == 98)
            .map(|u| u.index)
            .collect();
        o.sort_unstable();
        o
    };
    while built.sim.frame < 1567 {
        built.tick();
    }
    let before = squad(&built.sim);
    while built.sim.frame < 1569 {
        built.tick();
    }
    let born: Vec<i16> = squad(&built.sim)
        .into_iter()
        .filter(|o| !before.contains(o))
        .collect();
    assert_eq!(born, [34, 35, 36], "run1413's squad of 1568");
    assert!(
        !built
            .sim
            .units
            .iter()
            .any(|u| u.alive() && u.owner == 2 && u.index == 23),
        "23 still held by the Bombard's records"
    );
}

/// **who=1's dead Despot is reborn at its Senate as The Monarch** (item
/// 3786, `docs/GOLDEN.md` §149.11). `1/28` (TI 352) dies on 1463, and
/// `Unit::close` stamps `gov_hero_frame = GOV_HERO_RESPAWN × 15 + frame`,
/// 1913; `Build::process` trains `get_gov_hero`'s type at the Senate
/// `1/2007` on the frame the stamp is one ahead of, 1912 — run1413's three
/// `Guy::init_real < Unit::init < Objects::init_unit` there, and its block
/// 2001's `1/15`, uid 81, TI 354, three figures, where ours had held the
/// slot for a Citizen. Its closing block reads who=1's `gov_hero_frame`
/// 1912, `Unit::init`'s own stamp.
#[test]
fn gunpowder_melee_s_dead_despot_is_reborn_at_its_senate() {
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(sib), Some(tr)) = (
        dump(MELEE_LONG.0),
        dump(MELEE_START),
        crate::diff::testkit::trace(MELEE_LONG.1),
    ) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", MELEE_LONG.0);
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let sib_text = crate::capture::read(&sib);
    let log = Log::parse(&text);
    let sib_log = Log::parse(&sib_text);
    let sib_init = sib_log.initial().expect("a start dump");
    let mut init = log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sib_init]);
    crate::diff::setup::borrow_pasture(&mut init, &tr);
    let mut built = crate::diff::setup::with_staged_script(melee_script(), || {
        build_sim(&loaded, &init, Tuning::RON)
    });
    let hero = |sim: &sim::Sim, o: i16| {
        sim.units
            .iter()
            .find(|u| u.alive() && u.owner == 1 && u.index == o)
            .map(|u| u.type_index)
    };
    while built.sim.frame < 1463 {
        built.tick();
    }
    assert_eq!(hero(&built.sim, 28), Some(352), "the Despot stands to 1463");
    while built.sim.frame < 1465 {
        built.tick();
    }
    assert_eq!(hero(&built.sim, 28), None, "and is gone after it");
    assert_eq!(built.sim.tech[1].gov_hero_frame, 1913, "1463 + 30 × 15");
    while built.sim.frame < 1912 {
        built.tick();
    }
    assert_eq!(hero(&built.sim, 15), None, "nobody in 15 before 1912");
    while built.sim.frame < 1913 {
        built.tick();
    }
    assert_eq!(
        hero(&built.sim, 15),
        Some(354),
        "The Monarch, run1413's 1/15"
    );
    assert_eq!(
        built.sim.tech[1].gov_hero_frame, 1912,
        "run1413's closing block"
    );
}

/// **The Bombard's ground swing waits on its crew's turn** (item 3762,
/// `docs/GOLDEN.md` §149.3). On 759 who=2's Bombard `2/23` (TI 267, land)
/// re-searches (`Unit::fight+0x9b0`) and fires on the Camel Raider `0/17`
/// through `Unit::do_attack_ground`, whose `Unit::set_angle(…, 0)` writes
/// every crew figure's `des_angle` and leaves its `angle`: the two
/// trackless crew defer the swing, and guy 0's `Guy::move` turns them and
/// pays it at `+0x166` on the same frame. `Unit::fight`'s own strike
/// re-seats the figures (`set_new_location(…, 1, 0)`) before it swings,
/// and that snaps the crew — which is why every other lobby's crew swing
/// is `Unit::set_anim+0xb6`.
#[test]
fn gunpowder_melee_s_bombard_crew_pays_its_ground_swing_in_guy_move() {
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(sib), Some(tr)) = (
        dump(MELEE_LONG.0),
        dump(MELEE_START),
        crate::diff::testkit::trace(MELEE_LONG.1),
    ) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", MELEE_LONG.0);
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let sib_text = crate::capture::read(&sib);
    let log = Log::parse(&text);
    let sib_log = Log::parse(&sib_text);
    let sib_init = sib_log.initial().expect("a start dump");
    let mut init = log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sib_init]);
    crate::diff::setup::borrow_pasture(&mut init, &tr);
    let mut built = crate::diff::setup::with_staged_script(melee_script(), || {
        build_sim(&loaded, &init, Tuning::RON)
    });
    built.sim.trace_phases = true;
    const WORD: i64 = 759;
    while built.sim.frame < WORD {
        built.tick();
    }
    built.tick();
    let (f, ours) = built.frame_sites.last().expect("759's draws");
    assert_eq!(*f, WORD);
    let bombard: Vec<String> = crate::diff::harness::attributed_sites(&built)
        .into_iter()
        .filter(|(_, who)| who == "unit 2/23")
        .map(|(label, _)| label)
        .collect();
    assert_eq!(
        bombard,
        [
            sim::fight::SITE_FIGHT_RESEARCH,
            sim::anim::SITE_ATTACK_STAND,
            sim::anim::SITE_ATTACK_STAND
        ],
        "2/23's draws on 759: the re-search, then the two trackless crew's deferred swing"
    );
    assert_eq!(*ours, tr.labels(WORD), "759's whole draw sequence");
    let u = built
        .sim
        .units
        .iter()
        .position(|x| x.owner == 2 && x.index == 23)
        .expect("2/23");
    assert!(
        matches!(
            built.sim.units[u].orders.front().map(|o| &o.body),
            Some(sim::orders::Body::AttackGround(_))
        ),
        "the swing is the ground order's"
    );
}

/// run1461b: the leaders' window 595..1700 at `end:MISC,LEADERS=2` — every
/// leader's `bucket` on every frame from the staging (item 3788). The stride
/// blocks print no `LEADERDATA`, and who=2's stockpile had parted by 1700.
pub(crate) const RUN1461B: &str =
    "gamelog-run1461b-greatsahara-gunpowder-melee-leaders-window-595-1700.txt";

/// run1461: the same records with `UNITS=3,BUILDS=7,CITIES=5`, 1700..2161.
pub(crate) const RUN1461: &str =
    "gamelog-run1461-greatsahara-gunpowder-melee-leaders-window-1700-2161.txt";

/// The six goods as [`crate::diff::leader::rows`] and the dump spell a
/// bucket: `bucket[2:wealth]`.
const GOODS: [&str; 6] = ["food", "timber", "wealth", "knowledge", "metal", "oil"];

/// **An Inca unit refunds its price, and a hero pays its killer** (item
/// 3788, `docs/GOLDEN.md` §149.12): run1461b's and run1461's every frame,
/// both computers' stockpiles, walked from run1412's start with the script
/// staged. The widening's leader rows read seats 0 and 1 only
/// (`harness.rs`, `for who in 0..2`, and the oracle's two-leader cache), so
/// who=2 — the Inca — is read here. Without the refund who=2's buckets
/// part on logger frame 860 (+16 food, +11 timber) and 864 (+25 timber,
/// +25 metal); without the hero's plunder on 1464 (+100 wealth).
#[test]
fn gunpowder_melee_run1461b_s_leaders_hold_the_inca_refund() {
    let _pins = Pins::hold();
    let Some(inst) = install() else {
        return;
    };
    let (Some(long), Some(start), Some(tr), Some(early), Some(late)) = (
        dump(MELEE_LONG.0),
        dump(MELEE_START),
        trace(MELEE_LONG.1),
        dump(RUN1461B),
        dump(RUN1461),
    ) else {
        eprintln!("skipping: no {RUN1461B} (set RON_GAMELOG_DIR)");
        return;
    };
    let rows = crate::diff::setup::with_staged_script(melee_script(), || {
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(long);
        let start_text = crate::capture::read(start);
        let log = Log::parse(&text);
        let start_log = Log::parse(&start_text);
        let mut init = log.initial().unwrap();
        let sibling = start_log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sibling]);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        // Each window's buckets, by logger frame and seat.
        let mut theirs = std::collections::BTreeMap::new();
        for path in [early, late] {
            let t = crate::capture::read(path);
            let l = Log::parse(&t);
            for (n, b) in l.frames() {
                for l in b.kids("LEADERDATA") {
                    let Some(who) = l.int("who").filter(|w| (1..=2).contains(w)) else {
                        continue;
                    };
                    let m = crate::diff::leader::theirs(&l);
                    let bucket: Vec<Option<i64>> = (0..6)
                        .map(|g| m.get(&format!("bucket[{g}:{}]", GOODS[g])).copied())
                        .collect();
                    theirs.insert((n, who), bucket);
                }
            }
        }
        // A key that reads on neither side compares nothing (the first
        // spelling of this test read `bucket[2]` and passed every mutant).
        assert!(
            theirs.values().all(|b| b.iter().all(Option::is_some)),
            "every printed bucket read"
        );
        let last = theirs.keys().map(|(n, _)| *n).max().unwrap_or(0);
        let mut compared = 0usize;
        let mut parted = Vec::new();
        while built.sim.frame < last {
            built.tick();
            // Logger frame n prints the state ours holds after frame n − 1.
            let n = built.sim.frame;
            for who in 1..=2usize {
                let Some(want) = theirs.get(&(n, who as i64)) else {
                    continue;
                };
                let m: std::collections::BTreeMap<String, i64> =
                    crate::diff::leader::rows(&loaded, &built, who)
                        .into_iter()
                        .collect();
                let got: Vec<Option<i64>> = (0..6)
                    .map(|g| m.get(&format!("bucket[{g}:{}]", GOODS[g])).copied())
                    .collect();
                assert!(got.iter().all(Option::is_some), "ours' buckets read");
                compared += 1;
                if &got != want && parted.len() < 4 {
                    parted.push(format!("{n} who={who}: ours {got:?} theirs {want:?}"));
                }
            }
        }
        (theirs.len(), compared, parted)
    });
    let (printed, compared, parted) = rows;
    eprintln!("run1461b+run1461 leaders: {printed} printed, {compared} compared, {parted:?}");
    pin_eq!(
        parted,
        Vec::<String>::new(),
        "the computers' buckets that part"
    );
    pin_eq!(compared, printed, "every printed leader compared");
    // Item 3788: 1566 frames (595..2161) × who=1 and who=2.
    pin_eq!(printed, 3132, "the leader records printed");
}

/// **The melee's closing state** (item 3788): run1413's closing block, the
/// whole map's units, walked from run1412's start with the script staged.
#[test]
fn gunpowder_melee_run1413_s_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) =
        (dump(MELEE_LONG.0), dump(MELEE_START), trace(MELEE_LONG.1))
    else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", MELEE_LONG.0);
        return;
    };
    let Some(e) = crate::diff::setup::with_staged_script(melee_script(), || {
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(path);
        let start_text = crate::capture::read(start);
        let log = Log::parse(&text);
        let start_log = Log::parse(&start_text);
        let mut init = log.initial().unwrap();
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
            "Gunpowder melee closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
            e.frame,
            e.compared,
            e.counts(),
            e.torn,
            e.off,
            e.unlinked,
            e.extra
        );
        Some(e)
    }) else {
        return;
    };
    pin_eq!(e.frame, 2601, "the closing block");
    // Item 3788, the game whole: 145 units compared, every count zero, no
    // extra (`docs/GOLDEN.md` §149.12).
    pin_eq!(e.compared, 145, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(e.extra.clone(), vec![], "the closing extras");
}
