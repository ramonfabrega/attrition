//! The blind list — the cited functions no coverage trace has entered.
//!
//! `docs/DECISIONS.md` entry 29 names it the second counter of "sim done",
//! entry 41 makes it one of the four: **the blind list at an enumerated
//! residue**. A diff can only check what a run reaches, so a function a
//! document cites and no run of the original has ever entered is a claim
//! resting on a reading alone. `tools/trace/report.py … blind docs/` prints
//! the list; this module is the same measurement as an assertion, so the
//! list cannot grow in silence (`docs/CENSUS.md`, "The blind list, ranked").
//!
//! **What is pinned is the list, not a count.** [`NEVER`] is every
//! never-entered cited function by address, so a failure names the
//! function that moved: a new citation of a function no pinned trace
//! enters, or a pinned trace (a new capture added to [`TRACES`]) entering
//! one. A count would say only that something did. The residue DECISIONS
//! 29 asks to be "enumerated and accepted deliberately, function by
//! function" is this constant.
//!
//! **What is not pinned**: the traces themselves, which live outside the
//! repo in the capture corpus (`docs/ORACLE.md`), and the Ghidra export's
//! `INDEX.tsv`, which never enters it. A machine without either skips and
//! says so, through the same door every kept dump uses, so the release
//! gate's fixture audit sees each trace requested.
//!
//! The measurement is `report.py`'s `blind` verb exactly: a function is
//! **cited** when `docs/**/*.md` writes `name@00xxxxxx` — joined across a
//! line break after the `@` (parked 835) — and the address is a function
//! the export lists; it is **entered** when a trace carries a `HIT` record
//! for it, or, for the five trampolined functions that carry no stub, a
//! record of the kind their hook emits. The proxied call sites `report.py`
//! also credits change nothing on this corpus (item 923 measured both), so
//! they are not read here.

