# Golden record, chapter sixty-five — the city alarm's heavy arms (item
# 2235, `docs/GOLDEN.md` §81; parked 2110).
#
# `Object::do_damage@0064a480`'s building branch (`0064a9f8`..`0064b2c4`)
# stamps the struck building's city. A light hit writes `raid_stamp`; a heavy
# one, on a city that is not the capital, writes `attack_stamp` and
# `raid_stamp`; a heavy one on the capital writes neither (the leader's
# `capital_stamp[attacker]` instead). Heavy is a siege attacker
# (`UnitTypeData::is_siege`, `unit_flags & 0x20000`) or `damage + dealt >=`
# the ceiling: `hits / 10` for a fort or tower (`ObjectTypeData::is_defense`,
# type vslot `+0x104`) or a city, else `hits / 4`.
#
# What the disk already holds (run437, chapter forty-one): the light arm, the
# worker arm (a struck citizen gathering at a city building raids it), and
# the capital's heavy arm at its boundary — `1/2002` at 95 + 5 = 100 = 400/4
# on 946, after which the original writes no `raid_stamp`. What no capture
# holds: `attack_stamp` written by anything but a transfer. No city but a
# capital is ever struck on disk, so the non-capital heavy arm, the siege
# arm and the `/10` ceiling are this staging's three:
#
# 1. A non-capital city: `add small_city` founds one (`init_build` +
#    `Build::activate`), and the Tower and the Barracks join it.
# 2. The `/10` ceiling: the Hoplites `0/6` on the Tower (750 hits, ceiling
#    75). Light — `raid_stamp` alone — until damage + dealt reaches 75; then
#    `attack_stamp` with it. Under `/4` (187) it would stay light past 1100.
# 3. The siege arm: the Catapult `0/9` engages the Small City `1/2006` on its
#    own (this crate's walk: 834, 70 a round). Without the siege arm that hit
#    is light (70 < 1200/10), and `attack_stamp` stays 0 until the Hoplites'.
#
# THE STAGING, walked by this crate on run437's start (RON_STAGE=ch41:
# chapter65:1100, RON_STAGE_ALL): who=1's Small City `1/2006` on tile
# (196, 116), its Tower `1/2007` on (200, 113) and Barracks `1/2008` on
# (190, 121); who=0's Hoplites `0/6`-`0/8` on (206, 113) and Catapult `0/9`
# on (180, 128). 620: the Hoplites attack the Tower. A call on trace frame F
# is on block F+2.
#
# CENSUS ROWS the arm reaches, never backed (`census.py --unbacked`):
# `Armies::emergency@006f3250` is the alarm's consumer, and a computer under
# `!ai off` holds no army, so this staging does not reach it; the worker arm's
# `UnitData::get_action@00608450` is run437's.
#
# WHAT WOULD FALSIFY IT, read before the run. (1) `1/2006` is not a city of
# its own, or the Tower and Barracks are not its members (`city` != the new
# record): the cheat's activation does not found. (2) `1/2006`'s
# `attack_stamp` is 0 on the Catapult's first round: no siege arm. (3) The
# first Hoplite hit at damage + dealt >= 75 leaves `attack_stamp` behind
# `raid_stamp`: the ceiling is not `/10` for a tower. (4) Any `attack_stamp`
# on `1/2000`: the capital arm writes the city.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch65 \
#       --map 14 --end-frame 1100 --log-window 605 1100 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=2,AMMO=5,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter65.cmd
0 !ai off

606 add small_city who=1 196,116
608 add tower who=1 200,113
610 add barracks who=1 190,121
612 add hoplite who=0 206,113
614 add catapult who=0 180,128

620 @attack 0 2007 1 6
