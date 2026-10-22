# Item 3478: a goody box under the camera. The golden start (map 14, the Nubian human, tribe 4; computer tribe 11) at
# cover=1 on the click-free pool, `--no-ffwd` so every frame from 600 is drawn.
# The boxes, on disk: run1084's frame-1 `WORLD=6` block has 22 cells with `down -3` (an Item: `Objects::init_item`
# writes 0xfffd to the cell's `down` and the item's index to `down_who`), counted by the per-cell `was_seen` key,
# 60 wide, row-major. The nearest to the human: cell (1, 27) `down_who 1` and cell (8, 53) `down_who 5`. `camera`
# takes quarter cells (item 3472: `camera 186,190` framed tile (186, 190)), so a cell's centre is `4c + 2`.
# The filler (3476's rule): `ObjectsOut::draw_overlays@00655120` calls slot +0x5c (`ItemOut::draw_overlays@00677590`
# on an Item) on the render tree `objects +0x224`, and only `ObjectsOut::sort_all@00657320` inserts an object there,
# from a cell's `down`/`down_who` chain while `down >= 0`. A box's cell holds `down -3`, so the prediction is that
# `ItemOut::draw_overlays` is NOT entered; `ItemOut::draw` (slot +0x58, `pop_all_zoomed_in` over `sort_items`' tree
# `objects +0x2dc`) is the box's drawing. Both takes stage `--cursor 1400,1000` (off the map: Loop 3432).
# The twin is this file at cover=0: every input shared, so there is no `--from`.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/run1094 \
#       --map 14 --end-frame 640 --log-window 600 640 --timeout 3600 --no-ffwd \
#       --detail end:LEADERS=2,BUILDS=7,CITIES=5,UNITS=3 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/coverage/goody3478-start.cmd --cover cover=1 \
#       --cursor 1400,1000
0 !ai off
600 explore all
602 camera 6,110
630 camera 34,214
