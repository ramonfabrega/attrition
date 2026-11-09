# Golden record, chapter one hundred and thirty-six -- the boats: an idle boat's walk to a coastal Dock city, and a
# soldier barge whose group has no army (item 3995, GOLDEN 188; territory adjudication 4 rows 49, 50; parked 3411a, 3411b).
# The golden start, map 14, with the AI off: who=1's units keep `unit_masks & 0x40000`, so `Unit::think@005f6e40`'s
# tail runs for them and `Unit::think_carry@005f2ee0` is asked for every who=1 hull with a CARRY.
# No water lies inside any city's CITY_RADIUS on this start (the walk's map: who=1's capital reaches x 200, the east
# lake ends at x 187), so a Dock joins no city there; a second who=1 city is added on the lake's east shore.
#   Arm 1: `Unit::think_carry_idle@005f2f30` -- an empty Transport Freighter `1/6` (TI 322, sea) born on the east lake
#          (610) thinks on its first idle frame: who=1's live cities, the open, coastal (`Region::is_coast(my, city
#          region)`) ones with a DOCK, the nearest; `UnitType::find_nearby_spot(type, city x, y, 0xf00, -1, 0,
#          0x55555555, FILTER_NOT_ME, o, who, 0, ...)`; a MOVE_TO to the spot, 48-snapped. The new city `1/2006` (602)
#          and its Dock `1/2007` (604) are the one candidate. The walk (this crate, wired): MOVEORDER to (36168, 19560)
#          from 610.
#   Arm 2: `Unit::think_carry_military@005f3260:29`-`:42` -- a Hoplites squad born on the lake by the console's
#          `create_unit` (640, chapter one hundred and fourteen's line for who=1: `action_transport(1)` builds a barge at
#          the captain's point and the squad goes inside, in a group of its own). Idle with a captain that attacks
#          aboard, the barge takes the military arm; the group (`+0x80` of the head rider) has no army under the AI off,
#          so `eject_contents(0, -1, 0, 1)`, and an emptied boat dies. This crate answers false there (a SEAM): the
#          barge sits idle with the squad aboard.
#   Not staged (parked by the item): the army whose target dies in transit (no army forms with the AI off), and the
#          razed trade city (no verb razes a city).
# Killers, before the take:
#   (1) no city `1/2006` or no Dock `1/2007` in who=1's city -- the `add` of a city type founds none in the original;
#   (2) `1/6` holds no MOVEORDER by block 612 -- the arm refused (the spot test or the dock test), or the think did
#       not reach it;
#   (3) its MOVEORDER is not (36168, 19560) -- the spot search differs from `find_nearby_spot_type_for`;
#   (4) the barge stands with the squad aboard on block 700 -- the no-army arm does not eject in theirs.
#   run1553 (cover=0):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch136 \
#       --map 14 --end-frame 900 --log-window 600 900 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=2,GROUPS=1,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter136.cmd
0 !ai off
600 library who=1 1
602 add small_city who=1 198,84
604 add dock who=1 188,84
610 add transport_freighter who=1 181,82
640 !eval create_unit(2, 181, 88, "Hoplites", 1)