/// The coverage traces the list is measured against, by archive name.
///
/// **Twenty-one of the eighty-nine on disk before item 923**, and they
/// enter exactly what all eighty-nine do — 7,664 functions by `HIT`
/// record, 7,666 with the hooks — the greedy cover item 923 measured,
/// largest gain first:
/// `run29` alone enters 7,175. The other sixty-eight add nothing, so
/// reading them would cost 1.2 GB and change no line of [`NEVER`]. A
/// capture taken to shrink the list joins here in the landing that pins
/// its effect.
pub const TRACES: &[&str] = &[
    "rontrace-run29.log",
    "rontrace-run16.log",
    "rontrace-run50.log",
    "rontrace-run218.log",
    "rontrace-run49.log",
    "rontrace-run20.log",
    "rontrace-run44.log",
    "rontrace-run24.log",
    "rontrace-run47.log",
    "rontrace-run48.log",
    "rontrace-run74.log",
    "rontrace-run18a.log",
    "rontrace-run32.log",
    "rontrace-run30.log",
    "rontrace-run66.log",
    "rontrace-fuzz-424242.log",
    "rontrace-run14.log",
    "rontrace-run15.log",
    "rontrace-run22.log",
    "rontrace-run64.log",
    "rontrace-run906.log",
    // Item 923's capture: chapter twenty-three's staging under `cover=1`,
    // whose `@` issuers the coverage jmps refuse (`docs/RUNS.md` run314).
    // It enters `Object::do_launch` and `Leader::set_age`, and nothing
    // else no other trace here does.
    "rontrace-run314.log",
    // Item 935's capture: British Isles with a Large Town start, quit at
    // frame 5 (`docs/RUNS.md` run316). It enters `Map::place_start_in_region`
    // and `Setup::large_city_buildings`, the two of row 4's nine that a solo
    // lobby reaches.
    "rontrace-run316.log",
    // Item 934's captures: the issuer chapters' scripts under `cover=1`,
    // the `@` guard reading through the coverage jmp (`docs/RUNS.md`
    // run318 onward). run318 is chapter twenty-three's, run314 again.
    "rontrace-run318.log",
    "rontrace-run319.log", // ch10
    "rontrace-run320.log", // ch11
    "rontrace-run321.log", // ch12
    "rontrace-run322.log", // ch13
    "rontrace-run323.log", // ch14
    "rontrace-run324.log", // ch15
    "rontrace-run325.log", // ch16
    "rontrace-run326.log", // ch17
    "rontrace-run327.log", // ch18
    "rontrace-run328.log", // ch19
    "rontrace-run329.log", // ch20
    "rontrace-run330.log", // ch21
    "rontrace-run331.log", // ch24
    "rontrace-run332.log", // ch25
    "rontrace-run333.log", // ch26
    "rontrace-run334.log", // ch27
    "rontrace-run335.log", // ch28
    "rontrace-run336.log", // ch29
    "rontrace-run337.log", // ch30
    // Item 959's captures: a staged economy for row 6 (`docs/RUNS.md`
    // run340–run342). run342, a Tower's arrows on the caravan, enters
    // nothing run341 does not.
    "rontrace-run340.log",
    "rontrace-run341.log",
    "rontrace-run342.log",
    // Item 965's: three Towers' arrows kill the caravan on its owner's road
    // (`docs/RUNS.md` run353), and `Object::take_damage`'s death path calls
    // `Caravans::new_danger` and `Caravan::restart_trade_route` on 1007.
    "rontrace-run353.log",
    // Item 1011's captures: the issuer chapters thirty-one to thirty-four,
    // captured at `cover=0` (run338, run344, run358, run362), re-run line
    // for line under `cover=1` on the queue lane (`docs/RUNS.md` run364–
    // run367), each the golden game on every frame.
    "rontrace-run364.log", // ch31
    "rontrace-run365.log", // ch32
    "rontrace-run366.log", // ch33
    "rontrace-run367.log", // ch34
    // Item 1019's capture: chapter thirty-five at `cover=1` on the queue
    // lane (`docs/RUNS.md` run372), run371's game on every frame.
    "rontrace-run372.log", // ch35
    // Item 1091's capture: chapter thirty-seven at `cover=1` on the queue
    // lane to 3210 (`docs/RUNS.md` run398), run397's game on every frame.
    "rontrace-run398.log", // ch37
    // Item 1102: chapter thirty-eight at `cover=1` on the queue lane to
    // 1030 (`docs/RUNS.md` run405), run404's game on every frame.
    "rontrace-run405.log", // ch38
    // Item 1111: chapter thirty-nine at `cover=1` on the queue lane to 830
    // (`docs/RUNS.md` run423), run422's game on every frame.
    "rontrace-run423.log", // ch39
    // Item 1167: chapter forty at `cover=1` on the queue lane to 910
    // (`docs/RUNS.md` run431), run430's game on every frame.
    "rontrace-run431.log", // ch40
    // Item 1182: chapter forty-one at `cover=1` on the queue lane to 1140
    // (`docs/RUNS.md` run438), run437's game on every frame.
    "rontrace-run438.log", // ch41
    // Item 1209: chapter forty-two at `cover=1` on the queue lane to 920
    // (`docs/RUNS.md` run461), run460's game on every frame.
    "rontrace-run461.log", // ch42
    // Item 3417: the peace pair's lobby (East Indies, the Lakota, `RUSH_RULES
    // 10`) at `cover=1` on the queue lane to 600, with `500 unmeet who=1`
    // (`docs/RUNS.md` run1016), run847's game on every frame.
    "rontrace-run1016.log", // peace pair
    // Item 3418: Great Sahara at Toughest (run470's lobby) at `cover=1` on
    // the click-free pool to 15432, the game's own last frame
    // (`docs/RUNS.md` run1017), run470's game on every frame.
    "rontrace-run1017.log",
    // Item 3431: the middle ages, Great Sahara at Toughest (run470's lobby)
    // at `STARTING_TECHNOLOGY 3` and `5`, each at `cover=1` on the click-free
    // pool to 6000 (`docs/RUNS.md` run1041, run1043), its `cover=0` twin's
    // game (run1040, run1042) on every frame. They enter no `NEVER` row;
    // they are pinned for the ages their dumps hold, Gunpowder and
    // Industrial, which no other pinned trace's dump holds (`docs/RUBRIC.md`).
    "rontrace-run1041.log",
    "rontrace-run1043.log",
    // Item 3420: the coverage pair (East Indies, the Persians, All
    // Technologies) at `cover=1` on the click-free pool to 3150, run717's
    // recipe (`docs/RUNS.md` run1019), run717's and run652's game on every
    // frame.
    "rontrace-run1019.log",
    // Item 3421: Great Sahara in the coverage pair's lobby (the Persians,
    // All Technologies) at `cover=1` on the click-free pool to 3770,
    // run790's recipe (`docs/RUNS.md` run1020), run790's and run676's game
    // on every frame.
    "rontrace-run1020.log",
    // Item 3419: Great Lakes, the Iroquois at difficulty 3 (run897b's lobby)
    // at `cover=1` on the click-free pool to 12658, the game's own last
    // frame (`docs/RUNS.md` run1018), run897b's game on every frame.
    "rontrace-run1018.log",
    // Item 3422: the successor pair's lobby (East Indies, the Mongols, All
    // Technologies) at `cover=1` on the click-free pool to 2210, run785's
    // recipe with the window moved (`docs/RUNS.md` run1021), run775's,
    // run780's and run785's game on every frame.
    "rontrace-run1021.log",
    // Item 3423: two short Great Sahara retakes at `cover=1` on the
    // click-free pool (`docs/RUNS.md` run1022, run1024): the Mongols, All
    // Technologies, to 3000 (run834's and run831's game on every frame),
    // and the Dutch at difficulty 3 to 320 (run995's and run990b's). The
    // third, the Bantu's run1023 to 2810, enters no blind row and is not
    // pinned: the script's `can_queue` is the copy at `004d2030`.
    "rontrace-run1022.log",
    "rontrace-run1024.log",
    // Item 3424: golden chapter sixty-four (`chapter64.cmd` line for line)
    // at `cover=1` on the click-free pool to 820 (`docs/RUNS.md` run1025),
    // run835's game on every frame.
    "rontrace-run1025.log",
    // Item 3424: golden chapter ninety-one (`chapter91.cmd` line for line)
    // at `cover=1` on the click-free pool to 1200 (`docs/RUNS.md` run1026),
    // run894's game on every frame.
    "rontrace-run1026.log",
    // Item 3425: chapter three's lobby with an `@attackground` on 630 and
    // chapter thirteen's with an `@comeout` on 800, each at `cover=1` on the
    // click-free pool (`docs/RUNS.md` run1029, run1030; `docs/GOLDEN.md`
    // §145): run145's and run208's game to the issue's frame.
    "rontrace-run1029.log",
    "rontrace-run1030.log",
    // Item 3426: the golden start (map 14, the human Nubian's Market) with
    // the tracer's `@sell 0 1 0` on 620, `@buy 0 1 0` on 640 and `@sell 0 1
    // 1` on 660, at `cover=1` on the click-free pool to 700 (`docs/RUNS.md`
    // run1031), run1031b's game on every frame.
    "rontrace-run1031.log",
    // Item 3427: the click layer's first pass, driven on the queue lane at
    // `cover=1` through `tools/gamelog/handdriver.sh` (`docs/RUNS.md`
    // run1032–run1034; `docs/GOLDEN.md` §144.3): chapter eleven's lobby to
    // 793 (a right-click guard on 624, a Move-button follow on 708),
    // chapter thirteen's to 773 (a Barracks train click on 736, its Eject
    // on 748), chapter twenty's to 814 (Auto Transport on 626 and 801).
    // Each is its golden twin's game until the first input that differs.
    "rontrace-run1032.log",
    "rontrace-run1033.log",
    "rontrace-run1034.log",
    // Item 3428: a recorded game's playback at `cover=1` on the click-free
    // pool, `-config input.rcx` from frame 0 to 262, the recording's last
    // package (`docs/RUNS.md` run1036): run349's own recording (East Indies,
    // run346's lobby), run349's game on every frame.
    "rontrace-run1036.log",
    // Item 3436: the click layer's second pass, the same driver and lane
    // (`docs/RUNS.md` run1044–run1056; `docs/GOLDEN.md` §144.3): chapter
    // fifteen's lobby to 749 (a right-click attack on 736), seventeen's to
    // 673 (a plane's right-click on its Airbase, 656), twenty-three's to
    // 801 (the Airbase's repeat button, 788), twenty-five's to 889 (a
    // cancel on the Barracks' Hoplite, 876), eighteen's to 713 (a Barracks
    // build button and drop, 700) and twenty-one's to 813 (a Citizen's
    // right-click on the Barracks, 796). run1046 and run1047 ran unclicked.
    "rontrace-run1044.log",
    "rontrace-run1045.log",
    "rontrace-run1048.log",
    "rontrace-run1049.log",
    "rontrace-run1055.log",
    "rontrace-run1056.log",
    // Item 3438: golden chapter eighty-five (`chapter85.cmd` line for line,
    // the cloaked Spy's `@spell 0 629`, Bribe, on 1112) at `cover=1` on the
    // click-free pool to 1260 (`docs/RUNS.md` run1054), run880's game on
    // every frame both hold.
    "rontrace-run1054.log",
    // Item 3442: chapter fourteen's lobby (GOLDEN §22) at `cover=1` on the
    // click-free pool to 1150, run210's lines with its `740 @form` sent as
    // three bound keys through the tracer's `@key` (`tools/trace/key.h`) and
    // the group selected by `736 select 6 who=0` (`docs/RUNS.md` run1057);
    // run210's game on every frame before 736 (rngcmp 742 identical).
    "rontrace-run1057.log",
    // Item 3443: three clicks on the click-free pool. Chapter thirty-nine's
    // lobby to 880 with Citizen A's To Arms pressed on its card (624;
    // run1058), run1051's chapter-thirteen stanza to 780 with who=0's
    // buckets drained before the train click (736; run1059), and chapter
    // seven's lobby to 910 with a Small City and the Caravan right-clicked
    // onto it (650; run1060).
    "rontrace-run1058.log",
    "rontrace-run1059.log",
    "rontrace-run1060.log",
    // Item 3444: chapter nine's lobby (GOLDEN §17) at `cover=1` on the
    // click-free pool to 800, run180's lines with the chariot's `620 @move`
    // replaced by a plain `@rclick` on the minimap (`docs/RUNS.md` run1061);
    // run180's game on every block before 621.
    "rontrace-run1061.log",
    // Item 3444: the peace pair's lobby at `cover=1` on the click-free pool
    // to 1560, the diplomacy window opened by a bound key on 1480 and its
    // Declare button clicked on 1484 (`docs/RUNS.md` run1062); run847's
    // game on every frame both hold (rngcmp 1561 identical).
    "rontrace-run1062.log",
    // Item 3445: chapter seventy-four's lobby (GOLDEN §91, run864's lines)
    // at `cover=1` on the click-free pool. run1063a is run864's lines to 760,
    // `701 finish` (`run_cmd` case 0x4f), run864's game on every frame; run1063
    // takes `701 hurry` (case 0x51, `Console::hurry_action`) to 960; run1065
    // to 960 saves the Options window mid-game (the Game Menu at 702, its
    // clicks on `@after` lines, `OptionsWinGame::do_save` on 702); run1064
    // to 960 renames Napata `0/2000` through its card on 706 (`@key` and
    // `@char` into the popup's edit box, Return as both), and
    // `process_rename_city` calls `HotKeyGroups::find_group`.
    "rontrace-run1063a.log",
    "rontrace-run1063.log",
    "rontrace-run1064.log",
    "rontrace-run1065.log",
    // Item 3446: the coverage pair's lobby (East Indies, the Persians, All
    // Technologies) at `cover=1` on the click-free pool to 1300, AI on, the
    // carrier staged by console and `@` lines from 600
    // (`tools/gamelog/coverage/carrier3446.cmd`, `docs/RUNS.md` run1066);
    // run652's game on every frame before 603 (rngcmp 603 identical).
    "rontrace-run1066.log",
    // Item 3447: the Nomad lobby (`--map 18 --profile STARTING_TOWN=0`, AI
    // on, Nubians against a British computer) at `cover=1` on the
    // click-free pool, to 600; `ScenarioFuncSet::citizen_repair_order` on
    // 376, the computer's nomad step 2 (`economic.bhs:398`). Its `cover=0`
    // twin run1068b on every frame (rngcmp 601 identical; `docs/RUNS.md`).
    "rontrace-run1068.log",
    // Item 3448: the golden start (map 14, run215's lobby to 600) at
    // `cover=1` on the click-free pool. run1069 to 1350 `@gather`s eight
    // Citizens onto the Woodcutter's Camp `0/2001`: the three past its cap
    // take `add_think_order` on 621 and `do_think_order` on 622. run1070 to
    // 1250 is chapter fifteen's cast `@gather`ed onto the same camp (the
    // soldiers' group move, no action bit) with a shift-queued `@attackadd`
    // on the Chariot behind it: the leader collides with a Supply Wagon and
    // `Group::distribute_attack` runs on 686. Both are run215's game to their
    // first unshared input (`docs/RUNS.md`). run1071 to 1150 is the same
    // start with a Library: a Lookout and, shift-queued (`@buildadd`), a
    // Siege Factory, and `check_build_order` on 861, the human arm (run1019
    // took it off the list on a computer's builder).
    "rontrace-run1069.log",
    "rontrace-run1070.log",
    "rontrace-run1071.log",
    // Item 3453: the peace pair's lobby at `cover=1` on the click-free pool
    // to its own end at 6853, a who=0 Hoplite squad added in the Lakota's
    // land on 1386 (`tools/gamelog/coverage/nowar3453-peace.cmd`,
    // `docs/RUNS.md` run1072); `Game::say_no_war` on 1392. run847's game on
    // every frame before 1387 (rngcmp 1387 identical).
    "rontrace-run1072.log",
    // Item 3454: chapter seven's lobby (map 14, run141's to 920) at
    // `cover=1` on the click-free pool, by console lines alone: two Pyramids
    // `add`ed complete on 640 and 645 (`0/2007`, `0/2008`, wonder slots 0
    // and 1), then `die 2007,0` on 660 and `die 2008,0` on 670.
    // `Wonders::close_wonder` on 660; `wonder_mark` 2 -> 2 -> 0. run141's
    // game to 640 (`docs/RUNS.md`).
    "rontrace-run1074.log",
    // Item 3455: the coverage pair's lobby (East Indies, the Persians, All
    // Technologies) at `cover=1` on the click-free pool, AI on. run1075b to
    // 2560 kills the computer's navy (army 2, group 72, nine ships) by
    // console `die` on 2284 (`tools/gamelog/coverage/disband3455-die.cmd`):
    // its 2296 tick disbands it, `Army::send_here` on 2296; run652's draw
    // stream on every frame before 2297 (rngcmp 2297 identical). run1076 to
    // 700 right-clicks a human Trireme onto the human's own Dock
    // (`dock3455-rclick.cmd`): `ObjectData::is_dock` on 615; run652's game
    // before 601 (rngcmp 601 identical; `docs/RUNS.md`).
    "rontrace-run1075b.log",
    "rontrace-run1076.log",
    // Item 3456: the team lobby on the click-free pool, Great Lakes (map 14)
    // with `--profile PLAYERS=2` (a human and two computers), `ally 1` on
    // 100, `!ffwd 6`; a help taunt to the ally on 5410 (`--bindkey
    // OPTION_CHAT_ALLIES`, "11", Return: leader 1's `last_taunt[0]` 16,
    // `taunt_frame[0]` 5411) and pings on 5422 and 5702 (`--bindkey
    // OPTION_PING`, `@click 960 540`): `Leader::action_ping` on 5422. No
    // army of either computer has a captain, so `find_useful_army` finds
    // none and `send_here` is not called. To 5710
    // (`tools/gamelog/coverage/ping3456-team.cmd`, `docs/RUNS.md` run1077).
    "rontrace-run1077.log",
    // Item 3460: the coverage pair's lobby (map 18, the Persians, All
    // Technologies, DIFFICULTY 5) with `--profile TEAM_STYLE=0`
    // (Diplomacy; `GAME INFO` `TEAM_STYLE 0`, leaders at peace from the
    // start), `meet who=1` on 900, the stock 100000 up on 901, the
    // diplomacy window (`--bindkey OPTION_DIPLO`, F18) on 905 and its
    // Declare button with Return on 910 (`@click 955 366`, `@key 13`,
    // `@char 13`): `Leader::action_declare` and `Leader::pay_dow` on 910;
    // `CM process_declare 1 0 911`; leader 0's `diplos[1]` 1 -> 0 and
    // `dow[1]` 0 -> 1 on block 912. Its `cover=0` twin is run1079 (rngcmp
    // 961 identical). To 960 (`tools/gamelog/coverage/declare3460-diplomacy.cmd`,
    // `docs/RUNS.md` run1078).
    "rontrace-run1078.log",
    // Item 3461: the coverage pair's lobby (3446's carrier lobby) at
    // `cover=1` on the click-free pool, AI on, to 760
    // (`tools/gamelog/coverage/halt3461.cmd`): a who=0 squad selected and
    // F13 bound to OPTION_HALT, `process_halt 621`; F14 bound to
    // OPTION_CYCLE_GENERAL with a General `add`ed, `find_group_const` on
    // 640; the Aircraft Carrier `0/14` alone selected and right-clicked on
    // open sea, `is_tocean` on 660 (`process_move_to 642 9474`). run652's
    // game before 601 (rngcmp 601 identical; `docs/RUNS.md` run1080).
    "rontrace-run1080.log",
    // Item 3462: golden chapter eighty-five's lobby (run880's game) at
    // `cover=1` on the click-free pool to 1260
    // (`tools/gamelog/coverage/spy3462-ch85.cmd`): the second Spy `0/7`
    // selected and F13 bound to OPTION_SPY_BRIBE, the pick clicked on 1112,
    // `find_target` entered (it found no target: no second `process_spell
    // 629`); Pilfer and Restock Supplies asked by `@spell` and refused by
    // `is_castable`'s head (both `FROM None`). run880's game before 1107
    // (rngcmp 1,109 identical; `docs/RUNS.md` run1081).
    "rontrace-run1081.log",
    // Item 3463: the first pair's East Indies lobby (DIFFICULTY 0, AI on)
    // at `cover=1` on the click-free pool, `--ffwd-minute 11`, to 10100
    // (`tools/gamelog/coverage/agetip3463-eastindies.cmd`): `age who=0 3`
    // on 9905 (the human's `ages_get()` 3 from block 9906) and the
    // computer's stock 100000 up on 9906; `Leaders::best_human_age_stamp`
    // on 9979, the first `research_techs` after leader 1's script ends on
    // 9977. The `@cursor` lines on the food counter (9910..9990) raise no
    // tooltip (parked 3463a). Its `cover=0` twin is run1083
    // (`docs/RUNS.md` run1082).
    "rontrace-run1082.log",
    // Item 3464: the golden start (run1031's lines, `market3426.cmd`) at
    // `cover=1` on the click-free pool to 700, with `600 !cursor tinfo`
    // (`run_cmd` case 0x3d, `World::get_wdata` on 600) and `602
    // !startareas` (case 0x60, `WorldData::is_forest` on 602) before the
    // twin's first input (`tools/gamelog/coverage/console3464-start.cmd`).
    // Neither writes sim state: rngcmp against run1031b 701 identical, 0
    // differing (`docs/RUNS.md` run1084).
    "rontrace-run1084.log",
    // Item 3470: run1077's team lobby (`--profile PLAYERS=2`, `ally 1`) at
    // `cover=1` on the click-free pool to 700, with `resource who=1 all
    // +100000` and the food and timber taunts to allies on 600 and 620
    // (`tools/gamelog/coverage/taunt3470-team.cmd`): `receive_taunt` queues
    // each for 60 frames, and on 661 `Leader::process_taunt` case 1 takes
    // `action_offer` and `Leader::action_respond`. rngcmp against run1087
    // 701 identical, 0 differing (`docs/RUNS.md` run1086).
    "rontrace-run1086.log",
    // Item 3471: the coverage pair's lobby (run1080's) at `cover=1` on the
    // click-free pool to 700 (`tools/gamelog/coverage/eval3471.cmd`), the
    // console's `eval` and `netcmd`: `!eval rand_real()` on 610 (its draw
    // record, through `Compiler::eval_command`), `!eval set_bonus_cap(1,
    // "Wealth", 200)` on 620 (leader 0's `bonus_cap[2]` 0 -> 200 on block
    // 621), `!eval is_on_transport(1, 15)` on 630 (`in_a_ship`). The plain
    // `netcmd` on 640 enters `issue_console_cmd` only: its 0x209-byte
    // command never fits `add_command`'s 0x200. Its `cover=0` twin is
    // run1089 (rngcmp 701 identical; `docs/RUNS.md` run1088).
    "rontrace-run1088.log",
    // Item 3477: the golden start (run1084's lines to 660,
    // `console3464-start.cmd`) at `cover=1` on the click-free pool to 720,
    // the cursor pinned at (1400, 1000) (Loop 3432), then one `!eval` a
    // frame pair from 670 (`tools/gamelog/coverage/eval3477.cmd`): the
    // seventeen `ScenarioFuncSet` rows, each on its own line's frame, and
    // `disable_type_by_tribe` on 704 for `Tribes::find`. Its `cover=0` twin
    // is run1093 (rngcmp 721 identical, 0 differing; `docs/RUNS.md` run1092).
    "rontrace-run1092.log",
    // Item 3479: the golden start at `cover=1` on the click-free pool to 640,
    // the param console bound to F12 (`--bindkey
    // SYSTEM_TOGGLE_PARAM_CONSOLE=123`) and opened after `600 keys 1`; inside
    // its modal loop, `@after` lines send a Return on Lights/Sun, pick Nuke off
    // the category pulldown and send a Return on it
    // (`tools/gamelog/coverage/param3479.cmd`). Its `cover=0` twin is run1102
    // (`docs/RUNS.md` run1096).
    "rontrace-run1096.log",
    // Item 3481: the team lobby (run1090's, `--profile PLAYERS=2`) at
    // `cover=1` on the click-free pool to 5420
    // (`tools/gamelog/coverage/herd3481-reveal.cmd`): `5405 reveal 0` clears
    // `gamec->semaphore` bit 1, which every pool lane's `StartReveal=1` sets
    // through `Game::solo_checks`. With the bit clear, `sort_all` calls vtable
    // +0x48, and `AnimalData::is_seen` is entered on 5406 under `camera
    // 186,190` on herd 9. The packet run1100 named the gate. The `cover=0`
    // twin is run1110 (rngcmp 5421 identical, 0 differing; `docs/RUNS.md`
    // run1109).
    "rontrace-run1109.log",
    // Item 3491: the team lobby (run1086's, `--profile PLAYERS=2`) with no
    // `ally` line, `--no-ffwd`, at `cover=1` on the click-free pool to 640
    // (`tools/gamelog/coverage/tournament3491-team.cmd`): `600 !eval
    // defeat(3)` defeats leader 2 through `ScenarioFuncSet::defeat` without a
    // quit, `602 be 2` moves the console to the defeated chair and `604 reveal
    // 0` clears semaphore bit 1, so `Game::is_tournament` is entered on 604.
    // The `cover=0` twin is run1114 (rngcmp 641 identical, 0 differing;
    // `docs/RUNS.md` run1113).
    "rontrace-run1113.log",
    // Item 3492: the golden start (run1084's lines, `console3464-start.cmd`)
    // at `cover=1` on the click-free pool to 700 with `--rise
    // AutoSaveLoad=650`: `Game::do_frame` saves and reloads the game when the
    // frame becomes 650, through `WalkDataGame::walk_data` and the goods' and
    // items' +0x7c walkers. Its `cover=0` twin is run1116 (rngcmp 701
    // identical, 0 differing; `docs/RUNS.md` run1115).
    "rontrace-run1115.log",
    // Item 3494: the golden start (map 14), `--no-ffwd`, at `cover=1` on the
    // click-free pool to 640 (`tools/gamelog/coverage/hover3494-start.cmd`):
    // `600 @cursor 70 33` pins the cursor on the food counter, and `--hover`
    // (`rontrace.cfg` `hover=1`, `tools/trace/hover.h`) re-sends the pinned
    // `OnMouseMove` at `Scene::describe_mouseover`'s entry, after
    // `Scene::render` has zeroed `mouseover->type`. Its `cover=0` twin is
    // run1130 (rngcmp 641 identical, 0 differing; `docs/RUNS.md` run1129).
    "rontrace-run1129.log",
    // Item 3501: the golden start at `cover=1` on the click-free pool to 690
    // (`tools/gamelog/coverage/graft3501-start.cmd`): `670 !eval
    // disable_type_by_tribe("Barracks", "Nubians")` clears the human's tribe
    // bit on Barracks, and `672 !eval train_unit(1, 1, "Hoplites")` grafts
    // the unit's `where` through `LeaderData::get_graft@0047df40`, whose
    // `+0x68` on the Barracks enters `TypeData::type_graft` on 672. Its
    // `cover=0` twin is run1135 (rngcmp 691 identical, 0 differing;
    // `docs/RUNS.md` run1134).
    "rontrace-run1134.log",
    // Item 3502: the golden start (map 14), `--no-ffwd`, at `cover=1` on the
    // click-free pool to 640 (`tools/gamelog/coverage/siege3502-althold.cmd`):
    // a catapult (o 6) right-clicked onto who 1's Barracks (o 2006) on 612
    // with `610 @hold 18 616` (`tools/trace/hold.h`) answering VK_MENU held
    // through the exe's `GetKeyState` import. Its `cover=0` twin is run1137
    // (rngcmp 641 identical, 0 differing; `docs/RUNS.md` run1136).
    "rontrace-run1136.log",
    // Item 3531: the German late-age pair's long lobby (East Indies, the
    // Germans, Toughest, All Technologies; run1145's recipe) at `cover=1` on
    // the click-free pool, to the game's own end at 4175 (`docs/RUNS.md`
    // run1176): run1145b's and run1145's game on every frame (rngcmp 4176
    // identical, 0 differing, against both) — the first lockstep trace of an
    // age-7 game.
    "rontrace-run1176.log",
    // Item 3532: the Aztec late-age pair's long lobby (Great Sahara, the
    // Aztecs, All Technologies; run1147's recipe) at `cover=1` on the
    // click-free pool, to the game's own end at 4565 (`docs/RUNS.md`
    // run1177): run1147's game on every frame (rngcmp 4566 identical, 0
    // differing) — the land map's age-7 line.
    "rontrace-run1177.log",
    // Item 3533: the American late-age pair's long lobby (Great Sahara, the
    // Americans, `DIFFICULTY 3`, a held Modern age; run1151's recipe) at
    // `cover=1` on the click-free pool, to the game's own end at 4548
    // (`docs/RUNS.md` run1178): run1151's game on every frame (rngcmp 4549
    // identical, 0 differing) — the first lockstep trace of an age-6 game.
    "rontrace-run1178.log",
    // Item 3534: the Russian late-age pair's long lobby (East Indies, the
    // Russians, a held Modern age; run1149's recipe) at `cover=1` on the
    // click-free pool, to the game's own end at 5841 (`docs/RUNS.md`
    // run1179): run1149's game on every frame (rngcmp 5842 identical, 0
    // differing) — the sea map's age-6 line.
    "rontrace-run1179.log",
    // Item 3548: the Turkish late-age pair's long lobby (Great Sahara, the
    // Turks, `DIFFICULTY 3`, a held Enlightenment age; run1159's recipe) at
    // `cover=1` on the click-free pool, to the game's own end at 3812
    // (`docs/RUNS.md` run1187): run1159's game on every frame (rngcmp 3813
    // identical, 0 differing) — the first lockstep trace of an age-5 game.
    "rontrace-run1187.log",
    // Item 3549: the British late-age pair's long lobby (East Indies, the
    // British, `DIFFICULTY 5`, a held Enlightenment age; run1157's recipe)
    // at `cover=1` on the click-free pool, to the game's own end at 5522
    // (`docs/RUNS.md` run1188): run1157's game on every frame (rngcmp 5523
    // identical, 0 differing) — the sea map's age-5 line.
    "rontrace-run1188.log",
    // Item 3551: the Japanese late-age pair's long lobby (East Indies, the
    // Japanese, `DIFFICULTY 5`, a held Industrial age; run1153's recipe) at
    // `cover=1` on the click-free pool, to the game's own end at 4547
    // (`docs/RUNS.md` run1190): run1153's game on every frame (rngcmp 4548
    // identical, 0 differing) — the sea map's age-5 Industrial line.
    "rontrace-run1190.log",
    // Item 3552: the Chinese late-age pair's long lobby (Great Sahara, the
    // Chinese, `DIFFICULTY 3`, a held Industrial age; run1155's recipe) at
    // `cover=1` on the click-free pool, to the game's own end at 4467
    // (`docs/RUNS.md` run1191): run1155's game on every frame (rngcmp 4468
    // identical, 0 differing) — the land map's age-5 Industrial line.
    "rontrace-run1191.log",
    // Item 3572: the Roman Classical pair's long lobby (Great Sahara, the
    // Romans, `DIFFICULTY 5`, a held Classical age; run1194's recipe) at
    // `cover=1` on the click-free pool, to the game's own end at 7130
    // (`docs/RUNS.md` run1215): run1194's game on every frame (rngcmp 7131
    // identical, 0 differing) — the land map's age-1 Classical line.
    "rontrace-run1215.log",
    // Item 3573: the Persian Medieval pair's long lobby (East Indies, the
    // Persians, `DIFFICULTY 5`, a held Medieval age; run1181's recipe) at
    // `cover=1` on the click-free pool, to the game's own end at 4759
    // (`docs/RUNS.md` run1216): run1181's game on every frame (rngcmp 4760
    // identical, 0 differing) — the sea map's age-2 Medieval line.
    "rontrace-run1216.log",
    // Item 3733: the first melee (Great Sahara, three seats, a held
    // Gunpowder age, `tools/gamelog/golden/melee-gunpowder.cmd`) at
    // `cover=1` on the click-free pool, to 2600 (`docs/RUNS.md` run1413b):
    // run1413's game on every frame (rngcmp 2601 identical, 0 differing) —
    // the first coverage trace of a game that holds Gunpowder.
    "rontrace-run1413b.log",
    // Item 3774: the second melee (Great Sahara, three seats, a held
    // Classical age, `tools/gamelog/golden/melee-classical.cmd`) at
    // `cover=1` on the click-free pool, to 2600 (`docs/RUNS.md` run1450b):
    // run1450's game on every frame (rngcmp 2601 identical, 0 differing).
    "rontrace-run1450b.log",
    // Item 3797: the third melee (Great Sahara, three seats, a held
    // Information age, `tools/gamelog/golden/melee-information.cmd`) at
    // `cover=1` on the click-free pool, to 2600 (`docs/RUNS.md` run1468b):
    // run1468's game on every frame (rngcmp 2601 identical, 0 differing).
    "rontrace-run1468b.log",
    // Item 3847: the fourth melee (East Indies, three seats, a held
    // Information age, a sea fight, `tools/gamelog/golden/melee-sea.cmd`)
    // at `cover=1` on the click-free pool, to 2600 (`docs/RUNS.md`
    // run1483b): run1483's game on every frame (rngcmp 2601 identical, 0
    // differing).
    "rontrace-run1483b.log",
    // Item 3868: the fifth melee (the fourth's lobby with an Airbase a
    // computer, `tools/gamelog/golden/melee-airbase.cmd`) at `cover=1` on
    // the click-free pool, to 2600 (`docs/RUNS.md` run1492b): run1492's
    // game on every frame (rngcmp 2601 identical, 0 differing).
    "rontrace-run1492b.log",
    // Item 3879: item 3876's planted cell (East Indies, `--map 18`, run1156's
    // lobby, `tools/gamelog/coverage/weather3879-eastindies.cmd`) at
    // `cover=1` on the click-free pool, to 640 (`docs/RUNS.md` run1497):
    // `weather` at (2, 17) enters `TerrainOut::check_for_diagonal` on 620 at
    // (3, 16), the 80 printed writes `plant.py`'s; run1157's game on every
    // frame (rngcmp 641 identical, 0 differing).
    "rontrace-run1497.log",
    // Item 3858: two of item 3493's click-free style sweep (`docs/RUNS.md`
    // run1118, run1122), the default lobby at `--map 10` and `--map 15`
    // under `cover=1` on the click-free pool, to their `--end-frame` 36,
    // seed 12345, no twin taken. Each enters its map script on −1, before
    // any frame a twin compares: run1118 (Southwest Mesa, the one style with
    // a live cliff group) the cliff chain and `TerrainOut::has_plus_slope`,
    // run1122 (Warring States) `MapWarringStates::make_continents` and
    // `MapFairness::calc_distances` (`docs/MAPGEN.md` §18).
    "rontrace-run1118.log",
    "rontrace-run1122.log",
    // Item 3881: chapter fifteen's lobby with `611 add general who=0` in the
    // selection (`docs/RUNS.md` run1498; `docs/AI.md` §382.4), `cover=1` on
    // the click-free pool to 1000, seed 12345. The General's attack takes
    // `Unit::do_attack`'s unarmed, grouped arm and enters
    // `GroupData::get_max_range@00711080` on 735 (block 736); its twin,
    // run215, is the same game before the `add` line (rngcmp 617 identical,
    // the first differing 612, harness word 611).
    "rontrace-run1498.log",
    // Item 3889: the golden start (`--map 14`, run1129's lobby,
    // `tools/gamelog/coverage/river3889-start.cmd`) at `cover=1` on the
    // click-free pool, to 640 (`docs/RUNS.md` run1499): the console's
    // `river grow` at (32..34, 28) by `@aim`, then `river calc`, enters
    // `River::calculate@008858e0` on 623 (`docs/MAPGEN.md` §21); run1130's
    // game on every frame (rngcmp 641 identical, 0 differing).
    "rontrace-run1499.log",
    // Item 3898: the golden start (`--map 14`, run1129's lobby,
    // `tools/gamelog/coverage/fort3898-start.cmd`) at `cover=1` on the
    // click-free pool, to 640 (`docs/RUNS.md` run1502): `add fort who=1` on
    // 620 stands `1/2007` finished, `die 2007,1` on 622 runs `Build::close`,
    // which enters `Forts::close_fort@0073ec90` on 622 (`docs/AI.md` §387);
    // run1130's game on every frame (rngcmp 641 identical, 0 differing).
    "rontrace-run1502.log",
    // Item 3912: golden chapter one hundred and nine (`chapter109.cmd`,
    // chapter thirty-three's lines to its 2280 launch patrol) at `cover=1` on
    // the click-free pool to 2600 (`docs/RUNS.md` run1506b): `select 2007
    // who=0` on 2290, then `@key 124` (OPTION_AIRBASE_RECALL) on 2292 and
    // `@key 125` (OPTION_AIRBASE_SCRAMBLE) on 2312 with `--bindkey`, entering
    // `Options::recall_aircraft`, `process_recall` and `Group::action_recall`
    // on 2292, and `Options::scramble_aircraft`, `process_scramble` and
    // `Group::action_scramble` on 2312 (`docs/GOLDEN.md` §159).
    "rontrace-run1506b.log",
    // Item 3908: eleven more of items 3480's and 3493's click-free style
    // takes (`docs/RUNS.md` run1098, run1099, run1117, run1119–run1121,
    // run1123–run1126, run1128), item 3858's shape: the default lobby at
    // `--map 21, 8, 6, 11, 12, 13, 16, 17, 19, 20, 9` under `cover=1` on the
    // click-free pool, to their `--end-frame` 36, seed 12345, no twin taken.
    // Each enters its style's `make_continents` on −1, inside `Map::make`'s
    // +0x40 dispatch before any frame a twin compares, and run1098, run1125
    // and run1126 `Map::eliminate_edge_canals`, run1125
    // `Map::find_region_centroid` (`docs/MAPGEN.md` §22). No `NEVER` row is
    // entered by any of them.
    "rontrace-run1098.log",
    "rontrace-run1099.log",
    "rontrace-run1117.log",
    "rontrace-run1119.log",
    "rontrace-run1120.log",
    "rontrace-run1121.log",
    "rontrace-run1123.log",
    "rontrace-run1124.log",
    "rontrace-run1125.log",
    "rontrace-run1126.log",
    "rontrace-run1128.log",
    // Item 3913: golden chapter one hundred and ten (`chapter110.cmd`,
    // chapter nineteen's lines with `766 select 6 who=0` and `770 @key 124`,
    // `--bindkey OPTION_QUEUE=124`) at `cover=1` on the click-free pool, to
    // 900 (`docs/RUNS.md` run1507b): mid-hold, the key reaches
    // `Options::exec`'s 0xa6 arm and `GroupOut::issue_stop_spell@00708a20`
    // on 770, and the pump `Group::action_stop_spell@006fd7a0` (`docs/AI.md`
    // §391.2 row 6, `docs/GOLDEN.md` §160); run1507's game on every frame
    // (rngcmp 901 identical, 0 differing).
    "rontrace-run1507b.log",
    // Item 3916: golden chapter one hundred and thirteen (`chapter113.cmd`,
    // chapter thirty-nine's lobby and three `@key 124` presses on selected
    // Citizens, `--bindkey OPTION_PEASANT_ALARM=124`) at `cover=1` on the
    // click-free pool, to 900 (`docs/RUNS.md` run1510d): on 622 the key
    // reaches `Options::do_alarm@0071c3f0`, and the pump
    // `CommandPackage::process_alarm` → `Group::action_alarm` →
    // `Group::action_alarm_peasant@006fd980` (`docs/AI.md` §391.2 row 9,
    // `docs/GOLDEN.md` §163); run1510c's game on every frame (rngcmp 901
    // identical, 0 differing).
    "rontrace-run1510d.log",
    // Item 3918: golden chapter one hundred and fifteen (`chapter115.cmd`,
    // chapter thirteen's cast, a second Barracks in the City's city and three
    // `@queueup 0 666 1 <b>` presses of `DISBAND`) at `cover=1` on the
    // click-free pool, to 1100 (`docs/RUNS.md` run1512b): `0/2007`'s entry
    // completes and `Build::finished` → `SpellType::cast` →
    // `SpellType::cast_disband@00671ed0` → `Build::plunder` →
    // `Object::disband` on trace frame 831 (`docs/AI.md` §392.3,
    // `docs/GOLDEN.md` §165); run1512's game on every frame (rngcmp 1101
    // identical, 0 differing).
    "rontrace-run1512b.log",
    // Item 3919: golden chapter one hundred and sixteen (`chapter116.cmd`, a
    // who=0 General and a Hoplite squad two tiles off, then `@spell 0 637 -1
    // -1 0 0 6`) at `cover=1` on the click-free pool, to 1000 (`docs/RUNS.md`
    // run1513b): the cast lands on 680, its sixtieth frame, and enters
    // `SpellType::cast_entrench@00672820` (`docs/AI.md` §392.3,
    // `docs/GOLDEN.md` §166); run1513's game on every frame (rngcmp 1001
    // identical, 0 differing).
    "rontrace-run1513b.log",
    // Item 3930: golden chapter one hundred and eighteen (`chapter118.cmd`,
    // chapter one hundred and fifteen's staging and `840 @queueup 0 666 1
    // 2008` at the Barracks in city 0) at `cover=1` on the click-free pool,
    // to 1200 (`docs/RUNS.md` run1517b): the entry is laid on 841 and closes
    // the Barracks on 1051, and `CityData::is_disbanding@007380a0` is never
    // entered — `is_castable`'s city arm skips a `NO_CITY` type
    // (`docs/GOLDEN.md` §168, parked 3930a); run1517's game on every frame
    // (rngcmp 1201 identical, 0 differing).
    "rontrace-run1517b.log",
    // Item 3934: golden chapter one hundred and nineteen (`chapter119.cmd`,
    // chapter one hundred and eighteen's staging with a Temple in city 0 in
    // place of the second Barracks) at `cover=1` on the click-free pool, to
    // 1200 (`docs/RUNS.md` run1519b): the Temple (`jam`, no `NO_CITY`) takes
    // its `DISBAND` on 841 and closes on 1051, and `is_castable`'s city arm
    // enters `CityData::is_disbanding@007380a0` on 840 (`docs/GOLDEN.md`
    // §169); run1519's game on every frame (rngcmp 1201 identical, 0
    // differing).
    "rontrace-run1519b.log",
    // Item 3939: golden chapter one hundred and twenty-one (`chapter121.cmd`,
    // chapter one hundred and twenty's standing Envelop with the
    // `@attackadd` on a who=1 Citizen walked north, away from the Catapults)
    // at `cover=1` on the click-free pool, to 1000 (`docs/RUNS.md`
    // run1522b): the leader `0/6`'s sixteenth frame 714 inside
    // `[GROUP_MOVE][MOVE_TO][ATTACK]` enters `Group::is_moving_to@00713bd0`,
    // `Unit::repath` and `Group::kill_next_order` (`docs/GOLDEN.md` §171);
    // run1522's game on every frame.
    "rontrace-run1522b.log",
    // Item 3911: chapter seventeen's air group actions at `cover=1` on the
    // click-free pool (`docs/RUNS.md` run1505, run1505b;
    // `tools/gamelog/golden/chapter17-air.cmd` to 1100 and
    // `chapter17-airground.cmd` to 1060). run1505's `@patrol` on the
    // flying Fighter enters `Group::action_air_patrol` on 680 and its
    // `OPTION_RETURN` `Options::recall_aircraft`, `Group::action_recall`
    // and `Group::action_return` on 852; run1505b's `@attackground` enters
    // `Group::action_air_attack_ground` on 680 and its right-click with the
    // Airbase selected `GroupData::is_in_air_range` on 806
    // (`docs/GOLDEN.md` §158).
    "rontrace-run1505.log",
    "rontrace-run1505b.log",
    // Item 3914: golden chapter one hundred and eleven (`chapter111.cmd`,
    // chapter nine's Chariot with `620 select 6 who=0` and `622 @key 124`,
    // `--bindkey OPTION_DISBAND=124`) at `cover=1` on the click-free pool,
    // to 900 (`docs/RUNS.md` run1508b): the key reaches `Options::exec`'s
    // 0xf arm, `Options::do_disband@0071df20` and
    // `GroupOut::validate_disband@0070a4a0` on 622, and the pump
    // `Group::action_disband@0070e260` (`docs/AI.md` §391.2 row 7,
    // `docs/GOLDEN.md` §161); run1508's game on every frame (rngcmp 901
    // identical, 0 differing).
    "rontrace-run1508b.log",
    // Item 3915: golden chapter one hundred and twelve (`chapter112.cmd`,
    // the golden City selected and `610 @key 124` with
    // `--bindkey OPTION_VILLAGE_GATHER_ALL=124`) at `cover=1` on the
    // click-free pool, to 1300 (`docs/RUNS.md` run1509b, the second take):
    // the key reaches `Options::do_city_gather@0071cdb0` and
    // `GroupOut::issue_city_gather@0070ac70` on 610, and the pump
    // `CommandPackage::process_city_gather@00948a10` and
    // `Group::action_city_gather@00701780` (`docs/AI.md` §391.2 row 8,
    // `docs/GOLDEN.md` §162); run1509's game on every frame (rngcmp 1301
    // identical, 0 differing).
    "rontrace-run1509b.log",
    // Item 3917: golden chapter one hundred and fourteen (`chapter114.cmd`,
    // chapter twenty's lobby to 604 with `620 !eval create_unit(1, 62, 182,
    // "Hoplites", 1)` on the lake and `640 !eval create_unit(1, 34, 181,
    // "Hoplites", 2)` on land) at `cover=1` on the click-free pool, to 900
    // (`docs/RUNS.md` run1511b): the console's `eval` enters
    // `ScenarioFuncSet::create_unit@009f4c50`, `add_unit@009e2220`,
    // `Group::action_transport@00702620`, `Group::dbg_jump_to_action@00707a00`
    // and `Unit::dbg_jump_to_action@00602ea0` on 620 (`docs/AI.md` §391.2
    // row 10, §393.1 row 2; `docs/GOLDEN.md` §164).
    "rontrace-run1511b.log",
    // Item 3921: the golden start (`--map 14`, run1129's lobby) at `cover=1`
    // on the click-free pool with one line, `620 !eval
    // find_extra_starting_loc(4, 4, 1)`, to 640 (`docs/RUNS.md` run1515,
    // `docs/MAPGEN.md` §23): the console's `eval` enters
    // `ScenarioFuncSet::find_extra_starting_loc@009e5070` and
    // `MapConquest::check_starting_loc@0068d7e0` on 620; run1515c's game on
    // every frame (rngcmp 641 identical, 0 differing).
    "rontrace-run1515.log",
    // Item 3920: golden chapter one hundred and seventeen (`chapter117.cmd`,
    // chapter ninety-one's lobby with `606 add scout who=0`, `616 add spy
    // who=1` two tiles off and `620 @spell 0 631 6 1 25728 24960 6`) at
    // `cover=1` on the click-free pool, to 900 (`docs/RUNS.md` run1514b): the
    // Scout's own `DETECT` disc lights the Spy, the cast is issued, and
    // `SpellType::cast_counterintel@00673740` is entered on 658 (`docs/AI.md`
    // §392.3, `docs/GOLDEN.md` §167); run1514's game on every frame (rngcmp
    // 901 identical, 0 differing).
    "rontrace-run1514b.log",
    // Item 3924: Southwest Mesa (`--map 10`, run1493's default lobby) at
    // `cover=1` on the click-free pool with one line, `100 add mine who=0
    // 10,43`, beside who=0's cliff on a style with no mountain, to 200
    // (`docs/RUNS.md` run1516, `docs/AI.md` §395): `calc_gather`'s mine arm
    // enters `CliffsData::find_nearest@008a8da0` and
    // `CliffsData::gather_size@008a8f80` on 100.
    "rontrace-run1516.log",
];

