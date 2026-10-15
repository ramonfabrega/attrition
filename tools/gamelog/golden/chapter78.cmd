# Golden record, chapter seventy-eight — a helicopter's attack distance held
# on a human (item 2627, `docs/GOLDEN.md` §97;
# `ObjectData::attack_dist@006488f0`, vslot `+0xc0` = `UnitData::is_plane`).
#
# THE STAGING (walked on run871's start, `RON_STAGE=ch76:chapter78:1000`, `RON_STAGE_ALL=1`):
# a Barracks `1/2006` at (190, 82), chapter forty-one's, kept so
# the target is `1/2007` — an `@attack` at `1/2006` was dropped, walked twice
# — a Barracks `1/2007` at (212, 104), a legal point (chapter forty-one's
# Airbase stood there), and a human Helicopter `0/6` at (190, 66)
# ordered onto it. `add` takes no age. The Helicopter closes until its
# edge-to-edge distance is within range; the centre-to-centre arm would stop
# it nearer. (A Biplane control was walked and dropped: a plane with no base
# burns fuel and wanders off its order.)
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch78 \
#       --map 14 --end-frame 1100 --log-window 600 1100 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=2,AMMO=5,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter78.cmd
#
0 !ai off
606 add barracks who=1 190,82
608 add barracks who=1 212,104
610 add helicopter who=0 190,66
612 add biplane who=0 212,88
630 @attack 0 2007 1 6
630 @attack 0 2006 1 7
