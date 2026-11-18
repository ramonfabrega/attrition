# The blind list's input issuers, entered by one capture (item 4378,
# docs/CLIENT_INPUT.md §13.1, §22.1, §23.1, §26.2, §27.1, §29.1). A coverage
# staging, not a chapter: no word is pinned on it.
#
# Chapter thirteen's lobby and staging (`--map 14`, the Barracks `0/2007`,
# the Chariot `0/6` and the Hoplites `0/7` garrisoned in it), then each open
# issuer by the game's own path where one is reachable without a screen
# point, and by the DLL's `@` line where none is:
#
#   720  ctrl and 1 over the selected City `0/2000` (`@hold 17` is
#        GetKeyState's ctrl, which `OnKey` ORs in as 0x20000):
#        `Console::on_key_down@007cc210` → `GroupOut::issue_hotkey@00708800`
#        → `CommandManager::issue_hotkey@00941fd0`.
#   740  F14 over `0/7`, bound to KEYMAP_STANCE_AGGRESSIVE (177;
#        `keymap_options[177]` is (17, 0), OPTION_STANCE):
#        `Options::exec` case 0x11 → `Options::do_stance@00721480`.
#   760  `@gstance 0 1 7`: `GroupOut::issue_stance@0070b100`, which nothing
#        in the executable calls (Loop 3473).
#   770  `@pause 0`: `CommandManager::issue_pause@00940ba0`, which nothing in
#        the export calls; state 0 on an unpaused game, so
#        `TurnControl::process_pause` only logs it.
#   800  F15, bound to KEYMAP_OPTION_PAUSE (89 → OPTION_PAUSE 153), pressed
#        and pressed again three seconds later on the loop's clock:
#        `Options::exec` → `TurnControl::issue_toggle_pause@00956b40`, twice.
#   830  `@gdisband 0 0 7`: `GroupOut::issue_disband@0070a470`, whose one
#        caller is the selected panel's one-object arm, a button with no point
#        here.
#   850  `!eval unit_eject_order(1, 7)`: the console's eval reaches
#        `ScenarioFuncSet::unit_eject_order@009f84e0`, whose container test
#        is `Options::exec` case 0xaa's, and it calls
#        `UnitOut::issue_come_out@00607be0` on `0/7`. run1671 had it on 700,
#        where `0/7` was still walking to the Barracks (`inside_up` -1 to
#        760, 6 from 800): entered, and it issued nothing.
#   900  chapter thirteen's eject; the Chariot `0/6` out.
#   920  F13 over `0/6`, bound to KEYMAP_OPTION_GARRISON (149 →
#        OPTION_GARRISON 26): `Options::exec` case 0x1a →
#        `Options::do_garrison@0071d6b0`. Last, because its pick may leave
#        the options in a selecting mode.
#
# run1671b (cover=1) and run1672b (its cover=0 twin), on the click-free pool;
# `--ai-tribe 11` because run1672's lane drew tribe 2 for who 1:
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/run1671b \
#       --map 14 --ai-tribe 11 --end-frame 1000 --log-window 690 1000 --timeout 3600 \
#       --detail end:MISC,UNITS=3,BUILDS=7 --detail misc:COMMANDMANAGER=1 \
#       --bindkey OPTION_GARRISON=124 --bindkey STANCE_AGGRESSIVE=125 --bindkey OPTION_PAUSE=126 \
#       --cmd-file tools/gamelog/coverage/blind4378-issuers.cmd --cover cover=1
#
# WHAT WOULD FALSIFY IT, read before the run. (1) A function named above is
# not entered on or after its line's frame. (2) An `I_KEY` record's mapped
# action is not the bound one (177, 89, 149). (3) An `I_ISSUE` for the three
# DLL lines is refused (nonzero refusal). (4) `rngcmp.py` against the twin
# differs on any frame.

0 !ai off

606 add barracks who=0 14,74

610 add chariot who=0 20,60
614 add hoplite who=0 10,90

620 @garrison 0 2007 0 6
640 @garrison 0 2007 0 7

720 select 2000 who=0
722 @hold 17 726
724 @key 49

740 select 7 who=0
744 @key 125

760 @gstance 0 1 7
770 @pause 0

800 @key 126
800 @after 3000 key 126

830 @gdisband 0 0 7

850 !eval unit_eject_order(1, 7)

900 @eject 0 2007

920 select 6 who=0
924 @key 124