/// **Off [`NEVER`], and entered only by the instrument** (item 934; parked
/// 953). Each is a `CommandManager::issue_*` out-of-line copy that nothing
/// in the executable calls or points at (item 940's scan): the game's
/// right-click reaches the inline twin in `GroupOut`, `BuildOut` or
/// `UnitOut`, and only the DLL's `@` line calls this address. A capture
/// under `cover=1` entered each, and so it left the blind list; what that
/// entry checked is the issuer's body as the instrument calls it, never a
/// path the game takes. Counted apart so the blind list's shrink says how
/// much of it is the game's own: 77 off with item 934, 69 of them the
/// game's. Of the other three of 940's eleven, `issue_gather` left with
/// run431 (item 1167, chapter forty's `@gather`), `issue_stance` with run1025
/// (item 3424, chapter sixty-four's `@stance` under `cover=1`), and
/// `issue_attack_ground` and `issue_come_out` with run1029 and run1030 (item
/// 3425, `docs/GOLDEN.md` §145).
pub const ENTERED_BY_THE_DLL_ONLY: &[(u32, &str)] = &[
    (
        0x0094_1580,
        "CommandManager::issue_form: run323, chapter fourteen's `@form`",
    ),
    (
        0x0094_1800,
        "CommandManager::issue_patrol: run319, chapter ten's `@patrol`",
    ),
    (
        0x0094_1910,
        "CommandManager::issue_set_transport: run329, chapter twenty's `@settransport`",
    ),
    (
        0x0094_1a70,
        "CommandManager::issue_garrison: run322, chapter thirteen's `@garrison`",
    ),
    (
        0x0094_1e70,
        "CommandManager::issue_follow: run321, chapter twelve's `@follow`",
    ),
    (
        0x0094_1ed0,
        "CommandManager::issue_guard: run320, chapter eleven's `@guard`",
    ),
    (
        0x0094_1f80,
        "CommandManager::issue_buildmask: run318, chapter twenty-three's `@buildmask`",
    ),
    (
        0x0094_2c40,
        "CommandManager::issue_unqueue (the building overload): run332, chapter twenty-five's `@unqueue`",
    ),
    (
        0x0094_1a20,
        "CommandManager::issue_gather: run431, chapter forty's `@gather`",
    ),
    (
        0x0094_1530,
        "CommandManager::issue_stance: run1025, chapter sixty-four's `@stance`",
    ),
    (
        0x0094_17a0,
        "CommandManager::issue_attack_ground: run1029, chapter three's lobby's `@attackground`",
    ),
    (
        0x0094_2c90,
        "CommandManager::issue_come_out: run1030, chapter thirteen's lobby's `@comeout`",
    ),
];

