# Golden record, chapter one hundred and thirty-two -- land-based air: a blow on an Airbase scrambles its Fighter,
# a home razed while its planes are out, and a Helicopter ordered into its own Airbase (item 3990, GOLDEN 183;
# adjudication 3 row 35, adjudication 4 rows 48 and 46). Chapter seventeen's lobby and lines (GOLDEN 25, run223:
# the Airbase `0/2007`, the Fighter `0/6` inside it from 722 with its tank full on 778, the Bombers `0/7`, `0/8`
# out on the Barracks `1/2006`), and five more.
#   Arm 1: `Object::attempt_launch@00643a10`, from `Object::do_damage@0064a480:194`-`:198` on every blow of a live,
#          non-missile attacker. A Hoplite squad of who=1's added beside `0/2007` on 780 strikes it by its own
#          opportunity; the base holds `0/6` with no order and a full tank, `launch_frames` 15 and `launching`
#          empty, so `0/6` takes `add_air_patrol_order` at the striker's point with the action bit, and the next
#          `do_launch` puts it out.
#   Arm 2: `Unit::check_fuel@005e9be0:60`-`:136`. A second Airbase `0/2008` on 786, 28 tiles south. `die 2007,0`
#          on 900 razes the first while all three planes are out; each, at its empty tank (`returning`), finds its
#          home gone and re-homes to the nearest building that `can_carry`s it, `0/2008`. `die 2008,0` on 1230,
#          before any of them lands: the search finds nothing, and each fixed-wing plane dies (`Unit::die(2)`).
#   Arm 3: `Unit::do_garrison@005e6b80:63`-`:83`. An Airbase `0/2007` on 1250 (the number `Objects::find_free`
#          hands out again) and a Helicopter `0/9` on 1254; `@garrison` on 1260 puts it in through
#          `Group::action_garrison`'s zero-limit arm (`is(AIRBASE)`), and `do_garrison`'s helicopter arm kills the
#          garrison order and adds a strafe with no target home there (`returning` 1): the helicopter flies in,
#          hovers at the point and comes down 25 a frame, and lands.
# The staging was walked on run223's start (`RON_STAGE=ch17:chapter132:1400`): the blow on 795, `0/6` out by 797;
# its tank empty and re-homed by 1229; all three planes dead on 1230; the Helicopter's strafe on 1261, inside on 1317.
# Killers, before the take (a cheat on trace frame F shows on block F+2):
#   (1) no `AIRPATROLORDER` on `0/6` while it is inside `0/2007`, between the first blow and its launch -- the
#       launch gate is not the reading's (or the hoplites never strike);
#   (2) `0/6` leaves `0/2007` with no order, or with anything but the patrol at a hoplite's point -- a different adder;
#   (3) any plane's `AIRORDER oxx` other than 2008 on the block after its tank reads empty (after 902) -- no re-home;
#   (4) a plane alive past 1234, or one dead before 1232 -- the empty search does not kill, or kills otherwise;
#   (5) `0/9` with a `GARRISONORDER` on 1262, or no `STRAFEORDER` on it with `returning 1` -- the helicopter arm missed;
#   (6) `0/9` not inside `0/2007` by 1399.
#   run1548 (cover=0), chapter seventeen's lobby:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch132 \
#       --map 14 --end-frame 1400 --log-window 605 1400 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter132.cmd
0 !ai off
600 library who=0 6
606 add airbase who=0 60,72
610 add fighter who=0 60,84
612 add bomber who=0 52,84
614 add bomber who=0 68,84
616 add barracks who=1 110,86
620 @strike 0 2006 1 7 8
640 @flight 0 2007 0 6
660 @flight 0 2007 0 7 8
664 @strike 0 2006 1 7 8
780 add hoplite who=1 64,76
786 add airbase who=0 60,100
900 die 2007,0
1230 die 2008,0
1250 add airbase who=0 40,100
1254 add helicopter who=0 44,94
1260 @garrison 0 2007 0 9
