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
    count: 863,
    sequence: 863,
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

/// **The word's window** (item 3733, moved by 3757, 3759, 3762, 3763 and
/// 3767): run1413's stride blocks 801..1101 — the word's frame 863 writes no
/// block (the stride prints every hundred), so the window is the stride
/// block before it through 250 frames past it. 3733's was 101..432, on
/// 182; 3757's 601..934, on 684; 3759's 701..1009, on 759; 3762's
/// 701..1010, on 760; 3763's 801..1089, on 839.
pub(crate) const WIDENING_GUNPOWDER_MELEE: (i64, i64) = (801, 1113);

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
            &[],
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
    pin_eq!(w.blocks, 4, "every stride block in the window");
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
    pin_eq!(w.firsts.len(), 669, "run1413's word window");
    pin_eq!(w.rounds_read, (27, 30), "the rounds read");
    pin_eq!(w.rounds.len(), 22, "the rounds that part");
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