/// Every cited function no trace in [`TRACES`] enters, ascending — the
/// blind list itself: 228 against 1,120 cited on the corpus as it stood on
/// 2026-09-27, 227 with run314 (item 923; `Object::do_launch` off), **225**
/// with run316 (item 935; `Map::place_start_in_region` and
/// `Setup::large_city_buildings` off), **228** with item 928's three
/// citations (`CommandManager::issue_gather_point`, `CommandPackage::
/// process_gather_point`, `Group::action_gather_point`: an issuer chapter's,
/// which `cover=1` refuses, as run314 found), **151** with run318–337
/// (item 934: the issuer chapters under `cover=1`, 77 off, eight of them
/// [`ENTERED_BY_THE_DLL_ONLY`]), **152** with item 955's
/// `BuildData::get_first_gather` (`docs/GOLDEN.md` §40; run337 enters its
/// `num_gather` and the gather list's `add`, not this), **148** with run340
/// and run341 (item 959: row 6's staged economy, `lumber_level`,
/// `resource_cap_add`, `replace_gather` and `close_caravan` off), **149**
/// with item 947's `Group::action_launch_flight` (`docs/GOLDEN.md` §41),
/// **147** with run353 (item 965: a Build's killing blow on a caravan,
/// `new_danger` and `restart_trade_route` off), **148** with item 976's
/// `Group::action_launch_patrol` (`docs/GOLDEN.md` §42, run358 at
/// `cover=0`), **145** with run364–run367 (item 1011: chapters thirty-one to
/// thirty-four at `cover=1`; run366 enters `Group::action_launch_flight`,
/// `Group::action_launch_patrol` and `CommandManager::issue_launch_patrol`,
/// the last the game's own, called from `GroupOut::issue_launch_patrol`).
/// `BuildData::get_first_gather` stayed: its one caller in play,
/// `Build::train`, reaches it only for a missile or a Helicopter trained
/// under a gather point (`docs/CENSUS.md`, "item 1011"), which chapter
/// thirty-five stages; **146** with item
/// 1019's `LeaderData::bucket_set` (`docs/GOLDEN.md` §44, the `resource`
/// verb, before run371); **143** with run372 (item 1019: chapter
/// thirty-five at `cover=1`; `LeaderData::bucket_set`,
/// `BuildData::get_first_gather` and `Build::do_missile_launch` off, the
/// three rows the staging named); **144** with item 1050's
/// `ObjectData::is_in_range@0064e4a0` (`docs/PRODUCTION.md`, "The missile's
/// launch and round": a missile outside skips its test), which the tracer
/// cannot hook — `rontrace.funcs.excluded.txt`, "branch target inside the
/// displaced range" — so no capture takes it off.
/// **145** with item 1078's `Nuke::add_nuke@0092ba30` (`docs/GOLDEN.md`
/// §45: the nuke, dropped from chapter thirty-six with its reasons); the
/// chapter's other citations are entered already, and run390 at `cover=0`
/// takes nothing off. **144** with run398 (item 1091: chapter
/// thirty-seven at `cover=1`, `docs/GOLDEN.md` §46; `Nuke::add_nuke`
/// entered on 3200, the one row its staging named); **141** with run405
/// (item 1102: chapter thirty-eight at `cover=1`, `docs/GOLDEN.md` §47;
/// `UnitData::is_flying_low@0060a140` and `is_flying_high@0060a310` on
/// 730, `Ammo::init_crash@0067b800` on 1006, the three rows its staging
/// named); **142** with item 1117's `Guy::set_pivot_angle@005d8fc0`
/// (`docs/COMBAT.md` §85.2: turret 0 pointed ahead when no node bears),
/// dead in the image — `Unit::move_step` carries it inline — and so on
/// [`RESIDUE`] too. **139** with run423 (item 1111: chapter thirty-nine at
/// `cover=1`, `docs/GOLDEN.md` §48; `SpellType::cast_to_arms@00670880` on
/// 625, `cast_civilian@006704a0` on 665 and `cast_create_decoy@00674370`
/// on 820, the three rows its staging named); **141** with the two §48
/// cites that no trace enters: `Group::action_alarm@0070ec30`, the City's
/// alarm, an issuer no chapter stages, and `Options::do_spell@0071d7a0`,
/// the button the DLL's `@spell` stands in for; **138** with run431 (item
/// 1167: chapter forty at `cover=1`, `docs/GOLDEN.md` §49;
/// `Group::action_gather@00700b90` on 650 and `Group::action_alarm@0070ec30`
/// on 880, the two issuers its staging named, and `CommandManager::
/// issue_gather@00941a20`, [`ENTERED_BY_THE_DLL_ONLY`]); **140** with item
/// 1174's escrow writers no capture reaches, `Leader::action_respond@006d03c0`
/// and `Leader::pay_dow@006d2b10` (`docs/AI.md` §98.6); **138** with run438
/// (item 1182: chapter forty-one at `cover=1`, `docs/GOLDEN.md` §50;
/// `UnitData::get_speed@006086f0` on 778, the computer's sortie, and
/// `ObjectData::is_siege@0046ef90` on 1088, the trireme's spill onto the
/// repairing Citizen — the two rows of CENSUS row 7 a staging reaches;
/// three more of the row are [`RESIDUE`]); **137** with run461 (item
/// 1209: chapter forty-two at `cover=1`, `docs/GOLDEN.md` §51;
/// `Unit::resolve_block@005fccc0` on 621, the Knight's `TAKE` on the
/// ring's corner); **138** with item 1241's cite of
/// `BuildTypeData::get_enhancing_good@00639880` (`docs/AI.md` §99.7): the
/// Granary it decides is placed on run470 and run476, both at `cover=0`;
/// **140** with item 1305's cites of `SpellType::cast_march@00671500` and
/// `Leader::verify_spell_flags@006ce190` (`docs/AI.md` §99.13): run470
/// casts the march on 9112 and holds 9240's coin by its flag, but at
/// `cover=0` its trace enters no function; **141** with item 1365's cite
/// of `LeaderData::get_vehicle_speed_upgrade@006da8a0`
/// (`docs/PRODUCTION.md`, "The tail's first caller"): no traced game
/// trains a tank, siege or vehicle unit; **142** with item 1454's cite of
/// `Wonder::init@0073c5e0` (`docs/TECH.md`, "A built wonder is built for
/// everyone"): the second writer of `Game::wonders`, which no traced game
/// reaches. **144** with item 1528's cites of
/// `Army::charge@006f9a90` and `WallData::hits@00642bb0` (`docs/AI.md` §142):
/// run668 shows the charge's orders and the city's stamp on the block the
/// word's frame writes, but `cover=0` traces enter no function. **145**
/// with item 1583's cite of `LeaderData::get_nukes@006ebe50` (`docs/AI.md`
/// §158): read off its listing, and no trace enters it. **146** with item
/// 1602's cite of `Object::disband@006455c0` (`docs/AI.md` §168): run711's
/// original disbands the Oil Platform site `1/2048` on 2048, but its
/// tracer does not hook the function (`rontrace.funcs` lacks it).
/// **148** with item 1625's cites of `Spline::set_min_seg_length@009125e0`
/// and `Spline::calc_from_dir@00913960` (`docs/AI.md` §174): run717's
/// `AMMO` prints the spline they build on every one of `1/49`'s rounds,
/// but its `cover=0` trace enters no function.
/// **331** with item 2107's cite of
/// `BuildQueueData::get_next_non_caravan@00630c20` (`docs/AI.md` §211):
/// run834's queue shows the Merchant it advances, and no trace enters it.
/// **332** with item 2165's cite of `CommandManager::issue_stance@00941530`
/// (`docs/GOLDEN.md` §79): run835's `@stance` calls it and the dump logs
/// `process_stance`, but its `cover=0` trace enters no function; a
/// `cover=1` take would move it to [`ENTERED_BY_THE_DLL_ONLY`].
/// **333** with item 2287's cite of `Farms::add_animals@008d92d0`
/// (`docs/PARKED.md` 2289): the no-argument overload has no port, and no
/// trace enters it.
/// **335** with item 2320's cites of `Game::say_no_war@00592b00` and
/// `Game::current_age@005946d0` (`docs/AI.md` §218, `docs/PARKED.md` 2321):
/// the rush rules' arms, which no pinned trace sets; the peace pair's
/// lobby reaches them at `cover=0`, so a `cover=1` take would enter them.
/// **336** with item 2341's cite of `Group::kill_group_move@007123f0`
/// (`docs/PARKED.md` 2345): `Unit::repath`'s grouped-`GROUP_MOVE` arm,
/// swept under the emulator with it stubbed, and no trace enters it.
/// **337** with item 2474's cite of `Console::hurry_action@007c3810`
/// (`docs/GOLDEN.md` §90): ~~run860's `finish` calls it~~ only `hurry`
/// (`run_cmd` case `0x51`) and cheat key `0x45` call it, and no capture
/// has issued either (item 3408, `docs/GOLDEN.md` §144.2).
/// **338** with item 3058's cite of `TurnControl::end_cannon_time@00956470`
/// (`docs/journal/2026-10-17-item-3058.md`): swept under the emulator with its
/// callees stubbed, and no trace ends a cannon time.
/// **367** with item 3297's cite of `SpellType::cast_ambush@00671610`
/// (`docs/AI.md` §261): The Despot's Ambush, whose `0x2000` run943's
/// `unit_masks` print, but no pinned trace enters the cast.
/// **372** with item 3387's cite of `SpellTypeData::find_target@00674920`
/// (`docs/PARKED.md` 3387d): swept under the emulator with `Console::find_target`
/// stubbed, and no pinned trace casts a spell search.
/// **373** with item 3409's cite of `Leader::action_declare@006dab50`
/// (`docs/AI.md` §98.6): `pay_dow`'s one caller, which a skirmish at war
/// from frame 0 leaves at its `is_enemy` head, and no pinned trace enters.
/// **378** with item 3412's cites of `LeaderOut::issue_sell@006d3eb0`,
/// `CommandManager::issue_sell@00942a80`, `LeaderData::can_buy_sell@006d53e0`,
/// `Options::do_declare_war@007216b0` and
/// `ConquestGame::build_conquest_game@00797c40` (`docs/NATIONS.md` §9.1):
/// the human's Sell and Buy, an unaffordable declaration of war, and
/// Conquer the World's start, none of which a pinned trace takes.
/// **380** with item 3414's cites of `Game::run_playback@00586ea0`
/// (`docs/ATTRITION.md`, the never-entered functions) and `Wonder::close@0073c5d0`
/// (`docs/TECH.md`): a recorded game's playback, which no pinned trace
/// plays, and the function whose `ret` ends where the dead `Wonder::init` begins.
/// **376** with run1016 (item 3417: the peace pair's lobby at `cover=1` to
/// 600, `500 unmeet who=1`): `Game::current_age@005946d0` from frame 0
/// under `RUSH_RULES 10`, `Leader::treaty_off@006d0370` at 500 (on a clear
/// bit: the two leaders meet at 1474), and, unasked,
/// `BuildTypeData::get_enhancing_good@00639880` (frame 7) and
/// `LeaderData::get_vehicle_speed_upgrade@006da8a0` (frame 582).
/// **372** with run1017 (item 3418: Great Sahara at Toughest at `cover=1`
/// to 15432; `docs/AI.md` §298): `cast_march` on 9112,
/// `verify_spell_flags` on 9263, `UnitData::is_gov_hero@0046ce70` on 11882
/// and `Army::charge` on 15377 (it enters run1016's `get_enhancing_good`
/// and `get_vehicle_speed_upgrade` too, on 5376 and 8782).
/// `WallData::hits@00642bb0` stays: it is only the `+0x11c` slot of
/// `Wall`'s three vtables, the struck city's `hits(0)` dispatches to
/// `BuildData::hits`, and no traced game constructs a kind-2 object
/// (`Wall::Wall@0063e390`, also here).
/// **363** with run1019 (item 3420: the coverage pair at `cover=1` to
/// 3150, run717's recipe; `docs/AI.md` §296 row 3): the four it was taken
/// for, `LeaderData::can_transport` on 1182, `Object::disband` on 1419 and
/// both `Spline` functions on 2933, and five unasked:
/// `Unit::check_build_order@00603470` (193), `LeaderData::get_nukes@006ebe50`
/// (780), `Unit::reset_move_orders@005fd080` (2458),
/// `Armies::send_navy@006f2c90` (2812) and `Army::use_spies@006f4af0` (3096).
/// **360** with run1020 (item 3421: Great Sahara in the coverage pair's
/// lobby at `cover=1` to 3770, run790's recipe; `docs/AI.md` §296 row 4):
/// `BuildQueueData::get_next_helicopter@00630bb0` on 3525, the one of the
/// two asked it could enter, and two unasked:
/// `BuildQueueData::get_next_non_unit@00630b10` (3525) and
/// `Units::make_valid@0061a960` (3546). `Unit::think_supply@005f4730` has
/// no caller (`Unit::think` adds a supply wagon to its army inline).
/// **358** with run1018 (item 3419: Great Lakes, the Iroquois at difficulty
/// 3, at `cover=1` to 12658; `docs/AI.md` §296 row 2, §299):
/// `Army::use_scouts@006f49a0` on 5144 and `SpellType::cast_ambush@00671610`
/// on 10520 (it enters run1017's `cast_march` and `verify_spell_flags` too,
/// on 7960 and 8111, and the Ambush's leader bit falls on block 10972).
/// **356** with run1021 (item 3422: the successor pair's lobby at `cover=1`
/// to 2210; `docs/AI.md` §296 row 5): `Unit::think_carry@005f2ee0` and,
/// unasked, `Unit::think_carry_civilian@005f37c0`, both on 2178, the idle
/// Freighter `1/72`'s walk. `think_carry_idle@005f2f30` and
/// `think_carry_military@005f3260` stay: every boat to 2210 carries a
/// worker.
/// **354** with run1022 and run1024 (item 3423: two Great Sahara retakes
/// at `cover=1`; `docs/AI.md` §296 rows 6 and 8):
/// `BuildQueueData::get_next_non_caravan@00630c20` on 2995, the Mongols'
/// Merchant behind a held Caravan, and `Unit::go_to_city@005f79c0` on
/// 309, the Dutch merchant drafted to its city. Row 7's
/// `BuildData::can_queue@0062da00` stays: its one caller is
/// `GroupOut::issue_queue_up` (a click), and the script's research calls
/// the identical copy at `004d2030`.
/// **350** with run1025 and run1026 (item 3424: golden chapters sixty-four
/// and ninety-one at `cover=1`; `docs/GOLDEN.md` §144.3):
/// `CommandManager::issue_stance@00941530` and
/// `CommandPackage::process_stance@00949ed0` on 680,
/// `GroupData::get_stance_option@0070bab0` on 720 (the `-1` cycle), and
/// `SpellType::cast_sabotage@006720e0` on 806; `issue_stance` moves to
/// [`ENTERED_BY_THE_DLL_ONLY`].
/// **344** with run1029 and run1030 (item 3425: two `@` lines staged at
/// `cover=1`, `docs/GOLDEN.md` §145): `CommandManager::issue_attack_ground@
/// 009417a0`, `CommandPackage::process_attack_ground@009494a0` and
/// `Group::action_attack_ground@00704520` on 630; `issue_come_out@00942c90`
/// and `process_come_out@009465d0` on 800; and, unasked, the `RESIDUE` row
/// `TypeData::get_desc_parsed@006626e0` on 511, the resource tooltip under
/// the resting cursor (`MouseOver::set_good`).
/// **339** with run1031 (item 3426: the human's Market, `docs/NATIONS.md`
/// §9.2, the tracer's `@sell`/`@buy`): `LeaderOut::issue_sell@006d3eb0`,
/// `CommandManager::issue_sell@00942a80` and
/// `CommandPackage::process_sell@009468a0` on 620, `LeaderData::
/// can_buy_sell@006d53e0` and `has_bonus@006e0b80` on 640 (the Nubian arm,
/// before Coinage). The DLL calls the leader issuer, the Sell and Buy
/// buttons' own call (`Options::exec@007188c0:1149`, `:1162`), so neither
/// issuer is the DLL's only.
/// **331** with run1032, run1033 and run1034 (item 3427: the click layer's
/// first pass, driven at `cover=1`; `docs/GOLDEN.md` §144.3):
/// `GroupOut::issue_guard@007088e0` on 624 (a right-click on an own unit),
/// `GroupOut::issue_follow@00708980` and `Options::picked_spot@00721c40`
/// on 708 (the Move button, then a pick on a unit),
/// `BuildData::can_queue@0062da00` on 736 (a train click, parked 3423a),
/// `GroupOut::issue_eject_all@00708b90` and `Options::do_eject_all@0071c470`
/// on 748, and `GroupOut::issue_set_transport@0070ad10` and
/// `Options::do_transport@0071c500` on 626.
/// **329** with run1036 (item 3428: run349's recording played back at
/// `cover=1`, `docs/ATTRITION.md`'s owed capture): `Game::run_playback@
/// 00586ea0` and `RecordGame::read_package@00952d90`, both before the first
/// `FRAME` record, on `process_turn`'s playback arm; nothing else.
/// **321** with run1044, run1045, run1048, run1049, run1055 and run1056
/// (item 3436: the click layer's second pass, driven the same way):
/// `GroupOut::issue_attack@0070b060` on 736 (a right-click on an enemy),
/// `GroupOut::issue_flight@00708b10` on 656 (a plane's right-click on its
/// own Airbase), `Options::set_air_repeat@0071c740` and
/// `GroupOut::issue_buildmask@00708820` on 788 (the Airbase's repeat
/// button), `BuildOut::issue_unqueue@00629e70` on 876 (a click on the
/// Barracks' Hoplite in training), `GroupOut::issue_build@00708c60` on 700
/// (the Barracks build button and a drop) with, unasked,
/// `BuildTypeData::is_civilian@00639930` beside it, and
/// `GroupOut::issue_swarm_around@0070afc0` on 796 (a Citizen's
/// right-click on its own Barracks).
/// **319** with run1054 (item 3438: golden chapter eighty-five at
/// `cover=1`, `docs/GOLDEN.md` §108, §144.3): `SpellType::cast_bribe@
/// 00672d00` and `Unit::swap_team@006118d0` on 1212, the cast's frame.
/// **315** with run1057 (item 3442: the pool's keystroke, `@key` through
/// `SysMessageHandler::OnKey`, three keys bound by `--bindkey`):
/// `Options::do_formation@007215b0` on 740 (FORM_E_RIGHT),
/// `GroupData::get_form_option@0070beb0` on 800 (OPTION_FORM's "next",
/// under `Group::action_form`), and `Options::do_rotate@0071dfd0` with
/// `GroupData::get_unit@0070cc90` (the rotation's −2 arm) on 860.
/// `docs/CENSUS.md`'s "The blind list, ranked" groups it by the staging
/// that would enter each family.
/// **311** with run1058, run1059 and run1060 (item 3443: three clicks on
/// the pool, `docs/GOLDEN.md` §144.3, `docs/NATIONS.md` §9.2):
/// `Options::do_spell@0071d7a0` on 624 (To Arms on a Citizen's card),
/// `LeaderOut::warn_resources@006d3650` on 736 (an unaffordable train
/// button), and `CommandManager::issue_trade@00941960` with, unasked,
/// `Group::action_trade@00701cc0` on 650 (a Caravan's right-click on a
/// second city).
/// **310** with run1061 (item 3444: chapter nine's lobby, `docs/GOLDEN.md`
/// §144.3): `WorldMap::on_right_up@008c7050` on 620, a plain right-click
/// on the minimap, issued as `process_move_to` form −1, width −1.
/// **309** with run1062 (item 3444: the peace pair's lobby, `docs/NATIONS.md`
/// §9.2): `Options::do_declare_war@007216b0` on 1484, the diplomacy window's
/// Declare button, refused at the team-style test (`TEAM_STYLE 1`).
/// **306** with run1063a, run1063 and run1065 (item 3445: chapter
/// seventy-four's lobby on the pool, `docs/GOLDEN.md` §91, `docs/ORDERS.md`
/// §42.1): `BuildQueue::set_job_counter@00630ab0` on 701 (`finish`, case
/// 0x4f), `Console::hurry_action@007c3810` on 701 (`hurry`, case 0x51), and
/// `OptionsWinGame::do_save@0082c520` on 702 (the Options window's OK).
/// **304** with run1064 (item 3445, `docs/GROUPS.md` §38): the city rename
/// on 706, `CommandPackage::process_rename_city@00944090` and
/// `HotKeyGroups::find_group@00714d00`.
/// **303** with run1068 (item 3447: the Nomad lobby on the pool,
/// `docs/GOLDEN.md` §148): `ScenarioFuncSet::citizen_repair_order@009f85b0`
/// on 376, the computer's three Citizens re-sent to their Small City.
/// **299** with run1066 (item 3446: the coverage pair's lobby, the carrier,
/// `docs/GOLDEN.md` §147): `Unit::action_unqueue@005e1f20` on 620 (the
/// carrier's cancel by `@unqueueunit`, `num_queued` 2 → 1),
/// `Unit::think_carry_military@005f3260` on 770 and
/// `think_carry_idle@005f2f30` on 793 (who=1's soldier barge and General
/// barge at their sea points), and `UnitData::has_repeat_air@0046cec0` on
/// 831, the carrier's `Object::do_launch` before the plane's strike.
/// **291** with run1069 and run1070 (item 3448, `docs/ORDERS.md` §42.3):
/// `Unit::add_think_order@005e3df0` on 621 and `do_think_order@005e5bf0`
/// on 622 with the think order's own `walk_data`, `clear` and `get_type`;
/// `Group::distribute_attack@00713390` on 686 with both `kill_group_move`s.
/// **290** with run1072 (item 3453: the peace pair's lobby, `docs/AI.md`
/// §296 row 9b): `Game::say_no_war@00592b00` on 1392, through
/// `Unit::process_attrition` for the human's squad in the Lakota's land
/// before the peace ends (leader 0's `attrition_stamp` 0 → 1392).
/// **289** with run1074 (item 3454: chapter seven's lobby, `docs/AI.md`
/// §75.3): `Wonders::close_wonder@0073c7e0` on 660, a console `die` on the
/// lower of two Pyramids.
/// **288** with run1077 (item 3456: the team lobby, `docs/ARMY.md` §26):
/// `Leader::action_ping@006d18a0` on 5422, a human's ping 11 frames after a
/// help taunt to the computer ally.
/// **286** with run1075b and run1076 (item 3455, `docs/ARMY.md` §6 and
/// `docs/VISION.md` §8): `Army::send_here@006f98a0` on 2296, the disband
/// arm of `Army::process` for the computer's navy whose nine ships died on
/// 2284 (army 2's record gone from the 2297 whole dump); and
/// `ObjectData::is_dock@004711e0` on 615, the human's Trireme right-clicked
/// onto the human's own Dock (`process_garrison 2007 0 2 616`).
/// **283** with run1080 (item 3461: the carrier lobby, CENSUS row 820):
/// `CommandPackage::process_halt@00949140` on 620 (a bound OPTION_HALT key,
/// `CM` `process_halt 621`), `GroupsData::find_group_const@007084c0` on 640
/// (a bound OPTION_CYCLE_GENERAL key) and `WorldData::is_tocean@0046fb10` on
/// 660 (a carrier right-clicked on open sea, `orders_x/y` (648, 9480)).
/// **281** with run1078 (item 3460: the coverage pair's lobby at
/// `TEAM_STYLE 0`, `docs/AI.md` §296 row 12): `Leader::action_declare@006dab50`
/// and `Leader::pay_dow@006d2b10` on 910, a human's declaration through the
/// diplomacy window, paid (the third good's `bucket` 640 down on block 912).
/// **280** with run1081 (item 3462: golden chapter eighty-five's lobby):
/// `SpellTypeData::find_target@00674920` on 1112, the Bribe pick armed by a
/// bound OPTION_SPY_BRIBE key (`keymap_options[0x179]` = option 4, 629).
/// **278** with run1084 (item 3464: the golden start, CENSUS row 840):
/// `World::get_wdata@0046d220` on 600 (`!cursor tinfo`, console case 0x3d)
/// and `WorldData::is_forest@006b5560` on 602 (`!startareas`, case 0x60);
/// **279** on the tree merged with `b41737e50`, whose Loop 3473 cites
/// `Compiler::eval_command@009befe0`, which no trace enters.
/// **278** with run1082 (item 3463: the first pair's East Indies lobby,
/// `docs/AI.md` §2.14): `Leaders::best_human_age_stamp@006b81e0` on 9979,
/// the computer's age techs paced against a human three ages ahead;
/// **277** with run1086 (item 3470: the team lobby's food taunt to a
/// computer ally, `docs/AI.md` §296 row 11): `Leader::action_respond@006d03c0`
/// on 661.
/// **273** with run1088 (item 3471: the coverage pair's lobby, the console):
/// `Compiler::eval_command@009befe0` and `MathUtilFuncSet::rand_real@009e18b0`
/// on 610 (`!eval rand_real()`), `ScenarioFuncSet::set_bonus_cap@009fbd60` on
/// 620 (leader 0's `bonus_cap[2]` 200 from block 621) and
/// `ObjectData::in_a_ship@006440c0` on 630 (`!eval is_on_transport(1, 15)`).
/// **255** with run1092 (item 3477: the golden start, the script host's
/// seventeen `ScenarioFuncSet` rows through `!eval` from 670, `docs/AI.md`
/// §296's struck proposal), each on its line's frame — who 0's
/// `base_rate[0]` 80 from block 683, `leader_flags2` 8 and 10 from 697 and
/// 699, who 1's 16 from 703 — and `Tribes::find@006ef2d0` on 704
/// (`disable_type_by_tribe` on two names that match nothing).
/// **254** with run1096 (item 3479: the golden start, the param console):
/// `Nuke::on_field_change@00925cb0` on 602, a Return in the window's modal
/// loop with Nuke picked off its category pulldown.
/// **253** with run1109 (item 3481: the team lobby, herd 9 under `camera
/// 186,190`): `AnimalData::is_seen@005d8160` on 5406, once `5405 reveal 0`
/// cleared the semaphore bit that the lane profile's `StartReveal=1` sets.
/// `ObjectsOut::sort_all` skips vtable +0x48 while `semaphore & 0x42` holds
/// (the packet run1100, `docs/RUNS.md`).
/// **252** with run1113 (item 3491: the team lobby, a defeated computer's
/// chair): `Game::is_tournament@0043ec90` on 604, once `be 2` put the
/// console on a player `is_over` and `reveal 0` cleared `semaphore & 0x42`.
/// **250** with run1115 (item 3492: the golden start with `AutoSaveLoad=650`):
/// `Good::walk_data@0066e5d0` and `Item::walk_data@00677150` on 649's
/// do_frame, the save's and the reload's +0x7c walkers.
/// **248** with run1129 (item 3494: the golden start, the food counter's
/// hover re-sent at `describe_mouseover`): `Game::is_turbo_resources@0043ecb0`
/// and `LeaderData::resource_cap_get@0046ee80` on 600, through
/// `MouseOver::describe` case 0x200 and `Options::describe@00724460`.
/// **247** with run1134 (item 3501: the golden start, the script host's
/// `disable_type_by_tribe` and `train_unit`): `TypeData::type_graft@00472a50`
/// on 672, `get_graft`'s `+0x68` on a Barracks the Nubians may not build.
/// **246** with run1136 (item 3502: the golden start, a catapult's
/// Alt-right-click on a Barracks): `CommandPackage::process_siege_attack@00949ae0`
/// on 612, the type-5 command `GroupOut::issue_siege_attack` built under a
/// held Alt.
/// **247** with item 3504's park (`docs/PARKED.md` 3504a, no capture joins
/// [`TRACES`]): `Group::kill_group_order@00710aa0` is newly cited, the one
/// caller of `Unit::kill_group_order@005e3310`, and runs 1139–1142 never enter it.
/// **248** with item 3693 (no capture joins [`TRACES`]): `GroupData::get_max_range@00711080`
/// is newly cited (`docs/AI.md` §348.3, `do_attack`'s grouped arm), and no trace enters it.
/// **249** with item 3708 (no capture joins [`TRACES`]): `Forts::close_fort@0073ec90` is
/// newly cited (`docs/PARKED.md` 3708b: a fort's death or capture takes it off the list),
/// and no trace enters it.
/// **256** with item 3734 (no capture joins [`TRACES`]): `docs/MAPGEN.md` cites
/// `Map::make_continents@00694ab0` (the base mask, no corpus style calls it),
/// `Map::force_continent_spacing@0068e6a0`, `MapFairness::calc_distances@0068a1c0`,
/// `Cliffs::place_cliff@008a7ea0`, `Cliffs::position_cliff@008a8bc0`,
/// `Cliffs::verify_defensive_position@008a8680` and `TerrainOut::has_plus_slope@00869790`
/// — no corpus tileset places a cliff group, and no traced grid gives `assess_wcoord` a 0x40 cell.
/// **263** with item 3856's cite of the routes around them (no capture joins [`TRACES`]):
/// `Map::run_map_script@0068c870`, `MapScenario::make@0068c8f0`,
/// `MapWarringStates::make_continents@00698b60`, `Cliffs::verify_placement@008a6310`,
/// `Cliffs::add_cliff@008a6690`, `ScenarioEditor::generate_map@009a0950` and
/// `ScenarioEditor::import_image@009a3f30`; run1118 (Southwest Mesa) and run1122
/// (Warring States) are archived traces that enter five of 256's seven, not yet here.
/// **255** with run1118 and run1122 (item 3858: two of item 3493's style sweep, joined with
/// no capture), each row entered on −1: run1118 (Southwest Mesa) `TerrainOut::has_plus_slope`,
/// `Cliffs::verify_placement`, `Cliffs::add_cliff`, `Cliffs::place_cliff`,
/// `Cliffs::verify_defensive_position` and `Cliffs::position_cliff`; run1122 (Warring States)
/// `MapFairness::calc_distances` and `MapWarringStates::make_continents` — five of 256's
/// rows and three of 263's.
/// **258** with item 3859's cite of the second route to `force_continent_spacing`'s one
/// caller and of the style writers outside the skirmish lobby (no capture joins [`TRACES`]):
/// `MapConquest::make_continents@0068dba0`, `GameSpy::UpdateLobbyFromDTO@00962c70` and
/// `ScenarioRead::import@009b1430`.
/// **261** with item 3861's cite of the storm's registrations and the parameter window's
/// scroll handler (no capture joins [`TRACES`]): `Storm::registration@00842060`,
/// `StormOut::registration@00842070` and `ParamRegisterWindow::on_scroll@009343b0`.
/// **262** with item 3863's cite of the diagonal-transition arm (no capture joins [`TRACES`]):
/// `TerrainOut::check_for_diagonal@008595f0`, kept out with `WorldData::is_valid@00471170`
/// by `docs/MAPGEN.md` §19's predicate, which a blind second reading owes.
/// **264** with item 3864's cite of the transition rebuild's callers (no capture joins [`TRACES`]):
/// `TerrainOut::generate_transitions@00861960` and `River::calculate@008858e0`; the second
/// reading agrees on §19's predicate, and the arm stays open on parked 3863a's depth leak.
/// **264** unmoved by the sixty-sixth pass, which ratified ten open rows as [`RESIDUE`]
/// (DECISIONS 106): 3859a's six on scope (the editor, Conquer the World's mask, GameSpy,
/// scenario play, and the console's `map` command clamped to 0..22), 3861a's three (the
/// debug parameter window, out of v1 with the renderer it tunes) and the scan's one dead row
/// (`TerrainOut::generate_transitions@00861960`); residue 213 → 223, open 51 → 41.
/// **263** with run1498 (item 3881: chapter fifteen with a General in the selection):
/// `GroupData::get_max_range@00711080` on 735, `do_attack`'s unarmed, grouped arm; open 41 → 40.
/// **262** with run1497 (item 3879: item 3876's planted cell on East Indies, `weather` at
/// (2, 17)): `TerrainOut::check_for_diagonal@008595f0` entered on 620 at (3, 16);
/// `WorldData::is_valid@00471170` not, so it stays; open 40 → 39.
/// **266** with item 3884's cite of the destructor chain whose stores-only windows keep the
/// `Wall` and `Object` tables off every dispatch (`docs/AI.md` §383.2–§383.3; no capture joins
/// [`TRACES`]): `WallData::~WallData@00472cc0`, `WallOut::~WallOut@00472d10`,
/// `Wall::~Wall@0063e440` and `Object::~Object@00646f50` — inlined on every live path, their
/// out-of-line callers only unwind funclets and the wall-range vector constructors (§383.6,
/// [`RESIDUE`] candidates on a second reading); open 39 → 43.
/// **266** unmoved by item 3887's blind second reading (`docs/audit/2026-11-05-item-3887.md`),
/// which agrees on all fifteen of `docs/AI.md` §382.6's and §383.7's candidates —
/// `Unit::do_group_attack`, `Unit::do_await_board`, the nine data accessors and the four
/// destructors — and moves them to [`RESIDUE`]; residue 223 → 238, open 43 → 28.
/// **265** with run1499 (item 3889: the console's `river grow` ×3 and `river calc` on the
/// golden start): `River::calculate@008858e0` entered on 623; open 28 → 27.
/// **266** with the sixty-seventh pass (DECISIONS 107 §3): `TerrainMap::river_stop@009b5d20`
/// is cited (`docs/MAPGEN.md` §21, parked 3889a) and ratified [`RESIDUE`] on scope with seven
/// open rows — the editor's four (`TerrainMap::on_left_up`, `on_right_up`,
/// `ScenarioEditor::set_transport`, `TypeData::is_item_type`), Conquer the World's
/// `MapConquest::check_sea_map`, a recording's forged byte 8 (`process_move_near`, 3505d) and
/// the scan's one dead row (`World::set_orig_coast`, its RVA's one match a call's rel32);
/// residue 238 → 246, open 27 → 20. The scan over all 27 open rows: 0 dead but that one,
/// which `report.py` called "pointer only" on the rel32 and `pe.references` excludes.
/// **268** with item 3891's blind second reading (`docs/audit/2026-11-05-item-3891.md`; no
/// capture joins [`TRACES`]), which agrees on `docs/MAPGEN.md` §18.1's and §20's four
/// candidates — `WorldData::is_valid`, `MapScenario::make`, `Map::run_map_script` and
/// `ScenarioRead::load_unit_chunk` — and moves them to [`RESIDUE`], and whose cite of
/// `load_unit_chunk`'s route adds `ScenarioRead::load_objects_chunk@009ab9c0` and
/// `ScenarioRead::load_scenario_screen@009b2e40`, scenario-only on both readings and
/// [`RESIDUE`] with them; residue 246 → 252, open 20 → 16.
/// **268** unmoved by item 3893's blind second reading (`docs/audit/2026-11-05-item-3893.md`),
/// which agrees on all eight of `docs/AI.md` §384's slot rows — `ObjectData::support`,
/// `WallData::can_make`, `GatherPoint::walk_data`, `SpecialAnimOrder::walk_data` (the one
/// disagreement, settled to `do_spec_anim`'s same-call kill), `ObjectsData::find_town_at`,
/// `TargetOrder::get_target`, `SubObjectOut::is_on_screen` and `ObjectTypeOut::draw` — and
/// moves them to [`RESIDUE`]; residue 252 → 260, open 16 → 8.
/// **268** unmoved by item 3897's blind second reading (`docs/audit/2026-11-05-item-3897.md`),
/// which agrees on all four of `docs/AI.md` §385's rows — `TurnControl::end_cannon_time` on
/// scope, `wincpuid`, `wincpufeatures` and `check_80386` on reach — and moves them to
/// [`RESIDUE`]; residue 260 → 264, open 8 → 4.
/// **267** with run1502 (item 3898: a who=1 Fort added and killed by console `die` on the
/// golden start): `Forts::close_fort@0073ec90` entered on 622; open 4 → 3 on the tree merged
/// with item 3897.
/// **267** unmoved by item 3899's blind second reading (`docs/audit/2026-11-06-item-3899.md`),
/// which agrees with `docs/AI.md` §386 that no v1 game reaches `GoodTypeData::is_flat`, and
/// moves it to [`RESIDUE`]; residue 264 → 265, open 3 → 2 on the tree merged with 3897 and 3898.
/// **267** unmoved by item 3902's blind second reading (`docs/audit/2026-11-06-item-3902.md`),
/// which disagrees with `docs/AI.md` §388 on the `kill_group_order` pair: `fight`'s second
/// `find_new_target` (`5fe25f`) after `Group::action_attack(QUEUE_FIRST)` can meet a re-issued
/// `GROUP_PATROL`, so `Unit::kill_group_order@005e3310` and `Group::kill_group_order@00710aa0`
/// stay here (in [`RESIDUE`] since item 3940), the take that settles them parked as 3902a; open 2, unmoved, on the tree merged
/// with 3899.
/// **267** unmoved by item 3903 (`docs/AI.md` §389), which reads 3902a's route shut with no take:
/// its cast is the Machine Gun line, `Object::valid_target` refuses that line a capture-eligible
/// city, any other target leaves the unit an `ATTACK`, and `fight`'s second-call gate is shut
/// for a land building besides; the pair stays here pending a blind second reading (park 3903a)
/// and route 2 (3903b); open 2, unmoved.
/// **267** unmoved by item 3904 (`docs/AI.md` §390), which reads route 2 half-open: a regroup
/// keeps the old group's move queued and two slots' group-move ids meet, so the duplicate list
/// has a human producer, but no entry into `find_new_target` was found for that list: with a
/// group move current only `do_move:105` admits it, behind a stash; neither a take nor
/// `RESIDUE`, the entry parked as 3904a; open 2.
/// **297** with item 3906's cite of the implicit residue's `groups` stem (`docs/AI.md` §391; no
/// capture joins [`TRACES`]): the fourteen group rows and the sixteen issuer-chain functions
/// their stagings enter — `Options::recall_aircraft`, `scramble_aircraft`, `do_alarm`,
/// `do_city_gather`, `do_disband`, `GroupOut::issue_stop_spell`, `issue_city_gather`,
/// `validate_disband`, six `CommandPackage::process_*` arms, `ScenarioFuncSet::create_unit` and
/// `add_unit`; eleven rows take a staging (§391.2: nine `@key` with `--bindkey`, one `@rclick`,
/// one `!eval create_unit`), `Group::is_moving_to` stays open with none (park AT-b), and
/// `GroupData::show_interface`, `Group::dbg_pack_or_unpack` and
/// `CommandPackage::process_transport` go to [`RESIDUE`] on scope (§391.3); residue 265 → 268,
/// open 2 → 29.
/// **306** with item 3907's cite of the eight spells no capture ever cast and the editor's
/// capital (`docs/AI.md` §392; no capture joins [`TRACES`]): `SpellType::cast_immortal_hinf`,
/// `cast_immortal_arch`, `cast_move_capital`, `cast_assimilate` and `cast_paradrop` — their
/// `craftrules.xml` rows are `FROM`/`FROM2` `None`, so `is_castable`'s head refuses every caster —
/// and `ScenarioEditor::make_city_capital@009a1380` go to [`RESIDUE`]; `cast_disband@00671ed0`
/// (`@queueup 0 666 1 <b>` on a who=0 Barracks), `cast_entrench@00672820` (`@spell 0 637` from a
/// staged General) and `cast_counterintel@00673740` (`@spell 0 631` from a staged Scout, a
/// `DETECT` caster, on a who=1 Spy) stay open with their stagings named; residue 268 → 274,
/// open 29 → 32 on the tree merged with item 3906.
/// **309** with item 3908's cite of the implicit residue's `map` stem (`docs/MAPGEN.md` §22), and
/// eleven of items 3480's and 3493's archived style takes joined to [`TRACES`] with no capture
/// (run1098, run1099, run1117, run1119–run1121, run1123–run1126, run1128): the eleven styles'
/// `make_continents`, `Map::find_region_centroid` and `Map::eliminate_edge_canals` are entered
/// on −1 and stay off; `MapConquest::check_starting_loc@0068d7e0` and its script caller
/// `ScenarioFuncSet::find_extra_starting_loc@009e5070` join with the `!eval` staging that enters
/// both (§22.3), and `MapConquest::add_additional_players@0068d260` joins as [`RESIDUE`] on
/// Conquer the World's scope; residue 274 → 275, open 32 → 34 on the
/// tree merged with item 3907.
/// **309** unmoved by item 3910 (`docs/AI.md` §394), which reads route 2's entry shut: no caller
/// runs `find_new_target` with a group move current over the list. `do_move` is entered with one
/// current only from `do_group_move`'s leader arm, whose stash is ungrouped or group-killed the
/// frame it is made. `cavarch_fight`'s `fight(…, 1)` never reaches a `find_new_target` (the
/// listing keys every site on `param_5 = 0`). `target_opportunity:393` is behind `is_move` and
/// the `ATTACK`'s block, and no `action_move_near` path lays a plain move over a list it keeps.
/// The pair is a [`RESIDUE`] candidate and stays here until park 3910a's blind second reading of
/// §390 and §394 agrees; open 34, unmoved, on the tree merged with item 3908.
/// **366** with item 3909's cite of the implicit residue's last stems (`docs/AI.md` §393; no
/// capture joins [`TRACES`]): the forty-four rows outside `map` and `spelltype` and thirteen
/// callers §393 cites; `CliffsData::gather_size`, `Unit::dbg_jump_to_action` and
/// `Group::dbg_jump_to_action` stay open with a staging each (§393.1, park AW-a, AW-b), and
/// fifty-four go to [`RESIDUE`] on scope — the editor, Conquer the World, the unit-balance
/// harness, multiplayer and sync recovery, debug verbs, the renderer and the interface, one
/// `add_mountain` mode no caller passes, and one with no reference (§393.2); residue 275 → 329,
/// open 34 → 37 on the tree merged with items 3907, 3908 and 3910.
/// **363** with item 3913's golden chapter one hundred and ten (`docs/GOLDEN.md` §160; run1507b
/// joins [`TRACES`]): `OPTION_QUEUE` on chapter nineteen's Spy mid-hold enters
/// `GroupOut::issue_stop_spell@00708a20` on 770 and, at the pump,
/// `CommandPackage::process_stop_spell@00947cc0` and `Group::action_stop_spell@006fd7a0`
/// (`docs/AI.md` §391.2 row 6); open 37 → 34.
/// **357** with item 3912's run1506b (`docs/GOLDEN.md` §159), the `cover=1` twin of chapter one
/// hundred and nine: `select 2007` and the Airbase's two keys enter `Options::recall_aircraft`,
/// `CommandPackage::process_recall` and `Group::action_recall` on 2292, and
/// `Options::scramble_aircraft`, `CommandPackage::process_scramble` and `Group::action_scramble`
/// on 2312; open 34 → 28 (§391.2 rows 4 and 5) on the tree merged
/// with item 3913.
/// **353** with item 3911's run1505 and run1505b joining [`TRACES`] (`docs/GOLDEN.md` §158,
/// `docs/AI.md` §391.2 rows 1–3 and 11): `Group::action_air_patrol@007029d0`,
/// `Group::action_air_attack_ground@00703d80`, `Group::action_return@006fad40` and
/// `GroupData::is_in_air_range@0070b6b0` are entered (`action_recall`, `recall_aircraft` and
/// `process_recall`, which run1505 enters too, left with item 3912); open 28 → 24 on the tree
/// merged with item 3912.
/// **349** with item 3914's golden chapter one hundred and eleven (`docs/GOLDEN.md` §161;
/// run1508b joins [`TRACES`]): `OPTION_DISBAND` on chapter nine's Chariot enters
/// `Options::do_disband@0071df20` and `GroupOut::validate_disband@0070a4a0` on 622 and, at the
/// pump, `CommandPackage::process_disband@00948660` and `Group::action_disband@0070e260`
/// (`docs/AI.md` §391.2 row 7); open 24 → 20 on the tree merged with item 3911.
/// **347** with item 3916's run1510d joining [`TRACES`] (`docs/GOLDEN.md` §163, `docs/AI.md`
/// §391.2 row 9): `OPTION_PEASANT_ALARM` on a selected Citizen enters `Options::do_alarm@0071c3f0`
/// on 622 and, at the pump, `Group::action_alarm_peasant@006fd980`; open 20 → 18 on the tree
/// merged with item 3914.
/// **343** with item 3915's run1509b (`docs/GOLDEN.md` §162), the `cover=1` twin of chapter one
/// hundred and twelve: `select 2000` and `OPTION_VILLAGE_GATHER_ALL` enter
/// `Options::do_city_gather` and `GroupOut::issue_city_gather` on 610 and, at the pump,
/// `CommandPackage::process_city_gather` and `Group::action_city_gather`; open 18 → 14 (§391.2
/// row 8) on the tree merged with items 3911, 3914 and 3916.
/// **338** with item 3917's run1511b (`docs/GOLDEN.md` §164), the `cover=1` twin of chapter one
/// hundred and fourteen: the console's `eval create_unit` enters `ScenarioFuncSet::create_unit@009f4c50`
/// and `add_unit@009e2220`, a land type on the lake `Group::action_transport@00702620`, and the
/// formed group `Group::dbg_jump_to_action@00707a00` and `Unit::dbg_jump_to_action@00602ea0`, all
/// on 620 (`docs/AI.md` §391.2 row 10, §393.1 row 2); open 14 → 9 on the tree merged with items
/// 3915 and 3916.
/// **337** with item 3918's run1512b joining [`TRACES`] (`docs/GOLDEN.md` §165, `docs/AI.md`
/// §392.3): `DISBAND` queued at a Barracks completes, and `Build::finished` casts it through
/// `SpellType::cast` into `SpellType::cast_disband@00671ed0` on trace frame 831. **338** again with
/// `CityData::is_disbanding@007380a0` on: §165 cites it, and the research arm refused run1512's
/// city press before `is_castable` asked it (parked 3918a, the staging).
/// **336** with item 3921's run1515 joining [`TRACES`] (`docs/MAPGEN.md` §23, §22.3's staging):
/// the console's `eval find_extra_starting_loc(4, 4, 1)` on the golden start enters
/// `ScenarioFuncSet::find_extra_starting_loc@009e5070` and `MapConquest::check_starting_loc@0068d7e0`
/// on 620; open 9 → 7.
/// **335** with item 3919's run1513b (`docs/GOLDEN.md` §166), the `cover=1` twin of chapter one
/// hundred and sixteen: a General's `@spell 0 637` enters `SpellType::cast_entrench@00672820` on
/// 680 (`docs/AI.md` §392.3); open 7 → 6 on the tree merged with items 3917, 3918 and 3921.
/// **335** unmoved by item 3930's run1517b joining [`TRACES`] (`docs/GOLDEN.md` §168): a `DISBAND` at the
/// Barracks in city 0 queues and closes, and `CityData::is_disbanding@007380a0` stays `NEVER` —
/// `is_castable`'s city arm skips a `NO_CITY` type (parked 3930a, a Temple's staging).
/// **335** unmoved by item 3923's blind second reading of `docs/AI.md` §390 and §394
/// (`docs/audit/2026-11-06-item-3923.md`), which disagrees and names an entry: `kill_group_move`'s
/// walk meets a matching node again, so route 2's own list empties and faults at `5ff752`; but
/// `ungroup_move_order` lays a `MOVE_TO` without `0x04` over a leader's list and keeps its stash, and
/// below a single `GM X` a duplicate pair of another id reaches `5ff78f` through `do_move:105`.
/// `Unit::kill_group_order@005e3310` and `Group::kill_group_order@00710aa0` stay here (in
/// [`RESIDUE`] since item 3940, `docs/AI.md` §399), the take parked as 3923a; open 6, unmoved, on the tree merged with item 3919.
/// **334** with item 3920's run1514b (`docs/GOLDEN.md` §167), the `cover=1` twin of chapter one
/// hundred and seventeen: a Scout's `@spell 0 631` on a who=1 Spy two tiles off enters
/// `SpellType::cast_counterintel@00673740` on 658 (`docs/AI.md` §392.3); open 6 → 5 on the tree
/// merged with items 3919, 3921 and 3923.
/// **333** with item 3924's run1516 joining [`TRACES`] (`docs/AI.md` §395, §393.1 row 1): a
/// Mine added beside a cliff on Southwest Mesa enters `CliffsData::gather_size@008a8f80` on 100;
/// open 5 → 4 on the tree merged with item 3920.
/// **332** with item 3934's run1519b joining [`TRACES`] (`docs/GOLDEN.md` §169): a `DISBAND` at a
/// Temple in city 0 enters `CityData::is_disbanding@007380a0` on 840 — the Temple's type is
/// `jam`, without `NO_CITY`, so `is_castable`'s city arm asks it (parked 3930a); open 4 → 3.
/// **332** unmoved by item 3935's blind second reading of `docs/AI.md` §391.3 (§397,
/// `docs/audit/2026-11-06-item-3935.md`): `show_interface`, `dbg_pack_or_unpack` and
/// `process_transport` stay [`RESIDUE`], ratified; `Group::is_moving_to@00713bd0` stays here as a
/// staging — a FORM's bit-4-clear group move heads the leader's list over a shift-queued `ATTACK`,
/// and `update_action` skips the move (parked 3935a); open 3, unmoved.
/// **332** unmoved by item 3936's chapter one hundred and twenty (`docs/GOLDEN.md` §170, run1520,
/// run1520b): two Catapults' leader holds `[GROUP_MOVE flags 1][ATTACK]` on an enemy Barracks
/// on its sixteenth frames 698 and 714, and `Group::is_moving_to@00713bd0` is not entered —
/// `check_target_path:138-141` asks the target's vslot `+0x1c`, which `Build`'s vtable answers 1
/// for every building (`00b42190` → `0041e0e0`, `mov eax,1`; `Unit`'s is `0041bff0`,
/// `xor eax,eax`), so only a unit target reaches the arm, and only a fleeing one (parked 3936a).
/// **332** unmoved by item 3940's recon3940-1 (`docs/AI.md` §399): a walker meets V head-on under
/// the ungroup's `MOVEORDER` copy (`group` 11), and the touch is mutual in block 895. V side-steps
/// with `start_dist` 0, so the search finds a path and does not suspend. On the six refusals,
/// `Unit::kill_group_order@005e3310` and `Group::kill_group_order@00710aa0` move to [`RESIDUE`],
/// residue 329 → 331, open 3 → 1.
/// **331** with item 3939's run1522b joining [`TRACES`] (`docs/GOLDEN.md` §171): chapter one
/// hundred and twenty's Envelop with the shift-attack on a who=1 Citizen walking north, away from
/// the Catapults and out of reach of the move's point; the leader's sixteenth frame 714 enters
/// `Group::is_moving_to@00713bd0` from `check_target_path:191`, which answers 1, and the group arm's
/// `repath`, `kill_next_order` and `action_attack(.., QUEUE_FIRST)` follow; open 1 → 0, on the tree merged with
/// item 3940.
#[rustfmt::skip]
pub const NEVER: &[u32] = &[
    0x0045_2730, 0x0046_cef0, 0x0046_ed60, 0x0046_ed70, 0x0046_f370, 0x0047_03f0, 0x0047_0430,
    0x0047_0570, 0x0047_0680, 0x0047_0760, 0x0047_07a0, 0x0047_0800, 0x0047_0e50, 0x0047_0f30, 0x0047_1170, 0x0047_22f0, 0x0047_2370,
    0x0047_2420, 0x0047_2a40, 0x0047_2cb0, 0x0047_2cc0, 0x0047_2d10, 0x0047_6aa0, 0x0047_70d0, 0x0047_80c0, 0x0047_df90, 0x0048_43e0, 0x0048_49a0, 0x0048_5140,
    0x0048_5a40, 0x0048_5a60, 0x0048_6c60, 0x0048_89a0, 0x004a_1ca0, 0x0054_cea0, 0x0054_cf90, 0x0054_d100, 0x0058_3380, 0x0058_3390, 0x0058_33a0,
    0x0058_33b0, 0x0058_33c0, 0x0058_33d0, 0x0058_3400, 0x0058_3410, 0x0058_3420, 0x0058_3450, 0x0058_3470, 0x0058_3480, 0x0058_3490, 0x0058_34a0, 0x0058_5cb0, 0x0058_5e90,
    0x0058_60c0, 0x0058_6440, 0x0058_6790, 0x0058_69e0, 0x0058_7060, 0x0058_8560, 0x0058_8f00, 0x0058_90b0, 0x0059_30c0, 0x0059_3f70, 0x0059_4080, 0x0059_4480, 0x0059_4630, 0x0059_4640, 0x0059_4650, 0x0059_4660,
    0x0059_46c0, 0x0059_47e0, 0x005a_ac70, 0x005a_bc70, 0x005d_86a0, 0x005d_86b0, 0x005d_8fb0, 0x005d_8fc0, 0x005e_2bb0, 0x005e_3310,
    0x005e_4c80, 0x005e_4d10, 0x005e_5be0, 0x005e_75a0, 0x005e_8670, 0x005e_d040, 0x005e_d1f0, 0x005f_2480, 0x005f_4720, 0x005f_4730, 0x0060_3130, 0x0060_4550,
    0x0060_8850, 0x0060_9a20, 0x0060_a600, 0x0060_a6e0, 0x0060_a800, 0x0060_a810, 0x0060_fde0, 0x0061_d740, 0x0061_daf0, 0x0061_db10, 0x0061_dc50,
    0x0061_dc60, 0x0062_2ce0, 0x0062_3310, 0x0062_d430, 0x0062_d5b0, 0x0062_e2f0, 0x0063_0590, 0x0063_06b0, 0x0063_3390, 0x0063_89b0, 0x0063_9830,
    0x0063_e390, 0x0063_e440, 0x0063_fa30, 0x0064_2bb0, 0x0064_2d90, 0x0064_50b0, 0x0064_50c0, 0x0064_52f0, 0x0064_6c00, 0x0064_6c10, 0x0064_6c30, 0x0064_6f50,
    0x0064_7150, 0x0064_86a0, 0x0064_e4a0, 0x0065_3d00, 0x0065_8190, 0x0065_b850, 0x0065_cfd0, 0x0066_1b80, 0x0066_1d70, 0x0066_2670, 0x0066_3b60, 0x0066_3b70,
    0x0066_3b90, 0x0066_9100, 0x0066_9110, 0x0066_9120, 0x0066_9130, 0x0066_d810, 0x0066_d830, 0x0066_d850, 0x0066_fd40, 0x0066_fd50,
    0x0067_0600, 0x0067_0740, 0x0067_14c0, 0x0067_14f0, 0x0067_1600, 0x0067_1760, 0x0067_1820, 0x0067_1f80, 0x0067_3e80,
    0x0067_7590, 0x0067_7df0, 0x0067_d370, 0x0067_e600, 0x0067_e700, 0x0068_1b20, 0x0068_1c50, 0x0068_3730, 0x0068_8310, 0x0068_9cb0, 0x0068_a1b0, 0x0068_c870,
    0x0068_c8f0, 0x0068_d1a0, 0x0068_d260, 0x0068_dba0, 0x0068_e6a0, 0x0069_4ab0, 0x0069_5050, 0x006b_0180, 0x006b_1b80, 0x006b_1ba0, 0x006b_1d60,
    0x006b_22e0,
    0x006b_2390, 0x006b_2510, 0x006b_3020, 0x006b_3960, 0x006b_4230, 0x006b_4290, 0x006b_46b0, 0x006b_4880, 0x006b_5590, 0x006b_76a0,
    0x006b_88b0, 0x006d_5230, 0x006d_6660, 0x006d_6740, 0x006d_6e80, 0x006d_a740,
    0x006e_c170, 0x006e_de80, 0x006e_fb20, 0x006e_fb50, 0x006e_fda0, 0x006e_ff70, 0x006f_0230, 0x006f_a090, 0x0070_0010, 0x0070_20c0, 0x0070_7aa0, 0x0070_7b50, 0x0070_ce70, 0x0070_f8a0, 0x0071_0aa0, 0x0071_37f0, 0x0071_3bb0,
    0x0072_ff60, 0x0073_6470, 0x0073_69c0, 0x0073_8180, 0x0073_9ab0, 0x0073_9e20, 0x0073_b540, 0x0073_b5d0, 0x0073_b880, 0x0073_bb20, 0x0073_c1b0,
    0x0073_c5d0, 0x0073_c5e0, 0x0073_c780, 0x0073_e000, 0x0073_ebc0, 0x0073_f650, 0x0074_0030, 0x0074_0060, 0x0074_1a40, 0x0074_1d50,
    0x0078_be90, 0x0079_6140, 0x0079_7200, 0x0079_7c40, 0x0079_8880, 0x0079_99e0, 0x0079_ea80, 0x007a_a400, 0x007a_a560, 0x0081_2d20, 0x0081_34d0, 0x0084_2050,
    0x0084_2060, 0x0084_2070, 0x0084_b8d0, 0x0086_1960, 0x0089_0060, 0x0089_4060, 0x0089_41b0, 0x0089_7a50, 0x0089_7a60, 0x0089_7bc0, 0x0089_7e20, 0x0089_8920,
    0x0089_8cf0, 0x0089_8f10, 0x0089_be90, 0x0089_bea0, 0x0089_beb0, 0x0089_d3a0, 0x0089_ddc0, 0x008a_4040, 0x008a_4060, 0x008a_60c0, 0x008a_60e0,
    0x008b_0060, 0x008b_13b0, 0x008b_2a80, 0x008c_6db0, 0x008c_7810, 0x008d_1830, 0x008d_9190, 0x008d_92d0, 0x0092_8f30, 0x0092_d0b0, 0x0092_fc50, 0x0093_43b0,
    0x0093_a140, 0x0093_a150, 0x0093_e9e0, 0x0093_ee70, 0x0094_3f30, 0x0094_8cb0, 0x0094_8e00,
    0x0094_9050, 0x0094_95c0, 0x0094_c1c0, 0x0095_4c60, 0x0095_6470, 0x0095_6ac0, 0x0095_9110, 0x0095_9500, 0x0096_2c70, 0x0099_6ac0, 0x0099_bc20, 0x0099_c750,
    0x009a_0950, 0x009a_1380, 0x009a_3f30, 0x009a_adc0, 0x009a_b9c0, 0x009a_df40, 0x009b_1430, 0x009b_2e40, 0x009b_32f0, 0x009b_3320, 0x009b_3340, 0x009b_3590,
    0x009b_37a0, 0x009b_39e0, 0x009b_4820, 0x009b_4930, 0x009b_4da0, 0x009b_59f0, 0x009b_5d20, 0x009b_5e00, 0x009b_5ec0, 0x009b_5f20, 0x009b_6de0, 0x009b_6fa0,
    0x009b_7060, 0x009b_71c0, 0x009b_7d30, 0x009b_7de0, 0x009b_8ac0, 0x009f_b2d0, 0x00a4_69f0,
];

