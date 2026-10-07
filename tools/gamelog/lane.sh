#!/bin/zsh
# lane.sh N — build click-free capture lane N (N ≥ 2) and admit it to the
# pool (parked 1139; items 1568, 1569): a prefix of its own, a copy of the
# install, a profile, and a reproduction run that must match run676's
# trace before any capture is staged on it. Idempotent: what exists is left
# as it is, so a rerun only finishes what an earlier one did not.
#
#   zsh tools/gamelog/viadriver.sh tools/gamelog/lane.sh 3
#
# The admission run is a game, so it needs a GUI session: through
# `viadriver.sh`, or from a pool runner (which is already under RonDriver).
# `unattended_capture.py`'s pool calls this when no lane is free and fewer
# than `RON_LANES_MAX` exist.
#
# 1. **The prefix**, `~/wine-ron-N`: `wineboot -i` with Mono and Gecko
#    declined (the game needs neither, and each would put up an installer
#    dialog nobody answers), then `prefix.sh` — DXVK-macOS's three DLLs,
#    `ShowCrashDialog=0` and `C:\users\crossover` (`docs/ORACLE.md`, "Off
#    CrossOver", says why each is there).
# 2. **The install**, `~/ron-capture-lane-N/game`: lane 1's, cloned with
#    `cp -c`, which on APFS shares the blocks — 2.8 GB of names for the
#    cost of the directory entries.
# 3. **The profile**: the pool's template, `~/ron-capture-lanes/template`,
#    which is lane 1's profile copied once while lane 1 was free — a lane
#    is grown exactly when every lane is held, and a held lane's INIs are
#    staged. The prefix's `AppData\Roaming\Microsoft Games` links to it.
# 4. **The admission**: run676's recipe on lane N (`RON_CAPTURE_LANE=N`,
#    the runner's `-` install and profile), then `rngcmp.py` against
#    run676's archived trace; 4341 frames in common and 0 differing writes
#    `admitted`, naming the run, the archive and the counts, so a reader
#    can run the check again; anything else writes `admission-failed`, and
#    the pool never tries the lane again until a person removes it.
#    `RON_ADMISSION_RUN=runNNN` names the run in the file and the output.
#
# Nothing here enters the repo; nothing of lane 1 is written.
set -e
T=${0:A:h}
W=${T:h:h}
N=${1:?usage: lane.sh N (N ≥ 2)}
(( N >= 2 )) || { echo "lane.sh: lane 1 is ~/wine-ron, the reference; N ≥ 2" >&2; exit 64 }
export RON_CAPTURE_LANE=$N
source "$T/lanes.sh" || exit 64
P=$RON_LANE_PREFIX
WINE=${RON_WINE_BIN:-/Applications/Wine Stable.app/Contents/Resources/wine/bin/wine}
LANE1_INSTALL=/Users/rf-studio/code/fun/attrition/game
LANE1_GAMES="$RON_LANES_ROOT/ron-data/AppData/Roaming/Microsoft Games"
TEMPLATE=$RON_LANES_ROOT/ron-capture-lanes/template
GAMES=${RON_LANE_PROFILE:h}
REFERENCE=${RON_ADMISSION_REFERENCE:-$LANE1_GAMES/Rise of Nations/Logs/rontrace-run676.log}

