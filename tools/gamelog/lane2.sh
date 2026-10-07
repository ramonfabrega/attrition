#!/bin/zsh
# lane2.sh — build the second click-free capture lane (parked 1139, item
# 1567): a prefix of its own, a copy of the install and a profile, each
# where `lanes.sh` says lane 2's are. Idempotent: what exists is left as it
# is, so a rerun only finishes what an earlier one did not.
#
#   zsh tools/gamelog/lane2.sh
#   RON_CAPTURE_LANE=2 zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh <out> …
#
# 1. **The prefix**, `~/wine-ron-2`: `wineboot -i` with Mono and Gecko
#    declined (the game needs neither, and each would put up an installer
#    dialog nobody answers), then `prefix.sh` — DXVK-macOS's three DLLs,
#    `ShowCrashDialog=0` and `C:\users\crossover` (`docs/ORACLE.md`, "Off
#    CrossOver", says why each is there).
# 2. **The install**, `~/ron-capture-lane-2/game`: lane 1's, cloned with
#    `cp -c`, which on APFS shares the blocks — 2.8 GB of names for the
#    cost of the directory entries — and becomes a copy only where a run
#    writes. A run's directory links the install's data, so two lanes on
#    one install would share whatever the game writes there.
# 3. **The profile**, `~/ron-capture-lane-2/AppData/Roaming/Microsoft
#    Games`: lane 1's INIs and `PlayerProfile` as they stand restored, and
#    an empty `Logs`; the prefix's `AppData\Roaming\Microsoft Games` is a
#    link to it, as lane 1's is to `~/ron-data`. Copied only while lane 1
#    is free, so a capture's staged INIs are never what lane 2 starts from.
#
# Nothing here enters the repo; nothing of lane 1 is written.
set -e
T=${0:A:h}
export RON_CAPTURE_LANE=2
source "$T/lanes.sh"
P=$RON_LANE_PREFIX
W=${RON_WINE_BIN:-/Applications/Wine Stable.app/Contents/Resources/wine/bin/wine}
LANE1_INSTALL=/Users/rf-studio/code/fun/attrition/game
LANE1_GAMES="$HOME/ron-data/AppData/Roaming/Microsoft Games"
GAMES=${RON_LANE_PROFILE:h}

if [ ! -f "$P/system.reg" ]; then
  echo "wineboot $P"
  WINEPREFIX="$P" WINEDEBUG=-all WINEDLLOVERRIDES="mscoree,mshtml=" "$W" wineboot -i
  WINEPREFIX="$P" "${W:h}/wineserver" -w
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
  state=$(RON_CAPTURE_LANE=1 zsh -c "source '$T/winelaunch.sh'; ron_lane_state") || {
    echo "lane 1 is $state; its profile is staged — rerun when it is free" >&2; exit 75 }
  echo "copying lane 1's profile -> $GAMES"
  mkdir -p "$RON_LANE_PROFILE/Logs"
  for f in "$LANE1_GAMES/Rise of Nations"/*(N) "$LANE1_GAMES/Rise of Nations"/.*(N); do
    case ${f:t} in
      Logs|StateBackup|.attrition-capture.lock|.DS_Store) ;;
      *) cp -Rp "$f" "$RON_LANE_PROFILE/" ;;
    esac
  done
  for f in "$LANE1_GAMES"/*(N); do
    [ "${f:t}" = "Rise of Nations" ] || cp -Rp "$f" "$GAMES/"
  done
else
  echo "profile already there: $RON_LANE_PROFILE"
fi

roaming="$P/drive_c/users/$USER/AppData/Roaming"
if [ ! -L "$roaming/Microsoft Games" ]; then
  [ -e "$roaming/Microsoft Games" ] && { echo "$roaming/Microsoft Games is not a link; leaving it" >&2; exit 1 }
  mkdir -p "$roaming"
  ln -s "$GAMES" "$roaming/Microsoft Games"
  echo "linked Microsoft Games -> $GAMES"
else
  echo "Microsoft Games already linked -> $(readlink "$roaming/Microsoft Games")"
fi
echo "lane 2: prefix $P, install $RON_LANE_INSTALL, profile $RON_LANE_PROFILE"