/// The blind list's accepted residue: functions on [`NEVER`] that no
/// capture this project can stage will enter, each with the reason — the
/// "enumerated and accepted deliberately, function by function" of
/// `docs/DECISIONS.md` entry 29. A row leaves only by a capture entering
/// it, which fails the pin, or by its citation going; either way
/// [`the_residue_is_on_the_blind_list`](tests) says so.
///
/// **"No reference in the executable"** means the image holds no `call`,
/// `jmp` or `jcc rel32` to the entry, no rel8 jump to it at an instruction
/// boundary, and no four-byte copy of its address at any offset of any
/// section, `.rdata`'s vtables included: the out-of-line copy of a
/// function the compiler inlined at every use, or one nothing calls. The
/// scan is `tools/trace/report.py <exe> refs`, run over all of [`NEVER`]
/// on 2026-09-27; the rows that say it are re-scanned against the install
/// by [`every_unreferenced_row_is_unreferenced_in_the_image`](tests).
/// Where the live copy is known, the row names it — the claim a document
/// cites at the orphan's address is then a claim about that caller.
///
/// Item 935 took seven (row 4 of `docs/CENSUS.md`'s ranked list); item
/// 940 took `docs/EMULATOR.md` §4's other twelve and ten more the scan
/// found. Eleven `CommandManager::issue_*` are unreferenced too and are
/// **not** here: the DLL's `@` issuer lines call them by address, so a
/// coverage capture can enter them (`docs/CENSUS.md`, "The blind list,
/// 2026-09-27, item 940").
pub const RESIDUE: &[(u32, &str)] = &[
    (
        0x0045_2730,
        "std::pair<wstring,wstring>::pair: crossplay's — \
         ConnectionData::send_player, GameSpy::UpdateLobbyFromDTO, \
         LobbyManager::ConvertToPlatformLobby and an \
         unordered_map<wstring,wstring> insert under Crossplay::Lobby::DTO and \
         LobbyManager; multiplayer (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x0046_cef0,
        "ObjectData::support: +0x13c of all fifteen object tables, and the only \
         [reg+0x13c] dispatches are Window::on_middle_click and \
         on_window_middle_click on a Window; two readings, item 3893 \
         (docs/audit/2026-11-05-item-3893.md)",
    ),
    (
        0x0046_ed60,
        "OrderList::get_new_data: its one home is OrderList::vftable +0x4, and \
         nothing dispatches through a list's own vptr; the save and load walker \
         OrderList::walk_data inlines it (two readings, item 3505)",
    ),
    (
        0x0046_ed70,
        "OrderList::get_data_type: its one home is OrderList::vftable +0x0, and \
         nothing dispatches through a list's own vptr; OrderList::walk_data \
         inlines it, calling the order's +0x10 itself (two readings, item 3505)",
    ),
    (
        0x0046_f370,
        "SubObject::swap_team: live only as Good's and Item's +0x94, and every \
         object dispatch of +0x94 (Unit::swap_team, Leader::defeat_by, \
         City::find_buildings, Cities::capture_city, the bribe and assimilate \
         spells, the scenario functions) draws its receiver from the per-player \
         lists, which hold no Good or Item (two readings, item 3505)",
    ),
    (
        0x0047_03f0,
        "ObjectTypeOut::draw: type +0x140 of twelve tables; its two type \
         dispatchers go through unittypes[i], where a live UnitType holds \
         UnitTypeOut::draw; two readings, item 3893 \
         (docs/audit/2026-11-05-item-3893.md)",
    ),
    (
        0x0047_0430,
        "ObjectTypeData::is_upgrade: Type +0x128 of the object types, and every \
         +0x128 dispatch is on an object or a window, never a type (two readings, \
         item 3505)",
    ),
    (
        0x0047_0570,
        "TypeData::is_specialty_type: Type +0x4c of every type vftable, no other \
         reference, and every +0x4c dispatch is on an object, an order or a \
         window, never a type (two readings, item 3505)",
    ),
    (
        0x0047_0680,
        "TypeData::is_gov_hero_type: Type +0x30 of every type vftable, never \
         overridden; its one guard, in LeaderData::type_avail, always takes the \
         inline arm, and no unguarded +0x30 dispatch has a type receiver (two \
         readings, item 3505)",
    ),
    (
        0x0047_0760,
        "TypeData::is_item_type: the scenario editor's resources page only — \
         Options::draw_cursor_follows and mouse_move call slot +0x20 on \
         scenario_editor->cur_rare, and Type::lookup_display_name guards the \
         slot against this body (parked 3505c); the editor is cut from v1 \
         (DECISIONS 107 §3)",
    ),
    (
        0x0047_07a0,
        "TypeData::is_gaia_type: Type +0x18 of every type vftable, never \
         overridden; its guards, in Balance::compute_modifier and \
         Balance::return_pack, always take the inline arm, and no unguarded +0x18 \
         dispatch has a type receiver (two readings, item 3505)",
    ),
    (
        0x0047_0800,
        "TypeData::is_good_type: Type +0x8 of every type vftable, never \
         overridden, so its 24 guards always take the inline arm; the unguarded \
         +0x8 calls are COM releases and objects, never a type (two readings, \
         item 3505)",
    ),
    (
        0x0047_0e50,
        "Build::finished (the thunk to Build::finished@00628490): a vtable \
         slot, +0x1ac of Build, BuildOut, BuildData and Wall, that no \
         instruction dispatches; the image's one call or jmp through +0x1ac is \
         a Window thunk",
    ),
    (
        0x0047_0f30,
        "GatherPoint::walk_data: GatherPoint's own +0x0, and both walks in \
         PtrLinkListAbstract<GatherPoint>::walk_data guard on its table and \
         inline the body; GatherPoint has no subclass; two readings, item 3893 \
         (docs/audit/2026-11-05-item-3893.md)",
    ),
    (
        0x0047_1170,
        "WorldData::is_valid: called only from MapScenario::make (Conquer the \
         World) and TerrainOut::check_for_diagonal's four calls, which run only \
         when no orthogonal pair test fires; an entered cell's corner fires its \
         own test unless its one-up flanks differ, and distinct flanks need a \
         mutually unacceptable orthogonal pair no grid or plant on disk holds; \
         two readings, item 3891 (docs/audit/2026-11-05-item-3891.md)",
    ),
    (
        0x0047_22f0,
        "WallData::is_sea_trade: Build's and Wall's +0x28; \
         Console::find_target_daemon guard-inlines it, and Group::action_trade's \
         one unguarded +0x28 call runs only when is_trade answers 0 on the \
         destination whose is_trade already answered non-zero at the function's \
         head (+0xb0 is get_graphicwinxml, return this; the type is static) (two \
         readings, item 3505)",
    ),
    (
        0x0047_2370,
        "WallOut::draw (the zero-argument wrapper): Build's and Wall's +0x58, and \
         ObjectsOut::pop_all_zoomed_in draws a building by a direct call to the \
         one-argument WallOut::draw at 0x655fb0; the +0x58 object dispatches, in \
         sort_guys, see units and animals only (two readings, item 3505)",
    ),
    (
        0x0047_2420,
        "WallData::is_under_attack: Build's and Wall's +0x188, and every object \
         dispatch of +0x188 is guarded by it (Unit::do_repair, Unit::do_build) or \
         by an exact Build::vftable test, so the inline arm always runs; the rest \
         are units and animals (two readings, item 3505)",
    ),
    (
        0x0047_2a40,
        "BuildTypeData::is_mine: BuildType +0x98, and no +0x98 dispatch has a \
         build-type receiver (orders, windows, objects) (two readings, item 3505)",
    ),
    (
        0x0047_2cb0,
        "WallData::can_make: a folded return 0 at the Wall triple's \
         +0x190/+0x198, which Build overrides and no live Wall reaches, and \
         IncrementalLoad's +0x18, whose base table AnimMgr::AnimMgr replaces with \
         no call between; two readings, item 3893 \
         (docs/audit/2026-11-05-item-3893.md)",
    ),
    (
        0x0047_2cc0,
        "WallData::~WallData: inlined on every live path; its six callers are \
         unwind funclets; two readings, item 3887 \
         (docs/audit/2026-11-05-item-3887.md)",
    ),
    (
        0x0047_2d10,
        "WallOut::~WallOut: inlined on every live path; its five callers are \
         unwind funclets; two readings, item 3887 \
         (docs/audit/2026-11-05-item-3887.md)",
    ),
    (
        0x0047_6aa0,
        "ObjectTypeData::get_stance_type: four bytes, Type +0x138 of the object, \
         good and item types; every +0x138 dispatch is on a unit's or a \
         building's type, which override it; its folded twin at Profile::vftable \
         +0x94 is never dispatched (two readings, item 3505)",
    ),
    (
        0x0047_70d0,
        "TypeData::avail: +0x8c of the tech, bonus and spell types (the object \
         types override it), and every +0x8c dispatch is on an order, an animal, \
         a texture or the lobby, never a type (two readings, item 3505)",
    ),
    (
        0x0047_80c0,
        "GoodTypeData::is_flat: +0x94 of the three good-type tables only; the one \
         good-receiver dispatch is Leader::produce_building's gather arm \
         (6e2312), which only a direct Oil Platform request reaches, and no v1 \
         caller makes one (create_buildings skips water non-docks; the Setup \
         types are fixed; the rest are Conquer the World and the scenario VM); \
         two readings, item 3899 (docs/audit/2026-11-06-item-3899.md)",
    ),
    (
        0x0047_df90,
        "LeaderData::has_bonus (the 0047df90 copy): the tooltip's \
         byte-identical copy of the Nubian market gate, called only from \
         Options::describe; the sim's copy, has_bonus@006e0b80, stays open \
         (docs/NATIONS.md §9.1, §9.3; the forty-eighth pass)",
    ),
    (
        0x0048_43e0,
        "PatrolOrder::get_patrol_order: reached only through its vtordisp \
         thunk's jmp (0048387c), whose one home is PatrolOrder::vftable \
         (+0x5c, +0xc8), and no PatrolOrder is ever the object's own type: \
         its constructor's two callers are GroupPatrolOrder@00481fe0 and \
         AirPatrolOrder@004843f0, which re-point the vptr at vftables whose \
         +0x5c is another thunk, and neither its constructor nor its \
         destructor dispatches; a held-Shift draw_orders on a player's patrol \
         asks GroupPatrolOrder's (item 3495)",
    ),
    (
        0x0048_49a0,
        "SpecialAnimOrder::walk_data: the order's +0x8, but come_out and \
         land_plane (through work and do_job) run do_spec_anim, which kills the \
         order in the call that added it, so no walk or save sees one; two \
         readings, adjudicated, item 3893 (docs/audit/2026-11-05-item-3893.md)",
    ),
    (
        0x0048_5140,
        "GroupAttackOrder::log_data: reached only through its vtordisp thunk's \
         jmp, and nothing asks get_obj(GROUP_ATTACK), so no order list holds \
         one to print (docs/GOLDEN.md §144.4; the forty-eighth pass)",
    ),
    (
        0x0048_5a40,
        "GroupMoveOrder::is_fleeing: reached only through its vtordisp thunk's \
         jmp, at +0x28 of GroupMoveOrder and GroupAttackToOrder, and no \
         instruction dispatches +0x28 on an order; UnitData::is_fleeing asks \
         +0x10 (two readings, item 3505; the MoveOrder::is_fleeing row's slot)",
    ),
    (
        0x0048_5a60,
        "GroupOrder::clear: reached only through its vtordisp thunk's jmp, whose \
         one home is the abstract GroupOrder::vftable +0x4 (+0x10 is purecall); \
         its constructors and destructors install it transiently and dispatch \
         nothing while it is the vptr (two readings, item 3505)",
    ),
    (
        0x0048_6c60,
        "FollowOrder::get_follow_order: entered, and no instrument sees it: four \
         bytes (lea eax,[ecx-0x24]; ret), the tracer's 'next entry under 5 bytes \
         away' (rontrace.funcs.excluded.txt); its one predecessor, the vtordisp \
         thunk at 00486c64, ends in an unconditional jmp to it and is entered by \
         run321 on 620 and run1032 on 708 (item 3505)",
    ),
    (
        0x0048_89a0,
        "MoveOrder::is_fleeing: reached only through vslot +0x28 of the move \
         orders' vtables (its two vtordisp thunks), and no instruction \
         dispatches that slot on an order (docs/ORDERS.md §1.3, two readings); \
         the live predicate is UnitData::is_fleeing@0046efa0 (item 1182)",
    ),
    (
        0x004a_1ca0,
        "BorderSpline::on_field_change: the debug parameter window only \
         (ParamRegisterWindow, ParameterWindow.obj; keymap 0x1e under \
         Console::cheat_keys), whose Scrolls write a registered object's parameter \
         and call its +0xc; the window tunes renderer objects and is out of v1 with \
         the renderer it tunes (two readings, item 3861; the sixty-sixth pass, \
         DECISIONS 106)",
    ),
    (
        0x0054_cea0,
        "wincpuid: once at startup, bighuge_init_real -> cpunormspeed -> \
         cpuspeed, and cpuspeed runs only when the registry's \
         CentralProcessor\\0 ~MHz/~Mhz/~mhz all fail, which the prefix \
         answers; the tracer arms before the exe's entry and records \
         cpunormspeed at frame -1 on every traced launch; out on reach, \
         the host's registry and not a lobby (two readings, item 3897, \
         docs/audit/2026-11-05-item-3897.md)",
    ),
    (
        0x0054_cf90,
        "wincpufeatures: cpuspeed's second unconditional call, behind the \
         same registry gate as wincpuid; out on reach (two readings, item \
         3897, docs/audit/2026-11-05-item-3897.md)",
    ),
    (
        0x0054_d100,
        "check_80386: wincpuid's one call, taken only when EFLAGS.ID cannot \
         be toggled (a processor without CPUID), behind cpuspeed's registry \
         gate; out on reach twice over (two readings, item 3897, \
         docs/audit/2026-11-05-item-3897.md)",
    ),
    (
        0x0058_3380,
        "GameAccessConst::get_scary_scenario_editor: a GameAccessConst getter \
         inlined at every use; no reference in the executable (the fiftieth \
         pass's scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0058_3390,
        "GameAccessConst::get_scary_iface_main: a GameAccessConst getter \
         inlined at every use; no reference in the executable (the fiftieth \
         pass's scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0058_33a0,
        "GameAccessConst::get_scary_camera: a GameAccessConst getter inlined at \
         every use; no reference in the executable (the fiftieth pass's scan \
         over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0058_33b0,
        "GameAccessConst::get_scary_fog: a GameAccessConst getter inlined at \
         every use; no reference in the executable (the fiftieth pass's scan \
         over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0058_33c0,
        "GameAccessConst::get_scary_console: a GameAccessConst getter inlined \
         at every use; no reference in the executable (the fiftieth pass's scan \
         over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0058_33d0,
        "GameAccessConst::get_scary_iface_current: a GameAccessConst getter \
         inlined at every use; no reference in the executable (the fiftieth \
         pass's scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0058_3400,
        "GameAccessConst::get_scary_terrain: a GameAccessConst getter inlined \
         at every use; no reference in the executable (the fiftieth pass's scan \
         over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0058_3410,
        "GameAccessConst::get_scary_options: a GameAccessConst getter inlined \
         at every use; no reference in the executable (the fiftieth pass's scan \
         over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0058_3420,
        "GameAccessConst::get_scary_doobers: a GameAccessConst getter inlined \
         at every use; no reference in the executable (the fiftieth pass's scan \
         over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0058_3450,
        "GameAccessConst::get_scary_select_group: a GameAccessConst getter \
         inlined at every use; no reference in the executable (the fiftieth \
         pass's scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0058_3470,
        "GameAccessConst::get_scary_scene: a GameAccessConst getter inlined at \
         every use; no reference in the executable (the fiftieth pass's scan \
         over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0058_3480,
        "GameAccessConst::get_scary_message_win: a GameAccessConst getter \
         inlined at every use; no reference in the executable (the fiftieth \
         pass's scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0058_3490,
        "GameAccessConst::get_scary_graphic_pieces: a GameAccessConst getter \
         inlined at every use; no reference in the executable (the fiftieth \
         pass's scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0058_34a0,
        "GameAccessConst::get_scary_console_who: a GameAccessConst getter \
         inlined at every use; no reference in the executable (the fiftieth \
         pass's scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0058_5cb0,
        "Game::run_skilltest: Main::run_game_loop's GAME_SKILLTEST, a front-end \
         game type (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x0058_5e90,
        "Game::run_campaign: Main::run_game_loop's GAME_SCENARIO and \
         GAME_TUTORIAL, the scripted campaigns on the trigger system cut from \
         v1 (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x0058_60c0,
        "Game::run_scenario: the scenario, campaign, tutorial, skill-test and \
         Conquer the World paths, cut from v1; a skirmish profile that has \
         played takes run_solo (docs/ATTRITION.md, the never-entered functions; \
         parked 3414c; the forty-eighth pass)",
    ),
    (
        0x0058_6440,
        "Game::run_editor: run_game_loop's GAME_EDITOR arm, the scenario \
         editor, cut from v1 (docs/ATTRITION.md, the never-entered functions; \
         the forty-eighth pass)",
    ),
    (
        0x0058_6790,
        "Game::run_unit_balance: Main::run_game_loop's GAME_UNIT_BALANCE, the \
         unit-balance harness, which sets semaphore bit 9 (docs/AI.md §393.2, \
         item 3909)",
    ),
    (
        0x0058_69e0,
        "Game::run_oos_recovery: Game::run under semaphore bit 27, set by \
         CommandManager::recover_from_oos and by CommandPackage::end_process \
         past the checksum-failure threshold; sync recovery (docs/AI.md §393.2, \
         item 3909)",
    ),
    (
        0x0058_7060,
        "Game::run_gamespy: the GameSpy online lobby, run_game_loop's \
         GAME_GAMESPY_* arms, cut from v1 (docs/GOLDEN.md §144.4; the \
         forty-eighth pass)",
    ),
    (
        0x0058_8560,
        "Game::learn_as_you_play_popup: Main::run_game_loop's first-game popup, \
         only while player_profile.played_game is 0; interface (docs/AI.md \
         §393.2, item 3909)",
    ),
    (
        0x0058_8f00,
        "Game::action_screen_shot: ScenarioFuncSet::take_screenshot's \
         screenshot; interface (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x0058_90b0,
        "Game::restore_rules_data: ConquestGame::setup_scenario and \
         SetupWin::exec under game->reload_rules, whose one writer is \
         Game::end_game_close — a reload between games, before any frame \
         (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x0059_30c0,
        "Game::action_cheat_ai_toggle: no reference in the executable; \
         CommandPackage::process_cheat_ai_toggle flips ai_off inline",
    ),
    (
        0x0059_3f70,
        "Game::is_team_game: a Game rule getter inlined at every use; no \
         reference in the executable (the fiftieth pass's scan over the 192 \
         open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0059_4080,
        "Game::record_results: Game::run after the game, under a pref that \
         defaults to 0; it writes a results log and no game state (docs/AI.md \
         §393.2, item 3909)",
    ),
    (
        0x0059_4480,
        "Game::is_raining: a Game rule getter inlined at every use; no \
         reference in the executable (the fiftieth pass's scan over the 192 \
         open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0059_4630,
        "Game::is_deathmatch: a Game rule getter inlined at every use; no \
         reference in the executable (the fiftieth pass's scan over the 192 \
         open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0059_4640,
        "Game::is_infinite: a Game rule getter inlined at every use; no \
         reference in the executable (the fiftieth pass's scan over the 192 \
         open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0059_4650,
        "Game::is_nomad: a Game rule getter inlined at every use; no reference \
         in the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0059_4660,
        "Game::is_never_war: a Game rule getter inlined at every use; no \
         reference in the executable (the fiftieth pass's scan over the 192 \
         open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0059_46c0,
        "Game::is_no_rush: no reference in the executable (docs/AI.md §218, \
         §296; report.py refs reads it dead; the forty-eighth pass)",
    ),
    (
        0x0059_47e0,
        "Game::is_sandbox: a Game rule getter inlined at every use; no \
         reference in the executable (the fiftieth pass's scan over the 192 \
         open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x005a_ac70,
        "Setup::init_wild_life: no reference in the executable",
    ),
    (
        0x005a_bc70,
        "Setup::build_leader: no reference in the executable",
    ),
    (
        0x005d_86a0,
        "Guy::set_pitch: the out-of-line copy of a function the compiler \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x005d_86b0,
        "Guy::set_bank: the out-of-line copy of a function the compiler inlined \
         at every use, or one nothing calls; no reference in the executable \
         (the fiftieth pass's scan over the 192 open rows, 2026-10-22, Loop \
         3465)",
    ),
    (
        0x005d_8fb0,
        "Guy::set_elev_angle: the out-of-line copy of a function the compiler \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x005d_8fc0,
        "Guy::set_pivot_angle: no call and no absolute reference in the \
         executable; Unit::move_step's cavalry-archer arm carries it inline \
         (item 1117, docs/COMBAT.md §85.2)",
    ),
    (
        0x005e_2bb0,
        "Unit::kill_current_action: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x005e_3310,
        "Unit::kill_group_order: its one caller is Group::kill_group_order+0x7e, \
         on the RESIDUE row 0x00710aa0 below, and that function is reached by \
         no route a capture stages (docs/AI.md §399, item 3940)",
    ),
    (
        0x005e_4c80,
        "Unit::add_await_board_order: lives under the BoardOrder only \
         Group::action_board_ship makes, whose command no site issues \
         (docs/GOLDEN.md §144.4; the forty-eighth pass)",
    ),
    (
        0x005e_4d10,
        "Unit::add_board_order: lives under the BoardOrder only \
         Group::action_board_ship makes, whose command no site issues \
         (docs/GOLDEN.md §144.4; the forty-eighth pass)",
    ),
    (
        0x005e_5be0,
        "Unit::do_spec_unit_anim: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x005e_75a0,
        "Unit::do_group_attack: its one call is Unit::do_job's GROUP_ATTACK case \
         (617b0c), and no order-factory site makes a GroupAttackOrder (type 0x14) \
         except as a copy or a load of one that exists; two readings, item 3887 \
         (docs/audit/2026-11-05-item-3887.md)",
    ),
    (
        0x005e_8670,
        "Unit::do_form_change: do_job's CHANGE_FORM arm, run only on a \
         FormOrder, which no site in the image makes — none of get_obj's 47 \
         calls pushes 0x12 (docs/GROUPS.md §38; six register-typed calls read \
         by prefix, parked 3413c; the forty-eighth pass)",
    ),
    (
        0x005e_d040,
        "Unit::do_await_board: its one call is Unit::do_job's AWAIT_BOARD case \
         (617a9c), and the order's one maker is Unit::add_await_board_order, \
         already RESIDUE under the board-ship command no site writes; two \
         readings, item 3887 (docs/audit/2026-11-05-item-3887.md)",
    ),
    (
        0x005e_d1f0,
        "Unit::do_board: lives under the BoardOrder only \
         Group::action_board_ship makes, whose command no site issues \
         (docs/GOLDEN.md §144.4; the forty-eighth pass)",
    ),
    (
        0x005f_2480,
        "Unit::do_flee_to: no call or jump — do_job folds FLEE_TO onto do_move \
         — and the image's four copies of its RVA are immediates inside \
         SetupWin instructions (docs/GOLDEN.md §144.2, §144.4; the forty-eighth \
         pass)",
    ),
    (
        0x005f_4720,
        "Unit::think_hero: no reference in the executable (parked 3421a, the \
         forty-ninth pass; `report.py refs` scans dead, the byte before is \
         padding); the live copy is Unit::think@005f6e40's tail, which reads \
         is_hero (vslot +0xc4, unit_flags & 0x20) and calls add_to_army itself",
    ),
    (
        0x005f_4730,
        "Unit::think_supply: no reference in the executable (parked 3421a, the \
         forty-ninth pass; run1020's supply wagon never entered it): its one \
         hit is the RVA-shaped rel32 of `call Mountains::get_range` at \
         006a841b, inside TerrainGroups::place_all; the live copy is \
         Unit::think@005f6e40's tail, which reads is_supply (vslot +0xcc, \
         unit_flags & 0x40) and calls add_to_army itself",
    ),
    (
        0x0060_3130,
        "Unit::dbg_jump_to_waypoint: Group::dbg_jump_to_waypoint's, which \
         Console::exec_keymap's case 0x19 takes only under cheat_keys, and \
         ScenarioEditor::inc_selected_health; a debug verb (docs/AI.md §393.2, \
         item 3909)",
    ),
    (
        0x0060_4550,
        "Unit::check_meet_ship: lives under the BoardOrder only \
         Group::action_board_ship makes, whose command no site issues; item \
         3410's sweep backs its arithmetic (docs/GOLDEN.md §144.4; the \
         forty-eighth pass)",
    ),
    (
        0x0060_8850,
        "UnitData::can_gather: no reference in the executable",
    ),
    (
        0x0060_9a20,
        "UnitData::supplies_left: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0060_a600,
        "UnitData::turn_speed: no reference in the executable; the live copy is \
         GuyData::turn_speed",
    ),
    (
        0x0060_a6e0,
        "UnitData::is_blown: the out-of-line copy of a function the compiler \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0060_a800,
        "UnitData::uber_size: the out-of-line copy of an accessor inlined at \
         every use (the type's +0x308); no reference in the executable; two \
         readings, item 3887 (docs/audit/2026-11-05-item-3887.md)",
    ),
    (
        0x0060_a810,
        "UnitData::is_entrenched: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0060_fde0,
        "UnitData::support_cost: slot +0x140 of the Unit and Animal triples, and \
         the six [reg+0x140] sites dispatch on a Window, graphsys or a UnitType, \
         never an object; two readings, item 3887 \
         (docs/audit/2026-11-05-item-3887.md)",
    ),
    (
        0x0061_d740,
        "UnitTypeData::get_relative_value: the out-of-line copy of a function \
         the compiler inlined at every use, or one nothing calls; no reference \
         in the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0061_daf0,
        "UnitTypeData::get_underlay_circle_radius: the out-of-line copy of a \
         function the compiler inlined at every use, or one nothing calls; no \
         reference in the executable (the fiftieth pass's scan over the 192 \
         open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0061_db10,
        "UnitTypeData::get_circle_radius: the out-of-line copy of a function \
         the compiler inlined at every use, or one nothing calls; no reference \
         in the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0061_dc50,
        "UnitTypeData::is_spellcaster: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0061_dc60,
        "UnitTypeData::needs_packing: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0062_2ce0,
        "Build::add_attack_order: no reference in the executable",
    ),
    (
        0x0062_3310,
        "Build::update_max_gatherers: no reference in the executable",
    ),
    (
        0x0062_d430,
        "BuildData::num_scholars: no reference in the executable",
    ),
    (
        0x0062_d5b0,
        "BuildData::is_infinite: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0062_e2f0,
        "BuildData::most_shots: Build's +0x1bc, dispatched only inside \
         BuildData::get_most_shots and get_garrison_arrows, which are virtual on \
         Build alone, so their guard always takes the inline arm (two readings, \
         item 3505)",
    ),
    (
        0x0063_0590,
        "BuildData::max_gatherers: no reference in the executable",
    ),
    (
        0x0063_06b0,
        "BuildQueueData::get_size: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0063_3390,
        "BuildType::set_domain: no reference in the executable",
    ),
    (
        0x0063_89b0,
        "BuildTypeData::area: the out-of-line copy of a function the compiler \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0063_9830,
        "BuildTypeData::is_embassy: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0063_e390,
        "Wall::Wall: its only references are the per-kind constructor tables \
         (ObjectsArray::get_new_data, Objects::find_free, Objects::init); \
         num_def_walls is only ever written 0 and no find_free call asks for \
         the wall range (docs/GOLDEN.md §144.4; the forty-eighth pass)",
    ),
    (
        0x0063_e440,
        "Wall::~Wall: inlined on every live path; called only from unwind \
         funclets and pushed by the wall-range constructors and Wall's vector \
         deleting destructor, none of which a v1 game runs; two readings, item \
         3887 (docs/audit/2026-11-05-item-3887.md)",
    ),
    (
        0x0063_fa30,
        "WallData::support_cost: slot +0x140 of the Build and Wall triples, and \
         no site dispatches +0x140 on an object; two readings, item 3887 \
         (docs/audit/2026-11-05-item-3887.md)",
    ),
    (
        0x0064_2bb0,
        "WallData::hits: slot +0x11c of the Wall triple only (Build overrides \
         it), and no live object's most-derived class is Wall; two readings, item \
         3887 (docs/audit/2026-11-05-item-3887.md)",
    ),
    (
        0x0064_2d90,
        "WallData::construct_time: slot +0x18c of the Wall triple only (Build \
         overrides it), and no live object's most-derived class is Wall; two \
         readings, item 3887 (docs/audit/2026-11-05-item-3887.md)",
    ),
    (
        0x0064_50b0,
        "ObjectData::los: slot +0x128 of the Object triple only, and \
         Object::Object's three callers are subclass constructors whose table \
         windows are call-free; two readings, item 3887 \
         (docs/audit/2026-11-05-item-3887.md)",
    ),
    (
        0x0064_50c0,
        "ObjectData::hits: slot +0x11c of the Object triple only, and no object \
         is a bare Object; two readings, item 3887 \
         (docs/audit/2026-11-05-item-3887.md)",
    ),
    (
        0x0064_52f0,
        "ObjectData::support_cost: slot +0x140 of the Object triple only, never \
         dispatched on an object, and no object is a bare Object; two readings, \
         item 3887 (docs/audit/2026-11-05-item-3887.md)",
    ),
    (
        0x0064_6c00,
        "ObjectData::max_range: slot +0x130 of the Object and Wall triples only \
         (Build and Unit override it), and no live object is a bare Object or a \
         Wall; two readings, item 3887 (docs/audit/2026-11-05-item-3887.md)",
    ),
    (
        0x0064_6c10,
        "ObjectData::has_range: the out-of-line copy of a function the compiler \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0064_6c30,
        "ObjectData::recharge: Build's and Wall's +0x134; its two object \
         dispatches, Build::do_attack and Object::do_launch, are guarded by an \
         exact Build::vftable test on a receiver that is always a Build, and no \
         Wall receiver is dispatched (two readings, item 3505)",
    ),
    (
        0x0064_6f50,
        "Object::~Object: inlined on every live path; its nine callers are unwind \
         funclets of the UnitData, BuildData, Wall, WallOut and WallData \
         constructors and destructors; two readings, item 3887 \
         (docs/audit/2026-11-05-item-3887.md)",
    ),
    (
        0x0064_7150,
        "Object::execute_events: one byte (ret), Build's and Wall's +0x154, \
         dispatched only by Objects::inc_time's loop over the unit indices, which \
         holds units and animals, both overriding it (two readings, item 3505)",
    ),
    (
        0x0064_86a0,
        "ObjectData::assault_benefit: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0064_e4a0,
        "ObjectData::is_in_range: entered, and no instrument sees it — the \
         tracer cannot hook it (rontrace.funcs.excluded.txt, a branch target \
         inside the displaced range); Unit::do_air_attack_ground+0x92 calls it \
         on every missile step past its first three tests, and run372's strike \
         on 2701 fires only past its answer (item 1182)",
    ),
    (
        0x0065_3d00,
        "Objects::remove_good: ScenarioEditor::delete_good's, under \
         Options::exec_editor and picked_spot; the editor (docs/AI.md §393.2, \
         item 3909)",
    ),
    (
        0x0065_8190,
        "ObjectsData::find_town_at: called by BuildTypeData::get_town only for \
         the l build flag (0x800) with e clear, and no shipped buildingrules.xml \
         row has l; find_town, its other caller, is unreferenced; two readings, \
         item 3893 (docs/audit/2026-11-05-item-3893.md)",
    ),
    (
        0x0065_b850,
        "Objects::remove_item: the out-of-line copy of a function the compiler \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0065_cfd0,
        "ObjectsData::find_dock: no reference in the executable",
    ),
    (
        0x0066_1b80,
        "ObjectTypeData::isnt: Type +0xf8 of every object type, and its one type \
         dispatch, ObjectData::isnt's guard, always takes the inline arm (two \
         readings, item 3505)",
    ),
    (
        0x0066_1d70,
        "ObjectTypeData::blocked_site: Type +0x11c of the object types, and every \
         +0x11c dispatch is on an object (the hit-points slot) or a window; the \
         22 blocked_site calls are BuildTypeData::blocked_site's, direct (two \
         readings, item 3505)",
    ),
    (
        0x0066_2670,
        "SubObjectOut::is_on_screen: +0x70 of twenty-two tables; its nineteen \
         .text copies are guards every table satisfies, and the Build, Good and \
         Item table-identity guards always hold; two readings, item 3893 \
         (docs/audit/2026-11-05-item-3893.md)",
    ),
    (
        0x0066_3b60,
        "Type::set_graph_name: a type-table accessor inlined at every use, or \
         one nothing calls; no reference in the executable (the fiftieth pass's \
         scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0066_3b70,
        "Type::set_type_name: a type-table accessor inlined at every use, or \
         one nothing calls; no reference in the executable (the fiftieth pass's \
         scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0066_3b90,
        "Type::set_help_name: a type-table accessor inlined at every use, or \
         one nothing calls; no reference in the executable (the fiftieth pass's \
         scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0066_9100,
        "get_unique_types2: a type-table accessor inlined at every use, or one \
         nothing calls; no reference in the executable (the fiftieth pass's \
         scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0066_9110,
        "get_type_count2: a type-table accessor inlined at every use, or one \
         nothing calls; no reference in the executable (the fiftieth pass's \
         scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0066_9120,
        "get_unique_types: a type-table accessor inlined at every use, or one \
         nothing calls; no reference in the executable (the fiftieth pass's \
         scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0066_9130,
        "get_type_count: a type-table accessor inlined at every use, or one \
         nothing calls; no reference in the executable (the fiftieth pass's \
         scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0066_d810,
        "TechType::restore: the out-of-line copy of a function the compiler \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0066_d830,
        "TechType::backup: the out-of-line copy of a function the compiler \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0066_d850,
        "Good::inc_time: one byte (ret), Good's +0xa0 only, and \
         Objects::inc_time's goods loop skips the call on an exact Good::vftable \
         test; goods are in no other list (two readings, item 3505)",
    ),
    (
        0x0066_fd40,
        "GoodTypeData::avail_drop: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0066_fd50,
        "GoodTypeData::avail_ctw: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0067_0600,
        "SpellType::cast_immortal_hinf: Go Melee (craftrules.xml row 664 is FROM \
         none and FROM2 None, so SpellTypeData::is_castable@00675bc0's head \
         answers 0 for every caster (is(-1) and get_graft(-1) both fail) and \
         SpellType::cast@00676ce0, the arm's only caller, never reaches its \
         switch; cast_supplies' and cast_pilfer's reason (parked 3462a), one \
         reading, item 3907 (docs/AI.md §392))",
    ),
    (
        0x0067_0740,
        "SpellType::cast_immortal_arch: Go Ranged (craftrules.xml row 663 is FROM \
         none and FROM2 None, so SpellTypeData::is_castable@00675bc0's head \
         answers 0 for every caster (is(-1) and get_graft(-1) both fail) and \
         SpellType::cast@00676ce0, the arm's only caller, never reaches its \
         switch; cast_supplies' and cast_pilfer's reason (parked 3462a), one \
         reading, item 3907 (docs/AI.md §392))",
    ),
    (
        0x0067_14c0,
        "SpellType::cast_supplies: Restock Supplies (638) is FROM None and \
         FROM2 None in craftrules.xml, so SpellTypeData::is_castable@00675bc0 \
         answers 0 for every caster at its head and SpellType::cast@00676ce0, \
         the arm's only caller, never reaches its switch; run1081 asked it \
         (process_spell 638 on 1233) and the wagon's mana_burn stayed 0 (parked \
         3462a; the fiftieth pass)",
    ),
    (
        0x0067_14f0,
        "SpellType::cast_jam_radar: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0067_1600,
        "SpellType::cast_rally: the out-of-line copy of a function the compiler \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0067_1760,
        "SpellType::cast_move_capital: Move Capital Here (craftrules.xml row 647 \
         is FROM None and FROM2 None, so SpellTypeData::is_castable@00675bc0's \
         head answers 0 for every caster (is(-1) and get_graft(-1) both fail) and \
         SpellType::cast@00676ce0, the arm's only caller, never reaches its \
         switch; cast_supplies' and cast_pilfer's reason (parked 3462a), one \
         reading, item 3907 (docs/AI.md §392)); its one other caller is the \
         scenario editor's make_city_capital",
    ),
    (
        0x0067_1820,
        "SpellType::cast_assimilate: Assimilate (craftrules.xml row 632 is FROM \
         None and FROM2 None, so SpellTypeData::is_castable@00675bc0's head \
         answers 0 for every caster (is(-1) and get_graft(-1) both fail) and \
         SpellType::cast@00676ce0, the arm's only caller, never reaches its \
         switch; cast_supplies' and cast_pilfer's reason (parked 3462a), one \
         reading, item 3907 (docs/AI.md §392))",
    ),
    (
        0x0067_1f80,
        "SpellType::cast_paradrop: Paradrop (craftrules.xml row 644 is FROM None \
         and FROM2 None, so SpellTypeData::is_castable@00675bc0's head answers 0 \
         for every caster (is(-1) and get_graft(-1) both fail) and \
         SpellType::cast@00676ce0, the arm's only caller, never reaches its \
         switch; cast_supplies' and cast_pilfer's reason (parked 3462a), one \
         reading, item 3907 (docs/AI.md §392))",
    ),
    (
        0x0067_3e80,
        "SpellType::cast_pilfer: Pilfer Resources (630) is FROM None and FROM2 \
         None in craftrules.xml, so SpellTypeData::is_castable@00675bc0 answers \
         0 for every caster at its head and SpellType::cast@00676ce0, the arm's \
         only caller, never reaches its switch; run1081 asked it (process_spell \
         630 on 1231) and neither leader's resources moved (parked 3462a; the \
         fiftieth pass)",
    ),
    (
        0x0067_7590,
        "ItemOut::draw_overlays: live only as Item::vftable +0x5c (ItemOut::vftable \
         is Item::Item's transient vptr); an Item lives in items and the \
         objects +0x2dc tree alone, whose dispatchers call +0x58 and +0x60, and \
         ObjectsOut::draw_overlays's +0x5c sites take the per-team object table \
         and goods, which hold no Item (two readings, item 3861)",
    ),
    (
        0x0067_7df0,
        "ItemTypeData::snap_center: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0067_d370,
        "Ammo::process: the out-of-line copy of a function the compiler inlined \
         at every use, or one nothing calls; no reference in the executable \
         (the fiftieth pass's scan over the 192 open rows, 2026-10-22, Loop \
         3465)",
    ),
    (
        0x0067_e600,
        "Search::walk_data: an element's dump walk, inlined into its \
         container's walk, which a `@dumpall` line enters (run1065); no \
         reference in the executable (the fiftieth pass's scan over the 192 \
         open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0067_e700,
        "Lands::walk_data: an element's dump walk, inlined into its container's \
         walk, which a `@dumpall` line enters (run1065); no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0068_1b20,
        "WCoordData::walk_data: an element's dump walk, inlined into its \
         container's walk, which a `@dumpall` line enters (run1065); no \
         reference in the executable (the fiftieth pass's scan over the 192 \
         open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0068_1c50,
        "TCoordData::walk_data: an element's dump walk, inlined into its \
         container's walk, which a `@dumpall` line enters (run1065); no \
         reference in the executable (the fiftieth pass's scan over the 192 \
         open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0068_3730,
        "PathFinder::find_wpath_army: no reference in the executable",
    ),
    (
        0x0068_8310,
        "PathFinderData::get_estimate: no reference in the executable (the rel8 \
         lookalike at 006882f9 is inside a cmpb); inlined four times in \
         PathFinder::astar_path",
    ),
    (
        0x0068_9cb0,
        "PathFinder::walk_data: an element's dump walk, inlined into its \
         container's walk, which a `@dumpall` line enters (run1065); no \
         reference in the executable (the fiftieth pass's scan over the 192 \
         open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0068_a1b0,
        "MapFairness::point_lowers_highest: a terrain-side getter or builder \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0068_c870,
        "Map::run_map_script: slot +0x68 of all 22 Map-family vtables and no \
         rel32 caller; none of the image's +0x68 dispatches (393 call, 1 jmp, \
         the mov+call forms) has a Map receiver; two readings, item 3891 \
         (docs/audit/2026-11-05-item-3891.md)",
    ),
    (
        0x0068_c8f0,
        "MapScenario::make: MapScenario::vftable +0x64 alone, installed by \
         Map::new_map only for style 0x1a, which only \
         ConquestGame::build_conquest_game mints (a custom-map Conquer the World) \
         and the editor's generate_map reaches; the lobby's and the console's \
         style stops at 22; two readings, item 3891 (docs/audit/2026-11-05-item-3891.md)",
    ),
    (
        0x0068_d1a0,
        "MapConquest::check_sea_map: called only from \
         MapConquest::make_continents+0x994, itself residue on Conquer the \
         World's cut (DECISIONS 106 §3, 107 §3)",
    ),
    (
        0x0068_d260,
        "MapConquest::add_additional_players: its two callers are \
         MapScenario::make+0x550 (style 0x1a) and MapConquest::make_continents+0x986 \
         (styles 0x18/0x19), both residue on Conquer the World's and the editor's \
         cut; no lobby, console command or script reaches either (docs/MAPGEN.md \
         §22.3, item 3908)",
    ),
    (
        0x0068_dba0,
        "MapConquest::make_continents: the mask of map styles 0x18/0x19, which only \
         ConquestGame::build_conquest_game mints (Conquer the World, cut from v1); \
         the skirmish combo stops at 22, the console's map command clamps its parse \
         to 0..22 (range(0x16) at 007d9ec9), and a scenario file's style byte is \
         scenario play, cut (item 3859; the sixty-sixth pass, DECISIONS 106)",
    ),
    (
        0x0068_e6a0,
        "Map::force_continent_spacing: its one caller ScenarioEditor::import_image \
         has two callers, ScenarioEditor::generate_map (the editor, cut from v1) \
         and MapConquest::make_continents (styles 0x18/0x19, Conquer the World, \
         cut); no lobby, console command or shipped skirmish reaches either (two \
         readings, items 3856 and 3859; the sixty-sixth pass, DECISIONS 106)",
    ),
    (
        0x0069_4ab0,
        "Map::make_continents: live only as MapScenario::vftable +0x40 (style \
         0x1a; Map::vftable is the ctor's and dtor's transient vptr), and a \
         MapScenario's +0x64 is MapScenario::make, which dispatches no +0x40, \
         while the other +0x40 dispatcher, Map::run_map_script at +0x68, has no \
         dispatch site on a map in the listing (two readings, item 3859)",
    ),
    (
        0x0069_5050,
        "MapGrass::make_continents: map style 23 (BLANK_MAP), which only \
         ScenarioEditor::generate_map asks Map::new_map for; the lobby's styles \
         stop at 22",
    ),
    (
        0x006b_0180,
        "World::repair_world: ConsoleWin::run_cmd case 0x18, the console-only \
         `repair`, \"rebuilds some world data\"; a debug verb (docs/AI.md §393.2, \
         item 3909)",
    ),
    (
        0x006b_1b80,
        "World::set_orig_coast: no reference in the executable — no call, no \
         vtable slot; the one four-byte match of its RVA, 0x67b5ac, is the rel32 \
         of AmmoOut::draw's call at 0x67b5ab, which the scan excludes (the \
         sixty-seventh pass, DECISIONS 107 §3; MAPGEN §21)",
    ),
    (
        0x006b_1ba0,
        "World::get_was_seen: the out-of-line copy of a function the compiler \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x006b_1d60,
        "World::revert_coast: ScenarioEditor::add_random_terrain's; the editor \
         (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x006b_22e0,
        "World::clear_danger: no reference in the executable",
    ),
    (
        0x006b_2390,
        "WorldData::is_dead_build: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x006b_2510,
        "WorldData::get_who2: the out-of-line copy of a function the compiler \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x006b_3020,
        "WorldData::is_coast: the out-of-line copy of a function the compiler \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x006b_3960,
        "WorldData::get_region2: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x006b_4230,
        "World::set_behind: no reference in the executable; its writers \
         (Wall::mark_behind_tiles, Mountains::add_mountain) carry it inline",
    ),
    (
        0x006b_4290,
        "World::get_seen: the out-of-line copy of a function the compiler \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x006b_46b0,
        "World::set_gathered_at: no reference in the executable",
    ),
    (
        0x006b_4880,
        "WorldData::is_impassable: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x006b_5590,
        "WorldData::is_mountains: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x006b_76a0,
        "World::adjust_fog: the out-of-line copy of a function the compiler \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x006b_88b0,
        "Leader::process: no reference in the executable; inlined in \
         Leaders::process_all, which calls its gather and process_elimination",
    ),
    (
        0x006d_5230,
        "LeaderData::locked_transport: no reference in the executable",
    ),
    (
        0x006d_6660,
        "LeaderData::get_econ_mod: no reference in the executable \
         (docs/NATIONS.md §9.1: the listing answers 0.5, 0.75 or 1.0 by \
         difficulty, dead; the forty-eighth pass)",
    ),
    (
        0x006d_6740,
        "LeaderData::get_handicap_level: no reference in the executable",
    ),
    (
        0x006d_6e80,
        "LeaderData::get_fishermen: no reference in the executable",
    ),
    (
        0x006d_a740,
        "LeaderData::get_handicap: all four callers gate on game semaphore bit \
         2, the multiplayer flag; the capture lanes are solo",
    ),
    (
        0x006e_c170,
        "LeaderData::is_human: no reference in the executable (inlined at every \
         use)",
    ),
    (
        0x006e_de80,
        "Player::drop: DropControl::on_player_timed_out and process_drop; \
         multiplayer (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x006e_fb20,
        "Tribes::walk_data: its one home is Tribes::vftable +0x0, the one \
         instance is the tribes global, and nothing dispatches its slot 0; the \
         save and load walker WalkDataGame::walk_data walks tribes.list inline \
         (two readings, item 3505)",
    ),
    (
        0x006e_fb50,
        "Tribe::sub_nation_name: Scene::set_help_type's help text, under \
         Options::describe; interface (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x006e_fda0,
        "Tribe::set_substitute_tribe: ConquestGame::setup_scenario alone; \
         Conquer the World (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x006e_ff70,
        "Tribe::set_barbarian: ConquestGame::setup_scenario alone; Conquer the \
         World (docs/AI.md §393.2, item 3909)",
    ),
    (0x006f_0230, "Tribe::init: no reference in the executable"),
    (
        0x006f_a090,
        "Army::find_waiting_unit: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x0070_0010,
        "Group::action_board_ship: the board-ship command's action, whose \
         command no site issues (type 0x0f), and the one maker of the \
         BoardOrder (docs/GOLDEN.md §144.4; the forty-eighth pass)",
    ),
    (
        0x0070_20c0,
        "Group::action_repair: the repair command's action, which no \
         add_command site issues (type 0x10) but ScenarioData::issue_order; \
         item 3411's sweep backs its arithmetic (docs/GOLDEN.md §144.4; the \
         forty-eighth pass)",
    ),
    (
        0x0070_7aa0,
        "Group::dbg_jump_to_waypoint: Console::exec_keymap's case 0x19 under \
         cheat_keys, and ScenarioEditor::inc_selected_health; a debug verb \
         (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x0070_7b50,
        "Group::dbg_pack_or_unpack: its one caller, Options::target_spell, \
         takes it only under gamec->semaphore bit 11, which ConsoleWin::run_cmd \
         sets when it opens the in-game scenario editor (and Game::run_editor, \
         ScenarioEditor::end_test and a save load's Game::walk_data); it casts a \
         pack or unpack outside the command stream, and the editor is cut from v1 \
         (DECISIONS 107 §3; docs/AI.md §391.3, §397, items 3906, 3935)",
    ),
    (
        0x0070_ce70,
        "GroupData::show_interface: an interface body — whether the selection \
         panel opens (IFaceMainBase::update_visibility) and the mouseover text \
         (Scene::render_mouseover_text); it reads the selection, the console and \
         the diplomacy and writes nothing (docs/AI.md §391.3, item 3906)",
    ),
    (
        0x0070_f8a0,
        "GroupData::get_speed: the out-of-line copy of a function the compiler \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0071_0aa0,
        "Group::kill_group_order: its call at do_group_attack+0x124 runs only \
         under a GroupAttackOrder, which nothing makes (docs/AI.md §382.1), and \
         its call at find_new_target+0xef needs a group order at the head after \
         repath. Six attempts each shut one step: route 1's cast is refused a \
         capture city (§389, item 3903); route 2's duplicate id has a producer and \
         no entry (§390, 3904); no caller runs find_new_target with the group \
         move current (§394, 3910); the regroup ungroups the move the entry \
         needs (§396.3, 3932); a grouped leader's first friendly touch ungroups \
         it with no search (§398, 3937); and a mutual touch under the ungroup's \
         copy searches to a path without suspending, so no stash is held \
         (§399, 3940)",
    ),
    (
        0x0071_37f0,
        "Group::leader_report_speed: no reference in the executable; its three \
         statements stand inline in Unit::do_group_move",
    ),
    (
        0x0071_3bb0,
        "Group::report_speed: no reference in the executable",
    ),
    (
        0x0072_ff60,
        "TargetOrder::get_target: slot 0 of all fourteen TargetOrder tables, and \
         each of its six .text copies is a guard that inlines the read on \
         equality, which every table satisfies; two readings, item 3893 \
         (docs/audit/2026-11-05-item-3893.md)",
    ),
    (
        0x0073_6470,
        "CityData::get_trade_routes: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0073_69c0,
        "CaravanLink::walk_data: an element's dump walk, inlined into its \
         container's walk, which a `@dumpall` line enters (run1065); no \
         reference in the executable (the fiftieth pass's scan over the 192 \
         open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0073_8180,
        "CityData::pop_growth: the out-of-line copy of a function the compiler \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0073_9ab0,
        "Caster::walk_data: an element's dump walk, inlined into its \
         container's walk, which a `@dumpall` line enters (run1065); no \
         reference in the executable (the fiftieth pass's scan over the 192 \
         open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0073_9e20,
        "Hero::walk_data: an element's dump walk, inlined into its container's \
         walk, which a `@dumpall` line enters (run1065); no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0073_b540,
        "Supply::walk_data: an element's dump walk, inlined into its \
         container's walk, which a `@dumpall` line enters (run1065); no \
         reference in the executable (the fiftieth pass's scan over the 192 \
         open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0073_b5d0,
        "Supply::deplete: the out-of-line copy of a function the compiler \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0073_b880,
        "UnbuiltCity::walk_data: an element's dump walk, inlined into its \
         container's walk, which a `@dumpall` line enters (run1065); no \
         reference in the executable (the fiftieth pass's scan over the 192 \
         open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0073_bb20,
        "UnbuiltFort::walk_data: an element's dump walk, inlined into its \
         container's walk, which a `@dumpall` line enters (run1065); no \
         reference in the executable (the fiftieth pass's scan over the 192 \
         open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0073_c1b0,
        "UnbuiltWonder::walk_data: an element's dump walk, inlined into its \
         container's walk, which a `@dumpall` line enters (run1065); no \
         reference in the executable (the fiftieth pass's scan over the 192 \
         open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0073_c5d0,
        "Wonder::close: the out-of-line copy of a function the compiler inlined \
         at every use, or one nothing calls; no reference in the executable \
         (the fiftieth pass's scan over the 192 open rows, 2026-10-22, Loop \
         3465)",
    ),
    (
        0x0073_c5e0,
        "Wonder::init: no reference in the executable (docs/TECH.md, what is \
         not established: its body is inlined in Wonders::init_wonder and the \
         byte before it is Wonder::close's ret; the forty-eighth pass)",
    ),
    (
        0x0073_c780,
        "Wonder::walk_data: an element's dump walk, inlined into its \
         container's walk, which a `@dumpall` line enters (run1065); no \
         reference in the executable (the fiftieth pass's scan over the 192 \
         open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0073_e000,
        "Caravan::process: no reference in the executable (its address's one \
         copy is a coincidence inside .reloc's fixup entries); its body, `if \
         (making_road) build_road()`, stands inline in Unit::work",
    ),
    (
        0x0073_ebc0,
        "Fort::walk_data: an element's dump walk, inlined into its container's \
         walk, which a `@dumpall` line enters (run1065); no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0073_f650,
        "OilWell::walk_data: an element's dump walk, inlined into its \
         container's walk, which a `@dumpall` line enters (run1065); no \
         reference in the executable (the fiftieth pass's scan over the 192 \
         open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0074_0030,
        "Special::walk_data: no reference in the executable; Specials::walk_data \
         inlines it, for the save, the load and the checksums (two readings, item \
         3505)",
    ),
    (
        0x0074_0060,
        "SpecialData::get_radius: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0074_1a40,
        "Herd::walk_data: an element's dump walk, inlined into its container's \
         walk, which a `@dumpall` line enters (run1065); no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0074_1d50,
        "Herds::walk_data: an element's dump walk, inlined into its container's \
         walk, which a `@dumpall` line enters (run1065); no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0078_be90,
        "ConquestGame::setup_scenario_bonuses: Conquer the World only (its one \
         caller is build_conquest_game, under semaphore[2] & 2), cut from v1 \
         (docs/NATIONS.md §9.1, §9.3; the forty-eighth pass)",
    ),
    (
        0x0079_6140,
        "ConquestGame::build_cities: Conquer the World only \
         (Setup::build_empire takes it under semaphore[2] & 2), cut from v1 \
         (docs/NATIONS.md §9.1, §9.3; the forty-eighth pass)",
    ),
    (
        0x0079_7200,
        "ConquestGame::build_units: Conquer the World (docs/AI.md §393.2, item \
         3909)",
    ),
    (
        0x0079_7c40,
        "ConquestGame::build_conquest_game: Conquer the World's start, cut from \
         v1 (docs/NATIONS.md §9.1, §9.3; the forty-eighth pass)",
    ),
    (
        0x0079_8880,
        "ConquestGame::place_reinforcements: Game::do_frame under semaphore bit \
         17, which Game::run_solo sets only for an initialized conquest game; \
         Conquer the World (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x0079_99e0,
        "ConquestGame::setup_scenario: Conquer the World (docs/AI.md §393.2, \
         item 3909)",
    ),
    (
        0x0079_ea80,
        "ConquestLeader::plan_strategy: the campaign map's turn, from \
         ConquestWin::post_turn_processing, cut from v1 (docs/NATIONS.md §9.1, \
         §9.3; the forty-eighth pass)",
    ),
    (
        0x007a_a400,
        "ReinforcementArmy::check_edge: ReinforcementArmy::place_army alone; \
         Conquer the World (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x007a_a560,
        "ReinforcementArmy::place_army: ConquestGame::place_reinforcements \
         alone; Conquer the World (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x0081_2d20,
        "IFaceMarket::on_redraw: interface only, the Market window's text \
         through vslot +0xb8; writes no sim state (docs/NATIONS.md §9.1, §9.3; \
         the forty-eighth pass)",
    ),
    (
        0x0081_34d0,
        "IFaceMarket::setup_selected: interface only, the Market window's Buy \
         and Sell buttons through vslot +0x4f8; writes no sim state \
         (docs/NATIONS.md §9.1, §9.3; the forty-eighth pass)",
    ),
    (
        0x0084_2050,
        "Storm::on_field_change: live in no object: Storm::vftable +0xc, a vtable \
         written only by Storm's two destructors, of a class nothing constructs \
         (no Storm::Storm), and StormOut::registration, its one registration \
         with the parameter window, has no reference (two readings, item 3861)",
    ),
    (
        0x0084_2060,
        "Storm::registration: live in no object: Storm::vftable +0x8, written \
         only by Storm's two destructors, of a class nothing constructs (two \
         readings, item 3861)",
    ),
    (
        0x0084_2070,
        "StormOut::registration: the storm's one registration with the parameter \
         window; no reference in the executable (two readings, item 3861)",
    ),
    (
        0x0084_b8d0,
        "Terrain::convert_errant_sandy: a terrain-side getter or builder \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0086_1960,
        "TerrainOut::generate_transitions: no reference in the executable (the \
         sixty-sixth pass's scan; the byte before is the preceding function's own \
         ret at 0x86195f, so no fall-through)",
    ),
    (
        0x0089_0060,
        "RoadsOut::render_debug: RoadsOut::render's debug pass; the renderer \
         (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x0089_4060,
        "add_single_line: RoadsOut::render_debug's; the renderer (docs/AI.md \
         §393.2, item 3909)",
    ),
    (
        0x0089_41b0,
        "add_line: RoadsOut::render_debug's (not StatWin::add_line); the \
         renderer (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x0089_7a50,
        "Signposts::cur_y: a terrain-side getter or builder inlined at every \
         use, or one nothing calls; no reference in the executable (the \
         fiftieth pass's scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0089_7a60,
        "Signposts::cur_x: a terrain-side getter or builder inlined at every \
         use, or one nothing calls; no reference in the executable (the \
         fiftieth pass's scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0089_7bc0,
        "Signposts::get_num: a terrain-side getter or builder inlined at every \
         use, or one nothing calls; no reference in the executable (the \
         fiftieth pass's scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0089_7e20,
        "Signposts::add: a terrain-side getter or builder inlined at every use, \
         or one nothing calls; no reference in the executable (the fiftieth \
         pass's scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0089_8920,
        "Mountains::sliding_verify: Mountains::add_mountain's case 3, and none \
         of add_mountain's four callers passes 3 (map generation passes 4 and \
         5) (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x0089_8cf0,
        "Mountains::verify_template: Mountains::add_mountain's case 1, which \
         only ScenarioEditor::modify_terrain_map passes; the editor (docs/AI.md \
         §393.2, item 3909)",
    ),
    (
        0x0089_8f10,
        "Mountains::remove_mountain: TerrainMap::remove_mountain alone; the \
         editor (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x0089_be90,
        "MountainsOut::mnt_locs: a terrain-side getter or builder inlined at \
         every use, or one nothing calls; no reference in the executable (the \
         fiftieth pass's scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0089_bea0,
        "MountainsOut::mnt_ranges: a terrain-side getter or builder inlined at \
         every use, or one nothing calls; no reference in the executable (the \
         fiftieth pass's scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0089_beb0,
        "MountainsOut::mnt_types: a terrain-side getter or builder inlined at \
         every use, or one nothing calls; no reference in the executable (the \
         fiftieth pass's scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0089_d3a0,
        "Mountains::tileset_texture_load: a terrain-side getter or builder \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0089_ddc0,
        "Surf::on_field_change: Surf::vftable +0xc, re-phasing the shore waves \
         in floats; the renderer (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x008a_4040,
        "Cliff::walk_data: an element's dump walk, inlined into its container's \
         walk, which a `@dumpall` line enters (run1065); no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x008a_4060,
        "Cliff::average_neighbor_norms: a terrain-side getter or builder \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x008a_60c0,
        "Cliff::render: a terrain-side getter or builder inlined at every use, \
         or one nothing calls; no reference in the executable (the fiftieth \
         pass's scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x008a_60e0,
        "Cliffs::delete_cliff: ScenarioEditor::modify_terrain_map and \
         TerrainMap::remove_cliff; the editor (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x008b_0060,
        "TerrainVis::get_verts_per_bucket: a terrain-side getter or builder \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x008b_13b0,
        "TerrainVis::wcoord_was_cached: a terrain-side getter or builder \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x008b_2a80,
        "TerrainVis::set_base_source_pool: a terrain-side getter or builder \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x008c_6db0,
        "CircleRender::on_field_change: the debug parameter window only; the \
         circle's registration from Game::init under Misc Effects is reached by a \
         Scroll of ParamRegisterWindow and nothing else (two readings, item 3861; \
         the sixty-sixth pass, DECISIONS 106)",
    ),
    (
        0x008c_7810,
        "WorldMap::on_extra_up: one .rdata pointer, the minimap's extra-button \
         release into Console::map_click; interface (docs/AI.md §393.2, item \
         3909)",
    ),
    (
        0x008d_1830,
        "WorldMapFog::get_fog_lookup: a terrain-side getter or builder inlined \
         at every use, or one nothing calls; no reference in the executable \
         (the fiftieth pass's scan over the 192 open rows, 2026-10-22, Loop \
         3465)",
    ),
    (
        0x008d_9190,
        "FarmsData::get_wheat_status: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x008d_92d0,
        "Farms::add_animals: the out-of-line copy of a function the compiler \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0092_8f30,
        "NukeOut::update: the out-of-line copy of a function the compiler \
         inlined at every use, or one nothing calls; no reference in the \
         executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0092_d0b0,
        "angle_diff: the out-of-line copy of a function the compiler inlined at \
         every use, or one nothing calls; no reference in the executable (the \
         fiftieth pass's scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x0092_fc50,
        "GameLog::dump_armies: no reference in the executable, and gamelog.ini \
         has no ARMY key",
    ),
    (
        0x0093_43b0,
        "ParamRegisterWindow::on_scroll: the debug parameter window's scroll \
         handler, its one home ParamRegisterWindow::vftable +0x330; the window is \
         keymap 0x1e under Console::cheat_keys and out of v1 (item 3861; the \
         sixty-sixth pass, DECISIONS 106)",
    ),
    (
        0x0093_a140,
        "SyncPoint::on_player_deleted: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0093_a150,
        "SyncPoint::process_sync_signal: NetDaemon::process's; multiplayer \
         (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x0093_e9e0,
        "CommandManager::recover_from_oos: CommandManager::process_turn's \
         out-of-sync recovery; multiplayer (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x0093_ee70,
        "CommandManager::set_mp_playback: multiplayer replay playback, a prefs2 \
         path read under netsys before ReplayWin opens; no skirmish capture \
         reaches it (docs/GOLDEN.md §144.4; the forty-eighth pass)",
    ),
    (
        0x0094_3f30,
        "CommandPackage::process_console_cmd: its one writer, \
         CommandManager::issue_console_cmd@00940fd0, packages a 0x209-byte \
         command and CommandPackage::add_command@0094bae0 refuses any that \
         would pass 0x200, so every netcmd is dropped before it is packaged; \
         run1088 entered issue_console_cmd on 640 and printed no \
         process_command_cmd line at COMMANDMANAGER=2 (parked 3471a; the \
         fiftieth pass)",
    ),
    (
        0x0094_8cb0,
        "CommandPackage::process_repair: no add_command site in the image \
         writes type 0x10 (102 sites read); the command's other issuer is \
         ScenarioData::issue_order, a scenario file's initial orders \
         (docs/GOLDEN.md §144.4; the forty-eighth pass)",
    ),
    (
        0x0094_8e00,
        "CommandPackage::process_board_ship: no add_command site in the image \
         writes type 0x0f; its sync line is mislabelled process_set_transport \
         (docs/GOLDEN.md §144.4; the forty-eighth pass)",
    ),
    (
        0x0094_9050,
        "CommandPackage::process_transport: no add_command site in the image \
         writes type 0x0d (102 sites read); Group::action_transport's live \
         caller is ScenarioFuncSet::add_unit, which the console's eval reaches \
         (docs/AI.md §391.1, §391.3, item 3906)",
    ),
    (
        0x0094_95c0,
        "CommandPackage::process_move_near: command byte 8 has no emitter \
         (COMMANDS §3, §6; AI §382.3) and process dispatches only what a \
         recording carries — a hand-edited .rec is not play (parked 3505d, \
         ruled DECISIONS 107 §3); reopened by any add_command site or console \
         verb that writes byte 8",
    ),
    (
        0x0094_c1c0,
        "CommandPackage::clear: no reference in the executable",
    ),
    (
        0x0095_4c60,
        "TimeSync::test_accept_ping: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0095_6470,
        "TurnControl::end_cannon_time: check_cannon_time's one call, gated on \
         cannon_time_who >= 0, whose one non-negative writer is \
         start_cannon_time <- command 0x4d <- issue_cannon_time, refused \
         unless semaphore bit 2, set only by Game::run_gamespy; a save \
         carrying bit 2 is headed RoNMultiSave, which every solo load \
         refuses; out on scope (two readings, item 3897, \
         docs/audit/2026-11-05-item-3897.md)",
    ),
    (
        0x0095_6ac0,
        "TurnControl::is_cannon_time: the out-of-line copy of a function the \
         compiler inlined at every use, or one nothing calls; no reference in \
         the executable (the fiftieth pass's scan over the 192 open rows, \
         2026-10-22, Loop 3465)",
    ),
    (
        0x0095_9110,
        "DropControl::on_player_timed_out: multiplayer (docs/AI.md §393.2, item \
         3909)",
    ),
    (
        0x0095_9500,
        "DropControl::process_drop: multiplayer (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x0096_2c70,
        "GameSpy::UpdateLobbyFromDTO: the online lobby, GameSpy, cut from v1; its \
         three callers are GameSpy::OnLobbyJoined and two lambdas of the lobby \
         client (item 3859; the sixty-sixth pass)",
    ),
    (
        0x0099_6ac0,
        "ScenarioData::issue_order: a scenario file's initial orders, applied \
         once at ScenarioRead::import, not the trigger system; no capture loads \
         a scenario (docs/GOLDEN.md §144.2, §144.4; the forty-eighth pass)",
    ),
    (
        0x0099_bc20,
        "ScenarioEditor::set_transport: called only from \
         IFaceEditorNations::on_combobox_select+0x23e, the editor's nations \
         page; the editor is cut from v1 (DECISIONS 107 §3)",
    ),
    (
        0x0099_c750,
        "ScenarioEditor::add_unit: Options::picked_spot (the editor) and \
         ConquestGame::build_units (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x009a_0950,
        "ScenarioEditor::generate_map: the scenario editor, cut from v1; its two \
         callers are Options::exec_editor's (item 3859; the sixty-sixth pass)",
    ),
    (
        0x009a_1380,
        "ScenarioEditor::make_city_capital: the scenario editor, cut from v1; it \
         calls SpellType::cast_move_capital directly, the one caller of that body \
         outside SpellType::cast (item 3907, docs/AI.md §392)",
    ),
    (
        0x009a_3f30,
        "ScenarioEditor::import_image: the editor's image import and the Conquer \
         the World mask's (MapConquest::make_continents), both cut from v1 (item \
         3859; the sixty-sixth pass)",
    ),
    (
        0x009a_adc0,
        "ScenarioRead::load_unit_chunk: called only from load_objects_chunk, \
         called only from ScenarioRead::import, whose callers are scenario play \
         (build_game's is_scenario arm, load_scenario_screen), the editor and \
         MapScenario::make; a saved game loads through WalkDataGame::walk_data and \
         the console calls build_game with 0, 0, 0; two readings, item 3891 \
         (docs/audit/2026-11-05-item-3891.md)",
    ),
    (
        0x009a_b9c0,
        "ScenarioRead::load_objects_chunk: called only from ScenarioRead::import \
         (case 5), whose callers are scenario play, the editor and \
         MapScenario::make; two readings, item 3891 (docs/audit/2026-11-05-item-3891.md)",
    ),
    (
        0x009a_df40,
        "ScenarioRead::load_mountains_chunk: ScenarioRead::load_terrain_chunk \
         under ScenarioRead::import, a scenario file's import that no \
         random-map game takes (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x009b_1430,
        "ScenarioRead::import: a scenario file's load — Setup::build_game's \
         scenario arm, MapScenario::make (style 0x1a) and the editor's test and \
         load screens; scenario play is cut from v1 with the editor and the trigger \
         system (Game::run_scenario's row, the forty-eighth pass; item 3859; the \
         sixty-sixth pass, DECISIONS 106)",
    ),
    (
        0x009b_2e40,
        "ScenarioRead::load_scenario_screen: called only from Game::run_scenario \
         and ScenarioEditor::load_file, scenario play and the editor (cut from \
         v1); two readings, item 3891 (docs/audit/2026-11-05-item-3891.md)",
    ),
    (
        0x009b_32f0,
        "TerrainMap::river_point: a terrain-side getter or builder inlined at \
         every use, or one nothing calls; no reference in the executable (the \
         fiftieth pass's scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x009b_3320,
        "TerrainMap::start_river: a terrain-side getter or builder inlined at \
         every use, or one nothing calls; no reference in the executable (the \
         fiftieth pass's scan over the 192 open rows, 2026-10-22, Loop 3465)",
    ),
    (
        0x009b_3340,
        "TerrainMap::remove_cliff: TerrainMap::remove_feature's, under the \
         editor's terrain window (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x009b_3590,
        "TerrainMap::remove_river: TerrainMap::remove_feature's, under the \
         editor's terrain window (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x009b_37a0,
        "TerrainMap::remove_mountain: TerrainMap::remove_feature's, under the \
         editor's terrain window (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x009b_39e0,
        "TerrainMap::remove_feature: TerrainMap::draw and on_left_down, the \
         editor's terrain window (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x009b_4820,
        "TerrainMap::on_right_up: a mouse handler of the editor's TerrainMap, \
         which IFaceEditorMain::init alone builds; its vtable pointer only; \
         the editor is cut from v1 (DECISIONS 107 §3)",
    ),
    (
        0x009b_4930,
        "TerrainMap::on_left_up: a mouse handler of the editor's TerrainMap, \
         which IFaceEditorMain::init alone builds; its vtable pointer only; \
         the editor is cut from v1 (DECISIONS 107 §3)",
    ),
    (
        0x009b_4da0,
        "TerrainMap::on_redraw: IFaceEditorTerrainMap::on_redraw's; the editor \
         (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x009b_59f0,
        "TerrainMap::map_change: ScenarioEditor::generate_map, \
         modify_terrain_map, reset_terrain_map and load_file; the editor \
         (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x009b_5d20,
        "TerrainMap::river_stop: called only by TerrainMap::on_left_down and \
         ScenarioEditor::modify_terrain_map, the editor's UI (MAPGEN §21, \
         parked 3889a); the editor is cut from v1 (DECISIONS 107 §3)",
    ),
    (
        0x009b_5e00,
        "TerrainMap::erase_highlight: ScenarioEditor::modify_terrain_map's; the \
         editor (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x009b_5ec0,
        "TerrainMap::get_terrain_type: a terrain-side getter or builder inlined \
         at every use, or one nothing calls; no reference in the executable \
         (the fiftieth pass's scan over the 192 open rows, 2026-10-22, Loop \
         3465)",
    ),
    (
        0x009b_5f20,
        "TerrainMap::synch_mountain_data: ScenarioEditor::add_random_features, \
         generate_map and load_file; the editor (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x009b_6de0,
        "UnitBalance::write_screen_details: UnitBalance::next's; the \
         unit-balance harness (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x009b_6fa0,
        "UnitBalance::write_sub_header: UnitBalance::next's; the unit-balance \
         harness (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x009b_7060,
        "UnitBalance::write_header: UnitBalance::next's; the unit-balance \
         harness (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x009b_71c0,
        "UnitBalance::valid_combat: UnitBalance::next's; the unit-balance \
         harness (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x009b_7d30,
        "UnitBalance::check_for_timeout: Game::check_victory under semaphore \
         bit 9, which only Game::run_unit_balance sets; the unit-balance \
         harness (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x009b_7de0,
        "UnitBalance::write_results: Game::check_victory under semaphore bit 9; \
         the unit-balance harness (docs/AI.md §393.2, item 3909)",
    ),
    (
        0x009b_8ac0,
        "UnitBalance::next: its three callers (Game::check_victory, \
         Setup::build_empire, TurnControl::toggle_pause) reach it only under \
         game semaphore byte 1's bit 2, which Game::run_unit_balance alone sets \
         — the GAME_UNIT_BALANCE shell mode, not a match (item 1182)",
    ),
    (
        0x009f_b2d0,
        "ScenarioFuncSet::group_jump_move: its overload at 009fa710, on a named \
         ScenarioGroup; the trigger system, cut from v1 (docs/AI.md §393.2, \
         item 3909)",
    ),
    (
        0x00a4_69f0,
        "cos_table: no reference in the executable; a two-instruction thunk \
         (add ecx, 0x3fffffff; jmp sin_table) whose one use, \
         MapGrass::make_continents, adds the constant inline",
    ),
];

