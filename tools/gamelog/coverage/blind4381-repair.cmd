# The blind list's last open row, entered by one capture (item 4381,
# docs/CLIENT_INPUT.md §32.13). A coverage staging, not a chapter: no word is
# pinned on it.
#
# `Options::do_repair@0071ccb0` is the standard panel's REPAIR button: its
# press, `Options::exec@007188c0` case 3. A playback presses nothing, and a
# `--cmd-file` beside a `--recording` is refused, so the press is a key over
# a live selection, as item 4378's stance and garrison were:
#
#   KEYMAP_OPTION_REPAIR is 392 (the PDB's KeyMap enum), and
#   `KeyMap::keymap_options[392]` (PE .rdata, the S_GDATA32 at 0002:141168,
#   two shorts a row) is (3, -1): OPTION_REPAIR with -1.
#   `Console::exec_keymap@007c2ce0` passes the row's second short through
#   `LeaderData::get_key_type`, which returns a negative one unchanged, to
#   `Options::exec`, whose case 3 is `do_repair`.
#
# Chapter thirteen's lobby (`--map 14`, as item 4378's), then:
#
#   606  a Barracks for slot 0, `0/2007` (4378's).
#   610  a Citizen for slot 0, `0/6` (slot 0 holds 0..5; run1671b).
#   640  `!eval damage_object(1, 2007, 40)`: the console's eval reaches
#        `ScenarioFuncSet::damage_object@009f5bd0` (player 1-based, the
#        object, a percent below 0x65 of its maximum hit points), the wound
#        a building of the human's has had on no Great Lakes walk (4361a).
#        An `@attack 1 ...` is no route: the DLL refuses a line whose `who`
#        is not the console's play. The damage is read off BUILDS.
#   720  the Citizen selected; 724 F13, bound to KEYMAP_OPTION_REPAIR (392):
#        `Console::exec_keymap` -> `Options::exec` case 3 ->
#        `Options::do_repair`.
#   760  again, the selection made and the key pressed, in case the first
#        is lost to a frame's input.
#   800  `@repair 0 2007 0 6`: the Citizen on the wounded Barracks, the
#        REPAIR swarm the pick's second arm issues, so the records show the
#        building's damage fall.
#
# run1675 (cover=1), on the click-free pool:
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/run1675 \
#       --map 14 --ai-tribe 11 --end-frame 1000 --log-window 690 1000 --timeout 3600 \
#       --detail end:MISC,UNITS=3,BUILDS=7 --detail misc:COMMANDMANAGER=1 \
#       --bindkey OPTION_REPAIR=124 \
#       --cmd-file tools/gamelog/coverage/blind4381-repair.cmd --cover cover=1
#
# WHAT WOULD FALSIFY IT, read before the run. (1) `Options::do_repair` is
# not entered on or after 724. (2) An `I_KEY` record's mapped action is not
# 392. (3) The Barracks' `damage` is 0 on 720 (the eval did not wound it;
# the entry still stands, the wound does not).

0 !ai off

606 add barracks who=0 14,74
610 add citizen who=0 18,70

640 !eval damage_object(1, 2007, 40)

720 select 6 who=0
724 @key 124

760 select 6 who=0
764 @key 124

800 @repair 0 2007 0 6