if [ ! -d "$TEMPLATE/Rise of Nations" ]; then
  state=$(RON_CAPTURE_LANE=1 zsh -c "source '$T/winelaunch.sh'; ron_lane_state") || {
    echo "lane 1 is $state; its profile is staged — the template is taken while it is free" >&2; exit 75 }
  echo "template: lane 1's profile -> $TEMPLATE"
  mkdir -p "$TEMPLATE/Rise of Nations/Logs"
  for f in "$LANE1_GAMES/Rise of Nations"/*(N) "$LANE1_GAMES/Rise of Nations"/.*(N); do
    case ${f:t} in
      Logs|StateBackup|.attrition-capture.lock|.DS_Store) ;;
      *) cp -Rp "$f" "$TEMPLATE/Rise of Nations/" ;;
    esac
  done
  for f in "$LANE1_GAMES"/*(N); do
    [ "${f:t}" = "Rise of Nations" ] || cp -Rp "$f" "$TEMPLATE/"
  done
fi

if [ ! -f "$P/system.reg" ]; then
  echo "wineboot $P"
  WINEPREFIX="$P" WINEDEBUG=-all WINEDLLOVERRIDES="mscoree,mshtml=" "$WINE" wineboot -i
  WINEPREFIX="$P" "${WINE:h}/wineserver" -w
fi
RON_WINEPREFIX="$P" zsh "$T/prefix.sh"

if [ ! -d "$RON_LANE_INSTALL" ]; then
  echo "cloning $LANE1_INSTALL -> $RON_LANE_INSTALL"
  mkdir -p "${RON_LANE_INSTALL:h}"
  cp -Rc "$LANE1_INSTALL" "$RON_LANE_INSTALL"
else
  echo "install already there: $RON_LANE_INSTALL"
fi

if [ ! -d "$RON_LANE_PROFILE" ]; then
  echo "profile: the template -> $GAMES"
  mkdir -p "$GAMES"
  cp -Rp "$TEMPLATE"/. "$GAMES/"
else
  echo "profile already there: $RON_LANE_PROFILE"
fi

roaming="$P/drive_c/users/$USER/AppData/Roaming"
if [ ! -L "$roaming/Microsoft Games" ]; then
  [ -e "$roaming/Microsoft Games" ] && { echo "$roaming/Microsoft Games is not a link; leaving it" >&2; exit 1 }
  mkdir -p "$roaming"
  ln -s "$GAMES" "$roaming/Microsoft Games"
  echo "linked Microsoft Games -> $GAMES"
fi

if [ -f "$RON_LANE_HOME/admitted" ]; then
  echo "lane $N already admitted: $(cat "$RON_LANE_HOME/admitted")"
  exit 0
fi
[ -f "$REFERENCE" ] || { echo "no reference trace $REFERENCE" >&2; exit 66 }
run=${RON_ADMISSION_RUN:-admission}
out=$RON_LANE_HOME/$run-$(date +%Y%m%d-%H%M%S)
echo "admission: run676's recipe on lane $N -> $out"
set +e
(cd "$W/tools/explore" && python3 unattended_capture.py - "$out" - \
  --map 7 --end-frame 24000 --timeout 2400 --log-window 0 24001 --ffwd-minute 27 \
  --cover cover=0 --callwin 0 24000 --ai-tribe 23 --profile STARTING_TECHNOLOGY=8 \
  --profile STARTING_RESOURCES=7 --profile DIFFICULTY=5 --detail end:MISC \
  --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
  --detail misc:CHECKSUM=2 \
  --detail endgame:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,DEATHS=1 --allow-early-end)
rc=$?
cmp=$(python3 "$T/rngcmp.py" "$REFERENCE" "$out/map-7/rontrace.log" 2>&1)
set -e
common=$(print -r -- "$cmp" | sed -n 's/^frames in common: //p')
differing=$(print -r -- "$cmp" | sed -n 's/^differing frames: \([0-9]*\).*/\1/p')
line="$run on $(date '+%Y-%m-%dT%H:%M:%S%z'), $out: runner exit $rc; rngcmp.py $REFERENCE $out/map-7/rontrace.log: ${common:-?} frames in common, ${differing:-?} differing"
if [[ $rc == 0 && $common == 4341 && $differing == 0 ]]; then
  print -r -- "$line" > "$RON_LANE_HOME/admitted"
  echo "admitted: $line"
else
  print -r -- "$line" > "$RON_LANE_HOME/admission-failed"
  echo "admission failed: $line" >&2
  exit 1
fi
