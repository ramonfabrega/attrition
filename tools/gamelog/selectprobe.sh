#!/bin/zsh
# selectprobe.sh N TMPDIR — which way of selecting units actually selects them?
#
# `longtrace.sh`'s DRIVER hook, for a diagnostic rather than a capture.
#
# `docs/ORACLE.md` says "`select` is scriptable, so a group capture needs one
# human click and not twelve", and gives `160 select slinger who=0` /
# `170 select hoplite who=0 +` as putting twelve portraits in the tray. run46
# contradicts it: eight `add hoplite` lines ran and were accepted, 4,728
# `type 132` records prove the hoplites exist, `select hoplite who=0` at
# frame 180 returned 1 — and the screenshot at frame 212 shows the **capital**
# still selected and the right-click producing no `GroupMoveOrder` at all.
#
# `parse_cmd` returning 1 means the line was accepted, not that it matched
# anything, so "returned 1" was never evidence the selection took. This probe
# takes the four candidate ways in turn and photographs the tray after each,
# which is evidence:
#
#   1. `select hoplite who=0`   — ORACLE's form, the one run46 used
#   2. `select hoplite`         — no `who`, in case `who=0` is eaten as the ob#
#   3. `select 0 0`             — the help's `[ob#] [who]`, positionally
#   4. a mouse band-select      — a drag box over the units, no console at all
#
# The stanza supplies 1-3 as `cmd:` lines on frames this script's timings
# match; 4 is this script's own drag. Read the four shots afterwards: the
# bottom-centre panel names what is selected.
set -e
N=${1:-47}
T=${2:-/tmp/ron-runs}
L="$HOME/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"

shot() {
  screencapture -x "$T/r$N-$1.png" 2>/dev/null || true
  sips -Z 900 "$T/r$N-$1.png" --out "$T/r$N-${1}s.png" >/dev/null 2>&1 || true
  echo "$(date +%H:%M:%S) shot $1 at frame $(frame_now)"
}

frame_now() {
  tail -c 2000000 "$L/gamelog.txt" 2>/dev/null \
    | grep -a -o 'BEGIN FRAME [0-9]*' | tail -1 | awk '{print $3}'
}

wait_frame() {
  local want=$1 i f
  for i in {1..400}; do
    f=$(frame_now)
    if [ -n "$f" ] && [ "$f" -ge "$want" ]; then return 0; fi
    sleep 2
  done
}

focus() {
  osascript -e 'tell application "System Events" to set frontmost of process "riseofnations_trace.exe" to true' >/dev/null 2>&1
  sleep 0.4
}

wait_frame 185; shot "a-select-who"
wait_frame 205; shot "b-select-bare"
wait_frame 225; shot "c-select-obnum"

# 4. The band-select. The stanza's `camera 15,168` at frame 240 puts the
# hoplites at the viewport centre, so a box around the centre covers them.
wait_frame 245
focus
echo "$(date +%H:%M:%S) band-select drag"
cliclick m:700,330
sleep 0.3
cliclick dd:700,330
sleep 0.3
cliclick m:1220,560
sleep 0.3
cliclick du:1220,560
sleep 1.5
shot "d-bandselect"

# And a right-click with whatever that left selected, to see if an order
# comes out of it at all.
wait_frame 265
focus
cliclick m:960,445
sleep 0.4
cliclick rc:960,445
sleep 1.5
shot "e-afterclick"
echo "probe done"
