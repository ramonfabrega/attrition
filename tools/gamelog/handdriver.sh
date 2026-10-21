#!/bin/zsh
# handdriver.sh N TMPDIR — a driver whose clicks are written while the game runs.
#
# `longtrace.sh`'s `DRIVER` hook, for a capture whose clicks cannot be fixed
# before launch (item 3427, the click layer): a command card's buttons sit
# where a screenshot says they sit, and a pick on a unit lands where the unit
# is drawn, so the clicks are aimed from the run's own screenshots. Every
# earlier driver (`clickdriver.sh`, `echelon.sh`) is blind by design; this
# one is told.
#
# It runs under RonDriver with the capture (`viadriver.sh` → `runqueue.sh` →
# `longtrace.sh`), so `cliclick`, `screencapture` and `osascript` carry the
# app's grants, and a caller outside that tree only appends lines to a file:
#
#   $HAND_DIR/cmd       one command a line, appended by anyone; read in order
#   $HAND_DIR/done.log  each line as run: wall clock, the dump's last frame,
#                       the line, its exit
#   $HAND_DIR/*.png     what `shot` took, and a 1400-wide copy `*s.png`
#
# HAND_DIR defaults to `$TMPDIR/hand-$N`. Lines written before the launch
# (the first, timed click) run as soon as the game is up, in order:
#
#   wait F          block until the dump has written `BEGIN FRAME F`
#   focus           bring the game's window to the front (`focus.sh`)
#   m X Y           move the pointer, with a jiggle (a move to the point the
#                   pointer is already on emits no event: `cheat.sh`)
#   c X Y           left click        rc X Y   right click
#   sc X Y          shift + left click
#   key TEXT        `osascript keystroke TEXT`
#   shot NAME       full-desktop screenshot (the game is the 1920x1080
#                   desktop, so a region grab would be the same picture)
#   sleep S         pause S seconds
#   stop            end the driver
#
# It ends on `stop`, when the game process is gone, or at HAND_MAX seconds
# (default 3600). What a click did is read afterwards from the dump
# (`COMMANDMANAGER=1`'s `process_*` line, `UNITS=3`'s order) and the trace,
# never from the screenshot alone.
RON_TOOLS=${0:A:h}
N=${1:?run number}
T=${2:-/tmp/ron-runs}
D=${HAND_DIR:-$T/hand-$N}
L=${RON_LOGS:-"$HOME/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"}
mkdir -p "$D"
touch "$D/cmd" "$D/done.log"

frame_now() {
  tail -c 2000000 "$L/gamelog.txt" 2>/dev/null \
    | grep -a -o 'BEGIN FRAME [0-9]*' | tail -1 | awk '{print $3}'
}

game_alive() { pgrep -f 'riseofnations_trace\.exe' >/dev/null 2>&1 }

move() {
  cliclick m:$(($1 + 40)),$(($2 + 30))
  sleep 0.15
  cliclick m:$1,$2
  sleep 0.35
}

echo "driver: reading $D/cmd"
n=0
deadline=$((SECONDS + ${HAND_MAX:-3600}))
while (( SECONDS < deadline )); do
  total=$(wc -l < "$D/cmd" | tr -d ' ')
  while (( n < total )); do
    n=$((n + 1))
    line=$(sed -n "${n}p" "$D/cmd")
    words=(${=line})
    rc=0
    case ${words[1]} in
      wait)
        while game_alive || (( SECONDS < 120 )); do
          f=$(frame_now); (( ${f:-0} >= ${words[2]} )) && break; sleep 0.2
        done ;;
      focus) zsh "$RON_TOOLS/focus.sh" >/dev/null 2>&1 || rc=$? ;;
      m) move ${words[2]} ${words[3]} ;;
      c) move ${words[2]} ${words[3]}; cliclick c:${words[2]},${words[3]} || rc=$? ;;
      rc) move ${words[2]} ${words[3]}; cliclick rc:${words[2]},${words[3]} || rc=$? ;;
      sc) move ${words[2]} ${words[3]}; cliclick kd:shift c:${words[2]},${words[3]} ku:shift || rc=$? ;;
      key) osascript -e "tell application \"System Events\" to keystroke \"${words[2]}\"" >/dev/null 2>&1 || rc=$? ;;
      shot)
        screencapture -x "$D/${words[2]}.png" || rc=$?
        sips -Z 1400 "$D/${words[2]}.png" --out "$D/${words[2]}s.png" >/dev/null 2>&1 ;;
      sleep) sleep ${words[2]} ;;
      stop) echo "$(date +%H:%M:%S) frame $(frame_now) line $n: $line rc=0" >> "$D/done.log"; exit 0 ;;
      ""|\#*) ;;
      *) rc=64 ;;
    esac
    echo "$(date +%H:%M:%S) frame $(frame_now) line $n: $line rc=$rc" >> "$D/done.log"
  done
  game_alive || { (( SECONDS > 120 )) && break; }
  sleep 0.3
done
echo "driver: done after $n lines"
