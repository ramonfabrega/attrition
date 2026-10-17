# Golden record, chapter eighty, run878's window 998–1002 (item 2785, GOLDEN §103) — chapter80.cmd without its 1197 Scout:
# the 1000 pass's own blocks. Capture: --end-frame 1002 --log-window 998 1002 (the details as chapter80.cmd, its `end:` set
# landing on the window only when the window reaches the end frame).
# Golden record, chapter eighty — the stealth arm of `UnitData::is_seen`
# on a human's units under a computer's `calc_danger` (item 2755,
# `docs/GOLDEN.md` §102; `is_seen@00607a60`, `is_detected@0060a630`,
# `World::seen3`, built by item 2725, §101, and held by no walk until now).
#
# THE STAGING (walked on run871's start, `RON_STAGE=ch76:chapter80:1204`):
# an Airbase `1/2006` (190, 60) — los 12, no weapon, so it lights ~14 tiles
# and draws no attack from a Partisan whose own sight is 10 — and, 14 tiles
# off it, an idle Partisan `0/6` (`FLAGS s`, attack 240) south, a Partisan
# `0/7` east that a who=1 Scout (`DETECT`) added on 1197 reaches, and three
# Hoplites west as the plain control. `calc_danger` runs on every 200th
# frame: the 1000 pass weighs both Partisans half, the 1200 pass `0/7` whole.
# Nearer or weaker lights failed in this crate: a Barracks or a Citizen draws
# the Partisan's attack, a Castle shoots it, `add town` places nothing.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch80 \
#       --map 14 --end-frame 1204 --log-window 1198 1204 --timeout 3600 \
#       --detail end:WORLD=6,UNITS=3,BUILDS=7,CITIES=5,LEADERS=2,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter80.cmd
#
0 !ai off
606 add airbase who=1 190,60
608 add partisan who=0 190,74
610 add partisan who=0 204,60
612 add hoplite who=0 183,64
