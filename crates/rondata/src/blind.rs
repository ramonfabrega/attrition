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
#[rustfmt::skip]
pub const NEVER: &[u32] = &[
    0x0043_ec90, 0x0043_ecb0, 0x0046_cef0, 0x0046_ed60, 0x0046_ed70, 0x0046_ee80, 0x0046_f370, 0x0047_03f0, 0x0047_0430,
    0x0047_0570, 0x0047_0680, 0x0047_0760, 0x0047_07a0, 0x0047_0800, 0x0047_0e50, 0x0047_0f30, 0x0047_1170, 0x0047_22f0, 0x0047_2370,
    0x0047_2420, 0x0047_2a40, 0x0047_2a50, 0x0047_2cb0, 0x0047_6aa0, 0x0047_70d0, 0x0047_80c0, 0x0047_df90, 0x0048_43e0, 0x0048_49a0, 0x0048_5140,
    0x0048_5a40, 0x0048_5a60, 0x0048_6c60, 0x0048_89a0, 0x004a_1ca0, 0x0054_cea0, 0x0054_cf90, 0x0054_d100, 0x0058_3380, 0x0058_3390, 0x0058_33a0,
    0x0058_33b0, 0x0058_33c0, 0x0058_33d0, 0x0058_3400, 0x0058_3410, 0x0058_3420, 0x0058_3450, 0x0058_3470, 0x0058_3480, 0x0058_3490, 0x0058_34a0,
    0x0058_60c0, 0x0058_6440, 0x0058_7060, 0x0059_30c0, 0x0059_3f70, 0x0059_4480, 0x0059_4630, 0x0059_4640, 0x0059_4650, 0x0059_4660,
    0x0059_46c0, 0x0059_47e0, 0x005a_ac70, 0x005a_bc70, 0x005d_8160, 0x005d_86a0, 0x005d_86b0, 0x005d_8fb0, 0x005d_8fc0, 0x005e_2bb0, 0x005e_3310,
    0x005e_4c80, 0x005e_4d10, 0x005e_5be0, 0x005e_75a0, 0x005e_8670, 0x005e_d040, 0x005e_d1f0, 0x005f_2480, 0x005f_4720, 0x005f_4730, 0x0060_4550,
    0x0060_8850, 0x0060_9a20, 0x0060_a600, 0x0060_a6e0, 0x0060_a800, 0x0060_a810, 0x0060_fde0, 0x0061_d740, 0x0061_daf0, 0x0061_db10, 0x0061_dc50,
    0x0061_dc60, 0x0062_2ce0, 0x0062_3310, 0x0062_d430, 0x0062_d5b0, 0x0062_e2f0, 0x0063_0590, 0x0063_06b0, 0x0063_3390, 0x0063_89b0, 0x0063_9830,
    0x0063_e390, 0x0063_fa30, 0x0064_2bb0, 0x0064_2d90, 0x0064_40c0, 0x0064_50b0, 0x0064_50c0, 0x0064_52f0, 0x0064_6c00, 0x0064_6c10, 0x0064_6c30,
    0x0064_7150, 0x0064_86a0, 0x0064_e4a0, 0x0065_8190, 0x0065_b850, 0x0065_cfd0, 0x0066_1b80, 0x0066_1d70, 0x0066_2670, 0x0066_3b60, 0x0066_3b70,
    0x0066_3b90, 0x0066_9100, 0x0066_9110, 0x0066_9120, 0x0066_9130, 0x0066_d810, 0x0066_d830, 0x0066_d850, 0x0066_e5d0, 0x0066_fd40, 0x0066_fd50,
    0x0067_14c0, 0x0067_14f0, 0x0067_1600, 0x0067_3e80, 0x0067_7150, 0x0067_7590, 0x0067_7df0, 0x0067_d370, 0x0067_e600, 0x0067_e700,
    0x0068_1b20, 0x0068_1c50, 0x0068_3730, 0x0068_8310, 0x0068_9cb0, 0x0068_a1b0, 0x0068_d1a0, 0x0069_5050, 0x006b_1b80, 0x006b_1ba0, 0x006b_22e0,
    0x006b_2390, 0x006b_2510, 0x006b_3020, 0x006b_3960, 0x006b_4230, 0x006b_4290, 0x006b_46b0, 0x006b_4880, 0x006b_5590, 0x006b_76a0,
    0x006b_88b0, 0x006d_5230, 0x006d_6660, 0x006d_6740, 0x006d_6e80, 0x006d_a740,
    0x006e_c170, 0x006e_f2d0, 0x006e_fb20, 0x006f_0230, 0x0070_0010, 0x0070_20c0, 0x0070_f8a0, 0x0071_37f0, 0x0071_3bb0,
    0x0072_ff60, 0x0073_6470, 0x0073_69c0, 0x0073_8180, 0x0073_9ab0, 0x0073_9e20, 0x0073_b540, 0x0073_b5d0, 0x0073_b880, 0x0073_bb20, 0x0073_c1b0,
    0x0073_c5d0, 0x0073_c5e0, 0x0073_c780, 0x0073_e000, 0x0073_ebc0, 0x0073_f650, 0x0074_0030, 0x0074_0060, 0x0074_1a40, 0x0074_1d50,
    0x0078_be90, 0x0079_6140, 0x0079_7c40, 0x0079_ea80, 0x0081_2d20, 0x0081_34d0, 0x0084_2050, 0x0084_b8d0, 0x0089_7a50, 0x0089_7a60, 0x0089_7bc0,
    0x0089_7e20, 0x0089_be90, 0x0089_bea0, 0x0089_beb0, 0x0089_d3a0, 0x008a_4040, 0x008a_4060, 0x008a_60c0, 0x008b_0060, 0x008b_13b0, 0x008b_2a80,
    0x008c_6db0, 0x008d_1830, 0x008d_9190, 0x008d_92d0, 0x0092_5cb0, 0x0092_8f30, 0x0092_d0b0, 0x0092_fc50, 0x0093_a140, 0x0093_ee70, 0x0094_3f30,
    0x0094_8cb0, 0x0094_8e00, 0x0094_95c0, 0x0094_9ae0, 0x0094_c1c0, 0x0095_4c60, 0x0095_6470, 0x0095_6ac0, 0x0099_6ac0, 0x0099_bc20,
    0x009a_adc0, 0x009b_32f0, 0x009b_3320, 0x009b_4820, 0x009b_4930, 0x009b_5ec0, 0x009b_8ac0, 0x009b_efe0, 0x009e_18b0, 0x009e_5320, 0x009f_45e0, 0x009f_99e0,
    0x009f_9ad0, 0x009f_b790, 0x009f_b880, 0x009f_b970, 0x009f_b9e0, 0x009f_bab0, 0x009f_bb80, 0x009f_bd60, 0x009f_f5e0, 0x009f_f620, 0x009f_f860,
    0x009f_f8e0, 0x009f_fa10, 0x009f_fbf0, 0x00a0_0220, 0x00a4_69f0,
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
        0x0047_0e50,
        "Build::finished (the thunk to Build::finished@00628490): a vtable slot, \
         +0x1ac of Build, BuildOut, BuildData and Wall, that no instruction \
         dispatches; the image's one call or jmp through +0x1ac is a Window thunk",
    ),
    (
        0x0047_df90,
        "LeaderData::has_bonus (the 0047df90 copy): the tooltip's \
         byte-identical copy of the Nubian market gate, called only from \
         Options::describe; the sim's copy, has_bonus@006e0b80, stays open \
         (docs/NATIONS.md §9.1, §9.3; the forty-eighth pass)",
    ),
    (
        0x0048_5140,
        "GroupAttackOrder::log_data: reached only through its vtordisp thunk's \
         jmp, and nothing asks get_obj(GROUP_ATTACK), so no order list holds \
         one to print (docs/GOLDEN.md §144.4; the forty-eighth pass)",
    ),
    (
        0x0048_89a0,
        "MoveOrder::is_fleeing: reached only through vslot +0x28 of the move \
         orders' vtables (its two vtordisp thunks), and no instruction \
         dispatches that slot on an order (docs/ORDERS.md §1.3, two readings); \
         the live predicate is UnitData::is_fleeing@0046efa0 (item 1182)",
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
        0x0058_7060,
        "Game::run_gamespy: the GameSpy online lobby, run_game_loop's \
         GAME_GAMESPY_* arms, cut from v1 (docs/GOLDEN.md §144.4; the \
         forty-eighth pass)",
    ),
    (
        0x0059_30c0,
        "Game::action_cheat_ai_toggle: no reference in the executable; \
         CommandPackage::process_cheat_ai_toggle flips ai_off inline",
    ),
    (
        0x0059_46c0,
        "Game::is_no_rush: no reference in the executable (docs/AI.md §218, \
         §296; report.py refs reads it dead; the forty-eighth pass)",
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
        0x005d_8fc0,
        "Guy::set_pivot_angle: no call and no absolute reference in the \
         executable; Unit::move_step's cavalry-archer arm carries it inline \
         (item 1117, docs/COMBAT.md §85.2)",
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
        0x005e_8670,
        "Unit::do_form_change: do_job's CHANGE_FORM arm, run only on a \
         FormOrder, which no site in the image makes — none of get_obj's 47 \
         calls pushes 0x12 (docs/GROUPS.md §38; six register-typed calls read \
         by prefix, parked 3413c; the forty-eighth pass)",
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
         forty-ninth pass; run1020's supply wagon never entered it): its one hit \
         is the RVA-shaped rel32 of `call Mountains::get_range` at 006a841b, \
         inside TerrainGroups::place_all; the live copy is Unit::think@005f6e40's \
         tail, which reads is_supply (vslot +0xcc, unit_flags & 0x40) and calls \
         add_to_army itself",
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
        0x0060_a600,
        "UnitData::turn_speed: no reference in the executable; the live copy \
         is GuyData::turn_speed",
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
        0x0063_0590,
        "BuildData::max_gatherers: no reference in the executable",
    ),
    (
        0x0063_3390,
        "BuildType::set_domain: no reference in the executable",
    ),
    (
        0x0063_e390,
        "Wall::Wall: its only references are the per-kind constructor tables \
         (ObjectsArray::get_new_data, Objects::find_free, Objects::init); \
         num_def_walls is only ever written 0 and no find_free call asks for \
         the wall range (docs/GOLDEN.md §144.4; the forty-eighth pass)",
    ),
    (
        0x0064_e4a0,
        "ObjectData::is_in_range: entered, and no instrument sees it — the \
         tracer cannot hook it (rontrace.funcs.excluded.txt, a branch target \
         inside the displaced range); Unit::do_air_attack_ground+0x92 calls it \
         on every missile step past its first three tests, and run372's \
         strike on 2701 fires only past its answer (item 1182)",
    ),
    (
        0x0065_cfd0,
        "ObjectsData::find_dock: no reference in the executable",
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
        0x0069_5050,
        "MapGrass::make_continents: map style 23 (BLANK_MAP), which only \
         ScenarioEditor::generate_map asks Map::new_map for; the lobby's styles stop at 22",
    ),
    (
        0x006b_22e0,
        "World::clear_danger: no reference in the executable",
    ),
    (
        0x006b_4230,
        "World::set_behind: no reference in the executable; its writers \
         (Wall::mark_behind_tiles, Mountains::add_mountain) carry it inline",
    ),
    (
        0x006b_46b0,
        "World::set_gathered_at: no reference in the executable",
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
        "LeaderData::get_handicap: all four callers gate on game semaphore bit 2, \
         the multiplayer flag; the capture lanes are solo",
    ),
    (
        0x006e_c170,
        "LeaderData::is_human: no reference in the executable (inlined at every use)",
    ),
    (0x006f_0230, "Tribe::init: no reference in the executable"),
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
        0x0071_37f0,
        "Group::leader_report_speed: no reference in the executable; its three \
         statements stand inline in Unit::do_group_move",
    ),
    (
        0x0071_3bb0,
        "Group::report_speed: no reference in the executable",
    ),
    (
        0x0073_c5e0,
        "Wonder::init: no reference in the executable (docs/TECH.md, what is \
         not established: its body is inlined in Wonders::init_wonder and the \
         byte before it is Wonder::close's ret; the forty-eighth pass)",
    ),
    (
        0x0073_e000,
        "Caravan::process: no reference in the executable (its address's one \
         copy is a coincidence inside .reloc's fixup entries); its body, \
         `if (making_road) build_road()`, stands inline in Unit::work",
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
        0x0079_7c40,
        "ConquestGame::build_conquest_game: Conquer the World's start, cut from \
         v1 (docs/NATIONS.md §9.1, §9.3; the forty-eighth pass)",
    ),
    (
        0x0079_ea80,
        "ConquestLeader::plan_strategy: the campaign map's turn, from \
         ConquestWin::post_turn_processing, cut from v1 (docs/NATIONS.md §9.1, \
         §9.3; the forty-eighth pass)",
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
        0x0092_fc50,
        "GameLog::dump_armies: no reference in the executable, and gamelog.ini \
         has no ARMY key",
    ),
    (
        0x0093_ee70,
        "CommandManager::set_mp_playback: multiplayer replay playback, a prefs2 \
         path read under netsys before ReplayWin opens; no skirmish capture \
         reaches it (docs/GOLDEN.md §144.4; the forty-eighth pass)",
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
        0x0094_c1c0,
        "CommandPackage::clear: no reference in the executable",
    ),
    (
        0x0099_6ac0,
        "ScenarioData::issue_order: a scenario file's initial orders, applied \
         once at ScenarioRead::import, not the trigger system; no capture loads \
         a scenario (docs/GOLDEN.md §144.2, §144.4; the forty-eighth pass)",
    ),
    (
        0x009b_8ac0,
        "UnitBalance::next: its three callers (Game::check_victory, \
         Setup::build_empire, TurnControl::toggle_pause) reach it only under \
         game semaphore byte 1's bit 2, which Game::run_unit_balance alone \
         sets — the GAME_UNIT_BALANCE shell mode, not a match (item 1182)",
    ),
    (
        0x009e_5320,
        "ScenarioFuncSet::get_difficulty: a script getter no shipped skirmish \
         script calls; only game/conquest/* and ctw_lib.bhs do (docs/NATIONS.md \
         §9.1, §9.3; the forty-eighth pass)",
    ),
    (
        0x009f_45e0,
        "ScenarioFuncSet::toggle_infinite_queue: a script function with one \
         caller in the install, scenario/Custom/capture/capture.bhs; scenarios \
         are cut from v1 (docs/GOLDEN.md §144.4; the forty-eighth pass)",
    ),
    (
        0x009f_99e0,
        "ScenarioFuncSet::repeat_orders_enable: a script function no script in \
         the install calls (docs/GOLDEN.md §144.4; the forty-eighth pass)",
    ),
    (
        0x009f_9ad0,
        "ScenarioFuncSet::repeat_orders_disable: a script function no script in \
         the install calls (docs/GOLDEN.md §144.4; the forty-eighth pass)",
    ),
    (
        0x009f_b790,
        "ScenarioFuncSet::buy_good: a script function no shipped skirmish \
         script calls (docs/NATIONS.md §9.1, §9.3; the forty-eighth pass)",
    ),
    (
        0x009f_b880,
        "ScenarioFuncSet::sell_good: a script function no shipped skirmish \
         script calls (docs/NATIONS.md §9.1, §9.3; the forty-eighth pass)",
    ),
    (
        0x009f_b970,
        "ScenarioFuncSet::can_buy_sell: a script function no shipped skirmish \
         script calls (docs/NATIONS.md §9.1, §9.3; the forty-eighth pass)",
    ),
    (
        0x009f_b9e0,
        "ScenarioFuncSet::find_sell_price: a script function no shipped \
         skirmish script calls; its one caller in the install is \
         game/conquest/Alexander/sogdiana.bhs (docs/NATIONS.md §9.1, §9.3; the \
         forty-eighth pass)",
    ),
    (
        0x009f_bab0,
        "ScenarioFuncSet::find_buy_price: a script function no shipped skirmish \
         script calls (docs/NATIONS.md §9.1, §9.3; the forty-eighth pass)",
    ),
    (
        0x009f_bb80,
        "ScenarioFuncSet::set_base_rate: a script function called only by \
         game/conquest/* and four scenario/* scripts, never an AI script \
         (docs/ECONOMY.md, base_rate; the forty-eighth pass)",
    ),
    (
        0x009f_f5e0,
        "ScenarioFuncSet::enable_production_ai: a scenario switch reached only \
         through the script host's table and called only by scenario scripts, \
         cut from v1 (docs/AI.md §1, §296; docs/CENSUS.md row 10; the \
         forty-eighth pass)",
    ),
    (
        0x009f_f620,
        "ScenarioFuncSet::set_auto_peasant_level: reached only through the \
         script host's table, and only the three tutorials call it; scenarios \
         are cut from v1 (docs/ORDERS.md §42.1, §42.4; the forty-eighth pass)",
    ),
    (
        0x009f_f860,
        "ScenarioFuncSet::disable_combat_ai: a scenario switch reached only \
         through the script host's table and called only by scenario scripts, \
         cut from v1 (docs/AI.md §1, §296; the forty-eighth pass)",
    ),
    (
        0x009f_f8e0,
        "ScenarioFuncSet::disable_all_unit_ai: a scenario switch reached only \
         through the script host's table and called only by scenario scripts, \
         cut from v1 (docs/AI.md §1, §296; the forty-eighth pass)",
    ),
    (
        0x009f_fa10,
        "ScenarioFuncSet::disable_unit_ai: a scenario switch reached only \
         through the script host's table and called only by scenario scripts, \
         cut from v1 (docs/AI.md §1, §296; the forty-eighth pass)",
    ),
    (
        0x009f_fbf0,
        "ScenarioFuncSet::disable_city_ai: a scenario switch reached only \
         through the script host's table and called only by scenario scripts, \
         cut from v1 (docs/AI.md §1, §296; the forty-eighth pass)",
    ),
    (
        0x00a0_0220,
        "ScenarioFuncSet::set_capital: a script setter no shipped skirmish \
         script calls; only seventeen game/conquest/* scripts do \
         (docs/NATIONS.md §9.1, §9.3; the forty-eighth pass)",
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
}