/// The five trampolined functions (`tools/trace/tracer.c`'s `HOOKS`), by
/// RVA, and the record kind each emits. They carry no coverage stub, so a
/// trace enters one exactly when it holds a record of its kind —
/// `report.py`'s own rule.
const HOOKS: [(u32, u32); 5] = [
    (0x0019_1ef0, 2), // `Game::do_frame`, FRAME
    (0x0063_9cf0, 1), // `Random::get()`
    (0x0063_9d70, 3), // `Random::get(a, b)`
    (0x005e_18b0, 4), // `rand_real`
    (0x0063_9d30, 6), // `Random::reseed`
];

/// The functions one trace entered, folded to the export's numbering.
/// `None` when the bytes are not a trace. Reads only the record kind and
/// the `HIT` address, so a 93 MB trace costs one pass and no allocation
/// beyond the set.
pub fn entered(bytes: &[u8]) -> Option<std::collections::BTreeSet<u32>> {
    let word = |off: usize| {
        u32::from_le_bytes([bytes[off], bytes[off + 1], bytes[off + 2], bytes[off + 3]])
    };
    if bytes.len() < 32 || word(0) != 0x544E_4F52 {
        return None;
    }
    let base = word(8);
    let mut out = std::collections::BTreeSet::new();
    let mut kinds = [false; 16];
    for rec in bytes[32..].chunks_exact(32) {
        let kind = u32::from_le_bytes([rec[0], rec[1], rec[2], rec[3]]);
        if kind == 0 {
            let va = u32::from_le_bytes([rec[4], rec[5], rec[6], rec[7]]);
            out.insert(va.wrapping_sub(base).wrapping_add(crate::trace::IMAGE_BASE));
        } else if let Some(k) = kinds.get_mut(kind as usize) {
            *k = true;
        }
    }
    for (rva, kind) in HOOKS {
        if kinds[kind as usize] {
            out.insert(rva + crate::trace::IMAGE_BASE);
        }
    }
    Some(out)
}

