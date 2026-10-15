# Golden record, chapter seventy-nine — a plane's attack distance held on a
# human (item 2649, `docs/GOLDEN.md` §98;
# `ObjectData::attack_dist@006488f0`, vslot `+0xc0` = `UnitData::is_plane`,
# the centre-to-centre arm). Chapter seventy-eight held the helicopter's
# edge-to-edge arm; its Biplane control parted (parked 2630) because an
# `add`ed plane is outside a base: it flies in the original and sits in ours.
# Chapter forty-one's recipe puts it home first (`@flight`), and a human's
# plane leaves a base by `@launchpatrol` (no sortie for a human, `do_launch`).
#
# THE STAGING (walked on run871's start, `RON_STAGE=ch76:chapter79:1100`):
# an Airbase `0/2007` (212, 104), a Biplane `0/6` outside it, a Barracks
# `1/2006` (190, 82), and a human Citizen `0/7` at (190, 86) for sight
# (chapter seventy-eight: a target nothing of the player's sees drops an
# order). The Biplane flies home on 620, and is launched on patrol over the
# Barracks on 830 (the tank empties on 808; a plane refuelling is passed over).
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch79 \
#       --map 14 --end-frame 1500 --log-window 600 1500 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=2,AMMO=5,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter79.cmd
#
0 !ai off
606 add airbase who=0 212,104
608 add biplane who=0 212,118
610 add barracks who=1 190,82
612 add citizen who=0 190,86
620 @flight 0 2007 0 6
830 @launchpatrol 0 36480 15744 2007