/// Every address a document's text cites as `name@00xxxxxx`, in the
/// grammar of `report.py`'s `blind` verb: the character before the `@` is
/// a name's (`[A-Za-z0-9_:~<>]`, with a letter or `_` somewhere in the
/// run), and a line break with its indentation may stand between the `@`
/// and the address (parked 835).
pub fn cited(text: &str) -> Vec<u32> {
    let b = text.as_bytes();
    let namey = |c: u8| c.is_ascii_alphanumeric() || b"_:~<>".contains(&c);
    let mut out = Vec::new();
    for (at, _) in text.match_indices('@') {
        let mut i = at;
        let mut lead = false;
        while i > 0 && namey(b[i - 1]) {
            i -= 1;
            lead |= b[i].is_ascii_alphabetic() || b[i] == b'_';
        }
        if i == at || !lead {
            continue;
        }
        let mut j = at + 1;
        if b.get(j) == Some(&b'\n') {
            j += 1;
            while matches!(b.get(j), Some(b' ' | b'\t')) {
                j += 1;
            }
        }
        let Some(hex) = b.get(j..j + 8) else { continue };
        if hex.starts_with(b"00")
            && hex
                .iter()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(c))
            && let Ok(a) = u32::from_str_radix(std::str::from_utf8(hex).unwrap_or(""), 16)
        {
            out.push(a);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    /// `docs/**/*.md`, each cited address with the documents that cite it.
    fn citations() -> BTreeMap<u32, BTreeSet<String>> {
        fn walk(
            dir: &std::path::Path,
            root: &std::path::Path,
            out: &mut BTreeMap<u32, BTreeSet<String>>,
        ) {
            let mut entries: Vec<_> = std::fs::read_dir(dir)
                .expect("docs/ is readable")
                .flatten()
                .map(|e| e.path())
                .collect();
            entries.sort();
            for p in entries {
                if p.is_dir() {
                    walk(&p, root, out);
                } else if p.extension().is_some_and(|x| x == "md") {
                    let text = std::fs::read_to_string(&p).unwrap_or_default();
                    let rel = p.strip_prefix(root).unwrap_or(&p).display().to_string();
                    for a in cited(&text) {
                        out.entry(a).or_default().insert(rel.clone());
                    }
                }
            }
        }
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
        let mut out = BTreeMap::new();
        walk(&root, &root, &mut out);
        out
    }

    /// **The blind list is the pinned residue, function for function.**
    ///
    /// Fails in either direction and names each function that moved: one
    /// cited and entered by no pinned trace that [`NEVER`] does not list
    /// (a reading's new citation — take the capture that enters it, or
    /// add it to the residue on purpose), and one [`NEVER`] lists that is
    /// now entered or no longer cited (a capture joined [`TRACES`], or a
    /// citation went — take it off, so the list only ever shrinks by a
    /// line someone wrote).
    #[test]
    fn the_blind_list_is_the_pinned_residue() {
        let home = std::env::var("HOME").unwrap_or_default();
        let index_path = format!("{home}/ghidra-projects/decomp/INDEX.tsv");
        let Ok(index) = std::fs::read_to_string(&index_path) else {
            eprintln!("skipping: no {index_path} (the Ghidra export is not on this machine)");
            return;
        };
        let names: BTreeMap<u32, &str> = index
            .lines()
            .filter_map(|l| {
                let mut it = l.split('\t');
                Some((u32::from_str_radix(it.next()?, 16).ok()?, it.next()?))
            })
            .collect();
        assert!(names.len() > 40_000, "the export's index is 48k functions");

        let mut entered_any = BTreeSet::new();
        for name in TRACES {
            let Some(path) = crate::testenv::dump(name) else {
                eprintln!("skipping: no {name} (set RON_GAMELOG_DIR)");
                return;
            };
            let bytes = std::fs::read(&path).expect("a kept trace is readable");
            entered_any.extend(entered(&bytes).unwrap_or_else(|| panic!("{name} is not a trace")));
        }

        let cites = citations();
        let never: BTreeSet<u32> = cites
            .keys()
            .copied()
            .filter(|a| names.contains_key(a) && !entered_any.contains(a))
            .collect();
        let pinned: BTreeSet<u32> = NEVER.iter().copied().collect();
        assert_eq!(pinned.len(), NEVER.len(), "NEVER lists an address twice");
        assert!(NEVER.is_sorted(), "NEVER is kept ascending");

        let name = |a: &u32| names.get(a).copied().unwrap_or("?");
        let mut failures = Vec::new();
        for a in never.difference(&pinned) {
            let docs: Vec<&str> = cites[a].iter().map(String::as_str).collect();
            failures.push(format!(
                "  new on the blind list: `{}@{a:08x}` is cited ({}) and no trace in TRACES enters it",
                name(a),
                docs.join(", ")
            ));
        }
        for a in pinned.difference(&never) {
            let why = if entered_any.contains(a) {
                "is now entered"
            } else {
                "is no longer cited"
            };
            failures.push(format!("  off the blind list: `{}@{a:08x}` {why}", name(a)));
        }
        assert!(
            failures.is_empty(),
            "the blind list moved ({} cited, {} never, {} pinned) — re-pin `blind::NEVER` \
             to the measurement, or take the capture a new line names:\n{}",
            cites.keys().filter(|a| names.contains_key(a)).count(),
            never.len(),
            NEVER.len(),
            failures.join("\n")
        );
    }

    /// **Every accepted residue row is still on the blind list**, once and
    /// in order — a residue row a capture entered, or whose citation went,
    /// is a row to delete, not a reason that still stands.
    #[test]
    fn the_residue_is_on_the_blind_list() {
        assert!(
            RESIDUE.is_sorted_by_key(|r| r.0),
            "RESIDUE is kept ascending"
        );
        for (a, why) in RESIDUE {
            assert!(
                NEVER.binary_search(a).is_ok(),
                "residue row `{why}` ({a:08x}) is not on NEVER — delete the row"
            );
        }
    }

    /// **Every row that says "no reference" has none**: the image holds
    /// no `call`/`jmp`/`jcc rel32` and no four-byte pointer to it
    /// (`crate::pe::Pe::references`). The controls are the two of item
    /// 935's rows that are referenced, so the scan is seen to find a caller
    /// and a vtable slot before its silence is believed.
    #[test]
    fn every_unreferenced_row_is_unreferenced_in_the_image() {
        let Some(root) = crate::testenv::install_root() else {
            eprintln!("skipping: no install (set RON_INSTALL)");
            return;
        };
        let pe = crate::pe::Pe::open(&format!("{root}/riseofnations.exe"))
            .expect("riseofnations.exe is a PE file");
        assert_eq!(
            pe.references(0x006d_a740),
            vec![0x0065_0b67, 0x0065_1780, 0x006b_107f, 0x006d_66bd],
            "LeaderData::get_handicap's four callers"
        );
        assert_eq!(
            pe.references(0x0069_5050),
            vec![0x00b4_5634],
            "MapGrass::make_continents's vtable slot"
        );
        assert_eq!(
            pe.references(0x0069_4ab0),
            vec![0x00b4_54dc, 0x00b4_5da8],
            "Map::make_continents's two vtable slots, MapScenario's and Map's \
             +0x40, and no call (item 3859)"
        );
        assert_eq!(
            pe.references(0x0067_7590),
            vec![0x00b4_4ed0, 0x00b4_4f90],
            "ItemOut::draw_overlays's two vtable slots, ItemOut's and Item's \
             +0x5c, and no call (item 3861)"
        );
        assert_eq!(
            pe.references(0x0084_2050),
            vec![0x00b5_3f1c],
            "Storm::on_field_change's one vtable slot, Storm's +0xc, and no call \
             (item 3861)"
        );
        assert_eq!(
            pe.references(0x0084_2070),
            Vec::<u32>::new(),
            "StormOut::registration, the storm's one registration with the \
             parameter window, is referenced nowhere (item 3861)"
        );
        assert_eq!(
            pe.references(0x006b_1b80),
            Vec::<u32>::new(),
            "World::set_orig_coast: no call and no vtable slot; the one four-byte match \
             of its RVA, 0x67b5ac, is the rel32 of AmmoOut::draw's call at 0x67b5ab, \
             which this scan excludes (the sixty-seventh pass, DECISIONS 107 §3)"
        );
        let mut failures = Vec::new();
        for (a, why) in RESIDUE {
            if why.contains("no reference in the executable") {
                let sites = pe.references(*a);
                if !sites.is_empty() {
                    failures.push(format!("  `{why}` ({a:08x}): referenced at {sites:08x?}"));
                }
            }
        }
        assert!(
            failures.is_empty(),
            "a residue row says unreferenced and the image says otherwise — \
             read the site (`report.py <exe> refs`) and fix the row:\n{}",
            failures.join("\n")
        );
    }

    /// **The two rows the image points at and nothing calls** (item 959).
    /// `Build::finished@00470e50` sits in four vtables at slot `+0x1ac`, and
    /// the one `call`/`jmp [reg+0x1ac]` in `.text` is `jmp [eax+0x1ac]` at
    /// `0041c9f8`, a COMDAT-folded `Window` thunk; the control is slot
    /// `+0x1b0`, `Build::finished@00628490`'s, which the traces enter.
    /// `Caravan::process`'s address occurs once, inside `.reloc`
    /// (`0x00f15000..`), whose entries are two-byte fixups, never pointers.
    #[test]
    fn the_pointed_at_rows_are_never_called() {
        let Some(root) = crate::testenv::install_root() else {
            eprintln!("skipping: no install (set RON_INSTALL)");
            return;
        };
        let pe = crate::pe::Pe::open(&format!("{root}/riseofnations.exe"))
            .expect("riseofnations.exe is a PE file");
        assert_eq!(
            pe.references(0x0047_0e50),
            vec![0x00b4_2320, 0x00b4_2500, 0x00b4_2888, 0x00b4_2ea4],
            "Build::finished@00470e50's four vtable slots"
        );
        assert_eq!(
            pe.dispatches(0x0047_0e50, 0x1ac),
            vec![0x0041_c9f8],
            "the one jmp through slot +0x1ac, Window's"
        );
        assert!(
            !pe.dispatches(0x0047_0e50, 0x1b0).is_empty(),
            "slot +0x1b0, Build::finished@00628490's, is dispatched"
        );
        // Parked 967, the seventeenth pass: the scan counted this copy, at
        // `00f85888`, and `report.py refs` printed it "pointer only". The
        // base relocation directory holds page headers and two-byte
        // fixups; four of its bytes spelling an address is a coincidence,
        // and the scan skips the directory now.
        assert_eq!(
            pe.references(0x0073_e000),
            Vec::<u32>::new(),
            "Caravan::process's one copy is inside .reloc, which holds no pointer"
        );
    }

    /// **A function entered only by the DLL is off the blind list and
    /// unreferenced in the image**: were the game to call one, it would
    /// not belong apart; were it back on [`NEVER`], a trace stopped
    /// entering it.
    #[test]
    fn what_only_the_dll_enters_is_off_the_list_and_unreferenced() {
        for (a, why) in ENTERED_BY_THE_DLL_ONLY {
            assert!(!NEVER.contains(a), "`{why}` ({a:08x}) is back on NEVER");
        }
        let Some(root) = crate::testenv::install_root() else {
            eprintln!("skipping the image half: no install (set RON_INSTALL)");
            return;
        };
        let pe = crate::pe::Pe::open(&format!("{root}/riseofnations.exe"))
            .expect("riseofnations.exe is a PE file");
        for (a, why) in ENTERED_BY_THE_DLL_ONLY {
            let sites = pe.references(*a);
            assert!(
                sites.is_empty(),
                "`{why}` ({a:08x}) is referenced at {sites:08x?}"
            );
        }
    }

    /// **The `@` issuer guard reads through the coverage `jmp`** (item
    /// 934). `tools/trace/issue_guard_test.c` builds `issue_guard.h` — the
    /// guard `tracer.c` compiles — on the host and asks it about the
    /// shipped prologue, `arm_all`'s `jmp` to the entry's own stub (called:
    /// run314 refused it with code 2), and five images it must refuse. With
    /// an install, each verb's issuer is also armed from the coverage table
    /// `build.sh` wrote beside the DLL and must be called. Made to fail
    /// first on the guard run314 ran: every issuer refused.
    #[test]
    fn the_issuer_guard_reads_through_the_coverage_jmp() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/trace");
        let exe = std::env::temp_dir().join(format!("issue_guard_test-{}", std::process::id()));
        let built = std::process::Command::new("cc")
            .args(["-Wall", "-Wextra", "-Werror", "-o"])
            .arg(&exe)
            .arg(format!("{root}/issue_guard_test.c"))
            .output();
        match built {
            Ok(o) if o.status.success() => {}
            Ok(o) => panic!("issue_guard_test.c: {}", String::from_utf8_lossy(&o.stderr)),
            Err(e) => {
                eprintln!("skipping: no host C compiler ({e})");
                return;
            }
        }
        let mut run = std::process::Command::new(&exe);
        match crate::testenv::install_root().map(|r| format!("{r}/rontrace.funcs")) {
            Some(table) if std::path::Path::new(&table).exists() => {
                run.arg(table);
            }
            _ => eprintln!("the table half skipped: no rontrace.funcs in the install"),
        }
        let out = run.output().expect("issue_guard_test runs");
        let _ = std::fs::remove_file(&exe);
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(out.status.success(), "the issuer guard:\n{text}");
    }

    /// **The log stride** (Loop 3085, item 3117): `rontrace.cfg`'s
    /// `stride=N` rewrites the engine's log window at every `do_frame` entry
    /// so one block in `N` of `--log-window` is dumped. `stride.h` is the
    /// decision, compiled here as the DLL compiles it.
    #[test]
    fn the_log_stride_opens_one_block_in_n() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/trace");
        let exe = std::env::temp_dir().join(format!("stride_test-{}", std::process::id()));
        let built = std::process::Command::new("cc")
            .args(["-Wall", "-Wextra", "-Werror", "-o"])
            .arg(&exe)
            .arg(format!("{root}/stride_test.c"))
            .output();
        match built {
            Ok(o) if o.status.success() => {}
            Ok(o) => panic!("stride_test.c: {}", String::from_utf8_lossy(&o.stderr)),
            Err(e) => {
                eprintln!("skipping: no host C compiler ({e})");
                return;
            }
        }
        let out = std::process::Command::new(&exe)
            .output()
            .expect("stride_test runs");
        let _ = std::fs::remove_file(&exe);
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(out.status.success(), "the log stride:\n{text}");
    }

    /// The grammar: a split citation is joined, a bare address is not a
    /// citation, and a name's template brackets are part of it.
    #[test]
    fn a_citation_is_a_name_an_at_and_an_address() {
        let text = "`Unit::do_patrol@005f1910` and `Group::action_\
                    patrol@\n    007030c0`, `Foo<Bar>@00400000`, \
                    and not @00400010, 0x10@00400020, nor `x@0040003`.";
        assert_eq!(
            cited(text),
            vec![0x005f_1910, 0x0070_30c0, 0x0040_0000, 0x0040_0020]
        );
    }

    /// The scanner: a `HIT` folds to the export's base, a hook is entered
    /// by its record kind alone, and something that is not a trace is
    /// refused.
    #[test]
    fn a_trace_enters_its_hits_and_its_hooks() {
        let rec = |r: [u32; 8]| r.iter().flat_map(|w| w.to_le_bytes()).collect::<Vec<u8>>();
        let mut bytes = rec([0x544E_4F52, 2, 0x0100_0000, 0, 0, 48_233, 0, 0]);
        bytes.extend(rec([0, 0x0120_0000, 0, 0, 0, 0, 0, u32::MAX]));
        bytes.extend(rec([2, 0, 0, 0, 0, 0, 0, 0]));
        let got = entered(&bytes).expect("a trace");
        assert_eq!(
            got.into_iter().collect::<Vec<_>>(),
            vec![0x0059_1ef0, 0x0060_0000]
        );
        assert!(entered(b"not a trace, not at all, no..........").is_none());
    }

    /// Item 3903 (`docs/AI.md` §389): route 1 to the `kill_group_order` pair
    /// (`docs/audit/2026-11-06-item-3902.md`) reaches `fight`'s first
    /// `find_new_target` (`5fe113`) only on a type that packs
    /// (`unit_flags2 & 4`) and is not `is_siege` (`unit_flags & 0x20000`),
    /// and reaches `Group::action_attack`'s capture arm only if
    /// `find_nearby_target` hands it a capture-eligible city, which
    /// `Object::valid_target@00648ba0` refuses to a type without the MISSILE
    /// mask (`obj_masks & 1 << 27`) unless it carries VEHICLE and WAR_MACHINE
    /// under a mandatory `ATTACK` on that city. Every armed type of the first
    /// set is the Machine Gun line, MG42 with it, and none carries either.
    #[test]
    fn route_one_s_cast_is_refused_a_capture_eligible_city() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        const MISSILE: u32 = 1 << 27;
        const VEHICLE_AND_WAR_MACHINE: u32 = (1 << 21) | (1 << 19);
        let cast: Vec<usize> = (0..loaded.unit_types.len())
            .filter(|&t| {
                let c = &loaded.unit_types[t].cols;
                c.unit_flags2 & 4 != 0
                    && c.unit_flags & 0x20000 == 0
                    && c.is(sim::ai_load::role::MILITARY)
            })
            .collect();
        // Machine Gun ×2, Heavy ×2, Advanced ×3 and the German MG42; each
        // carries `FGT` (0x80060), WAR_MACHINE without VEHICLE.
        assert_eq!(cast.len(), 8, "the Machine Gun line's eight records");
        for &t in &cast {
            let u = &loaded.unit_types[t];
            let name = &loaded.unit_names[t];
            assert!(u.rel.machinegun, "{name} packs, is armed and is not siege");
            assert_eq!(u.rel.masks & MISSILE, 0, "{name}");
            assert_ne!(
                u.rel.masks & VEHICLE_AND_WAR_MACHINE,
                VEHICLE_AND_WAR_MACHINE,
                "{name}"
            );
        }
    }
}
